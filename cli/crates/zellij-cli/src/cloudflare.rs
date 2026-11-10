//! Native Cloudflare Pages Direct Upload for generated static exports.
//! Protocol: Cloudflare API pages/assets and project deployments; hash format
//! follows cloudflare/workers-sdk packages/deploy-helpers/.../helpers/hash.ts.
use base64::{Engine as _, engine::general_purpose::STANDARD};
use camino::{Utf8Path, Utf8PathBuf};
use reqwest::{
    Method, Url,
    blocking::{Client as HttpClient, RequestBuilder, multipart},
    header,
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    io::{Read, Write},
    time::{Duration, Instant},
};
use zellij_core::{Error, Result, delivery::Provider};

const API: &str = "https://api.cloudflare.com/client/v4/";
const MAX_FILE: u64 = 25 * 1024 * 1024;
const MAX_FILES: usize = 20_000;
const RESPONSE_LIMIT: u64 = 8 * 1024 * 1024;
const BATCH_BYTES: usize = 8 * 1024 * 1024;
fn failed(message: impl Into<String>) -> Error {
    Error::DeploymentFailed(message.into())
}
fn credential(value: &str, name: &str) -> Result<header::HeaderValue> {
    if value.is_empty() || !value.bytes().all(|b| b.is_ascii_graphic()) {
        return Err(Error::InvalidArgument(format!(
            "{name} must be a nonempty token without whitespace"
        )));
    }
    let mut header = header::HeaderValue::from_str(&format!("Bearer {value}"))
        .map_err(|_| failed(format!("invalid {name}")))?;
    header.set_sensitive(true);
    Ok(header)
}
pub(crate) struct Client {
    http: HttpClient,
    base: Url,
    token: header::HeaderValue,
    account: String,
    timeout: Duration,
    poll_interval: Duration,
}
#[derive(Deserialize)]
struct Envelope<T> {
    success: bool,
    result: T,
}
#[derive(Deserialize)]
struct Project {
    name: String,
    production_branch: String,
}
pub(crate) struct Target {
    project: String,
    branch: String,
    production: bool,
}
#[derive(Deserialize)]
struct Stage {
    name: String,
    status: String,
}
#[derive(Deserialize)]
struct Deployment {
    id: String,
    environment: String,
    latest_stage: Stage,
    url: String,
    #[serde(default)]
    is_skipped: bool,
}
pub(crate) struct Deployed {
    pub id: String,
    url: Url,
}
impl Deployed {
    pub fn url(&self, prefix: &str) -> Url {
        let mut url = self.url.clone();
        url.set_path(&format!("{}/", prefix.trim_end_matches('/')));
        url
    }
}
impl Client {
    pub fn from_environment() -> Result<Self> {
        let token = std::env::var("CLOUDFLARE_API_TOKEN").map_err(|_| Error::InvalidArgument("set CLOUDFLARE_API_TOKEN to an API token with Cloudflare Pages Edit access; zel does not run login or read Wrangler credentials".into()))?;
        let account = std::env::var("CLOUDFLARE_ACCOUNT_ID").map_err(|_| {
            Error::InvalidArgument(
                "set CLOUDFLARE_ACCOUNT_ID to the account containing the existing Pages project"
                    .into(),
            )
        })?;
        Self::new(
            API,
            &token,
            &account,
            Duration::from_secs(300),
            Duration::from_secs(2),
        )
    }
    // Endpoint substitution is private, solely for loopback protocol tests.
    fn new(
        base: &str,
        token: &str,
        account: &str,
        timeout: Duration,
        poll_interval: Duration,
    ) -> Result<Self> {
        let token = credential(token, "CLOUDFLARE_API_TOKEN")?;
        if account.len() != 32 || !account.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err(Error::InvalidArgument(
                "CLOUDFLARE_ACCOUNT_ID must be a 32-character hexadecimal account ID".into(),
            ));
        }
        let http = HttpClient::builder()
            .redirect(reqwest::redirect::Policy::none())
            .connect_timeout(Duration::from_secs(15))
            .timeout(Duration::from_secs(60))
            .user_agent(concat!("zel/", env!("CARGO_PKG_VERSION")))
            .build()
            .map_err(|_| failed("could not initialize the HTTPS client"))?;
        Ok(Self {
            http,
            base: Url::parse(base).map_err(|_| failed("invalid API origin"))?,
            token,
            account: account.into(),
            timeout,
            poll_interval,
        })
    }
    fn request(&self, method: Method, parts: &[&str]) -> RequestBuilder {
        self.request_with_token(method, parts, self.token.clone())
    }
    fn request_with_token(
        &self,
        method: Method,
        parts: &[&str],
        token: header::HeaderValue,
    ) -> RequestBuilder {
        let mut url = self.base.clone();
        url.path_segments_mut()
            .expect("hierarchical API")
            .pop_if_empty()
            .extend(parts.iter().copied());
        self.http
            .request(method, url)
            .header(header::AUTHORIZATION, token)
    }
    fn project_request(&self, method: Method, project: &str, suffix: &[&str]) -> RequestBuilder {
        let mut parts = vec!["accounts", &self.account, "pages", "projects", project];
        parts.extend_from_slice(suffix);
        self.request(method, &parts)
    }
    fn json<T: serde::de::DeserializeOwned>(
        &self,
        request: RequestBuilder,
        stage: &str,
        deadline: Instant,
    ) -> Result<T> {
        let remaining = deadline
            .checked_duration_since(Instant::now())
            .ok_or_else(|| {
                failed("Cloudflare deployment timed out; check the dashboard before retrying")
            })?;
        let response = request
            .timeout(remaining.min(Duration::from_secs(60)))
            .send()
            .map_err(|e| {
                failed(format!(
                    "{stage}: {}; check the Cloudflare dashboard before retrying",
                    if e.is_timeout() {
                        "request timed out"
                    } else {
                        "HTTPS request failed"
                    }
                ))
            })?;
        let status = response.status();
        if !status.is_success() {
            let hint = match status.as_u16() {
                401 | 403 => "check CLOUDFLARE_API_TOKEN and Pages Edit access to the account",
                404 => "check CLOUDFLARE_ACCOUNT_ID and the existing project name",
                429 => "Cloudflare rate limit reached; wait before retrying",
                300..=399 => "redirect refused; credentials were not forwarded",
                _ => "check the Cloudflare dashboard before retrying",
            };
            return Err(failed(format!(
                "{stage}: HTTP {} — {hint}",
                status.as_u16()
            )));
        }
        let mut bytes = Vec::new();
        response
            .take(RESPONSE_LIMIT + 1)
            .read_to_end(&mut bytes)
            .map_err(|_| failed(format!("{stage}: could not read response")))?;
        if bytes.len() as u64 > RESPONSE_LIMIT {
            return Err(failed(format!("{stage}: response exceeded 8 MiB")));
        }
        // Never expose API bodies, client debug output or credentials in errors.
        let result: Envelope<T> = serde_json::from_slice(&bytes)
            .map_err(|_| failed(format!("{stage}: invalid API response")))?;
        if !result.success {
            return Err(failed(format!(
                "{stage}: Cloudflare rejected the request; check project settings and token permissions in the dashboard"
            )));
        }
        Ok(result.result)
    }
    pub fn check_project(&self, project: &str, production: bool) -> Result<Target> {
        Provider::Cloudflare.validate_site(project)?;
        let result: Project = self.json(
            self.project_request(Method::GET, project, &[]),
            "project preflight",
            Instant::now() + self.timeout,
        )?;
        if result.name != project
            || result.production_branch.is_empty()
            || result.production_branch.len() > 256
            || result.production_branch.chars().any(char::is_control)
        {
            return Err(failed(
                "project preflight: invalid project identity or production branch",
            ));
        }
        // Never infer intent from the local Git branch. The fixed preview name
        // is checked against the project before any build, upload or creation.
        if !production && result.production_branch == "zel-preview" {
            return Err(failed(
                "project preflight: zel-preview is this project's production branch; change the project's production branch before using preview deployment",
            ));
        }
        Ok(Target {
            project: project.into(),
            branch: if production {
                result.production_branch
            } else {
                "zel-preview".into()
            },
            production,
        })
    }
    pub fn deploy(&self, target: &Target, export: &Utf8Path) -> Result<Deployed> {
        let files = Manifest::read(export)?;
        let deadline = Instant::now() + self.timeout;
        #[derive(Deserialize)]
        struct UploadToken {
            jwt: String,
        }
        let result: UploadToken = self.json(
            self.project_request(Method::GET, &target.project, &["upload-token"]),
            "upload authorization",
            deadline,
        )?;
        let jwt = credential(&result.jwt, "Cloudflare upload token")?;
        let hashes: Vec<_> = files.by_hash.keys().collect();
        let asset_request = |endpoint| {
            self.request_with_token(Method::POST, &["pages", "assets", endpoint], jwt.clone())
        };
        let missing: Vec<String> = self.json(
            asset_request("check-missing").json(&json!({"hashes": hashes})),
            "asset lookup",
            deadline,
        )?;
        // Reject an unrecognized hash before uploading any data.
        let mut uploads = BTreeMap::new();
        for hash in missing {
            let asset = files.by_hash.get(&hash).ok_or_else(|| {
                failed("asset lookup requested a hash outside the generated manifest")
            })?;
            uploads.insert(hash, asset);
        }
        let mut batch = Vec::new();
        let mut bytes = 0;
        for (hash, asset) in uploads {
            let data = read_asset(&asset.path)?;
            if hash_file(&asset.path, &data) != hash {
                return Err(failed(
                    "generated asset changed during deployment; rebuild and retry",
                ));
            }
            let encoded = STANDARD.encode(&data);
            if !batch.is_empty() && (bytes + encoded.len() > BATCH_BYTES || batch.len() == 1000) {
                let _: Option<Value> = self.json(
                    asset_request("upload").json(&batch),
                    "asset upload",
                    deadline,
                )?;
                batch.clear();
                bytes = 0;
            }
            bytes += encoded.len();
            batch.push(json!({"key":hash,"value":encoded,"base64":true,"metadata":{"contentType":mime_guess::from_path(&asset.path).first_or_octet_stream().as_ref()}}));
        }
        if !batch.is_empty() {
            let _: Option<Value> = self.json(
                asset_request("upload").json(&batch),
                "asset upload",
                deadline,
            )?;
        }
        // Acknowledgements may omit `result` (the live cache API does so).
        // Data-bearing endpoints above and below still require their payload.
        let _: Option<Value> = self.json(
            asset_request("upsert-hashes").json(&json!({"hashes": hashes})),
            "asset cache update",
            deadline,
        )?;
        let form = multipart::Form::new()
            .text("branch", target.branch.clone())
            .text(
                "manifest",
                serde_json::to_string(&files.hashes).expect("string map"),
            );
        // Do not retry this POST: an uncertain response may already have
        // published a deployment. Creation is the only production mutation.
        let mut deployed: Deployment = self.json(
            self.project_request(Method::POST, &target.project, &["deployments"])
                .multipart(form),
            "create deployment",
            deadline,
        )?;
        let id = deployed.id.clone();
        if id.is_empty()
            || id.len() > 128
            || !id.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')
        {
            return Err(failed("invalid deployment ID in response"));
        }
        zellij_core::statusln!("  Deploy ID {id}");
        loop {
            if deployed.is_skipped {
                return Err(failed(
                    "Cloudflare skipped this deployment; inspect the project's deployment settings in the dashboard",
                ));
            }
            if deployed.id != id
                || deployed.environment
                    != if target.production {
                        "production"
                    } else {
                        "preview"
                    }
            {
                return Err(failed(
                    "deployment returned an unexpected ID or environment; inspect the Cloudflare dashboard",
                ));
            }
            match deployed.latest_stage.status.as_str() {
                "failure" | "canceled" | "cancelled" => {
                    return Err(failed(
                        "Cloudflare reported a failed/canceled deployment; inspect the dashboard",
                    ));
                }
                "success" if deployed.latest_stage.name == "deploy" => {
                    let url = Url::parse(&deployed.url).ok().filter(|u| u.scheme() == "https" && u.host_str().is_some_and(|host| host.ends_with(".pages.dev")) && u.username().is_empty() && u.password().is_none() && u.port().is_none() && u.path() == "/" && u.query().is_none() && u.fragment().is_none())
                        .ok_or_else(|| failed("deployment is ready, but its HTTPS pages.dev URL is invalid; find it in the dashboard"))?;
                    return Ok(Deployed { id, url });
                }
                "idle" | "active" | "success" => {}
                _ => {
                    return Err(failed(
                        "unexpected deployment state; inspect the Cloudflare dashboard",
                    ));
                }
            }
            let remaining = deadline
                .checked_duration_since(Instant::now())
                .ok_or_else(|| failed("deployment timed out; it may still complete remotely"))?;
            std::thread::sleep(self.poll_interval.min(remaining));
            deployed = self.json(
                self.project_request(Method::GET, &target.project, &["deployments", &id]),
                "deployment status",
                deadline,
            )?;
        }
    }
}
// Cloudflare treats exports without a root 404.html as single-page apps and
// returns index.html with HTTP 200 for missing pages. Only call this on the
// private export staging directory; preserve generated/authored 404 pages.
pub(crate) fn ensure_not_found_page(export: &Utf8Path) -> Result<()> {
    let path = export.join("404.html");
    match std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
    {
        Ok(mut file) => {
            file.write_all(b"<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width, initial-scale=1\"><title>Page not found</title></head><body><h1>Page not found</h1><p>The requested page does not exist.</p></body></html>\n")?;
            Ok(())
        }
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
            let metadata = std::fs::symlink_metadata(&path)?;
            if zellij_core::site::is_link(&metadata) || !metadata.is_file() {
                return Err(failed("Cloudflare 404.html must be a regular file"));
            }
            Ok(())
        }
        Err(error) => Err(error.into()),
    }
}

struct Asset {
    path: Utf8PathBuf,
}
struct Manifest {
    hashes: BTreeMap<String, String>,
    by_hash: BTreeMap<String, Asset>,
}
fn read_asset(path: &Utf8Path) -> Result<Vec<u8>> {
    let meta = std::fs::symlink_metadata(path)?;
    if zellij_core::site::is_link(&meta) || !meta.is_file() {
        return Err(failed(
            "export contains a symlink, junction or non-regular file",
        ));
    }
    if meta.len() > MAX_FILE {
        return Err(failed(
            "Cloudflare Pages assets must be at most 25 MiB each",
        ));
    }
    let mut bytes = Vec::new();
    std::fs::File::open(path)?
        .take(MAX_FILE + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() as u64 > MAX_FILE {
        return Err(failed(
            "Cloudflare Pages assets must be at most 25 MiB each",
        ));
    }
    Ok(bytes)
}
fn hash_file(path: &Utf8Path, bytes: &[u8]) -> String {
    let mut digest = blake3::Hasher::new();
    digest.update(STANDARD.encode(bytes).as_bytes());
    digest.update(path.extension().unwrap_or("").as_bytes());
    digest.finalize().to_hex()[..32].to_owned()
}
impl Manifest {
    fn read(root: &Utf8Path) -> Result<Self> {
        let mut hashes = BTreeMap::new();
        let mut by_hash = BTreeMap::new();
        for entry in walkdir::WalkDir::new(root)
            .follow_links(false)
            .sort_by_file_name()
        {
            let entry = entry.map_err(|_| failed("could not enumerate generated export"))?;
            let meta = std::fs::symlink_metadata(entry.path())?;
            if zellij_core::site::is_link(&meta) {
                return Err(failed("export contains a symlink or junction"));
            }
            let path = Utf8Path::from_path(entry.path())
                .ok_or_else(|| failed("export path is not UTF-8"))?;
            let relative = path
                .strip_prefix(root)
                .map_err(|_| failed("file outside export"))?;
            // A static export must not silently activate/ignore platform code
            // or rules. Custom provider configuration is a separate feature.
            if relative.components().next().is_some_and(|c| {
                matches!(
                    c.as_str(),
                    "_worker.js"
                        | "_worker.bundle"
                        | "_routes.json"
                        | "_headers"
                        | "_redirects"
                        | "functions"
                )
            }) {
                return Err(failed(
                    "Cloudflare deployment supports static assets only; remove Workers, functions and custom _headers/_redirects/_routes.json from the export",
                ));
            }
            if meta.is_dir() {
                continue;
            }
            if relative == ".zellij-output" {
                continue;
            }
            let name = format!(
                "/{}",
                relative
                    .components()
                    .map(|c| c.as_str())
                    .collect::<Vec<_>>()
                    .join("/")
            );
            if name
                .chars()
                .any(|c| c.is_control() || matches!(c, '#' | '?' | '\\'))
            {
                return Err(failed(
                    "Cloudflare asset paths cannot contain #, ?, backslashes or control characters",
                ));
            }
            let bytes = read_asset(path)?;
            let hash = hash_file(path, &bytes);
            hashes.insert(name, hash.clone());
            by_hash.entry(hash).or_insert(Asset {
                path: path.to_owned(),
            });
            if hashes.len() > MAX_FILES {
                return Err(failed(
                    "zel Cloudflare deployments support at most 20,000 files",
                ));
            }
        }
        if !hashes.keys().any(|name| name.ends_with("/index.html")) {
            return Err(failed(
                "generated export has no index.html; no deployment was created",
            ));
        }
        Ok(Self { hashes, by_hash })
    }
}
#[cfg(test)]
#[path = "cloudflare_tests.rs"]
mod tests;
