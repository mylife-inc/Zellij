//! Test native JSON orchestration without a daemon; CI also runs the real engine.
#![cfg(unix)]
use serde_json::{Value, json};
use std::{fs, os::unix::fs::PermissionsExt, path::Path, process::Command};

fn fixture(root: &Path) {
    fs::create_dir(root.join("bin")).unwrap();
    fs::create_dir(root.join("docs 日本 \"quoted\"")).unwrap();
    fs::write(
        root.join("docs 日本 \"quoted\"/site.yaml"),
        "name: Hosted\n",
    )
    .unwrap();
    fs::write(
        root.join("bin/docker"),
        r#"#!/bin/sh
case "$1" in
  info) exit 0;;
  --version) echo 'fixture docker'; exit 0;;
  run)
    printf '%s\n' "$@" > arguments
    cat engine-report
    echo 'internal fixture stderr must not pollute JSON' >&2
    exit "${FAKE_EXIT:-0}";;
  *) exit 99;;
esac
"#,
    )
    .unwrap();
    fs::set_permissions(root.join("bin/docker"), fs::Permissions::from_mode(0o755)).unwrap();
}

fn command(root: &Path) -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_zel"));
    cmd.current_dir(root)
        .env(
            "PATH",
            format!("{}:/usr/bin:/bin", root.join("bin").display()),
        )
        .env("DOCKER_HOST", "unix:///fixture-docker.sock")
        .env("ZELLIJ_IMAGE", "fixture/engine:test");
    for key in [
        "DOCKER_CONTEXT",
        "CONTAINER_HOST",
        "CONTAINER_CONNECTION",
        "CONTAINERD_ADDRESS",
        "CONTAINERD_NAMESPACE",
    ] {
        cmd.env_remove(key);
    }
    cmd
}

fn report(mut cmd: Command, exit: i32) -> Value {
    let output = cmd.output().unwrap();
    assert_eq!(
        output.status.code(),
        Some(exit),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        output.stderr.is_empty(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("exactly one JSON report")
}

fn engine_report() -> Value {
    json!({"schema_version": 1, "target": "static", "root": "/content", "deep": true,
        "strict": true, "ok": true, "pages": 2, "diagnostics": []})
}

#[test]
fn forwards_validation_options_and_preserves_success_errors_and_strict_warnings() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    fixture(root);
    let content = root.join("docs 日本 \"quoted\"").canonicalize().unwrap();
    for (severity, exit) in [(None, 0), (Some("error"), 3), (Some("warning"), 3)] {
        let mut expected = engine_report();
        if let Some(severity) = severity {
            expected["ok"] = json!(false);
            expected["diagnostics"] = json!([{"severity": severity, "code": "deep", "file": null,
                "message": "guide/page.mdx: no page resolves for /content/help"}]);
        }
        fs::write(
            root.join("engine-report"),
            serde_json::to_vec(&expected).unwrap(),
        )
        .unwrap();
        let mut cmd = command(root);
        cmd.args([
            "check",
            "docs 日本 \"quoted\"",
            "--deep",
            "--strict",
            "--json",
            "--quiet",
        ])
        .env("FAKE_EXIT", if exit == 0 { "0" } else { "1" });
        let actual = report(cmd, exit);
        expected["root"] = json!(content.to_str().unwrap());
        assert_eq!(actual, expected);
        let arguments = fs::read_to_string(root.join("arguments")).unwrap();
        assert!(arguments.contains(":/content:ro\n"));
        assert!(arguments.contains("fixture/engine:test\n"));
        assert!(arguments.contains("check /content --deep --strict --json"));
        assert_eq!(
            fs::read_to_string(content.join("site.yaml")).unwrap(),
            "name: Hosted\n"
        );
        assert_eq!(fs::read_dir(&content).unwrap().count(), 1);
    }
}

#[test]
fn incompatible_engine_and_runtime_failures_are_not_content_failures() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    fixture(root);
    for (body, engine_exit) in [
        ("old engine: check passed".to_string(), "0"),
        (engine_report().to_string(), "125"),
        ("{}".to_string(), "1"),
    ] {
        fs::write(root.join("engine-report"), body).unwrap();
        let mut cmd = command(root);
        cmd.args([
            "check",
            "docs 日本 \"quoted\"",
            "--deep",
            "--strict",
            "--json",
        ])
        .env("FAKE_EXIT", engine_exit);
        let actual = report(cmd, 1);
        assert_eq!(actual["ok"], false);
        assert_eq!(actual["pages"], Value::Null);
        assert_eq!(actual["diagnostics"][0]["code"], "engine");
        assert!(
            actual["diagnostics"][0]["message"]
                .as_str()
                .unwrap()
                .contains("ZELLIJ_IMAGE")
        );
        assert!(!actual.to_string().contains("internal fixture stderr"));
    }
    let mut cmd = command(root);
    cmd.args(["check", "docs 日本 \"quoted\"", "--json"])
        .env("PATH", root.join("no-tools"));
    let missing = report(cmd, 4);
    assert_eq!(missing["diagnostics"][0]["code"], "runtime");
    assert_eq!(missing["ok"], false);
    fs::write(
        root.join("zel.yaml"),
        "version: 1\nrenderer: portable\nname: Wrong renderer\n",
    )
    .unwrap();
    let mut cmd = command(root);
    cmd.args(["check", ".", "--target", "static", "--json"]);
    let config = report(cmd, 2);
    assert_eq!(config["diagnostics"][0]["code"], "configuration");
}
