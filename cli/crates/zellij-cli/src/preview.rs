//! Native static preview orchestration; build and serve share their normal paths.
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
use zellij_docs::server::{StaticServer, listener};
use zellij_docs::watch::{LiveReload, Snapshot, WatchOptions, Watcher};

pub struct ViewOptions<'a> {
    pub out: Option<&'a str>,
    pub name: Option<&'a str>,
    pub theme: Option<&'a str>,
    pub port: Option<u16>,
    pub no_open: bool,
    pub browser: Option<&'a str>,
    pub watch: bool,
}

pub fn serve(dir: Option<&str>, port: Option<u16>, base_path: &str) -> Result<()> {
    let selected = crate::select_site(dir)?;
    let (output, default_port) = if zellij_core::site::present(&selected) {
        let site = zellij_core::site::Site::load(&selected)?;
        (site.output, site.port)
    } else {
        (selected, 3000)
    };
    run(
        StaticServer::bind(&output, port.unwrap_or(default_port), base_path)?,
        None,
        signals()?,
        None,
    )
}

pub fn view(dir: Option<&str>, options: ViewOptions<'_>) -> Result<()> {
    let selected = crate::select_site(dir)?;
    let site = zellij_core::site::Site::load(&selected)?;
    if site.renderer != zellij_core::site::Renderer::Portable {
        return hosted::view(site, options);
    }
    // Reserve before writing output, so a bind failure cannot replace a site.
    let socket = listener(options.port.unwrap_or(site.port))?;
    let source = site.root;
    let initial = options
        .watch
        .then(|| Snapshot::site(&source, site.configured))
        .transpose()?;
    let stop = signals()?;
    let output = options
        .out
        .map(camino::Utf8PathBuf::from)
        .unwrap_or(site.output);
    let prepared = zellij_docs::prepare(
        &source,
        &output,
        zellij_docs::BuildOptions {
            name: options.name,
            theme: options.theme,
        },
    );
    if stop.load(Ordering::Acquire) {
        println!("Stopped.");
        return Ok(());
    }
    let report = prepared?.publish()?;
    for warning in report.warnings {
        eprintln!("warning: {warning}");
    }
    println!("✓ {output}  ({} HTML pages)", report.pages);
    let mut server = StaticServer::from_listener(&output, socket, "/")?;
    let watcher = if let Some(initial) = initial {
        let live = Arc::new(LiveReload::default());
        server = server.with_reload(live.clone());
        Some(Watcher::spawn(
            WatchOptions {
                source,
                output: zellij_core::files::absolute_path(&output)?,
                name: options.name.map(str::to_owned),
                theme: options.theme.map(str::to_owned),
            },
            initial,
            live,
            stop.clone(),
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

fn signals() -> Result<Arc<AtomicBool>> {
    let stop = Arc::new(AtomicBool::new(false));
    let signal = stop.clone();
    ctrlc::set_handler(move || signal.store(true, Ordering::Release)).map_err(io::Error::other)?;
    Ok(stop)
}

fn run(
    server: StaticServer,
    open_browser: Option<Option<&str>>,
    stop: Arc<AtomicBool>,
    watcher: Option<Watcher>,
) -> Result<()> {
    println!("Serving at {}", server.url());
    if watcher.is_some() {
        println!(
            "Watching content and configuration. Successful rebuilds reload the browser. Press Ctrl-C to stop."
        );
    } else {
        println!("Press Ctrl-C to stop. Files are served as built; no watching or rebuilding.");
    }
    io::stdout().flush()?;
    if let Some(executable) = open_browser {
        let url = server.url().to_owned();
        let executable = executable.map(str::to_owned);
        // Launchers can wait for a GUI to close. Keep HTTP and Ctrl-C responsive
        // while they run, and retain the usable URL if no desktop is available.
        thread::Builder::new().name("zel-browser".into()).spawn(move || {
            if let Err(error) = launch(&url, executable.as_deref()) {
                eprintln!("warning: could not open the browser: {error}\n  Open {url} manually; the server is still running.");
            }
        })?;
    }
    let result = server.run(&stop);
    // Join the worker before reporting completion; an in-progress staged build
    // is discarded after cancellation instead of publishing after shutdown.
    drop(watcher);
    result?;
    println!("Stopped.");
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
