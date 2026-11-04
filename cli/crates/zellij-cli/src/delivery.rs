//! Execute a resolved delivery plan without re-entering public CLI commands.
use camino::Utf8Path;
use zellij_core::{
    Result,
    delivery::{DeliveryPlan, ImageKind, ImagePublication},
    features::Features,
    site::{Renderer, Site},
};
use zellij_engine::BuildOptions;

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
    let build = options.overrides.options(options.features);
    validate_options(&site, &plan, &build)?;
    if options.dry_run || !zellij_core::reporting::options().quiet {
        describe(&plan, &site, &content);
        describe_options(&build);
    }
    if options.dry_run {
        zellij_core::reportln!(
            "\nPlan only: prepare/build once, then deliver.\nNo content validation, tool/credential checks, build or remote request was performed."
        );
        return Ok(());
    }
    execute_plan(&site, &content, &plan, &build)
}

fn validate_options(site: &Site, plan: &DeliveryPlan, options: &BuildOptions) -> Result<()> {
    if site.renderer == Renderer::Portable
        && !matches!(
            plan,
            DeliveryPlan::Push(ImagePublication {
                kind: ImageKind::Next,
                ..
            })
        )
    {
        crate::images::require_native_options(options)?;
    }
    Ok(())
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

pub(crate) fn describe_options(options: &BuildOptions) {
    describe_features(options.features);
    if let Some(prefix) = &options.overrides.base_path {
        let prefix = prefix.to_string();
        zellij_core::reportln!(
            "  Base path {} (this invocation only)",
            if prefix.is_empty() { "/" } else { &prefix }
        );
    }
    if let Some(origin) = &options.overrides.site_url {
        zellij_core::reportln!("  Site URL {origin} (this invocation only)");
    }
    if options.overrides.no_check {
        zellij_core::reportln!(
            "  Preflight skipped; preparation and rendering still validate content"
        );
    }
}

pub(crate) fn execute_plan(
    site: &Site,
    content: &Utf8Path,
    plan: &DeliveryPlan,
    options: &BuildOptions,
) -> Result<()> {
    validate_options(site, plan, options)?;
    match plan {
        DeliveryPlan::Push(plan) => push_image(site, content, plan, options),
        DeliveryPlan::Netlify {
            site: id,
            production,
        } => {
            let client = crate::netlify::Client::from_environment()?;
            deploy(site, content, &client, id, *production, options)
        }
    }
}

pub(crate) fn deploy(
    site: &Site,
    content: &Utf8Path,
    client: &crate::netlify::Client,
    id: &str,
    production: bool,
    options: &BuildOptions,
) -> Result<()> {
    if site.renderer == Renderer::Portable {
        crate::images::require_native_options(options)?;
    }
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
            Some(engine) => engine.build_for_deployment_with_options(content, &export, options),
            None => {
                let report = zellij_docs::build(
                    &site.input,
                    &export,
                    zellij_docs::BuildOptions {
                        features: options.features,
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
    options: &BuildOptions,
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
        options,
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
