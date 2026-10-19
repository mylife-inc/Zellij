//! Maintenance stays independent of site discovery and authored content.
pub(crate) mod upgrade;

use sha2::{Digest, Sha256};
use zellij_core::{Error, Result};

pub(crate) fn version(json: bool) -> Result<()> {
    let package: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../engine/packages/zellij/package.json"
    ))
    .expect("embedded engine package manifest");
    let schema = format!("{:x}", Sha256::digest(zellij_core::schema::SCHEMA_JSON));
    let image = zellij_engine::configured_image(None);
    let report = serde_json::json!({
        "cli": {"version": env!("CARGO_PKG_VERSION"), "target": crate::TARGET},
        "configuration_version": 1,
        "embedded_engine_version": package["version"],
        "content_schema_sha256": schema,
        "engine_image": image,
    });
    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&report).expect("version JSON")
        );
    } else {
        println!(
            "zel                 {} ({})",
            env!("CARGO_PKG_VERSION"),
            crate::TARGET
        );
        println!("zel.yaml version    1");
        println!(
            "Embedded engine     {}",
            package["version"].as_str().expect("engine version")
        );
        println!("Content schema      sha256:{schema}");
        println!("Selected image      {image}");
        println!("Image contents are not inspected; use `zel doctor` for local availability.");
    }
    Ok(())
}

pub(crate) fn pull(image: Option<&str>) -> Result<()> {
    let image = zellij_engine::configured_image(image);
    // A digest is useful for reproducible engine selection, unlike image tags
    // being built/pushed, which cannot accept digest destinations.
    if let Some((name, digest)) = image.split_once('@') {
        zellij_core::delivery::validate_image_reference(name)?;
        if !digest
            .strip_prefix("sha256:")
            .is_some_and(|hash| hash.len() == 64 && hash.bytes().all(|c| c.is_ascii_hexdigit()))
        {
            return Err(Error::InvalidArgument(
                "image digest must be sha256 followed by 64 hexadecimal digits".into(),
            ));
        }
    } else {
        zellij_core::delivery::validate_image_reference(&image)?;
    }
    let engine = zellij_engine::ContainerEngine::detect(Some(&image))?;
    println!("Pulling {image} with {}", engine.runtime().program);
    engine.pull()?;
    println!("Engine image ready: {image}");
    Ok(())
}
