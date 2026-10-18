//! Renderer-aware static images. Keep exports ephemeral so failed packaging
//! cannot replace a user's last build, and source/config files stay untouched.
use camino::{Utf8Path, Utf8PathBuf};
use zellij_core::{
    Result,
    site::{Renderer, Site},
};
use zellij_engine::{ContainerEngine, static_image};

use crate::args::ImageKind;

fn selection(dir: Option<&str>) -> Result<(Site, Utf8PathBuf)> {
    let selected = crate::select_site(dir)?;
    let site = Site::load(&selected)?;
    let content = match site.renderer {
        Renderer::Portable => site.content.clone(),
        Renderer::Hosted => crate::resolve_content(selected.as_str())?,
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

pub(crate) fn static_image(dir: Option<&str>, tag: Option<&str>) -> Result<()> {
    let (site, content) = selection(dir)?;
    let tag = tag
        .map(str::to_owned)
        .unwrap_or_else(|| default_tag(&site, "static"));
    // Detect before compiling or creating any temporary output. No automatic
    // runtime installation, service start, or renderer fallback.
    let engine = ContainerEngine::detect(None)?;
    println!("  Content  {content}");
    println!("  Kind     static HTML ({:?})", site.renderer);
    println!("  Tag      {tag}\n");
    match site.renderer {
        Renderer::Portable => {
            let temporary = tempfile::Builder::new().prefix("zel-image-").tempdir()?;
            let export = Utf8Path::from_path(temporary.path())
                .ok_or_else(|| {
                    zellij_core::Error::InvalidArgument("temporary directory is not UTF-8".into())
                })?
                .join("site");
            let report = zellij_docs::build(&site.root, &export, Default::default())?;
            for warning in report.warnings {
                eprintln!("warning: {warning}");
            }
            // This is our private export, not an existing user's output.
            std::fs::remove_file(export.join(".zellij-output"))?;
            engine.image_build_recipe(&export, &tag, &static_image::portable(), None)?;
        }
        Renderer::Hosted => {
            engine.image_build_recipe(
                &content,
                &tag,
                &static_image::hosted(engine.image()),
                None,
            )?;
        }
    }
    println!("\n✓ {tag}");
    println!("  Run it:   zel image run {tag}");
    println!("  Push it:  zel image push {tag} --registry ghcr.io/you");
    Ok(())
}

pub(crate) fn next_image(dir: Option<&str>, tag: Option<&str>) -> Result<()> {
    let (site, content) = selection(dir)?;
    let tag = tag
        .map(str::to_owned)
        .unwrap_or_else(|| default_tag(&site, "next"));
    let engine = ContainerEngine::detect(None)?;
    println!(
        "  Content  {content}\n  Kind     Next.js ({:?})\n  Tag      {tag}\n",
        site.renderer
    );
    match site.renderer {
        Renderer::Portable => {
            let temporary = tempfile::Builder::new().prefix("zel-next-").tempdir()?;
            let adapted = Utf8Path::from_path(temporary.path())
                .ok_or_else(|| {
                    zellij_core::Error::InvalidArgument("temporary directory is not UTF-8".into())
                })?
                .join("content");
            let report = zellij_docs::adapt_hosted(&site.root, &adapted)?;
            for warning in report.warnings {
                eprintln!("warning: {warning}");
            }
            let features = report.features.join(",");
            engine.image_build(&adapted, &tag, Some(&features), None)?;
        }
        Renderer::Hosted => engine.image_build(&content, &tag, None, None)?,
    }
    println!(
        "\n✓ {tag}\n  Run it:   zel image run {tag}\n  Push it:  zel image push {tag} --registry ghcr.io/you"
    );
    Ok(())
}

pub(crate) fn dockerfile(dir: Option<&str>, kind: ImageKind) -> Result<()> {
    if kind == ImageKind::Next {
        let selected = crate::select_site(dir)?;
        if selected.is_dir() && Site::load(&selected)?.renderer == Renderer::Portable {
            return Err(zellij_core::Error::InvalidArgument(
                "a Next Dockerfile takes hosted content as its context; use `zel image next` / `zel ni` to adapt and package portable Markdown automatically".into(),
            ));
        }
        return crate::image_dockerfile(selected.as_str());
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
