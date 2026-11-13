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

fn doctor_report(root: &Path, args: &[&str], code: i32) -> serde_json::Value {
    let output = command(root).args(args).arg("--json").output().unwrap();
    assert_eq!(output.status.code(), Some(code), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["schema_version"], 1);
    assert_eq!(report["exit_code"], code);
    assert_eq!(report["ok"], code == 0);
    report
}

#[test]
fn portable_doctor_discovers_site_without_runtime_or_filesystem_writes() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    let site = root.join("docs/zellij");
    fs::create_dir_all(site.join("content")).unwrap();
    let config =
        "version: 1\nrenderer: portable\nbuild:\n  out: generated/site\npreview:\n  port: 4321\n";
    fs::write(site.join("zel.yaml"), config).unwrap();
    // Doctor checks settings, not content references. check remains necessary.
    fs::write(site.join("content/index.md"), "[Broken](missing.md)").unwrap();
    #[cfg(unix)]
    runtime(root, "docker", "exit 99");
    let output = command(root)
        .args(["doctor", "--quiet", "--json"])
        .env("DOCKER_HOST", "ssh://secret-endpoint")
        .env("CONTAINER_HOST", "ssh://conflicting-secret")
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    assert!(output.stderr.is_empty());
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["site"]["renderer"], "portable");
    assert_eq!(report["site"]["layout"], "versioned");
    assert_eq!(report["site"]["preview_port"], 4321);
    let canonical = site.canonicalize().unwrap();
    assert_eq!(
        Path::new(report["site"]["root"].as_str().unwrap()),
        canonical
    );
    assert_eq!(
        Path::new(report["site"]["content"].as_str().unwrap()),
        canonical.join("content")
    );
    assert_eq!(
        Path::new(report["site"]["output"].as_str().unwrap()),
        canonical.join("generated/site")
    );
    assert_eq!(report["runtime"]["status"], "not_required");
    assert!(report["image"].is_null());
    assert!(!String::from_utf8_lossy(&output.stdout).contains("secret"));
    assert!(!root.join("docker-calls").exists());
    assert!(!site.join("generated").exists());
    assert_eq!(fs::read_to_string(site.join("zel.yaml")).unwrap(), config);
    assert_eq!(fs::read_dir(&site).unwrap().count(), 2);
    let human = command(root).args(["doctor", "--quiet"]).output().unwrap();
    assert!(human.status.success());
    assert!(
        String::from_utf8_lossy(&human.stdout)
            .contains("not required for portable web-only sites; not probed")
    );
}

#[test]
fn doctor_honors_paths_custom_config_and_working_directory() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    fs::create_dir_all(root.join("Manual 日本/content")).unwrap();
    fs::write(
        root.join("Manual 日本/custom.yaml"),
        "version: 1\nrenderer: portable\n",
    )
    .unwrap();
    // Explicit selection must ignore even an invalid conventional configuration.
    fs::write(root.join("Manual 日本/zel.yaml"), "invalid: [").unwrap();
    let first = doctor_report(root, &["doctor", "--config", "Manual 日本/custom.yaml"], 0);
    let second = doctor_report(
        root,
        &["--cwd", "Manual 日本", "--config", "custom.yaml", "doctor"],
        0,
    );
    assert_eq!(first, second);
    fs::remove_file(root.join("Manual 日本/zel.yaml")).unwrap();
    fs::rename(
        root.join("Manual 日本/custom.yaml"),
        root.join("Manual 日本/zel.yaml"),
    )
    .unwrap();
    assert_eq!(
        doctor_report(root, &["doctor", "Manual 日本"], 0),
        doctor_report(root, &["doctor", "--site-root", "Manual 日本"], 0)
    );
}

#[test]
fn doctor_supports_raw_markdown_and_legacy_portable_config() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    fs::create_dir(root.join("pages")).unwrap();
    fs::write(root.join("pages/index.md"), "# Example").unwrap();
    let raw = doctor_report(root, &["doctor", "pages"], 0);
    assert!(raw["site"]["config"].is_null());
    assert_eq!(raw["site"]["layout"], "legacy");
    assert_eq!(raw["site"]["renderer"], "portable");
    assert!(Path::new(raw["site"]["output"].as_str().unwrap()).ends_with("pages-site"));
    fs::write(root.join("pages/zel.yaml"), "name: Example\ntheme: slate\n").unwrap();
    let configured = doctor_report(root, &["doctor", "pages"], 0);
    assert!(!configured["site"]["config"].is_null());
    assert_eq!(configured["site"]["layout"], "legacy");
    assert_eq!(configured["runtime"]["status"], "not_required");
}

#[test]
fn doctor_configuration_errors_stop_before_runtime_discovery() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    fs::create_dir(root.join("content")).unwrap();
    for config in [
        "version: 1\nrenderer: invalid\n",
        "version: 1\nrenderer: portable\ntheme: invalid\n",
        "version: 1\nrenderer: portable\nunknown: value\n",
        "version: 1\nrenderer: hosted\nunknown: value\n",
        "invalid: [",
    ] {
        fs::write(root.join("zel.yaml"), config).unwrap();
        let report = doctor_report(root, &["doctor"], 2);
        assert_eq!(report["runtime"]["status"], "not_checked");
        assert_eq!(report["error"]["kind"], "configuration");
    }
    for args in [
        vec!["doctor", "missing"],
        vec!["doctor", "--config", "missing.yaml"],
    ] {
        let report = doctor_report(root, &args, 2);
        assert_eq!(report["runtime"]["status"], "not_checked");
    }
    // A bad cwd config takes precedence over a valid conventional nested site.
    fs::create_dir_all(root.join("docs/zellij/content")).unwrap();
    fs::write(
        root.join("docs/zellij/zel.yaml"),
        "version: 1\nrenderer: portable\n",
    )
    .unwrap();
    doctor_report(root, &["doctor"], 2);
    assert!(!root.join("site").exists());
}

#[test]
fn doctor_reports_hosted_site_even_when_runtime_is_unavailable() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    content(root);
    let legacy = doctor_report(root, &["doctor"], 4);
    assert_eq!(legacy["site"]["renderer"], "hosted");
    assert_eq!(legacy["site"]["layout"], "legacy");
    assert!(Path::new(legacy["site"]["content"].as_str().unwrap()).ends_with("content"));
    assert!(Path::new(legacy["site"]["output"].as_str().unwrap()).ends_with("dist"));
    fs::write(root.join("zel.yaml"), "version: 1\nrenderer: hosted\n").unwrap();
    let versioned = doctor_report(root, &["doctor"], 4);
    assert_eq!(versioned["site"]["layout"], "versioned");
    assert_eq!(versioned["runtime"]["status"], "unavailable");
    assert_eq!(versioned["error"]["kind"], "runtime");
    assert!(Path::new(versioned["site"]["output"].as_str().unwrap()).ends_with("site"));
}

#[test]
fn doctor_outside_site_retains_runtime_diagnostics_without_inventing_a_site() {
    let temp = tempfile::tempdir().unwrap();
    fs::write(temp.path().join("README.md"), "# Product").unwrap();
    let report = doctor_report(temp.path(), &["doctor"], 4);
    assert!(report["site"].is_null());
    assert_eq!(report["runtime"]["status"], "unavailable");
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
        vec!["watch", "content"],
        vec!["dev", "content"],
        vec!["dev", "content", "--features", "all"],
        vec!["view", "content", "--no-open"],
        vec!["view", "content", "--no-open", "--features", "none"],
        vec!["v", "content", "--watch", "--no-open"],
        vec!["image", "build", "content"],
        vec!["image", "static", "content"],
        vec!["si", "content"],
        vec!["ni", "content"],
        vec!["image", "next", "content"],
        vec!["image", "run", "test:latest"],
        vec!["image", "push", "test:latest"],
        vec!["b", "content", "--target", "static", "--out", "site"],
        vec!["c", "content"],
        vec!["ir", "test:latest"],
        vec!["ip", "test:latest"],
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
    let previous = fs::read(root.join("site/index.html")).unwrap();
    for kind in ["si", "ni"] {
        let output = command(root).args([kind, "pages"]).output().unwrap();
        assert_eq!(output.status.code(), Some(4));
    }
    assert_eq!(fs::read(root.join("site/index.html")).unwrap(), previous);
    assert_eq!(fs::read_dir(root.join("pages")).unwrap().count(), 1);
}

#[test]
fn static_recipes_select_renderer_without_probing_or_writing() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    content(root);
    fs::create_dir(root.join("pages")).unwrap();
    fs::write(root.join("pages/index.md"), "# Native\n").unwrap();
    for (dir, hosted) in [("content", true), ("pages", false)] {
        let output = command(root)
            .args(["idf", dir, "--kind", "static"])
            .env("DOCKER_HOST", "tcp://unreachable.invalid:2376")
            .env("ZELLIJ_IMAGE", "example.test/base:test")
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let body = String::from_utf8(output.stdout).unwrap();
        assert!(body.contains("nginxinc/nginx-unprivileged:stable-alpine"));
        assert_eq!(
            body.contains("FROM example.test/base:test AS build"),
            hosted
        );
        assert_eq!(body.contains("npm run build"), hosted);
        assert!(!root.join("pages-site").exists());
    }
    let legacy = command(root).args(["idf", "content"]).output().unwrap();
    let explicit = command(root)
        .args(["idf", "content", "--kind", "next"])
        .output()
        .unwrap();
    assert_eq!(legacy.stdout, explicit.stdout);
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
        format!(
            "info\n--version\nimage inspect ghcr.io/mylife-inc/zellij-base:{}\n",
            env!("CARGO_PKG_VERSION")
        )
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
    let output = command(root)
        .args(["doctor", "--json"])
        .env(
            "DOCKER_HOST",
            "ssh://username:password-do-not-print@example.invalid",
        )
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(4));
    assert!(output.stderr.is_empty());
    assert!(!String::from_utf8_lossy(&output.stdout).contains("password-do-not-print"));
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["error"]["kind"], "runtime");
    assert!(!root.join("podman-calls").exists());
}

#[cfg(unix)]
#[test]
fn hosted_doctor_json_reports_local_image_and_redacts_selected_endpoint() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    content(root);
    runtime(
        root,
        "docker",
        "case \"$1\" in info|image) exit 0;; --version) echo 'docker fixture';; *) exit 99;; esac",
    );
    let output = command(root)
        .args(["--json", "doctor"])
        .env("DOCKER_HOST", "ssh://username:secret@example.invalid")
        .env("ZELLIJ_IMAGE", "registry.example/base:pinned")
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    assert!(output.stderr.is_empty());
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["runtime"]["status"], "available");
    assert_eq!(report["runtime"]["endpoint"], "DOCKER_HOST (value hidden)");
    assert_eq!(report["image"]["reference"], "registry.example/base:pinned");
    assert_eq!(report["image"]["status"], "local");
    assert!(!String::from_utf8_lossy(&output.stdout).contains("secret"));
    assert_eq!(
        fs::read_to_string(root.join("docker-calls")).unwrap(),
        "info\n--version\nimage inspect registry.example/base:pinned\n"
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

#[cfg(unix)]
#[test]
fn an_explicit_runtime_can_respond_after_the_automatic_probe_deadline() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    runtime(
        root,
        "docker",
        "case \"$1\" in info) /bin/sleep 4;; --version) echo 'docker slow fixture';; image) exit 1;; *) exit 99;; esac",
    );
    runtime(root, "podman", "exit 0");
    let start = std::time::Instant::now();
    let output = command(root)
        .arg("doctor")
        .env("DOCKER_CONTEXT", "slow-fixture")
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    assert!(start.elapsed() < std::time::Duration::from_secs(15));
    assert!(String::from_utf8_lossy(&output.stdout).contains("docker slow fixture"));
    assert!(!root.join("podman-calls").exists());
}
