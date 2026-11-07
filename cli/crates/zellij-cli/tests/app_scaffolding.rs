use serde_json::Value;
use std::{
    collections::BTreeMap,
    fs,
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
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
fn prompt(root: &Path, args: &[&str], input: &str, code: i32) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_zel"))
        .current_dir(root)
        .args(args)
        .env("PATH", root.join("no-tools"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let _ = child.stdin.take().unwrap().write_all(input.as_bytes());
    let out = child.wait_with_output().unwrap();
    assert_eq!(out.status.code(), Some(code), "{args:?}: {out:?}");
    out
}

fn hosted(root: &Path) {
    write(
        root,
        "docs/zellij/zel.yaml",
        "version: 1\nrenderer: hosted\npreview: {port: 4124}\n",
    );
    write(root, "docs/zellij/content/site.yaml", "name: Existing\n");
    write(
        root,
        "docs/zellij/content/guide/index.mdx",
        "# Preserve this content\n",
    );
}

#[test]
fn app_wizard_matches_flag_output_for_both_sources_and_all_feature_choices() {
    for with_docs in [false, true] {
        for feature in ["none", "maths", "diagrams", "all"] {
            let temp = tempfile::tempdir().unwrap();
            let root = temp.path();
            engine(root, "local engine #1%");
            hosted(root);
            let original = tree(&root.join("docs"));
            let selection = if with_docs {
                "starter\n"
            } else {
                "existing\ndocs/zellij\n"
            };
            prompt(
                root,
                &["na", "wizard", "-i"],
                &format!("local engine #1%\n{selection}Acme 日本\n4125\n{feature}\n"),
                0,
            );
            let mut args = vec![
                "new",
                "app",
                "flags",
                "--engine",
                "local engine #1%",
                "--name",
                "Acme 日本",
                "--port",
                "4125",
                "--features",
                feature,
            ];
            if with_docs {
                args.push("--with-docs");
            } else {
                args.extend(["--site-root", "docs/zellij"]);
            }
            run(root, &args, 0);
            assert_eq!(tree(&root.join("wizard")), tree(&root.join("flags")));
            assert_eq!(tree(&root.join("docs")), original);
            let package = manifest(root, "wizard");
            assert_eq!(
                package["dependencies"].get("mermaid").is_some(),
                ["diagrams", "all"].contains(&feature)
            );
            assert_eq!(
                package["dependencies"].get("rehype-mathjax").is_some(),
                ["maths", "all"].contains(&feature)
            );
            for path in ["node_modules", "package-lock.json", ".next"] {
                assert!(!root.join("wizard").join(path).exists());
            }
        }
    }
}

#[test]
fn wizard_defaults_match_discovery_and_explicit_none_matches_no_features() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    engine(root, "engine/packages/zellij");
    hosted(root);
    prompt(root, &["na", "website", "-i"], "\n\n\n\n\n\n", 0);
    run(root, &["na", "flags", "--name", "website"], 0);
    run(
        root,
        &["na", "none", "--name", "website", "--features", "none"],
        0,
    );
    assert_eq!(tree(&root.join("website")), tree(&root.join("flags")));
    assert_eq!(tree(&root.join("none")), tree(&root.join("flags")));
    assert_eq!(
        manifest(root, "website")["scripts"]["dev"],
        "zellij dev -- --port 4124"
    );
    assert_eq!(
        manifest(root, "website")["zellij"]["features"],
        serde_json::json!([])
    );
}

#[test]
fn explicit_settings_skip_questions_and_quiet_retains_needed_prompts() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    engine(root, "engine/packages/zellij");
    hosted(root);
    for (target, source) in [
        ("starter", vec!["--with-docs"]),
        ("external", vec!["--site-root", "docs/zellij"]),
    ] {
        let mut args = vec![
            "--quiet",
            "na",
            target,
            "-i",
            "--engine",
            "engine/packages/zellij",
            "--name",
            "Preset",
            "--port",
            "4126",
            "--features",
            "none",
        ];
        args.extend(source);
        let out = prompt(root, &args, "", 0);
        assert!(out.stdout.is_empty());
        let text = String::from_utf8(out.stderr).unwrap();
        for label in [
            "Built engine package [",
            "Content [",
            "Hosted site folder [",
            "App name [",
            "Dev/start port [",
            "Optional features [",
        ] {
            assert!(!text.contains(label), "{text}");
        }
    }
    let out = prompt(
        root,
        &[
            "--quiet",
            "na",
            "partial",
            "-i",
            "--with-docs",
            "--engine",
            "engine/packages/zellij",
            "--features",
            "none",
        ],
        "\n\n",
        0,
    );
    assert!(out.stdout.is_empty());
    assert!(
        String::from_utf8(out.stderr)
            .unwrap()
            .contains("App name [partial]")
    );
    assert_eq!(
        manifest(root, "partial")["scripts"]["start"],
        "next start --port 3000"
    );
}

#[test]
fn invalid_answers_retry_without_writes_and_choices_accept_numeric_crlf_input() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    engine(root, "engine/packages/zellij");
    engine(root, "bad-engine");
    write(root, "bad-engine/package.json", "{}");
    hosted(root);
    run(root, &["init", "portable", "--minimal"], 0);
    let out = prompt(
        root,
        &["na", "website", "-i"],
        "missing\r\nbad-engine\r\nengine/packages/zellij\r\nmissing\r\n0\r\n1\r\nmissing\r\nportable\r\ndocs/zellij\r\n \r\nbad\tname\r\nAcme 日本\r\n0\r\n65536\r\n-1\r\nabc\r\n4127\r\nmissing\r\n4\r\n",
        0,
    );
    let text = String::from_utf8(out.stderr).unwrap();
    assert!(text.contains("requires a built local engine"));
    assert!(text.contains("matching wrapper/feature capabilities"));
    assert!(text.contains("choose a name or number"));
    assert!(text.contains("references hosted content"));
    assert!(text.contains("app name must be nonempty"));
    assert!(text.contains("between 1 and 65535"));
    assert_eq!(
        manifest(root, "website")["scripts"]["start"],
        "next start --port 4127"
    );
    assert_eq!(
        manifest(root, "website")["zellij"]["features"],
        serde_json::json!(["maths", "diagrams"])
    );
}

#[test]
fn cancellation_at_every_question_leaves_no_wrapper_and_preserves_dotfiles() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    engine(root, "engine/packages/zellij");
    hosted(root);
    write(root, "existing/.git/config", "authored git metadata");
    let before = tree(&root.join("existing"));
    let answers = [
        "engine/packages/zellij",
        "existing",
        "docs/zellij",
        "Acme",
        "4124",
    ];
    for n in 0..=answers.len() {
        let prefix = answers[..n]
            .iter()
            .map(|a| format!("{a}\n"))
            .collect::<String>();
        for suffix in ["", ":q\n"] {
            for destination in ["missing/wrapper", "existing"] {
                let out = prompt(
                    root,
                    &["na", destination, "-i"],
                    &(prefix.clone() + suffix),
                    2,
                );
                assert!(
                    String::from_utf8(out.stderr)
                        .unwrap()
                        .contains("app creation cancelled")
                );
                assert!(out.stdout.is_empty());
                assert!(!root.join("missing").exists());
                assert_eq!(tree(&root.join("existing")), before);
            }
        }
    }
    prompt(
        root,
        &[
            "na",
            "missing",
            "-i",
            "--with-docs",
            "--engine",
            "engine/packages/zellij",
        ],
        "Acme\n4124\n:q\n",
        2,
    );
    assert!(!root.join("missing").exists());
}

#[test]
fn bad_flags_sources_and_late_collisions_fail_without_partial_wrapper_files() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    engine(root, "engine/packages/zellij");
    hosted(root);
    run(root, &["init", "portable", "--minimal"], 0);
    for flags in [
        vec!["--name", "bad\nname"],
        vec!["--features", "none,all"],
        vec!["--engine", "missing"],
        vec!["--site-root", "portable"],
        vec!["--port", "0"],
        vec!["--with-docs", "--config", "docs/zellij/zel.yaml"],
    ] {
        let mut args = vec!["na", "missing", "-i"];
        args.extend(flags);
        let out = prompt(root, &args, "", 2);
        assert!(
            !String::from_utf8(out.stderr)
                .unwrap()
                .contains("Built engine package [")
        );
        assert!(!root.join("missing").exists());
    }
    prompt(
        root,
        &[
            "na",
            "docs/zellij/content/inside",
            "-i",
            "--site-root",
            "docs/zellij",
        ],
        "",
        2,
    );
    assert!(!root.join("docs/zellij/content/inside").exists());
    prompt(
        root,
        &[
            "na",
            "engine/packages/zellij/inside",
            "-i",
            "--engine",
            "engine/packages/zellij",
        ],
        "",
        2,
    );
    assert!(!root.join("engine/packages/zellij/inside").exists());
    write(root, "visible/keep.txt", "authored");
    for args in [
        vec!["na", "visible", "-i"],
        vec!["na", "visible", "--with-docs"],
    ] {
        let out = prompt(root, &args, "", 2);
        let text = String::from_utf8(out.stderr).unwrap();
        assert!(text.contains("choose a different destination"));
        assert!(!text.contains("--force"));
        assert_eq!(tree(&root.join("visible")).len(), 1);
    }
    write(root, "hidden/.npmrc", "authored");
    prompt(
        root,
        &[
            "na",
            "hidden",
            "-i",
            "--with-docs",
            "--engine",
            "engine/packages/zellij",
        ],
        "Acme\n4124\nnone\n",
        2,
    );
    assert_eq!(tree(&root.join("hidden")).len(), 1);
    assert_eq!(
        fs::read_to_string(root.join("hidden/.npmrc")).unwrap(),
        "authored"
    );
}

#[test]
fn explicit_config_and_cwd_skip_source_questions_and_keep_selected_port() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    let product = root.join("Product 日本");
    fs::create_dir(&product).unwrap();
    engine(&product, "engine/packages/zellij");
    hosted(&product);
    fs::rename(
        product.join("docs/zellij/zel.yaml"),
        product.join("docs/zellij/selected.yaml"),
    )
    .unwrap();
    write(&product, "docs/zellij/zel.yaml", "invalid: [");
    let before = tree(&product.join("docs"));
    let out = prompt(
        root,
        &[
            "--cwd",
            "Product 日本",
            "--config",
            "docs/zellij/selected.yaml",
            "na",
            "wrapper",
            "-i",
        ],
        "\n\n\n\n",
        0,
    );
    let text = String::from_utf8(out.stderr).unwrap();
    assert!(!text.contains("Content ["));
    assert!(!text.contains("Hosted site folder ["));
    assert_eq!(
        manifest(&product, "wrapper")["scripts"]["dev"],
        "zellij dev -- --port 4124"
    );
    assert_eq!(tree(&product.join("docs")), before);
    assert!(!root.join("wrapper").exists());
}

#[cfg(unix)]
#[test]
fn interactive_final_plan_refuses_symlinked_files_without_writing_earlier_entries() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    engine(root, "engine/packages/zellij");
    write(root, "outside", "retain");
    fs::create_dir(root.join("wrapper")).unwrap();
    std::os::unix::fs::symlink(root.join("outside"), root.join("wrapper/.npmrc")).unwrap();
    prompt(
        root,
        &[
            "na",
            "wrapper",
            "-i",
            "--with-docs",
            "--engine",
            "engine/packages/zellij",
        ],
        "Acme\n3000\nnone\n",
        2,
    );
    assert_eq!(fs::read_dir(root.join("wrapper")).unwrap().count(), 1);
    assert_eq!(fs::read_to_string(root.join("outside")).unwrap(), "retain");
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
