//! Optional, isolated LuaLaTeX execution. Never mount the source project or socket.
use camino::{Utf8Path, Utf8PathBuf};
use sha2::{Digest, Sha256};
use std::{
    fs,
    process::{Command, Stdio},
    sync::atomic::{AtomicBool, Ordering},
    thread,
    time::{Duration, Instant},
};
use zellij_core::{
    Error, Result, files,
    process::CommandExt,
    site::{Site, SiteInput},
};
use zellij_engine::Runtime;

// Official Pandoc Ubuntu variant includes LuaLaTeX and Latin Modern.
// Multi-platform index verified 2026-11-12; keep optional and separate from engine images.
pub(crate) const IMAGE: &str = "docker.io/pandoc/latex:3.11-ubuntu@sha256:24cd38c7f0e9f7020cb3faaa5122eca719f5e72039f45d15b699bb3168055d44";
const SCRIPT: &str = r#"set -eu
for pass in 1 2 3; do
  lualatex -no-shell-escape -interaction=nonstopmode -halt-on-error -file-line-error -output-directory=/out book.tex > /out/console.log 2>&1 || { cat /out/console.log; exit 1; }
done
"#;
fn failed(message: impl std::fmt::Display) -> Error {
    Error::PdfFailed(format!("{message}; previous output was preserved"))
}
pub(crate) fn standalone(args: &crate::args::PdfArgs) -> Result<()> {
    let source = crate::select_site(args.dir.as_ref())?;
    if args.check {
        zellij_docs::pdf::check(&source)?;
        zellij_core::statusln!(
            "✓ Book is compatible with the classic PDF adapter (typesetting not run)"
        );
        return Ok(());
    }
    let site = Site::load(&source)?;
    let output = args
        .out
        .as_ref()
        .map(Utf8PathBuf::from)
        .map(Ok::<_, Error>)
        .unwrap_or_else(|| {
            if site.configured {
                Ok(site.root.join("pdf"))
            } else {
                let name = site
                    .root
                    .file_name()
                    .ok_or_else(|| failed("specify --out for a filesystem root"))?;
                Ok(site.root.with_file_name(format!("{name}-pdf")))
            }
        })?;
    let stop = crate::preview::signals()?;
    let prepared = zellij_docs::pdf::prepare(
        &source,
        &output,
        zellij_docs::BuildOptions::default(),
        false,
    )?;
    let report = typeset(prepared, &stop)?.publish()?;
    for warning in report.warnings {
        zellij_core::diagnosticln!("warning: {warning}");
    }
    zellij_core::statusln!("✓ {output} (PDF)");
    Ok(())
}

/// Preparing and publishing are separate so previews can use the revision lock
/// and discard a completed PDF if inputs changed during typesetting.
pub(crate) enum PreparedSite {
    Web(zellij_docs::PreparedBuild),
    Pdf(zellij_docs::pdf::Prepared),
}
impl PreparedSite {
    pub(crate) fn publish(self) -> Result<zellij_docs::BuildReport> {
        match self {
            Self::Web(build) => build.publish(),
            Self::Pdf(build) => build.publish(),
        }
    }
}
pub(crate) fn prepare_site(
    source: &SiteInput,
    output: &Utf8Path,
    options: zellij_docs::BuildOptions<'_>,
    pdf: Option<bool>,
    stop: &AtomicBool,
) -> Result<PreparedSite> {
    let configured = Site::load(source)?.configured;
    let before = zellij_docs::watch::Snapshot::site(source, configured)?;
    let site = Site::load(source)?;
    let prepared = if pdf.unwrap_or(site.pdf.enabled) {
        let prepared = zellij_docs::pdf::prepare(source, output, options, true)?;
        typeset(prepared, stop).map(PreparedSite::Pdf)
    } else {
        zellij_docs::prepare(source, output, options).map(PreparedSite::Web)
    }?;
    if zellij_docs::watch::Snapshot::site(source, configured)? != before {
        return Err(Error::InvalidArgument("content changed during the build; retry after saving (watch mode retries automatically)".into()));
    }
    Ok(prepared)
}
pub(crate) fn build(
    source: &SiteInput,
    out: Option<&str>,
    options: zellij_docs::BuildOptions<'_>,
    pdf: Option<bool>,
) -> Result<Utf8PathBuf> {
    let output = out
        .map(Utf8PathBuf::from)
        .map(Ok)
        .unwrap_or_else(|| zellij_docs::default_output(source))?;
    let report = build_to(source, &output, options, pdf)?;
    for warning in report.warnings {
        zellij_core::diagnosticln!("warning: {warning}");
    }
    zellij_core::statusln!("✓ {output} ({} HTML pages)", report.pages);
    Ok(output)
}
pub(crate) fn build_to(
    source: &SiteInput,
    output: &Utf8Path,
    options: zellij_docs::BuildOptions<'_>,
    pdf: Option<bool>,
) -> Result<zellij_docs::BuildReport> {
    let stop = crate::preview::signals()?;
    let prepared = prepare_site(source, output, options, pdf, &stop)?;
    if stop.load(Ordering::Acquire) {
        return Err(failed("cancelled"));
    }
    prepared.publish()
}

fn typeset(
    prepared: zellij_docs::pdf::Prepared,
    stop: &AtomicBool,
) -> Result<zellij_docs::pdf::Prepared> {
    // Validate all content and ownership before contacting a runtime.
    if stop.load(Ordering::Acquire) {
        return Err(failed("cancelled"));
    }
    let runtime=zellij_engine::runtime::detect().map_err(|e| Error::NoEngine { reason: format!("PDF requires a running Docker, Podman or nerdctl Linux runtime. {e}\nUse `zel build --no-pdf` for a native web-only build. No runtime was installed or started.") })?;
    let rootless = runtime.rootless()?;
    let temp = tempfile::Builder::new()
        .prefix("zel-pdf-input-")
        .tempdir()?;
    let input =
        Utf8Path::from_path(temp.path()).ok_or_else(|| failed("temporary path is not UTF-8"))?;
    files::write_files(
        input,
        &prepared
            .source
            .files
            .iter()
            .map(|(p, b)| (p.as_path(), b.as_slice()))
            .collect::<Vec<_>>(),
        false,
    )?;
    let name = format!(
        "zel-pdf-{}-{}",
        std::process::id(),
        temp.path().file_name().unwrap().to_string_lossy()
    );
    let mut guard = Container {
        runtime: &runtime,
        name,
        active: true,
    };
    let mut command = runtime.command();
    command.args([
        "run",
        "--name",
        &guard.name,
        "--network",
        "none",
        "--read-only",
        "--userns",
        "host",
        "--cap-drop",
        "ALL",
        "--security-opt",
        "no-new-privileges",
        "--pids-limit",
        "64",
        "--ulimit",
        "fsize=134217728:134217728",
        "--memory",
        "1g",
        "--cpus",
        "2",
        "--tmpfs",
        "/tmp:rw,noexec,nosuid,size=512m",
        "--workdir",
        "/input",
        "--entrypoint",
        "sh",
    ]);
    command.args([
        "--env",
        "HOME=/tmp",
        "--env",
        "TEXMFVAR=/tmp/texmf-var",
        "--env",
        "TEXMFCONFIG=/tmp/texmf-config",
        "--env",
        "openin_any=p",
        "--env",
        "openout_any=p",
    ]);
    command.args([
        "--volume",
        &mount(input, "/input", true)?,
        "--volume",
        &mount(&prepared.destination, "/out", false)?,
    ]);
    if rootless {
        command.args(["--user", "0:0"]);
    } else {
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            let meta = fs::metadata(&prepared.destination)?;
            command.args(["--user", &format!("{}:{}", meta.uid(), meta.gid())]);
        }
        #[cfg(windows)]
        command.args(["--user", "1000:1000"]);
    }
    command.args([IMAGE, "-c", SCRIPT]);
    zellij_core::statusln!(
        "  PDF: LuaLaTeX in the pinned optional toolchain (first use may download its image)"
    );
    let result = execute(&mut command, stop, Duration::from_secs(600));
    let cleaned = guard.remove();
    result?;
    cleaned?;
    if stop.load(Ordering::Acquire) {
        return Err(failed("cancelled"));
    }
    let log = regular(&prepared.destination.join("book.log"), 8 * 1024 * 1024)?;
    let text = String::from_utf8_lossy(&log);
    for marker in [
        "Missing character:",
        "Undefined control sequence",
        "undefined references",
        "undefined citations",
        "has been referenced but does not exist",
        "Rerun to get cross-references right",
        "Overfull \\hbox",
        "Overfull \\vbox",
    ] {
        if text.contains(marker) {
            let detail = text
                .lines()
                .find(|line| line.contains(marker))
                .unwrap_or(marker);
            return Err(failed(format!(
                "typesetting reported {detail}; adjust the affected content or use the web output"
            )));
        }
    }
    let pdf = regular(&prepared.destination.join("book.pdf"), 128 * 1024 * 1024)?;
    if !pdf.starts_with(b"%PDF-") || !pdf.windows(5).rev().take(1024).any(|w| w == b"%%EOF") {
        return Err(failed("invalid or incomplete PDF"));
    }
    // Export only verified artifacts. Auxiliary files cannot become published links.
    for item in fs::read_dir(&prepared.destination)? {
        let item = item?;
        let path = Utf8PathBuf::from_path_buf(item.path())
            .map_err(|_| failed("non-UTF-8 toolchain output"))?;
        let meta = fs::symlink_metadata(&path)?;
        if !meta.is_file() || meta.file_type().is_symlink() {
            return Err(failed("unexpected toolchain output type"));
        }
        fs::remove_file(path)?;
    }
    files::write_files(
        &prepared.destination,
        &prepared
            .source
            .files
            .iter()
            .map(|(p, b)| (p.as_path(), b.as_slice()))
            .collect::<Vec<_>>(),
        false,
    )?;
    fs::write(prepared.destination.join("book.pdf"), &pdf)?;
    fs::write(prepared.destination.join("build.log"), log)?;
    let receipt = serde_json::json!({"schema_version":1,"backend":"lualatex","template":"classic-v1","toolchain_image":IMAGE,"revision":prepared.source.revision,"settings":prepared.source.settings,"pdf_sha256":format!("{:x}",Sha256::digest(&pdf))});
    fs::write(
        prepared.destination.join("pdf.json"),
        serde_json::to_vec_pretty(&receipt).map_err(|e| failed(e.to_string()))?,
    )?;
    if stop.load(Ordering::Acquire) {
        return Err(failed("cancelled"));
    }
    Ok(prepared)
}

fn regular(path: &Utf8Path, limit: u64) -> Result<Vec<u8>> {
    let meta = fs::symlink_metadata(path)?;
    if !meta.is_file() || meta.file_type().is_symlink() || meta.len() > limit {
        return Err(failed(
            "missing, oversized or nonregular toolchain artifact",
        ));
    }
    Ok(fs::read(path)?)
}
fn mount(path: &Utf8Path, target: &str, readonly: bool) -> Result<String> {
    let path = path.canonicalize_utf8()?;
    #[cfg(windows)]
    let value = path
        .as_str()
        .strip_prefix(r"\\?\")
        .unwrap_or(path.as_str())
        .replace('\\', "/");
    #[cfg(not(windows))]
    let value = path.to_string();
    if value.contains(',') || value.contains('\n') || (cfg!(unix) && value.contains(':')) {
        return Err(failed(
            "container mount path contains an unsupported separator",
        ));
    }
    Ok(format!(
        "{value}:{target}{}",
        if readonly { ":ro" } else { "" }
    ))
}
fn execute(command: &mut Command, stop: &AtomicBool, timeout: Duration) -> Result<()> {
    command.stdin(Stdio::null());
    let mut child = command.zel_spawn()?;
    let deadline = Instant::now() + timeout;
    loop {
        if let Some(status) = child.try_wait()? {
            return if status.success() {
                Ok(())
            } else {
                Err(failed("toolchain command failed"))
            };
        }
        if stop.load(Ordering::Acquire) || Instant::now() >= deadline {
            child.cancel()?;
            return Err(failed(if stop.load(Ordering::Acquire) {
                "cancelled".to_owned()
            } else {
                format!("container command exceeded {} seconds", timeout.as_secs())
            }));
        }
        thread::sleep(Duration::from_millis(50));
    }
}
struct Container<'a> {
    runtime: &'a Runtime,
    name: String,
    active: bool,
}
impl Container<'_> {
    fn remove(&mut self) -> Result<()> {
        let result = execute(
            self.runtime
                .command()
                .args(["rm", "--force", &self.name])
                .stdout(Stdio::null())
                .stderr(Stdio::null()),
            &AtomicBool::new(false),
            Duration::from_secs(15),
        );
        if result.is_ok() {
            self.active = false;
        }
        result
    }
}
impl Drop for Container<'_> {
    fn drop(&mut self) {
        if self.active {
            let _ = self.remove();
        }
    }
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    #[test]
    fn deadline_stops_and_reaps_an_unresponsive_tool() {
        let error = execute(
            Command::new("/bin/sh").args(["-c", "while :; do :; done"]),
            &AtomicBool::new(false),
            Duration::from_millis(100),
        )
        .unwrap_err();
        assert!(error.to_string().contains("exceeded"));
    }
}
