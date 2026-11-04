use super::{CONTENT_INVALID, EMPTY_MOUNT};
use crate::runtime::{Probe, Runtime, probe_output, probe_status};
use camino::Utf8PathBuf;
use serde::Deserialize;
use std::{process::ExitStatus, time::Duration};
use zellij_core::{Error, Result, process::Running};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DevReady {
    pub version: u8,
    pub session: String,
    pub base_path: String,
    pub revision: String,
}

/// Own one named container and its foreground client through cancellation.
pub struct DevSession {
    runtime: Runtime,
    _identity: tempfile::TempDir,
    name: String,
    content: Utf8PathBuf,
    process: Running,
    finished: bool,
}

impl DevSession {
    pub(super) fn new(
        runtime: Runtime,
        identity: tempfile::TempDir,
        name: String,
        content: Utf8PathBuf,
        process: Running,
    ) -> Self {
        Self {
            runtime,
            _identity: identity,
            name,
            content,
            process,
            finished: false,
        }
    }

    pub fn ready(&self) -> Option<DevReady> {
        // The session marker comes from this exact container, not any existing
        // listener on the requested port. No host-side metadata mount is needed.
        let text = probe_output(self.runtime.command().args([
            "exec", &self.name, "node", "-e",
            "process.stdout.write(require('node:fs').readFileSync('/app/public/_zellij/dev-revision.json','utf8'))",
        ]), Duration::from_millis(500))?;
        let ready: DevReady = serde_json::from_str(&text).ok()?;
        (ready.version == 1 && ready.session == self.name && !ready.revision.is_empty())
            .then_some(ready)
    }

    pub fn try_wait(&mut self) -> Result<Option<()>> {
        self.process
            .try_wait()?
            .map(|status| self.result(status))
            .transpose()
    }

    pub fn wait(mut self) -> Result<()> {
        let status = self.process.wait()?;
        self.result(status)
    }

    fn result(&mut self, status: ExitStatus) -> Result<()> {
        self.finished = true;
        match status.code() {
            Some(EMPTY_MOUNT) => Err(Error::MountEmpty {
                path: self.content.clone(),
            }),
            Some(0) | Some(130) | None => Ok(()),
            Some(CONTENT_INVALID) => Err(Error::ContentInvalid),
            Some(_) => Err(Error::BuildFailed),
        }
    }

    pub fn stop(&mut self) {
        // Stop the owned container even if the foreground client has already
        // received Ctrl-C. A client exiting does not prove its container exited.
        let stopped = probe_status(
            self.runtime.command().args(["stop", "-t", "2", &self.name]),
            Duration::from_secs(3),
        ) == Probe::Ready;
        let _ = self.process.cancel();
        // Reap a failed/pulling client before the final removal attempt, so it
        // cannot create the container after that attempt has completed.
        let removed = stopped
            || probe_status(
                self.runtime.command().args(["rm", "--force", &self.name]),
                Duration::from_secs(2),
            ) == Probe::Ready;
        self.finished = true;
        if !removed {
            zellij_core::diagnosticln!(
                "warning: could not confirm container shutdown; check `{0} ps` and stop {1} if it remains running.",
                self.runtime.program,
                self.name
            );
        }
    }
}

impl Drop for DevSession {
    fn drop(&mut self) {
        if !self.finished {
            self.stop();
        }
    }
}
