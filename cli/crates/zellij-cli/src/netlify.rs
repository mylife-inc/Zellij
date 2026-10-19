//! Native digest deployment: no external CLI, project discovery, remote build,
//! functions or automatic site creation. Protocol reference:
//! https://docs.netlify.com/api-and-cli-guides/api-guides/get-started-with-api/
use camino::{Utf8Path, Utf8PathBuf};
use reqwest::{
    Method, Url,
    blocking::{Client as HttpClient, RequestBuilder, Response},
    header,
};
use serde::Deserialize;
use sha1::{Digest, Sha1};
use std::{
    collections::BTreeMap,
    fs::File,
    io::Read,
    time::{Duration, Instant},
};
use zellij_core::{Error, Result};

const API: &str = "https://api.netlify.com/api/v1/";
const RESPONSE_LIMIT: u64 = 8 * 1024 * 1024;
fn failed(message: impl Into<String>) -> Error {
    Error::DeploymentFailed(message.into())
}

pub(crate) struct Client {
    http: HttpClient,
    base: Url,
    token: header::HeaderValue,
    timeout: Duration,
    poll_interval: Duration,
}

#[derive(Deserialize)]
struct SiteResponse {
    id: String,
}

#[derive(Deserialize)]
struct Deployment {
    id: String,
    state: String,
    #[serde(default)]
    required: Option<Vec<String>>,
    #[serde(default)]
    required_functions: Option<Vec<String>>,
    #[serde(default)]
    deploy_ssl_url: Option<String>,
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
        let token = std::env::var("NETLIFY_AUTH_TOKEN").map_err(|_| Error::InvalidArgument("set NETLIFY_AUTH_TOKEN to a Netlify personal access token; zel does not run login or read another CLI's credential store".into()))?;
        Self::new(
            API,
            &token,
            Duration::from_secs(300),
            Duration::from_secs(2),
        )
    }

    // Endpoint injection is private and used only by loopback unit tests.
    // Production never takes an API URL from config, arguments or environment.
    fn new(base: &str, token: &str, timeout: Duration, poll_interval: Duration) -> Result<Self> {
        if token.is_empty() || !token.bytes().all(|c| c.is_ascii_graphic()) {
            return Err(Error::InvalidArgument(
                "NETLIFY_AUTH_TOKEN must be a nonempty token without whitespace".into(),
            ));
        }
        let mut token = header::HeaderValue::from_str(&format!("Bearer {token}"))
            .map_err(|_| Error::InvalidArgument("invalid NETLIFY_AUTH_TOKEN".into()))?;
        token.set_sensitive(true);
        let http = HttpClient::builder()
            .redirect(reqwest::redirect::Policy::none())
            .connect_timeout(Duration::from_secs(15))
            .timeout(Duration::from_secs(60))
            .user_agent(concat!("zel/", env!("CARGO_PKG_VERSION")))
            .build()
            .map_err(|_| failed("could not initialize the HTTPS client"))?;
        let base = Url::parse(base).map_err(|_| failed("invalid API origin"))?;
        Ok(Self {
            http,
            base,
            token,
            timeout,
            poll_interval,
        })
    }

    fn request(&self, method: Method, parts: &[&str]) -> RequestBuilder {
        let mut url = self.base.clone();
        url.path_segments_mut()
            .expect("API is hierarchical")
            .pop_if_empty()
            .extend(parts.iter().copied());
        self.http
            .request(method, url)
            .header(header::AUTHORIZATION, self.token.clone())
    }

    fn send(&self, request: RequestBuilder, stage: &str) -> Result<Response> {
        // Do not echo provider bodies, headers, tokens, or HTTP-client debug
        // output. They can contain credentials or terminal control sequences.
        let response = request.send().map_err(|error| {
            failed(format!(
                "{stage}: {}; check connectivity and the Netlify dashboard before retrying",
                if error.is_timeout() {
                    "request timed out"
                } else {
                    "HTTPS request failed"
                }
            ))
        })?;
        let status = response.status();
        if !status.is_success() {
            let hint = match status.as_u16() {
                401 | 403 => "check NETLIFY_AUTH_TOKEN and access to this site/team",
                404 => "check the existing site ID and account access",
                429 => "Netlify rate limit reached; wait before retrying",
                300..=399 => "redirect refused; credentials were not forwarded",
                _ => "check the Netlify dashboard before retrying",
            };
            return Err(failed(format!(
                "{stage}: HTTP {} — {hint}",
                status.as_u16()
            )));
        }
        Ok(response)
    }

    fn json<T: serde::de::DeserializeOwned>(
        &self,
        request: RequestBuilder,
        stage: &str,
    ) -> Result<T> {
        let response = self.send(request, stage)?;
        let mut bytes = Vec::new();
        response
            .take(RESPONSE_LIMIT + 1)
            .read_to_end(&mut bytes)
            .map_err(|_| failed(format!("{stage}: could not read the response")))?;
        if bytes.len() as u64 > RESPONSE_LIMIT {
            return Err(failed(format!("{stage}: response exceeded 8 MiB")));
        }
        serde_json::from_slice(&bytes).map_err(|_| failed(format!("{stage}: invalid API response")))
    }

    pub fn check_site(&self, site: &str) -> Result<()> {
        let result: SiteResponse = self.json(
            self.request(Method::GET, &["sites", site]),
            "site preflight",
        )?;
        zellij_core::delivery::validate_site_identifier(&result.id)
            .map_err(|_| failed("site preflight: invalid site ID in response"))
    }

    fn deadline_request(
        &self,
        request: RequestBuilder,
        deadline: Instant,
    ) -> Result<RequestBuilder> {
        let remaining = deadline.checked_duration_since(Instant::now()).ok_or_else(|| failed("deployment timed out after waiting for Netlify; it may still complete remotely"))?;
        Ok(request.timeout(remaining.min(Duration::from_secs(60))))
    }

    fn poll(&self, site: &str, id: &str, deadline: Instant) -> Result<Deployment> {
        let remaining = deadline
            .checked_duration_since(Instant::now())
            .ok_or_else(|| failed("deployment timed out; check the Netlify dashboard"))?;
        std::thread::sleep(self.poll_interval.min(remaining));
        let request = self.deadline_request(
            self.request(Method::GET, &["sites", site, "deploys", id]),
            deadline,
        )?;
        let deploy: Deployment = self.json(request, "deployment status")?;
        if deploy.id != id {
            return Err(failed("deployment status returned an unexpected ID"));
        }
        Ok(deploy)
    }

    pub fn deploy(&self, site: &str, export: &Utf8Path, production: bool) -> Result<Deployed> {
        let files = Manifest::read(export)?;
        let deadline = Instant::now() + self.timeout;
        let request = self
            .request(Method::POST, &["sites", site, "deploys"])
            .json(&serde_json::json!({"files": files.hashes, "async": true, "draft": !production}));
        // Never retry POST automatically: an uncertain response can already
        // have created a deployment, including a production deployment.
        let mut deployment: Deployment = self.json(
            self.deadline_request(request, deadline)?,
            "create deployment",
        )?;
        let id = deployment.id.clone();
        if id.is_empty() || !id.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'-') {
            return Err(failed(
                "create deployment returned an invalid deployment ID",
            ));
        }
        zellij_core::statusln!("  Deploy ID {id}");
        while matches!(deployment.state.as_str(), "preparing" | "new") {
            deployment = self.poll(site, &id, deadline)?;
        }
        if matches!(deployment.state.as_str(), "prepared" | "uploading") {
            if deployment
                .required_functions
                .as_ref()
                .is_some_and(|v| !v.is_empty())
            {
                return Err(failed(
                    "unexpected function uploads requested for a static deployment",
                ));
            }
            let required = deployment
                .required
                .as_ref()
                .ok_or_else(|| failed("deployment response omitted the required file list"))?;
            // Validate the entire response before uploading any file.
            let mut uploads = BTreeMap::new();
            for hash in required {
                let file = files.by_hash.get(hash).ok_or_else(|| {
                    failed("deployment requested a file outside the generated manifest")
                })?;
                uploads.insert(hash, file);
            }
            for (_, (name, path)) in uploads {
                let mut parts = vec!["deploys", &id, "files"];
                parts.extend(name.trim_start_matches('/').split('/'));
                let request = self
                    .request(Method::PUT, &parts)
                    .header(header::CONTENT_TYPE, "application/octet-stream")
                    .body(File::open(path)?);
                self.send(self.deadline_request(request, deadline)?, "upload file")?;
            }
        }
        loop {
            match deployment.state.as_str() {
                "ready" => {
                    let url = deployment.deploy_ssl_url.as_deref().and_then(|v| Url::parse(v).ok())
                        .filter(|u| u.scheme() == "https" && u.host_str().is_some() && u.username().is_empty() && u.password().is_none() && u.query().is_none() && u.fragment().is_none())
                        .ok_or_else(|| failed("deployment is ready, but its HTTPS preview URL is missing or invalid; find it in the Netlify dashboard"))?;
                    return Ok(Deployed { id, url });
                }
                "prepared" | "uploading" | "uploaded" | "processing" | "enqueued" | "retrying" => {
                    deployment = self.poll(site, &id, deadline)?
                }
                "error" | "failed" | "rejected" | "canceled" | "cancelled" => {
                    return Err(failed(
                        "Netlify reported a failed/canceled deployment; inspect the deployment in its dashboard",
                    ));
                }
                _ => {
                    return Err(failed(
                        "unexpected deployment state; inspect the deployment in the Netlify dashboard",
                    ));
                }
            }
        }
    }
}

struct Manifest {
    hashes: BTreeMap<String, String>,
    by_hash: BTreeMap<String, (String, Utf8PathBuf)>,
}
impl Manifest {
    fn read(root: &Utf8Path) -> Result<Self> {
        let mut hashes = BTreeMap::new();
        let mut by_hash = BTreeMap::new();
        for entry in walkdir::WalkDir::new(root)
            .follow_links(false)
            .sort_by_file_name()
        {
            let entry = entry.map_err(|_| failed("could not enumerate the generated export"))?;
            let metadata = std::fs::symlink_metadata(entry.path())?;
            if zellij_core::site::is_link(&metadata) {
                return Err(failed("generated export contains a symlink or junction"));
            }
            if metadata.is_dir() {
                continue;
            }
            if !metadata.is_file() {
                return Err(failed("generated export contains a non-regular file"));
            }
            let path = Utf8Path::from_path(entry.path())
                .ok_or_else(|| failed("export path is not UTF-8"))?;
            let relative = path
                .strip_prefix(root)
                .map_err(|_| failed("file is outside export"))?;
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
                return Err(Error::InvalidArgument("Netlify file paths cannot contain #, ?, backslashes or control characters; rename the referenced content asset".into()));
            }
            let mut file = File::open(path)?;
            let mut digest = Sha1::new();
            let mut buffer = [0; 65536];
            loop {
                let size = file.read(&mut buffer)?;
                if size == 0 {
                    break;
                }
                digest.update(&buffer[..size]);
            }
            let hash = format!("{:x}", digest.finalize());
            hashes.insert(name.clone(), hash.clone());
            by_hash.entry(hash).or_insert((name, path.to_owned()));
        }
        if !hashes.keys().any(|path| path.ends_with("/index.html")) {
            return Err(failed(
                "generated export has no index.html; no deployment was created",
            ));
        }
        Ok(Self { hashes, by_hash })
    }
}

#[cfg(test)]
#[path = "netlify_tests.rs"]
mod tests;
