//! Preview the same rich export that can be opened directly from disk.
use super::{ViewOptions, run, signals};
use camino::{Utf8Path, Utf8PathBuf};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use zellij_core::{
    Error, Result, files,
    site::{Renderer, Site, SiteInput},
};
use zellij_docs::{
    server::{StaticServer, listener_on},
    watch::{LiveReload, Snapshot, Watcher},
};

fn snapshot(source: &SiteInput, configured: bool) -> Result<Snapshot> {
    Snapshot::site_for(source, configured, Renderer::Hosted)
}

struct Build<'a> {
    source: &'a SiteInput,
    output: &'a Utf8Path,
    configured: bool,
    features: Option<zellij_core::features::Features>,
    offline: bool,
    stop: &'a AtomicBool,
}
impl Build<'_> {
    fn prepare(
        &self,
        expected: &Snapshot,
        prefix: Option<&str>,
    ) -> Result<Option<crate::rich_build::PreparedRich>> {
        if snapshot(self.source, self.configured)? != *expected {
            return Err(Error::InvalidArgument("content changed during the build; retry after saving (watch mode retries automatically)".into()));
        }
        let content = crate::resolve_content(self.source)?;
        let Some(build) = crate::rich_build::prepare(
            &content,
            self.output,
            &zellij_engine::BuildOptions::with_features(self.features),
            self.offline,
            Some(self.stop),
        )?
        else {
            return Ok(None);
        };
        if snapshot(self.source, self.configured)? != *expected {
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
        Ok(Some(build))
    }
}

pub(super) fn view(site: Site, options: ViewOptions<'_>) -> Result<()> {
    if options.pdf == Some(true) {
        return Err(Error::InvalidArgument(
            "--pdf requires a portable book".into(),
        ));
    }
    if options.name.is_some() || options.theme.is_some() {
        return Err(Error::InvalidArgument("--name and --theme apply only to portable previews; configure rich sites in content/site.yaml".into()));
    }
    // Binding failure must not download a renderer or change the previous site.
    let socket = listener_on(options.host, options.port.unwrap_or(site.port))?;
    let output = files::absolute_path(&options.out.map(Utf8PathBuf::from).unwrap_or(site.output))?;
    let source = site.input;
    let configured = site.configured;
    let initial = snapshot(&source, configured)?;
    let stop = signals()?;
    let features = options.features;
    let build = Build {
        source: &source,
        output: &output,
        configured,
        features,
        offline: options.offline,
        stop: &stop,
    }
    .prepare(&initial, None);
    if stop.load(Ordering::Acquire) {
        zellij_core::statusln!("Stopped.");
        return Ok(());
    }
    let Some(build) = build? else {
        return Ok(());
    };
    let prefix = build.base_path.clone();
    build.publish()?;
    zellij_core::statusln!("✓ {output}");
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
                let build = Build {
                    source: &source,
                    output: &output,
                    configured,
                    features,
                    offline: true,
                    stop: &signal,
                }
                .prepare(expected, Some(&prefix));
                if signal.load(Ordering::Acquire) {
                    return Ok(None);
                }
                let Some(build) = build? else {
                    return Ok(None);
                };
                publish.publish_with(|| build.publish())?;
                Ok(Some("Rebuilt rich site; reloading preview.".into()))
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
