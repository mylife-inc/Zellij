use reqwest::{Url, blocking::Client, redirect::Policy};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, HashSet},
    fs::{self, File},
    io::{Read, Write},
    path::{Path, PathBuf},
    time::Duration,
};
use zellij_core::{Error, Result};
const VERSION: &str = env!("CARGO_PKG_VERSION");
const ARCHIVE_LIMIT: u64 = 160 * 1024 * 1024;
const EXPANDED_LIMIT: u64 = 512 * 1024 * 1024;
fn fail(message: impl Into<String>) -> Error {
    Error::InvalidArgument(format!("Rich renderer: {}", message.into()))
}

pub(super) fn platform() -> Result<String> {
    let os = match std::env::consts::OS {
        "macos" => "darwin",
        "windows" => "win",
        "linux" => "linux",
        _ => return Err(fail("this operating system has no published rich renderer")),
    };
    let arch = match std::env::consts::ARCH {
        "x86_64" => "x64",
        "aarch64" => "arm64",
        _ => return Err(fail("this architecture has no published rich renderer")),
    };
    Ok(format!("{os}-{arch}"))
}
fn root() -> Result<PathBuf> {
    if let Some(value) = std::env::var_os("ZEL_ENGINE_CACHE").filter(|value| !value.is_empty()) {
        return Ok(PathBuf::from(value));
    }
    Ok(crate::user_home::root()?.join("cache/engines"))
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    format: u32,
    version: String,
    platform: String,
    node_version: String,
    files: BTreeMap<String, String>,
}
fn safe_name(name: &str) -> bool {
    !name.is_empty()
        && !name
            .chars()
            .any(|c| c.is_control() || matches!(c, '\\' | ':' | '<' | '>' | '"' | '|' | '?' | '*'))
        && name.split('/').all(|p| {
            let stem = p.split('.').next().unwrap_or("").to_ascii_uppercase();
            !p.is_empty()
                && p != "."
                && p != ".."
                && !p.ends_with(['.', ' '])
                && !matches!(
                    stem.as_str(),
                    "CON"
                        | "PRN"
                        | "AUX"
                        | "NUL"
                        | "COM1"
                        | "COM2"
                        | "COM3"
                        | "COM4"
                        | "COM5"
                        | "COM6"
                        | "COM7"
                        | "COM8"
                        | "COM9"
                        | "LPT1"
                        | "LPT2"
                        | "LPT3"
                        | "LPT4"
                        | "LPT5"
                        | "LPT6"
                        | "LPT7"
                        | "LPT8"
                        | "LPT9"
                )
        })
}
fn regular(path: &Path) -> Result<()> {
    let metadata = fs::symlink_metadata(path)?;
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        return Err(fail("cached engine contains a link or non-file"));
    }
    Ok(())
}
fn digest(path: &Path) -> Result<String> {
    regular(path)?;
    let mut file = File::open(path)?;
    let mut hash = Sha256::new();
    let mut buffer = [0; 65536];
    loop {
        let n = file.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        hash.update(&buffer[..n]);
    }
    Ok(format!("{:x}", hash.finalize()))
}
fn validate(root: &Path, target: &str) -> Result<()> {
    if fs::symlink_metadata(root)?.file_type().is_symlink() {
        return Err(fail("engine cache must not be a symlink"));
    }
    regular(&root.join("manifest.json"))?;
    if fs::metadata(root.join("manifest.json"))?.len() > 1024 * 1024 {
        return Err(fail("cached manifest exceeds its size limit"));
    }
    let manifest: Manifest = serde_json::from_slice(&fs::read(root.join("manifest.json"))?)
        .map_err(|_| fail("invalid cached manifest"))?;
    if manifest.format != 1
        || manifest.version != VERSION
        || manifest.platform != target
        || !manifest.node_version.starts_with("v24.")
        || manifest.files.len() > 10000
    {
        return Err(fail("cached engine is incompatible with this CLI/platform"));
    }
    for required in [
        if cfg!(windows) { "node.exe" } else { "node" },
        "engine/engine.mjs",
        "engine/runtime.js",
        "engine/styles.css",
    ] {
        if !manifest.files.contains_key(required) {
            return Err(fail(format!("engine is missing {required}")));
        }
    }
    for (name, expected) in &manifest.files {
        if !safe_name(name)
            || expected.len() != 64
            || !expected.bytes().all(|b| b.is_ascii_hexdigit())
        {
            return Err(fail("invalid engine file manifest"));
        }
        let mut current = root.to_owned();
        for component in name.split('/') {
            current.push(component);
            if fs::symlink_metadata(&current)?.file_type().is_symlink() {
                return Err(fail("engine cache contains a symlink"));
            }
        }
        if digest(&current)? != *expected {
            return Err(fail(format!(
                "cached file {name} failed integrity verification; remove {} and retry",
                root.display()
            )));
        }
    }
    Ok(())
}
fn base_url() -> Result<Url> {
    let value = std::env::var("ZEL_ENGINE_BASE_URL").unwrap_or_else(|_| {
        format!("https://github.com/mylife-inc/releases/releases/download/zellij-v{VERSION}")
    });
    let url = Url::parse(&(value.trim_end_matches('/').to_owned() + "/"))
        .map_err(|_| fail("invalid ZEL_ENGINE_BASE_URL"))?;
    if !(url.scheme() == "https"
        || url.scheme() == "http"
            && matches!(url.host_str(), Some("localhost" | "127.0.0.1" | "[::1]")))
        || !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
    {
        return Err(fail(
            "engine URL must use HTTPS (or loopback HTTP) without credentials, query or fragment",
        ));
    }
    Ok(url)
}
fn download(client: &Client, url: Url, limit: u64, output: &mut impl Write) -> Result<()> {
    let response = client.get(url).send().map_err(|_| {
        fail("download failed; check connectivity or use a populated cache with --offline")
    })?;
    if !response.status().is_success() {
        return Err(fail(format!(
            "download returned HTTP {}; install a released CLI with matching renderer assets",
            response.status()
        )));
    }
    let bytes = std::io::copy(&mut response.take(limit + 1), output)?;
    if bytes > limit {
        return Err(fail("download exceeds its size limit"));
    }
    Ok(())
}
fn extract(archive: &Path, into: &Path) -> Result<()> {
    let mut reader = tar::Archive::new(
        flate2::read::GzDecoder::new(File::open(archive)?).take(EXPANDED_LIMIT + 1),
    );
    let mut total = 0u64;
    let mut names = HashSet::new();
    for entry in reader.entries()? {
        let mut entry = entry?;
        let name = entry
            .path()?
            .to_str()
            .ok_or_else(|| fail("archive path is not UTF-8"))?
            .to_owned();
        if !entry.header().entry_type().is_file()
            || !safe_name(&name)
            || !names.insert(name.to_lowercase())
        {
            return Err(fail(
                "archive contains unsafe, duplicate or non-file entries",
            ));
        }
        total = total
            .checked_add(entry.size())
            .ok_or_else(|| fail("archive size overflow"))?;
        if total > EXPANDED_LIMIT || names.len() > 10000 {
            return Err(fail("archive exceeds its expansion limits"));
        }
        let dest = into.join(&name);
        fs::create_dir_all(dest.parent().unwrap())?;
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&dest)?;
        if std::io::copy(&mut entry, &mut file)? != entry.size() {
            return Err(fail("truncated engine archive"));
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            file.set_permissions(fs::Permissions::from_mode(if name == "node" {
                0o755
            } else {
                0o644
            }))?;
        }
    }
    let mut decoder = reader.into_inner();
    std::io::copy(&mut decoder, &mut std::io::sink())?;
    if decoder.limit() == 0 {
        return Err(fail("archive expansion limit exceeded"));
    }
    Ok(())
}
pub(crate) fn inspect() -> Result<serde_json::Value> {
    let platform = platform()?;
    let installed = root()?.join(format!("{VERSION}-{platform}"));
    let cached = installed.exists();
    if cached {
        validate(&installed, &platform)?;
    }
    Ok(
        serde_json::json!({"status":if cached {"cached"} else {"download_required"},"version":VERSION,"platform":platform,"cache":installed.to_string_lossy()}),
    )
}
pub(super) fn obtain(offline: bool) -> Result<PathBuf> {
    let platform = platform()?;
    let cache = root()?;
    let installed = cache.join(format!("{VERSION}-{platform}"));
    if installed.exists() {
        validate(&installed, &platform)?;
        return fs::canonicalize(installed).map_err(Into::into);
    }
    if offline {
        return Err(fail(
            "engine is not cached; run this build once with internet access before using --offline",
        ));
    }
    fs::create_dir_all(&cache)?;
    let stage = tempfile::Builder::new()
        .prefix(".download-")
        .tempdir_in(&cache)?;
    let base = base_url()?;
    let origin = base.origin();
    let client = Client::builder()
        .user_agent(concat!("zel/", env!("CARGO_PKG_VERSION")))
        .connect_timeout(Duration::from_secs(15))
        .timeout(Duration::from_secs(300))
        .redirect(Policy::custom(move |attempt| {
            let url = attempt.url();
            if attempt.previous().len() < 5
                && url.username().is_empty()
                && url.password().is_none()
                && (url.scheme() == "https" || url.scheme() == "http" && url.origin() == origin)
            {
                attempt.follow()
            } else {
                attempt.stop()
            }
        }))
        .build()
        .map_err(|_| fail("could not initialize HTTPS client"))?;
    let name = format!("zel-renderer-v{VERSION}-{platform}.tar.gz");
    let mut checksum = Vec::new();
    download(
        &client,
        base.join(&(name.clone() + ".sha256")).unwrap(),
        1024,
        &mut checksum,
    )?;
    let checksum = String::from_utf8(checksum).map_err(|_| fail("invalid archive checksum"))?;
    let fields: Vec<_> = checksum.split_whitespace().collect();
    if fields.len() != 2
        || fields[1] != name
        || fields[0].len() != 64
        || !fields[0].bytes().all(|c| c.is_ascii_hexdigit())
    {
        return Err(fail("invalid archive checksum"));
    }
    zellij_core::statusln!(
        "  Downloading rich renderer {VERSION} ({platform}); later builds use the cache."
    );
    let archive = stage.path().join("download.tar.gz");
    download(
        &client,
        base.join(&name).unwrap(),
        ARCHIVE_LIMIT,
        &mut File::create(&archive)?,
    )?;
    if digest(&archive)? != fields[0] {
        return Err(fail(
            "download checksum mismatch; engine was not installed or executed",
        ));
    }
    let unpacked = stage.path().join("engine");
    fs::create_dir(&unpacked)?;
    extract(&archive, &unpacked)?;
    validate(&unpacked, &platform)?;
    if let Err(error) = fs::rename(&unpacked, &installed) {
        if installed.exists() {
            validate(&installed, &platform)?;
        } else {
            return Err(error.into());
        }
    }
    fs::canonicalize(installed).map_err(Into::into)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn portable_archive_paths_reject_traversal_and_windows_aliases() {
        for name in [
            "",
            "/absolute",
            "../escape",
            "a/../b",
            "./a",
            "a//b",
            "a\\b",
            "C:/file",
            "node:stream",
            "a\nfile",
            "AUX.txt",
            "a/COM1",
            "a/name.",
            "a/name ",
        ] {
            assert!(!safe_name(name), "{name:?}");
        }
        for name in [
            "node",
            "node.exe",
            "engine/files/inter.woff2",
            "engine/engine.mjs",
        ] {
            assert!(safe_name(name));
        }
    }
    fn archive(path: &Path, link: bool, duplicate: bool) {
        let gzip = flate2::write::GzEncoder::new(
            File::create(path).unwrap(),
            flate2::Compression::default(),
        );
        let mut tar = tar::Builder::new(gzip);
        let mut header = tar::Header::new_gnu();
        header.set_mode(0o644);
        header.set_size(if link { 0 } else { 3 });
        if link {
            header.set_entry_type(tar::EntryType::Symlink);
            header.set_link_name("../outside").unwrap();
        }
        header.set_cksum();
        tar.append_data(
            &mut header,
            "engine/file",
            if link { &b""[..] } else { &b"abc"[..] },
        )
        .unwrap();
        if duplicate {
            let mut header = tar::Header::new_gnu();
            header.set_mode(0o644);
            header.set_size(3);
            header.set_cksum();
            tar.append_data(&mut header, "ENGINE/file", &b"abc"[..])
                .unwrap();
        }
        tar.into_inner().unwrap().finish().unwrap();
    }
    #[test]
    fn extraction_rejects_links_and_case_collisions() {
        for (link, duplicate) in [(true, false), (false, true), (false, false)] {
            let dir = tempfile::tempdir().unwrap();
            let input = dir.path().join("archive.tar.gz");
            let output = dir.path().join("out");
            fs::create_dir(&output).unwrap();
            archive(&input, link, duplicate);
            assert_eq!(extract(&input, &output).is_ok(), !link && !duplicate);
            assert!(!dir.path().join("outside").exists());
        }
    }
    #[test]
    fn cache_validation_requires_matching_version_and_complete_file_hashes() {
        let dir = tempfile::tempdir().unwrap();
        let target = platform().unwrap();
        let node = if cfg!(windows) { "node.exe" } else { "node" };
        let mut files = BTreeMap::new();
        for name in [
            node,
            "engine/engine.mjs",
            "engine/runtime.js",
            "engine/styles.css",
        ] {
            let file = dir.path().join(name);
            fs::create_dir_all(file.parent().unwrap()).unwrap();
            fs::write(&file, name).unwrap();
            files.insert(name, digest(&file).unwrap());
        }
        let mut manifest = serde_json::json!({"format":1,"version":VERSION,"platform":target,"node_version":"v24.22.3","files":files});
        let file = dir.path().join("manifest.json");
        fs::write(&file, manifest.to_string()).unwrap();
        validate(dir.path(), &target).unwrap();
        manifest["version"] = serde_json::json!("0.0.0");
        fs::write(&file, manifest.to_string()).unwrap();
        assert!(validate(dir.path(), &target).is_err());
        manifest["version"] = serde_json::json!(VERSION);
        fs::write(&file, manifest.to_string()).unwrap();
        fs::write(dir.path().join(node), "changed").unwrap();
        assert!(validate(dir.path(), &target).is_err());
    }
}
