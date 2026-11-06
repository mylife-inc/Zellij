//! GitHub.com Pages branch publication through the Git database and Pages APIs.
//! Only an existing gh-pages root is replaced; history is retained and refs are
//! updated without force. This provider has no draft mode.
use base64::{Engine as _, engine::general_purpose::STANDARD};
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
    cell::Cell,
    collections::BTreeMap,
    io::Read,
    time::{Duration, Instant},
};
use zellij_core::{Error, Result, delivery::Provider, site::Renderer};
use zellij_engine::BuildOptions;

const API: &str = "https://api.github.com/";
const RESPONSE_LIMIT: u64 = 8 * 1024 * 1024;
const MAX_FILE: u64 = 25 * 1024 * 1024;
const MAX_TOTAL: u64 = 100 * 1024 * 1024;
const MAX_FILES: usize = 5_000;
const INLINE_FILE: usize = 64 * 1024;
const INLINE_TOTAL: usize = 2 * 1024 * 1024;
const MAX_UPLOADS: usize = 200;
const MAX_TREE_JSON: usize = 6 * 1024 * 1024;
fn failed(message: impl Into<String>) -> Error {
    Error::DeploymentFailed(message.into())
}
fn valid_sha(sha: &str) -> bool {
    sha.len() == 40 && sha.bytes().all(|b| b.is_ascii_hexdigit())
}
#[derive(Deserialize, PartialEq, Eq)]
struct Repository {
    id: u64,
    full_name: String,
    default_branch: String,
    archived: bool,
}
#[derive(Deserialize, PartialEq, Eq)]
struct Source {
    branch: String,
    path: String,
}
#[derive(Deserialize, PartialEq, Eq)]
struct Pages {
    build_type: String,
    source: Source,
    html_url: String,
    cname: Option<String>,
}
#[derive(Deserialize)]
struct Oid {
    sha: String,
}
#[derive(Deserialize)]
struct RefObject {
    sha: String,
    #[serde(rename = "type")]
    kind: String,
}
#[derive(Deserialize)]
struct Reference {
    #[serde(rename = "ref")]
    name: String,
    object: RefObject,
}
#[derive(Deserialize)]
struct Commit {
    sha: String,
    tree: Oid,
    parents: Vec<Oid>,
}
#[derive(Deserialize)]
struct BuildError {
    message: Option<String>,
}
#[derive(Deserialize)]
struct Build {
    commit: String,
    status: String,
    error: Option<BuildError>,
}
pub(crate) struct Target {
    repository: Repository,
    pages: Pages,
    head: String,
    url: Url,
    mount: String,
}
pub(crate) struct Deployed {
    pub commit: String,
    pub url: Url,
}
pub(crate) struct Client {
    http: HttpClient,
    base: Url,
    token: header::HeaderValue,
    timeout: Duration,
    poll_interval: Duration,
    mutation_interval: Duration,
    last_mutation: Cell<Option<Instant>>,
}
impl Target {
    pub fn build_options(
        &self,
        renderer: Renderer,
        options: &BuildOptions,
    ) -> Result<BuildOptions> {
        let mut build = options.clone();
        if renderer == Renderer::Hosted {
            let prefix = build
                .overrides
                .base_path
                .as_ref()
                .map(ToString::to_string)
                .unwrap_or_else(|| self.mount.clone());
            self.check_prefix(&prefix)?;
            build.overrides.base_path = Some(prefix.parse().map_err(Error::InvalidArgument)?);
            if build.overrides.site_url.is_none() {
                build.overrides.site_url = Some(
                    self.url
                        .origin()
                        .ascii_serialization()
                        .parse()
                        .map_err(Error::InvalidArgument)?,
                );
            }
        }
        Ok(build)
    }
    fn check_prefix(&self, prefix: &str) -> Result<()> {
        let _: zellij_core::build_overrides::BasePath =
            prefix.parse().map_err(Error::InvalidArgument)?;
        if prefix != self.mount && !prefix.starts_with(&format!("{}/", self.mount)) {
            return Err(Error::InvalidArgument(format!(
                "GitHub Pages --base-path must equal or sit below the configured Pages path {}; it cannot point outside this site's mount",
                if self.mount.is_empty() {
                    "/"
                } else {
                    &self.mount
                }
            )));
        }
        Ok(())
    }
    fn same_site(&self, other: &Self) -> bool {
        self.repository == other.repository && self.pages == other.pages && self.url == other.url
    }
}
impl Client {
    pub fn from_environment() -> Result<Self> {
        let token = std::env::var("GITHUB_PAGES_TOKEN").map_err(|_| Error::InvalidArgument("set GITHUB_PAGES_TOKEN to a dedicated token with Contents and Pages write access to the existing repository; zel does not read gh credentials or select the built-in GITHUB_TOKEN".into()))?;
        Self::new(
            API,
            &token,
            Duration::from_secs(600),
            Duration::from_secs(2),
            Duration::from_secs(1),
        )
    }
    // Origin substitution and zero-delay mutation pacing are test-only callers.
    fn new(
        base: &str,
        token: &str,
        timeout: Duration,
        poll_interval: Duration,
        mutation_interval: Duration,
    ) -> Result<Self> {
        if token.is_empty() || !token.bytes().all(|b| b.is_ascii_graphic()) {
            return Err(Error::InvalidArgument(
                "GITHUB_PAGES_TOKEN must be a nonempty token without whitespace".into(),
            ));
        }
        let mut token = header::HeaderValue::from_str(&format!("Bearer {token}"))
            .map_err(|_| failed("invalid GITHUB_PAGES_TOKEN"))?;
        token.set_sensitive(true);
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
            timeout,
            poll_interval,
            mutation_interval,
            last_mutation: Cell::new(None),
        })
    }
    fn request(&self, method: Method, repository: &str, suffix: &[&str]) -> RequestBuilder {
        let mut url = self.base.clone();
        let (owner, repo) = repository.split_once('/').expect("validated repository");
        url.path_segments_mut()
            .expect("hierarchical API")
            .pop_if_empty()
            .extend(["repos", owner, repo])
            .extend(suffix.iter().copied());
        self.http
            .request(method, url)
            .header(header::AUTHORIZATION, self.token.clone())
            .header(header::ACCEPT, "application/vnd.github+json")
            .header("X-GitHub-Api-Version", "2022-11-28")
    }
    fn json<T: serde::de::DeserializeOwned>(
        &self,
        request: RequestBuilder,
        stage: &str,
        deadline: Instant,
        mutation: bool,
    ) -> Result<T> {
        self.optional_json(request, stage, deadline, mutation, false)?
            .ok_or_else(|| failed(format!("{stage}: missing response")))
    }
    fn optional_json<T: serde::de::DeserializeOwned>(
        &self,
        request: RequestBuilder,
        stage: &str,
        deadline: Instant,
        mutation: bool,
        allow_missing: bool,
    ) -> Result<Option<T>> {
        if mutation {
            if let Some(last) = self.last_mutation.get() {
                let wait = self.mutation_interval.saturating_sub(last.elapsed());
                if wait >= deadline.saturating_duration_since(Instant::now()) {
                    return Err(failed(
                        "GitHub Pages operation timed out; inspect the repository before retrying",
                    ));
                }
                std::thread::sleep(wait);
            }
            self.last_mutation.set(Some(Instant::now()));
        }
        let remaining = deadline
            .checked_duration_since(Instant::now())
            .ok_or_else(|| {
                failed("GitHub Pages operation timed out; inspect the repository before retrying")
            })?;
        let response = request
            .timeout(remaining.min(Duration::from_secs(60)))
            .send()
            .map_err(|e| {
                failed(format!(
                    "{stage}: {}; inspect the repository before retrying",
                    if e.is_timeout() {
                        "request timed out"
                    } else {
                        "HTTPS request failed"
                    }
                ))
            })?;
        let status = response.status();
        if status.as_u16() == 404 && allow_missing {
            return Ok(None);
        }
        if !status.is_success() {
            let hint = match status.as_u16() {
                401 | 403 => {
                    "check GITHUB_PAGES_TOKEN permissions, repository rules and GitHub rate limits"
                }
                404 => {
                    "check the owner/repository, existing Pages configuration and gh-pages branch"
                }
                409 | 422 => {
                    "the branch may have changed, a repository rule may forbid publication, or GitHub rejected the request; inspect Pages and branch settings"
                }
                429 => "GitHub rate limit reached; wait before retrying",
                300..=399 => "redirect refused; credentials were not forwarded",
                _ => "inspect the repository and Pages builds before retrying",
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
            .map_err(|error| {
                let timed_out = Instant::now() >= deadline
                    || error.kind() == std::io::ErrorKind::TimedOut
                    || error
                        .get_ref()
                        .and_then(|source| source.downcast_ref::<reqwest::Error>())
                        .is_some_and(reqwest::Error::is_timeout);
                failed(format!(
                    "{stage}: {}; inspect the repository before retrying",
                    if timed_out {
                        "response read timed out"
                    } else {
                        "could not read response"
                    }
                ))
            })?;
        if bytes.len() as u64 > RESPONSE_LIMIT {
            return Err(failed(format!("{stage}: response exceeded 8 MiB")));
        }
        // No API response body, response error message or token is logged.
        serde_json::from_slice(&bytes)
            .map(Some)
            .map_err(|_| failed(format!("{stage}: invalid API response")))
    }
    pub fn check_site(&self, repository: &str) -> Result<Target> {
        self.check_site_at(repository, Instant::now() + self.timeout)
    }
    fn check_site_at(&self, repository: &str, deadline: Instant) -> Result<Target> {
        Provider::GithubPages.validate_site(repository)?;
        let repo: Repository = self.json(
            self.request(Method::GET, repository, &[]),
            "repository preflight",
            deadline,
            false,
        )?;
        if repo.id == 0
            || !repo.full_name.eq_ignore_ascii_case(repository)
            || repo.archived
            || repo.default_branch == "gh-pages"
            || repo.default_branch.is_empty()
        {
            return Err(failed(
                "repository preflight: unexpected/archived repository or gh-pages is the source/default branch",
            ));
        }
        let pages: Pages = self.json(
            self.request(Method::GET, repository, &["pages"]),
            "Pages preflight",
            deadline,
            false,
        )?;
        if pages.build_type != "legacy"
            || pages.source.branch != "gh-pages"
            || pages.source.path != "/"
        {
            return Err(failed(
                "configure the existing GitHub Pages site to Deploy from a branch: gh-pages / (root); workflow-mode, other branches and /docs sources are not supported by this adapter",
            ));
        }
        let url = site_url(&pages.html_url)?;
        let mount = url.path().trim_end_matches('/').to_owned();
        let _: zellij_core::build_overrides::BasePath = mount
            .parse()
            .map_err(|_| failed("Pages preflight: unsupported URL path"))?;
        if let Some(cname) = &pages.cname {
            let domain = site_url(&format!("https://{cname}/"))?;
            if domain
                .host_str()
                .is_none_or(|host| !host.eq_ignore_ascii_case(cname))
                || domain.path() != "/"
                || domain.host_str() != url.host_str()
            {
                return Err(failed("Pages preflight: invalid custom domain"));
            }
        }
        let head = self.branch(repository, deadline)?;
        Ok(Target {
            repository: repo,
            pages,
            head,
            url,
            mount,
        })
    }
    fn branch(&self, repo: &str, deadline: Instant) -> Result<String> {
        let reference: Reference = self.json(
            self.request(Method::GET, repo, &["git", "ref", "heads", "gh-pages"]),
            "branch preflight",
            deadline,
            false,
        )?;
        if reference.name != "refs/heads/gh-pages"
            || reference.object.kind != "commit"
            || !valid_sha(&reference.object.sha)
        {
            return Err(failed("branch preflight: unexpected gh-pages reference"));
        }
        Ok(reference.object.sha)
    }
    pub fn deploy(
        &self,
        target: &Target,
        export: &Utf8Path,
        renderer: Renderer,
        prefix: &str,
    ) -> Result<Deployed> {
        if renderer == Renderer::Hosted {
            target.check_prefix(prefix)?;
        } else if !prefix.is_empty() {
            return Err(failed(
                "portable deployment returned an unexpected build prefix",
            ));
        }
        let manifest = Manifest::read(export, target, renderer, prefix)?;
        let tree_body = json!({"tree":manifest.entries});
        if serde_json::to_vec(&tree_body).expect("JSON tree").len() > MAX_TREE_JSON {
            return Err(failed(
                "GitHub Pages tree payload exceeds zel's 6 MiB limit; reduce file/path count",
            ));
        }
        let deadline = Instant::now() + self.timeout;
        let repo = &target.repository.full_name;
        for (sha, asset) in &manifest.uploads {
            let bytes = read_asset(&asset.path)?;
            if blob_hash(&bytes) != *sha {
                return Err(failed(
                    "export changed during upload; the publishing branch was not updated",
                ));
            }
            let uploaded: Oid = self.json(
                self.request(Method::POST, repo, &["git", "blobs"])
                    .json(&json!({"content":STANDARD.encode(&bytes),"encoding":"base64"})),
                "blob upload",
                deadline,
                true,
            )?;
            if uploaded.sha != *sha {
                return Err(failed(
                    "GitHub returned an unexpected blob digest; the publishing branch was not updated",
                ));
            }
        }
        let tree: Oid = self.json(
            self.request(Method::POST, repo, &["git", "trees"])
                .json(&tree_body),
            "create tree",
            deadline,
            true,
        )?;
        if !valid_sha(&tree.sha) {
            return Err(failed("invalid tree digest in response"));
        }
        let commit: Commit = self.json(self.request(Method::POST,repo,&["git","commits"]).json(&json!({"message":"Publish Zellij static site","tree":tree.sha,"parents":[target.head]})),"create commit",deadline,true)?;
        if !valid_sha(&commit.sha)
            || commit.tree.sha != tree.sha
            || commit.parents.len() != 1
            || commit.parents[0].sha != target.head
        {
            return Err(failed(
                "GitHub returned an unexpected commit/tree/parent; the publishing branch was not updated",
            ));
        }
        // Detect settings/domain/branch changes during the build or uploads.
        // The final non-forced update also rejects a concurrent divergent push.
        let current = self.check_site_at(repo, deadline)?;
        if !target.same_site(&current) || current.head != target.head {
            return Err(failed(
                "Pages settings or gh-pages changed during deployment; the publishing branch was not updated; inspect and retry",
            ));
        }
        let updated: Reference = self.json(
            self.request(Method::PATCH, repo, &["git", "refs", "heads", "gh-pages"])
                .json(&json!({"sha":commit.sha,"force":false})),
            "publish branch",
            deadline,
            true,
        )?;
        if updated.name != "refs/heads/gh-pages"
            || updated.object.kind != "commit"
            || updated.object.sha != commit.sha
        {
            return Err(failed(
                "unexpected branch update response; publication may already have started; inspect GitHub before retrying",
            ));
        }
        zellij_core::statusln!("  Published commit {}", commit.sha);
        // Explicitly request the Pages build; never infer readiness from a push.
        // Neither publication mutation is automatically retried.
        let _: Value = self.json(
            self.request(Method::POST, repo, &["pages", "builds"]),
            "request Pages build",
            deadline,
            true,
        )?;
        loop {
            if self.branch(repo, deadline)? != commit.sha {
                return Err(failed(
                    "gh-pages was superseded by another commit; inspect GitHub before retrying",
                ));
            }
            let build: Option<Build> = self.optional_json(
                self.request(Method::GET, repo, &["pages", "builds", "latest"]),
                "Pages build status",
                deadline,
                false,
                true,
            )?;
            if let Some(build) = build.filter(|b| b.commit == commit.sha) {
                if build
                    .error
                    .and_then(|e| e.message)
                    .is_some_and(|m| !m.is_empty())
                {
                    return Err(failed(
                        "GitHub Pages reported a build error; inspect Pages build logs",
                    ));
                }
                match build.status.as_str() {
                    "built" => {
                        let current = self.check_site_at(repo, deadline)?;
                        if !target.same_site(&current) || current.head != commit.sha {
                            return Err(failed(
                                "Pages configuration or branch changed while publishing; inspect GitHub",
                            ));
                        }
                        let mut url = target.url.clone();
                        if renderer == Renderer::Hosted {
                            url.set_path(&format!("{}/", prefix.trim_end_matches('/')));
                        }
                        return Ok(Deployed {
                            commit: commit.sha,
                            url,
                        });
                    }
                    "queued" | "building" => {}
                    "errored" => {
                        return Err(failed(
                            "GitHub Pages build failed; inspect Pages build logs",
                        ));
                    }
                    _ => {
                        return Err(failed(
                            "unsupported GitHub Pages build status; inspect Pages build logs",
                        ));
                    }
                }
            }
            let remaining=deadline.checked_duration_since(Instant::now()).ok_or_else(||failed("GitHub Pages timed out waiting for the published commit; the remote build may still finish; inspect GitHub before retrying"))?;
            std::thread::sleep(self.poll_interval.min(remaining));
        }
    }
}
fn site_url(value: &str) -> Result<Url> {
    let url = Url::parse(value).map_err(|_| failed("Pages preflight: invalid site URL"))?;
    if url.scheme() != "https"
        || url.host_str().is_none()
        || url.port().is_some()
        || !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
        || value
            .chars()
            .any(|c| c.is_control() || c.is_whitespace() || c == '\\')
    {
        return Err(failed(
            "Pages requires an existing HTTPS site URL without credentials, query or fragment; configure HTTPS in Pages settings first",
        ));
    }
    Ok(url)
}
fn blob_hash(bytes: &[u8]) -> String {
    let mut hash = Sha1::new();
    hash.update(format!("blob {}\0", bytes.len()).as_bytes());
    hash.update(bytes);
    format!("{:x}", hash.finalize())
}
fn read_asset(path: &Utf8Path) -> Result<Vec<u8>> {
    let meta = std::fs::symlink_metadata(path)?;
    if zellij_core::site::is_link(&meta) || !meta.is_file() {
        return Err(failed(
            "export contains a symlink, junction or non-regular file",
        ));
    }
    if meta.len() > MAX_FILE {
        return Err(failed("zel GitHub Pages files must be at most 25 MiB each"));
    }
    let mut bytes = Vec::new();
    std::fs::File::open(path)?
        .take(MAX_FILE + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() as u64 > MAX_FILE {
        return Err(failed("zel GitHub Pages files must be at most 25 MiB each"));
    }
    Ok(bytes)
}
struct Asset {
    path: Utf8PathBuf,
}
struct Manifest {
    entries: Vec<Value>,
    uploads: BTreeMap<String, Asset>,
}
impl Manifest {
    fn read(root: &Utf8Path, target: &Target, renderer: Renderer, prefix: &str) -> Result<Self> {
        let mount = if renderer == Renderer::Hosted {
            target.mount.trim_start_matches('/')
        } else {
            ""
        };
        let subpath = if renderer == Renderer::Hosted {
            prefix
                .strip_prefix(&target.mount)
                .unwrap_or("")
                .trim_matches('/')
        } else {
            ""
        };
        let expected_index = if subpath.is_empty() {
            "index.html".to_owned()
        } else {
            format!("{subpath}/index.html")
        };
        let mut entries = Vec::new();
        let mut uploads = BTreeMap::new();
        let mut total = 0u64;
        let mut inline = 0usize;
        let mut has_index = false;
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
            if meta.is_dir() {
                continue;
            }
            if relative == ".zellij-output" {
                continue;
            }
            let relative = relative
                .strip_prefix(mount)
                .map_err(|_| failed("generated hosted files sit outside the Pages mount"))?;
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
                    "GitHub Pages asset paths cannot contain #, ?, backslashes or control characters",
                ));
            }
            if relative.components().next().is_some_and(|p| {
                matches!(
                    p.as_str().to_ascii_lowercase().as_str(),
                    ".git"
                        | ".github"
                        | ".gitmodules"
                        | ".gitattributes"
                        | "_config.yml"
                        | "_config.yaml"
                )
            }) {
                return Err(failed(
                    "GitHub Pages upload accepts static output only; remove Git/workflow/Jekyll configuration from the export",
                ));
            }
            let bytes = read_asset(path)?;
            total += bytes.len() as u64;
            if total > MAX_TOTAL {
                return Err(failed(
                    "zel GitHub Pages exports support at most 100 MiB of content",
                ));
            }
            if name == "CNAME" {
                if !target.pages.cname.as_ref().is_some_and(|c| {
                    std::str::from_utf8(&bytes).is_ok_and(|v| v.trim().eq_ignore_ascii_case(c))
                }) {
                    return Err(failed(
                        "export CNAME conflicts with existing Pages settings; configure the domain in GitHub first",
                    ));
                }
                continue;
            }
            if name == ".nojekyll" {
                continue;
            }
            has_index |= name == expected_index;
            let mut item = json!({"path":name,"mode":"100644","type":"blob"});
            if bytes.len() <= INLINE_FILE
                && inline + bytes.len() <= INLINE_TOTAL
                && let Ok(text) = std::str::from_utf8(&bytes)
            {
                inline += bytes.len();
                item["content"] = json!(text);
            } else {
                let sha = blob_hash(&bytes);
                item["sha"] = json!(sha);
                uploads.entry(sha).or_insert(Asset {
                    path: path.to_owned(),
                });
            }
            entries.push(item);
            if entries.len() > MAX_FILES {
                return Err(failed(
                    "zel GitHub Pages exports support at most 5,000 static files",
                ));
            }
        }
        if !has_index {
            return Err(failed(
                "generated export has no index.html at the selected Pages URL; no branch was updated",
            ));
        }
        if uploads.len() > MAX_UPLOADS {
            return Err(failed(
                "zel GitHub Pages supports at most 200 distinct binary/large-file uploads per invocation; reduce the export size",
            ));
        }
        entries.push(json!({"path":".nojekyll","mode":"100644","type":"blob","content":""}));
        if let Some(cname) = &target.pages.cname {
            entries.push(json!({"path":"CNAME","mode":"100644","type":"blob","content":format!("{cname}\n")}));
        }
        Ok(Self { entries, uploads })
    }
}
#[cfg(test)]
#[path = "github_pages_tests.rs"]
mod tests;
