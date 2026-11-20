//! Serial validation watching; no output, browser, server or publication.
use std::{
    io::{self, Write},
    sync::atomic::Ordering,
    thread,
    time::Duration,
};
use zellij_core::site::SiteInput;
use zellij_core::{
    Result,
    site::{Renderer, Site},
};
use zellij_docs::watch::Snapshot;
use zellij_engine::{CheckOptions, ContainerEngine, Engine};

pub fn run(
    dir: Option<&SiteInput>,
    target: Option<&str>,
    options: CheckOptions,
    offline: bool,
) -> Result<()> {
    let selected = crate::select_site(dir)?;
    let mut site = Site::load(&selected)?;
    let target = target.unwrap_or(if site.renderer == Renderer::Hosted {
        "file"
    } else {
        "portable"
    });
    site.require(if target == "portable" {
        Renderer::Portable
    } else {
        Renderer::Hosted
    })?;
    if offline && target != "file" {
        return Err(zellij_core::Error::InvalidArgument(
            "--offline applies to standalone rich validation".into(),
        ));
    }
    if target != "portable" && !site.configured {
        site.input = crate::resolve_content(&selected)?.into();
    }
    // Missing prerequisites fail once at startup; bad content remains watched.
    let engine = (target == "static")
        .then(|| ContainerEngine::detect(None))
        .transpose()?;
    if target == "file" {
        crate::rich_build::ensure_cached(offline)?;
    }
    let stop = crate::preview::signals()?;
    let snapshot = || {
        Snapshot::site_for(&site.input, site.configured, site.renderer).map_err(|e| e.to_string())
    };
    let validate = || {
        zellij_core::statusln!("Checking {}", site.input.root());
        let result = if let Some(engine) = &engine {
            crate::resolve_content(&site.input).and_then(|content| {
                let report = engine.check(&content, options)?;
                zellij_core::reporting::emit(format_args!("{}", report.output), false);
                if report.ok { Ok(0) } else { Ok(3) }
            })
        } else if target == "file" {
            crate::resolve_content(&site.input).and_then(|content| {
                let Some(report) = crate::rich_build::check(&content, options, true, Some(&stop))?
                else {
                    return Ok(0);
                };
                crate::validation::emit_hosted(&report);
                Ok(if report.ok { 0 } else { 3 })
            })
        } else {
            crate::validation::portable(&site.input, options.strict, false)
        };
        if stop.load(Ordering::Acquire) {
            return;
        }
        match result {
            Ok(0) => zellij_core::statusln!("Validation passed."),
            Ok(_) if zellij_core::reporting::options().quiet => {
                zellij_core::diagnosticln!("Validation failed; watching for a corrected edit.")
            }
            Ok(_) => zellij_core::reportln!("Validation failed; watching for a corrected edit."),
            Err(error) => zellij_core::diagnosticln!(
                "Validation failed; watching for a corrected edit.\n{error}"
            ),
        }
        let _ = io::stdout().flush();
    };
    let mut attempted = snapshot();
    match &attempted {
        Ok(_) => validate(),
        Err(error) => {
            zellij_core::diagnosticln!("Validation failed; watching for a corrected edit.\n{error}")
        }
    }
    zellij_core::statusln!(
        "Watching {} for validation. Press Ctrl-C to stop.",
        site.input.root()
    );
    io::stdout().flush()?;
    let mut observed = attempted.clone();
    while !stop.load(Ordering::Acquire) {
        thread::sleep(Duration::from_millis(300));
        if stop.load(Ordering::Acquire) {
            break;
        }
        let current = snapshot();
        if current != observed {
            observed = current;
            continue;
        }
        if current == attempted {
            continue;
        }
        attempted = current.clone();
        match current {
            Ok(_) => validate(),
            Err(error) => zellij_core::diagnosticln!(
                "Validation failed; watching for a corrected edit.\n{error}"
            ),
        }
    }
    // Synchronous checks finish before shutdown: no detached worker or runtime
    // child survives the watcher. No further check starts after cancellation.
    zellij_core::statusln!("Stopped.");
    Ok(())
}
