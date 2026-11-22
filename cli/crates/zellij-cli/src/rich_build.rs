//! Native launcher for verified rich rendering and nonexecuting validation.
mod cache;
pub(crate) use cache::inspect as inspect_cache;
use camino::{Utf8Path, Utf8PathBuf};
use std::{
    fs,
    process::{Command, ExitStatus},
    sync::atomic::{AtomicBool, Ordering},
    thread,
    time::Duration,
};
use zellij_core::{Error, Result, files, output::StaticOutput, process::CommandExt};
use zellij_engine::{BuildOptions, CheckOptions, HostedCheckReport};

pub(crate) fn ensure_cached(offline: bool) -> Result<()> {
    cache::obtain(offline, true).map(|_| ())
}

// Node reports ordinary Windows paths, without Rust's verbatim prefix.
fn content_path(content: &Utf8Path) -> Result<Utf8PathBuf> {
    let content = files::absolute_path(content)?;
    if cfg!(windows) {
        if let Some(unc) = content.as_str().strip_prefix(r"\\?\UNC\") {
            return Ok(format!(r"\\{unc}").into());
        }
        if let Some(ordinary) = content.as_str().strip_prefix(r"\\?\") {
            return Ok(ordinary.into());
        }
    }
    Ok(content)
}

fn execute(
    work: &std::path::Path,
    request: serde_json::Value,
    options: &BuildOptions,
    offline: bool,
    silent: bool,
    stop: Option<&AtomicBool>,
) -> Result<Option<ExitStatus>> {
    if stop.is_some_and(|s| s.load(Ordering::Acquire)) {
        return Ok(None);
    }
    let engine = cache::obtain(offline, !silent)?;
    if stop.is_some_and(|s| s.load(Ordering::Acquire)) {
        return Ok(None);
    }
    let request_path = work.join("request.json");
    fs::write(
        &request_path,
        serde_json::to_vec(&request).map_err(std::io::Error::other)?,
    )?;
    let mut command = Command::new(engine.join(if cfg!(windows) { "node.exe" } else { "node" }));
    command
        .arg(engine.join("engine/engine.mjs"))
        .arg(request_path)
        .current_dir(work)
        .env_remove("NODE_OPTIONS")
        .env_remove("NODE_PATH")
        .env_remove("ZELLIJ_BUILD_OVERRIDES")
        .env_remove("ZELLIJ_BASE_PATH")
        .env_remove("ZELLIJ_MATHS")
        .env_remove("NEXT_PUBLIC_ZELLIJ_DIAGRAMS")
        .env("NODE_ENV", "production");
    if let Some(encoded) = options.overrides.encoded() {
        command.env("ZELLIJ_BUILD_OVERRIDES", encoded);
    }
    let mut child = if silent {
        command.zel_spawn_silent()?
    } else {
        command.zel_spawn()?
    };
    loop {
        if stop.is_some_and(|s| s.load(Ordering::Acquire)) {
            child.cancel()?;
            return Ok(None);
        }
        if let Some(status) = child.try_wait()? {
            return Ok(Some(status));
        }
        thread::sleep(Duration::from_millis(20));
    }
}

fn read_report(work: &std::path::Path, name: &str) -> Result<Vec<u8>> {
    let path = work.join(name);
    if fs::metadata(&path)?.len() > 16 * 1024 * 1024 {
        return Err(Error::CheckFailed(
            "cached renderer report exceeds 16 MiB".into(),
        ));
    }
    Ok(fs::read(path)?)
}

pub(crate) fn check(
    content: &Utf8Path,
    options: CheckOptions,
    offline: bool,
    stop: Option<&AtomicBool>,
) -> Result<Option<HostedCheckReport>> {
    let content = content_path(content)?;
    let work = tempfile::Builder::new().prefix("zel-check-").tempdir()?;
    let Some(status) = execute(
        work.path(),
        serde_json::json!({
            "operation":"check", "content":content.as_str(), "features":["maths","diagrams"],
            "deep":options.deep, "strict":options.strict,
        }),
        &BuildOptions::default(),
        offline,
        true,
        stop,
    )?
    else {
        return Ok(None);
    };
    let report = read_report(work.path(), "check-report.json").map_err(|_| Error::CheckFailed(
        format!("cached renderer did not produce a validation report (exit {:?}); verify its cache with zel doctor", status.code())
    ))?;
    HostedCheckReport::decode_cached(&report, status.code(), &content, options).map(Some)
}

pub(crate) struct PreparedRich {
    output: StaticOutput,
    pub base_path: String,
}
impl PreparedRich {
    pub fn directory(&self) -> &Utf8Path {
        &self.output.directory
    }
    pub fn publish(self) -> Result<()> {
        self.output.publish()
    }
}

pub(crate) fn prepare(
    content: &Utf8Path,
    out: &Utf8Path,
    options: &BuildOptions,
    offline: bool,
    stop: Option<&AtomicBool>,
) -> Result<Option<PreparedRich>> {
    let content = content_path(content)?;
    // Refuse unsafe output before downloading or starting a renderer.
    let output = StaticOutput::prepare(&content, out)?;
    let work = tempfile::Builder::new().prefix("zel-render-").tempdir()?;
    let features = options.features.unwrap_or(zellij_core::features::Features {
        maths: true,
        diagrams: true,
    });
    let selected: Vec<_> = [("maths", features.maths), ("diagrams", features.diagrams)]
        .into_iter()
        .filter_map(|(name, enabled)| enabled.then_some(name))
        .collect();
    zellij_core::statusln!(
        "  Engine   cached rich renderer {}\n  Content  {}\n  Target   file → {}",
        env!("CARGO_PKG_VERSION"),
        content,
        out
    );
    let Some(status) = execute(
        work.path(),
        serde_json::json!({
            "operation":"build", "content":content.as_str(), "output":output.directory.as_str(),
            "features":selected, "noCheck":options.overrides.no_check,
        }),
        options,
        offline,
        false,
        stop,
    )?
    else {
        return Ok(None);
    };
    if !status.success()
        || !output.directory.join("index.html").is_file()
        || !output
            .directory
            .join("_zellij/file/manifest.json")
            .is_file()
    {
        return Err(Error::BuildFailed);
    }
    #[derive(serde::Deserialize)]
    struct Report {
        #[serde(rename = "basePath")]
        base_path: String,
    }
    let report: Report = serde_json::from_slice(&read_report(work.path(), "build-report.json")?)
        .map_err(|_| Error::BuildFailed)?;
    Ok(Some(PreparedRich {
        output,
        base_path: report.base_path,
    }))
}

pub(crate) fn build(
    content: &Utf8Path,
    out: &Utf8Path,
    options: &BuildOptions,
    offline: bool,
) -> Result<()> {
    let prepared = prepare(content, out, options, offline, None)?.ok_or(Error::BuildFailed)?;
    prepared.publish()?;
    zellij_core::statusln!(
        "  Open {} directly in your browser.",
        out.join("index.html")
    );
    Ok(())
}
