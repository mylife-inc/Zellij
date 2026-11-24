use reqwest::Url;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet, HashSet},
    fs::{self, File},
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::atomic::AtomicBool,
};
use zellij_core::{Error, Result, site::is_link};
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
    Ok(crate::user_home::root()?.join("cache").join("engines"))
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
fn regular(path: &Path) -> Result<fs::Metadata> {
    let metadata = fs::symlink_metadata(path)?;
    if !metadata.is_file() || is_link(&metadata) {
        return Err(fail("cached engine contains a link or non-file"));
    }
    Ok(metadata)
}
fn check_stop(stop: &AtomicBool) -> Result<()> {
    crate::package_transfer::check_stop(stop, "Rich renderer")
}
fn transfer(
    input: &mut impl Read,
    output: &mut impl Write,
    limit: u64,
    stop: &AtomicBool,
) -> Result<(u64, String)> {
    let mut total = 0u64;
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
            return Err(fail("file exceeds its size limit"));
        }
        check_stop(stop)?;
        hash.update(&buffer[..n]);
        output.write_all(&buffer[..n])?;
    }
    check_stop(stop)?;
    Ok((total, format!("{:x}", hash.finalize())))
}
fn digest(path: &Path, stop: &AtomicBool) -> Result<String> {
    regular(path)?;
    Ok(transfer(
        &mut File::open(path)?,
        &mut std::io::sink(),
        EXPANDED_LIMIT,
        stop,
    )?
    .1)
}

fn validate(root: &Path, target: &str, stop: &AtomicBool) -> Result<()> {
    validate_files(root, target, stop).map_err(|error| {
        Error::InvalidArgument(format!("{error} (renderer cache: {})", root.display()))
    })
}

fn validate_files(root: &Path, target: &str, stop: &AtomicBool) -> Result<()> {
    check_stop(stop)?;
    let metadata = fs::symlink_metadata(root)?;
    if !metadata.is_dir() || is_link(&metadata) {
        return Err(fail(
            "engine cache must be a directory, not a link or junction",
        ));
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
    let mut dirs = BTreeSet::new();
    let mut folded = BTreeMap::new();
    for (name, expected) in &manifest.files {
        check_stop(stop)?;
        if !safe_name(name)
            || name == "manifest.json"
            || expected.len() != 64
            || !expected.bytes().all(|b| b.is_ascii_hexdigit())
        {
            return Err(fail("invalid engine file manifest"));
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
                return Err(fail("case-colliding engine paths"));
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
        let entry = entry.map_err(|_| fail("cannot inspect engine cache"))?;
        let relative = entry
            .path()
            .strip_prefix(root)
            .unwrap()
            .to_str()
            .ok_or_else(|| fail("non-UTF-8 engine path"))?
            .replace(std::path::MAIN_SEPARATOR, "/");
        let metadata = fs::symlink_metadata(entry.path())?;
        if is_link(&metadata) {
            return Err(fail("engine cache contains a link or junction"));
        }
        if metadata.is_dir() {
            if !dirs.contains(&relative) {
                return Err(fail("engine cache contains an unlisted directory"));
            }
            continue;
        }
        if !metadata.is_file() {
            return Err(fail("engine cache contains a special file"));
        }
        if relative == "manifest.json" {
            continue;
        }
        let expected = manifest
            .files
            .get(&relative)
            .ok_or_else(|| fail("engine cache contains an unlisted file"))?;
        total = total
            .checked_add(metadata.len())
            .ok_or_else(|| fail("cache size overflow"))?;
        if total > EXPANDED_LIMIT {
            return Err(fail("engine cache exceeds its size limit"));
        }
        if digest(entry.path(), stop)? != *expected {
            return Err(fail(format!(
                "cached file {relative} failed integrity verification; remove {} and retry",
                root.display()
            )));
        }
        found.insert(relative);
    }
    if found.len() != manifest.files.len() {
        return Err(fail("engine cache is incomplete"));
    }
    check_stop(stop)
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
fn extract(archive: &Path, into: &Path, stop: &AtomicBool) -> Result<()> {
    check_stop(stop)?;
    let mut reader = tar::Archive::new(
        flate2::read::GzDecoder::new(File::open(archive)?).take(EXPANDED_LIMIT + 1),
    );
    let mut total = 0u64;
    let mut names = HashSet::new();
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
        if total > EXPANDED_LIMIT || names.len() > 10000 {
            return Err(fail("archive exceeds its expansion limits"));
        }
        let dest = into.join(&name);
        fs::create_dir_all(dest.parent().unwrap())?;
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&dest)?;
        if transfer(&mut entry, &mut file, EXPANDED_LIMIT, stop)?.0 != entry.size() {
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
    transfer(&mut decoder, &mut std::io::sink(), EXPANDED_LIMIT, stop)?;
    if decoder.limit() == 0 {
        return Err(fail("archive expansion limit exceeded"));
    }
    Ok(())
}
fn present(path: &Path) -> Result<bool> {
    match fs::symlink_metadata(path) {
        Ok(_) => Ok(true),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(e) => Err(e.into()),
    }
}
pub(crate) fn inspect() -> Result<serde_json::Value> {
    let platform = platform()?;
    let installed = root()?.join(format!("{VERSION}-{platform}"));
    let cached = present(&installed)?;
    if cached {
        validate(&installed, &platform, &AtomicBool::new(false))?;
    }
    Ok(
        serde_json::json!({"status":if cached {"cached"} else {"download_required"},"version":VERSION,"platform":platform,"cache":installed.to_string_lossy()}),
    )
}
pub(super) fn obtain(offline: bool, announce: bool, stop: &AtomicBool) -> Result<PathBuf> {
    check_stop(stop)?;
    let platform = platform()?;
    let cache = root()?;
    let installed = cache.join(format!("{VERSION}-{platform}"));
    if present(&installed)? {
        validate(&installed, &platform, stop)?;
        return fs::canonicalize(installed).map_err(Into::into);
    }
    if offline {
        return Err(fail(
            "engine is not cached; run zel build or zel check once with internet access before using --offline",
        ));
    }
    fs::create_dir_all(&cache)?;
    let stage = tempfile::Builder::new()
        .prefix(".download-")
        .tempdir_in(&cache)?;
    let base = base_url()?;
    let name = format!("zel-renderer-v{VERSION}-{platform}.tar.gz");
    let mut checksum = Vec::new();
    crate::package_transfer::download(
        base.join(&(name.clone() + ".sha256")).unwrap(),
        1024,
        &mut checksum,
        stop,
        "Rich renderer",
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
    if announce {
        zellij_core::statusln!(
            "  Downloading rich renderer {VERSION} ({platform}); later commands use the cache."
        );
    }
    let archive = stage.path().join("download.tar.gz");
    crate::package_transfer::download(
        base.join(&name).unwrap(),
        ARCHIVE_LIMIT,
        &mut File::create(&archive)?,
        stop,
        "Rich renderer",
    )?;
    if digest(&archive, stop)? != fields[0] {
        return Err(fail(
            "download checksum mismatch; engine was not installed or executed",
        ));
    }
    let unpacked = stage.path().join("engine");
    fs::create_dir(&unpacked)?;
    extract(&archive, &unpacked, stop)?;
    validate(&unpacked, &platform, stop)?;
    check_stop(stop)?;
    if let Err(error) = fs::rename(&unpacked, &installed) {
        if present(&installed)? {
            validate(&installed, &platform, stop)?;
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
            assert_eq!(
                extract(&input, &output, &AtomicBool::new(false)).is_ok(),
                !link && !duplicate
            );
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
            files.insert(name, digest(&file, &AtomicBool::new(false)).unwrap());
        }
        let mut manifest = serde_json::json!({"format":1,"version":VERSION,"platform":target,"node_version":"v24.22.3","files":files});
        let file = dir.path().join("manifest.json");
        fs::write(&file, manifest.to_string()).unwrap();
        validate(dir.path(), &target, &AtomicBool::new(false)).unwrap();
        manifest["version"] = serde_json::json!("0.0.0");
        fs::write(&file, manifest.to_string()).unwrap();
        assert!(validate(dir.path(), &target, &AtomicBool::new(false)).is_err());
        manifest["version"] = serde_json::json!(VERSION);
        fs::write(&file, manifest.to_string()).unwrap();
        fs::write(dir.path().join(node), "changed").unwrap();
        assert!(validate(dir.path(), &target, &AtomicBool::new(false)).is_err());
    }
    fn fixture(root: &Path) {
        let mut files = BTreeMap::new();
        for name in [
            if cfg!(windows) { "node.exe" } else { "node" },
            "engine/engine.mjs",
            "engine/runtime.js",
            "engine/styles.css",
        ] {
            let file = root.join(name);
            fs::create_dir_all(file.parent().unwrap()).unwrap();
            fs::write(&file, name).unwrap();
            files.insert(name, digest(&file, &AtomicBool::new(false)).unwrap());
        }
        fs::write(root.join("manifest.json"), serde_json::json!({
            "format":1, "version":VERSION, "platform":platform().unwrap(), "node_version":"v24.22.3", "files":files
        }).to_string()).unwrap();
    }
    #[test]
    fn validation_rejects_unlisted_files_directories_missing_files_and_aliases() {
        for mutation in [
            "extra_file",
            "extra_directory",
            "missing",
            "alias",
            "self",
            "oversized",
        ] {
            let dir = tempfile::tempdir().unwrap();
            fixture(dir.path());
            let path = dir.path().join("manifest.json");
            let mut manifest: serde_json::Value =
                serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
            match mutation {
                "extra_file" => {
                    fs::write(dir.path().join("engine/unlisted.js"), "must not execute").unwrap()
                }
                "extra_directory" => fs::create_dir(dir.path().join("engine/unlisted")).unwrap(),
                "missing" => fs::remove_file(dir.path().join("engine/runtime.js")).unwrap(),
                "alias" => {
                    manifest["files"]["ENGINE/other.js"] = serde_json::json!("0".repeat(64));
                }
                "self" => {
                    manifest["files"]["manifest.json"] = serde_json::json!("0".repeat(64));
                }
                "oversized" => File::create(dir.path().join("engine/styles.css"))
                    .unwrap()
                    .set_len(EXPANDED_LIMIT + 1)
                    .unwrap(),
                _ => unreachable!(),
            }
            fs::write(&path, manifest.to_string()).unwrap();
            assert!(
                validate(dir.path(), &platform().unwrap(), &AtomicBool::new(false)).is_err(),
                "{mutation}"
            );
        }
    }
    #[test]
    fn cancellation_checks_precede_acquisition_and_stop_mid_transfer() {
        use std::sync::atomic::Ordering;
        let stop = AtomicBool::new(true);
        assert!(
            obtain(false, false, &stop)
                .unwrap_err()
                .to_string()
                .contains("cancelled")
        );
        let dir = tempfile::tempdir().unwrap();
        fixture(dir.path());
        assert!(
            validate(dir.path(), &platform().unwrap(), &stop)
                .unwrap_err()
                .to_string()
                .contains("cancelled")
        );
        assert!(
            extract(&dir.path().join("absent"), dir.path(), &stop)
                .unwrap_err()
                .to_string()
                .contains("cancelled")
        );
        struct Interrupt<'a>(&'a AtomicBool);
        impl Read for Interrupt<'_> {
            fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
                buffer[0] = 1;
                self.0.store(true, Ordering::Release);
                Ok(1)
            }
        }
        stop.store(false, Ordering::Release);
        let mut written = Vec::new();
        assert!(
            transfer(&mut Interrupt(&stop), &mut written, 1024, &stop)
                .unwrap_err()
                .to_string()
                .contains("cancelled")
        );
        assert!(
            written.is_empty(),
            "a cancellation observed after reading must precede writing"
        );
    }
    #[test]
    fn extraction_rejects_truncated_gzip_trailer() {
        let dir = tempfile::tempdir().unwrap();
        let input = dir.path().join("archive.tar.gz");
        archive(&input, false, false);
        let bytes = fs::read(&input).unwrap();
        fs::write(&input, &bytes[..bytes.len() - 8]).unwrap();
        let out = dir.path().join("out");
        fs::create_dir(&out).unwrap();
        assert!(extract(&input, &out, &AtomicBool::new(false)).is_err());
    }
    #[cfg(unix)]
    #[test]
    fn cache_links_including_dangling_roots_are_rejected_without_following() {
        let dir = tempfile::tempdir().unwrap();
        let outside = dir.path().join("outside");
        fs::write(&outside, "authored").unwrap();
        let root = dir.path().join("cache");
        fs::create_dir(&root).unwrap();
        fixture(&root);
        fs::remove_file(root.join("engine/styles.css")).unwrap();
        std::os::unix::fs::symlink(&outside, root.join("engine/styles.css")).unwrap();
        assert!(validate(&root, &platform().unwrap(), &AtomicBool::new(false)).is_err());
        assert_eq!(fs::read_to_string(&outside).unwrap(), "authored");
        let dangling = dir.path().join("dangling");
        std::os::unix::fs::symlink(dir.path().join("absent"), &dangling).unwrap();
        assert!(present(&dangling).unwrap());
        assert!(validate(&dangling, &platform().unwrap(), &AtomicBool::new(false)).is_err());
    }
    #[cfg(unix)]
    #[test]
    fn literal_backslash_file_cannot_alias_a_manifested_nested_path() {
        let dir = tempfile::tempdir().unwrap();
        fixture(dir.path());
        fs::copy(
            dir.path().join("engine/styles.css"),
            dir.path().join(r"engine\styles.css"),
        )
        .unwrap();
        assert!(
            validate(dir.path(), &platform().unwrap(), &AtomicBool::new(false))
                .unwrap_err()
                .to_string()
                .contains("unlisted")
        );
    }
}
