//! Exercise real command dispatch with an isolated PATH. No installed runtime,
//! real daemon, registry, or network connection participates in these tests.
use std::{fs, path::Path, process::Command};

fn command(root: &Path) -> Command {
    #[cfg(not(windows))]
    let executable = Path::new(env!("CARGO_BIN_EXE_zel")).to_path_buf();
    #[cfg(windows)]
    let executable = {
        // Windows also searches the executable and system directories before
        // the inherited PATH. Put non-executable placeholders beside a private
        // copy of zel so a runner's real Docker can never enter this fixture.
        // https://doc.rust-lang.org/std/process/struct.Command.html#platform-specific-behavior
        let bin = root.join("bin");
        fs::create_dir_all(&bin).unwrap();
        let executable = bin.join("zel.exe");
        if !executable.exists() {
            fs::copy(env!("CARGO_BIN_EXE_zel"), &executable).unwrap();
            for program in ["docker.exe", "podman.exe", "nerdctl.exe"] {
                fs::write(bin.join(program), b"not an executable").unwrap();
            }
        }
        executable
    };
    let mut command = Command::new(executable);
    command.current_dir(root).env("PATH", root.join("bin"));
    for variable in [
        "DOCKER_CONTEXT",
        "DOCKER_HOST",
        "CONTAINER_CONNECTION",
        "CONTAINER_HOST",
        "CONTAINERD_ADDRESS",
        "CONTAINERD_NAMESPACE",
        "ZELLIJ_IMAGE",
    ] {
        command.env_remove(variable);
    }
    command
}

fn content(root: &Path) {
    fs::create_dir(root.join("content")).unwrap();
    fs::write(root.join("content/site.yaml"), "name: Test\n").unwrap();
    fs::write(root.join("content/index.md"), "# Test\n").unwrap();
}

#[test]
fn container_commands_fail_with_exit_four_before_touching_content_or_output() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    content(root);
    fs::create_dir(root.join("site")).unwrap();
    fs::write(root.join("site/index.html"), "previous export").unwrap();
    for args in [
        vec!["build", "content", "--target", "static", "--out", "site"],
        vec!["build", "content", "--target", "image"],
        vec!["check", "content"],
        vec!["dev", "content"],
        vec!["image", "build", "content"],
        vec!["image", "run", "test:latest"],
        vec!["image", "push", "test:latest"],
        vec!["doctor"],
    ] {
        let output = command(root).args(&args).output().unwrap();
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert_eq!(output.status.code(), Some(4), "{args:?}: {stderr}");
        let expected = if cfg!(windows) {
            "docker: could not execute the client"
        } else {
            "docker: executable not found"
        };
        assert!(stderr.contains(expected), "{args:?}: {stderr}");
        assert!(
            stderr.contains("Portable build/check/serve/view"),
            "{stderr}"
        );
        assert!(!String::from_utf8_lossy(&output.stdout).contains(expected));
        assert_eq!(
            fs::read_to_string(root.join("site/index.html")).unwrap(),
            "previous export"
        );
        assert_eq!(
            fs::read_to_string(root.join("content/site.yaml")).unwrap(),
            "name: Test\n"
        );
        assert_eq!(fs::read_dir(root.join("content")).unwrap().count(), 2);
    }
}

#[test]
fn dockerfile_generation_requires_content_but_no_runtime_and_honors_image_override() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    content(root);
    let output = command(root)
        .args(["image", "dockerfile", "content"])
        // Even an unusable explicit endpoint must not be probed by generation.
        .env("DOCKER_HOST", "tcp://unreachable.invalid:2376")
        .env("ZELLIJ_IMAGE", "registry.example/zellij-base:test")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stderr.is_empty());
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .contains("FROM registry.example/zellij-base:test AS build")
    );
    assert_eq!(
        fs::read_dir(root)
            .unwrap()
            .filter(|entry| entry.as_ref().unwrap().file_name() != "bin")
            .count(),
        1
    );
    let missing = command(root)
        .args(["image", "dockerfile", "absent"])
        .output()
        .unwrap();
    assert!(!missing.status.success());
    assert!(missing.stdout.is_empty());
    assert!(String::from_utf8_lossy(&missing.stderr).contains("no Zellij content folder"));
}

#[test]
fn portable_build_and_check_ignore_even_conflicting_runtime_settings() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    fs::create_dir(root.join("pages")).unwrap();
    fs::write(root.join("pages/index.md"), "# No Docker needed\n").unwrap();
    for args in [
        vec!["check", "pages", "--strict"],
        vec!["build", "pages", "--out", "site"],
    ] {
        let output = command(root)
            .args(args)
            .env("DOCKER_HOST", "tcp://unreachable.invalid:2376")
            .env("CONTAINER_HOST", "ssh://unreachable.invalid")
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(output.stderr.is_empty());
    }
    assert!(root.join("site/index.html").is_file());
}

#[cfg(unix)]
fn runtime(root: &Path, name: &str, body: &str) {
    use std::os::unix::fs::PermissionsExt;
    fs::create_dir_all(root.join("bin")).unwrap();
    let path = root.join("bin").join(name);
    fs::write(
        &path,
        format!("#!/bin/sh\nprintf '%s\\n' \"$*\" >> '{name}-calls'\n{body}\n"),
    )
    .unwrap();
    fs::set_permissions(path, fs::Permissions::from_mode(0o755)).unwrap();
}

#[cfg(unix)]
#[test]
fn doctor_uses_a_working_alternative_without_starting_services_or_pulling_images() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    runtime(root, "docker", "exit 1");
    runtime(
        root,
        "podman",
        "case \"$1\" in info) exit 0;; --version) echo 'podman test';; image) exit 1;; *) exit 99;; esac",
    );
    let output = command(root).arg("doctor").output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("podman test"), "{stdout}");
    assert!(stdout.contains("not pulled"), "{stdout}");
    assert_eq!(
        fs::read_to_string(root.join("docker-calls")).unwrap(),
        "info\n"
    );
    assert_eq!(
        fs::read_to_string(root.join("podman-calls")).unwrap(),
        "info\n--version\nimage inspect ghcr.io/mylife-inc/zellij-base:latest\n"
    );
}

#[cfg(unix)]
#[test]
fn configured_image_run_uses_site_identity_and_explicit_tag_needs_no_site() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    let site = root.join("docs/zellij");
    fs::create_dir_all(site.join("content")).unwrap();
    fs::write(site.join("zel.yaml"), "version: 1\nrenderer: hosted\n").unwrap();
    runtime(root, "docker", "exit 0");
    let output = command(root).args(["image", "run"]).output().unwrap();
    assert!(output.status.success(), "{output:?}");
    assert!(
        fs::read_to_string(root.join("docker-calls"))
            .unwrap()
            .contains("zellij:latest")
    );
    fs::remove_dir_all(root.join("docs")).unwrap();
    let output = command(root)
        .args(["image", "run", "custom:v2"])
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    assert!(
        fs::read_to_string(root.join("docker-calls"))
            .unwrap()
            .contains("custom:v2")
    );
}

#[cfg(unix)]
#[test]
fn explicit_broken_endpoint_is_not_replaced_and_client_errors_do_not_leak_secrets() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    runtime(root, "docker", "echo 'password-do-not-print' >&2; exit 1");
    runtime(root, "podman", "exit 0");
    let output = command(root)
        .arg("doctor")
        .env(
            "DOCKER_HOST",
            "ssh://username:password-do-not-print@example.invalid",
        )
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(4));
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stdout.contains("password-do-not-print"));
    assert!(!stderr.contains("password-do-not-print"));
    assert!(stderr.contains("no other runtime or local socket was substituted"));
    assert!(!root.join("podman-calls").exists());
    assert_eq!(
        fs::read_to_string(root.join("docker-calls")).unwrap(),
        "info\n"
    );
}

#[cfg(unix)]
#[test]
fn doctor_bounds_metadata_probes_and_reports_unknown_instead_of_missing_image() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    runtime(
        root,
        "docker",
        "case \"$1\" in info) exit 0;; *) exec /bin/sleep 30;; esac",
    );
    let start = std::time::Instant::now();
    let output = command(root).arg("doctor").output().unwrap();
    assert!(start.elapsed() < std::time::Duration::from_secs(15));
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("version unavailable"), "{stdout}");
    assert!(
        stdout.contains("unknown — image inspection unavailable or timed out"),
        "{stdout}"
    );
    assert!(!stdout.contains("not pulled"), "{stdout}");
}
