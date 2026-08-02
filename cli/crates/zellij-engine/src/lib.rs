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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildMode {
    /// Static export, for a CDN.
    Static,
    /// A traced Node server, for anything needing image optimisation.
    Standalone,
}

#[derive(Debug, Default)]
pub struct CheckReport {
    pub ok: bool,
    pub output: String,
}

pub trait Engine: Send + Sync {
    fn check(&self, content: &Utf8Path) -> zellij_core::Result<CheckReport>;

    fn build(&self, content: &Utf8Path, out: &Utf8Path, mode: BuildMode)
    -> zellij_core::Result<()>;
}
