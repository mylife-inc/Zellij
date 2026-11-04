//! Renderer-aware image builds. Keep exports ephemeral so failed packaging
//! cannot replace a user's last build, and source/config files stay untouched.
use camino::{Utf8Path, Utf8PathBuf};
use zellij_core::site::SiteInput;
use zellij_core::{
    Result,
    delivery::{ImageKind as BuildKind, ImagePlatform, validate_image_reference},
    features::Features,
    site::{Renderer, Site},
};
use zellij_engine::{BuildOptions, ContainerEngine, static_image};

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
        .or(site.image.as_ref().and_then(|image| image.name.as_deref()))
        .map(str::to_owned)
        .unwrap_or_else(|| default_tag(site, kind.as_str()));
    validate_image_reference(&tag)?;
    Ok(tag)
}

pub(crate) fn static_image(
    dir: Option<&SiteInput>,
    tag: Option<&str>,
    platform: Option<ImagePlatform>,
    options: &BuildOptions,
) -> Result<()> {
    image(dir, tag, BuildKind::Static, platform, options)
}
pub(crate) fn next_image(
    dir: Option<&SiteInput>,
    tag: Option<&str>,
    platform: Option<ImagePlatform>,
    options: &BuildOptions,
) -> Result<()> {
    image(dir, tag, BuildKind::Next, platform, options)
}
fn image(
    dir: Option<&SiteInput>,
    tag: Option<&str>,
    kind: BuildKind,
    platform: Option<ImagePlatform>,
    options: &BuildOptions,
) -> Result<()> {
    let (site, content) = selection(dir)?;
    if kind == BuildKind::Static && site.renderer == Renderer::Portable {
        require_native_options(options)?;
    }
    let tag = selected_tag(&site, tag, kind)?;
    let platform = platform.or(site.image.as_ref().and_then(|image| image.platform));
    let engine = ContainerEngine::detect(None)?;
    zellij_core::statusln!(
        "  Content  {content}\n  Kind     {} ({:?})\n  Tag      {tag}\n",
        kind.as_str(),
        site.renderer
    );
    build_image(&site, &content, &engine, kind, &tag, platform, options)?;
    zellij_core::statusln!(
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
    platform: Option<ImagePlatform>,
    options: &BuildOptions,
) -> Result<()> {
    let features = options.features;
    if let Some(platform) = platform {
        zellij_core::statusln!("  Platform {platform}");
    }
    match (kind, site.renderer) {
        (BuildKind::Static, Renderer::Portable) => {
            require_native_options(options)?;
            let temporary = tempfile::Builder::new().prefix("zel-image-").tempdir()?;
            let export = Utf8Path::from_path(temporary.path())
                .ok_or_else(|| {
                    zellij_core::Error::InvalidArgument("temporary directory is not UTF-8".into())
                })?
                .join("site");
            let report = zellij_docs::build(
                &site.input,
                &export,
                zellij_docs::BuildOptions {
                    features,
                    ..Default::default()
                },
            )?;
            for warning in report.warnings {
                zellij_core::diagnosticln!("warning: {warning}");
            }
            std::fs::remove_file(export.join(".zellij-output"))?;
            engine.image_build_recipe(&export, tag, &static_image::portable(), platform)
        }
        (BuildKind::Static, Renderer::Hosted) => engine.image_build_recipe(
            content,
            tag,
            &static_image::hosted_with_options(engine.image(), options),
            platform,
        ),
        (BuildKind::Next, Renderer::Portable) => {
            let temporary = tempfile::Builder::new().prefix("zel-next-").tempdir()?;
            let adapted = Utf8Path::from_path(temporary.path())
                .ok_or_else(|| {
                    zellij_core::Error::InvalidArgument("temporary directory is not UTF-8".into())
                })?
                .join("content");
            let report = zellij_docs::adapt_hosted_with_options(
                &site.input,
                &adapted,
                zellij_docs::BuildOptions {
                    features,
                    ..Default::default()
                },
            )?;
            for warning in report.warnings {
                zellij_core::diagnosticln!("warning: {warning}");
            }
            let selected = features.unwrap_or(Features {
                maths: report.features.contains(&"maths"),
                diagrams: report.features.contains(&"diagrams"),
            });
            engine.image_build_with_options(
                &adapted,
                tag,
                platform,
                &BuildOptions {
                    features: Some(selected),
                    ..options.clone()
                },
            )
        }
        (BuildKind::Next, Renderer::Hosted) => {
            engine.image_build_with_options(content, tag, platform, options)
        }
    }
}

pub(crate) fn dockerfile(
    dir: Option<&SiteInput>,
    kind: ImageKind,
    options: &BuildOptions,
) -> Result<()> {
    if kind == ImageKind::Next {
        let selected = crate::select_site(dir)?;
        if (selected.explicit_config() || selected.root().is_dir())
            && Site::load(&selected)?.renderer == Renderer::Portable
        {
            return Err(zellij_core::Error::InvalidArgument(
                "a Next Dockerfile takes hosted content as its context; use `zel image next` / `zel ni` to adapt and package portable Markdown automatically".into(),
            ));
        }
        return crate::image_dockerfile(&selected, options);
    }
    let (site, _) = selection(dir)?;
    let body = match site.renderer {
        Renderer::Portable => {
            require_native_options(options)?;
            if options.features.is_some() {
                return Err(zellij_core::Error::InvalidArgument("a portable static Dockerfile packages already compiled HTML; apply --features with `zel build` or `zel image static`".into()));
            }
            static_image::portable()
        }
        Renderer::Hosted => {
            static_image::hosted_with_options(&zellij_engine::configured_image(None), options)
        }
    };
    use std::io::Write as _;
    std::io::stdout().lock().write_all(body.as_bytes())?;
    Ok(())
}

pub(crate) fn require_native_options(options: &BuildOptions) -> Result<()> {
    if !options.overrides.is_empty() {
        return Err(zellij_core::Error::InvalidArgument(
            "--base-path, --site-url and --no-check apply to hosted rendering. Portable HTML uses relative URLs at any prefix, and validates while compiling. For a hosted image from Markdown, use `zel ni`.".into()
        ));
    }
    Ok(())
}
