//! Native Vercel prebuilt static deployment. The prebuilt query flag and
//! .vercel/output layout follow vercel/vercel's official client implementation.
use camino::{Utf8Path, Utf8PathBuf};
use reqwest::{
    Method, Url,
    blocking::{Client as HttpClient, RequestBuilder},
    header,
};
use serde::Deserialize;
use serde_json::{Value, json};
use sha1::{Digest, Sha1};
use std::{
    collections::BTreeMap,
    io::Read,
    time::{Duration, Instant},
};
use zellij_core::{Error, Result, delivery::Provider};

const API: &str = "https://api.vercel.com/";
// Deliberately bounded initial adapter; provider plan limits may be lower.
const MAX_FILE: u64 = 100 * 1024 * 1024;
const MAX_TOTAL: u64 = 100 * 1024 * 1024;
const MAX_FILES: usize = 15_000;
const RESPONSE_LIMIT: u64 = 8 * 1024 * 1024;
fn failed(message: impl Into<String>) -> Error {
    Error::DeploymentFailed(message.into())
}
fn identifier(value: &str, prefix: &str) -> bool {
    value.len() <= 128
        && value
            .strip_prefix(prefix)
            .is_some_and(|v| !v.is_empty() && v.bytes().all(|b| b.is_ascii_alphanumeric()))
}
pub(crate) struct Client {
    http: HttpClient,
    base: Url,
    token: header::HeaderValue,
    team: Option<String>,
    timeout: Duration,
    poll_interval: Duration,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Project {
    id: String,
    name: String,
    account_id: String,
}
pub(crate) struct Target {
    project: Project,
    production: bool,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Created {
    id: String,
    ready_state: String,
    target: Option<String>,
    error_code: Option<String>,
    alias_error: Option<Value>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Deployment {
    #[serde(flatten)]
    created: Created,
    project_id: String,
    url: String,
    alias_assigned: Option<Value>,
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
        let token = std::env::var("VERCEL_TOKEN").map_err(|_| Error::InvalidArgument("set VERCEL_TOKEN to a token with access to the existing project; zel does not run login or read Vercel CLI credentials".into()))?;
        let team = std::env::var("VERCEL_TEAM_ID")
            .ok()
            .filter(|v| !v.is_empty());
        Self::new(
            API,
            &token,
            team.as_deref(),
            Duration::from_secs(300),
            Duration::from_secs(2),
        )
    }
    // API origin substitution is private and only used by loopback tests.
    fn new(
        base: &str,
        token: &str,
        team: Option<&str>,
        timeout: Duration,
        poll_interval: Duration,
    ) -> Result<Self> {
        if token.is_empty() || !token.bytes().all(|b| b.is_ascii_graphic()) {
            return Err(Error::InvalidArgument(
                "VERCEL_TOKEN must be a nonempty token without whitespace".into(),
            ));
        }
        let mut token = header::HeaderValue::from_str(&format!("Bearer {token}"))
            .map_err(|_| failed("invalid VERCEL_TOKEN"))?;
        token.set_sensitive(true);
        if team.is_some_and(|t| !identifier(t, "team_")) {
            return Err(Error::InvalidArgument("VERCEL_TEAM_ID must be a team_… identifier, not a name or URL; omit it for personal scope".into()));
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
            team: team.map(str::to_owned),
            timeout,
            poll_interval,
        })
    }
    fn request(&self, method: Method, parts: &[&str]) -> RequestBuilder {
        let mut url = self.base.clone();
        url.path_segments_mut()
            .expect("hierarchical API")
            .pop_if_empty()
            .extend(parts.iter().copied());
        if let Some(team) = &self.team {
            url.query_pairs_mut().append_pair("teamId", team);
        }
        self.http
            .request(method, url)
            .header(header::AUTHORIZATION, self.token.clone())
    }
    fn send(&self, request: RequestBuilder, stage: &str, deadline: Instant) -> Result<Vec<u8>> {
        let remaining = deadline
            .checked_duration_since(Instant::now())
            .ok_or_else(|| {
                failed("Vercel deployment timed out; check the dashboard before retrying")
            })?;
        let response = request
            .timeout(remaining.min(Duration::from_secs(60)))
            .send()
            .map_err(|e| {
                failed(format!(
                    "{stage}: {}; check the Vercel dashboard before retrying",
                    if e.is_timeout() {
                        "request timed out"
                    } else {
                        "HTTPS request failed"
                    }
                ))
            })?;
        if !response.status().is_success() {
            let status = response.status().as_u16();
            let hint = match status {
                401 | 403 => "check VERCEL_TOKEN and its access to the project/team",
                404 => {
                    "check VERCEL_PROJECT_ID and VERCEL_TEAM_ID; an existing project is required"
                }
                429 => "Vercel rate limit reached; wait before retrying",
                300..=399 => "redirect refused; credentials were not forwarded",
                _ => "check Vercel project settings and deployment logs before retrying",
            };
            return Err(failed(format!("{stage}: HTTP {status} — {hint}")));
        }
        let mut bytes = Vec::new();
        response
            .take(RESPONSE_LIMIT + 1)
            .read_to_end(&mut bytes)
            .map_err(|_| failed(format!("{stage}: could not read response")))?;
        if bytes.len() as u64 > RESPONSE_LIMIT {
            return Err(failed(format!("{stage}: response exceeded 8 MiB")));
        }
        Ok(bytes)
    }
    fn json<T: serde::de::DeserializeOwned>(
        &self,
        request: RequestBuilder,
        stage: &str,
        deadline: Instant,
    ) -> Result<T> {
        let bytes = self.send(request, stage, deadline)?;
        // Do not expose response bodies (which can echo credentials/user data).
        let value: Value = serde_json::from_slice(&bytes)
            .map_err(|_| failed(format!("{stage}: invalid API response")))?;
        if value.get("error").is_some_and(|e| !e.is_null()) {
            return Err(failed(format!(
                "{stage}: Vercel rejected the request; inspect the dashboard"
            )));
        }
        serde_json::from_value(value).map_err(|_| failed(format!("{stage}: invalid API response")))
    }
    pub fn check_project(&self, name: &str, production: bool) -> Result<Target> {
        Provider::Vercel.validate_site(name)?;
        let project: Project = self.json(
            self.request(Method::GET, &["v9", "projects", name]),
            "project preflight",
            Instant::now() + self.timeout,
        )?;
        if !identifier(&project.id, "prj_")
            || (project.id != name && project.name != name)
            || Provider::Vercel.validate_site(&project.name).is_err()
            || project.account_id.is_empty()
            || self.team.as_ref().is_some_and(|t| t != &project.account_id)
        {
            return Err(failed(
                "project preflight: unexpected project identity or team",
            ));
        }
        Ok(Target {
            project,
            production,
        })
    }
    pub fn deploy(&self, target: &Target, export: &Utf8Path) -> Result<Deployed> {
        let manifest = Manifest::read(export)?;
        let deadline = Instant::now() + self.timeout;
        // Content-addressed assets are uploaded once per distinct SHA. Always
        // upload before creating: a creation POST is never retried implicitly.
        for (sha, asset) in &manifest.assets {
            let bytes = read_asset(&asset.path)?;
            if bytes.len() as u64 != asset.size || hash(&bytes) != *sha {
                return Err(failed(
                    "generated export changed during upload; no deployment was created",
                ));
            }
            let _: Value = self.json(
                self.request(Method::POST, &["v2", "files"])
                    .header("x-vercel-digest", sha)
                    .header(header::CONTENT_TYPE, "application/octet-stream")
                    .body(bytes),
                "asset upload",
                deadline,
            )?;
        }
        let mut files = manifest.files;
        files.push(json!({"file":".vercel/output/config.json", "data": output_config().to_string(), "encoding":"utf-8"}));
        let mut body = json!({"name": target.project.name, "project":target.project.id, "version":2, "files":files});
        if target.production {
            body["target"] = json!("production");
        }
        // The official client sends prebuilt=1, not a body `prebuilt` field.
        let created: Created = self.json(
            self.request(Method::POST, &["v13", "deployments"])
                .query(&[("prebuilt", "1")])
                .json(&body),
            "create deployment",
            deadline,
        )?;
        if !identifier(&created.id, "dpl_") {
            return Err(failed("invalid deployment ID in response"));
        }
        self.check_created(&created, target)?;
        let id = created.id;
        zellij_core::statusln!("  Deploy ID {id}");
        loop {
            // Creation can return a reduced object. Always fetch full detail
            // to verify project identity and deployment URL, even if READY.
            let deployed: Deployment = self.json(
                self.request(Method::GET, &["v13", "deployments", &id]),
                "deployment status",
                deadline,
            )?;
            if deployed.created.id != id || deployed.project_id != target.project.id {
                return Err(failed(
                    "deployment returned an unexpected ID or project; inspect the Vercel dashboard",
                ));
            }
            self.check_created(&deployed.created, target)?;
            let aliases_ready = deployed
                .alias_assigned
                .as_ref()
                .is_some_and(|v| v == &json!(true) || v.as_u64().is_some_and(|n| n > 0));
            if deployed.created.ready_state == "READY" && (!target.production || aliases_ready) {
                let url = deployment_url(&deployed.url)?;
                return Ok(Deployed { id, url });
            }
            let remaining = deadline.checked_duration_since(Instant::now()).ok_or_else(|| failed("Vercel deployment timed out; it may still complete remotely; check the dashboard before retrying"))?;
            std::thread::sleep(self.poll_interval.min(remaining));
        }
    }
    fn check_created(&self, d: &Created, target: &Target) -> Result<()> {
        if d.target.as_deref()
            != if target.production {
                Some("production")
            } else {
                None
            }
        {
            return Err(failed(
                "deployment returned an unexpected environment; inspect the Vercel dashboard",
            ));
        }
        if d.error_code.is_some() || d.alias_error.is_some() {
            return Err(failed(
                "Vercel reported a deployment/alias error; inspect deployment logs in the dashboard",
            ));
        }
        match d.ready_state.as_str() {
            "READY" | "QUEUED" | "INITIALIZING" | "BUILDING" => Ok(()),
            "ERROR" | "CANCELED" => Err(failed(
                "Vercel reported a failed/canceled deployment; inspect the dashboard",
            )),
            _ => Err(failed(
                "Vercel returned an unsupported deployment state; inspect the dashboard",
            )),
        }
    }
}
fn deployment_url(host: &str) -> Result<Url> {
    let url =
        Url::parse(&format!("https://{host}")).map_err(|_| failed("invalid deployment URL"))?;
    if url.host_str() != Some(host)
        || !host.ends_with(".vercel.app")
        || url.port().is_some()
        || !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
    {
        return Err(failed(
            "unexpected deployment URL; inspect the Vercel dashboard",
        ));
    }
    Ok(url)
}
fn output_config() -> Value {
    // Keep literal .html links valid (portable renderer) while exposing clean
    // hosted routes. Filesystem hits win; only existing static files resolve.
    json!({"version":3,"routes":[
        {"handle":"filesystem"},
        {"src":"^/$","dest":"/index.html","check":true},
        {"src":"^/(.+)/$","dest":"/$1/index.html","check":true},
        {"src":"^/(.+)$","dest":"/$1.html","check":true},
        {"src":"^/(.+)$","dest":"/$1/index.html","check":true}
    ]})
}
struct Asset {
    path: Utf8PathBuf,
    size: u64,
}
struct Manifest {
    files: Vec<Value>,
    assets: BTreeMap<String, Asset>,
}
fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha1::digest(bytes))
}
fn read_asset(path: &Utf8Path) -> Result<Vec<u8>> {
    let meta = std::fs::symlink_metadata(path)?;
    if zellij_core::site::is_link(&meta) || !meta.is_file() {
        return Err(failed(
            "export contains a symlink, junction or non-regular file",
        ));
    }
    if meta.len() > MAX_FILE {
        return Err(failed("zel Vercel assets must be at most 100 MiB each"));
    }
    let mut bytes = Vec::new();
    std::fs::File::open(path)?
        .take(MAX_FILE + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() as u64 > MAX_FILE {
        return Err(failed("zel Vercel assets must be at most 100 MiB each"));
    }
    Ok(bytes)
}
impl Manifest {
    fn read(root: &Utf8Path) -> Result<Self> {
        let mut files = Vec::new();
        let mut assets = BTreeMap::new();
        let mut total = 0u64;
        let mut index = false;
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
            if relative.components().next().is_some_and(|p| {
                matches!(p.as_str(), ".vercel" | "vercel.json" | ".now" | "now.json")
            }) {
                return Err(failed(
                    "Vercel deployment supports generated static assets only; remove custom Vercel configuration/functions from the export",
                ));
            }
            if meta.is_dir() || relative == ".zellij-output" {
                continue;
            }
            let name = relative
                .components()
                .map(|c| c.as_str())
                .collect::<Vec<_>>()
                .join("/");
            if name
                .chars()
                .any(|c| c.is_control() || matches!(c, '#' | '?' | '\\'))
            {
                return Err(failed(
                    "Vercel asset paths cannot contain #, ?, backslashes or control characters",
                ));
            }
            let bytes = read_asset(path)?;
            total += bytes.len() as u64;
            if total > MAX_TOTAL {
                return Err(failed(
                    "zel Vercel deployments support at most 100 MiB of static content",
                ));
            }
            let sha = hash(&bytes);
            index |= name == "index.html" || name.ends_with("/index.html");
            files.push(json!({"file":format!(".vercel/output/static/{name}"), "sha":sha, "size":bytes.len()}));
            assets.entry(sha).or_insert(Asset {
                path: path.to_owned(),
                size: bytes.len() as u64,
            });
            if files.len() > MAX_FILES {
                return Err(failed(
                    "zel Vercel deployments support at most 15,000 static files",
                ));
            }
        }
        if !index {
            return Err(failed(
                "generated export has no index.html; no deployment was created",
            ));
        }
        Ok(Self { files, assets })
    }
}
#[cfg(test)]
#[path = "vercel_tests.rs"]
mod tests;
