//! Execute a resolved delivery plan without re-entering public CLI commands.
use camino::Utf8Path;
use zellij_core::{
    Result,
    delivery::{DeliveryPlan, ImagePublication},
    features::Features,
    site::{Renderer, Site},
};

pub(crate) fn execute(provider: Option<&str>, options: &crate::args::DeliveryArgs) -> Result<()> {
    let (site, content) = crate::images::selection(options.dir.as_ref())?;
    let environment_site = std::env::var("NETLIFY_SITE_ID").ok();
    let plan = DeliveryPlan::resolve(
        &site,
        provider,
        options.to.as_deref(),
        options.tag.as_deref(),
        options.site_id.as_deref(),
        environment_site.as_deref(),
        options.prod,
    )?
    .with_platform(options.platform)?;
    if options.dry_run || !zellij_core::reporting::options().quiet {
        describe(&plan, &site, &content);
        describe_features(options.features);
    }
    if options.dry_run {
        zellij_core::reportln!(
            "\nPlan only: validate/build once, then deliver.\nNo content validation, tool/credential checks, build or remote request was performed."
        );
        return Ok(());
    }
    execute_plan(&site, &content, &plan, options.features)
}

pub(crate) fn describe(plan: &DeliveryPlan, site: &Site, content: &Utf8Path) {
    zellij_core::reportln!(
        "  Site     {}\n  Content  {content}\n  Renderer {:?}",
        site.root,
        site.renderer
    );
    describe_destination(plan);
}

pub(crate) fn describe_destination(plan: &DeliveryPlan) {
    match plan {
        DeliveryPlan::Push(plan) => {
            zellij_core::reportln!("  Kind     {}\n  Push to  {}", plan.kind.as_str(), plan.tag);
            if let Some(platform) = plan.platform {
                zellij_core::reportln!("  Platform {platform}");
            }
        }
        DeliveryPlan::Netlify { site, production } => zellij_core::reportln!(
            "  Provider netlify\n  Site ID  {site}\n  Mode     {}",
            if *production {
                "production"
            } else {
                "draft preview"
            }
        ),
    }
}

pub(crate) fn describe_features(features: Option<Features>) {
    if let Some(features) = features {
        zellij_core::reportln!("  Features {} (this invocation only)", features.as_str());
    }
}

pub(crate) fn execute_plan(
    site: &Site,
    content: &Utf8Path,
    plan: &DeliveryPlan,
    features: Option<Features>,
) -> Result<()> {
    match plan {
        DeliveryPlan::Push(plan) => push_image(site, content, plan, features),
        DeliveryPlan::Netlify {
            site: id,
            production,
        } => {
            let client = crate::netlify::Client::from_environment()?;
            deploy(site, content, &client, id, *production, features)
        }
    }
}

pub(crate) fn deploy(
    site: &Site,
    content: &Utf8Path,
    client: &crate::netlify::Client,
    id: &str,
    production: bool,
    features: Option<Features>,
) -> Result<()> {
    // Authenticate/read the existing site before an expensive hosted build.
    client.check_site(id)?;
    let engine = match site.renderer {
        Renderer::Hosted => Some(zellij_engine::ContainerEngine::detect(None)?),
        Renderer::Portable => None,
    };
    let mut temporary_builder = tempfile::Builder::new();
    temporary_builder.prefix(".zel-deploy-");
    let temporary = if engine.is_some() {
        // Keep the bind-mounted export beside the selected content: runtimes
        // such as Colima may share the user's project but not the OS temp dir.
        // A sibling also stays outside legacy raw-content roots.
        temporary_builder.tempdir_in(content.parent().ok_or_else(|| {
            zellij_core::Error::InvalidArgument(
                "hosted deployment requires a content folder below the filesystem root".into(),
            )
        })?)?
    } else {
        temporary_builder.tempdir()?
    };
    let export = Utf8Path::from_path(temporary.path())
        .ok_or_else(|| {
            zellij_core::Error::InvalidArgument("temporary directory is not UTF-8".into())
        })?
        .join("site");
    zellij_core::statusln!("Building static site…");
    let build = || -> Result<String> {
        match engine {
            Some(engine) => engine.build_for_deployment_with_features(content, &export, features),
            None => {
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
                Ok(String::new())
            }
        }
    };
    let prefix = build().inspect_err(|_| {
        zellij_core::diagnosticln!(
            "Deployment stopped during validation/build; no Netlify deployment was created."
        )
    })?;
    zellij_core::statusln!("✓ Built static site\nUploading to Netlify…");
    let deployed = client.deploy(id, &export, production).inspect_err(|_| zellij_core::diagnosticln!("Static build completed, but deployment did not complete. Check the Netlify dashboard before retrying; the remote deployment may still finish. Source and previous local output are unchanged."))?;
    zellij_core::statusln!(
        "✓ Netlify deployment {} is ready\n  {}",
        deployed.id,
        deployed.url(&prefix)
    );
    Ok(())
}

fn push_image(
    site: &Site,
    content: &Utf8Path,
    plan: &ImagePublication,
    features: Option<Features>,
) -> Result<()> {
    let engine = zellij_engine::ContainerEngine::detect(None)?;
    zellij_core::statusln!("\nBuilding image…");
    if let Err(error) = crate::images::build_image(
        site,
        content,
        &engine,
        plan.kind,
        &plan.tag,
        plan.platform,
        &zellij_engine::BuildOptions::with_features(features),
    ) {
        zellij_core::diagnosticln!(
            "Publication stopped during validation/build; no push was attempted."
        );
        return Err(error);
    }
    zellij_core::statusln!("✓ Built {}\nPushing image…", plan.tag);
    if let Err(error) = engine.image_push(&plan.tag, None) {
        zellij_core::diagnosticln!(
            "Image build completed, but the push did not complete. The local image remains available as {}. Fix registry access and retry only the push with `zel image push \"{}\"`.",
            plan.tag,
            plan.tag
        );
        return Err(error);
    }
    zellij_core::statusln!("✓ Published {}", plan.tag);
    Ok(())
}
