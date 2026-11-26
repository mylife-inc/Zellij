use super::{DevReady, process::Tree};
use camino::Utf8Path;
use std::{fs, net::IpAddr, process::Command, sync::atomic::AtomicBool};
use zellij_core::{Error, Result, features::Features, files};

pub(super) struct Session {
    // Drop the entire process tree before deleting its working directory.
    process: Tree,
    work: tempfile::TempDir,
    identity: String,
}
impl Session {
    pub fn start(
        content: &Utf8Path,
        port: u16,
        host: IpAddr,
        features: Option<Features>,
        offline: bool,
        stop: &AtomicBool,
    ) -> Result<Self> {
        let node = crate::rich_build::private_node(offline, stop)?;
        let sources = crate::source_package::obtain(offline, stop)?;
        let dependencies = crate::development_package::obtain(offline, stop)?;
        let home = crate::source_package::utf8(&crate::user_home::root()?)?;
        let sessions = files::contained_path(&home, Utf8Path::new("cache/development-sessions"))?;
        fs::create_dir_all(&sessions)?;
        let work = tempfile::Builder::new()
            .prefix("zel-dev-")
            .rand_bytes(16)
            .tempdir_in(&sessions)?;
        let root = crate::source_package::utf8(work.path())?;
        sources.copy("app", &root, stop)?;
        // Dependencies carry their own build manifest; the app manifest is the
        // canonical source wrapper and must keep its feature selection.
        fs::remove_file(root.join("package.json"))?;
        let modules = dependencies.prepare(work.path(), stop)?;
        let modules = ordinary_path(crate::source_package::utf8(&modules)?.as_str());
        sources.copy("engine", &root.join("node_modules/@shebka/zellij"), stop)?;
        let mut manifest: serde_json::Value =
            serde_json::from_slice(&fs::read(root.join("package.json"))?)
                .map_err(std::io::Error::other)?;
        let features = features.unwrap_or_default();
        manifest["type"] = serde_json::json!("module");
        let selection = [
            if features.maths { Some("maths") } else { None },
            if features.diagrams {
                Some("diagrams")
            } else {
                None
            },
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();
        manifest["zellij"] = serde_json::json!({"features": selection});
        fs::write(
            root.join("package.json"),
            serde_json::to_vec(&manifest).map_err(std::io::Error::other)?,
        )?;
        let content = files::absolute_path(content)?;
        // JSON is also valid YAML. No authored string is interpolated into JS.
        let content = ordinary_path(content.as_str());
        fs::write(
            root.join("zellij.config.yaml"),
            serde_json::to_vec(&serde_json::json!({"version":1,"content":content}))
                .map_err(std::io::Error::other)?,
        )?;
        fs::write(
            root.join("request.json"),
            serde_json::to_vec(
                &serde_json::json!({"content":content,"host":host.to_string(),"port":port,"modules":modules}),
            )
            .map_err(std::io::Error::other)?,
        )?;
        crate::development_package::check_stop(stop)?;
        let identity = root.file_name().unwrap().to_owned();
        let mut command = Command::new(&node);
        command
            .arg(root.join("run.mjs"))
            .arg(root.join("request.json"))
            .current_dir(&root)
            // Turbopack's CSS worker resolves Node by name. Expose only the
            // verified private runtime to descendants, never the user's tools.
            .env(
                "PATH",
                ordinary_path(crate::source_package::utf8(node.parent().unwrap())?.as_str()),
            )
            .env_remove("NODE_OPTIONS")
            .env_remove("NODE_PATH")
            .env_remove("NODE_V8_COVERAGE")
            .env_remove("ZELLIJ_OUTPUT")
            .env_remove("ZELLIJ_BASE_PATH")
            .env_remove("ZELLIJ_BUILD_OVERRIDES")
            .env_remove("ZELLIJ_MATHS")
            .env_remove("NEXT_PUBLIC_ZELLIJ_DIAGRAMS")
            .env("NODE_ENV", "development")
            .env("NEXT_TELEMETRY_DISABLED", "1")
            .env("ZELLIJ_DEV_SESSION", &identity);
        let process = Tree::spawn(&mut command)?;
        Ok(Self {
            process,
            work,
            identity,
        })
    }
    pub fn ready(&self) -> Option<DevReady> {
        let path = self.work.path().join("public/_zellij/dev-revision.json");
        if fs::metadata(&path).ok()?.len() > 4096 {
            return None;
        }
        let ready: DevReady = serde_json::from_slice(&fs::read(path).ok()?).ok()?;
        (ready.version == 1 && ready.session == self.identity && !ready.revision.is_empty())
            .then_some(ready)
    }
    pub fn try_wait(&mut self) -> Result<Option<()>> {
        self.process
            .try_wait()?
            .map(|s| {
                if s.success() || s.code() == Some(130) || s.code() == Some(143) {
                    Ok(())
                } else {
                    Err(Error::BuildFailed)
                }
            })
            .transpose()
    }
    pub fn stop(&mut self) {
        self.process.stop();
    }
}
fn ordinary_path(value: &str) -> String {
    if cfg!(windows) {
        if let Some(path) = value.strip_prefix(r"\\?\UNC\") {
            return format!(r"\\{path}");
        }
        if let Some(path) = value.strip_prefix(r"\\?\") {
            return path.into();
        }
    }
    value.into()
}
