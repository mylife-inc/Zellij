//! A small native launcher for a verified, cached rich-renderer distribution.
mod cache;
pub(crate) use cache::inspect as inspect_cache;
use camino::Utf8Path;
use std::{fs, process::Command};
use zellij_core::{Error, Result, files, output::StaticOutput, process::CommandExt};

pub(crate) fn build(
    content: &Utf8Path,
    out: &Utf8Path,
    options: &zellij_engine::BuildOptions,
    offline: bool,
) -> Result<()> {
    // Refuse an unsafe destination before any network or runtime work.
    let content = files::absolute_path(content)?;
    let output = StaticOutput::prepare(&content, out)?;
    let engine = cache::obtain(offline)?;
    let work = tempfile::Builder::new().prefix("zel-render-").tempdir()?;
    let features = options.features.unwrap_or(zellij_core::features::Features {
        maths: true,
        diagrams: true,
    });
    let selected: Vec<_> = [("maths", features.maths), ("diagrams", features.diagrams)]
        .into_iter()
        .filter_map(|(name, enabled)| enabled.then_some(name))
        .collect();
    let request = work.path().join("request.json");
    fs::write(&request, serde_json::to_vec(&serde_json::json!({
        "content":content.as_str(),"output":output.directory.as_str(),"features":selected,"noCheck":options.overrides.no_check,
    })).map_err(std::io::Error::other)?)?;
    let node = engine.join(if cfg!(windows) { "node.exe" } else { "node" });
    let mut command = Command::new(node);
    command
        .arg(engine.join("engine/engine.mjs"))
        .arg(request)
        .current_dir(work.path())
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
    zellij_core::statusln!(
        "  Engine   cached rich renderer {}\n  Content  {}\n  Target   file → {}",
        env!("CARGO_PKG_VERSION"),
        content,
        out
    );
    if !command.zel_status()?.success() {
        return Err(Error::BuildFailed);
    }
    if !output.directory.join("index.html").is_file()
        || !output
            .directory
            .join("_zellij/file/manifest.json")
            .is_file()
    {
        return Err(Error::BuildFailed);
    }
    output.publish()?;
    zellij_core::statusln!(
        "  Open {} directly in your browser.",
        out.join("index.html")
    );
    Ok(())
}
