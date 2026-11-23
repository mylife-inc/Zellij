//! Maintainer acceptance harness; the installed CLI does not expose these knobs.
use std::{
    path::PathBuf,
    sync::atomic::{AtomicBool, Ordering},
    time::Duration,
};
use zellij_engine::pdf_runner::{self, Event, Options, Runner};

fn main() {
    if let Err(error) = execute() {
        zellij_core::diagnosticln!("{error}");
        std::process::exit(1);
    }
}

fn execute() -> zellij_core::Result<()> {
    let args = std::env::args_os()
        .skip(1)
        .map(PathBuf::from)
        .collect::<Vec<_>>();
    if args.len() != 5 {
        return Err(zellij_core::Error::InvalidArgument(
            "expected NODE SCRIPT BUNDLE INPUT NEW_OUTPUT".into(),
        ));
    }
    zellij_core::reporting::initialize(zellij_core::reporting::Options {
        quiet: std::env::var_os("ZEL_PDF_TEST_QUIET").is_some(),
        no_color: std::env::var_os("ZEL_PDF_TEST_NO_COLOR").is_some(),
        verbose: false,
    });
    let stop = AtomicBool::new(std::env::var_os("ZEL_PDF_TEST_PREABORT").is_some());
    let mut options = Options::default();
    if let Ok(ms) = std::env::var("ZEL_PDF_TEST_TIMEOUT_MS") {
        options.timeout =
            Duration::from_millis(ms.parse().map_err(|_| {
                zellij_core::Error::InvalidArgument("invalid test deadline".into())
            })?);
    }
    if let Ok(ms) = std::env::var("ZEL_PDF_TEST_PASS_MS") {
        options.pass_timeout = Duration::from_millis(ms.parse().map_err(|_| {
            zellij_core::Error::InvalidArgument("invalid test pass deadline".into())
        })?);
    }
    pdf_runner::run(
        Runner {
            node: &args[0],
            script: &args[1],
            bundle: &args[2],
        },
        &args[3],
        &args[4],
        &stop,
        options,
        |event| {
            // Private evidence lets the outer test prove the actual compiler was
            // reaped; events never become user-facing protocol output.
            if let Some(file) = std::env::var_os("ZEL_PDF_TEST_PID") {
                std::fs::write(file, event.pid().to_string())?;
            }
            let phase = match event {
                Event::Ready { .. } => "ready".into(),
                Event::Pass { pass, .. } => pass.to_string(),
                Event::Complete { .. } => "complete".into(),
            };
            if std::env::var("ZEL_PDF_TEST_CANCEL").ok().as_deref() == Some(&phase) {
                stop.store(true, Ordering::Release);
            }
            if std::env::var("ZEL_PDF_TEST_OBSERVER_ERROR").ok().as_deref() == Some(&phase) {
                return Err(zellij_core::Error::InvalidArgument(
                    "test observer failed".into(),
                ));
            }
            if std::env::var("ZEL_PDF_TEST_EXIT_PARENT").ok().as_deref() == Some(&phase) {
                std::process::exit(71);
            }
            Ok(())
        },
    )?;
    // Exactly one harmless result, independent of quiet/color policy. A real
    // CLI caller reports its own final output path after publishing the site.
    println!("verified");
    Ok(())
}
