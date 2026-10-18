//! Test target selection and failure before container/filesystem side effects.
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

#[test]
fn plain_markdown_builds_natively_with_default_and_explicit_targets() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    fs::create_dir(root.join("Docs 日本")).unwrap();
    fs::write(root.join("Docs 日本/README.md"), "# Hello\n").unwrap();
    let first = run(root, &["build", "Docs 日本"]);
    assert!(
        first.status.success(),
        "{}",
        String::from_utf8_lossy(&first.stderr)
    );
    assert!(first.stderr.is_empty());
    assert!(root.join("Docs 日本-site/index.html").is_file());
    let explicit = run(
        root,
        &[
            "build",
            "Docs 日本",
            "--target",
            "portable",
            "--out",
            "published",
            "--name",
            "Company docs",
            "--theme",
            "forest",
        ],
    );
    assert!(
        explicit.status.success(),
        "{}",
        String::from_utf8_lossy(&explicit.stderr)
    );
    let html = fs::read_to_string(root.join("published/index.html")).unwrap();
    assert!(html.contains("Company docs"));
    assert!(html.contains("data-zellij-theme=\"forest\""));
    assert_eq!(
        fs::read_to_string(root.join("Docs 日本/README.md")).unwrap(),
        "# Hello\n"
    );
    assert_eq!(fs::read_dir(root.join("Docs 日本")).unwrap().count(), 1);
}

#[test]
fn structured_content_keeps_the_hosted_default_and_can_be_overridden() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    fs::write(root.join("site.yaml"), "name: Docs\n").unwrap();
    fs::write(root.join("README.md"), "# Portable too\n").unwrap();
    // Rejecting portable-only options establishes selection before starting
    // Docker, and does not depend on any runtime installed on the test runner.
    let hosted = run(root, &["build", ".", "--name", "Portable name"]);
    assert_eq!(hosted.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&hosted.stderr).contains("only to portable builds"));
    let output = root.parent().unwrap().join(format!(
        "{}-site",
        root.file_name().unwrap().to_string_lossy()
    ));
    let portable = run(
        root,
        &[
            "build",
            ".",
            "--target",
            "portable",
            "--out",
            output.to_str().unwrap(),
        ],
    );
    assert!(
        portable.status.success(),
        "{}",
        String::from_utf8_lossy(&portable.stderr)
    );
    assert!(output.join("index.html").is_file());
    fs::remove_dir_all(output).unwrap();
}

#[test]
fn invalid_build_options_fail_before_creating_output() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    fs::create_dir(root.join("docs")).unwrap();
    fs::write(root.join("docs/index.md"), "# Docs\n").unwrap();
    for extra in [
        vec!["--target", "unknown"],
        vec!["--target", "portable", "--theme", "unknown"],
        vec!["--target", "portable", "--name", " "],
        vec!["--target", "portable", "--tag", "image"],
        vec!["--target", "image"],
        vec!["--target", "static", "--name", "Docs"],
    ] {
        let mut args = vec!["build", "docs", "--out", "output"];
        args.extend(extra);
        let failure = run(root, &args);
        assert_eq!(
            failure.status.code(),
            Some(2),
            "{args:?}: {}",
            String::from_utf8_lossy(&failure.stderr)
        );
        assert!(!root.join("output").exists());
        assert!(failure.stdout.is_empty());
    }
}

#[test]
fn portable_configuration_selects_native_build_and_flags_override_defaults() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    fs::create_dir(root.join("docs")).unwrap();
    fs::write(root.join("docs/README.md"), "# Home").unwrap();
    fs::write(
        root.join("docs/zel.yaml"),
        "name: Configured\ntheme: forest\n",
    )
    .unwrap();
    // An explicit portable config wins over hosted auto-discovery, while an
    // explicit --target keeps its meaning. No external programs are available.
    fs::write(root.join("docs/site.yaml"), "name: Hosted").unwrap();
    let configured = run(root, &["build", "docs", "--out", "site"]);
    assert!(
        configured.status.success(),
        "{}",
        String::from_utf8_lossy(&configured.stderr)
    );
    let html = fs::read_to_string(root.join("site/index.html")).unwrap();
    assert!(html.contains("Home · Configured"));
    assert!(html.contains("data-zellij-theme=\"forest\""));
    let overridden = run(
        root,
        &[
            "build",
            "docs",
            "--out",
            "site",
            "--name",
            "Override",
            "--theme",
            "cupertino",
        ],
    );
    assert!(
        overridden.status.success(),
        "{}",
        String::from_utf8_lossy(&overridden.stderr)
    );
    let html = fs::read_to_string(root.join("site/index.html")).unwrap();
    assert!(html.contains("Home · Override"));
    assert!(html.contains("data-zellij-theme=\"cupertino\""));
    let explicit = run(
        root,
        &["build", "docs", "--target", "static", "--name", "No"],
    );
    assert!(String::from_utf8_lossy(&explicit.stderr).contains("only to portable builds"));
    fs::remove_file(root.join("docs/zel.yaml")).unwrap();
    fs::create_dir(root.join("docs/zel.yaml")).unwrap();
    let invalid = run(root, &["build", "docs", "--out", "site"]);
    assert!(!invalid.status.success());
    assert!(String::from_utf8_lossy(&invalid.stderr).contains("regular file"));
    assert_eq!(
        fs::read_to_string(root.join("site/index.html")).unwrap(),
        html
    );
}
