//! The native checker must work without external tools and without an output tree.
use serde_json::Value;
use std::{
    fs,
    path::Path,
    process::{Command, Output},
};

fn run(root: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_zel"))
        .current_dir(root)
        .args(args)
        .env("PATH", root.join("no-external-tools"))
        .output()
        .unwrap()
}
fn report(root: &Path, args: &[&str], code: i32) -> Value {
    let output = run(root, args);
    assert_eq!(
        output.status.code(),
        Some(code),
        "{args:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        output.stderr.is_empty(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("stdout is exactly one JSON report")
}

#[test]
fn plain_folder_checks_without_output_or_external_tools_in_both_formats() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    let source = root.join("Docs 日本");
    fs::create_dir(&source).unwrap();
    fs::write(source.join("README.md"), "# Hello\n").unwrap();
    for args in [
        vec!["check", "Docs 日本", "--json"],
        vec![
            "--json",
            "check",
            "Docs 日本",
            "--target",
            "portable",
            "--strict",
            "--deep",
        ],
    ] {
        let value = report(root, &args, 0);
        assert_eq!(value["schema_version"], 1);
        assert_eq!(value["ok"], true);
        assert_eq!(value["target"], "portable");
        assert_eq!(value["pages"], 1);
        assert_eq!(value["diagnostics"], serde_json::json!([]));
    }
    let human = run(&source, &["check"]);
    assert!(
        human.status.success(),
        "{}",
        String::from_utf8_lossy(&human.stderr)
    );
    assert!(
        String::from_utf8_lossy(&human.stdout).contains("1 HTML pages checked; no files written")
    );
    assert_eq!(fs::read_dir(root).unwrap().count(), 1);
    assert_eq!(fs::read_dir(&source).unwrap().count(), 1);
    assert_eq!(
        fs::read_to_string(source.join("README.md")).unwrap(),
        "# Hello\n"
    );
}

#[test]
fn mermaid_bounds_have_native_diagnostics_but_grammar_is_a_browser_concern() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    fs::write(root.join("zel.yaml"), "markdown: {mermaid: true}").unwrap();
    fs::write(root.join("README.md"), "```mermaid\n\n```\n").unwrap();
    let invalid = report(root, &["check", "--json"], 3);
    assert_eq!(invalid["diagnostics"][0]["code"], "rendering");
    assert_eq!(invalid["diagnostics"][0]["file"], "README.md");
    assert!(
        invalid["diagnostics"][0]["message"]
            .as_str()
            .unwrap()
            .contains("mermaid 1")
    );
    fs::write(root.join("README.md"), "```mermaid\ninvalid grammar\n```\n").unwrap();
    assert_eq!(
        report(root, &["check", "--strict", "--json"], 0)["ok"],
        true
    );
    assert_eq!(fs::read_dir(root).unwrap().count(), 2);
}

#[test]
fn invalid_mindmaps_have_structured_diagnostics_and_recover_without_tools() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    fs::write(
        root.join("README.md"),
        "# Maps\n\n```mindmap\nA: 1\nB: 2\n```\n",
    )
    .unwrap();
    let invalid = report(root, &["check", "--json"], 3);
    assert_eq!(invalid["diagnostics"][0]["code"], "rendering");
    assert_eq!(invalid["diagnostics"][0]["file"], "README.md");
    assert!(
        invalid["diagnostics"][0]["message"]
            .as_str()
            .unwrap()
            .contains("mindmap 1")
    );
    fs::write(
        root.join("README.md"),
        "# Maps\n\n```mindmap\nRoot: [Child]\n```\n",
    )
    .unwrap();
    assert_eq!(
        report(root, &["check", "--strict", "--json"], 0)["ok"],
        true
    );
    assert_eq!(fs::read_dir(root).unwrap().count(), 1);
}

#[test]
fn invalid_math_has_a_structured_source_diagnostic_and_recovers_without_tools() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    fs::write(root.join("zel.yaml"), "markdown: {math: true}").unwrap();
    fs::write(root.join("README.md"), "# Equations\n\n$\\unknown$\n").unwrap();
    let invalid = report(root, &["check", "--json"], 3);
    assert_eq!(invalid["ok"], false);
    assert_eq!(invalid["diagnostics"][0]["code"], "rendering");
    assert_eq!(invalid["diagnostics"][0]["file"], "README.md");
    assert!(
        invalid["diagnostics"][0]["message"]
            .as_str()
            .unwrap()
            .contains("math expression 1")
    );
    fs::write(root.join("README.md"), "# Equations\n\n$\\frac{1}{2}$\n").unwrap();
    assert_eq!(
        report(root, &["check", "--strict", "--json"], 0)["ok"],
        true
    );
    assert_eq!(fs::read_dir(root).unwrap().count(), 2);
}

#[test]
fn strict_and_invalid_checks_have_nonzero_exits_and_valid_json() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    fs::write(root.join("README.md"), "# Home\n\n<span>Literal</span>\n").unwrap();
    assert_eq!(
        report(root, &["check", "--json"], 0)["diagnostics"][0]["severity"],
        "warning"
    );
    let strict = report(root, &["check", "--strict", "--json"], 3);
    assert_eq!(strict["strict"], true);
    assert_eq!(strict["ok"], false);
    assert_eq!(strict["pages"], 1);
    let human = run(root, &["check", "--strict"]);
    assert_eq!(human.status.code(), Some(3));
    assert!(String::from_utf8_lossy(&human.stderr).contains("warnings are rejected by --strict"));
    fs::write(
        root.join("README.md"),
        "# Home\n\n[Missing](absent.md)\n\n![Gone](gone.png)\n",
    )
    .unwrap();
    let invalid = report(root, &["check", "--json"], 3);
    assert_eq!(invalid["diagnostics"].as_array().unwrap().len(), 2);
    assert_eq!(invalid["diagnostics"][0]["file"], "README.md");
    assert_eq!(invalid["diagnostics"][0]["code"], "reference");
    assert!(invalid["pages"].is_null());
    let missing = report(root, &["check", "missing", "--json"], 1);
    assert_eq!(missing["diagnostics"][0]["code"], "source");
    assert_eq!(fs::read_dir(root).unwrap().count(), 1);
}

#[test]
fn check_target_selection_matches_build_and_hosted_json_fails_before_execution() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    fs::write(root.join("README.md"), "# Home\n").unwrap();
    fs::write(root.join("site.yaml"), "name: Hosted\n").unwrap();
    let hosted = run(root, &["check", "--json"]);
    assert_eq!(hosted.status.code(), Some(2));
    assert!(hosted.stdout.is_empty());
    assert!(String::from_utf8_lossy(&hosted.stderr).contains("hosted validation"));
    assert!(
        report(root, &["check", "--target", "portable", "--json"], 0)["ok"]
            .as_bool()
            .unwrap()
    );
    fs::write(root.join("zel.yaml"), "name: Portable\n").unwrap();
    assert_eq!(report(root, &["check", "--json"], 0)["ok"], true);
    assert_eq!(
        run(root, &["check", "--target", "static", "--json"])
            .status
            .code(),
        Some(2)
    );
    fs::write(root.join("zel.yaml"), "typo: value\n").unwrap();
    let invalid = report(root, &["check", "--json"], 3);
    assert_eq!(invalid["diagnostics"][0]["code"], "configuration");
    assert_eq!(invalid["diagnostics"][0]["file"], "zel.yaml");
    assert_eq!(
        run(root, &["check", "--target", "image", "--json"])
            .status
            .code(),
        Some(2)
    );
    assert_eq!(fs::read_dir(root).unwrap().count(), 3);
}
