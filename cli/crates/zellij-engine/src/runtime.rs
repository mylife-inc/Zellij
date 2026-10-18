//! Bounded, non-interactive discovery of a usable container runtime.
//!
//! Explicit endpoint environment variables select a runtime. Without them,
//! try each runtime's configured connection, then Docker's usual local sockets.
//! Discovery never installs software, starts services, or changes configuration.

use std::{
    io::{Read, Seek},
    process::{Command, Stdio},
    time::{Duration, Instant},
};
use zellij_core::{Error, Result};

const INFO_TIMEOUT: Duration = Duration::from_secs(3);
const DISCOVERY_BUDGET: Duration = Duration::from_secs(12);
pub(crate) const METADATA_TIMEOUT: Duration = Duration::from_secs(2);

/// A container runtime this machine can actually use.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Runtime {
    /// The executable: `docker`, `podman`, `nerdctl`.
    pub program: String,
    /// A discovered local Docker socket; explicit endpoints stay in the environment.
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

    /// Explicit endpoint values can contain credentials. Report their variable
    /// names instead of printing the values in diagnostics.
    pub fn endpoint(&self) -> String {
        if let Some(host) = &self.host {
            return host.clone();
        }
        for (program, variables) in SELECTORS {
            if *program == self.program {
                for variable in *variables {
                    if env_is_set(variable) {
                        return format!("{variable} (value hidden)");
                    }
                }
            }
        }
        "default context".to_owned()
    }
}

const SELECTORS: &[(&str, &[&str])] = &[
    ("docker", &["DOCKER_CONTEXT", "DOCKER_HOST"]),
    ("podman", &["CONTAINER_CONNECTION", "CONTAINER_HOST"]),
    ("nerdctl", &["CONTAINERD_ADDRESS", "CONTAINERD_NAMESPACE"]),
];

fn env_is_set(name: &str) -> bool {
    std::env::var_os(name).is_some_and(|value| !value.is_empty())
}

fn selected_program(is_set: impl Fn(&str) -> bool) -> Result<Option<&'static str>> {
    let selected: Vec<_> = SELECTORS
        .iter()
        .filter(|(_, variables)| variables.iter().any(|name| is_set(name)))
        .collect();
    match selected.as_slice() {
        [] => Ok(None),
        [(program, _)] => Ok(Some(program)),
        _ => Err(Error::NoEngine {
            reason: format!(
                "endpoint environment variables select multiple runtimes: {}.\n\
                 Keep the endpoint variables for the runtime you intend to use and unset the others.",
                selected
                    .iter()
                    .map(|(program, _)| *program)
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        }),
    }
}

/// Docker-compatible API sockets. Podman uses CONTAINER_HOST rather than
/// DOCKER_HOST, so only the Docker client is probed against these overrides.
fn candidate_hosts(home: Option<&str>, runtime_dir: Option<&str>) -> Vec<String> {
    let mut hosts = Vec::new();
    if let Some(home) = home {
        hosts.push(format!("unix://{home}/.colima/default/docker.sock"));
        hosts.push(format!("unix://{home}/.docker/run/docker.sock"));
        hosts.push(format!("unix://{home}/.rd/docker.sock"));
    }
    if let Some(dir) = runtime_dir {
        hosts.push(format!("unix://{dir}/podman/podman.sock"));
    } else if let Some(home) = home {
        hosts.push(format!("unix://{home}/.local/run/podman/podman.sock"));
    }
    hosts.push("unix:///var/run/docker.sock".to_owned());
    hosts
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Probe {
    Ready,
    Failed,
    Missing,
    Unavailable,
    TimedOut,
}

/// Let the OS resolve executables, including `.exe` on Windows. A manual
/// `PATH`/is_file check misclassifies Windows installations and Unix permissions.
/// Null stdin prevents the probe waiting on an ordinary interactive prompt.
fn run_probe(command: &mut Command, stdout: Stdio, timeout: Duration) -> Probe {
    if timeout.is_zero() {
        return Probe::TimedOut;
    }
    let started = Instant::now();
    let mut child = match command
        .stdin(Stdio::null())
        .stdout(stdout)
        .stderr(Stdio::null())
        .spawn()
    {
        Ok(child) => child,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Probe::Missing,
        Err(_) => return Probe::Unavailable,
    };
    let outcome = loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                return if status.success() {
                    Probe::Ready
                } else {
                    Probe::Failed
                };
            }
            Ok(None) => {}
            Err(_) => break Probe::Unavailable,
        }
        let remaining = timeout.saturating_sub(started.elapsed());
        if remaining.is_zero() {
            break Probe::TimedOut;
        }
        std::thread::sleep(remaining.min(Duration::from_millis(10)));
    };
    // Reap the child on every unsuccessful wait path, not just on timeout.
    let _ = child.kill();
    let _ = child.wait();
    outcome
}

pub(crate) fn probe_status(command: &mut Command, timeout: Duration) -> Probe {
    run_probe(command, Stdio::null(), timeout)
}

/// A file avoids an undrained stdout pipe blocking a verbose client. Read only
/// a small prefix after exit; never wait for a descendant to close a pipe.
pub(crate) fn probe_output(command: &mut Command, timeout: Duration) -> Option<String> {
    let mut output = tempfile::tempfile().ok()?;
    let stdout = output.try_clone().ok()?;
    if run_probe(command, stdout.into(), timeout) != Probe::Ready {
        return None;
    }
    output.rewind().ok()?;
    let mut text = String::new();
    output.take(4096).read_to_string(&mut text).ok()?;
    Some(text.trim().to_owned())
}

fn setup_guidance(os: &str) -> &'static str {
    match os {
        "macos" => {
            "Start Docker Desktop, or your existing Colima/Podman machine (`colima start` / `podman machine start`).\nIf no runtime is installed, install Docker Desktop, Colima with the Docker CLI, or Podman, then retry."
        }
        "windows" => {
            "Start Docker Desktop in Linux-container mode, or your existing Podman machine (`podman machine start`).\nIf no runtime is installed, install Docker Desktop or Podman, then retry."
        }
        "linux" => {
            "Start your installed Docker/containerd service, or check that rootless Podman works.\nCheck socket permissions and the selected connection. If no runtime is installed, install Docker Engine, Podman, or containerd with nerdctl."
        }
        _ => {
            "Install or start a supported Docker, Podman, or nerdctl runtime, and check its connection and permissions."
        }
    }
}

fn select_runtime(
    selected: Option<&str>,
    hosts: &[String],
    mut probe: impl FnMut(&str, Option<&str>) -> Probe,
) -> Result<Runtime> {
    let mut failures = Vec::new();
    let mut docker_available = false;
    for program in ["docker", "podman", "nerdctl"] {
        if selected.is_some_and(|selected| selected != program) {
            continue;
        }
        let result = probe(program, None);
        if result == Probe::Ready {
            return Ok(Runtime {
                program: program.into(),
                host: None,
                version: String::new(),
            });
        }
        docker_available |=
            program == "docker" && !matches!(result, Probe::Missing | Probe::Unavailable);
        let description = match result {
            Probe::Missing => "executable not found",
            Probe::Unavailable => {
                "could not execute the client; check its installation and permissions"
            }
            Probe::Failed => "`info` failed; check the service, connection and permissions",
            Probe::TimedOut => "`info` timed out",
            Probe::Ready => unreachable!(),
        };
        failures.push(format!("  {program}: {description}"));
    }
    if selected.is_none() && docker_available {
        for host in hosts {
            let result = probe("docker", Some(host));
            if result == Probe::Ready {
                return Ok(Runtime {
                    program: "docker".into(),
                    host: Some(host.clone()),
                    version: String::new(),
                });
            }
            if result == Probe::TimedOut {
                failures.push("  docker: local socket probe timed out".into());
            }
        }
        if !hosts.is_empty() {
            failures.push(
                "  No usable connection found at the discovered local Docker sockets.".into(),
            );
        }
    }
    let selection = if selected.is_some() {
        "\nEndpoint environment variables select this runtime; no other runtime or local socket was substituted. Check those variables or unset them to enable discovery."
    } else {
        ""
    };
    Err(Error::NoEngine {
        reason: format!(
            "\n{}{selection}\n{}\nRun the selected client's `info` command for its detailed error.\nPortable build/check/serve/view and Dockerfile generation work without a container runtime.",
            failures.join("\n"),
            setup_guidance(std::env::consts::OS)
        ),
    })
}

/// Detect a runtime without an unbounded wait. The info probes share a 12-second
/// budget, with at most 3 seconds per process; version reporting gets 2 seconds.
pub fn detect() -> Result<Runtime> {
    let selected = selected_program(env_is_set)?;
    let hosts = if cfg!(unix) && selected.is_none() {
        candidate_hosts(
            std::env::var("HOME").ok().as_deref(),
            std::env::var("XDG_RUNTIME_DIR").ok().as_deref(),
        )
        .into_iter()
        .filter(|host| std::path::Path::new(host.trim_start_matches("unix://")).exists())
        .collect::<Vec<_>>()
    } else {
        Vec::new()
    };
    let started = Instant::now();
    let mut runtime = select_runtime(selected, &hosts, |program, host| {
        let mut command = Command::new(program);
        if let Some(host) = host {
            command.env("DOCKER_HOST", host);
        }
        probe_status(
            command.arg("info"),
            INFO_TIMEOUT.min(DISCOVERY_BUDGET.saturating_sub(started.elapsed())),
        )
    })?;
    runtime.version = probe_output(runtime.command().arg("--version"), METADATA_TIMEOUT)
        .filter(|version| !version.is_empty())
        .unwrap_or_else(|| "version unavailable".into());
    Ok(runtime)
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
    }

    #[test]
    fn local_sockets_do_not_require_mutating_the_process_environment() {
        let hosts = candidate_hosts(Some("/home/someone"), Some("/run/user/42"));
        assert!(hosts.contains(&"unix:///home/someone/.colima/default/docker.sock".into()));
        assert!(hosts.contains(&"unix:///run/user/42/podman/podman.sock".into()));
        assert!(hosts.contains(&"unix:///var/run/docker.sock".into()));
        assert!(
            candidate_hosts(None, Some("/run/user/42"))
                .contains(&"unix:///run/user/42/podman/podman.sock".into())
        );
    }

    #[test]
    fn falls_back_to_another_working_client() {
        let mut calls = Vec::new();
        let runtime = select_runtime(None, &[], |program, host| {
            calls.push((program.to_owned(), host.map(str::to_owned)));
            if program == "podman" {
                Probe::Ready
            } else {
                Probe::TimedOut
            }
        })
        .unwrap();
        assert_eq!(runtime.program, "podman");
        assert_eq!(calls, [("docker".into(), None), ("podman".into(), None)]);
    }

    #[test]
    fn only_docker_is_probed_with_a_docker_socket_override() {
        let runtime = select_runtime(None, &["unix:///test.sock".into()], |program, host| {
            if host.is_some() {
                assert_eq!(program, "docker");
                Probe::Ready
            } else {
                Probe::Failed
            }
        })
        .unwrap();
        assert_eq!(runtime.host.as_deref(), Some("unix:///test.sock"));
    }

    #[test]
    fn explicit_endpoints_never_fall_back_to_an_unrelated_runtime_or_socket() {
        for selected in ["docker", "podman", "nerdctl"] {
            let mut calls = 0;
            let error = select_runtime(
                Some(selected),
                &["unix:///test.sock".into()],
                |program, host| {
                    calls += 1;
                    assert_eq!(program, selected);
                    assert_eq!(host, None);
                    Probe::Failed
                },
            )
            .unwrap_err();
            assert_eq!(calls, 1);
            assert_eq!(error.exit_code(), 4);
            assert!(
                error
                    .to_string()
                    .contains("no other runtime or local socket was substituted")
            );
        }
    }

    #[test]
    fn selects_endpoint_variables_without_reading_or_disclosing_their_values() {
        assert_eq!(selected_program(|_| false).unwrap(), None);
        for (program, variables) in SELECTORS {
            for variable in *variables {
                assert_eq!(
                    selected_program(|key| key == *variable).unwrap(),
                    Some(*program)
                );
            }
        }
        assert!(selected_program(|key| ["DOCKER_HOST", "CONTAINER_HOST"].contains(&key)).is_err());
        assert_eq!(
            selected_program(|key| ["DOCKER_HOST", "DOCKER_CONTEXT"].contains(&key)).unwrap(),
            Some("docker")
        );
    }

    #[test]
    fn failure_distinguishes_missing_broken_and_timed_out_clients() {
        let error = select_runtime(None, &[], |program, _| match program {
            "docker" => Probe::Missing,
            "podman" => Probe::Failed,
            _ => Probe::TimedOut,
        })
        .unwrap_err();
        let message = error.to_string();
        assert_eq!(error.exit_code(), 4);
        assert!(message.contains("docker: executable not found"));
        assert!(message.contains("podman: `info` failed"));
        assert!(message.contains("nerdctl: `info` timed out"));
        assert!(message.contains("Portable build/check/serve/view"));
    }

    #[test]
    fn guidance_matches_the_operating_system() {
        assert!(setup_guidance("windows").contains("Linux-container mode"));
        assert!(setup_guidance("macos").contains("colima start"));
        assert!(setup_guidance("linux").contains("rootless Podman"));
        assert!(!setup_guidance("linux").contains("Colima"));
        assert!(!setup_guidance("windows").contains("Colima"));
    }

    // Run this very test executable as a child so timeout coverage also runs on
    // Windows, with no shell, Docker installation, environment mutation or VM.
    #[test]
    fn probe_child() {
        match std::env::var("ZEL_TEST_PROBE").as_deref() {
            Ok("hang") => std::thread::sleep(Duration::from_secs(30)),
            Ok("fail") => std::process::exit(7),
            Ok("verbose") => {
                for _ in 0..10000 {
                    println!("a long version line");
                }
            }
            _ => {}
        }
    }

    fn child(mode: &str) -> Command {
        let mut command = Command::new(std::env::current_exe().unwrap());
        command
            .args(["--exact", "runtime::tests::probe_child", "--nocapture"])
            .env("ZEL_TEST_PROBE", mode);
        command
    }

    #[test]
    fn probes_reap_timed_out_children_and_capture_output_without_pipe_deadlocks() {
        let start = Instant::now();
        assert_eq!(
            probe_status(&mut child("hang"), Duration::from_millis(100)),
            Probe::TimedOut
        );
        assert!(start.elapsed() < Duration::from_secs(5));
        assert_eq!(
            probe_status(&mut child("fail"), Duration::from_secs(5)),
            Probe::Failed
        );
        let output = probe_output(&mut child("verbose"), Duration::from_secs(5)).unwrap();
        assert!(output.contains("a long version line"));
        assert!(output.len() <= 4096);
        assert_eq!(
            probe_output(&mut child("hang"), Duration::from_millis(100)),
            None
        );
    }

    #[test]
    fn the_os_resolves_executables_and_reports_missing_files() {
        let temp = tempfile::tempdir().unwrap();
        let name = if cfg!(windows) {
            "docker.exe"
        } else {
            "docker"
        };
        let program = temp.path().join(name);
        std::fs::copy(std::env::current_exe().unwrap(), &program).unwrap();
        // Omit .exe on Windows, just as real runtime invocations do.
        let mut command = Command::new(temp.path().join("docker"));
        command.args(["--exact", "runtime::tests::probe_child"]);
        assert_eq!(
            probe_status(&mut command, Duration::from_secs(5)),
            Probe::Ready
        );
        assert_eq!(
            probe_status(
                &mut Command::new(temp.path().join("missing")),
                Duration::from_secs(5)
            ),
            Probe::Missing
        );
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&program, std::fs::Permissions::from_mode(0o644)).unwrap();
            assert_eq!(
                probe_status(&mut Command::new(&program), Duration::from_secs(5)),
                Probe::Unavailable
            );
        }
    }
}
