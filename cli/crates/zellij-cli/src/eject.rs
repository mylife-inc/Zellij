//! Copy a stopped distribution container; do not execute image or host scripts.
use camino::{Utf8Path, Utf8PathBuf};
use std::{
    fs,
    process::{Command, Stdio},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::{Duration, Instant},
};
use zellij_core::{
    Error, Result, files,
    site::{Renderer, Site, is_link},
};
use zellij_engine::{ContainerEngine, runtime::Runtime};

pub(crate) fn run(options: &crate::args::EjectArgs) -> Result<()> {
    let requested = Utf8Path::new(&options.out);
    if fs::symlink_metadata(requested).is_ok_and(|m| is_link(&m)) {
        return Err(files::unsafe_path(
            requested,
            "destination cannot be a symlink or junction",
        ));
    }
    let out = files::absolute_path(requested)?;
    files::empty_directory(&out)?;
    let content = if options.with_docs {
        None
    } else {
        let root = crate::select_site(options.dir.as_ref())?;
        let site = Site::load(&root)?;
        if site.renderer != Renderer::Hosted {
            return Err(Error::InvalidArgument("eject references hosted content; use --with-docs for the image starter, or `zel image next` to adapt portable Markdown".into()));
        }
        files::disjoint(&site.root, &out)?;
        let content = crate::resolve_content(&root)?;
        files::disjoint(&content, &out)?;
        if content.join("marketing").exists() {
            return Err(Error::InvalidArgument(
                "migrate the legacy marketing/ layout before ejecting a wrapper".into(),
            ));
        }
        Some(content)
    };
    let image = zellij_engine::configured_image(options.image.as_deref());
    crate::maintenance::validate_image(&image)?;
    let engine = ContainerEngine::detect(Some(&image))?;
    let stop = Arc::new(AtomicBool::new(false));
    let signal = stop.clone();
    ctrlc::set_handler(move || signal.store(true, Ordering::Release))
        .map_err(std::io::Error::other)?;
    let parent = out
        .parent()
        .ok_or_else(|| files::unsafe_path(&out, "destination needs a parent"))?;
    fs::create_dir_all(parent)?;
    let temp = tempfile::Builder::new()
        .prefix("zel-eject-")
        .tempdir_in(parent)?;
    let stage = Utf8Path::from_path(temp.path())
        .ok_or_else(|| files::unsafe_path(&out, "non-UTF-8 staging path"))?;
    let app = stage.join("app");
    let vendor = stage.join("engine");
    fs::create_dir(&app)?;
    fs::create_dir(&vendor)?;
    let mut snapshot = Snapshot {
        runtime: engine.runtime(),
        name: stage.file_name().unwrap().to_owned(),
        active: true,
    };
    println!(
        "Extracting wrapper and engine from {image} with {}",
        engine.runtime().program
    );
    let result: Result<()> = (|| {
        execute(
            engine.runtime().command().args([
                "create",
                "--name",
                &snapshot.name,
                "--entrypoint",
                "node",
                &image,
                "--version",
            ]),
            &stop,
            300,
        )?;
        for (source, dest) in [("/app/.", &app), ("/opt/zellij/.", &vendor)] {
            execute(
                engine.runtime().command().args([
                    "cp",
                    &format!("{}:{source}", snapshot.name),
                    &local_path(dest),
                ]),
                &stop,
                120,
            )?;
        }
        Ok(())
    })();
    let cleaned = snapshot.remove();
    result?;
    cleaned?;
    if stop.load(Ordering::Acquire) {
        return Err(failed("extraction cancelled"));
    }
    validate_tree(&app)?;
    validate_tree(&vendor)?;
    crate::app::validate_engine(&vendor)?;
    for file in [
        "package.json",
        "tsconfig.json",
        "next.config.js",
        ".npmrc",
        "src/app/layout.tsx",
        "src/app/[[...slug]]/page.tsx",
    ] {
        if !app.join(file).is_file() {
            return Err(incompatible(
                "image does not contain a complete editable wrapper at /app",
            ));
        }
    }
    if app.join("vendor").exists()
        || app.join("docs").exists()
        || app.join("zellij.config.yaml").exists()
    {
        return Err(incompatible(
            "image wrapper already has vendor/, docs/ or zellij.config.yaml; incompatible extraction layout",
        ));
    }
    let mut manifest: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(app.join("package.json"))?)
            .map_err(|e| incompatible(&format!("invalid wrapper manifest: {e}")))?;
    if manifest["dependencies"]["@shebka/zellij"] != "file:/opt/zellij" {
        return Err(incompatible(
            "image wrapper must reference file:/opt/zellij",
        ));
    }
    manifest["dependencies"]["@shebka/zellij"] = serde_json::json!("file:vendor/zellij");
    let mut tsconfig: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(app.join("tsconfig.json"))?)
            .map_err(|e| incompatible(&format!("tsconfig.json must be JSON: {e}")))?;
    if !tsconfig.is_object() {
        return Err(incompatible("tsconfig.json must be an object"));
    }
    if tsconfig.get("exclude").is_none() {
        tsconfig["exclude"] = serde_json::json!(["node_modules"]);
    }
    let excludes = tsconfig["exclude"]
        .as_array_mut()
        .ok_or_else(|| incompatible("tsconfig exclude must be an array"))?;
    if !excludes.iter().any(|v| v == "vendor/zellij") {
        excludes.push(serde_json::json!("vendor/zellij"));
    }
    // The selected image's wrapper sources, scripts, features and overrides are
    // retained. Only container-specific locations are adapted for this folder.
    fs::create_dir(app.join("vendor"))?;
    fs::rename(&vendor, app.join("vendor/zellij"))?;
    let content = if let Some(content) = content {
        if app.join("content").exists() {
            fs::remove_dir_all(app.join("content"))?;
        }
        local_path(&content)
    } else {
        if !app.join("content/site.yaml").is_file() {
            return Err(incompatible("image has no hosted starter content"));
        }
        fs::create_dir_all(app.join("docs/zellij"))?;
        fs::rename(app.join("content"), app.join("docs/zellij/content"))?;
        fs::write(
            app.join("docs/zellij/zel.yaml"),
            "version: 1\nrenderer: hosted\n",
        )?;
        "docs/zellij/content".into()
    };
    files::write_files(
        &app,
        &[
            (
                Utf8Path::new("package.json"),
                format!("{}\n", serde_json::to_string_pretty(&manifest).unwrap()),
            ),
            (
                Utf8Path::new("tsconfig.json"),
                format!("{}\n", serde_json::to_string_pretty(&tsconfig).unwrap()),
            ),
            (
                Utf8Path::new("zellij.config.yaml"),
                format!(
                    "version: 1\ncontent: {}\n",
                    zellij_core::yaml::string(&content)
                ),
            ),
            (
                Utf8Path::new("README.md"),
                format!(
                    r#"# Ejected Zellij wrapper

Extracted from `{image}`. The selected distribution's editable wrapper is in
`src/app/`; its engine is included at `vendor/zellij` as a relative dependency.
Content is selected in `zellij.config.yaml`. External content is referenced,
not copied; update that path if you move it. `--with-docs` content moves with
this folder. Keep the vendored engine with the wrapper when moving it.

Install Node matching package.json's engines field, then run:

```sh
npm install
npm run check
npm run dev
```

`npm run build` builds the Next.js app and `npm start` serves it. This hosted
wrapper requires HTTP serving. `zel build` remains the native portable option.
`zel features all` (or maths, diagrams, none) changes rendering and direct feature dependencies.
Run npm install when enabling features, then rebuild/restart. Disabling a feature
also stops rendering when its packages remain installed for another dependency.

Extraction did not install dependencies or execute the image's application.
"#
                ),
            ),
        ],
        true,
    )?;
    validate_tree(&app)?;
    if stop.load(Ordering::Acquire) {
        return Err(failed("extraction cancelled"));
    }
    // Recheck immediately before publishing; never replace authored output.
    files::empty_directory(&out)?;
    if out.exists() {
        fs::remove_dir(&out)?;
    }
    fs::rename(&app, &out)?;
    println!(
        "✓ Editable wrapper: {out}\n  Content: {content}\n  Engine: vendor/zellij\n\nRun `npm install`, `npm run check` and `npm run dev` in the wrapper.\nNo dependencies were installed. The temporary stopped container was removed."
    );
    Ok(())
}
fn local_path(path: &Utf8Path) -> String {
    let path = path.as_str();
    if let Some(unc) = path.strip_prefix(r"\\?\UNC\") {
        format!(r"\\{unc}")
    } else {
        path.strip_prefix(r"\\?\").unwrap_or(path).into()
    }
}
fn incompatible(reason: &str) -> Error {
    Error::InvalidArgument(format!("incompatible engine distribution: {reason}"))
}
fn failed(reason: &str) -> Error {
    eprintln!("eject: {reason}");
    Error::ImageOperationFailed {
        operation: "extraction",
    }
}
fn execute(command: &mut Command, stop: &AtomicBool, timeout: u64) -> Result<()> {
    if stop.load(Ordering::Acquire) {
        return Err(failed("extraction cancelled"));
    }
    let mut child = command.stdin(Stdio::null()).spawn()?;
    let deadline = Instant::now() + Duration::from_secs(timeout);
    loop {
        if let Some(status) = child.try_wait()? {
            return if status.success() {
                Ok(())
            } else {
                Err(failed("container command failed"))
            };
        }
        if stop.load(Ordering::Acquire) || Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            return Err(failed("container command cancelled or timed out"));
        }
        thread::sleep(Duration::from_millis(50));
    }
}
struct Snapshot<'a> {
    runtime: &'a Runtime,
    name: String,
    active: bool,
}
impl Snapshot<'_> {
    fn remove(&mut self) -> Result<()> {
        self.active = false;
        let result = execute(
            self.runtime
                .command()
                .args(["rm", "-f", &self.name])
                .stdout(Stdio::null())
                .stderr(Stdio::null()),
            &AtomicBool::new(false),
            15,
        );
        if result.is_err() {
            eprintln!(
                "Could not remove stopped container {}; remove it with your container runtime after it responds.",
                self.name
            );
        }
        result
    }
}
impl Drop for Snapshot<'_> {
    fn drop(&mut self) {
        if self.active {
            let _ = self.remove();
        }
    }
}
fn validate_tree(root: &Utf8Path) -> Result<()> {
    let mut paths = Vec::new();
    let mut bytes = 0u64;
    for entry in walkdir::WalkDir::new(root).follow_links(false).min_depth(1) {
        let entry = entry.map_err(|e| failed(&e.to_string()))?;
        let path = Utf8Path::from_path(entry.path())
            .ok_or_else(|| failed("image has a non-UTF-8 filename"))?;
        let relative = path
            .strip_prefix(root)
            .unwrap()
            .iter()
            .collect::<Vec<_>>()
            .join("/");
        files::relative_path(Utf8Path::new(&relative))?;
        let metadata = fs::symlink_metadata(path)?;
        if is_link(&metadata) || (!metadata.is_file() && !metadata.is_dir()) {
            return Err(files::unsafe_path(
                path,
                "image extraction requires regular files/directories, not links or special files",
            ));
        }
        if relative
            .split('/')
            .any(|part| matches!(part, "node_modules" | ".next"))
        {
            return Err(failed(
                "image must contain sources without installed dependencies or build caches",
            ));
        }
        if metadata.is_file() {
            paths.push(Utf8PathBuf::from(relative));
            bytes = bytes.saturating_add(metadata.len());
            if paths.len() > 50_000 || bytes > 512 * 1024 * 1024 {
                return Err(failed(
                    "extracted distribution exceeds the supported source size",
                ));
            }
        }
    }
    files::preflight(
        root,
        &paths.iter().map(|p| p.as_path()).collect::<Vec<_>>(),
        true,
    )
}
