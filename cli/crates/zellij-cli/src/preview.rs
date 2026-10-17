//! Native static preview orchestration; build and serve share their normal paths.
use camino::Utf8Path;
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

pub struct ViewOptions<'a> {
    pub out: Option<&'a str>,
    pub name: Option<&'a str>,
    pub theme: Option<&'a str>,
    pub port: u16,
    pub no_open: bool,
    pub browser: Option<&'a str>,
}

pub fn serve(dir: &str, port: u16, base_path: &str) -> Result<()> {
    run(
        StaticServer::bind(Utf8Path::new(dir), port, base_path)?,
        None,
    )
}

pub fn view(dir: &str, options: ViewOptions<'_>) -> Result<()> {
    // Reserve the exact port before writing output. No later bind race can
    // switch the browser to a different process or leave a rebuilt site unserved.
    let socket = listener(options.port)?;
    let output = super::build_portable(dir, options.out, options.name, options.theme)?;
    let server = StaticServer::from_listener(&output, socket, "/")?;
    run(server, (!options.no_open).then_some(options.browser))
}

fn run(server: StaticServer, open_browser: Option<Option<&str>>) -> Result<()> {
    let stop = Arc::new(AtomicBool::new(false));
    let signal = stop.clone();
    ctrlc::set_handler(move || signal.store(true, Ordering::Release)).map_err(io::Error::other)?;
    println!("Serving at {}", server.url());
    println!("Press Ctrl-C to stop. Files are served as built; no watching or rebuilding.");
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
    server.run(&stop)?;
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
