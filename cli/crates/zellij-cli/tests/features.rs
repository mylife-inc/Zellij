use std::{
    fs,
    path::Path,
    process::{Command, Output},
};
fn write(root: &Path, name: &str, value: impl AsRef<[u8]>) {
    let path = root.join(name);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, value).unwrap();
}
fn run(root: &Path, args: &[&str], code: i32) -> Output {
    let output = Command::new(env!("CARGO_BIN_EXE_zel"))
        .current_dir(root)
        .args(args)
        .env("PATH", root.join("no-tools"))
        .env("DOCKER_HOST", "unavailable")
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(code), "{args:?}: {output:?}");
    output
}
#[test]
fn portable_selection_preserves_comments_flow_settings_and_changes_rendered_output() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    for source in [
        "# authored\nversion: 1\nrenderer: portable\nname: 'My docs' # inline\n",
        "---\nversion: 1\nrenderer: portable\nmarkdown:\n  # diagrams\n  mermaid: false # keep\n...\n",
        "{version: 1, renderer: portable, markdown: { math: false, mermaid: false }}\n",
        "version: 1\r\nrenderer: portable\r\nmarkdown: {} # inline\r\n",
        "version: 1\nrenderer: portable\nmarkdown:\n    math: false # formula\n",
    ] {
        write(root, "docs/zellij/zel.yaml", source);
        write(
            root,
            "docs/zellij/content/index.md",
            "# Hello\n\n$x^2$\n\n```mermaid\ngraph TD; A-->B;\n```\n",
        );
        run(root, &["features", "all"], 0);
        let changed = fs::read_to_string(root.join("docs/zellij/zel.yaml")).unwrap();
        for comment in [
            "# authored",
            "# inline",
            "# diagrams",
            "# keep",
            "# formula",
        ] {
            if source.contains(comment) {
                assert!(changed.contains(comment), "{changed}");
            }
        }
        let yaml: serde_yaml_ng::Value = serde_yaml_ng::from_str(&changed).unwrap();
        assert_eq!(yaml["markdown"]["math"], true);
        assert_eq!(yaml["markdown"]["mermaid"], true);
        run(root, &["features", "all"], 0);
        assert_eq!(
            fs::read_to_string(root.join("docs/zellij/zel.yaml")).unwrap(),
            changed
        );
    }
    run(root, &["build"], 0);
    let enabled = fs::read_to_string(root.join("docs/zellij/site/index.html")).unwrap();
    run(root, &["features", "none"], 0);
    run(root, &["build"], 0);
    let disabled = fs::read_to_string(root.join("docs/zellij/site/index.html")).unwrap();
    assert_ne!(enabled, disabled);
    assert!(enabled.contains("<math"));
    assert!(!disabled.contains("<math"));
    assert!(disabled.contains("$x^2$"));
    run(
        root,
        &["features", "diagrams", "--site-root", "docs/zellij"],
        0,
    );
    let yaml: serde_yaml_ng::Value =
        serde_yaml_ng::from_str(&fs::read_to_string(root.join("docs/zellij/zel.yaml")).unwrap())
            .unwrap();
    assert_eq!(yaml["markdown"]["math"], false);
    assert_eq!(yaml["markdown"]["mermaid"], true);
}
#[test]
fn feature_errors_do_not_modify_files_or_probe_runtimes() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    for source in [
        "version: 1\nrenderer: portable\nmarkdown: { math: nope }\n",
        "version: 1\nrenderer: portable\nmarkdown: &shared { math: true }\n",
        "version: 1\nrenderer: portable\nmarkdown: { math: false, math: true }\n",
        "version: 1\nrenderer: hosted\n",
    ] {
        write(root, "zel.yaml", source);
        write(root, "content/index.md", "# Hello\n");
        run(root, &["features", "none"], 2);
        assert_eq!(fs::read_to_string(root.join("zel.yaml")).unwrap(), source);
    }
    for args in [
        vec!["features", "none,maths"],
        vec!["features", "unknown"],
        vec!["features", "all", "--json"],
        vec!["features", "all,diagrams"],
    ] {
        run(root, &args, 2);
    }
    write(root, "zel.yaml", "version: 1\nrenderer: portable\n");
    run(root, &["eject", "--out", "out"], 2);
    assert!(!root.join("out").exists());
    run(root, &["eject", "--out", "out", "--with-docs"], 4);
    assert!(!root.join("out").exists());
    run(
        root,
        &[
            "eject",
            "--out",
            "out",
            "--with-docs",
            "--image",
            "--invalid",
        ],
        2,
    );
}
#[test]
fn wrapper_dependencies_keep_authored_fields_and_validate_engine_before_writing() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    let template = zellij_scaffold::app::template();
    write(root,"engine/package.json",serde_json::to_vec(&serde_json::json!({"name":"@shebka/zellij","version":template["engineVersion"],"zellij":{"wrapperConfigVersion":1,"featureSelectionVersion":1}})).unwrap());
    for name in [
        "bin/zellij.mjs",
        "next-config.mjs",
        "deployment-config.cjs",
        "feature-config.cjs",
        "feature-config.d.cts",
        "optional-maths.mjs",
        "optional-maths.d.mts",
        "optional-mermaid.mjs",
        "optional-mermaid.d.mts",
        "disabled-feature.mjs",
        "src/index.ts",
        "dist/cli/check-bundle.js",
        "dist/cli/dev-bundle.js",
        "dist/cli/init-bundle.js",
        "dist/cli/sync-assets-bundle.js",
        "dist/cli/write-search-index-bundle.js",
    ] {
        write(root, &format!("engine/{name}"), "fixture");
    }
    run(root, &["na", "app", "--with-docs", "--engine", "engine"], 0);
    let app = root.join("app");
    let manifest = || {
        serde_json::from_slice::<serde_json::Value>(&fs::read(app.join("package.json")).unwrap())
            .unwrap()
    };
    let mut authored = manifest();
    authored["custom"] = serde_json::json!({"preserve":true});
    authored["zellij"]["custom"] = serde_json::json!("kept");
    authored["dependencies"]["extra"] = "1.2.3".into();
    authored["overrides"]["mermaid"]["custom"] = "4".into();
    write(&app, "package.json", serde_json::to_vec(&authored).unwrap());
    write(&app, "package-lock.json", "authored lockfile");
    run(&app, &["features", "maths,diagrams"], 0);
    assert!(manifest()["dependencies"]["mermaid"].is_string());
    assert_eq!(
        manifest()["zellij"]["features"],
        serde_json::json!(["maths", "diagrams"])
    );
    assert_eq!(manifest()["overrides"]["mermaid"]["custom"], "4");
    run(root, &["features", "none", "--site-root", "app"], 0);
    let changed = manifest();
    assert_eq!(changed["zellij"]["features"], serde_json::json!([]));
    assert_eq!(changed["dependencies"]["mermaid"], serde_json::Value::Null);
    assert_eq!(changed["custom"], authored["custom"]);
    assert_eq!(changed["zellij"]["custom"], "kept");
    assert_eq!(changed["dependencies"]["extra"], "1.2.3");
    assert_eq!(
        fs::read_to_string(app.join("package-lock.json")).unwrap(),
        "authored lockfile"
    );
    let engine_before = fs::read(root.join("engine/package.json")).unwrap();
    let mut old_engine: serde_json::Value = serde_json::from_slice(&engine_before).unwrap();
    old_engine["zellij"]
        .as_object_mut()
        .unwrap()
        .remove("featureSelectionVersion");
    write(
        root,
        "engine/package.json",
        serde_json::to_vec(&old_engine).unwrap(),
    );
    let manifest_before = fs::read(app.join("package.json")).unwrap();
    run(&app, &["features", "all"], 2);
    assert_eq!(fs::read(app.join("package.json")).unwrap(), manifest_before);
    write(root, "engine/package.json", engine_before);
    fs::remove_file(root.join("engine/dist/cli/init-bundle.js")).unwrap();
    let before = fs::read(app.join("package.json")).unwrap();
    run(&app, &["features", "all"], 2);
    assert_eq!(fs::read(app.join("package.json")).unwrap(), before);
}
#[cfg(unix)]
#[test]
fn feature_selection_refuses_linked_configs() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    write(root, "authored", "version: 1\nrenderer: portable\n");
    fs::create_dir(root.join("content")).unwrap();
    std::os::unix::fs::symlink(root.join("authored"), root.join("zel.yaml")).unwrap();
    run(root, &["features", "all"], 2);
    assert_eq!(
        fs::read_to_string(root.join("authored")).unwrap(),
        "version: 1\nrenderer: portable\n"
    );
}
