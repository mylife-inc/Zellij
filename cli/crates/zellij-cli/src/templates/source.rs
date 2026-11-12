use super::*;
use reqwest::{Url, blocking::Client, redirect::Policy};
use serde::{Deserialize, Serialize};
use std::time::Duration;
const CATALOG_LIMIT: usize = 1024 * 1024;
const ARCHIVE_LIMIT: usize = 16 * 1024 * 1024;

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Catalog {
    pub schema: u32,
    pub release: String,
    pub templates: Vec<Entry>,
}
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Entry {
    pub manifest: Manifest,
    pub archive: Archive,
}
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Archive {
    pub url: String,
    pub sha256: String,
}
fn url(value: &str) -> Result<Url> {
    let url =
        Url::parse(value).map_err(|_| invalid("catalog/archive needs an absolute HTTPS URL"))?;
    let local = url.scheme() == "http"
        && url
            .host_str()
            .is_some_and(|h| matches!(h, "127.0.0.1" | "[::1]"));
    ensure(
        (url.scheme() == "https" || local)
            && url.host_str().is_some()
            && url.username().is_empty()
            && url.password().is_none()
            && url.query().is_none()
            && url.fragment().is_none(),
        "catalog/archive URL must use HTTPS without credentials, query or fragment (HTTP loopback is allowed for local testing)",
    )?;
    Ok(url)
}
fn archive_url(catalog: &str, value: &str) -> Result<Url> {
    if value.contains("://") {
        return url(value);
    }
    manifest::path(value)?;
    ensure(
        value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b)),
        "relative archive URLs must be simple filenames",
    )?;
    url(url(catalog)?
        .join(value)
        .map_err(|_| invalid("invalid archive filename"))?
        .as_str())
}
fn download(value: &str, limit: usize) -> Result<Vec<u8>> {
    let url = url(value)?;
    let origin = url.origin();
    let client = Client::builder()
        .user_agent(concat!("zel/", env!("CARGO_PKG_VERSION")))
        .connect_timeout(Duration::from_secs(10))
        .timeout(Duration::from_secs(60))
        .redirect(Policy::custom(move |a| {
            let u = a.url();
            if a.previous().len() < 5
                && u.username().is_empty()
                && u.password().is_none()
                && (u.scheme() == "https" || u.scheme() == "http" && u.origin() == origin)
            {
                a.follow()
            } else {
                a.stop()
            }
        }))
        .build()
        .map_err(|_| invalid("could not initialize template download"))?;
    let response = client.get(url).send().map_err(|_| {
        invalid("template download failed; use --offline with a previously cached catalog/archive")
    })?;
    ensure(
        response.status().is_success(),
        &format!("template download returned HTTP {}", response.status()),
    )?;
    ensure(
        response.content_length().is_none_or(|n| n <= limit as u64),
        "template download exceeds its size limit",
    )?;
    limited(response, limit)
}
fn regular(path: &Utf8Path, limit: usize) -> Result<Vec<u8>> {
    let meta = fs::symlink_metadata(path)?;
    ensure(
        !zellij_core::site::is_link(&meta) && meta.is_file(),
        "template/cache entries must be regular files, not links",
    )?;
    ensure(
        meta.len() <= limit as u64,
        "template/cache file exceeds size limit",
    )?;
    limited(fs::File::open(path)?, limit)
}
fn cache_root(options: &TemplateSourceArgs) -> Result<Utf8PathBuf> {
    if let Some(root) = &options.cache_dir {
        ensure(!root.is_empty(), "--cache-dir must not be empty")?;
        return Ok(root.into());
    }
    let root = if let Some(path) = std::env::var_os("XDG_CACHE_HOME").filter(|p| !p.is_empty()) {
        std::path::PathBuf::from(path)
    } else if cfg!(windows) {
        std::env::var_os("LOCALAPPDATA")
            .map(std::path::PathBuf::from)
            .ok_or_else(|| invalid("set --cache-dir or LOCALAPPDATA"))?
    } else {
        let home = std::env::var_os("HOME").ok_or_else(|| invalid("set --cache-dir or HOME"))?;
        std::path::PathBuf::from(home).join(if cfg!(target_os = "macos") {
            "Library/Caches"
        } else {
            ".cache"
        })
    };
    ensure(
        root.is_absolute(),
        "OS cache directory must be absolute; use --cache-dir",
    )?;
    Utf8PathBuf::from_path_buf(root.join("zel/templates"))
        .map_err(|_| invalid("cache path is not UTF-8; use --cache-dir"))
}
fn cache_path(options: &TemplateSourceArgs, name: &str) -> Result<Utf8PathBuf> {
    let root = cache_root(options)?;
    if let Ok(meta) = fs::symlink_metadata(&root) {
        ensure(
            !zellij_core::site::is_link(&meta) && meta.is_dir(),
            "cache root must be a directory, not a link",
        )?;
    }
    files::contained_path(&root, Utf8Path::new(name))
}
fn save(path: &Utf8Path, bytes: &[u8]) -> Result<()> {
    let parent = path.parent().expect("cache file has parent");
    fs::create_dir_all(parent)?;
    let mut temp = tempfile::NamedTempFile::new_in(parent)?;
    temp.write_all(bytes)?;
    temp.as_file().sync_all()?;
    temp.persist(path).map_err(|e| e.error)?;
    Ok(())
}
fn digest(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}
pub(super) fn catalog(location: &str, options: &TemplateSourceArgs) -> Result<Catalog> {
    url(location)?;
    let path = cache_path(
        options,
        &format!("catalog-{}.json", sha(location.as_bytes())),
    )?;
    let bytes = if options.offline {
        regular(&path, CATALOG_LIMIT)
            .map_err(|_| invalid("catalog is not available in the offline cache"))?
    } else {
        download(location, CATALOG_LIMIT)?
    };
    let result: Catalog =
        serde_json::from_slice(&bytes).map_err(|_| invalid("invalid template catalog JSON"))?;
    ensure(
        result.schema == 1 && !result.templates.is_empty() && result.templates.len() <= 128,
        "unsupported/empty/oversized catalog",
    )?;
    manifest::version(
        result
            .release
            .strip_prefix('v')
            .ok_or_else(|| invalid("catalog release must start with v"))?,
    )?;
    let mut ids = BTreeSet::new();
    for entry in &result.templates {
        entry.manifest.validate()?;
        archive_url(location, &entry.archive.url)?;
        ensure(
            digest(&entry.archive.sha256)
                && ids.insert((&entry.manifest.id, &entry.manifest.version)),
            "invalid checksum or duplicate catalog version",
        )?;
    }
    if !options.offline {
        save(&path, &bytes)?;
    }
    Ok(result)
}
pub(super) fn remote(
    id: &str,
    selected: Option<&str>,
    location: &str,
    options: &TemplateSourceArgs,
) -> Result<Bundle> {
    let mut entries = catalog(location, options)?
        .templates
        .into_iter()
        .filter(|e| e.manifest.id == id && selected.is_none_or(|v| v == e.manifest.version))
        .collect::<Vec<_>>();
    entries.sort_by_key(|e| manifest::version(&e.manifest.version).expect("validated version"));
    let entry = entries
        .pop()
        .ok_or_else(|| invalid("template/version is not in the selected catalog"))?;
    let path = cache_path(options, &format!("archive-{}.tar.gz", entry.archive.sha256))?;
    let cached = fs::symlink_metadata(&path).is_ok();
    let bytes = if cached {
        regular(&path, ARCHIVE_LIMIT)?
    } else {
        ensure(
            !options.offline,
            "template archive is not available in the offline cache",
        )?;
        download(
            archive_url(location, &entry.archive.url)?.as_str(),
            ARCHIVE_LIMIT,
        )?
    };
    ensure(
        sha(&bytes) == entry.archive.sha256,
        "template archive SHA-256 mismatch; nothing was created (remove a corrupt cache entry before retrying)",
    )?;
    let mut bundle = archive(&bytes, location.into())?;
    ensure(
        bundle.manifest == entry.manifest,
        "archive manifest differs from its catalog entry",
    )?;
    // Receipt identifies both the exact archive and its source catalog.
    bundle.digest = entry.archive.sha256;
    if !cached {
        save(&path, &bytes)?;
    }
    Ok(bundle)
}
pub(super) fn archive(bytes: &[u8], origin: String) -> Result<Bundle> {
    ensure(
        bytes.len() <= ARCHIVE_LIMIT,
        "template archive is oversized",
    )?;
    let gz =
        flate2::bufread::GzDecoder::new(bytes).take((TOTAL_LIMIT + COUNT_LIMIT * 2048) as u64 + 1);
    let mut archive = tar::Archive::new(gz);
    let mut entries = BTreeMap::new();
    let mut total = 0;
    for item in archive
        .entries()
        .map_err(|_| invalid("invalid template archive"))?
    {
        let mut entry = item.map_err(|_| invalid("invalid template archive entry"))?;
        let name = std::str::from_utf8(entry.path_bytes().as_ref())
            .map_err(|_| invalid("template archive paths must be UTF-8"))?
            .to_owned();
        manifest::path(&name)?;
        ensure(
            entry.header().entry_type().is_file(),
            "template archives may contain only regular files; links, directories and special entries are refused",
        )?;
        ensure(
            entry.size() <= FILE_LIMIT as u64 && entries.len() < COUNT_LIMIT,
            "template archive exceeds entry limits",
        )?;
        let body = limited(&mut entry, FILE_LIMIT)?;
        total += body.len();
        ensure(
            total <= TOTAL_LIMIT && !entries.contains_key(&name),
            "template archive is oversized or has duplicate paths",
        )?;
        entries.insert(name, body);
    }
    // Read to EOF to validate the gzip CRC; reject hidden trailing tar data too.
    let mut decoded = archive.into_inner();
    let tail = limited(&mut decoded, TOTAL_LIMIT)?;
    ensure(
        decoded.limit() > 0 && tail.iter().all(|b| *b == 0),
        "unexpected/oversized data after template archive",
    )?;
    ensure(
        decoded.into_inner().into_inner().is_empty(),
        "trailing compressed data after template archive",
    )?;
    Bundle::from_entries(entries, origin)
}
pub(super) fn local(root: &Utf8Path) -> Result<Bundle> {
    let meta = fs::symlink_metadata(root)?;
    ensure(
        meta.is_dir() && !zellij_core::site::is_link(&meta),
        "local template must be a directory, not a link",
    )?;
    let mut entries = BTreeMap::new();
    let mut total = 0;
    for item in walkdir::WalkDir::new(root)
        .follow_links(false)
        .max_depth(33)
        .min_depth(1)
    {
        let entry = item.map_err(|_| invalid("cannot read local template"))?;
        ensure(
            entry.depth() <= 32
                && !zellij_core::site::is_link(&fs::symlink_metadata(entry.path())?),
            "local template is too deep or contains a link",
        )?;
        let relative = entry
            .path()
            .strip_prefix(root)
            .expect("walk root")
            .to_str()
            .ok_or_else(|| invalid("template paths must be UTF-8"))?
            .replace(std::path::MAIN_SEPARATOR, "/");
        manifest::path(&relative)?;
        ensure(
            relative == "files" || relative == "template.yaml" || relative.starts_with("files/"),
            "local template may contain only template.yaml and files/",
        )?;
        if entry.file_type().is_dir() {
            ensure(
                relative != "template.yaml",
                "template.yaml must be a regular file",
            )?;
            continue;
        }
        ensure(
            entry.file_type().is_file() && entries.len() < COUNT_LIMIT,
            "local template has too many files or a special entry",
        )?;
        let body = regular(
            Utf8Path::from_path(entry.path())
                .ok_or_else(|| invalid("template path is not UTF-8"))?,
            FILE_LIMIT,
        )?;
        total += body.len();
        ensure(
            total <= TOTAL_LIMIT,
            "local template exceeds total size limit",
        )?;
        entries.insert(relative, body);
    }
    Bundle::from_entries(entries, "local".into())
}
