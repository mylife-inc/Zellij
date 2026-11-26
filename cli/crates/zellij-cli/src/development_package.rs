//! Verified, platform-specific Next development dependencies. No package manager runs on the user machine.
use camino::{Utf8Path, Utf8PathBuf};
use reqwest::Url;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, File},
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::atomic::{AtomicBool, Ordering},
};
use zellij_core::{Error, Result, files, site::is_link};
const VERSION: &str = env!("CARGO_PKG_VERSION");
const ARCHIVE_LIMIT: u64 = 512 * 1024 * 1024;
const EXPANDED_LIMIT: u64 = 2 * 1024 * 1024 * 1024;
const FILE_LIMIT: usize = 100_000;

fn fail(message: impl AsRef<str>) -> Error {
    Error::InvalidArgument(format!("development package: {}", message.as_ref()))
}
pub(crate) fn check_stop(stop: &AtomicBool) -> Result<()> {
    if stop.load(Ordering::Acquire) {
        Err(fail(
            "operation cancelled; no development server was started",
        ))
    } else {
        Ok(())
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    format: u32,
    kind: String,
    version: String,
    platform: String,
    node_major: u32,
    files: BTreeMap<String, String>,
}
fn safe_name(name: &str) -> bool {
    !name.is_empty()
        && !name
            .chars()
            .any(|c| c.is_control() || matches!(c, '\\' | ':' | '<' | '>' | '"' | '|' | '?' | '*'))
        && name.split('/').all(|part| {
            let stem = part.split('.').next().unwrap_or("").to_ascii_uppercase();
            !part.is_empty()
                && part != "."
                && part != ".."
                && !part.ends_with(['.', ' '])
                && !matches!(part.to_ascii_lowercase().as_str(), ".next" | ".git")
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
fn regular(path: &Path) -> Result<fs::Metadata> {
    let meta = fs::symlink_metadata(path)?;
    if !meta.is_file() || is_link(&meta) {
        return Err(fail("development files must be regular files, not links"));
    }
    Ok(meta)
}
fn transfer(
    input: &mut impl Read,
    output: &mut impl Write,
    limit: u64,
    stop: &AtomicBool,
) -> Result<(u64, String)> {
    let mut total = 0;
    let mut hash = Sha256::new();
    let mut buffer = [0; 65536];
    loop {
        check_stop(stop)?;
        let n = input.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        total += n as u64;
        if total > limit {
            return Err(fail("development size limit exceeded"));
        }
        hash.update(&buffer[..n]);
        output.write_all(&buffer[..n])?;
    }
    Ok((total, format!("{:x}", hash.finalize())))
}
fn validate(root: &Path, stop: &AtomicBool) -> Result<Manifest> {
    let meta = fs::symlink_metadata(root)?;
    if !meta.is_dir() || is_link(&meta) {
        return Err(fail("development cache must be a directory, not a link"));
    }
    let manifest_path = root.join("manifest.json");
    if regular(&manifest_path)?.len() > 16 * 1024 * 1024 {
        return Err(fail("manifest size limit exceeded"));
    }
    let manifest: Manifest = serde_json::from_slice(&fs::read(&manifest_path)?)
        .map_err(|_| fail("invalid development manifest"))?;
    if manifest.format != 1
        || manifest.kind != "next-development"
        || manifest.version != VERSION
        || manifest.platform != crate::rich_build::platform()?
        || manifest.node_major != 24
        || manifest.files.len() > FILE_LIMIT
    {
        return Err(fail("development package does not match this CLI version"));
    }
    for required in [
        "run.mjs",
        "package.json",
        "package-lock.json",
        "DEPENDENCIES.json",
        "THIRD-PARTY.md",
        "node_modules/next/package.json",
        "node_modules/next/dist/bin/next",
        "node_modules/react/package.json",
        "node_modules/react-dom/package.json",
        "node_modules/typescript/lib/typescript.js",
        "node_modules/@tailwindcss/postcss/package.json",
    ] {
        if !manifest.files.contains_key(required) {
            return Err(fail(format!("development package is missing {required}")));
        }
    }
    let mut dirs = BTreeSet::new();
    let mut folded = BTreeMap::new();
    for (name, expected) in &manifest.files {
        if !safe_name(name)
            || !(name.starts_with("node_modules/")
                || [
                    "run.mjs",
                    "package.json",
                    "package-lock.json",
                    "DEPENDENCIES.json",
                    "THIRD-PARTY.md",
                ]
                .contains(&name.as_str()))
            || expected.len() != 64
            || !expected.bytes().all(|b| b.is_ascii_hexdigit())
        {
            return Err(fail("invalid development file manifest"));
        }
        let mut current = String::new();
        let mut parts = name.split('/').peekable();
        while let Some(part) = parts.next() {
            if !current.is_empty() {
                current.push('/');
            }
            current.push_str(part);
            if let Some(previous) = folded.insert(current.to_lowercase(), current.clone())
                && previous != current
            {
                return Err(fail("case-colliding development paths"));
            }
            if parts.peek().is_some() {
                dirs.insert(current.clone());
            }
        }
    }
    let mut found = BTreeSet::new();
    let mut total = 0u64;
    for entry in walkdir::WalkDir::new(root).follow_links(false).min_depth(1) {
        check_stop(stop)?;
        let entry = entry.map_err(|_| fail("cannot inspect development cache"))?;
        let relative = entry
            .path()
            .strip_prefix(root)
            .unwrap()
            .to_str()
            .ok_or_else(|| fail("non-UTF-8 development path"))?
            .replace(std::path::MAIN_SEPARATOR, "/");
        let meta = fs::symlink_metadata(entry.path())?;
        if is_link(&meta) {
            return Err(fail("development cache contains a link or junction"));
        }
        if meta.is_dir() {
            if !dirs.contains(&relative) {
                return Err(fail("development cache contains an unlisted directory"));
            }
            continue;
        }
        if !meta.is_file() {
            return Err(fail("development cache contains a special file"));
        }
        if relative == "manifest.json" {
            continue;
        }
        let expected = manifest
            .files
            .get(&relative)
            .ok_or_else(|| fail("development cache contains an unlisted file"))?;
        total = total
            .checked_add(meta.len())
            .ok_or_else(|| fail("development size overflow"))?;
        if total > EXPANDED_LIMIT {
            return Err(fail("development cache is oversized"));
        }
        let (_, actual) = transfer(
            &mut File::open(entry.path())?,
            &mut std::io::sink(),
            EXPANDED_LIMIT,
            stop,
        )?;
        if &actual != expected {
            return Err(fail(format!(
                "cached file {relative} failed integrity verification; remove the damaged development cache and retry"
            )));
        }
        found.insert(relative);
    }
    if found.len() != manifest.files.len() {
        return Err(fail("development cache is incomplete"));
    }
    Ok(manifest)
}
fn base_url() -> Result<Url> {
    let value = std::env::var("ZEL_DEV_BASE_URL").unwrap_or_else(|_| {
        format!("https://github.com/mylife-inc/releases/releases/download/zellij-v{VERSION}")
    });
    let url = Url::parse(&(value.trim_end_matches('/').to_owned() + "/"))
        .map_err(|_| fail("invalid ZEL_DEV_BASE_URL"))?;
    if !(url.scheme() == "https"
        || url.scheme() == "http"
            && matches!(url.host_str(), Some("localhost" | "127.0.0.1" | "[::1]")))
        || !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
    {
        return Err(fail(
            "development URL must use HTTPS (or loopback HTTP), without credentials, query or fragment",
        ));
    }
    Ok(url)
}
fn download(url: Url, limit: u64, output: &mut impl Write, stop: &AtomicBool) -> Result<()> {
    crate::package_transfer::download(url, limit, output, stop, "development package")
}

fn extract(archive: &Path, into: &Path, stop: &AtomicBool) -> Result<()> {
    let mut reader = tar::Archive::new(
        flate2::read::GzDecoder::new(File::open(archive)?).take(EXPANDED_LIMIT + 16 * 1024 * 1024),
    );
    let mut names = BTreeSet::new();
    let mut total = 0u64;
    for entry in reader.entries()? {
        check_stop(stop)?;
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
        if total > EXPANDED_LIMIT || names.len() > FILE_LIMIT + 1 {
            return Err(fail("archive expansion limit exceeded"));
        }
        let dest = into.join(&name);
        fs::create_dir_all(dest.parent().unwrap())?;
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&dest)?;
        let (count, _) = transfer(&mut entry, &mut file, EXPANDED_LIMIT, stop)?;
        if count != entry.size() {
            return Err(fail("truncated development archive"));
        }
    }
    let mut decoder = reader.into_inner();
    transfer(&mut decoder, &mut std::io::sink(), EXPANDED_LIMIT, stop)?;
    if decoder.limit() == 0 {
        return Err(fail("archive expansion limit exceeded"));
    }
    Ok(())
}
pub(crate) struct Package {
    root: PathBuf,
    manifest: Manifest,
}
impl Package {
    pub(crate) fn prepare(&self, into: &Path, stop: &AtomicBool) -> Result<PathBuf> {
        for (name, expected) in &self.manifest.files {
            // Dependencies are verified by obtain and referenced read-only by
            // the private runner. Only mutable application metadata is copied.
            if name.starts_with("node_modules/") {
                continue;
            }
            check_stop(stop)?;
            let source = files::contained_path(&utf8(&self.root)?, Utf8Path::new(name))?;
            regular(source.as_std_path())?;
            let dest = into.join(name);
            fs::create_dir_all(dest.parent().unwrap())?;
            let mut file = fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&dest)?;
            let (_, actual) = transfer(&mut File::open(source)?, &mut file, EXPANDED_LIMIT, stop)?;
            if actual != *expected {
                return Err(fail(
                    "cache changed while copying; no development server was started",
                ));
            }
        }
        Ok(self.root.join("node_modules"))
    }
}
pub(crate) fn obtain(offline: bool, stop: &AtomicBool) -> Result<Package> {
    check_stop(stop)?;
    let resource_home = utf8(&crate::user_home::root()?)?;
    let cache = files::contained_path(&resource_home, Utf8Path::new("cache/development"))?
        .into_std_path_buf();
    let platform = crate::rich_build::platform()?;
    let installed = cache.join(format!("{VERSION}-{platform}"));
    let valid = || -> Result<Package> {
        Ok(Package {
            manifest: validate(&installed, stop)?,
            root: fs::canonicalize(&installed)?,
        })
    };
    if fs::symlink_metadata(&installed).is_ok() {
        return valid();
    }
    if offline {
        return Err(fail(
            "dependencies are not cached; run zel dev once with internet access before using --offline",
        ));
    }
    if let Ok(meta) = fs::symlink_metadata(&cache)
        && (!meta.is_dir() || is_link(&meta))
    {
        return Err(fail(
            "development cache root must be a directory, not a link",
        ));
    }
    fs::create_dir_all(&cache)?;
    let stage = tempfile::Builder::new()
        .prefix(".download-")
        .tempdir_in(&cache)?;
    let base = base_url()?;
    let name = format!("zel-development-v{VERSION}-{platform}.tar.gz");
    let mut checksum = Vec::new();
    download(
        base.join(&(name.clone() + ".sha256")).unwrap(),
        1024,
        &mut checksum,
        stop,
    )?;
    let checksum = String::from_utf8(checksum).map_err(|_| fail("invalid archive checksum"))?;
    let parts = checksum.split_whitespace().collect::<Vec<_>>();
    if parts.len() != 2
        || parts[1] != name
        || parts[0].len() != 64
        || !parts[0].bytes().all(|b| b.is_ascii_hexdigit())
    {
        return Err(fail("invalid archive checksum"));
    }
    zellij_core::statusln!(
        "Downloading Next development dependencies {VERSION} ({platform}); later commands reuse the cache."
    );
    let archive = stage.path().join("development.tar.gz");
    download(
        base.join(&name).unwrap(),
        ARCHIVE_LIMIT,
        &mut File::create(&archive)?,
        stop,
    )?;
    let (_, actual) = transfer(
        &mut File::open(&archive)?,
        &mut std::io::sink(),
        ARCHIVE_LIMIT,
        stop,
    )?;
    if actual != parts[0] {
        return Err(fail(
            "archive checksum mismatch; dependencies were not installed or executed",
        ));
    }
    let unpacked = stage.path().join("dependencies");
    fs::create_dir(&unpacked)?;
    extract(&archive, &unpacked, stop)?;
    validate(&unpacked, stop)?;
    check_stop(stop)?;
    if let Err(error) = fs::rename(&unpacked, &installed)
        && !installed.exists()
    {
        return Err(error.into());
    }
    valid()
}
fn utf8(path: &Path) -> Result<Utf8PathBuf> {
    Utf8PathBuf::from_path_buf(path.to_owned())
        .map_err(|_| fail("cache/staging path must be UTF-8"))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture(root: &Path) {
        let mut files = BTreeMap::new();
        for name in [
            "run.mjs",
            "package.json",
            "package-lock.json",
            "DEPENDENCIES.json",
            "THIRD-PARTY.md",
            "node_modules/next/package.json",
            "node_modules/next/dist/bin/next",
            "node_modules/react/package.json",
            "node_modules/react-dom/package.json",
            "node_modules/typescript/lib/typescript.js",
            "node_modules/@tailwindcss/postcss/package.json",
        ] {
            let path = root.join(name);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(&path, b"fixture").unwrap();
            files.insert(name, format!("{:x}", Sha256::digest(b"fixture")));
        }
        fs::write(
            root.join("manifest.json"),
            serde_json::to_vec(&serde_json::json!({
                "format":1,"kind":"next-development","version":VERSION,
                "platform":crate::rich_build::platform().unwrap(),"node_major":24,"files":files
            }))
            .unwrap(),
        )
        .unwrap();
    }
    #[test]
    fn verifies_identity_completeness_integrity_and_case_aliases() {
        let dir = tempfile::tempdir().unwrap();
        fixture(dir.path());
        let stop = AtomicBool::new(false);
        validate(dir.path(), &stop).unwrap();
        let file = dir.path().join("manifest.json");
        let original: serde_json::Value =
            serde_json::from_slice(&fs::read(&file).unwrap()).unwrap();
        for (key, value) in [
            ("version", serde_json::json!("0.0.0")),
            ("platform", serde_json::json!("wrong")),
            ("node_major", serde_json::json!(99)),
        ] {
            let mut wrong = original.clone();
            wrong[key] = value;
            fs::write(&file, serde_json::to_vec(&wrong).unwrap()).unwrap();
            assert!(validate(dir.path(), &stop).is_err(), "{key}");
        }
        fs::write(&file, serde_json::to_vec(&original).unwrap()).unwrap();
        for directory in [false, true] {
            let extra = dir.path().join("extra");
            if directory {
                fs::create_dir(&extra).unwrap();
            } else {
                fs::write(&extra, "unlisted").unwrap();
            }
            assert!(validate(dir.path(), &stop).is_err());
            if directory {
                fs::remove_dir(&extra).unwrap();
            } else {
                fs::remove_file(&extra).unwrap();
            }
        }
        fs::write(dir.path().join("run.mjs"), "tampered").unwrap();
        assert!(validate(dir.path(), &stop).is_err());
        fixture(dir.path());
        fs::remove_file(dir.path().join("run.mjs")).unwrap();
        assert!(validate(dir.path(), &stop).is_err());
        fixture(dir.path());
        let mut alias = original;
        alias["files"]["NODE_MODULES/other.js"] = serde_json::json!("a".repeat(64));
        fs::write(&file, serde_json::to_vec(&alias).unwrap()).unwrap();
        assert!(validate(dir.path(), &stop).is_err());
    }
    #[test]
    fn copies_are_isolated_and_changed_cache_or_cancellation_cannot_execute() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("cache");
        fs::create_dir(&source).unwrap();
        fixture(&source);
        let stop = AtomicBool::new(false);
        let package = Package {
            manifest: validate(&source, &stop).unwrap(),
            root: source.clone(),
        };
        let out = dir.path().join("session");
        package.prepare(&out, &stop).unwrap();
        fs::write(out.join("run.mjs"), "session change").unwrap();
        assert_eq!(fs::read(source.join("run.mjs")).unwrap(), b"fixture");
        fs::write(source.join("run.mjs"), "cache change").unwrap();
        assert!(
            package
                .prepare(&dir.path().join("changed"), &stop)
                .is_err()
        );
        stop.store(true, Ordering::Release);
        assert!(
            package
                .prepare(&dir.path().join("cancelled"), &stop)
                .is_err()
        );
        assert!(!dir.path().join("cancelled").exists());
    }
    #[cfg(unix)]
    #[test]
    fn rejects_cache_links_and_literal_backslash_aliases() {
        let dir = tempfile::tempdir().unwrap();
        fixture(dir.path());
        let stop = AtomicBool::new(false);
        let file = dir.path().join("run.mjs");
        fs::remove_file(&file).unwrap();
        std::os::unix::fs::symlink("package.json", &file).unwrap();
        assert!(validate(dir.path(), &stop).is_err());
        fs::remove_file(file).unwrap();
        fixture(dir.path());
        fs::write(dir.path().join(r"node_modules\extra.js"), "unlisted").unwrap();
        assert!(validate(dir.path(), &stop).is_err());
    }
    #[test]
    fn archives_reject_nonportable_paths_links_duplicates_and_truncation() {
        for name in [
            "../escape",
            "/absolute",
            "a/../b",
            "a\\b",
            "a:stream",
            "AUX.txt",
            "a//b",
            "a/name.",
            "a/name ",
            "a\nfile",
            "node_modules/.next/code.js",
            "node_modules/.Next/code.js",
            "engine/.GIT/config",
        ] {
            assert!(!safe_name(name), "{name}");
        }
        for (link, duplicate) in [(true, false), (false, true), (false, false)] {
            let dir = tempfile::tempdir().unwrap();
            let archive = dir.path().join("archive.tar.gz");
            let gzip = flate2::write::GzEncoder::new(
                File::create(&archive).unwrap(),
                flate2::Compression::default(),
            );
            let mut tar = tar::Builder::new(gzip);
            let mut header = tar::Header::new_gnu();
            header.set_mode(0o644);
            header.set_size(if link { 0 } else { 3 });
            if link {
                header.set_entry_type(tar::EntryType::Symlink);
                header.set_link_name("../escape").unwrap();
            }
            header.set_cksum();
            tar.append_data(
                &mut header,
                "engine/file",
                if link { &b""[..] } else { &b"abc"[..] },
            )
            .unwrap();
            if duplicate {
                tar.append_data(&mut header, "ENGINE/file", &b"abc"[..])
                    .unwrap();
            }
            tar.into_inner().unwrap().finish().unwrap();
            let out = dir.path().join("out");
            fs::create_dir(&out).unwrap();
            assert_eq!(
                extract(&archive, &out, &AtomicBool::new(false)).is_ok(),
                !link && !duplicate
            );
            assert!(!dir.path().join("escape").exists());
            if !link && !duplicate {
                let bytes = fs::read(&archive).unwrap();
                fs::write(&archive, &bytes[..bytes.len() - 8]).unwrap();
                let truncated = dir.path().join("truncated");
                fs::create_dir(&truncated).unwrap();
                assert!(extract(&archive, &truncated, &AtomicBool::new(false)).is_err());
            }
        }
    }
}
