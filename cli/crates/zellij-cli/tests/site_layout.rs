//! The site convention must select only the intended sources, retain legacy
//! explicit inputs, and preserve authored files and previous exports on errors.
use serde_json::Value;
use std::{
    fs,
    path::Path,
    process::{Command, Output},
};

fn run(root: &Path, args: &[&str], code: i32) -> Output {
    let result = Command::new(env!("CARGO_BIN_EXE_zel"))
        .current_dir(root)
        .args(args)
        .env("PATH", root.join("no-tools"))
        .env("DOCKER_HOST", "tcp://unreachable.invalid:2376")
        .output()
        .unwrap();
    assert_eq!(result.status.code(), Some(code), "{args:?}: {result:?}");
    result
}

fn write(root: &Path, path: &str, body: &str) {
    let file = root.join(path);
    fs::create_dir_all(file.parent().unwrap()).unwrap();
    fs::write(file, body).unwrap();
}
fn site(root: &Path, folder: &str) {
    write(
        root,
        &format!("{folder}/zel.yaml"),
        "version: 1\nrenderer: portable\nname: Selected site\nnav: [{page: index.md}]\n",
    );
    write(
        root,
        &format!("{folder}/content/index.md"),
        "# Selected page\n",
    );
}

#[test]
fn defaults_explicit_selection_and_config_relative_output_do_not_scan_other_docs() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    site(root, "docs/zellij");
    site(root, "docs/Manual 日本");
    write(
        root,
        "README.md",
        "# Product README\n[Unpublished](missing.md)",
    );
    write(
        root,
        "docs/specs/private.md",
        "# Unpublished spec\n[Broken](missing.md)",
    );
    write(
        root,
        "docs/zellij/notes.md",
        "# Unpublished note\n[Broken](missing.md)",
    );
    let checked = run(root, &["check", "--json", "--strict"], 0);
    assert_eq!(
        serde_json::from_slice::<Value>(&checked.stdout).unwrap()["pages"],
        1
    );
    assert!(!root.join("docs/zellij/site").exists());
    run(root, &["build"], 0);
    let output = root.join("docs/zellij/site/index.html");
    let first = fs::read(&output).unwrap();
    run(&root.join("docs/zellij"), &["build"], 0);
    assert_eq!(fs::read(&output).unwrap(), first);
    assert!(!root.join("docs/zellij/site/notes.html").exists());
    write(
        root,
        "docs/Manual 日本/zel.yaml",
        "version: 1\nrenderer: portable\nname: Custom\nbuild: {out: exports/html}\npreview: {port: 0}\n",
    );
    run(root, &["build", "docs/Manual 日本"], 0);
    assert!(
        root.join("docs/Manual 日本/exports/html/index.html")
            .is_file()
    );
    run(
        root,
        &[
            "build",
            "docs/Manual 日本",
            "--out",
            "override",
            "--name",
            "Flag name",
        ],
        0,
    );
    assert!(
        fs::read_to_string(root.join("override/index.html"))
            .unwrap()
            .contains("Flag name")
    );
    assert_eq!(fs::read(&output).unwrap(), first);
    // A config in cwd wins over docs/zellij; no parent search from content/.
    site(root, ".");
    run(root, &["build"], 0);
    assert!(root.join("site/index.html").is_file());
    run(&root.join("content"), &["build"], 2);
    assert!(root.join("README.md").is_file());
}

#[test]
fn missing_default_requires_an_explicit_legacy_folder() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    write(root, "README.md", "# Explicit legacy\n");
    let error = run(root, &["build"], 2);
    assert!(String::from_utf8_lossy(&error.stderr).contains("docs/zellij/zel.yaml"));
    run(root, &["build", ".", "--out", "."], 2); // output cannot contain source
    run(root, &["check", ".", "--json"], 0);
    site(root, "docs/other");
    run(root, &["check"], 2); // custom names are never guessed
    run(root, &["check", "docs/other"], 0);
}

#[test]
fn invalid_config_or_content_preserves_previous_output_and_reports_json() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    site(root, "manual");
    run(root, &["build", "manual"], 0);
    let old = fs::read(root.join("manual/site/index.html")).unwrap();
    for config in [
        "version: 2\nrenderer: portable",
        "version: '1'\nrenderer: portable",
        "version: 1",
        "version: 1\nrenderer: unknown",
        "renderer: portable",
        "version: 1\nrenderer: portable\ntyop: true",
        "version: 1\nrenderer: portable\nbuild: {out: 123}",
        "version: 1\nrenderer: portable\npreview: {port: 65536}",
        "version: 1\nrenderer: portable\npreview: {port: 0, typo: 1}",
    ] {
        write(root, "manual/zel.yaml", config);
        let result = run(root, &["check", "manual", "--json"], 3);
        assert!(result.stderr.is_empty());
        let report: Value = serde_json::from_slice(&result.stdout).unwrap();
        assert_eq!(report["diagnostics"][0]["code"], "configuration");
        run(root, &["build", "manual"], 2);
        assert_eq!(fs::read(root.join("manual/site/index.html")).unwrap(), old);
    }
    site(root, "manual");
    fs::remove_dir_all(root.join("manual/content")).unwrap();
    write(root, "manual/index.md", "# Must not be published");
    run(root, &["build", "manual"], 2);
    assert_eq!(fs::read(root.join("manual/site/index.html")).unwrap(), old);
}

#[test]
fn configured_output_retains_overlap_and_ownership_protection() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    site(root, "manual");
    write(root, "manual/site/authored.txt", "mine");
    run(root, &["build", "manual"], 2);
    for out in [".", "content", "content/generated", ".."] {
        write(
            root,
            "manual/zel.yaml",
            &format!("version: 1\nrenderer: portable\nbuild: {{out: '{out}'}}"),
        );
        run(root, &["build", "manual"], 2);
    }
    assert_eq!(
        fs::read_to_string(root.join("manual/site/authored.txt")).unwrap(),
        "mine"
    );
    assert!(root.join("manual/content/index.md").is_file());
}

#[test]
fn hosted_roots_route_to_the_engine_and_native_commands_explain_mismatches() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    run(root, &["new", "docs", "docs/manual", "--guide"], 0);
    assert!(root.join("docs/manual/content/site.yaml").is_file());
    assert!(root.join("docs/manual/zel.yaml").is_file());
    let error = run(root, &["check", "docs/manual", "--json"], 2);
    assert!(String::from_utf8_lossy(&error.stderr).contains("hosted validation"));
    run(root, &["image", "dockerfile", "docs/manual"], 0);
    run(root, &["build", "docs/manual", "--target", "portable"], 2);
    run(root, &["view", "docs/manual", "--name", "Portable only"], 2);
    let error = run(root, &["dev", "docs/manual", "--port", "0"], 2);
    assert!(String::from_utf8_lossy(&error.stderr).contains("view --watch --port 0"));
    run(root, &["new", "guide", "api", "--dir", "docs/manual"], 0);
    assert!(root.join("docs/manual/content/02-api/index.mdx").is_file());
    site(root, "docs/zellij");
    run(root, &["new", "page", "pricing"], 2);
    run(root, &["image", "dockerfile"], 2);
    assert!(!root.join("docs/zellij/content/sections").exists());
}

#[cfg(unix)]
#[test]
fn configured_content_and_config_links_are_rejected_before_writes() {
    use std::os::unix::fs::symlink;
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    site(root, "manual");
    fs::rename(root.join("manual/content"), root.join("outside")).unwrap();
    symlink(root.join("outside"), root.join("manual/content")).unwrap();
    run(root, &["build", "manual"], 2);
    fs::remove_file(root.join("manual/content")).unwrap();
    fs::rename(root.join("outside"), root.join("manual/content")).unwrap();
    fs::rename(root.join("manual/zel.yaml"), root.join("settings.yaml")).unwrap();
    symlink(root.join("settings.yaml"), root.join("manual/zel.yaml")).unwrap();
    run(root, &["build", "manual"], 2);
    assert!(!root.join("manual/site").exists());
}

#[test]
fn cwd_and_site_root_control_scaffolding_builds_outputs_and_native_writes() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    fs::create_dir(root.join("Product 日本")).unwrap();
    run(root, &["--cwd", "Product 日本", "init", "--minimal"], 0);
    assert!(!root.join("docs").exists());
    run(
        root,
        &["b", "--cwd", "Product 日本", "--out", "exported"],
        0,
    );
    assert!(root.join("Product 日本/exported/index.html").is_file());
    assert!(!root.join("exported").exists());
    let default = fs::read(root.join("Product 日本/docs/zellij/zel.yaml")).unwrap();
    site(root, "Product 日本/docs/Manual #1%");
    let args = [
        "--cwd",
        "Product 日本",
        "--site-root",
        "docs/Manual #1%",
        "c",
        "--json",
    ];
    let checked = run(root, &args, 0);
    assert_eq!(
        serde_json::from_slice::<Value>(&checked.stdout).unwrap()["pages"],
        1
    );
    run(
        root,
        &[
            "--cwd",
            "Product 日本",
            "features",
            "all",
            "--site-root",
            "docs/Manual #1%",
        ],
        0,
    );
    let config = fs::read_to_string(root.join("Product 日本/docs/Manual #1%/zel.yaml")).unwrap();
    assert_eq!(
        serde_yaml_ng::from_str::<serde_yaml_ng::Value>(&config).unwrap()["markdown"]["math"],
        true
    );
    assert_eq!(
        fs::read(root.join("Product 日本/docs/zellij/zel.yaml")).unwrap(),
        default
    );
    // Saved output is relative to the selected site; a flag is relative to --cwd.
    write(
        root,
        "Product 日本/docs/Manual #1%/zel.yaml",
        "version: 1\nrenderer: portable\nbuild: {out: exports/html}\n",
    );
    run(
        root,
        &[
            "b",
            "--site-root",
            "docs/Manual #1%",
            "--cwd",
            "Product 日本",
        ],
        0,
    );
    assert!(
        root.join("Product 日本/docs/Manual #1%/exports/html/index.html")
            .is_file()
    );
    run(
        root,
        &["--cwd", "Product 日本", "ctx", "context.json", "--json"],
        0,
    );
    assert!(root.join("Product 日本/context.json").is_file());
    assert!(!root.join("context.json").exists());
    run(
        root,
        &["--cwd", "Product 日本", "nd", "docs/hosted", "--minimal"],
        0,
    );
    run(
        root,
        &[
            "--cwd",
            "Product 日本",
            "--site-root",
            "docs/hosted",
            "ng",
            "api",
        ],
        0,
    );
    assert!(
        root.join("Product 日本/docs/hosted/content/01-api/index.mdx")
            .is_file()
    );
    run(
        root,
        &["ir", "--cwd", "Product 日本", "--site-root", "docs/hosted"],
        4,
    );
    run(
        root,
        &["idf", "--cwd", "Product 日本", "--site-root", "docs/hosted"],
        0,
    );
    // An absolute --cwd also uses the same selection and safety paths.
    run(
        root,
        &[
            "--cwd",
            root.join("Product 日本").to_str().unwrap(),
            "b",
            "--site-root",
            "docs/Manual #1%",
            "--out",
            "docs/Manual #1%/content",
        ],
        2,
    );
    assert!(
        root.join("Product 日本/docs/Manual #1%/content/index.md")
            .is_file()
    );
}

#[test]
fn invalid_working_directory_and_selection_fail_before_writes_or_tool_discovery() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    site(root, "docs/zellij");
    let before = fs::read(root.join("docs/zellij/zel.yaml")).unwrap();
    write(root, "ordinary-file", "keep");
    for path in ["missing", "ordinary-file"] {
        let result = run(root, &["--cwd", path, "build"], 2);
        assert!(String::from_utf8_lossy(&result.stderr).contains("cannot use --cwd"));
    }
    for args in [
        vec!["build", "docs/zellij", "--site-root", "docs/zellij"],
        vec!["new", "portable", "new-site", "--site-root", "docs/zellij"],
        vec!["ci", "run", "--site-root", "docs/zellij"],
        vec![
            "eject",
            "--with-docs",
            "--out",
            "wrapper",
            "--site-root",
            "docs/zellij",
        ],
        vec![
            "image",
            "push",
            "example:latest",
            "--site-root",
            "docs/zellij",
        ],
    ] {
        run(root, &args, 2);
    }
    assert_eq!(fs::read(root.join("docs/zellij/zel.yaml")).unwrap(), before);
    for path in ["new-site", "wrapper", "site", "docs/zellij/site"] {
        assert!(!root.join(path).exists());
    }
    // Site selectors retain the command's existing missing-path status.
    let selected = run(root, &["b", "--site-root", "missing"], 1);
    let positional = run(root, &["b", "missing"], 1);
    assert_eq!(selected.stderr, positional.stderr);
    run(root, &["features", "none", "--site-root", "missing"], 1);
    run(root, &["--cwd", "missing", "--help"], 0);
    run(root, &["--cwd", "missing", "--version"], 0);
}

#[test]
fn explicit_config_drives_build_check_and_edits_without_touching_default_config() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    let custom = "Product 日本/docs/manual/preview #1%.yaml";
    site(root, "Product 日本/docs/manual");
    let default = "not valid YAML: [";
    write(root, "Product 日本/docs/manual/zel.yaml", default);
    write(
        root,
        custom,
        "# keep this comment\nversion: 1\nrenderer: portable\nname: Custom configuration\nbuild: {out: preview}\nmarkdown: {math: false}\n",
    );
    run(
        root,
        &[
            "b",
            "--cwd",
            "Product 日本",
            "--config",
            "docs/manual/preview #1%.yaml",
        ],
        0,
    );
    let output = root.join("Product 日本/docs/manual/preview/index.html");
    assert!(
        fs::read_to_string(&output)
            .unwrap()
            .contains("Custom configuration")
    );
    let checked = run(root, &["--config", custom, "c", "--json", "--strict"], 0);
    assert_eq!(
        serde_json::from_slice::<Value>(&checked.stdout).unwrap()["pages"],
        1
    );
    run(root, &["features", "all", "--config", custom], 0);
    let settings = fs::read_to_string(root.join(custom)).unwrap();
    assert!(settings.contains("# keep this comment"));
    assert_eq!(
        serde_yaml_ng::from_str::<serde_yaml_ng::Value>(&settings).unwrap()["markdown"]["math"],
        true
    );
    run(root, &["fmt", "--config", custom], 0);
    run(root, &["fmt", "--check", "--config", custom], 0);
    assert_eq!(
        fs::read_to_string(root.join("Product 日本/docs/manual/zel.yaml")).unwrap(),
        default
    );
    let previous = fs::read(&output).unwrap();
    write(root, custom, "version: [");
    let error = run(root, &["c", "--config", custom, "--json"], 3);
    let report: Value = serde_json::from_slice(&error.stdout).unwrap();
    assert!(
        report["diagnostics"]
            .as_array()
            .unwrap()
            .iter()
            .any(|d| d["file"] == "preview #1%.yaml")
    );
    assert!(error.stderr.is_empty());
    run(root, &["b", "--config", custom], 2);
    fs::remove_file(root.join(custom)).unwrap();
    run(root, &["b", "--config", custom], 2);
    assert_eq!(fs::read(output).unwrap(), previous);
    assert!(!root.join("Product 日本/docs/manual/site").exists());
}

#[test]
fn explicit_hosted_and_legacy_configs_keep_their_content_and_output_bases() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    run(root, &["nd", "hosted", "--minimal"], 0);
    fs::rename(
        root.join("hosted/zel.yaml"),
        root.join("hosted/alternate.yaml"),
    )
    .unwrap();
    write(root, "hosted/zel.yaml", "invalid: [");
    run(root, &["ng", "api", "--config", "hosted/alternate.yaml"], 0);
    assert!(root.join("hosted/content/01-api/index.mdx").is_file());
    run(root, &["idf", "--config", "hosted/alternate.yaml"], 0);
    run(root, &["ir", "--config", "hosted/alternate.yaml"], 4);
    write(root, "raw/settings.yaml", "name: Legacy selected\n");
    write(root, "raw/zel.yaml", "invalid: [");
    write(root, "raw/index.md", "# Legacy page\n");
    run(root, &["b", "--config", "raw/settings.yaml"], 0);
    assert!(
        fs::read_to_string(root.join("raw-site/index.html"))
            .unwrap()
            .contains("Legacy selected")
    );
    run(
        root,
        &["b", "--config", "raw/settings.yaml", "--out", "raw"],
        2,
    );
    assert!(root.join("raw/index.md").is_file());
}

#[test]
fn explicit_config_rejects_directories_links_and_conflicts_before_writes() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    site(root, "docs/zellij");
    for args in [
        vec!["b", "--config", "docs/zellij"],
        vec!["b", "docs/zellij", "--config", "docs/zellij/zel.yaml"],
        vec![
            "--config",
            "missing.yaml",
            "b",
            "--site-root",
            "docs/zellij",
        ],
        vec!["init", "new-site", "--config", "docs/zellij/zel.yaml"],
        vec!["pub", "--config", "missing.yaml"],
    ] {
        run(root, &args, 2);
    }
    assert!(!root.join("docs/zellij/site").exists());
    assert!(!root.join("new-site").exists());
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(root.join("docs/zellij/zel.yaml"), root.join("linked.yaml"))
            .unwrap();
        run(root, &["features", "none", "--config", "linked.yaml"], 2);
        run(root, &["b", "--config", "linked.yaml"], 2);
    }
}
