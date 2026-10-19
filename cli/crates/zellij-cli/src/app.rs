//! Native wrapper scaffolding; no package manager, runtime or network execution.
use camino::{Utf8Path, Utf8PathBuf};
use zellij_core::{
    Error, Result, files,
    site::{Renderer, Site},
};

pub(crate) fn create(options: &crate::args::NewAppArgs) -> Result<()> {
    let cwd = Utf8PathBuf::from_path_buf(std::env::current_dir()?)
        .map_err(|_| Error::InvalidArgument("working directory is not UTF-8".into()))?;
    let out = files::absolute_path(Utf8Path::new(&options.dir))?;
    let engine_path = options
        .engine
        .as_deref()
        .map(Utf8PathBuf::from)
        .unwrap_or_else(|| cwd.join("engine/packages/zellij"));
    if !engine_path.is_dir() {
        return Err(Error::InvalidArgument("new app requires a built local engine package: pass --engine <path-to-engine/packages/zellij>; the engine is not published to npm".into()));
    }
    let engine = files::absolute_path(&engine_path)?;
    files::disjoint(&engine, &out)?;
    validate_engine(&engine)?;
    // npm's file: specifier is a path, not a percent-encoded URL. Encoding it
    // makes npm look for literal %20/%23 directory names. On Windows strip
    // canonicalization's extended-length prefix and use portable separators.
    #[cfg(windows)]
    let package_path = if let Some(unc) = engine.as_str().strip_prefix(r"\\?\UNC\") {
        format!("//{}", unc.replace('\\', "/"))
    } else {
        engine
            .as_str()
            .strip_prefix(r"\\?\")
            .unwrap_or(engine.as_str())
            .replace('\\', "/")
    };
    #[cfg(not(windows))]
    let package_path = engine.as_str();
    let dependency = format!("file:{package_path}");
    let (content, port) = if options.with_docs {
        (
            "docs/zellij/content".to_owned(),
            options.port.unwrap_or(3000),
        )
    } else {
        let root = crate::select_site(options.site_root.as_ref())?;
        let site = Site::load(&root)?;
        if site.renderer != Renderer::Hosted {
            return Err(Error::InvalidArgument("new app currently references hosted content; use --with-docs for a hosted starter, or `zel image next` to package portable Markdown with its adapter".into()));
        }
        let content = crate::resolve_content(&root)?;
        files::disjoint(&content, &out)?;
        if content.join("marketing").exists() {
            return Err(Error::InvalidArgument("new app requires the current hosted folder layout; migrate legacy marketing/ content first".into()));
        }
        (content.into_string(), options.port.unwrap_or(site.port))
    };
    let name = options
        .name
        .as_deref()
        .unwrap_or_else(|| out.file_name().unwrap_or("Zellij app"));
    let scaffold = zellij_scaffold::app::app(&zellij_scaffold::app::AppOptions {
        name,
        engine_dependency: &dependency,
        content: &content,
        port,
        features: &options.features,
        with_docs: options.with_docs,
    })?;
    let files = scaffold.write(&out, false)?;
    zellij_core::statusln!(
        "✓ Created Next.js wrapper at {out} ({} files)\n  Content: {content}\n  Engine:  {engine}\n\nIn the app folder, run `npm install`, then `npm run check` and `npm run dev`.\nScaffolding did not install dependencies or start a server.",
        files.len()
    );
    Ok(())
}

pub(crate) fn validate_engine(engine: &Utf8Path) -> Result<()> {
    let manifest = files::contained_path(engine, Utf8Path::new("package.json"))?;
    let manifest: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(manifest)?)
        .map_err(|_| Error::InvalidArgument("engine package.json is not valid JSON".into()))?;
    let template = zellij_scaffold::app::template();
    if manifest["name"] != "@shebka/zellij"
        || manifest["version"] != template["engineVersion"]
        || manifest["zellij"]["wrapperConfigVersion"] != template["wrapperConfigVersion"]
        || manifest["zellij"]["featureSelectionVersion"] != template["featureSelectionVersion"]
    {
        return Err(Error::InvalidArgument(format!(
            "selected engine must be @shebka/zellij version {} with matching wrapper/feature capabilities; update the built engine or pull the current image",
            template["engineVersion"]
        )));
    }
    for file in [
        "bin/zellij.mjs",
        "next-config.mjs",
        "deployment-config.cjs",
        "feature-config.cjs",
        "feature-config.d.cts",
        "optional-maths.mjs",
        "optional-maths.d.mts",
        "optional-mermaid.mjs",
        "optional-mermaid.d.mts",
        "disabled-feature.mjs",
        "src/index.ts",
        "dist/cli/check-bundle.js",
        "dist/cli/dev-bundle.js",
        "dist/cli/init-bundle.js",
        "dist/cli/sync-assets-bundle.js",
        "dist/cli/write-search-index-bundle.js",
    ] {
        if !files::contained_path(engine, Utf8Path::new(file))?.is_file() {
            return Err(Error::InvalidArgument("engine build is incomplete; run `npm ci` and `npm run build --workspace @shebka/zellij` in its checkout before scaffolding".into()));
        }
    }
    Ok(())
}
