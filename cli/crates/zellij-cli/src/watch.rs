//! Serial validation watching; no output, browser, server or publication.
use std::{
    io::{self, Write},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::Duration,
};
use zellij_core::{
    Result,
    site::{Renderer, Site},
};
use zellij_docs::watch::Snapshot;
use zellij_engine::{CheckOptions, ContainerEngine, Engine};

pub fn run(dir: Option<&str>, options: CheckOptions) -> Result<()> {
    let site = Site::load(&crate::select_site(dir)?)?;
    // Missing runtime is a startup error, not an endless watch loop. Portable
    // validation never probes external tools. Hosted checks reuse this engine.
    let engine = (site.renderer == Renderer::Hosted)
        .then(|| ContainerEngine::detect(None))
        .transpose()?;
    let stop = Arc::new(AtomicBool::new(false));
    let signal = stop.clone();
    ctrlc::set_handler(move || signal.store(true, Ordering::Release)).map_err(io::Error::other)?;
    let snapshot = || {
        Snapshot::site_for(&site.root, site.configured, site.renderer).map_err(|e| e.to_string())
    };
    let validate = || {
        println!("Checking {}", site.root);
        let result = if let Some(engine) = &engine {
            crate::resolve_content(site.root.as_str()).and_then(|content| {
                let report = engine.check(&content, options)?;
                print!("{}", report.output);
                if report.ok { Ok(0) } else { Ok(3) }
            })
        } else {
            crate::validation::portable(&site.root, options.strict, false)
        };
        match result {
            Ok(0) => println!("Validation passed."),
            Ok(_) => println!("Validation failed; watching for a corrected edit."),
            Err(error) => eprintln!("Validation failed; watching for a corrected edit.\n{error}"),
        }
        let _ = io::stdout().flush();
    };
    let mut attempted = snapshot();
    match &attempted {
        Ok(_) => validate(),
        Err(error) => eprintln!("Validation failed; watching for a corrected edit.\n{error}"),
    }
    println!(
        "Watching {} for validation. Press Ctrl-C to stop.",
        site.root
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
            Err(error) => eprintln!("Validation failed; watching for a corrected edit.\n{error}"),
        }
    }
    // Synchronous checks finish before shutdown: no detached worker or runtime
    // child survives the watcher. No further check starts after cancellation.
    println!("Stopped.");
    Ok(())
}
