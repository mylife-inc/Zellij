use std::{
    io::Read,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::atomic::Ordering,
    thread,
    time::{Duration, Instant},
};
use zellij_core::{
    Error, Result,
    features::Features,
    site::{Renderer, SiteInput},
};
use zellij_engine::{ContainerEngine, container::DevReady};

pub fn run(
    dir: Option<&SiteInput>,
    port: Option<u16>,
    host: Option<IpAddr>,
    open: bool,
    browser: Option<&str>,
    features: Option<Features>,
) -> Result<()> {
    let dir = crate::select_site(dir)?;
    let site = zellij_core::site::Site::load(&dir)?;
    let port = port.unwrap_or(site.port);
    if site.renderer == Renderer::Portable {
        return crate::preview::view(
            Some(&dir),
            crate::preview::ViewOptions {
                host: host.unwrap_or(Ipv4Addr::LOCALHOST.into()),
                out: None,
                name: None,
                theme: None,
                features,
                port: Some(port),
                no_open: !open && browser.is_none(),
                browser,
                watch: true,
            },
        );
    }
    if port == 0 {
        return Err(Error::InvalidArgument("hosted dev requires --port 1–65535; use view --watch --port 0 for an automatically selected static preview port".into()));
    }
    let content = crate::resolve_content(&dir)?;
    let engine = ContainerEngine::detect(None)?;
    let host = host.unwrap_or(Ipv4Addr::UNSPECIFIED.into());
    let address = zellij_core::network::browser_address(host, port);
    let origin =
        reqwest::Url::parse(&format!("http://{address}/")).map_err(std::io::Error::other)?;
    let client = reqwest::blocking::Client::builder()
        .no_proxy()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(Duration::from_millis(500))
        .build()
        .map_err(std::io::Error::other)?;
    let stop = crate::preview::signals()?;
    zellij_core::statusln!("  Engine   {}", engine.image());
    zellij_core::statusln!("  Content  {content}");
    zellij_core::statusln!(
        "  Binding  {}. Waiting for the development server; Ctrl-C to stop.",
        SocketAddr::new(host, port)
    );
    let mut session = engine.start_dev(&content, port, host, features)?;
    let mut next_probe = Instant::now();
    let deadline = Instant::now() + Duration::from_secs(300);
    let mut announced = false;
    loop {
        if stop.load(Ordering::Acquire) {
            session.stop();
            zellij_core::statusln!("Stopped.");
            return Ok(());
        }
        match session.try_wait() {
            Ok(Some(())) => return Ok(()),
            Err(error @ Error::MountEmpty { .. }) => {
                crate::report_empty_mount(&error);
                // process::exit bypasses destructors; release the session's
                // temporary identity before reporting this specialized error.
                drop(session);
                std::process::exit(error.exit_code());
            }
            Err(error) => return Err(error),
            Ok(None) => {}
        }
        if !announced && Instant::now() >= next_probe {
            next_probe = Instant::now() + Duration::from_secs(1);
            if let Some(ready) = session.ready()
                && let Some(url) = ready_url(&client, &origin, &ready)
                && !stop.load(Ordering::Acquire)
            {
                // A failed container start must not open an unrelated process.
                if session.try_wait()?.is_some() {
                    return Ok(());
                }
                announced = true;
                zellij_core::statusln!("Serving at {url}");
                zellij_core::statusln!(
                    "Watching content. Successful preparation reloads the browser. Ctrl-C to stop."
                );
                if open || browser.is_some() {
                    crate::preview::open(&url, browser)?;
                }
            } else if Instant::now() >= deadline {
                announced = true;
                zellij_core::diagnosticln!(
                    "warning: could not confirm the development URL within 5 minutes. Check the runtime output and open {origin} with your site's basePath manually. Automatic opening needs a current engine image and a published port reachable from this machine."
                );
            }
        }
        thread::sleep(Duration::from_millis(50));
    }
}

fn ready_url(
    client: &reqwest::blocking::Client,
    origin: &reqwest::Url,
    expected: &DevReady,
) -> Option<String> {
    let prefix = &expected.base_path;
    if (!prefix.is_empty() && !prefix.starts_with('/'))
        || prefix.contains(['?', '#', '\\'])
        || prefix.split('/').any(|part| part == "." || part == "..")
    {
        return None;
    }
    let mut url = origin.clone();
    url.set_path(&format!(
        "{}/_zellij/dev-revision.json",
        prefix.trim_end_matches('/')
    ));
    let response = client.get(url.clone()).send().ok()?;
    if !response.status().is_success() {
        return None;
    }
    let mut bytes = Vec::new();
    response.take(4096).read_to_end(&mut bytes).ok()?;
    let served: DevReady = serde_json::from_slice(&bytes).ok()?;
    if served.version != 1
        || served.session != expected.session
        || served.base_path != expected.base_path
        || served.revision.is_empty()
    {
        return None;
    }
    url.set_path(&format!("{}/", prefix.trim_end_matches('/')));
    Some(url.to_string())
}
