//! Read-only site/configuration and runtime preflight. Content validation is
//! deliberately left to `check`; doctor never builds, pulls or starts a service.
use camino::Utf8Path;
use serde_json::{Value, json};
use zellij_core::{
    Result, files,
    site::{self, Renderer, Site, SiteInput},
};

pub fn run(input: Option<&SiteInput>, as_json: bool) -> Result<()> {
    let mut report = json!({
        "schema_version": 1,
        "version": env!("CARGO_PKG_VERSION"),
        "target": crate::TARGET,
        "site": null,
        "runtime": { "status": "not_checked" },
        "image": null,
    });
    let result = inspect(input, &mut report);
    let code = result.as_ref().err().map_or(0, |error| error.exit_code());
    report["ok"] = json!(result.is_ok());
    report["exit_code"] = json!(code);
    report["error"] = result.as_ref().err().map_or(Value::Null, |error| {
        json!({
            "kind": if matches!(error, zellij_core::Error::NoEngine { .. }) {
                "runtime"
            } else { "configuration" },
            "message": error.to_string(),
        })
    });
    if as_json {
        crate::print_json(&report)?;
        if code != 0 {
            // A handled preflight failure is part of the JSON report. Do not
            // let main duplicate it on stderr (matching check --json).
            std::process::exit(code);
        }
    } else {
        human(&report);
    }
    result
}

fn inspect(input: Option<&SiteInput>, report: &mut Value) -> Result<()> {
    let selected = match input {
        Some(input) => Some(input.clone()),
        None => match site::select(Utf8Path::new("."), None) {
            Ok(root) => Some(root.into()),
            // Preserve doctor as an environment diagnostic outside a site.
            // Only legacy hosted markers imply a site without a config;
            // arbitrary Markdown directories require an explicit path.
            Err(_) if zellij_core::Project::discover(Utf8Path::new(".")).is_ok() => {
                Some(SiteInput::from("."))
            }
            Err(_) => None,
        },
    };
    if let Some(input) = selected {
        zellij_core::verbose!("Selected site input: {input}");
        if !input.root().is_dir() {
            return Err(zellij_core::Error::InvalidArgument(format!(
                "{}: expected an existing site/content directory",
                input.root()
            )));
        }
        let site = Site::load(&input)?;
        let portable = site.renderer == Renderer::Portable;
        let content = if portable {
            site.content.clone()
        } else {
            crate::resolve_content(&input)?
        };
        report["site"] = json!({
            "root": site.root.as_str(),
            "config": if input.has_config() { Some(site.config_path.as_str()) } else { None },
            "layout": if site.configured { "versioned" } else { "legacy" },
            "renderer": if portable { "portable" } else { "hosted" },
            "content": content.as_str(),
            "output": files::absolute_path(&site.output)?.as_str(),
            "preview_port": site.port,
        });
        if portable {
            zellij_docs::validate_settings(site.portable)?;
            report["site"]["pdf"] = json!(site.pdf);
            if !site.pdf.enabled {
                report["runtime"] = json!({ "status": "not_required" });
                return Ok(());
            }
            report["runtime"] = json!({ "status": "unavailable" });
            let runtime = zellij_engine::runtime::detect()?;
            report["runtime"] = json!({ "status": "available", "program": runtime.program,
                "version": runtime.version, "endpoint": runtime.endpoint(), "rootless": runtime.rootless()? });
            report["image"] = json!({ "reference": crate::pdf::IMAGE, "status": "not_checked" });
            return Ok(());
        }
    }
    report["runtime"] = json!({ "status": "unavailable" });
    let engine = zellij_engine::ContainerEngine::detect(None)?;
    let runtime = engine.runtime();
    report["runtime"] = json!({
        "status": "available",
        "program": runtime.program,
        "version": runtime.version,
        "endpoint": runtime.endpoint(),
    });
    report["image"] = json!({
        "reference": engine.image(),
        "status": match engine.image_is_local() {
            Some(true) => "local",
            Some(false) => "not_pulled",
            None => "unknown",
        },
    });
    Ok(())
}

fn human(report: &Value) {
    let text = |value: &Value| value.as_str().unwrap_or("unavailable").to_owned();
    zellij_core::reportln!(
        "zel            {}  ({})",
        text(&report["version"]),
        text(&report["target"])
    );
    zellij_core::reportln!();
    zellij_core::reportln!("Site");
    if report["site"].is_null() {
        zellij_core::reportln!("  · no site resolved — pass a site folder or --config <file>");
    } else {
        let site = &report["site"];
        for key in ["root", "config", "layout", "renderer", "content", "output"] {
            let value = if site[key].is_null() {
                "none (inferred)".into()
            } else {
                text(&site[key])
            };
            zellij_core::reportln!("  {key:<12} {value}");
        }
        zellij_core::reportln!("  preview port {}", site["preview_port"]);
    }
    zellij_core::reportln!();
    zellij_core::reportln!("Container runtime");
    let runtime = &report["runtime"];
    match runtime["status"].as_str() {
        Some("not_required") => {
            zellij_core::reportln!("  · not required for portable web-only sites; not probed")
        }
        Some("available") => {
            zellij_core::reportln!(
                "  ✓ {:<10} {} via {}",
                text(&runtime["program"]),
                text(&runtime["version"]),
                text(&runtime["endpoint"])
            );
            let state = match report["image"]["status"].as_str() {
                Some("local") => "local",
                Some("not_pulled") => "not pulled — the first build will fetch it",
                Some("not_checked") => "not inspected — optional PDF toolchain",
                _ => "unknown — image inspection unavailable or timed out",
            };
            zellij_core::reportln!(
                "  · engine     {}  ({state})",
                text(&report["image"]["reference"])
            );
        }
        Some("unavailable") => {
            zellij_core::reportln!("  ✗ unavailable");
            zellij_core::reportln!("  Native commands still work without a container runtime.");
        }
        _ => zellij_core::reportln!("  · not checked — resolve the configuration error first"),
    }
    zellij_core::reportln!();
    zellij_core::reportln!(
        "Content, output writability, credentials and remote services are not checked."
    );
    zellij_core::reportln!("Use `zel check [site]` for content validation.");
}
