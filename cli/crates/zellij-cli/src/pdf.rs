//! Optional PDF execution through a verified cache or an isolated container.
use camino::{Utf8Path, Utf8PathBuf};
use sha2::{Digest, Sha256};
use std::{
    fs,
    process::{Command, Stdio},
    sync::{
        OnceLock,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::{Duration, Instant},
};
use zellij_core::{
    Error, Result, files,
    process::CommandExt,
    site::{Site, SiteInput},
};
use zellij_engine::Runtime;

#[derive(Clone, Copy, Debug, PartialEq, Eq, clap::ValueEnum)]
pub(crate) enum Backend {
    Cached,
    Container,
}
static BACKEND: OnceLock<Backend> = OnceLock::new();
pub(crate) fn configure(backend: Backend) {
    let _ = BACKEND.set(backend);
}
pub(crate) fn selected_backend() -> Backend {
    *BACKEND.get().unwrap_or(&Backend::Container)
}

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
            "✓ Book is compatible with the selected PDF edition (typesetting not run)"
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
    let report = typeset(prepared, args.offline, &stop)?.publish()?;
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
    offline: bool,
    stop: &AtomicBool,
) -> Result<PreparedSite> {
    let configured = Site::load(source)?.configured;
    let before = zellij_docs::watch::Snapshot::site(source, configured)?;
    let site = Site::load(source)?;
    let prepared = if pdf.unwrap_or(site.pdf.enabled) {
        let prepared = zellij_docs::pdf::prepare(source, output, options, true)?;
        typeset(prepared, offline, stop).map(PreparedSite::Pdf)
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
    offline: bool,
) -> Result<Utf8PathBuf> {
    let output = out
        .map(Utf8PathBuf::from)
        .map(Ok)
        .unwrap_or_else(|| zellij_docs::default_output(source))?;
    let report = build_to(source, &output, options, pdf, offline)?;
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
    offline: bool,
) -> Result<zellij_docs::BuildReport> {
    let stop = crate::preview::signals()?;
    let prepared = prepare_site(source, output, options, pdf, offline, &stop)?;
    if stop.load(Ordering::Acquire) {
        return Err(failed("cancelled"));
    }
    prepared.publish()
}

/// Image preparation resolves the web and print forms once, then rejects edits
/// made while the optional typesetter was running before starting an image build.
pub(crate) fn adapt_hosted_to(
    source: &SiteInput,
    output: &Utf8Path,
    options: zellij_docs::BuildOptions<'_>,
) -> Result<zellij_docs::HostedReport> {
    let site = Site::load(source)?;
    if !site.pdf.enabled {
        return zellij_docs::adapt_hosted_with_options(source, output, options);
    }
    let before = zellij_docs::watch::Snapshot::site(source, site.configured)?;
    let stop = crate::preview::signals()?;
    let prepared = zellij_docs::pdf::prepare_hosted(source, output, options)?;
    let features = prepared.features.clone();
    let prepared = typeset(prepared, false, &stop)?;
    if stop.load(Ordering::Acquire) {
        return Err(failed("cancelled"));
    }
    if zellij_docs::watch::Snapshot::site(source, site.configured)? != before {
        return Err(failed(
            "content changed during typesetting; retry after saving",
        ));
    }
    let report = prepared.publish()?;
    // The adapted context is ephemeral, not a user-visible static export.
    fs::remove_file(output.join(".zellij-output"))?;
    Ok(zellij_docs::HostedReport {
        pages: report.pages,
        warnings: report.warnings,
        features,
    })
}

fn typeset(
    prepared: zellij_docs::pdf::Prepared,
    offline: bool,
    stop: &AtomicBool,
) -> Result<zellij_docs::pdf::Prepared> {
    match selected_backend() {
        Backend::Container => {
            if offline {
                return Err(failed("--offline PDFs require --pdf-backend cached"));
            }
            typeset_container(prepared, stop)
        }
        Backend::Cached => typeset_cached(prepared, offline, stop),
    }
}

fn typeset_cached(
    prepared: zellij_docs::pdf::Prepared,
    offline: bool,
    stop: &AtomicBool,
) -> Result<zellij_docs::pdf::Prepared> {
    if stop.load(Ordering::Acquire) {
        return Err(failed("cancelled"));
    }
    let base = std::env::var("ZEL_PDF_BASE_URL").unwrap_or_else(|_| {
        format!(
            "https://github.com/mylife-inc/releases/releases/download/zellij-v{}/",
            env!("CARGO_PKG_VERSION")
        )
    });
    let package = crate::pdf_package::obtain(&base, offline, stop, true)?;
    let node = crate::rich_build::private_node(offline, stop)?;
    let temp = tempfile::Builder::new()
        .prefix(".zel-pdf-cached-")
        .tempdir_in(
            prepared
                .destination
                .parent()
                .ok_or_else(|| failed("PDF output has no parent directory"))?,
        )?;
    let root =
        Utf8Path::from_path(temp.path()).ok_or_else(|| failed("temporary path is not UTF-8"))?;
    let input = root.join("input");
    fs::create_dir(&input)?;
    files::write_files(
        &input,
        &prepared
            .source
            .files
            .iter()
            .map(|(p, b)| (p.as_path(), b.as_slice()))
            .collect::<Vec<_>>(),
        false,
    )?;
    let output = root.join("compiled");
    zellij_core::statusln!("  PDF: verified cached LuaHBTeX compiler");
    let proof = zellij_engine::pdf_runner::run(
        zellij_engine::pdf_runner::Runner {
            node: &node,
            script: &package.script(),
            bundle: &package.resources(),
        },
        input.as_std_path(),
        output.as_std_path(),
        stop,
        Default::default(),
        |_| Ok(()),
    )?;
    let log = regular(&output.join("book.log"), 8 * 1024 * 1024)?;
    let pdf = regular(&output.join("book.pdf"), 128 * 1024 * 1024)?;
    let toolchain = serde_json::json!({"backend":"luahbtex-wasm", "toolchain_version":env!("CARGO_PKG_VERSION"),
        "toolchain_manifest_sha256":package.manifest_sha256, "source_archive":package.source_archive,
        "engine_release":proof["engineRelease"], "resource_lock_sha256":proof["resourceLockSha256"],
        "format_sha256":proof["formatSha256"], "font_metadata_sha256":proof["fontMetadataSha256"],
        "node":proof["execution"]["node"]});
    finish(prepared, log, pdf, toolchain, stop)
}

fn typeset_container(
    prepared: zellij_docs::pdf::Prepared,
    stop: &AtomicBool,
) -> Result<zellij_docs::pdf::Prepared> {
    // Validate all content and ownership before contacting a runtime.
    if stop.load(Ordering::Acquire) {
        return Err(failed("cancelled"));
    }
    let runtime=zellij_engine::runtime::detect().map_err(|e| Error::NoEngine { reason: format!("PDF requires a running Docker, Podman or nerdctl Linux runtime. {e}\nUse `zel build --no-pdf` for a native web-only build. No runtime was installed or started.") })?;
    let rootless = runtime.rootless()?;
    // VM-backed runtimes may share the output folder but not the OS temporary
    // directory (for example macOS /var/folders with Colima). Stage only the
    // generated print inputs beside the output so both mounts use that share.
    let temp = tempfile::Builder::new()
        .prefix("zel-pdf-input-")
        .tempdir_in(
            prepared
                .destination
                .parent()
                .ok_or_else(|| failed("PDF output has no parent directory"))?,
        )?;
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
    let pdf = regular(&prepared.destination.join("book.pdf"), 128 * 1024 * 1024)?;
    finish(
        prepared,
        log,
        pdf,
        serde_json::json!({"backend":"lualatex", "toolchain_image": IMAGE}),
        stop,
    )
}

fn finish(
    prepared: zellij_docs::pdf::Prepared,
    log: Vec<u8>,
    pdf: Vec<u8>,
    toolchain: serde_json::Value,
    stop: &AtomicBool,
) -> Result<zellij_docs::pdf::Prepared> {
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
    let mut receipt = serde_json::json!({"schema_version":1,"template":prepared.source.template,"revision":prepared.source.revision,"settings":prepared.source.settings,"pdf_sha256":format!("{:x}",Sha256::digest(&pdf))});
    receipt
        .as_object_mut()
        .unwrap()
        .extend(toolchain.as_object().unwrap().clone());
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
