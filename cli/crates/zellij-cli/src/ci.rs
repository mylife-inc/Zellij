//! Resolve local site/repository settings, then write one reviewable workflow.
use crate::args::CiInitArgs;
use camino::{Utf8Path, Utf8PathBuf};
use std::{
    io::Write,
    process::{Command, Stdio},
};
use zellij_core::site::SiteInput;
use zellij_core::{
    Error, Result,
    delivery::{DeliveryPlan, Provider, PublishSettings},
    files,
};
use zellij_scaffold::pipeline::{self, CiOptions, Publish};

fn invalid(message: &str) -> Error {
    Error::InvalidArgument(message.into())
}
fn repository(root: &Utf8Path) -> Result<Utf8PathBuf> {
    let output = Command::new("git")
        .args(["-C", root.as_str(), "rev-parse", "--show-toplevel"])
        .output()
        .map_err(|_| invalid("zel ci requires Git to locate the repository containing the site"))?;
    if !output.status.success() {
        return Err(invalid(
            "zel ci requires a site inside a Git working tree; initialize or select the intended repository first",
        ));
    }
    let path =
        String::from_utf8(output.stdout).map_err(|_| invalid("repository path must be UTF-8"))?;
    Ok(Utf8Path::new(path.trim_end_matches(['\r', '\n'])).canonicalize_utf8()?)
}

pub(crate) fn generate(system: &str, dir: Option<&SiteInput>, args: &CiInitArgs) -> Result<()> {
    if system != "github" {
        return Err(invalid("only the github workflow generator is implemented"));
    }
    let (site, _) = crate::images::selection(dir.or(args.site_root.as_ref()))?;
    let repo = repository(&site.root)?;
    let relative = site
        .root
        .strip_prefix(&repo)
        .map_err(|_| invalid("site must be contained in its Git working tree"))?;
    let relative = if relative.as_str().is_empty() {
        ".".into()
    } else {
        relative.iter().collect::<Vec<_>>().join("/")
    };
    let mut settings = site.ci.clone();
    settings.production |= args.prod;
    let version = args
        .zel_version
        .as_deref()
        .or(settings.zel_version.as_deref())
        .unwrap_or(env!("CARGO_PKG_VERSION"));
    let version = zellij_core::ci::version_pin(version).map_err(|error| invalid(&format!("CI needs a pinned compatible release (0.2.0 or later). Set ci.zel-version or pass --zel-version, for example 0.2.0-rc.1. This binary reports {}. Generation prepares a file but does not publish or verify release availability. {error}", env!("CARGO_PKG_VERSION"))))?;
    let provider = Provider::selected(&site, args.deploy.as_deref(), None)?;
    let environment_site = provider
        .and_then(|p| std::env::var(p.site_environment()).ok())
        .filter(|s| !s.trim().is_empty());
    let publish = match (args.deploy.as_deref(), args.target.as_deref()) {
        (None, Some("static")) => Publish::Nothing,
        (None, Some("image")) => Publish::LegacyImage,
        (_, None) if provider.is_some() => {
            let provider = provider.expect("selected provider");
            let saved = match &site.publish {
                Some(PublishSettings::Deploy {
                    provider: saved,
                    site,
                }) if *saved == provider => site.as_deref(),
                _ => None,
            };
            let site_id = args
                .site_id
                .as_deref()
                .or(environment_site.as_deref())
                .or(saved);
            if let Some(id) = site_id {
                provider.validate_site(id)?;
            }
            match provider {
                Provider::Netlify => Publish::Netlify {
                    site_id: site_id.map(str::to_owned),
                },
                Provider::Vercel => Publish::Vercel {
                    project: site_id.map(str::to_owned),
                },
                Provider::Cloudflare => Publish::Cloudflare {
                    project: site_id.map(str::to_owned),
                },
            }
        }
        (None, None) if matches!(site.publish, Some(PublishSettings::Push { .. })) => {
            let DeliveryPlan::Push(plan) =
                DeliveryPlan::resolve(&site, None, None, None, None, None, false)?
            else {
                unreachable!()
            };
            Publish::Image {
                kind: plan.kind,
                tag: plan.tag,
            }
        }
        (None, None) => Publish::Nothing,
        _ => return Err(invalid("conflicting CI delivery options")),
    };
    let deployment_provider = match publish {
        Publish::Netlify { .. } => Some(Provider::Netlify),
        Publish::Cloudflare { .. } => Some(Provider::Cloudflare),
        Publish::Vercel { .. } => Some(Provider::Vercel),
        _ => None,
    };
    if deployment_provider.is_none()
        && (args.site_id.is_some() || settings.production || args.provision)
    {
        return Err(invalid(
            "--site-id, production and --provision require static-site CI delivery",
        ));
    }
    let image_delivery = matches!(publish, Publish::Image { .. } | Publish::LegacyImage);
    if args.platform.is_some() && !image_delivery {
        return Err(invalid("--platform requires image CI delivery"));
    }
    let platform = if image_delivery {
        args.platform
            .or(site.image.as_ref().and_then(|image| image.platform))
    } else {
        None
    };
    let build = args.overrides.options(args.features);
    if !image_delivery {
        build.require_no_image_args()?;
    }
    if site.renderer == zellij_core::site::Renderer::Portable
        && !matches!(
            publish,
            Publish::Image {
                kind: zellij_core::delivery::ImageKind::Next,
                ..
            } | Publish::LegacyImage
        )
    {
        crate::images::require_native_options(&build)?;
    }
    let path = Utf8PathBuf::from(format!(".github/workflows/{}", settings.workflow));
    let options = CiOptions {
        config: if site.input.explicit_config() {
            Some(
                site.config_path
                    .strip_prefix(&repo)
                    .map_err(|_| invalid("configuration must be inside its Git working tree"))?
                    .iter()
                    .collect::<Vec<_>>()
                    .join("/"),
            )
        } else {
            None
        },
        site_root: relative,
        platform,
        features: args.features,
        overrides: build.overrides,
        build_args: build.build_args,
        settings,
        version,
        publish,
    };
    let yaml = pipeline::github(&options)?;
    for relative in [
        Utf8Path::new(".github"),
        Utf8Path::new(".github/workflows"),
        &path,
    ] {
        if std::fs::symlink_metadata(repo.join(relative))
            .is_ok_and(|meta| zellij_core::site::is_link(&meta))
        {
            return Err(files::unsafe_path(
                relative,
                "workflow destinations cannot follow symlinks or junctions",
            ));
        }
    }
    files::preflight(&repo, &[&path], args.force)?;
    // Token validation before the write, remote mutation only after it. Never
    // infer provisioning from a token merely existing in the environment.
    let token = if args.provision {
        {
            let name = deployment_provider
                .expect("validated deployment")
                .token_environment();
            Some(std::env::var(name).ok().filter(|t| !t.is_empty() && t.bytes().all(|b| b.is_ascii_graphic())).ok_or_else(|| invalid(&format!("--provision requires {name} in the environment; no workflow was written")))?)
        }
    } else {
        None
    };
    files::write_files(&repo, &[(&path, yaml)], args.force)?;
    zellij_core::statusln!("✓ {}", repo.join(&path));
    zellij_core::statusln!(
        "  Uses {}; release availability was not checked.",
        options.version
    );
    zellij_core::statusln!(
        "  Review and commit the workflow after that compatible release is published."
    );
    if let Some(token) = token {
        provision(
            &repo,
            deployment_provider
                .expect("validated deployment")
                .token_environment(),
            &token,
        )?;
    } else if let Some(provider) = deployment_provider {
        let name = provider.token_environment();
        zellij_core::statusln!(
            "  Set the repository secret {name}, or re-run with --force --provision to upload it from the environment."
        );
    }
    if deployment_provider == Some(Provider::Cloudflare) {
        zellij_core::statusln!(
            "  Set repository variable CLOUDFLARE_ACCOUNT_ID (the same-named secret is also accepted)."
        );
    }
    if deployment_provider == Some(Provider::Vercel) {
        zellij_core::statusln!(
            "  For a team project, set repository variable VERCEL_TEAM_ID (the same-named secret is also accepted)."
        );
    }
    match options.publish {
        Publish::Vercel { project: None } => zellij_core::statusln!(
            "  Set repository variable VERCEL_PROJECT_ID (the same-named secret is also accepted)."
        ),
        Publish::Cloudflare { project: None } => zellij_core::statusln!(
            "  Set repository variable CLOUDFLARE_PAGES_PROJECT_NAME (the same-named secret is also accepted)."
        ),
        Publish::Netlify { site_id: None } => zellij_core::statusln!(
            "  Set repository variable NETLIFY_SITE_ID (the same-named secret is also accepted)."
        ),
        Publish::Image { tag, .. } if !tag.starts_with("ghcr.io/") => zellij_core::statusln!(
            "  Set repository secrets REGISTRY_USERNAME and REGISTRY_PASSWORD for the selected registry."
        ),
        _ => {}
    }
    Ok(())
}
fn provision(repo: &Utf8Path, name: &str, token: &str) -> Result<()> {
    let operation = (|| -> std::io::Result<bool> {
        let mut child = Command::new("gh")
            .current_dir(repo)
            .args(["secret", "set", name])
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?;
        let written = child
            .stdin
            .take()
            .expect("piped stdin")
            .write_all(token.as_bytes());
        let status = child.wait()?;
        written?;
        Ok(status.success())
    })();
    if !matches!(operation, Ok(true)) {
        return Err(std::io::Error::other(
            format!("workflow written, but gh could not confirm {name} provisioning; check GitHub CLI installation, login and repository secret permissions. The remote result may be uncertain; inspect repository secrets before retrying. No deployment was started."),
        ).into());
    }
    zellij_core::statusln!("  {name} uploaded to repository secrets; no deployment started.");
    Ok(())
}
