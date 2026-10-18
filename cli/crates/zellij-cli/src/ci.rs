//! Resolve local site/repository settings, then write one reviewable workflow.
use crate::args::CiInitArgs;
use camino::{Utf8Path, Utf8PathBuf};
use std::{
    io::Write,
    process::{Command, Stdio},
};
use zellij_core::{
    Error, Result,
    delivery::{DeliveryPlan, PublishSettings},
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

pub(crate) fn generate(system: &str, dir: Option<&str>, args: &CiInitArgs) -> Result<()> {
    if system != "github" {
        return Err(invalid("only the github workflow generator is implemented"));
    }
    let (site, _) = crate::images::selection(dir.or(args.site_root.as_deref()))?;
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
    let environment_site = std::env::var("NETLIFY_SITE_ID")
        .ok()
        .filter(|s| !s.trim().is_empty());
    let publish = match (args.deploy.as_deref(), args.target.as_deref()) {
        (Some(provider), _) if provider != "netlify" => {
            return Err(invalid(
                "only Netlify static-site deployment is implemented",
            ));
        }
        (None, Some("static")) => Publish::Nothing,
        (None, Some("image")) => Publish::LegacyImage,
        (Some("netlify"), _) | (None, None)
            if args.deploy.is_some()
                || matches!(site.publish, Some(PublishSettings::Deploy { .. })) =>
        {
            let saved = match &site.publish {
                Some(PublishSettings::Deploy { site, .. }) => site.as_deref(),
                _ => None,
            };
            let site_id = args
                .site_id
                .as_deref()
                .or(environment_site.as_deref())
                .or(saved);
            // A repository variable/secret can provide the ID at execution time.
            // The shared resolver still validates the provider and supplied ID.
            DeliveryPlan::resolve(
                &site,
                Some("netlify"),
                None,
                None,
                Some(site_id.unwrap_or("configured-in-ci")),
                None,
                settings.production,
            )?;
            Publish::Netlify {
                site_id: site_id.map(str::to_owned),
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
    if !matches!(publish, Publish::Netlify { .. })
        && (args.site_id.is_some() || settings.production || args.provision)
    {
        return Err(invalid(
            "--site-id, production and --provision require Netlify CI delivery",
        ));
    }
    let path = Utf8PathBuf::from(format!(".github/workflows/{}", settings.workflow));
    let options = CiOptions {
        site_root: relative,
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
        Some(std::env::var("NETLIFY_AUTH_TOKEN").ok().filter(|t| !t.is_empty() && t.bytes().all(|b| b.is_ascii_graphic())).ok_or_else(|| invalid("--provision requires NETLIFY_AUTH_TOKEN in the environment; no workflow was written"))?)
    } else {
        None
    };
    files::write_files(&repo, &[(&path, yaml)], args.force)?;
    println!("✓ {}", repo.join(&path));
    println!(
        "  Uses {}; release availability was not checked.",
        options.version
    );
    println!("  Review and commit the workflow after that compatible release is published.");
    if let Some(token) = token {
        provision(&repo, &token)?;
    } else if matches!(options.publish, Publish::Netlify { .. }) {
        println!(
            "  Set the repository secret NETLIFY_AUTH_TOKEN, or re-run with --force --provision to upload it from the environment."
        );
    }
    match options.publish {
        Publish::Netlify { site_id: None } => println!(
            "  Set repository variable NETLIFY_SITE_ID (the same-named secret is also accepted)."
        ),
        Publish::Image { tag, .. } if !tag.starts_with("ghcr.io/") => println!(
            "  Set repository secrets REGISTRY_USERNAME and REGISTRY_PASSWORD for the selected registry."
        ),
        _ => {}
    }
    Ok(())
}
fn provision(repo: &Utf8Path, token: &str) -> Result<()> {
    let operation = (|| -> std::io::Result<bool> {
        let mut child = Command::new("gh")
            .current_dir(repo)
            .args(["secret", "set", "NETLIFY_AUTH_TOKEN"])
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
            "workflow written, but gh could not confirm NETLIFY_AUTH_TOKEN provisioning; check GitHub CLI installation, login and repository secret permissions. The remote result may be uncertain; inspect repository secrets before retrying. No deployment was started.",
        ).into());
    }
    println!("  NETLIFY_AUTH_TOKEN uploaded to repository secrets; no deployment started.");
    Ok(())
}
