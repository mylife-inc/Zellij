use std::{fs, path::Path, process::Command};
fn run(root: &Path, args: &[&str], code: i32) -> std::process::Output {
    let out = Command::new(env!("CARGO_BIN_EXE_zel"))
        .current_dir(root)
        .args(args)
        .env("PATH", root.join("no-tools"))
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
        for command in [
            vec!["create", id, "-t", id, "--offline"],
            vec![
                "new",
                "site",
                &format!("{id}-canonical"),
                "--template",
                &format!("{id}@0.1.0"),
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
    assert!(
        fs::read_to_string(root.join("manuscript/content/index.md"))
            .unwrap()
            .contains("A\\. Writer")
    );
    run(root, &["build", "manuscript"], 0);
}

#[test]
fn current_directory_is_not_replaced_or_detached() {
    let t = tempfile::tempdir().unwrap();
    let output = run(t.path(), &["create", ".", "--offline"], 2);
    assert!(String::from_utf8_lossy(&output.stderr).contains("current working directory"));
    assert_eq!(fs::read_dir(t.path()).unwrap().count(), 0);
}
