//! Direct Rust supervision of the optional PDF runner. The CLI cache must verify
//! the executable, script and resource package before calling this interface.
//! No runtime is downloaded here; this does not select the installed PDF backend.
use serde::Deserialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::Path,
    process::Command,
    sync::atomic::{AtomicBool, Ordering},
    thread,
    time::{Duration, Instant},
};
use zellij_core::{Error, Result, process::ProtocolChild, site::is_link};

pub struct Runner<'a> {
    pub node: &'a Path,
    pub script: &'a Path,
    pub bundle: &'a Path,
}

#[derive(Clone, Copy)]
pub struct Options {
    pub timeout: Duration,
    pub pass_timeout: Duration,
}
impl Default for Options {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(600),
            pass_timeout: Duration::from_secs(120),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(tag = "event", rename_all = "lowercase", deny_unknown_fields)]
pub enum Event {
    Ready { pid: u32, node: String },
    Pass { pid: u32, pass: u8 },
    Complete { pid: u32 },
}
impl Event {
    pub fn pid(&self) -> u32 {
        match self {
            Self::Ready { pid, .. } | Self::Pass { pid, .. } | Self::Complete { pid } => *pid,
        }
    }
}

fn fail(message: impl std::fmt::Display) -> Error {
    Error::PdfFailed(format!("{message}; previous output was preserved"))
}
fn active(stop: &AtomicBool) -> Result<()> {
    if stop.load(Ordering::Acquire) {
        return Err(fail("PDF process cancelled"));
    }
    Ok(())
}
fn absent(path: &Path) -> Result<()> {
    match fs::symlink_metadata(path) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e.into()),
        Ok(_) => Err(fail("PDF process output already exists")),
    }
}
fn regular(path: &Path, maximum: u64) -> Result<Vec<u8>> {
    let info = fs::symlink_metadata(path)?;
    if !info.is_file() || is_link(&info) || info.len() > maximum {
        return Err(fail("invalid PDF process artifact"));
    }
    let bytes = fs::read(path)?;
    if bytes.len() as u64 != info.len() {
        return Err(fail("PDF process artifact changed"));
    }
    Ok(bytes)
}
fn milliseconds(value: Duration, maximum: u64) -> Result<u64> {
    let ms = value.as_millis();
    if ms == 0 || ms > u128::from(maximum) || value.subsec_nanos() % 1_000_000 != 0 {
        return Err(fail("invalid PDF process deadline"));
    }
    Ok(ms as u64)
}

/// Compile into a new destination. The caller retains authored-output/revision
/// checks; this function owns and cleans only its private working directory.
pub fn run(
    runner: Runner<'_>,
    input: &Path,
    output: &Path,
    stop: &AtomicBool,
    options: Options,
    mut observe: impl FnMut(&Event) -> Result<()>,
) -> Result<Value> {
    let total_ms = milliseconds(options.timeout, 600_000)?;
    let pass_ms = milliseconds(options.pass_timeout, 120_000)?;
    active(stop)?;
    if !runner.node.is_absolute() || !runner.script.is_absolute() {
        return Err(fail(
            "PDF runner requires absolute executable and script paths",
        ));
    }
    let destination = std::path::absolute(output)?;
    absent(&destination)?;
    let parent = destination
        .parent()
        .ok_or_else(|| fail("PDF output has no parent"))?;
    fs::create_dir_all(parent)?;
    let stage = tempfile::Builder::new()
        .prefix(".pdf-rust-")
        .tempdir_in(parent)?;
    let result = stage.path().join("result");
    let mut command = Command::new(runner.node);
    command
        .args(["--max-old-space-size=128"])
        .arg(runner.script)
        .arg("--run")
        .arg(std::path::absolute(runner.bundle)?)
        .arg(std::path::absolute(input)?)
        .arg(&result)
        .arg(total_ms.to_string())
        .arg(pass_ms.to_string())
        .current_dir(stage.path())
        .env_clear()
        .env("LANG", "C.UTF-8")
        .env("TZ", "UTC")
        .env("NODE_V8_COVERAGE", "");
    for name in ["SystemRoot", "WINDIR", "TEMP", "TMP", "TMPDIR"] {
        if let Some(value) = std::env::var_os(name) {
            command.env(name, value);
        }
    }
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000); // CREATE_NO_WINDOW
    }
    // Declared after staging: its guard always reaps before stage cleanup.
    let mut child = ProtocolChild::spawn(&mut command, 16 * 1024, 64 * 1024)
        .map_err(|e| fail(format!("PDF process could not start: {e}")))?;
    let started = Instant::now();
    let mut pending = Vec::new();
    let mut diagnostics = Vec::new();
    let mut node = None;
    let mut passes = 0;
    let mut complete = false;
    loop {
        if stop.load(Ordering::Acquire) || started.elapsed() >= options.timeout {
            child.cancel()?;
            active(stop)?;
            return Err(fail("PDF process deadline exceeded"));
        }
        let delta = child
            .poll()
            .map_err(|e| fail(format!("PDF process: {e}")))?;
        pending.extend(delta.stdout);
        diagnostics.extend(delta.stderr);
        while let Some(end) = pending.iter().position(|b| *b == b'\n') {
            let event: Event = serde_json::from_slice(&pending[..end])
                .map_err(|_| fail("invalid PDF process event"))?;
            pending.drain(..=end);
            if event.pid() != child.id() || complete {
                return Err(fail("invalid PDF process event"));
            }
            match &event {
                Event::Ready { node: version, .. }
                    if node.is_none()
                        && passes == 0
                        && version.len() <= 40
                        && version.starts_with('v')
                        && version[1..].split('.').count() == 3
                        && version[1..]
                            .split('.')
                            .all(|v| !v.is_empty() && v.bytes().all(|b| b.is_ascii_digit())) =>
                {
                    node = Some(version.clone());
                }
                Event::Pass { pass, .. } if node.is_some() && *pass == passes + 1 && *pass <= 3 => {
                    passes = *pass
                }
                Event::Complete { .. } if node.is_some() && passes == 3 => complete = true,
                _ => return Err(fail("invalid PDF process event sequence")),
            }
            observe(&event)?;
        }
        if let Some(status) = delta.status {
            if !status.success() || !complete || !pending.is_empty() {
                let tail = &diagnostics[diagnostics.len().saturating_sub(16 * 1024)..];
                return Err(fail(format!(
                    "PDF process failed ({status}): {}",
                    String::from_utf8_lossy(tail)
                )));
            }
            break;
        }
        thread::sleep(Duration::from_millis(10));
    }
    active(stop)?;
    let mut evidence = validate(&result)?;
    evidence["execution"] = json!({ "kind": "rust-process", "node": node.unwrap() });
    fs::write(
        result.join("probe.json"),
        serde_json::to_vec_pretty(&evidence)
            .map_err(|e| fail(format!("invalid PDF receipt: {e}")))?,
    )?;
    active(stop)?;
    absent(&destination)?;
    active(stop)?;
    fs::rename(&result, &destination)?;
    Ok(evidence)
}

fn validate(result: &Path) -> Result<Value> {
    let info = fs::symlink_metadata(result)?;
    if !info.is_dir() || is_link(&info) {
        return Err(fail("invalid PDF process output directory"));
    }
    let mut names = fs::read_dir(result)?
        .map(|e| e.map(|e| e.file_name()))
        .collect::<std::io::Result<Vec<_>>>()?;
    names.sort();
    if names
        != [
            "book.log",
            "book.pdf",
            "pass-1.log",
            "pass-2.log",
            "pass-3.log",
            "probe.json",
        ]
        .map(std::ffi::OsString::from)
    {
        return Err(fail("unexpected PDF process output files"));
    }
    let evidence: Value =
        serde_json::from_slice(&regular(&result.join("probe.json"), 8 * 1024 * 1024)?)
            .map_err(|_| fail("invalid PDF process receipt"))?;
    let pdf = regular(&result.join("book.pdf"), 128 * 1024 * 1024)?;
    let final_log = regular(&result.join("book.log"), 8 * 1024 * 1024)?;
    for pass in 1..=3 {
        let log = regular(&result.join(format!("pass-{pass}.log")), 8 * 1024 * 1024)?;
        if pass == 3 && log != final_log {
            return Err(fail("PDF final log differs from the last pass"));
        }
    }
    let passes = evidence["passes"].as_array();
    if evidence["pdfSha256"] != format!("{:x}", Sha256::digest(&pdf))
        || !passes
            .is_some_and(|p| p.len() == 3 && p.iter().enumerate().all(|(i, p)| p["pass"] == i + 1))
        || !pdf.starts_with(b"%PDF-")
        || !pdf[pdf.len().saturating_sub(1024)..]
            .windows(5)
            .any(|w| w == b"%%EOF")
    {
        return Err(fail("invalid PDF process result"));
    }
    Ok(evidence)
}
