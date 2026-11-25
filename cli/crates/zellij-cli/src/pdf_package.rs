//! Verified PDF package acquisition. Released and qualification caches are separate.
use reqwest::Url;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, File},
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::atomic::AtomicBool,
};
use zellij_core::{Error, Result, site::is_link};

const VERSION: &str = env!("CARGO_PKG_VERSION");
const ARCHIVE_LIMIT: u64 = 32 * 1024 * 1024;
const EXPANDED_LIMIT: u64 = 128 * 1024 * 1024;
const FILE_LIMIT: u64 = 64 * 1024 * 1024;
const MANIFEST_LIMIT: u64 = 1024 * 1024;
const MAX_FILES: usize = 2048;
const RUNNER_FILES: [&str; 7] = [
    "bundle.mjs",
    "font-inputs.json",
    "format-inputs.json",
    "lock.json",
    "pass.mjs",
    "probe.mjs",
    "process.mjs",
];

fn fail(message: impl Into<String>) -> Error {
    Error::InvalidArgument(format!("PDF package: {}", message.into()))
}
fn check_stop(stop: &AtomicBool) -> Result<()> {
    crate::package_transfer::check_stop(stop, "PDF package")
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct SourceArchive {
    name: String,
    bytes: u64,
    sha256: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Record {
    bytes: u64,
    sha256: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    format: u32,
    kind: String,
    version: String,
    redistribution_review_complete: bool,
    node_major: u32,
    source_archive: SourceArchive,
    source_manifest_sha256: String,
    files: BTreeMap<String, Record>,
}

pub(crate) struct Verified {
    pub root: PathBuf,
    pub manifest_sha256: String,
    pub source_archive: SourceArchive,
}
impl Verified {
    pub fn script(&self) -> PathBuf {
        self.root.join("runner").join("process.mjs")
    }
    pub fn resources(&self) -> PathBuf {
        self.root.join("resources")
    }
}

fn hash_name(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c))
}
fn safe_name(name: &str) -> bool {
    !name.is_empty()
        && name.split('/').all(|p| {
            let stem = p.split('.').next().unwrap_or("").to_ascii_uppercase();
            !p.is_empty()
                && p != "."
                && p != ".."
                && !p.ends_with(['.', ' '])
                && p.bytes()
                    .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'-' | b'_'))
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
    let info = fs::symlink_metadata(path)?;
    if !info.is_file() || is_link(&info) {
        return Err(fail("expected a regular file without links or junctions"));
    }
    Ok(info)
}
fn transfer(
    input: &mut impl Read,
    output: &mut impl Write,
    limit: u64,
    stop: &AtomicBool,
) -> Result<(u64, String)> {
    let mut hash = Sha256::new();
    let mut size = 0;
    let mut buffer = [0; 65536];
    loop {
        check_stop(stop)?;
        let n = input.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        size += n as u64;
        if size > limit {
            return Err(fail("file exceeds its size limit"));
        }
        check_stop(stop)?;
        hash.update(&buffer[..n]);
        output.write_all(&buffer[..n])?;
    }
    check_stop(stop)?;
    Ok((size, format!("{:x}", hash.finalize())))
}

fn validate(root: &Path, stop: &AtomicBool, released: bool) -> Result<Verified> {
    validate_files(root, stop, released)
        .map_err(|error| Error::InvalidArgument(format!("{error} (PDF cache: {})", root.display())))
}
fn validate_files(root: &Path, stop: &AtomicBool, released: bool) -> Result<Verified> {
    check_stop(stop)?;
    let metadata = fs::symlink_metadata(root)?;
    if !metadata.is_dir() || is_link(&metadata) {
        return Err(fail("cache must be a real directory"));
    }
    let file = root.join("manifest.json");
    if regular(&file)?.len() > MANIFEST_LIMIT {
        return Err(fail("manifest exceeds its size limit"));
    }
    let mut bytes = Vec::new();
    let (_, manifest_sha256) = transfer(&mut File::open(file)?, &mut bytes, MANIFEST_LIMIT, stop)?;
    let manifest: Manifest =
        serde_json::from_slice(&bytes).map_err(|_| fail("invalid manifest"))?;
    if manifest.format != if released { 2 } else { 1 }
        || manifest.kind
            != if released {
                "zellij-pdf-toolchain"
            } else {
                "experimental-pdf-toolchain"
            }
        || manifest.version != VERSION
        || manifest.redistribution_review_complete != released
        || manifest.node_major != 24
        || manifest.files.is_empty()
        || manifest.files.len() > MAX_FILES
        || manifest.source_archive.name
            != if released {
                format!("zel-pdf-sources-v{VERSION}.tar.gz")
            } else {
                "experimental-pdf-sources.tar.gz".into()
            }
        || manifest.source_archive.bytes == 0
        || manifest.source_archive.bytes > 1024 * 1024 * 1024
        || !hash_name(&manifest.source_archive.sha256)
        || !hash_name(&manifest.source_manifest_sha256)
    {
        return Err(fail("incompatible PDF package or source identity"));
    }
    for required in RUNNER_FILES
        .map(|n| format!("runner/{n}"))
        .into_iter()
        .chain([
            "resources/manifest.json".into(),
            "notices/NOTICES.json".into(),
            "notices/ZELLIJ-ADAPTER-NOTICE.txt".into(),
            "notices/README.md".into(),
            "notices/originals/ec/00files.txt".into(),
            "notices/originals/ec/copyrite.txt".into(),
        ])
    {
        if !manifest.files.contains_key(&required) {
            return Err(fail(format!("missing {required}")));
        }
    }
    if released {
        for required in [
            "notices/ZELLIJ-PDF-ADAPTER-LICENSE.txt",
            "notices/DISTRIBUTION-REVIEW.json",
            "notices/originals/url/tex/latex/url/url.sty",
            "notices/originals/url/doc/latex/url/url.tex",
        ] {
            if !manifest.files.contains_key(required) {
                return Err(fail(format!("missing {required}")));
            }
        }
    }
    let mut dirs = BTreeSet::new();
    let mut folded = BTreeMap::new();
    let mut total = 0u64;
    for (name, record) in &manifest.files {
        check_stop(stop)?;
        let allowed = name
            .strip_prefix("runner/")
            .is_some_and(|n| RUNNER_FILES.contains(&n))
            || name
                .strip_prefix("resources/")
                .is_some_and(|n| n == "manifest.json" || hash_name(n))
            || name.starts_with("notices/");
        if !safe_name(name) || !allowed || !hash_name(&record.sha256) || record.bytes > FILE_LIMIT {
            return Err(fail("invalid file inventory"));
        }
        total += record.bytes;
        if total > EXPANDED_LIMIT {
            return Err(fail("expanded package exceeds its size limit"));
        }
        let mut current = String::new();
        let mut parts = name.split('/').peekable();
        while let Some(part) = parts.next() {
            if !current.is_empty() {
                current.push('/');
            }
            current.push_str(part);
            if let Some(old) = folded.insert(current.to_ascii_lowercase(), current.clone())
                && old != current
            {
                return Err(fail("case-colliding package paths"));
            }
            if parts.peek().is_some() {
                dirs.insert(current.clone());
            }
        }
    }
    let mut found = BTreeSet::new();
    for entry in walkdir::WalkDir::new(root).follow_links(false).min_depth(1) {
        check_stop(stop)?;
        let entry = entry.map_err(|_| fail("cannot inspect cached files"))?;
        let name = entry
            .path()
            .strip_prefix(root)
            .unwrap()
            .to_str()
            .ok_or_else(|| fail("non-UTF-8 cache path"))?
            .replace(std::path::MAIN_SEPARATOR, "/");
        let info = fs::symlink_metadata(entry.path())?;
        if is_link(&info) {
            return Err(fail("cache contains a link or junction"));
        }
        if info.is_dir() {
            if !dirs.contains(&name) {
                return Err(fail("cache contains an unlisted directory"));
            }
        } else {
            if !info.is_file() {
                return Err(fail("cache contains a special file"));
            }
            if name == "manifest.json" {
                continue;
            }
            let record = manifest
                .files
                .get(&name)
                .ok_or_else(|| fail("cache contains an unlisted file"))?;
            if info.len() != record.bytes {
                return Err(fail(format!("size mismatch: {name}")));
            }
            let (_, hash) = transfer(
                &mut File::open(entry.path())?,
                &mut std::io::sink(),
                record.bytes,
                stop,
            )?;
            if hash != record.sha256 {
                return Err(fail(format!("checksum mismatch: {name}")));
            }
            found.insert(name);
        }
    }
    if found.len() != manifest.files.len() {
        return Err(fail("cache is incomplete"));
    }
    check_stop(stop)?;
    Ok(Verified {
        root: fs::canonicalize(root)?,
        manifest_sha256,
        source_archive: manifest.source_archive,
    })
}

fn extract(archive: &Path, into: &Path, stop: &AtomicBool) -> Result<()> {
    let mut reader = tar::Archive::new(
        flate2::read::GzDecoder::new(File::open(archive)?).take(EXPANDED_LIMIT + 1),
    );
    let mut total = 0u64;
    let mut names = BTreeSet::new();
    for entry in reader.entries()? {
        check_stop(stop)?;
        let mut entry = entry?;
        let name = entry
            .path()?
            .to_str()
            .ok_or_else(|| fail("non-UTF-8 archive path"))?
            .to_owned();
        if !entry.header().entry_type().is_file()
            || !safe_name(&name)
            || !names.insert(name.to_ascii_lowercase())
        {
            return Err(fail("unsafe, duplicate or non-file archive entry"));
        }
        total = total
            .checked_add(entry.size())
            .ok_or_else(|| fail("archive size overflow"))?;
        if total > EXPANDED_LIMIT || entry.size() > FILE_LIMIT || names.len() > MAX_FILES + 1 {
            return Err(fail("archive exceeds expansion limits"));
        }
        let dest = into.join(&name);
        fs::create_dir_all(dest.parent().unwrap())?;
        let mut output = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(dest)?;
        if transfer(&mut entry, &mut output, FILE_LIMIT, stop)?.0 != entry.size() {
            return Err(fail("truncated package archive"));
        }
    }
    let mut decoder = reader.into_inner();
    transfer(&mut decoder, &mut std::io::sink(), EXPANDED_LIMIT, stop)?;
    if decoder.limit() == 0 {
        return Err(fail("archive exceeds expansion limits"));
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

/// Read-only preflight: inspect the released cache without fetching or executing.
pub(crate) fn inspect() -> Result<serde_json::Value> {
    let installed = crate::user_home::root()?.join("cache/pdf").join(VERSION);
    let cached = present(&installed)?;
    if cached {
        validate(&installed, &AtomicBool::new(false), true)?;
    }
    Ok(
        serde_json::json!({"status":if cached {"cached"} else {"download_required"},
        "version":VERSION,"cache":installed.to_string_lossy()}),
    )
}

/// Only the maintainer harness may select the qualification profile.
pub(crate) fn obtain(
    base: &str,
    offline: bool,
    stop: &AtomicBool,
    released: bool,
) -> Result<Verified> {
    check_stop(stop)?;
    let cache = crate::user_home::root()?.join("cache").join(if released {
        "pdf"
    } else {
        "pdf-qualification"
    });
    let installed = cache.join(VERSION);
    if present(&installed)? {
        return validate(&installed, stop, released);
    }
    if offline {
        return Err(fail(
            "compiler is not cached; offline acquisition is unavailable",
        ));
    }
    let base = Url::parse(&(base.trim_end_matches('/').to_owned() + "/"))
        .map_err(|_| fail("invalid package URL"))?;
    if !(base.scheme() == "https"
        || base.scheme() == "http"
            && matches!(base.host_str(), Some("localhost" | "127.0.0.1" | "[::1]")))
        || !base.username().is_empty()
        || base.password().is_some()
        || base.query().is_some()
        || base.fragment().is_some()
    {
        return Err(fail(
            "package URL must use HTTPS or loopback HTTP without credentials, query or fragment",
        ));
    }
    fs::create_dir_all(&cache)?;
    let stage = tempfile::Builder::new()
        .prefix(".download-")
        .tempdir_in(&cache)?;
    let name = if released {
        format!("zel-pdf-v{VERSION}.tar.gz")
    } else {
        format!("zel-pdf-qualification-v{VERSION}.tar.gz")
    };
    let mut checksum = Vec::new();
    crate::package_transfer::download(
        base.join(&(name.clone() + ".sha256")).unwrap(),
        1024,
        &mut checksum,
        stop,
        "PDF package",
    )?;
    let checksum = String::from_utf8(checksum).map_err(|_| fail("invalid checksum"))?;
    let fields: Vec<_> = checksum.split_whitespace().collect();
    if fields.len() != 2 || fields[1] != name || !hash_name(fields[0]) {
        return Err(fail("invalid checksum"));
    }
    let archive = stage.path().join("download.tar.gz");
    crate::package_transfer::download(
        base.join(&name).unwrap(),
        ARCHIVE_LIMIT,
        &mut File::create(&archive)?,
        stop,
        "PDF package",
    )?;
    if transfer(
        &mut File::open(&archive)?,
        &mut std::io::sink(),
        ARCHIVE_LIMIT,
        stop,
    )?
    .1 != fields[0]
    {
        return Err(fail(
            "archive checksum mismatch; compiler was not installed or executed",
        ));
    }
    let unpacked = stage.path().join("package");
    fs::create_dir(&unpacked)?;
    extract(&archive, &unpacked, stop)?;
    validate(&unpacked, stop, released)?;
    check_stop(stop)?;
    if let Err(error) = fs::rename(&unpacked, &installed)
        && !present(&installed)?
    {
        return Err(error.into());
    }
    validate(&installed, stop, released)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};
    static STOP: AtomicBool = AtomicBool::new(false);

    fn fixture(root: &Path) -> Value {
        fs::create_dir(root).unwrap();
        let mut files = BTreeMap::new();
        let names = RUNNER_FILES
            .map(|n| format!("runner/{n}"))
            .into_iter()
            .chain([
                "resources/manifest.json".into(),
                "notices/NOTICES.json".into(),
                "notices/ZELLIJ-ADAPTER-NOTICE.txt".into(),
                "notices/README.md".into(),
                "notices/originals/ec/00files.txt".into(),
                "notices/originals/ec/copyrite.txt".into(),
            ]);
        for name in names {
            let path = root.join(&name);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(&path, b"synthetic").unwrap();
            files.insert(
                name,
                json!({"bytes":9,"sha256":format!("{:x}",Sha256::digest(b"synthetic"))}),
            );
        }
        let manifest = json!({"format":1,"kind":"experimental-pdf-toolchain","version":VERSION,
            "redistribution_review_complete":false,"node_major":24,
            "source_archive":{"name":"experimental-pdf-sources.tar.gz","bytes":100,"sha256":"a".repeat(64)},
            "source_manifest_sha256":"b".repeat(64),"files":files});
        save(root, &manifest);
        manifest
    }
    fn save(root: &Path, manifest: &Value) {
        fs::write(
            root.join("manifest.json"),
            serde_json::to_vec(manifest).unwrap(),
        )
        .unwrap();
    }
    fn message(root: &Path) -> String {
        match validate(root, &STOP, false) {
            Ok(_) => panic!("accepted invalid cache"),
            Err(e) => e.to_string(),
        }
    }
    #[test]
    fn cache_requires_exact_version_source_and_qualification_state() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("resources café 日本");
        let original = fixture(&root);
        let package = validate(&root, &STOP, false).unwrap();
        assert_eq!(
            package.script(),
            fs::canonicalize(&root).unwrap().join("runner/process.mjs")
        );
        for (key, value) in [
            ("version", json!("0.0.0")),
            ("format", json!(2)),
            ("node_major", json!(26)),
            ("redistribution_review_complete", json!(true)),
            ("source_manifest_sha256", json!("bad")),
            ("unknown", json!(true)),
        ] {
            let mut altered = original.clone();
            altered[key] = value;
            save(&root, &altered);
            assert!(message(&root).contains("PDF package"));
        }
        let mut altered = original;
        altered["source_archive"]["name"] = json!("../source.tar.gz");
        save(&root, &altered);
        assert!(message(&root).contains("source identity"));
    }
    #[test]
    fn released_profile_requires_matching_identity_and_distribution_notices() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("cache");
        let mut manifest = fixture(&root);
        assert!(validate(&root, &STOP, true).is_err());
        manifest["format"] = json!(2);
        manifest["kind"] = json!("zellij-pdf-toolchain");
        manifest["redistribution_review_complete"] = json!(true);
        manifest["source_archive"]["name"] = json!(format!("zel-pdf-sources-v{VERSION}.tar.gz"));
        let required = [
            "notices/ZELLIJ-PDF-ADAPTER-LICENSE.txt",
            "notices/DISTRIBUTION-REVIEW.json",
            "notices/originals/url/tex/latex/url/url.sty",
            "notices/originals/url/doc/latex/url/url.tex",
        ];
        for name in required {
            save(&root, &manifest);
            assert!(validate(&root, &STOP, true).is_err());
            let file = root.join(name);
            fs::create_dir_all(file.parent().unwrap()).unwrap();
            fs::write(file, b"synthetic").unwrap();
            manifest["files"][name] =
                json!({"bytes":9,"sha256":format!("{:x}",Sha256::digest(b"synthetic"))});
        }
        save(&root, &manifest);
        assert!(validate(&root, &STOP, true).is_ok());
        assert!(validate(&root, &STOP, false).is_err());
        for (key, value) in [
            ("version", json!("0.0.0")),
            ("format", json!(1)),
            ("kind", json!("experimental-pdf-toolchain")),
            ("redistribution_review_complete", json!(false)),
        ] {
            let mut altered = manifest.clone();
            altered[key] = value;
            save(&root, &altered);
            assert!(validate(&root, &STOP, true).is_err());
        }
    }
    #[test]
    fn changed_missing_extra_and_linked_cache_files_are_preserved_and_rejected() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("cache");
        fixture(&root);
        let script = root.join("runner/process.mjs");
        fs::write(&script, b"corrupted").unwrap();
        assert!(message(&root).contains("checksum mismatch"));
        assert_eq!(fs::read(&script).unwrap(), b"corrupted");
        fs::remove_file(&script).unwrap();
        assert!(message(&root).contains("incomplete"));
        fs::write(&script, b"synthetic").unwrap();
        fs::create_dir(root.join("extra")).unwrap();
        assert!(message(&root).contains("unlisted directory"));
        fs::remove_dir(root.join("extra")).unwrap();
        fs::write(root.join("unexpected"), b"keep").unwrap();
        assert!(message(&root).contains("unlisted file"));
        fs::remove_file(root.join("unexpected")).unwrap();
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink("missing", root.join("dangling")).unwrap();
            assert!(message(&root).contains("link or junction"));
            fs::remove_file(root.join("dangling")).unwrap();
            fs::write(root.join("runner\\process.mjs"), b"synthetic").unwrap();
            assert!(message(&root).contains("unlisted file"));
        }
    }
    #[test]
    fn manifest_rejects_aliases_and_limits_without_reading_large_files() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("cache");
        let original = fixture(&root);
        for name in [
            "notices/../escape",
            "notices/CON.txt",
            "notices/a.",
            "notices/a\\b",
            "runner/evil.mjs",
            "resources/wrong",
        ] {
            let mut altered = original.clone();
            altered["files"][name] = json!({"bytes":0,"sha256":"a".repeat(64)});
            save(&root, &altered);
            assert!(message(&root).contains("invalid file inventory"), "{name}");
        }
        let mut altered = original.clone();
        altered["files"]["notices/readme.md"] = altered["files"]["notices/README.md"].clone();
        save(&root, &altered);
        assert!(message(&root).contains("case-colliding"));
        altered = original;
        altered["files"]["runner/process.mjs"]["bytes"] = json!(FILE_LIMIT + 1);
        save(&root, &altered);
        assert!(message(&root).contains("invalid file inventory"));
    }
    #[test]
    fn cancelled_hashing_writes_nothing() {
        let mut output = Vec::new();
        assert!(transfer(&mut &b"data"[..], &mut output, 4, &AtomicBool::new(true)).is_err());
        assert!(output.is_empty());
    }
    fn archive(path: &Path, members: &[(&str, tar::EntryType)]) {
        let gz = flate2::write::GzEncoder::new(
            File::create(path).unwrap(),
            flate2::Compression::default(),
        );
        let mut tar = tar::Builder::new(gz);
        for (name, kind) in members {
            let mut header = tar::Header::new_ustar();
            header.set_size(0);
            header.set_entry_type(*kind);
            if kind.is_symlink() {
                header.set_link_name("outside").unwrap();
            }
            header.set_cksum();
            tar.append_data(&mut header, name, &b""[..]).unwrap();
        }
        tar.into_inner().unwrap().finish().unwrap();
    }
    #[test]
    fn archive_rejects_links_duplicates_case_aliases_and_broken_gzip() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("archive.gz");
        for (index, members) in [
            vec![("notices/link", tar::EntryType::Symlink)],
            vec![
                ("a", tar::EntryType::Regular),
                ("a", tar::EntryType::Regular),
            ],
            vec![
                ("a", tar::EntryType::Regular),
                ("A", tar::EntryType::Regular),
            ],
            vec![("CON.txt", tar::EntryType::Regular)],
            vec![("a", tar::EntryType::Directory)],
        ]
        .iter()
        .enumerate()
        {
            archive(&path, members);
            let output = temp.path().join(index.to_string());
            fs::create_dir(&output).unwrap();
            assert!(extract(&path, &output, &STOP).is_err());
        }
        archive(&path, &[("file", tar::EntryType::Regular)]);
        let mut bytes = fs::read(&path).unwrap();
        let last = bytes.len() - 8;
        bytes[last] ^= 1;
        fs::write(&path, bytes).unwrap();
        let output = temp.path().join("crc");
        fs::create_dir(&output).unwrap();
        assert!(extract(&path, &output, &STOP).is_err());
    }
}
