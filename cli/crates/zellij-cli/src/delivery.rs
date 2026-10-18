//! Execute a resolved delivery plan without re-entering public CLI commands.
use camino::Utf8Path;
use zellij_core::{
    Result,
    delivery::{DeliveryPlan, ImagePublication},
    site::{Renderer, Site},
};

pub(crate) fn execute(provider: Option<&str>, options: &crate::args::DeliveryArgs) -> Result<()> {
    let (site, content) = crate::images::selection(options.dir.as_deref())?;
    let environment_site = std::env::var("NETLIFY_SITE_ID").ok();
    let plan = DeliveryPlan::resolve(
        &site,
        provider,
        options.to.as_deref(),
        options.tag.as_deref(),
        options.site_id.as_deref(),
        environment_site.as_deref(),
        options.prod,
    )?;
    println!(
        "  Site     {}\n  Content  {content}\n  Renderer {:?}",
        site.root, site.renderer
    );
    match &plan {
        DeliveryPlan::Push(plan) => {
            println!("  Kind     {}\n  Push to  {}", plan.kind.as_str(), plan.tag)
        }
        DeliveryPlan::Netlify { site, production } => println!(
            "  Provider netlify\n  Site ID  {site}\n  Mode     {}",
            if *production {
                "production"
            } else {
                "draft preview"
            }
        ),
    }
    if options.dry_run {
        println!(
            "\nPlan only: validate/build once, then deliver.\nNo content validation, tool/credential checks, build or remote request was performed."
        );
        return Ok(());
    }
    match plan {
        DeliveryPlan::Push(plan) => push_image(&site, &content, &plan),
        DeliveryPlan::Netlify {
            site: id,
            production,
        } => {
            let client = crate::netlify::Client::from_environment()?;
            deploy(&site, &content, &client, &id, production)
        }
    }
}

pub(crate) fn deploy(
    site: &Site,
    content: &Utf8Path,
    client: &crate::netlify::Client,
    id: &str,
    production: bool,
) -> Result<()> {
    // Authenticate/read the existing site before an expensive hosted build.
    client.check_site(id)?;
    let engine = match site.renderer {
        Renderer::Hosted => Some(zellij_engine::ContainerEngine::detect(None)?),
        Renderer::Portable => None,
    };
    let temporary = tempfile::Builder::new().prefix("zel-deploy-").tempdir()?;
    let export = Utf8Path::from_path(temporary.path())
        .ok_or_else(|| {
            zellij_core::Error::InvalidArgument("temporary directory is not UTF-8".into())
        })?
        .join("site");
    println!("Building static site…");
    let build = || -> Result<String> {
        match engine {
            Some(engine) => engine.build_for_deployment(content, &export),
            None => {
                let report = zellij_docs::build(&site.root, &export, Default::default())?;
                for warning in report.warnings {
                    eprintln!("warning: {warning}");
                }
                Ok(String::new())
            }
        }
    };
    let prefix = build().inspect_err(|_| {
        eprintln!("Deployment stopped during validation/build; no Netlify deployment was created.")
    })?;
    println!("✓ Built static site\nUploading to Netlify…");
    let deployed = client.deploy(id, &export, production).inspect_err(|_| eprintln!("Static build completed, but deployment did not complete. Check the Netlify dashboard before retrying; the remote deployment may still finish. Source and previous local output are unchanged."))?;
    println!(
        "✓ Netlify deployment {} is ready\n  {}",
        deployed.id,
        deployed.url(&prefix)
    );
    Ok(())
}

fn push_image(site: &Site, content: &Utf8Path, plan: &ImagePublication) -> Result<()> {
    let engine = zellij_engine::ContainerEngine::detect(None)?;
    println!("\nBuilding image…");
    if let Err(error) = crate::images::build_image(site, content, &engine, plan.kind, &plan.tag) {
        eprintln!("Publication stopped during validation/build; no push was attempted.");
        return Err(error);
    }
    println!("✓ Built {}\nPushing image…", plan.tag);
    if let Err(error) = engine.image_push(&plan.tag, None) {
        eprintln!(
            "Image build completed, but the push did not complete. The local image remains available as {}. Fix registry access and retry only the push with `zel image push \"{}\"`.",
            plan.tag, plan.tag
        );
        return Err(error);
    }
    println!("✓ Published {}", plan.tag);
    Ok(())
}
