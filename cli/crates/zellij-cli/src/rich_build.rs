//! Native launcher for verified rich rendering and nonexecuting validation.
mod cache;
pub(crate) use cache::inspect as inspect_cache;
pub(crate) use cache::platform;
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

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Target {
    File,
    Static,
}
impl Target {
    pub fn name(self) -> &'static str {
        match self {
            Self::File => "file",
            Self::Static => "static",
        }
    }
}

pub(crate) fn ensure_cached(offline: bool, stop: &AtomicBool) -> Result<()> {
    cache::obtain(offline, true, stop).map(|_| ())
}

/// Reuse only the runtime from a completely verified renderer package.
pub(crate) fn private_node(offline: bool, stop: &AtomicBool) -> Result<std::path::PathBuf> {
    let root = cache::obtain(offline, true, stop)?;
    Ok(root.join(if cfg!(windows) { "node.exe" } else { "node" }))
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
    // Watchers retain their existing signal owner; one-shot commands need one
    // before acquisition so a stalled download can clean up its owned stage.
    let owned_stop;
    let stop = match stop {
        Some(stop) => stop,
        None => {
            owned_stop = crate::preview::signals()?;
            &owned_stop
        }
    };
    if stop.load(Ordering::Acquire) {
        return Ok(None);
    }
    let engine = match cache::obtain(offline, !silent, stop) {
        Ok(engine) => engine,
        Err(_) if stop.load(Ordering::Acquire) => return Ok(None),
        Err(error) => return Err(error),
    };
    if stop.load(Ordering::Acquire) {
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
        if stop.load(Ordering::Acquire) {
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
    pub fn publish_for_deployment(self) -> Result<()> {
        let prefix: zellij_core::build_overrides::BasePath =
            self.base_path.parse().map_err(Error::InvalidArgument)?;
        let prefix = prefix.to_string();
        if !prefix.is_empty() {
            let relative = Utf8Path::new(prefix.trim_start_matches('/'));
            files::relative_path(relative)?;
            let flat = self.output.directory.with_file_name("flat-export");
            // Both paths are inside the CLI-owned staging directory. Move the
            // complete flat export aside first, so a prefix such as /guide can
            // never collide with an existing guide folder while relocating it.
            fs::rename(&self.output.directory, &flat)?;
            let mounted = self.output.directory.join(relative);
            fs::create_dir_all(mounted.parent().ok_or(Error::BuildFailed)?)?;
            fs::rename(flat, mounted)?;
        }
        self.publish()
    }
    pub fn publish(self) -> Result<()> {
        self.output.publish()
    }
}

pub(crate) fn prepare(
    target: Target,
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
    // Preserve each target's published defaults. The file builder enables
    // rich features; the standard HTTP image previously selected none.
    let features = options.features.unwrap_or(zellij_core::features::Features {
        maths: target == Target::File,
        diagrams: target == Target::File,
    });
    let selected: Vec<_> = [("maths", features.maths), ("diagrams", features.diagrams)]
        .into_iter()
        .filter_map(|(name, enabled)| enabled.then_some(name))
        .collect();
    zellij_core::statusln!(
        "  Engine   cached rich renderer {}\n  Content  {}\n  Target   {} → {}",
        env!("CARGO_PKG_VERSION"),
        content,
        target.name(),
        out
    );
    let Some(status) = execute(
        work.path(),
        serde_json::json!({
            "operation":"build", "target":target.name(), "content":content.as_str(), "output":output.directory.as_str(),
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
            .join(format!("_zellij/{}/manifest.json", target.name()))
            .is_file()
    {
        return Err(Error::BuildFailed);
    }
    #[derive(serde::Deserialize)]
    struct Report {
        #[serde(default)]
        target: Option<String>,
        #[serde(rename = "basePath")]
        base_path: String,
    }
    let report: Report = serde_json::from_slice(&read_report(work.path(), "build-report.json")?)
        .map_err(|_| Error::BuildFailed)?;
    // Old renderers understand only file output. Never publish a file export
    // as an HTTP build if an older artifact ignored the requested mode.
    if report.target.as_deref().unwrap_or("file") != target.name() {
        return Err(Error::BuildFailed);
    }
    Ok(Some(PreparedRich {
        output,
        base_path: report.base_path,
    }))
}

pub(crate) fn build(
    target: Target,
    content: &Utf8Path,
    out: &Utf8Path,
    options: &BuildOptions,
    offline: bool,
) -> Result<()> {
    let stop = crate::preview::signals()?;
    let prepared =
        prepare(target, content, out, options, offline, Some(&stop))?.ok_or_else(|| {
            Error::InvalidArgument("rich build cancelled; previous output was preserved".into())
        })?;
    crate::package_transfer::check_stop(&stop, "Rich renderer")?;
    prepared.publish()?;
    if target == Target::File {
        zellij_core::statusln!(
            "  Open {} directly in your browser.",
            out.join("index.html")
        );
    } else {
        zellij_core::statusln!("✓ {out} — serve it from any static host.");
        if fs::read_to_string(out.join("sitemap.xml")).is_ok_and(|body| !body.contains("<loc>")) {
            zellij_core::diagnosticln!(
                "! The sitemap is empty. Set siteUrl in content/site.yaml, or pass --site-url, to generate absolute page URLs."
            );
        }
    }
    Ok(())
}

#[cfg(test)]
mod deployment_tests {
    use super::*;

    #[test]
    fn deployment_mount_can_share_a_name_with_an_exported_directory() {
        let temp = tempfile::tempdir().unwrap();
        let root = Utf8Path::from_path(temp.path()).unwrap();
        let content = root.join("content");
        fs::create_dir(&content).unwrap();
        let output = StaticOutput::prepare(&content, &root.join("site")).unwrap();
        fs::create_dir(output.directory.join("guide")).unwrap();
        fs::write(output.directory.join("index.html"), "home").unwrap();
        fs::write(output.directory.join("guide/next.html"), "next").unwrap();
        PreparedRich {
            output,
            base_path: "/guide".into(),
        }
        .publish_for_deployment()
        .unwrap();
        assert_eq!(
            fs::read_to_string(root.join("site/guide/index.html")).unwrap(),
            "home"
        );
        assert_eq!(
            fs::read_to_string(root.join("site/guide/guide/next.html")).unwrap(),
            "next"
        );
        assert!(!root.join("site/index.html").exists());
    }

    #[test]
    fn unsafe_deployment_mounts_never_publish_output() {
        let temp = tempfile::tempdir().unwrap();
        let root = Utf8Path::from_path(temp.path()).unwrap();
        let content = root.join("content");
        fs::create_dir(&content).unwrap();
        for base_path in ["/../escape", "/C:/escape", "/.git/hooks", "/x\\escape"] {
            let output = StaticOutput::prepare(&content, &root.join("site")).unwrap();
            fs::write(output.directory.join("index.html"), "home").unwrap();
            assert!(
                PreparedRich {
                    output,
                    base_path: base_path.into()
                }
                .publish_for_deployment()
                .is_err()
            );
            assert!(!root.join("site").exists());
            assert!(!root.join("escape").exists());
        }
    }
}
