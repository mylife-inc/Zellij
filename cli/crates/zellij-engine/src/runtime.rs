//! Finding a container runtime, and being specific when there isn't one.
//!
//! "Docker is not running" is wrong more often than it is right. The usual
//! cause is a runtime that *is* running on a socket the default context does
//! not look at — Colima puts its socket under `~/.colima`, and `docker info`
//! fails with a message about the daemon while the daemon is fine. So the
//! sockets are probed, and whatever is found is reported by path.

use std::process::Command;
use zellij_core::{Error, Result};

/// A container runtime this machine can actually use.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Runtime {
    /// The executable: `docker`, `podman`, `nerdctl`.
    pub program: String,
    /// A `DOCKER_HOST` to set, when the default context does not find it.
    pub host: Option<String>,
    /// What `<program> --version` said, for `zel doctor`.
    pub version: String,
}

impl Runtime {
    /// A command against this runtime, with the socket already wired up.
    pub fn command(&self) -> Command {
        let mut command = Command::new(&self.program);
        if let Some(host) = &self.host {
            command.env("DOCKER_HOST", host);
        }
        command
    }

    /// Where it is talking to, in the terms `zel doctor` prints.
    pub fn endpoint(&self) -> String {
        self.host
            .clone()
            .unwrap_or_else(|| "default context".to_owned())
    }
}

/// Candidate sockets, in the order worth trying.
///
/// The user's own `DOCKER_HOST` is not in this list because it is honoured
/// first, by the plain `docker info` attempt — someone who set it meant it.
fn candidate_hosts() -> Vec<String> {
    let mut hosts = Vec::new();

    if let Ok(home) = std::env::var("HOME") {
        // Colima's default profile, then a named one people commonly use.
        hosts.push(format!("unix://{home}/.colima/default/docker.sock"));
        hosts.push(format!("unix://{home}/.docker/run/docker.sock"));
        hosts.push(format!("unix://{home}/.rd/docker.sock")); // Rancher Desktop
        hosts.push(format!(
            "unix://{}/podman/podman.sock",
            std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| format!("{home}/.local/run"))
        ));
    }

    hosts.push("unix:///var/run/docker.sock".to_owned());
    hosts
}

fn responds(program: &str, host: Option<&str>) -> bool {
    let mut command = Command::new(program);
    if let Some(host) = host {
        command.env("DOCKER_HOST", host);
    }

    command
        .arg("info")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

fn version_of(program: &str, host: Option<&str>) -> String {
    let mut command = Command::new(program);
    if let Some(host) = host {
        command.env("DOCKER_HOST", host);
    }

    command
        .arg("--version")
        .output()
        .ok()
        .and_then(|out| String::from_utf8(out.stdout).ok())
        .map(|text| text.trim().to_owned())
        .unwrap_or_default()
}

/// The first runtime that answers, or a failure naming what was tried.
pub fn detect() -> Result<Runtime> {
    // Whatever the environment already says, first. A CI runner, a remote
    // context, or somebody's deliberate DOCKER_HOST all land here.
    for program in ["docker", "podman", "nerdctl"] {
        if responds(program, None) {
            return Ok(Runtime {
                program: program.to_owned(),
                host: None,
                version: version_of(program, None),
            });
        }
    }

    // Then the sockets a working runtime hides behind.
    for host in candidate_hosts() {
        if !std::path::Path::new(host.trim_start_matches("unix://")).exists() {
            continue;
        }

        for program in ["docker", "podman"] {
            if responds(program, Some(&host)) {
                return Ok(Runtime {
                    program: program.to_owned(),
                    host: Some(host.clone()),
                    version: version_of(program, Some(&host)),
                });
            }
        }
    }

    let installed: Vec<&str> = ["docker", "podman", "nerdctl"]
        .into_iter()
        .filter(|program| which(program))
        .collect();

    let reason = if installed.is_empty() {
        "none of docker, podman or nerdctl is installed.\n\
         On a Mac, `brew install colima && colima start` is the smallest option."
            .to_owned()
    } else {
        format!(
            "{} is installed but not answering, and none of the usual sockets exist.\n\
             Try `colima start`, or start Docker Desktop.",
            installed.join(" and ")
        )
    };

    Err(Error::NoEngine { reason })
}

fn which(program: &str) -> bool {
    let Ok(path) = std::env::var("PATH") else {
        return false;
    };

    std::env::split_paths(&path).any(|dir| dir.join(program).is_file())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_runtime_carries_its_socket_into_every_command() {
        let runtime = Runtime {
            program: "docker".into(),
            host: Some("unix:///tmp/some.sock".into()),
            version: "27.0.0".into(),
        };

        let command = runtime.command();
        let host = command
            .get_envs()
            .find(|(key, _)| *key == "DOCKER_HOST")
            .and_then(|(_, value)| value);

        assert_eq!(host.unwrap(), "unix:///tmp/some.sock");
        assert_eq!(runtime.endpoint(), "unix:///tmp/some.sock");
    }

    #[test]
    fn a_runtime_on_the_default_context_sets_nothing() {
        let runtime = Runtime {
            program: "docker".into(),
            host: None,
            version: "27.0.0".into(),
        };

        assert!(runtime.command().get_envs().next().is_none());
        assert_eq!(runtime.endpoint(), "default context");
    }

    // The Colima socket has to be in the list, because that is the case the
    // whole probe exists for.
    #[test]
    fn the_candidates_include_colimas_socket() {
        unsafe { std::env::set_var("HOME", "/home/someone") };
        let hosts = candidate_hosts();

        assert!(
            hosts
                .iter()
                .any(|h| h == "unix:///home/someone/.colima/default/docker.sock"),
            "{hosts:?}"
        );
        assert!(
            hosts.iter().any(|h| h == "unix:///var/run/docker.sock"),
            "{hosts:?}"
        );
    }
}
