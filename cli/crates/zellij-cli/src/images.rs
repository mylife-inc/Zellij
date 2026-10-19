//! Renderer-aware image builds. Keep exports ephemeral so failed packaging
//! cannot replace a user's last build, and source/config files stay untouched.
use camino::{Utf8Path, Utf8PathBuf};
use zellij_core::site::SiteInput;
use zellij_core::{
    Result,
    delivery::{ImageKind as BuildKind, validate_image_reference},
    site::{Renderer, Site},
};
use zellij_engine::{ContainerEngine, static_image};

use crate::args::ImageKind;

pub(crate) fn selection(dir: Option<&SiteInput>) -> Result<(Site, Utf8PathBuf)> {
    let selected = crate::select_site(dir)?;
    let site = Site::load(&selected)?;
    let content = match site.renderer {
        Renderer::Portable => site.content.clone(),
        Renderer::Hosted => crate::resolve_content(&selected)?,
    };
    Ok((site, content))
}

fn default_tag(site: &Site, kind: &str) -> String {
    // New image kinds have separate, valid repository names. Preserve the
    // existing unsuffixed default only on the legacy Next build commands.
    let name = site.root.file_name().unwrap_or("zellij-site");
    let mut slug = String::new();
    for c in name.chars().take(230) {
        if c.is_ascii_alphanumeric() {
            slug.push(c.to_ascii_lowercase());
        } else if !slug.ends_with('-') {
            slug.push('-');
        }
    }
    let slug = slug.trim_matches('-');
    format!(
        "{}-{kind}:latest",
        if slug.is_empty() { "zellij-site" } else { slug }
    )
}

/// Explicit CLI tags win over saved names; local-only defaults remain distinct.
fn selected_tag(site: &Site, tag: Option<&str>, kind: BuildKind) -> Result<String> {
    let tag = tag
        .or(site.image.as_ref().map(|image| image.name.as_str()))
        .map(str::to_owned)
        .unwrap_or_else(|| default_tag(site, kind.as_str()));
    validate_image_reference(&tag)?;
    Ok(tag)
}

pub(crate) fn static_image(dir: Option<&SiteInput>, tag: Option<&str>) -> Result<()> {
    image(dir, tag, BuildKind::Static)
}
pub(crate) fn next_image(dir: Option<&SiteInput>, tag: Option<&str>) -> Result<()> {
    image(dir, tag, BuildKind::Next)
}
fn image(dir: Option<&SiteInput>, tag: Option<&str>, kind: BuildKind) -> Result<()> {
    let (site, content) = selection(dir)?;
    let tag = selected_tag(&site, tag, kind)?;
    let engine = ContainerEngine::detect(None)?;
    println!(
        "  Content  {content}\n  Kind     {} ({:?})\n  Tag      {tag}\n",
        kind.as_str(),
        site.renderer
    );
    build_image(&site, &content, &engine, kind, &tag)?;
    println!(
        "\n✓ {tag}\n  Run it:   zel image run \"{tag}\"\n  Push it:  zel image push \"{tag}\""
    );
    Ok(())
}

/// One build operation shared by local packaging and saved publication. The
/// caller resolves intent and probes the runtime once, before temporary writes.
pub(crate) fn build_image(
    site: &Site,
    content: &Utf8Path,
    engine: &ContainerEngine,
    kind: BuildKind,
    tag: &str,
) -> Result<()> {
    match (kind, site.renderer) {
        (BuildKind::Static, Renderer::Portable) => {
            let temporary = tempfile::Builder::new().prefix("zel-image-").tempdir()?;
            let export = Utf8Path::from_path(temporary.path())
                .ok_or_else(|| {
                    zellij_core::Error::InvalidArgument("temporary directory is not UTF-8".into())
                })?
                .join("site");
            let report = zellij_docs::build(&site.input, &export, Default::default())?;
            for warning in report.warnings {
                eprintln!("warning: {warning}");
            }
            std::fs::remove_file(export.join(".zellij-output"))?;
            engine.image_build_recipe(&export, tag, &static_image::portable(), None)
        }
        (BuildKind::Static, Renderer::Hosted) => {
            engine.image_build_recipe(content, tag, &static_image::hosted(engine.image()), None)
        }
        (BuildKind::Next, Renderer::Portable) => {
            let temporary = tempfile::Builder::new().prefix("zel-next-").tempdir()?;
            let adapted = Utf8Path::from_path(temporary.path())
                .ok_or_else(|| {
                    zellij_core::Error::InvalidArgument("temporary directory is not UTF-8".into())
                })?
                .join("content");
            let report = zellij_docs::adapt_hosted(&site.input, &adapted)?;
            for warning in report.warnings {
                eprintln!("warning: {warning}");
            }
            engine.image_build(&adapted, tag, Some(&report.features.join(",")), None)
        }
        (BuildKind::Next, Renderer::Hosted) => engine.image_build(content, tag, None, None),
    }
}

pub(crate) fn dockerfile(dir: Option<&SiteInput>, kind: ImageKind) -> Result<()> {
    if kind == ImageKind::Next {
        let selected = crate::select_site(dir)?;
        if (selected.explicit_config() || selected.root().is_dir())
            && Site::load(&selected)?.renderer == Renderer::Portable
        {
            return Err(zellij_core::Error::InvalidArgument(
                "a Next Dockerfile takes hosted content as its context; use `zel image next` / `zel ni` to adapt and package portable Markdown automatically".into(),
            ));
        }
        return crate::image_dockerfile(&selected);
    }
    let (site, _) = selection(dir)?;
    let body = match site.renderer {
        Renderer::Portable => static_image::portable(),
        Renderer::Hosted => static_image::hosted(&zellij_engine::configured_image(None)),
    };
    use std::io::Write as _;
    std::io::stdout().lock().write_all(body.as_bytes())?;
    Ok(())
}
