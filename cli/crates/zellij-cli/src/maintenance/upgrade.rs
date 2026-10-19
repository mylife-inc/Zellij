//! Verified, staged native upgrades. No shell, installer script or runtime.
//! Release protocol: https://docs.github.com/en/rest/releases/releases
use crate::args::UpgradeArgs;
use reqwest::{Url, blocking::Client, redirect::Policy};
use semver::Version;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::{
    fs::{self, File},
    io::{Read, Write},
    path::Path,
    process::{Command, Stdio},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};
use zellij_core::{Error, Result};

mod archive;
#[cfg(test)]
mod tests;

const RELEASES: &str = "https://api.github.com/repos/mylife-inc/releases/releases";
const DOWNLOADS: &str = "https://github.com/mylife-inc/releases/releases/download";
const ARCHIVE_LIMIT: u64 = 128 * 1024 * 1024;
const METADATA_LIMIT: u64 = 2 * 1024 * 1024;
const BINARY_LIMIT: u64 = 256 * 1024 * 1024;

fn failed(message: impl Into<String>) -> Error {
    Error::UpgradeFailed(message.into())
}
fn cancelled(stop: &AtomicBool) -> Result<()> {
    if stop.load(Ordering::Relaxed) {
        Err(failed(
            "cancelled; the installed executable was not replaced",
        ))
    } else {
        Ok(())
    }
}

fn parse_version(value: &str) -> Result<Version> {
    let version = Version::parse(value.strip_prefix('v').unwrap_or(value)).map_err(|_| {
        Error::InvalidArgument("expected an exact version such as 0.2.0 or v0.2.0-rc.1".into())
    })?;
    if !version.build.is_empty() {
        return Err(Error::InvalidArgument(
            "release versions cannot contain build metadata".into(),
        ));
    }
    Ok(version)
}

fn mirror(value: &str) -> Result<Url> {
    let url = Url::parse(value)
        .map_err(|_| Error::InvalidArgument("invalid release asset URL".into()))?;
    let loopback = matches!(url.host_str(), Some("localhost" | "127.0.0.1" | "[::1]"));
    if !(url.scheme() == "https" || url.scheme() == "http" && loopback)
        || !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
        || url.host_str().is_none()
    {
        return Err(Error::InvalidArgument("release asset URLs must be HTTPS (or loopback HTTP), without credentials, query or fragment".into()));
    }
    Ok(url)
}

fn client(base: &Url, redirects: bool) -> Result<Client> {
    let origin = base.origin();
    Client::builder()
        .user_agent(concat!("zel/", env!("CARGO_PKG_VERSION")))
        .connect_timeout(Duration::from_secs(10))
        .timeout(Duration::from_secs(60))
        .redirect(Policy::custom(move |attempt| {
            let url = attempt.url();
            if redirects
                && attempt.previous().len() < 5
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
        .map_err(|_| failed("could not initialize HTTPS downloads"))
}

fn copy_limited(
    mut input: impl Read,
    mut output: impl Write,
    limit: u64,
    stop: &AtomicBool,
) -> Result<()> {
    let mut size = 0u64;
    let mut buffer = [0u8; 65536];
    loop {
        cancelled(stop)?;
        let n = input
            .read(&mut buffer)
            .map_err(|_| failed("could not read the release download/archive"))?;
        if n == 0 {
            break;
        }
        size += n as u64;
        if size > limit {
            return Err(failed("release download/archive exceeds the size limit"));
        }
        output.write_all(&buffer[..n])?;
    }
    Ok(())
}

fn download(
    http: &Client,
    url: Url,
    output: impl Write,
    limit: u64,
    stop: &AtomicBool,
) -> Result<bool> {
    cancelled(stop)?;
    let response = http.get(url).send().map_err(|_| {
        failed("release download failed; check your connection and release asset availability")
    })?;
    if response.status() == 404 {
        return Ok(false);
    }
    if !response.status().is_success() {
        return Err(failed(format!(
            "release download returned HTTP {}",
            response.status().as_u16()
        )));
    }
    copy_limited(response, output, limit, stop)?;
    Ok(true)
}

#[derive(Deserialize)]
struct Release {
    tag_name: String,
    draft: bool,
    prerelease: bool,
}

fn stable(releases: &[Release]) -> Option<Version> {
    releases
        .iter()
        .filter(|r| !r.draft && !r.prerelease)
        .filter_map(|r| r.tag_name.strip_prefix("zellij-v"))
        .filter_map(|tag| Version::parse(tag).ok())
        .filter(|v| v.pre.is_empty() && v.build.is_empty())
        .max()
}

fn latest(stop: &AtomicBool) -> Result<Version> {
    let base = Url::parse(RELEASES).expect("release API URL");
    let http = client(&base, false)?;
    let mut newest = None;
    for page in 1..=10 {
        cancelled(stop)?;
        let mut request = http
            .get(base.clone())
            .query(&[("per_page", 100), ("page", page)])
            .header("Accept", "application/vnd.github+json")
            .header("X-GitHub-Api-Version", "2022-11-28");
        // Tokens are sent ONLY to the fixed GitHub API URL, with redirects off.
        if let Some(token) = std::env::var("GITHUB_TOKEN")
            .ok()
            .filter(|v| !v.is_empty())
            .or_else(|| std::env::var("GH_TOKEN").ok().filter(|v| !v.is_empty()))
        {
            request = request.bearer_auth(token);
        }
        let response = request.send().map_err(|_| {
            failed("GitHub release lookup failed; check connectivity or specify an exact version")
        })?;
        if !response.status().is_success() {
            return Err(failed(format!(
                "GitHub release lookup returned HTTP {}; set GITHUB_TOKEN for rate limits or specify an exact version",
                response.status().as_u16()
            )));
        }
        let mut bytes = Vec::new();
        copy_limited(response, &mut bytes, METADATA_LIMIT, stop)?;
        let releases: Vec<Release> = serde_json::from_slice(&bytes)
            .map_err(|_| failed("invalid GitHub release metadata"))?;
        newest = newest.max(stable(&releases));
        if releases.len() < 100 {
            return newest.ok_or_else(|| {
                failed("no stable Zellij release found; specify a version to select a prerelease")
            });
        }
    }
    Err(failed(
        "release history exceeds the lookup limit; specify an exact version",
    ))
}

fn checksum(text: &str, name: &str) -> Result<Option<String>> {
    let mut found: Option<String> = None;
    for line in text.lines() {
        let mut words = line.split_whitespace();
        let Some(digest) = words.next() else {
            continue;
        };
        let Some(file) = words.next() else {
            continue;
        };
        if file
            .trim_start_matches('*')
            .strip_prefix("./")
            .unwrap_or(file.trim_start_matches('*'))
            != name
        {
            continue;
        }
        if words.next().is_some()
            || digest.len() != 64
            || !digest.bytes().all(|c| c.is_ascii_hexdigit())
        {
            return Err(failed("malformed checksum entry for the release archive"));
        }
        let value = digest.to_ascii_lowercase();
        if found.as_ref().is_some_and(|previous| previous != &value) {
            return Err(failed("conflicting archive checksums"));
        }
        found = Some(value);
    }
    Ok(found)
}

fn digest(path: &Path) -> Result<String> {
    let mut file = File::open(path)?;
    let mut hash = Sha256::new();
    std::io::copy(&mut file, &mut hash)?;
    Ok(format!("{:x}", hash.finalize()))
}

fn installation_lock(executable: &Path) -> Result<File> {
    use std::io::Seek;
    const MARKER: &str = "Zellij native upgrade lock v1\n";
    let name = executable
        .file_name()
        .ok_or_else(|| failed("executable has no filename"))?;
    let mut lock_name = std::ffi::OsString::from(".");
    lock_name.push(name);
    lock_name.push(".upgrade.lock");
    let path = executable.with_file_name(lock_name);
    let (mut file, new) = match File::options()
        .read(true)
        .write(true)
        .create_new(true)
        .open(&path)
    {
        Ok(file) => (file, true),
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
            let metadata = fs::symlink_metadata(&path)?;
            if !metadata.is_file() || zellij_core::site::is_link(&metadata) {
                return Err(failed("upgrade lock path is not a regular file"));
            }
            (File::options().read(true).write(true).open(&path)?, false)
        }
        Err(_) => {
            return Err(failed(
                "the installation directory is not writable; use its package manager or install zel in a user-writable directory",
            ));
        }
    };
    file.try_lock().map_err(|_| {
        failed("another upgrade holds the installation lock; wait for it to finish")
    })?;
    if new {
        file.write_all(MARKER.as_bytes())?;
        file.sync_all()?;
    } else {
        let mut text = String::new();
        (&mut file).take(1024).read_to_string(&mut text)?;
        if text != MARKER {
            return Err(failed(
                "upgrade lock path contains an unrecognized file; it was not modified",
            ));
        }
    }
    file.rewind()?;
    // Leave this small owned file in place: unlinking a lock lets a concurrent
    // process open a different inode. OS locks are released even after crashes.
    Ok(file)
}

fn verify_binary(path: &Path, expected: &Version, stop: &AtomicBool) -> Result<()> {
    archive::verify_target(path, crate::TARGET)?;
    let mut output = tempfile::tempfile()?;
    let mut child = Command::new(path)
        .arg("--version")
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .stdout(output.try_clone()?)
        .spawn()
        .map_err(|_| {
            failed("downloaded binary could not run on this platform; nothing was installed")
        })?;
    let deadline = Instant::now() + Duration::from_secs(5);
    let status = loop {
        if stop.load(Ordering::Relaxed)
            || Instant::now() >= deadline
            || output.metadata()?.len() > 1024
        {
            let _ = child.kill();
            let _ = child.wait();
            return Err(failed(
                "binary verification cancelled, timed out or exceeded its output limit; nothing was installed",
            ));
        }
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) => std::thread::sleep(Duration::from_millis(20)),
            Err(error) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(error.into());
            }
        }
    };
    use std::io::Seek;
    output.rewind()?;
    let mut text = String::new();
    output.take(1025).read_to_string(&mut text)?;
    if !status.success() || text.trim() != format!("zel {expected}") {
        return Err(failed(
            "binary version does not match the requested release; nothing was installed",
        ));
    }
    Ok(())
}

pub(crate) fn run(options: &UpgradeArgs) -> Result<()> {
    archive::supported(crate::TARGET)?;
    let requested = options.version.as_deref().map(parse_version).transpose()?;
    let mirror = options
        .release_base_url
        .as_deref()
        .map(mirror)
        .transpose()?;
    let stop = Arc::new(AtomicBool::new(false));
    let signal = Arc::clone(&stop);
    ctrlc::set_handler(move || signal.store(true, Ordering::Relaxed))
        .map_err(|error| failed(format!("could not install cancellation handler: {error}")))?;
    let selected = match requested {
        Some(version) => version,
        None => latest(&stop)?,
    };
    let current = parse_version(env!("CARGO_PKG_VERSION"))?;
    println!(
        "Installed zel {current}; selected release {selected} ({})",
        crate::TARGET
    );
    if selected < current {
        return Err(failed("refusing to downgrade this executable"));
    }
    if options.check {
        println!(
            "{}",
            if selected > current {
                "Upgrade available; no files changed."
            } else {
                "Already current; no files changed."
            }
        );
        return Ok(());
    }
    if selected == current && !options.force {
        println!("Already current; use --force to reinstall this version.");
        return Ok(());
    }
    cancelled(&stop)?;

    let executable = std::env::current_exe()?.canonicalize()?;
    let _lock = installation_lock(&executable)?;
    let metadata = fs::symlink_metadata(&executable)?;
    if !metadata.is_file() || zellij_core::site::is_link(&metadata) {
        return Err(failed("the current executable must be a regular file"));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if metadata.permissions().mode() & 0o6000 != 0 {
            return Err(failed("refusing to update a setuid/setgid executable"));
        }
    }
    let original = digest(&executable)?;
    let parent = executable
        .parent()
        .ok_or_else(|| failed("executable has no parent directory"))?;
    // Prove the installation is writable before downloads; all renames stay on
    // one filesystem. No fixed temporary names can overwrite authored files.
    let stage = tempfile::Builder::new().prefix(".zel-upgrade-").tempdir_in(parent)
        .map_err(|_| failed("the installation directory is not writable; use its package manager or install zel in a user-writable directory"))?;
    let name = format!(
        "zel-v{selected}-{}.{}",
        crate::TARGET,
        if cfg!(windows) { "zip" } else { "tar.gz" }
    );
    let base = mirror.unwrap_or_else(|| {
        Url::parse(&format!("{DOWNLOADS}/zellij-v{selected}")).expect("release URL")
    });
    let http = client(&base, true)?;
    let asset = |name: &str| {
        Url::parse(&format!("{}/{name}", base.as_str().trim_end_matches('/')))
            .expect("validated asset URL")
    };
    let mut expected = None;
    for filename in ["SHA256SUMS".to_owned(), format!("{name}.sha256")] {
        let mut bytes = Vec::new();
        if download(&http, asset(&filename), &mut bytes, METADATA_LIMIT, &stop)? {
            let text =
                std::str::from_utf8(&bytes).map_err(|_| failed("invalid checksum document"))?;
            expected = checksum(text, &name)?;
            if expected.is_some() {
                break;
            }
        }
    }
    let expected = expected.ok_or_else(|| {
        failed("no valid checksum found for the platform archive; nothing was installed")
    })?;
    let archive = stage.path().join("download");
    let mut file = File::create(&archive)?;
    println!("Downloading verified release archive {name}");
    if !download(&http, asset(&name), &mut file, ARCHIVE_LIMIT, &stop)? {
        return Err(failed(
            "release has no archive for this platform; nothing was installed",
        ));
    }
    file.sync_all()?;
    drop(file);
    if digest(&archive)? != expected {
        return Err(failed("checksum mismatch; nothing was installed"));
    }
    let candidate = stage
        .path()
        .join(if cfg!(windows) { "zel.exe" } else { "zel" });
    archive::extract(&archive, &candidate, cfg!(windows), &stop)?;
    verify_binary(&candidate, &selected, &stop)?;
    fs::set_permissions(&candidate, metadata.permissions())?;
    fs::remove_file(&archive)?;
    cancelled(&stop)?;
    if digest(&executable)? != original {
        return Err(failed(
            "installed executable changed during download; retry from the current version",
        ));
    }
    // No cancellation between moving the old Windows executable and committing
    // the candidate. A failed commit restores the old name before returning.
    #[cfg(windows)]
    {
        let backup = stage.path().join("zel.previous.exe");
        fs::rename(&executable, &backup)?;
        let retained = stage.keep();
        if let Err(error) = fs::rename(&candidate, &executable) {
            if let Err(restore) = fs::rename(&backup, &executable) {
                return Err(failed(format!(
                    "replacement failed ({error}); restoration failed ({restore}); the working binary is retained at {}",
                    backup.display()
                )));
            }
            let _ = fs::remove_dir_all(&retained);
            return Err(error.into());
        }
        println!(
            "Previous executable retained at {} (removable after this command exits).",
            backup.display()
        );
    }
    #[cfg(not(windows))]
    fs::rename(&candidate, &executable)?;
    println!("Upgraded {} to zel {selected}.", executable.display());
    Ok(())
}
