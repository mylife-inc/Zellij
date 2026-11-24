//! Version-matched editable sources. No runtime or package manager is executed.
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
const ARCHIVE_LIMIT: u64 = 64 * 1024 * 1024;
const EXPANDED_LIMIT: u64 = 256 * 1024 * 1024;
const FILE_LIMIT: usize = 20_000;

fn fail(message: impl AsRef<str>) -> Error {
    Error::InvalidArgument(format!("editable sources: {}", message.as_ref()))
}
pub(crate) fn check_stop(stop: &AtomicBool) -> Result<()> {
    if stop.load(Ordering::Acquire) {
        Err(fail("operation cancelled; no wrapper was published"))
    } else {
        Ok(())
    }
}
pub(crate) fn stop_handler() -> Result<std::sync::Arc<AtomicBool>> {
    crate::preview::signals()
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    format: u32,
    kind: String,
    version: String,
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
                && !matches!(
                    part.to_ascii_lowercase().as_str(),
                    "node_modules" | ".next" | ".git"
                )
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
fn sha(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn regular(path: &Path) -> Result<fs::Metadata> {
    let meta = fs::symlink_metadata(path)?;
    if !meta.is_file() || is_link(&meta) {
        return Err(fail("source files must be regular files, not links"));
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
            return Err(fail("source size limit exceeded"));
        }
        hash.update(&buffer[..n]);
        output.write_all(&buffer[..n])?;
    }
    Ok((total, format!("{:x}", hash.finalize())))
}
fn validate(root: &Path, stop: &AtomicBool) -> Result<Manifest> {
    let meta = fs::symlink_metadata(root)?;
    if !meta.is_dir() || is_link(&meta) {
        return Err(fail("source cache must be a directory, not a link"));
    }
    let manifest_path = root.join("manifest.json");
    if regular(&manifest_path)?.len() > 4 * 1024 * 1024 {
        return Err(fail("manifest size limit exceeded"));
    }
    let manifest: Manifest = serde_json::from_slice(&fs::read(&manifest_path)?)
        .map_err(|_| fail("invalid source manifest"))?;
    if manifest.format != 1
        || manifest.kind != "editable-sources"
        || manifest.version != VERSION
        || manifest.files.len() > FILE_LIMIT
    {
        return Err(fail("source package does not match this CLI version"));
    }
    for required in [
        "app/package.json",
        "app/tsconfig.json",
        "app/next.config.js",
        "app/.npmrc",
        "app/src/app/layout.tsx",
        "app/src/app/[[...slug]]/page.tsx",
        "app/content/site.yaml",
        "engine/package.json",
        "engine/src/index.ts",
        "engine/bin/zellij.mjs",
        "engine/dist/cli/check-bundle.js",
        "engine/src/themes/fonts/licenses.txt",
        "engine/BUNDLED-THIRD-PARTY-NOTICES.txt",
    ] {
        if !manifest.files.contains_key(required) {
            return Err(fail(format!("source package is missing {required}")));
        }
    }
    let mut dirs = BTreeSet::new();
    let mut folded = BTreeMap::new();
    for (name, expected) in &manifest.files {
        if !safe_name(name)
            || !(name.starts_with("app/") || name.starts_with("engine/"))
            || expected.len() != 64
            || !expected.bytes().all(|b| b.is_ascii_hexdigit())
        {
            return Err(fail("invalid source file manifest"));
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
                return Err(fail("case-colliding source paths"));
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
        let entry = entry.map_err(|_| fail("cannot inspect source cache"))?;
        let relative = entry
            .path()
            .strip_prefix(root)
            .unwrap()
            .to_str()
            .ok_or_else(|| fail("non-UTF-8 source path"))?
            .replace(std::path::MAIN_SEPARATOR, "/");
        let meta = fs::symlink_metadata(entry.path())?;
        if is_link(&meta) {
            return Err(fail("source cache contains a link or junction"));
        }
        if meta.is_dir() {
            if !dirs.contains(&relative) {
                return Err(fail("source cache contains an unlisted directory"));
            }
            continue;
        }
        if !meta.is_file() {
            return Err(fail("source cache contains a special file"));
        }
        if relative == "manifest.json" {
            continue;
        }
        let expected = manifest
            .files
            .get(&relative)
            .ok_or_else(|| fail("source cache contains an unlisted file"))?;
        total = total
            .checked_add(meta.len())
            .ok_or_else(|| fail("source size overflow"))?;
        if total > EXPANDED_LIMIT {
            return Err(fail("source cache is oversized"));
        }
        let (_, actual) = transfer(
            &mut File::open(entry.path())?,
            &mut std::io::sink(),
            EXPANDED_LIMIT,
            stop,
        )?;
        if &actual != expected {
            return Err(fail(format!(
                "cached file {relative} failed integrity verification; remove the damaged source cache and retry"
            )));
        }
        found.insert(relative);
    }
    if found.len() != manifest.files.len() {
        return Err(fail("source cache is incomplete"));
    }
    let engine = Utf8Path::from_path(&root.join("engine"))
        .map(Utf8Path::to_owned)
        .ok_or_else(|| fail("source cache path must be UTF-8"))?;
    crate::app::validate_engine(&engine)?;
    Ok(manifest)
}
fn base_url() -> Result<Url> {
    let value = std::env::var("ZEL_SOURCE_BASE_URL").unwrap_or_else(|_| {
        format!("https://github.com/mylife-inc/releases/releases/download/zellij-v{VERSION}")
    });
    let url = Url::parse(&(value.trim_end_matches('/').to_owned() + "/"))
        .map_err(|_| fail("invalid ZEL_SOURCE_BASE_URL"))?;
    if !(url.scheme() == "https"
        || url.scheme() == "http"
            && matches!(url.host_str(), Some("localhost" | "127.0.0.1" | "[::1]")))
        || !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
    {
        return Err(fail(
            "source URL must use HTTPS (or loopback HTTP), without credentials, query or fragment",
        ));
    }
    Ok(url)
}
fn download(url: Url, limit: u64, output: &mut impl Write, stop: &AtomicBool) -> Result<()> {
    crate::package_transfer::download(url, limit, output, stop, "editable sources")
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
            return Err(fail("truncated source archive"));
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
    pub(crate) fn copy(&self, kind: &str, into: &Utf8Path, stop: &AtomicBool) -> Result<()> {
        let prefix = format!("{kind}/");
        for (name, expected) in &self.manifest.files {
            let Some(relative) = name.strip_prefix(&prefix) else {
                continue;
            };
            check_stop(stop)?;
            let source = files::contained_path(&utf8(&self.root)?, Utf8Path::new(name))?;
            regular(source.as_std_path())?;
            let dest = into.join(relative);
            fs::create_dir_all(dest.parent().unwrap())?;
            let mut file = fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&dest)?;
            let (_, actual) = transfer(&mut File::open(source)?, &mut file, EXPANDED_LIMIT, stop)?;
            if actual != *expected {
                return Err(fail(
                    "source cache changed while copying; no wrapper was published",
                ));
            }
        }
        Ok(())
    }
}
pub(crate) fn obtain(offline: bool, stop: &AtomicBool) -> Result<Package> {
    check_stop(stop)?;
    let resource_home = utf8(&crate::user_home::root()?)?;
    let cache =
        files::contained_path(&resource_home, Utf8Path::new("cache/sources"))?.into_std_path_buf();
    let installed = cache.join(VERSION);
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
            "sources are not cached; run zel eject or zel new app once with internet access before using --offline",
        ));
    }
    if let Ok(meta) = fs::symlink_metadata(&cache)
        && (!meta.is_dir() || is_link(&meta))
    {
        return Err(fail("source cache root must be a directory, not a link"));
    }
    fs::create_dir_all(&cache)?;
    let stage = tempfile::Builder::new()
        .prefix(".download-")
        .tempdir_in(&cache)?;
    let base = base_url()?;
    let name = format!("zel-sources-v{VERSION}.tar.gz");
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
        "Downloading editable sources {VERSION}; later commands reuse the cache."
    );
    let archive = stage.path().join("source.tar.gz");
    let mut bytes = Vec::new();
    download(base.join(&name).unwrap(), ARCHIVE_LIMIT, &mut bytes, stop)?;
    if sha(&bytes) != parts[0] {
        return Err(fail(
            "archive checksum mismatch; sources were not installed or executed",
        ));
    }
    fs::write(&archive, bytes)?;
    let unpacked = stage.path().join("sources");
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
// Shared staged publication for cache-backed new app; an existing authored
// directory is never replaced, including one that changed during preparation.
pub(crate) fn publish(stage: &Utf8Path, out: &Utf8Path, stop: &AtomicBool) -> Result<()> {
    check_stop(stop)?;
    if !out.exists() {
        fs::rename(stage, out)?;
    } else {
        // Native app scaffolding permits a fresh git repository's dotfiles.
        // Preserve those entries and use the common no-clobber file publisher.
        let mut entries = Vec::new();
        for entry in walkdir::WalkDir::new(stage)
            .follow_links(false)
            .min_depth(1)
        {
            check_stop(stop)?;
            let entry = entry.map_err(|_| fail("cannot inspect staged wrapper"))?;
            let meta = fs::symlink_metadata(entry.path())?;
            if is_link(&meta) {
                return Err(fail("staged wrapper contains a link"));
            }
            if meta.is_file() {
                entries.push((
                    Utf8PathBuf::from(
                        utf8(entry.path().strip_prefix(stage).unwrap())?
                            .as_str()
                            .replace('\\', "/"),
                    ),
                    fs::read(entry.path())?,
                ));
            } else if !meta.is_dir() {
                return Err(fail("staged wrapper contains a special file"));
            }
        }
        check_stop(stop)?;
        files::write_files(
            out,
            &entries
                .iter()
                .map(|(name, bytes)| (name.as_path(), bytes.as_slice()))
                .collect::<Vec<_>>(),
            false,
        )?;
    }
    Ok(())
}
pub(crate) fn utf8(path: &Path) -> Result<Utf8PathBuf> {
    Utf8PathBuf::from_path_buf(path.to_owned())
        .map_err(|_| fail("cache/staging path must be UTF-8"))
}

pub(crate) fn check_destination(out: &Utf8Path) -> Result<()> {
    let cache = utf8(&crate::user_home::root()?.join("cache/sources"))?;
    files::disjoint(&cache, out)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture(root: &Path) {
        let mut files = BTreeMap::new();
        for name in [
            "app/package.json",
            "app/tsconfig.json",
            "app/next.config.js",
            "app/.npmrc",
            "app/src/app/layout.tsx",
            "app/src/app/[[...slug]]/page.tsx",
            "app/content/site.yaml",
            "engine/src/index.ts",
            "engine/bin/zellij.mjs",
            "engine/dist/cli/check-bundle.js",
            "engine/src/themes/fonts/licenses.txt",
            "engine/BUNDLED-THIRD-PARTY-NOTICES.txt",
            "engine/next-config.mjs",
            "engine/deployment-config.cjs",
            "engine/feature-config.cjs",
            "engine/feature-config.d.cts",
            "engine/optional-maths.mjs",
            "engine/optional-maths.d.mts",
            "engine/optional-mermaid.mjs",
            "engine/optional-mermaid.d.mts",
            "engine/disabled-feature.mjs",
            "engine/dist/cli/dev-bundle.js",
            "engine/dist/cli/init-bundle.js",
            "engine/dist/cli/sync-assets-bundle.js",
            "engine/dist/cli/write-search-index-bundle.js",
        ] {
            let path = root.join(name);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(&path, b"fixture").unwrap();
            files.insert(name.to_owned(), sha(b"fixture"));
        }
        let engine=serde_json::to_vec(&serde_json::json!({"name":"@shebka/zellij","version":zellij_scaffold::app::template()["engineVersion"],"zellij":{"wrapperConfigVersion":1,"featureSelectionVersion":1}})).unwrap();
        fs::write(root.join("engine/package.json"), &engine).unwrap();
        files.insert("engine/package.json".into(), sha(&engine));
        fs::write(root.join("manifest.json"),serde_json::to_vec(&serde_json::json!({"format":1,"kind":"editable-sources","version":VERSION,"files":files})).unwrap()).unwrap();
    }
    #[test]
    fn cache_checks_identity_every_file_and_unlisted_entries() {
        let dir = tempfile::tempdir().unwrap();
        fixture(dir.path());
        let stop = AtomicBool::new(false);
        assert!(validate(dir.path(), &stop).is_ok());
        let manifest = fs::read(dir.path().join("manifest.json")).unwrap();
        let mut wrong: serde_json::Value = serde_json::from_slice(&manifest).unwrap();
        wrong["version"] = serde_json::json!("0.0.0");
        fs::write(
            dir.path().join("manifest.json"),
            serde_json::to_vec(&wrong).unwrap(),
        )
        .unwrap();
        assert!(validate(dir.path(), &stop).is_err());
        fs::write(dir.path().join("manifest.json"), manifest).unwrap();
        fs::write(dir.path().join("engine/extra.mjs"), "unlisted").unwrap();
        assert!(validate(dir.path(), &stop).is_err());
        fs::remove_file(dir.path().join("engine/extra.mjs")).unwrap();
        fs::write(dir.path().join("engine/src/index.ts"), "tampered").unwrap();
        assert!(validate(dir.path(), &stop).is_err());
        fixture(dir.path());
        fs::remove_file(dir.path().join("app/.npmrc")).unwrap();
        assert!(validate(dir.path(), &stop).is_err());
    }
    #[test]
    fn cache_copy_is_independent_and_cancellation_does_not_publish() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("source");
        fs::create_dir(&source).unwrap();
        fixture(&source);
        let stop = AtomicBool::new(false);
        let package = Package {
            manifest: validate(&source, &stop).unwrap(),
            root: source.clone(),
        };
        let output = utf8(&dir.path().join("out")).unwrap();
        package.copy("engine", &output, &stop).unwrap();
        fs::write(output.join("src/index.ts"), "edited").unwrap();
        assert_eq!(
            fs::read(source.join("engine/src/index.ts")).unwrap(),
            b"fixture"
        );
        stop.store(true, Ordering::Release);
        assert!(
            package
                .copy("app", &utf8(&dir.path().join("cancelled")).unwrap(), &stop)
                .is_err()
        );
        assert!(!dir.path().join("cancelled").exists());
    }
    #[test]
    fn staged_app_publishes_nested_paths_and_preserves_git_metadata() {
        let dir = tempfile::tempdir().unwrap();
        let stage = utf8(&dir.path().join("stage")).unwrap();
        let out = utf8(&dir.path().join("app")).unwrap();
        fs::create_dir_all(stage.join("docs/zellij/content")).unwrap();
        fs::write(stage.join("docs/zellij/content/index.mdx"), "# Nested").unwrap();
        fs::create_dir_all(out.join(".git")).unwrap();
        fs::write(out.join(".git/config"), "metadata").unwrap();
        publish(&stage, &out, &AtomicBool::new(false)).unwrap();
        assert_eq!(
            fs::read_to_string(out.join("docs/zellij/content/index.mdx")).unwrap(),
            "# Nested"
        );
        assert_eq!(
            fs::read_to_string(out.join(".git/config")).unwrap(),
            "metadata"
        );
        fs::write(stage.join("docs/zellij/content/index.mdx"), "replacement").unwrap();
        assert!(publish(&stage, &out, &AtomicBool::new(false)).is_err());
        assert_eq!(
            fs::read_to_string(out.join("docs/zellij/content/index.mdx")).unwrap(),
            "# Nested"
        );
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
            "engine/node_modules/code.js",
            "engine/Node_Modules/code.js",
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
    #[cfg(unix)]
    #[test]
    fn cached_links_and_directory_case_aliases_are_rejected() {
        let dir = tempfile::tempdir().unwrap();
        fixture(dir.path());
        let stop = AtomicBool::new(false);
        let file = dir.path().join("engine/src/index.ts");
        fs::remove_file(&file).unwrap();
        std::os::unix::fs::symlink("../../app/package.json", &file).unwrap();
        assert!(validate(dir.path(), &stop).is_err());
        fs::remove_file(&file).unwrap();
        fixture(dir.path());
        let file = dir.path().join("manifest.json");
        let mut manifest: serde_json::Value =
            serde_json::from_slice(&fs::read(&file).unwrap()).unwrap();
        manifest["files"]["ENGINE/other.ts"] = serde_json::json!(sha(b"fixture"));
        fs::write(&file, serde_json::to_vec(&manifest).unwrap()).unwrap();
        assert!(validate(dir.path(), &stop).is_err());
    }
    #[cfg(unix)]
    #[test]
    fn literal_backslash_file_cannot_alias_a_manifested_nested_path() {
        let dir = tempfile::tempdir().unwrap();
        fixture(dir.path());
        fs::copy(
            dir.path().join("app/package.json"),
            dir.path().join(r"app\package.json"),
        )
        .unwrap();
        assert!(
            validate(dir.path(), &AtomicBool::new(false))
                .err()
                .unwrap()
                .to_string()
                .contains("unlisted")
        );
    }
}
