//! Cached report orchestration; native-package CI also runs the real validator.
#![cfg(unix)]
use serde_json::{Value, json};
use std::{fs, path::Path, process::Command};
#[path = "support/cached_renderer.rs"]
mod cached_renderer;

fn fixture(root: &Path) {
    fs::create_dir(root.join("docs 日本 \"quoted\"")).unwrap();
    fs::write(
        root.join("docs 日本 \"quoted\"/site.yaml"),
        "name: Hosted\n",
    )
    .unwrap();
    cached_renderer::install(root);
}

fn command(root: &Path) -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_zel"));
    cmd.current_dir(root)
        .env("PATH", root.join("no-tools"))
        .env("DOCKER_HOST", "unix:///fixture-docker.sock")
        .env("CONTAINER_HOST", "unix:///fixture-podman.sock");
    cached_renderer::configure(&mut cmd, root);
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

fn engine_report(root: &Path) -> Value {
    let content = root.join("docs 日本 \"quoted\"").canonicalize().unwrap();
    json!({"schema_version": 1, "target": "static", "root": content.to_str().unwrap(), "deep": true,
        "strict": true, "ok": true, "pages": 2, "diagnostics": []})
}

#[test]
fn forwards_validation_options_and_preserves_success_errors_and_strict_warnings() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    fixture(root);
    let content = root.join("docs 日本 \"quoted\"").canonicalize().unwrap();
    for (severity, exit) in [(None, 0), (Some("error"), 3), (Some("warning"), 3)] {
        let mut expected = engine_report(root);
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
            "--target",
            "static",
            "docs 日本 \"quoted\"",
            "--deep",
            "--strict",
            "--json",
            "--offline",
            "--quiet",
        ])
        .env("FAKE_EXIT", if exit == 0 { "0" } else { "1" });
        let actual = report(cmd, exit);
        expected["root"] = json!(content.to_str().unwrap());
        assert_eq!(actual, expected);
        let request: Value =
            serde_json::from_slice(&fs::read(root.join("request")).unwrap()).unwrap();
        assert_eq!(request["operation"], "check");
        assert_eq!(request["content"], content.to_str().unwrap());
        assert_eq!(request["deep"], true);
        assert_eq!(request["strict"], true);
        assert_eq!(
            fs::read_to_string(content.join("site.yaml")).unwrap(),
            "name: Hosted\n"
        );
        assert_eq!(fs::read_dir(&content).unwrap().count(), 1);
    }
}

#[test]
fn incompatible_cached_reports_and_missing_cache_are_not_content_failures() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    fixture(root);
    for (body, engine_exit) in [
        ("old engine: check passed".to_string(), "0"),
        (engine_report(root).to_string(), "125"),
        ("{}".to_string(), "1"),
    ] {
        fs::write(root.join("engine-report"), body).unwrap();
        let mut cmd = command(root);
        cmd.args([
            "check",
            "--target",
            "static",
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
                .contains("cached renderer")
        );
        assert!(!actual.to_string().contains("internal fixture stderr"));
    }
    let mut cmd = command(root);
    cmd.args([
        "check",
        "--target",
        "static",
        "docs 日本 \"quoted\"",
        "--json",
        "--offline",
    ])
    .env("ZELLIJ_HOME", root.join("uncached"));
    let missing = report(cmd, 2);
    assert_eq!(missing["target"], "static");
    assert!(
        missing["diagnostics"][0]["message"]
            .as_str()
            .unwrap()
            .contains("not cached")
    );
    assert_eq!(missing["ok"], false);
    fs::write(
        root.join("zel.yaml"),
        "version: 1\nrenderer: portable\nname: Wrong renderer\n",
    )
    .unwrap();
    let mut cmd = command(root);
    cmd.args(["check", "--target", "static", ".", "--json"]);
    let config = report(cmd, 2);
    assert_eq!(config["diagnostics"][0]["code"], "configuration");
}
