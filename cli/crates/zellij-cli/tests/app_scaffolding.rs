use serde_json::Value;
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};
fn write(root: &Path, path: &str, body: impl AsRef<[u8]>) {
    let path = root.join(path);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, body).unwrap();
}
fn engine(root: &Path, path: &str) {
    let version = zellij_scaffold::app::template()["engineVersion"].clone();
    write(
        root,
        &format!("{path}/package.json"),
        serde_json::to_vec(&serde_json::json!({"name":"@shebka/zellij", "version":version, "zellij":{"wrapperConfigVersion":1,"featureSelectionVersion":1}}))
            .unwrap(),
    );
    for file in [
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
        write(root, &format!("{path}/{file}"), "fixture");
    }
}
fn run(root: &Path, args: &[&str], code: i32) -> Output {
    let result = Command::new(env!("CARGO_BIN_EXE_zel"))
        .current_dir(root)
        .args(args)
        .env("PATH", root.join("no-tools"))
        .env("DOCKER_HOST", "fixture-unusable")
        .output()
        .unwrap();
    assert_eq!(result.status.code(), Some(code), "{args:?}: {result:?}");
    result
}
fn tree(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    fn visit(base: &Path, root: &Path, files: &mut BTreeMap<PathBuf, Vec<u8>>) {
        for entry in fs::read_dir(root).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                visit(base, &path, files);
            } else {
                files.insert(
                    path.strip_prefix(base).unwrap().into(),
                    fs::read(path).unwrap(),
                );
            }
        }
    }
    let mut files = BTreeMap::new();
    visit(root, root, &mut files);
    files
}
fn manifest(root: &Path, app: &str) -> Value {
    serde_json::from_slice(&fs::read(root.join(app).join("package.json")).unwrap()).unwrap()
}
fn config(root: &Path, path: &str) -> serde_yaml_ng::Value {
    serde_yaml_ng::from_slice(&fs::read(root.join(path)).unwrap()).unwrap()
}
#[test]
fn native_scaffold_and_alias_share_templates_without_installation_or_runtime() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    engine(root, "local engine #1%");
    for command in [vec!["new", "app", "long"], vec!["na", "short"]] {
        let out = command[command.len() - 1];
        let mut args = command;
        args.extend([
            "--with-docs",
            "--engine",
            "local engine #1%",
            "--name",
            "My docs 日本: #1",
            "--features",
            "all",
            "--port",
            "4123",
        ]);
        run(root, &args, 0);
        let package = manifest(root, out);
        assert_eq!(package["name"], "my-docs-1");
        assert_eq!(package["scripts"]["dev"], "zellij dev -- --port 4123");
        assert_eq!(package["scripts"]["start"], "next start --port 4123");
        assert_eq!(
            package["overrides"],
            zellij_scaffold::app::template()["manifest"]["overrides"]
        );
        assert!(package["dependencies"]["mermaid"].is_string());
        assert!(package["dependencies"]["rehype-mathjax"].is_string());
        let engine_path = package["dependencies"]["@shebka/zellij"]
            .as_str()
            .unwrap()
            .strip_prefix("file:")
            .unwrap();
        assert!(engine_path.ends_with("local engine #1%"));
        assert_eq!(
            fs::canonicalize(engine_path).unwrap(),
            fs::canonicalize(root.join("local engine #1%")).unwrap()
        );
        assert_eq!(
            config(root, &format!("{out}/zellij.config.yaml"))["content"],
            "docs/zellij/content"
        );
        assert_eq!(
            config(root, &format!("{out}/zellij.config.yaml"))["version"],
            1
        );
        assert_eq!(
            config(root, &format!("{out}/docs/zellij/zel.yaml"))["renderer"],
            "hosted"
        );
        assert_eq!(
            config(root, &format!("{out}/docs/zellij/content/site.yaml"))["name"],
            "My docs 日本: #1"
        );
        for file in ["node_modules", "package-lock.json", ".next"] {
            assert!(!root.join(out).join(file).exists());
        }
    }
    assert_eq!(tree(&root.join("long")), tree(&root.join("short")));
}
#[test]
fn selects_configured_and_legacy_hosted_content_and_preserves_sources() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    engine(root, "engine/packages/zellij");
    write(
        root,
        "docs/zellij/zel.yaml",
        "version: 1\nrenderer: hosted\npreview: {port: 4124}\n",
    );
    write(
        root,
        "docs/zellij/content/site.yaml",
        "name: External\nbasePath: /manual\n",
    );
    write(root, "docs/zellij/content/assets/data.bin", [0, 255, 3]);
    let before = tree(&root.join("docs"));
    run(root, &["na", "app"], 0);
    assert_eq!(
        manifest(root, "app")["scripts"]["start"],
        "next start --port 4124"
    );
    assert_eq!(
        PathBuf::from(
            config(root, "app/zellij.config.yaml")["content"]
                .as_str()
                .unwrap()
        ),
        fs::canonicalize(root.join("docs/zellij/content")).unwrap()
    );
    assert!(!root.join("app/content").exists());
    run(
        root,
        &[
            "new",
            "app",
            "legacy",
            "--site-root",
            "docs/zellij/content",
            "--port",
            "4125",
            "--features",
            "maths",
        ],
        0,
    );
    assert_eq!(
        manifest(root, "legacy")["scripts"]["start"],
        "next start --port 4125"
    );
    assert!(
        manifest(root, "legacy")["dependencies"]
            .get("mermaid")
            .is_none()
    );
    assert_eq!(tree(&root.join("docs")), before);
    // Current-directory discovery takes precedence over the repository convention.
    write(root, "zel.yaml", "version: 1\nrenderer: hosted\n");
    write(root, "content/site.yaml", "name: Current\n");
    run(root, &["na", "current"], 0);
    assert_eq!(
        PathBuf::from(
            config(root, "current/zellij.config.yaml")["content"]
                .as_str()
                .unwrap()
        ),
        fs::canonicalize(root.join("content")).unwrap()
    );
}
#[test]
fn invalid_options_engines_sources_and_destinations_do_not_write() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    engine(root, "engine/packages/zellij");
    for args in [
        vec!["na", "out"],
        vec!["na", "out", "--engine", "missing", "--with-docs"],
        vec!["na", "out", "--with-docs", "--site-root", "docs/zellij"],
        vec!["na", "out", "--with-docs", "--port", "0"],
        vec!["na", "out", "--with-docs", "--features", "unknown"],
        vec!["na", "out", "--with-docs", "--name", "\ninvalid"],
        vec!["na", "out", "--with-docs", "--json"],
        vec!["na", "out", "--with-docs", "--force"],
        vec!["na", "engine/packages/zellij/inside", "--with-docs"],
    ] {
        run(root, &args, 2);
        assert!(!root.join("out").exists());
    }
    run(root, &["init", "portable", "--minimal"], 0);
    run(root, &["na", "out", "--site-root", "portable"], 2);
    assert!(!root.join("out").exists());
    write(
        root,
        "engine/packages/zellij/package.json",
        "{\"name\":\"other\",\"version\":\"0.1.0\"}",
    );
    run(root, &["na", "out", "--with-docs"], 2);
    write(
        root,
        "engine/packages/zellij/package.json",
        "{\"name\":\"@shebka/zellij\",\"version\":\"0.1.0\"}",
    );
    run(root, &["na", "out", "--with-docs"], 2);
    write(
        root,
        "engine/packages/zellij/package.json",
        "{\"name\":\"@shebka/zellij\",\"version\":\"999.0.0\",\"zellij\":{\"wrapperConfigVersion\":1}}",
    );
    run(root, &["na", "out", "--with-docs"], 2);
    engine(root, "engine/packages/zellij");
    fs::remove_file(root.join("engine/packages/zellij/dist/cli/dev-bundle.js")).unwrap();
    run(root, &["na", "out", "--with-docs"], 2);
    assert!(!root.join("out").exists());
    engine(root, "engine/packages/zellij");
    write(root, "out/.npmrc", "authored\n");
    run(root, &["na", "out", "--with-docs"], 2);
    assert_eq!(tree(&root.join("out")).len(), 1);
    assert_eq!(
        fs::read_to_string(root.join("out/.npmrc")).unwrap(),
        "authored\n"
    );
    fs::remove_file(root.join("out/.npmrc")).unwrap();
    write(root, "out/keep.txt", "authored");
    run(root, &["na", "out", "--with-docs"], 2);
    assert_eq!(tree(&root.join("out")).len(), 1);
    write(root, "source/site.yaml", "name: Source\n");
    run(root, &["na", "source/child", "--site-root", "source"], 2);
    assert!(!root.join("source/child").exists());
}
#[cfg(unix)]
#[test]
fn refuses_destination_symlinks_and_preserves_existing_git_metadata() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    engine(root, "engine/packages/zellij");
    write(root, "outside", "unchanged");
    fs::create_dir(root.join("out")).unwrap();
    std::os::unix::fs::symlink(root.join("outside"), root.join("out/.npmrc")).unwrap();
    run(root, &["na", "out", "--with-docs"], 2);
    assert_eq!(
        fs::read_to_string(root.join("outside")).unwrap(),
        "unchanged"
    );
    write(root, "fresh/.git/config", "authored git configuration");
    run(root, &["na", "fresh", "--with-docs"], 0);
    assert_eq!(
        fs::read_to_string(root.join("fresh/.git/config")).unwrap(),
        "authored git configuration"
    );
}
