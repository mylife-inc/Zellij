use super::{ViewOptions, run, signals};
use camino::{Utf8Path, Utf8PathBuf};
use std::sync::{Arc, atomic::Ordering};
use zellij_core::site::SiteInput;
use zellij_core::{
    Error, Result, files,
    site::{Renderer, Site},
};
use zellij_docs::{
    server::{StaticServer, listener},
    watch::{LiveReload, Snapshot, Watcher},
};
use zellij_engine::{ContainerEngine, container::PreparedPreview};

fn snapshot(source: &SiteInput, configured: bool) -> Result<Snapshot> {
    let current = Site::load(source)?;
    if current.configured != configured {
        return Err(Error::InvalidArgument(
            "site layout changed; restore zel.yaml or restart preview".into(),
        ));
    }
    if current.renderer != Renderer::Hosted {
        return Err(Error::InvalidArgument(
            "renderer changed; restore hosted content or restart preview".into(),
        ));
    }
    Snapshot::site_for(source, configured, Renderer::Hosted)
}

fn prepare(
    engine: &ContainerEngine,
    source: &SiteInput,
    output: &Utf8Path,
    configured: bool,
    expected: &Snapshot,
    prefix: Option<&str>,
) -> Result<PreparedPreview> {
    let content = crate::resolve_content(source)?;
    let build = engine.prepare_preview(&content, output)?;
    if snapshot(source, configured)? != *expected {
        return Err(Error::InvalidArgument("content changed during the build; retry after saving (watch mode retries automatically)".into()));
    }
    if prefix.is_some_and(|previous| previous != build.base_path) {
        return Err(Error::InvalidArgument(
            "basePath changed; restore it or restart preview to use the new URL".into(),
        ));
    }
    StaticServer::validate(
        build.directory(),
        if build.base_path.is_empty() {
            "/"
        } else {
            &build.base_path
        },
    )?;
    Ok(build)
}

pub(super) fn view(site: Site, options: ViewOptions<'_>) -> Result<()> {
    if options.name.is_some() || options.theme.is_some() {
        return Err(Error::InvalidArgument("--name and --theme apply only to portable previews; configure hosted sites in content/site.yaml".into()));
    }
    // A missing runtime fails before opening a listener or staging an export.
    let engine = ContainerEngine::detect(None)?;
    let socket = listener(options.port.unwrap_or(site.port))?;
    let source = site.input;
    let configured = site.configured;
    let output = files::absolute_path(&options.out.map(Utf8PathBuf::from).unwrap_or(site.output))?;
    let initial = snapshot(&source, configured)?;
    let stop = signals()?;
    println!("Building hosted preview. Ctrl-C waits for any active build to finish cleanup.");
    let prepared = prepare(&engine, &source, &output, configured, &initial, None);
    if stop.load(Ordering::Acquire) {
        println!("Stopped.");
        return Ok(());
    }
    let prepared = prepared?;
    let prefix = prepared.base_path.clone();
    prepared.publish()?;
    println!("✓ {output}");
    let mut server = StaticServer::from_listener(
        &output,
        socket,
        if prefix.is_empty() { "/" } else { &prefix },
    )?;
    let watcher = if options.watch {
        let live = Arc::new(LiveReload::default());
        server = server.with_reload(live.clone());
        let watched = source.clone();
        let publish = live.clone();
        let signal = stop.clone();
        Some(Watcher::spawn_with(
            initial,
            live,
            stop.clone(),
            move || snapshot(&watched, configured),
            move |expected| {
                let prepared = prepare(
                    &engine,
                    &source,
                    &output,
                    configured,
                    expected,
                    Some(&prefix),
                );
                // Also suppress a build error after cancellation: the old site
                // stays untouched and no further worker/build survives shutdown.
                if signal.load(Ordering::Acquire) {
                    return Ok(None);
                }
                let prepared = prepared?;
                publish.publish_with(|| prepared.publish())?;
                Ok(Some("Rebuilt hosted site; reloading preview.".into()))
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
