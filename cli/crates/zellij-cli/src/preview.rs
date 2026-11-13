//! Native static preview orchestration; build and serve share their normal paths.
use zellij_core::site::SiteInput;
mod hosted;
use std::{
    io::{self, Write},
    process::{Command, Stdio},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
};
use zellij_core::Result;
use zellij_docs::server::{StaticServer, listener_on};
use zellij_docs::watch::{LiveReload, Snapshot, Watcher};

pub struct ViewOptions<'a> {
    pub host: std::net::IpAddr,
    pub out: Option<&'a str>,
    pub name: Option<&'a str>,
    pub theme: Option<&'a str>,
    pub features: Option<zellij_core::features::Features>,
    pub port: Option<u16>,
    pub no_open: bool,
    pub browser: Option<&'a str>,
    pub watch: bool,
    pub pdf: Option<bool>,
}

pub fn serve(dir: Option<&SiteInput>, port: Option<u16>, base_path: &str) -> Result<()> {
    let selected = crate::select_site(dir)?;
    let (output, default_port) = if selected.has_config() {
        let site = zellij_core::site::Site::load(&selected)?;
        (site.output, site.port)
    } else {
        (selected.root().to_owned(), 3000)
    };
    run(
        StaticServer::bind(&output, port.unwrap_or(default_port), base_path)?,
        None,
        signals()?,
        None,
    )
}

pub fn view(dir: Option<&SiteInput>, options: ViewOptions<'_>) -> Result<()> {
    let selected = crate::select_site(dir)?;
    let site = zellij_core::site::Site::load(&selected)?;
    if site.renderer != zellij_core::site::Renderer::Portable {
        return hosted::view(site, options);
    }
    // Reserve before writing output, so a bind failure cannot replace a site.
    let socket = listener_on(options.host, options.port.unwrap_or(site.port))?;
    let source = site.input;
    let configured = site.configured;
    let initial = Snapshot::site(&source, configured)?;
    let stop = signals()?;
    let output = options
        .out
        .map(camino::Utf8PathBuf::from)
        .unwrap_or(site.output);
    let prepared = crate::pdf::prepare_site(
        &source,
        &output,
        zellij_docs::BuildOptions {
            name: options.name,
            theme: options.theme,
            features: options.features,
        },
        options.pdf,
        &stop,
    );
    if stop.load(Ordering::Acquire) {
        zellij_core::statusln!("Stopped.");
        return Ok(());
    }
    let prepared = prepared?;
    unchanged(&source, configured, &initial)?;
    let report = prepared.publish()?;
    for warning in report.warnings {
        zellij_core::diagnosticln!("warning: {warning}");
    }
    zellij_core::statusln!("✓ {output}  ({} HTML pages)", report.pages);
    let mut server = StaticServer::from_listener(&output, socket, "/")?;
    let watcher = if options.watch {
        let live = Arc::new(LiveReload::default());
        server = server.with_reload(live.clone());
        let watched = source.clone();
        let publish = live.clone();
        let signal = stop.clone();
        let output = zellij_core::files::absolute_path(&output)?;
        let name = options.name.map(str::to_owned);
        let theme = options.theme.map(str::to_owned);
        let features = options.features;
        let pdf = options.pdf;
        Some(Watcher::spawn_with(
            initial,
            live,
            stop.clone(),
            move || Snapshot::site(&watched, configured),
            move |expected| {
                unchanged(&source, configured, expected)?;
                let prepared = crate::pdf::prepare_site(
                    &source,
                    &output,
                    zellij_docs::BuildOptions {
                        name: name.as_deref(),
                        theme: theme.as_deref(),
                        features,
                    },
                    pdf,
                    &signal,
                )?;
                if signal.load(Ordering::Acquire) {
                    return Ok(None);
                }
                unchanged(&source, configured, expected)?;
                let report = publish.publish_with(|| prepared.publish())?;
                for warning in report.warnings {
                    zellij_core::diagnosticln!("warning: {warning}");
                }
                Ok(Some(format!(
                    "Rebuilt {} HTML pages; reloading preview.",
                    report.pages
                )))
            },
        )?)
    } else {
        None
    };
    run(
        server,
        (!options.no_open).then_some(options.browser),
        stop,
        watcher,
    )
}

fn unchanged(source: &SiteInput, configured: bool, expected: &Snapshot) -> Result<()> {
    if Snapshot::site(source, configured)? != *expected {
        return Err(zellij_core::Error::InvalidArgument("content changed during the build; retry after saving (watch mode retries automatically)".into()));
    }
    Ok(())
}

pub(crate) fn signals() -> Result<Arc<AtomicBool>> {
    use std::sync::{Mutex, OnceLock, Weak};
    static OWNERS: Mutex<Vec<Weak<AtomicBool>>> = Mutex::new(Vec::new());
    static INSTALLED: OnceLock<std::result::Result<(), String>> = OnceLock::new();
    // The ctrlc callback runs on its own thread. Keep one handler for the process
    // and weak owners for active operations, including sequential delivery builds.
    let installed = INSTALLED.get_or_init(|| {
        ctrlc::set_handler(|| {
            let mut owners = OWNERS.lock().unwrap_or_else(|e| e.into_inner());
            let mut active = false;
            owners.retain(|owner| {
                if let Some(stop) = owner.upgrade() {
                    stop.store(true, Ordering::Release);
                    active = true;
                    true
                } else {
                    false
                }
            });
            drop(owners);
            // After PDF generation, an upload or image command must remain interruptible.
            if !active {
                std::process::exit(130);
            }
        })
        .map_err(|e| e.to_string())
    });
    if let Err(error) = installed {
        return Err(io::Error::other(error.clone()).into());
    }
    let stop = Arc::new(AtomicBool::new(false));
    let mut owners = OWNERS.lock().unwrap_or_else(|e| e.into_inner());
    owners.retain(|owner| owner.strong_count() != 0);
    owners.push(Arc::downgrade(&stop));
    Ok(stop)
}

fn run(
    server: StaticServer,
    open_browser: Option<Option<&str>>,
    stop: Arc<AtomicBool>,
    watcher: Option<Watcher>,
) -> Result<()> {
    zellij_core::statusln!("Serving at {}", server.url());
    if watcher.is_some() {
        zellij_core::statusln!(
            "Watching content and configuration. Successful rebuilds reload the browser. Press Ctrl-C to stop."
        );
    } else {
        zellij_core::statusln!(
            "Press Ctrl-C to stop. Files are served as built; no watching or rebuilding."
        );
    }
    io::stdout().flush()?;
    if let Some(executable) = open_browser {
        open(server.url(), executable)?;
    }
    let result = server.run(&stop);
    // Join the worker before reporting completion; an in-progress staged build
    // is discarded after cancellation instead of publishing after shutdown.
    drop(watcher);
    result?;
    zellij_core::statusln!("Stopped.");
    Ok(())
}

pub(crate) fn open(url: &str, executable: Option<&str>) -> Result<()> {
    let url = url.to_owned();
    let executable = executable.map(str::to_owned);
    // GUI launchers can wait for a browser to close. Do not block HTTP or Ctrl-C.
    thread::Builder::new().name("zel-browser".into()).spawn(move || {
        if let Err(error) = launch(&url, executable.as_deref()) {
            zellij_core::diagnosticln!("warning: could not open the browser: {error}\n  Open {url} manually; the server is still running.");
        }
    })?;
    Ok(())
}

fn launch(url: &str, executable: Option<&str>) -> io::Result<()> {
    if let Some(executable) = executable {
        let status = Command::new(executable)
            .arg(url)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()?;
        if !status.success() {
            return Err(io::Error::other(format!(
                "browser launcher exited with {status}"
            )));
        }
        Ok(())
    } else {
        open::that(url)
    }
}
