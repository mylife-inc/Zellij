//! The engine, run in a container.
//!
//! `check` is the cheap one: the base image carries the engine's CLI as
//! bundled JavaScript at `/opt/zellij`, with nothing installed, so validating
//! a folder is a mount and a `node` invocation. No `npm install`, no Next.js,
//! about a second.
//!
//! `build` and `dev` are not cheap, because they run the real application.

use camino::Utf8Path;
use std::process::Stdio;
use zellij_core::{Error, Result};

use crate::runtime::Runtime;
use crate::{BuildMode, CheckReport, Engine};

/// Where the engine's own CLI lives inside the image.
const ENGINE_CLI: &str = "/opt/zellij/bin/zellij.mjs";

/// Where a mounted content folder appears inside the image.
///
/// Not `/app/content`: the wrapper lives at `/app`, and mounting over part of
/// it would replace a directory the image put there. A separate path means the
/// mount can never collide with the image's own layout.
const MOUNTED_CONTENT: &str = "/content";

pub struct ContainerEngine {
    runtime: Runtime,
    image: String,
}

impl ContainerEngine {
    pub fn new(runtime: Runtime, image: impl Into<String>) -> Self {
        Self {
            runtime,
            image: image.into(),
        }
    }

    /// The default image, unless `ZELLIJ_IMAGE` says otherwise — which is what
    /// an organisation mirroring it internally needs.
    pub fn detect(image: Option<&str>) -> Result<Self> {
        let runtime = crate::runtime::detect()?;
        let image = image
            .map(str::to_owned)
            .or_else(|| std::env::var("ZELLIJ_IMAGE").ok())
            .unwrap_or_else(|| crate::DEFAULT_IMAGE.to_owned());

        Ok(Self::new(runtime, image))
    }

    pub fn runtime(&self) -> &Runtime {
        &self.runtime
    }

    pub fn image(&self) -> &str {
        &self.image
    }

    /// Is the image already on this machine?
    pub fn image_is_local(&self) -> bool {
        self.runtime
            .command()
            .args(["image", "inspect", &self.image])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|status| status.success())
            .unwrap_or(false)
    }

    /// The mount argument for a content folder, read-only.
    ///
    /// Read-only because the engine has no business writing into somebody's
    /// content, and a container that cannot is a container that will not.
    fn content_mount(content: &Utf8Path) -> Result<String> {
        let absolute = content
            .canonicalize_utf8()
            .map_err(|_| Error::NoContentFolder {
                path: content.to_owned(),
            })?;

        Ok(format!("{absolute}:{MOUNTED_CONTENT}:ro"))
    }
}

impl Engine for ContainerEngine {
    fn check(&self, content: &Utf8Path) -> Result<CheckReport> {
        let mount = Self::content_mount(content)?;

        let output = self
            .runtime
            .command()
            .args(["run", "--rm", "-v", &mount, &self.image])
            .args(["node", ENGINE_CLI, "check", MOUNTED_CONTENT])
            .output()?;

        // The engine prints its findings to both streams, and a reader wants
        // them in one piece rather than sorted by which one they came from.
        let mut text = String::from_utf8_lossy(&output.stdout).into_owned();
        text.push_str(&String::from_utf8_lossy(&output.stderr));

        // The mounted path is an implementation detail. Somebody who ran
        // `zel check ./docs` should read about ./docs.
        let text = text.replace(MOUNTED_CONTENT, content.as_str());

        Ok(CheckReport {
            ok: output.status.success(),
            output: text,
        })
    }

    fn build(&self, _content: &Utf8Path, _out: &Utf8Path, _mode: BuildMode) -> Result<()> {
        Err(Error::NoEngine {
            reason: "build is not implemented yet".into(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn engine() -> ContainerEngine {
        ContainerEngine::new(
            Runtime {
                program: "docker".into(),
                host: None,
                version: "27.0.0".into(),
            },
            "ghcr.io/example/zellij-base:latest",
        )
    }

    #[test]
    fn a_missing_content_folder_is_named_rather_than_mounted() {
        let err =
            ContainerEngine::content_mount(Utf8Path::new("./definitely-not-here")).unwrap_err();
        assert!(matches!(err, Error::NoContentFolder { .. }), "{err:?}");
    }

    #[test]
    fn the_mount_is_absolute_and_read_only() {
        let dir = tempfile::tempdir().unwrap();
        let path = camino::Utf8PathBuf::from_path_buf(dir.path().to_path_buf()).unwrap();

        let mount = ContainerEngine::content_mount(&path).unwrap();

        assert!(mount.ends_with(":/content:ro"), "{mount}");
        assert!(
            mount.starts_with('/'),
            "the mount source must be absolute: {mount}"
        );
    }

    #[test]
    fn the_image_can_be_overridden_for_an_internal_mirror() {
        assert_eq!(engine().image(), "ghcr.io/example/zellij-base:latest");
    }
}
