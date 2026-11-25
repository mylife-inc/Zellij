//! Maintainer-only qualification profile; installed commands use the separate release profile.
#[path = "../src/package_transfer.rs"]
mod package_transfer;
#[path = "../src/pdf_package.rs"]
#[allow(dead_code)] // The installed CLI also exposes read-only release-cache diagnostics.
mod pdf_package;
#[path = "../src/user_home.rs"]
mod user_home;

use std::{
    path::PathBuf,
    sync::{Arc, atomic::AtomicBool},
    time::Instant,
};
fn main() {
    if let Err(error) = execute() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
fn execute() -> zellij_core::Result<()> {
    let started = Instant::now();
    let trace = std::env::var_os("ZEL_PDF_CACHE_TEST_TRACE").is_some();
    let phase = |label: &str| {
        if trace {
            eprintln!(
                "PDF cache phase: {} ms {label}",
                started.elapsed().as_millis()
            );
        }
    };
    let args = std::env::args_os().skip(1).collect::<Vec<_>>();
    if args.len() != 5 {
        return Err(zellij_core::Error::InvalidArgument(
            "expected BASE_URL MODE NODE INPUT NEW_OUTPUT (MODE is online/offline/inspect)".into(),
        ));
    }
    let mode = args[1].to_str().unwrap_or("");
    if !matches!(mode, "online" | "offline" | "inspect") {
        return Err(zellij_core::Error::InvalidArgument(
            "invalid acceptance mode".into(),
        ));
    }
    let stop = Arc::new(AtomicBool::new(false));
    if let Ok(value) = std::env::var("ZEL_PDF_CACHE_TEST_CANCEL_MS") {
        let ms: u64 = value.parse().map_err(|_| {
            zellij_core::Error::InvalidArgument("invalid test cancellation deadline".into())
        })?;
        let cancel = stop.clone();
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(ms));
            cancel.store(true, std::sync::atomic::Ordering::Release);
        });
    }
    let signal_stop = stop.clone();
    ctrlc::set_handler(move || signal_stop.store(true, std::sync::atomic::Ordering::Release))
        .map_err(|e| zellij_core::Error::InvalidArgument(e.to_string()))?;
    phase("acquisition started");
    let package = pdf_package::obtain(
        args[0].to_str().unwrap_or(""),
        mode != "online",
        &stop,
        false,
    )?;
    phase("cache verified");
    if mode != "inspect" {
        zellij_engine::pdf_runner::run(
            zellij_engine::pdf_runner::Runner {
                node: &PathBuf::from(&args[2]),
                script: &package.script(),
                bundle: &package.resources(),
            },
            &PathBuf::from(&args[3]),
            &PathBuf::from(&args[4]),
            &stop,
            zellij_engine::pdf_runner::Options::default(),
            |event| {
                use zellij_engine::pdf_runner::Event;
                match event {
                    Event::Ready { .. } => phase("compiler ready"),
                    Event::Pass { pass, .. } => phase(&format!("pass {pass} started")),
                    Event::Complete { .. } => phase("compiler complete"),
                }
                Ok(())
            },
        )?;
        phase("artifacts verified");
    }
    println!(
        "{}",
        serde_json::json!({"cache":package.root, "manifestSha256":package.manifest_sha256,
        "sourceArchive":package.source_archive, "experimental":true})
    );
    Ok(())
}
