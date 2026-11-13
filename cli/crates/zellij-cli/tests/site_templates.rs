use std::{fs, path::Path, process::Command};
fn run(root: &Path, args: &[&str], code: i32) -> std::process::Output {
    let out = Command::new(env!("CARGO_BIN_EXE_zel"))
        .current_dir(root)
        .args(args)
        .env("PATH", root.join("no-tools"))
        // Windows executable lookup can still reach installed runtimes despite
        // a replaced child PATH. Conflicting selectors stop discovery before
        // any runtime probe, just as in the native PDF integration fixture.
        .env("DOCKER_HOST", "unix:///fixture.sock")
        .env("CONTAINER_HOST", "unix:///fixture-other.sock")
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(code), "{args:?}: {out:?}");
    out
}
#[test]
fn real_cli_catalog_aliases_pinned_templates_and_native_builds_work_offline() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    let listed = run(root, &["templates", "--json", "--offline"], 0);
    let data: serde_json::Value = serde_json::from_slice(&listed.stdout).unwrap();
    assert_eq!(data["templates"].as_array().unwrap().len(), 3);
    for (id, portable) in [
        ("product-docs", true),
        ("book-classic", true),
        ("product-studio", false),
    ] {
        run(root, &["templates", "show", id, "--json"], 0);
        let version = data["templates"]
            .as_array()
            .unwrap()
            .iter()
            .find(|m| m["id"] == id)
            .unwrap()["version"]
            .as_str()
            .unwrap();
        for command in [
            vec!["create", id, "-t", id, "--offline"],
            vec![
                "new",
                "site",
                &format!("{id}-canonical"),
                "--template",
                &format!("{id}@{version}"),
                "--offline",
            ],
        ] {
            run(root, &command, 0);
        }
        assert_eq!(
            fs::read(root.join(id).join(".zel-template.json")).unwrap(),
            fs::read(
                root.join(format!("{id}-canonical"))
                    .join(".zel-template.json")
            )
            .unwrap()
        );
        if portable {
            run(root, &["build", id], 0);
            let html = fs::read_to_string(root.join(id).join("site/index.html")).unwrap();
            assert!(html.contains("<!DOCTYPE html>") || html.contains("<!doctype html>"));
            run(root, &["check", id, "--strict"], 0);
        }
    }
    run(root, &["create", "--offline"], 0);
    assert!(root.join("docs/zellij/zel.yaml").is_file());
}
#[test]
fn real_cli_refuses_unknown_versions_variables_and_nonempty_outputs() {
    let t = tempfile::tempdir().unwrap();
    let root = t.path();
    for args in [
        vec!["create", "out", "-t", "product-docs@999.0.0"],
        vec!["create", "out", "--set", "unknown=value"],
        vec!["create", "out", "--name", "Name", "--set", "title=Other"],
        vec!["create", "out", "--json"],
        vec!["create", "out", "--site-root", "elsewhere"],
    ] {
        run(root, &args, 2);
        assert!(!root.join("out").exists());
    }
    fs::create_dir(root.join("out")).unwrap();
    fs::write(root.join("out/.keep"), "authored").unwrap();
    run(root, &["create", "out"], 2);
    assert_eq!(
        fs::read_to_string(root.join("out/.keep")).unwrap(),
        "authored"
    );
}
#[test]
fn local_template_is_editable_and_has_no_network_or_engine_dependency() {
    let t = tempfile::tempdir().unwrap();
    let root = t.path();
    let source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../templates/book-classic")
        .canonicalize()
        .unwrap();
    run(
        root,
        &[
            "create",
            "manuscript",
            "-t",
            source.to_str().unwrap(),
            "--name",
            "Café: 日本",
            "--set",
            "author=A. Writer",
            "--offline",
        ],
        0,
    );
    let config: serde_yaml_ng::Value =
        serde_yaml_ng::from_slice(&fs::read(root.join("manuscript/zel.yaml")).unwrap()).unwrap();
    assert_eq!(config["name"].as_str(), Some("Café: 日本"));
    assert_eq!(config["book"]["title"].as_str(), Some("Café: 日本"));
    assert_eq!(config["book"]["authors"][0].as_str(), Some("A. Writer"));
    run(root, &["build", "manuscript"], 0);
}

#[test]
fn book_starter_has_shared_publication_and_opt_in_print_without_tools() {
    let t = tempfile::tempdir().unwrap();
    let root = t.path();
    let shown = run(root, &["templates", "show", "book-classic"], 0);
    assert!(String::from_utf8_lossy(&shown.stdout).contains("optional LuaLaTeX"));
    run(
        root,
        &[
            "create",
            "book",
            "-t",
            "book-classic@0.2.0",
            "--name",
            "A writer's journey",
            "--set",
            "author=J. Reader",
            "--offline",
        ],
        0,
    );
    run(root, &["pdf", "book", "--check"], 0);
    assert!(!root.join("book/pdf").exists());
    run(root, &["build", "book"], 0);
    let record: serde_json::Value =
        serde_json::from_slice(&fs::read(root.join("book/site/_zellij/book.json")).unwrap())
            .unwrap();
    assert_eq!(record["manifest"]["title"], "A writer's journey");
    assert_eq!(record["manifest"]["authors"][0], "J. Reader");
    assert_eq!(
        record["entries"]
            .as_array()
            .unwrap()
            .iter()
            .map(|e| e["id"].as_str().unwrap())
            .collect::<Vec<_>>(),
        ["title-page", "preface", "beginning", "path", "notes"]
    );
    assert!(!root.join("book/site/_zellij/pdf").exists());
    assert!(root.join("book/site/assets/wave.png").is_file());
    let home = fs::read(root.join("book/site/index.html")).unwrap();
    let failure = run(root, &["build", "book", "--pdf"], 4);
    let diagnostic = String::from_utf8_lossy(&failure.stderr);
    assert!(
        diagnostic.contains("PDF requires a running"),
        "{diagnostic}"
    );
    assert!(
        diagnostic.contains("select multiple runtimes"),
        "{diagnostic}"
    );
    assert_eq!(fs::read(root.join("book/site/index.html")).unwrap(), home);
    run(root, &["build", "book", "--no-pdf"], 0);
}

#[test]
fn current_directory_is_not_replaced_or_detached() {
    let t = tempfile::tempdir().unwrap();
    let output = run(t.path(), &["create", ".", "--offline"], 2);
    assert!(String::from_utf8_lossy(&output.stderr).contains("current working directory"));
    assert_eq!(fs::read_dir(t.path()).unwrap().count(), 0);
}
