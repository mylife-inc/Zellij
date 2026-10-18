//! Running the Zellij engine.
//!
//! Every container invocation goes through here, so nothing else in the tool
//! shells out to `docker` and a podman or containerd backend later is one more
//! implementation rather than a rewrite.
//!
//! The engine validates content; the CLI does not. Reimplementing the schemas
//! in Rust would create a second source of truth that disagrees with the first
//! the week after it was written.

use camino::Utf8Path;

pub mod container;
pub mod image;
mod output;
pub mod runtime;
pub mod static_image;

pub use container::ContainerEngine;
pub use runtime::Runtime;

/// The public image a site is built from.
pub const DEFAULT_IMAGE: &str = "ghcr.io/mylife-inc/zellij-base:latest";

/// Resolve the image name without requiring or contacting a container runtime.
pub fn configured_image(image: Option<&str>) -> String {
    image
        .map(str::to_owned)
        .or_else(|| std::env::var("ZELLIJ_IMAGE").ok())
        .unwrap_or_else(|| DEFAULT_IMAGE.to_owned())
}

/// What `zel build` produces. Two, not four — see the CLI spec §3.5.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildMode {
    /// A folder of HTML, CSS and JS, for a CDN.
    Static,
    /// A container running the site on a port.
    Image,
}

#[derive(Debug, Default)]
pub struct CheckReport {
    pub ok: bool,
    pub output: String,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct CheckOptions {
    pub deep: bool,
    pub strict: bool,
}

pub trait Engine: Send + Sync {
    fn check(&self, content: &Utf8Path, options: CheckOptions) -> zellij_core::Result<CheckReport>;

    fn build(&self, content: &Utf8Path, out: &Utf8Path, mode: BuildMode)
    -> zellij_core::Result<()>;
}
