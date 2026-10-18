//! Execute a resolved delivery plan without re-entering public CLI commands.
use zellij_core::{Error, Result, delivery::ImagePublication};

pub(crate) fn publish(
    dir: Option<&str>,
    to: Option<&str>,
    tag: Option<&str>,
    dry_run: bool,
) -> Result<()> {
    if to.is_some() {
        return Err(Error::InvalidArgument(
            "--to selects direct provider deployment, which is not implemented yet; saved publication currently supports action: push only".into(),
        ));
    }
    let (site, content) = crate::images::selection(dir)?;
    let plan = ImagePublication::resolve(&site, tag)?;
    println!(
        "  Site     {}\n  Content  {content}\n  Kind     {} ({:?})\n  Push to  {}",
        site.root,
        plan.kind.as_str(),
        site.renderer,
        plan.tag
    );
    if dry_run {
        println!(
            "\nPlan only: validate/build one {} image, then push {}.\nNo content validation, tool/credential checks, build or registry request was performed.",
            plan.kind.as_str(),
            plan.tag
        );
        return Ok(());
    }
    let engine = zellij_engine::ContainerEngine::detect(None)?;
    println!("\nBuilding image…");
    if let Err(error) = crate::images::build_image(&site, &content, &engine, plan.kind, &plan.tag) {
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
