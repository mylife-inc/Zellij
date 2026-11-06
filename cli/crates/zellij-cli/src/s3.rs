//! Native AWS S3 website uploads. Only existing commercial-region website buckets
//! are supported. No bucket configuration, policy, ACL, delete or retry operation.
use base64::{Engine as _, engine::general_purpose::STANDARD};
use camino::{Utf8Path, Utf8PathBuf};
use hmac::{Hmac, Mac};
use reqwest::{
    Method, Url,
    blocking::Client as HttpClient,
    header::{self, HeaderMap, HeaderValue},
};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    io::Read,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use zellij_core::{Error, Result, delivery::Provider, site::Renderer};
use zellij_engine::BuildOptions;

const MAX_FILE: u64 = 25 * 1024 * 1024;
const MAX_TOTAL: u64 = 100 * 1024 * 1024;
const MAX_OBJECTS: usize = 5_000;
const RESPONSE_LIMIT: u64 = 256 * 1024;
fn failed(message: impl Into<String>) -> Error {
    Error::DeploymentFailed(message.into())
}
fn invalid(message: impl Into<String>) -> Error {
    Error::InvalidArgument(message.into())
}
fn required(name: &str) -> Result<String> {
    std::env::var(name).ok().filter(|v| !v.is_empty()).ok_or_else(|| invalid(format!("set {name} in the environment; zel does not read AWS profiles, stored login, instance metadata or credential processes")))
}
struct Credentials {
    access: String,
    secret: String,
    session: Option<String>,
}
impl Credentials {
    fn new(access: &str, secret: &str, session: Option<&str>) -> Result<Self> {
        if !(16..=128).contains(&access.len()) || !access.bytes().all(|b| b.is_ascii_alphanumeric())
        {
            return Err(invalid(
                "AWS_ACCESS_KEY_ID must be an ASCII alphanumeric access key ID",
            ));
        }
        if secret.is_empty() || secret.len() > 4096 || !secret.bytes().all(|b| b.is_ascii_graphic())
        {
            return Err(invalid(
                "AWS_SECRET_ACCESS_KEY must be a nonempty key without whitespace",
            ));
        }
        if session.is_some_and(|s| {
            s.is_empty() || s.len() > 16_384 || !s.bytes().all(|b| b.is_ascii_graphic())
        }) {
            return Err(invalid(
                "AWS_SESSION_TOKEN must be a nonempty token without whitespace",
            ));
        }
        if access.starts_with("ASIA") && session.is_none() {
            return Err(invalid(
                "temporary AWS access keys require AWS_SESSION_TOKEN",
            ));
        }
        Ok(Self {
            access: access.into(),
            secret: secret.into(),
            session: session.map(str::to_owned),
        })
    }
    fn authorization(
        &self,
        region: &str,
        date: &str,
        method: &Method,
        url: &Url,
        headers: &BTreeMap<String, String>,
    ) -> String {
        let canonical_headers = headers
            .iter()
            .map(|(name, value)| {
                format!(
                    "{name}:{}\n",
                    value.split_whitespace().collect::<Vec<_>>().join(" ")
                )
            })
            .collect::<String>();
        let signed = headers
            .keys()
            .map(String::as_str)
            .collect::<Vec<_>>()
            .join(";");
        let mut query = url
            .query_pairs()
            .map(|(k, v)| (encode(&k, false), encode(&v, false)))
            .collect::<Vec<_>>();
        query.sort();
        let query = query
            .into_iter()
            .map(|(k, v)| format!("{k}={v}"))
            .collect::<Vec<_>>()
            .join("&");
        let canonical = format!(
            "{}\n{}\n{}\n{}\n{}\n{}",
            method.as_str(),
            url.path(),
            query,
            canonical_headers,
            signed,
            headers["x-amz-content-sha256"]
        );
        let day = &date[..8];
        let scope = format!("{day}/{region}/s3/aws4_request");
        let to_sign = format!(
            "AWS4-HMAC-SHA256\n{date}\n{scope}\n{}",
            hash(canonical.as_bytes())
        );
        let key = mac(format!("AWS4{}", self.secret).as_bytes(), day.as_bytes());
        let key = mac(&key, region.as_bytes());
        let key = mac(&key, b"s3");
        let key = mac(&key, b"aws4_request");
        let signature = hex(&mac(&key, to_sign.as_bytes()));
        format!(
            "AWS4-HMAC-SHA256 Credential={}/{scope}, SignedHeaders={signed}, Signature={signature}",
            self.access
        )
    }
}
fn mac(key: &[u8], bytes: &[u8]) -> Vec<u8> {
    let mut mac = Hmac::<Sha256>::new_from_slice(key).expect("HMAC accepts any key length");
    mac.update(bytes);
    mac.finalize().into_bytes().to_vec()
}
fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
fn hash(bytes: &[u8]) -> String {
    hex(&Sha256::digest(bytes))
}
fn encode(text: &str, slash: bool) -> String {
    let mut out = String::new();
    for b in text.bytes() {
        if b.is_ascii_alphanumeric() || b"-._~".contains(&b) || (slash && b == b'/') {
            out.push(b as char);
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}
// Explicit commercial-region website formats from AWS's endpoint table. The
// REST upload origin always uses TLS and path-style addressing (also for dots).
fn website_endpoint(region: &str) -> Result<String> {
    let dash = matches!(
        region,
        "us-east-1"
            | "us-west-1"
            | "us-west-2"
            | "ap-southeast-1"
            | "ap-southeast-2"
            | "ap-northeast-1"
            | "eu-west-1"
            | "sa-east-1"
    );
    if !dash
        && !matches!(
            region,
            "us-east-2"
                | "af-south-1"
                | "ap-east-1"
                | "ap-east-2"
                | "ap-south-1"
                | "ap-south-2"
                | "ap-southeast-3"
                | "ap-southeast-4"
                | "ap-southeast-5"
                | "ap-southeast-6"
                | "ap-southeast-7"
                | "ap-northeast-2"
                | "ap-northeast-3"
                | "ca-central-1"
                | "ca-west-1"
                | "eu-central-1"
                | "eu-central-2"
                | "eu-west-2"
                | "eu-west-3"
                | "eu-south-1"
                | "eu-south-2"
                | "eu-north-1"
                | "il-central-1"
                | "mx-central-1"
                | "me-south-1"
                | "me-central-1"
        )
    {
        return Err(invalid(
            "AWS_REGION must name a supported commercial AWS S3 website region; China, GovCloud, custom endpoints and S3-compatible services are not supported",
        ));
    }
    Ok(format!(
        "s3-website{}{region}.amazonaws.com",
        if dash { "-" } else { "." }
    ))
}
#[derive(Debug, PartialEq, Eq)]
struct Website {
    error_document: Option<String>,
}
fn website(bytes: &[u8]) -> Result<Website> {
    let text = std::str::from_utf8(bytes)
        .map_err(|_| failed("website preflight: invalid UTF-8 response"))?;
    let doc = roxmltree::Document::parse(text)
        .map_err(|_| failed("website preflight: invalid XML (DTDs are not supported)"))?;
    let root = doc.root_element();
    let ns = root.tag_name().namespace();
    if root.tag_name().name() != "WebsiteConfiguration"
        || ns.is_some_and(|n| n != "http://s3.amazonaws.com/doc/2006-03-01/")
    {
        return Err(failed(
            "website preflight: unexpected configuration response",
        ));
    }
    let mut index = false;
    let mut error_document = None;
    for node in root.children().filter(|n| n.is_element()) {
        if node.tag_name().namespace() != ns {
            return Err(failed("website preflight: unexpected namespace"));
        }
        let children = node
            .children()
            .filter(|n| n.is_element())
            .collect::<Vec<_>>();
        match node.tag_name().name() {
            "IndexDocument"
                if !index
                    && children.len() == 1
                    && children[0].tag_name().name() == "Suffix"
                    && children[0].tag_name().namespace() == ns
                    && children[0].text() == Some("index.html") =>
            {
                index = true;
            }
            "ErrorDocument"
                if error_document.is_none()
                    && children.len() == 1
                    && children[0].tag_name().name() == "Key"
                    && children[0].tag_name().namespace() == ns
                    && children[0].text().is_some_and(|s| !s.is_empty()) =>
            {
                error_document = children[0].text().map(str::to_owned);
            }
            "RoutingRules" if children.is_empty() => {}
            _ => {
                return Err(failed(
                    "configure an existing S3 website with index.html and no routing rules or redirect-all setting; zel does not change bucket configuration",
                ));
            }
        }
    }
    if !index {
        return Err(failed(
            "configure the existing S3 website's index document as index.html first",
        ));
    }
    Ok(Website { error_document })
}
pub(crate) struct Target {
    bucket: String,
    mount: String,
    website: Website,
    url: Url,
}
impl Target {
    fn check_prefix(&self, prefix: &str) -> Result<()> {
        let normalized: zellij_core::build_overrides::BasePath =
            prefix.parse().map_err(Error::InvalidArgument)?;
        if normalized.to_string() != prefix
            || (prefix != self.mount && !prefix.starts_with(&format!("{}/", self.mount)))
        {
            return Err(invalid(
                "S3 --base-path must equal or sit below the destination prefix; the website/CloudFront origin must serve the bucket root",
            ));
        }
        Ok(())
    }
    pub fn build_options(
        &self,
        renderer: Renderer,
        options: &BuildOptions,
    ) -> Result<BuildOptions> {
        let mut options = options.clone();
        if renderer == Renderer::Hosted {
            let prefix = options
                .overrides
                .base_path
                .as_ref()
                .map(ToString::to_string)
                .unwrap_or_else(|| self.mount.clone());
            self.check_prefix(&prefix)?;
            options.overrides.base_path = Some(prefix.parse().map_err(Error::InvalidArgument)?);
            if options.overrides.site_url.is_none() {
                options.overrides.site_url = Some(
                    self.url
                        .origin()
                        .ascii_serialization()
                        .parse()
                        .map_err(Error::InvalidArgument)?,
                );
            }
        }
        Ok(options)
    }
    fn destination(&self) -> String {
        format!("{}{}", self.bucket, self.mount)
    }
}
pub(crate) struct Uploaded {
    pub objects: usize,
    pub url: Url,
}
pub(crate) struct Client {
    http: HttpClient,
    base: Url,
    region: String,
    owner: String,
    credentials: Credentials,
    timeout: Duration,
}
struct Reply {
    status: u16,
    headers: HeaderMap,
    body: Vec<u8>,
}
impl Client {
    pub fn from_environment() -> Result<Self> {
        for name in ["AWS_ENDPOINT_URL", "AWS_ENDPOINT_URL_S3"] {
            if std::env::var_os(name).is_some_and(|v| !v.is_empty()) {
                return Err(invalid(format!(
                    "{name} is unsupported; zel only uploads to AWS S3 commercial-region endpoints"
                )));
            }
        }
        let access = required("AWS_ACCESS_KEY_ID")?;
        let secret = required("AWS_SECRET_ACCESS_KEY")?;
        let session = match std::env::var("AWS_SESSION_TOKEN") {
            Ok(s) if !s.is_empty() => Some(s),
            Err(std::env::VarError::NotUnicode(_)) => {
                return Err(invalid("invalid AWS_SESSION_TOKEN"));
            }
            _ => None,
        };
        let credentials = Credentials::new(&access, &secret, session.as_deref())?;
        let region = required("AWS_REGION")?;
        website_endpoint(&region)?;
        let owner = required("S3_EXPECTED_BUCKET_OWNER")?;
        Self::new(
            &format!("https://s3.{region}.amazonaws.com/"),
            &region,
            &owner,
            credentials,
            Duration::from_secs(900),
        )
    }
    // The API origin is fixed in production; local substitution is test-only.
    fn new(
        base: &str,
        region: &str,
        owner: &str,
        credentials: Credentials,
        timeout: Duration,
    ) -> Result<Self> {
        website_endpoint(region)?;
        if owner.len() != 12 || !owner.bytes().all(|b| b.is_ascii_digit()) {
            return Err(invalid(
                "S3_EXPECTED_BUCKET_OWNER must be the bucket owner's 12-digit AWS account ID",
            ));
        }
        let http = HttpClient::builder()
            .redirect(reqwest::redirect::Policy::none())
            .connect_timeout(Duration::from_secs(15))
            .timeout(Duration::from_secs(60))
            .user_agent(concat!("zel/", env!("CARGO_PKG_VERSION")))
            .build()
            .map_err(|_| failed("could not initialize S3 HTTPS client"))?;
        Ok(Self {
            http,
            base: Url::parse(base).map_err(|_| failed("invalid S3 API origin"))?,
            region: region.into(),
            owner: owner.into(),
            credentials,
            timeout,
        })
    }
    fn request(
        &self,
        method: Method,
        path: &str,
        body: Vec<u8>,
        extra: BTreeMap<String, String>,
        deadline: Instant,
    ) -> Result<Reply> {
        let mut url = self.base.clone();
        url.set_path(&encode(path, true));
        if method == Method::GET {
            url.set_query(Some("website="));
        }
        let seconds = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| failed("invalid system clock for AWS signing"))?
            .as_secs();
        let date = chrono::DateTime::from_timestamp(
            i64::try_from(seconds).map_err(|_| failed("invalid system clock"))?,
            0,
        )
        .ok_or_else(|| failed("invalid system clock"))?
        .format("%Y%m%dT%H%M%SZ")
        .to_string();
        let mut headers = extra;
        let host = match url.port() {
            Some(p) => format!("{}:{p}", url.host_str().expect("API origin")),
            None => url.host_str().expect("API origin").into(),
        };
        headers.insert("host".into(), host);
        headers.insert("x-amz-content-sha256".into(), hash(&body));
        headers.insert("x-amz-date".into(), date.clone());
        headers.insert("x-amz-expected-bucket-owner".into(), self.owner.clone());
        if let Some(token) = &self.credentials.session {
            headers.insert("x-amz-security-token".into(), token.clone());
        }
        let auth = self
            .credentials
            .authorization(&self.region, &date, &method, &url, &headers);
        headers.insert("authorization".into(), auth);
        let mut map = HeaderMap::new();
        for (key, value) in headers {
            let mut value =
                HeaderValue::from_str(&value).map_err(|_| failed("invalid S3 request header"))?;
            if matches!(key.as_str(), "authorization" | "x-amz-security-token") {
                value.set_sensitive(true);
            }
            map.insert(
                header::HeaderName::from_bytes(key.as_bytes()).expect("static header names"),
                value,
            );
        }
        let remaining = deadline
            .checked_duration_since(Instant::now())
            .ok_or_else(|| {
                failed("S3 operation timed out; inspect the destination before retrying")
            })?;
        let response = self
            .http
            .request(method.clone(), url)
            .headers(map)
            .body(body)
            .timeout(remaining.min(Duration::from_secs(60)))
            .send()
            .map_err(|e| {
                failed(if e.is_timeout() {
                    "S3 request timed out"
                } else {
                    "S3 HTTPS request failed"
                })
            })?;
        let status = response.status().as_u16();
        let headers = response.headers().clone();
        if !(200..300).contains(&status)
            && !(status == 404 && method == Method::HEAD && path.matches('/').count() > 1)
        {
            let hint = match status {
                301 | 307 => "wrong AWS_REGION or endpoint; redirects are refused",
                401 | 403 => {
                    "check environment credentials, expected owner, bucket policy and IAM permissions"
                }
                404 => "check the bucket and its existing website configuration",
                409 | 412 => {
                    "an object changed concurrently; inspect the destination before retrying"
                }
                429 | 503 => "S3 throttled the request; inspect the destination and retry later",
                _ => "inspect the destination before retrying",
            };
            return Err(failed(format!("S3 HTTP {status}: {hint}")));
        }
        let mut bytes = Vec::new();
        response
            .take(RESPONSE_LIMIT + 1)
            .read_to_end(&mut bytes)
            .map_err(|e| {
                let timed_out = Instant::now() >= deadline
                    || e.kind() == std::io::ErrorKind::TimedOut
                    || e.get_ref()
                        .and_then(|v| v.downcast_ref::<reqwest::Error>())
                        .is_some_and(reqwest::Error::is_timeout);
                failed(if timed_out {
                    "S3 response read timed out"
                } else {
                    "could not read S3 response"
                })
            })?;
        if bytes.len() as u64 > RESPONSE_LIMIT {
            return Err(failed("S3 response exceeded 256 KiB"));
        }
        Ok(Reply {
            status,
            headers,
            body: bytes,
        })
    }
    pub fn check_site(&self, destination: &str) -> Result<Target> {
        self.check_site_at(destination, Instant::now() + self.timeout)
    }
    fn check_site_at(&self, destination: &str, deadline: Instant) -> Result<Target> {
        Provider::S3.validate_site(destination)?;
        let (bucket, prefix) = destination.split_once('/').unwrap_or((destination, ""));
        let path = format!("/{bucket}");
        let head = self.request(Method::HEAD, &path, Vec::new(), BTreeMap::new(), deadline)?;
        if head.status != 200
            || head
                .headers
                .get("x-amz-bucket-region")
                .and_then(|v| v.to_str().ok())
                != Some(self.region.as_str())
        {
            return Err(failed(
                "S3 preflight returned an unexpected bucket region; check AWS_REGION",
            ));
        }
        let reply = self.request(Method::GET, &path, Vec::new(), BTreeMap::new(), deadline)?;
        if reply.status != 200 {
            return Err(failed("S3 website preflight returned an unexpected status"));
        }
        let website = website(&reply.body)?;
        let mount = if prefix.is_empty() {
            String::new()
        } else {
            format!("/{prefix}")
        };
        let url = Url::parse(&format!(
            "http://{bucket}.{}/",
            website_endpoint(&self.region)?
        ))
        .map_err(|_| failed("invalid S3 website URL"))?;
        let mut url = url;
        url.set_path(&format!("{mount}/"));
        Ok(Target {
            bucket: bucket.into(),
            mount,
            website,
            url,
        })
    }
    pub fn deploy(
        &self,
        target: &Target,
        root: &Utf8Path,
        renderer: Renderer,
        prefix: &str,
    ) -> Result<Uploaded> {
        if renderer == Renderer::Hosted {
            target.check_prefix(prefix)?;
        } else if !prefix.is_empty() {
            return Err(failed("portable build returned an unexpected prefix"));
        }
        let manifest = Manifest::read(root, target, renderer, prefix)?;
        let deadline = Instant::now() + self.timeout;
        // Capture all object preconditions before the first write. A changed key
        // fails its conditional PUT. S3 cannot atomically replace a whole site.
        let mut conditions = BTreeMap::new();
        for asset in &manifest.assets {
            let path = format!("/{}/{}", target.bucket, asset.key);
            let head = self.request(Method::HEAD, &path, Vec::new(), BTreeMap::new(), deadline)?;
            let condition = match head.status {
                404 => ("if-none-match", "*".to_owned()),
                200 => ("if-match", etag(&head.headers)?.to_owned()),
                _ => {
                    return Err(failed(
                        "unexpected object preflight status; nothing uploaded",
                    ));
                }
            };
            conditions.insert(asset.key.clone(), condition);
        }
        let check = self.check_site_at(&target.destination(), deadline)?;
        if check.website != target.website {
            return Err(failed(
                "S3 website configuration changed during build; nothing uploaded",
            ));
        }
        for asset in &manifest.assets {
            let bytes = read_asset(&asset.path)?;
            if hash(&bytes) != asset.digest {
                return Err(failed(
                    "generated output changed during upload; some objects may already have changed",
                ));
            }
            let checksum = STANDARD.encode(Sha256::digest(&bytes));
            let mut headers = BTreeMap::from([
                ("content-type".into(), asset.content_type.clone()),
                ("cache-control".into(), "no-cache".into()),
                ("x-amz-checksum-sha256".into(), checksum.clone()),
            ]);
            let (name, value) = &conditions[&asset.key];
            headers.insert((*name).into(), value.clone());
            let result = self
                .request(
                    Method::PUT,
                    &format!("/{}/{}", target.bucket, asset.key),
                    bytes,
                    headers,
                    deadline,
                )
                .map_err(|e| {
                    failed(format!(
                        "{e}; upload is non-atomic and may be partial; no automatic retry or rollback was attempted"
                    ))
                })?;
            if result.status != 200
                || !result.body.is_empty()
                || result
                    .headers
                    .get("x-amz-checksum-sha256")
                    .and_then(|v| v.to_str().ok())
                    != Some(checksum.as_str())
                || etag(&result.headers).is_err()
            {
                return Err(failed(
                    "S3 upload returned an unexpected status/checksum/ETag; some objects may already have changed; inspect before retrying",
                ));
            }
        }
        let check = self.check_site_at(&target.destination(), deadline)?;
        if check.website != target.website {
            return Err(failed(
                "S3 website configuration changed while uploading; inspect the destination",
            ));
        }
        let mut url = target.url.clone();
        if renderer == Renderer::Hosted {
            url.set_path(&format!("{prefix}/"));
        }
        Ok(Uploaded {
            objects: manifest.assets.len(),
            url,
        })
    }
}
pub(crate) fn validate_credentials(
    access: &str,
    secret: &str,
    session: Option<&str>,
) -> Result<()> {
    Credentials::new(access, secret, session).map(|_| ())
}
fn etag(headers: &HeaderMap) -> Result<&str> {
    headers
        .get(header::ETAG)
        .and_then(|v| v.to_str().ok())
        .filter(|v| {
            v.len() >= 3
                && v.len() <= 1024
                && v.starts_with('"')
                && v.ends_with('"')
                && v[1..v.len() - 1]
                    .bytes()
                    .all(|b| b.is_ascii_graphic() && b != b'"')
        })
        .ok_or_else(|| failed("S3 returned an invalid or missing ETag"))
}
fn read_asset(path: &Utf8Path) -> Result<Vec<u8>> {
    let meta = std::fs::symlink_metadata(path)?;
    if zellij_core::site::is_link(&meta) || !meta.is_file() {
        return Err(failed(
            "S3 export cannot contain links, junctions or non-regular files",
        ));
    }
    if meta.len() > MAX_FILE {
        return Err(failed("S3 adapter files must be at most 25 MiB"));
    }
    let mut bytes = Vec::new();
    std::fs::File::open(path)?
        .take(MAX_FILE + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() as u64 > MAX_FILE {
        return Err(failed("S3 adapter files must be at most 25 MiB"));
    }
    Ok(bytes)
}
struct Asset {
    path: Utf8PathBuf,
    key: String,
    digest: String,
    content_type: String,
    rank: u8,
}
struct Manifest {
    assets: Vec<Asset>,
}
impl Manifest {
    fn read(root: &Utf8Path, target: &Target, renderer: Renderer, prefix: &str) -> Result<Self> {
        let mut files = BTreeMap::new();
        let mut total = 0u64;
        let index = if renderer == Renderer::Hosted {
            format!("{}/index.html", prefix.trim_start_matches('/'))
        } else {
            format!("{}/index.html", target.mount.trim_start_matches('/'))
        };
        let index = index.trim_start_matches('/');
        for entry in walkdir::WalkDir::new(root)
            .follow_links(false)
            .sort_by_file_name()
        {
            let entry = entry.map_err(|_| failed("could not enumerate S3 export"))?;
            let meta = std::fs::symlink_metadata(entry.path())?;
            if zellij_core::site::is_link(&meta) {
                return Err(failed("S3 export cannot contain links or junctions"));
            }
            if meta.is_dir() {
                continue;
            }
            let path = Utf8Path::from_path(entry.path())
                .ok_or_else(|| failed("S3 export path is not UTF-8"))?;
            let relative = path
                .strip_prefix(root)
                .map_err(|_| failed("file outside S3 export"))?;
            if relative == ".zellij-output" {
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
                    "S3 asset paths cannot contain #, ?, backslashes or control characters",
                ));
            }
            let key = if renderer == Renderer::Portable && !target.mount.is_empty() {
                format!("{}/{name}", target.mount.trim_start_matches('/'))
            } else {
                name
            };
            if !target.mount.is_empty()
                && !key.starts_with(&format!("{}/", target.mount.trim_start_matches('/')))
            {
                return Err(failed(
                    "hosted export sits outside the S3 destination prefix",
                ));
            }
            if key.len() > 1024 {
                return Err(failed("S3 object keys must be at most 1,024 UTF-8 bytes"));
            }
            let bytes = read_asset(path)?;
            let digest = hash(&bytes);
            let html = path.extension() == Some("html");
            let content_type = mime_guess::from_path(path)
                .first_or_octet_stream()
                .to_string();
            let mut keys = vec![key.clone()];
            if renderer == Renderer::Hosted && html && path.file_name() != Some("index.html") {
                keys.push(key.strip_suffix(".html").expect("HTML extension").into());
            }
            for key in keys {
                total += bytes.len() as u64;
                if total > MAX_TOTAL {
                    return Err(failed(
                        "S3 uploads support at most 100 MiB including clean-route copies",
                    ));
                }
                let rank = if key == index {
                    2
                } else if html {
                    1
                } else {
                    0
                };
                let asset = Asset {
                    path: path.into(),
                    key: key.clone(),
                    digest: digest.clone(),
                    content_type: content_type.clone(),
                    rank,
                };
                if files.insert(key, asset).is_some() {
                    return Err(failed(
                        "S3 clean-route object collides with another generated file",
                    ));
                }
                if files.len() > MAX_OBJECTS {
                    return Err(failed(
                        "S3 uploads support at most 5,000 objects including clean-route copies",
                    ));
                }
            }
        }
        if !files.contains_key(index) {
            return Err(failed("S3 export has no index.html at its selected prefix"));
        }
        let mut assets = files.into_values().collect::<Vec<_>>();
        assets.sort_by(|a, b| a.rank.cmp(&b.rank).then(a.key.cmp(&b.key)));
        Ok(Self { assets })
    }
}
#[cfg(test)]
#[path = "s3_tests.rs"]
mod tests;
