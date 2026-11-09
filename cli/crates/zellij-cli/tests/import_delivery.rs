//! Import/delivery boundaries, using isolated paths and disposable fake tools.
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
fn tree(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    fn visit(root: &Path, dir: &Path, result: &mut BTreeMap<PathBuf, Vec<u8>>) {
        for entry in fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                visit(root, &path, result);
            } else {
                result.insert(
                    path.strip_prefix(root).unwrap().into(),
                    fs::read(path).unwrap(),
                );
            }
        }
    }
    let mut result = BTreeMap::new();
    visit(root, root, &mut result);
    result
}
fn fixture(root: &Path, format: &str) {
    if format == "sphinx" {
        write(
            root,
            "source/conf.py",
            "project = 'Import'\nroot_doc = 'index'\n",
        );
        write(
            root,
            "source/index.rst",
            "Import\n======\n\n.. toctree::\n\n   next\n\n:doc:`Next <next>`\n\n:download:`Download <media/report.pdf>`\n",
        );
        write(
            root,
            "source/next.rst",
            "Next\n====\n\n:doc:`Home <index>`\n",
        );
        write(
            root,
            "source/media/report.pdf",
            [0x25, 0x50, 0x44, 0x46, 0, 255],
        );
        return;
    }
    if format == "mkdocs" {
        write(
            root,
            "source/mkdocs.yml",
            "site_name: Import\nnav:\n  - index.md\n  - next.md\n  - External: https://example.com\n",
        );
    } else if format == "hugo" {
        write(
            root,
            "source/hugo.toml",
            "title='Import'\ncontentDir='docs'\n",
        );
    } else if format == "jekyll" {
        write(root, "source/_config.yml", "title: Import\nsource: docs\n");
        write(
            root,
            "source/docs/_data/navigation.yml",
            "- name: Home\n  link: /\n- name: Next\n  link: /next.html\n",
        );
    } else if format == "vitepress" {
        write(
            root,
            "source/.vitepress/config.ts",
            "export default {title:'Import', srcDir:'docs', themeConfig:{sidebar:[{link:'/'},{link:'/next'}]}};",
        );
    } else {
        write(
            root,
            "source/docusaurus.config.js",
            "module.exports = {title: 'Import'};",
        );
        write(
            root,
            "source/sidebars.js",
            "module.exports = {docs: ['index', 'next']};",
        );
    }
    write(
        root,
        "source/docs/index.md",
        "# Import\n\n[Next](next.md)\n\n[Download](media/report.pdf)\n",
    );
    write(root, "source/docs/next.md", "# Next\n\n[Home](index.md)\n");
    write(
        root,
        "source/docs/media/report.pdf",
        [0x25, 0x50, 0x44, 0x46, 0, 255],
    );
    write(root, "source/docs/orphan.md", "# Outside navigation\n");
    if format == "hugo" {
        fs::rename(
            root.join("source/docs/index.md"),
            root.join("source/docs/_index.md"),
        )
        .unwrap();
        write(root, "source/docs/next.md", "# Next\n\n[Home](_index.md)\n");
        write(
            root,
            "source/docs/orphan.md",
            "---\ndraft: true\n---\n# Outside navigation\n",
        );
    }
    if format == "jekyll" {
        for name in ["index.md", "next.md", "orphan.md"] {
            let file = format!("source/docs/{name}");
            let body = fs::read_to_string(root.join(&file)).unwrap();
            write(root, &file, format!("---\n---\n{body}"));
        }
    }
}
fn command(root: &Path) -> Command {
    #[cfg(not(windows))]
    let executable = PathBuf::from(env!("CARGO_BIN_EXE_zel"));
    #[cfg(windows)]
    let executable = {
        let bin = root.join("bin");
        fs::create_dir_all(&bin).unwrap();
        let executable = bin.join("zel.exe");
        if !executable.exists() {
            fs::copy(env!("CARGO_BIN_EXE_zel"), &executable).unwrap();
            for program in ["docker.exe", "podman.exe", "nerdctl.exe"] {
                fs::write(bin.join(program), b"not an executable").unwrap();
            }
        }
        executable
    };
    let mut command = Command::new(executable);
    command
        .current_dir(root)
        .env_remove("RAILWAY_TOKEN")
        .env_remove("RAILWAY_PROJECT_ID")
        .env_remove("RAILWAY_ENVIRONMENT_ID")
        .env_remove("RAILWAY_SERVICE_ID")
        .env("PATH", root.join("bin"));
    for key in [
        "DOCKER_HOST",
        "DOCKER_CONTEXT",
        "CONTAINER_HOST",
        "CONTAINER_CONNECTION",
        "CONTAINERD_ADDRESS",
        "CONTAINERD_NAMESPACE",
        "NETLIFY_AUTH_TOKEN",
        "NETLIFY_SITE_ID",
        "AWS_ACCESS_KEY_ID",
        "AWS_SECRET_ACCESS_KEY",
        "AWS_SESSION_TOKEN",
        "AWS_REGION",
        "S3_EXPECTED_BUCKET_OWNER",
        "S3_DESTINATION",
        "AWS_ENDPOINT_URL",
        "AWS_ENDPOINT_URL_S3",
    ] {
        command.env_remove(key);
    }
    command
}
fn run(root: &Path, args: &[&str], code: i32) -> Output {
    let result = command(root).args(args).output().unwrap();
    assert_eq!(result.status.code(), Some(code), "{args:?}: {result:?}");
    result
}
#[test]
fn configured_and_legacy_imports_preserve_routes_assets_aliases_and_source_bytes() {
    for format in ["mkdocs", "docusaurus", "vitepress", "jekyll", "hugo"] {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path();
        fixture(root, format);
        let original = tree(&root.join("source"));
        run(root, &["convert", "source", "--out", "legacy"], 0);
        run(
            root,
            &[
                "cv",
                "source",
                "--out",
                "explicit-legacy",
                "--layout",
                "content",
                "--assets",
                "copy",
            ],
            0,
        );
        run(
            root,
            &["cv", "source", "--out", "Manual 日本", "--layout", "site"],
            0,
        );
        assert_eq!(
            tree(&root.join("legacy")),
            tree(&root.join("explicit-legacy"))
        );
        assert_eq!(
            tree(&root.join("legacy")),
            tree(&root.join("Manual 日本/content"))
        );
        assert_eq!(
            fs::read_to_string(root.join("Manual 日本/zel.yaml")).unwrap(),
            "version: 1\nrenderer: hosted\n"
        );
        assert!(!root.join("Manual 日本/site").exists());
        assert_eq!(tree(&root.join("source")), original);
        for name in ["convert", "cv", "migrate", "mig"] {
            let result = run(
                root,
                &[
                    name,
                    "source",
                    "--out",
                    "plan",
                    "--layout",
                    "site",
                    "--dry-run",
                    "--from",
                    format,
                ],
                0,
            );
            let stdout = String::from_utf8_lossy(&result.stdout);
            assert!(
                stdout.contains("zel.yaml")
                    && stdout.contains("content/assets/imported/docs/media/report.pdf"),
                "{stdout}"
            );
            assert!(stdout.contains("orphan.md"), "{stdout}");
            assert!(!root.join("plan").exists());
        }
    }
}

#[test]
fn skipped_assets_keep_original_urls_while_document_links_are_converted() {
    for format in ["mkdocs", "docusaurus", "vitepress", "jekyll", "hugo"] {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path();
        fixture(root, format);
        if format == "docusaurus" {
            write(root, "source/static/img/logo.svg", "<svg/>");
            write(
                root,
                "source/docs/next.md",
                "# Next\n\n![Logo](/img/logo.svg)\n",
            );
        }
        let original = tree(&root.join("source"));
        for (name, layout) in [("convert", "content"), ("cv", "site")] {
            let result = run(
                root,
                &[
                    name, "source", "--out", layout, "--layout", layout, "--assets", "skip",
                ],
                0,
            );
            let content = root.join(if layout == "site" {
                "site/content"
            } else {
                "content"
            });
            assert!(!content.join("assets").exists());
            let page = fs::read_to_string(content.join("01-guide/index.mdx")).unwrap();
            assert!(page.contains("[Next](</guide/next>)"), "{page}");
            assert!(page.contains("[Download](media/report.pdf)"), "{page}");
            let report = String::from_utf8_lossy(&result.stdout);
            assert!(
                report.contains("media/report.pdf") && report.contains("asset copying is disabled"),
                "{report}"
            );
            if format == "docusaurus" {
                let page = fs::read_to_string(content.join("01-guide/next.mdx")).unwrap();
                assert!(page.contains("![Logo](/img/logo.svg)"), "{page}");
                assert!(report.contains("/img/logo.svg"), "{report}");
            }
        }
        assert_eq!(tree(&root.join("source")), original);
    }
}

#[test]
fn import_controls_reject_unknown_asset_policies_and_unsafe_skipped_assets_before_writing() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    fixture(root, "mkdocs");
    for name in ["convert", "cv", "migrate", "mig", "ship"] {
        let mut args = vec![name, "source", "--out", "out", "--assets", "unknown"];
        if name == "ship" {
            args.extend(["--image", "static", "--tag", "example/docs:v1"]);
        }
        run(root, &args, 2);
        assert!(!root.join("out").exists());
    }
    for href in ["../../private.txt", "../.git/config"] {
        write(
            root,
            "source/docs/index.md",
            format!("# Unsafe\n\n[File]({href})\n"),
        );
        run(
            root,
            &["cv", "source", "--out", "out", "--assets", "skip"],
            2,
        );
        assert!(!root.join("out").exists());
    }
    #[cfg(unix)]
    {
        write(root, "private.txt", "private bytes");
        std::os::unix::fs::symlink(
            root.join("private.txt"),
            root.join("source/docs/private.txt"),
        )
        .unwrap();
        write(
            root,
            "source/docs/index.md",
            "# Unsafe\n\n[File](private.txt)\n",
        );
        run(
            root,
            &["cv", "source", "--out", "out", "--assets", "skip"],
            2,
        );
        assert!(!root.join("out").exists());
    }
}

#[test]
fn validation_requests_are_inert_in_dry_runs_and_retain_imports_when_runtime_is_missing() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    fixture(root, "mkdocs");
    let original = tree(&root.join("source"));
    for name in ["convert", "cv", "migrate", "mig"] {
        for layout in ["content", "site"] {
            let destination = format!("{name}-{layout}");
            let args = [
                name,
                "source",
                "--out",
                &destination,
                "--layout",
                layout,
                "--assets",
                "skip",
                "--validate",
            ];
            let mut dry = args.to_vec();
            dry.push("--dry-run");
            let output = run(root, &dry, 0);
            assert!(String::from_utf8_lossy(&output.stdout).contains("Plan only"));
            assert!(!root.join(&destination).exists());
            let result = run(root, &args, 4);
            assert!(String::from_utf8_lossy(&result.stderr).contains("Import completed"));
            let content =
                root.join(&destination)
                    .join(if layout == "site" { "content" } else { "" });
            assert!(content.join("site.yaml").is_file());
            assert!(!content.join("assets").exists());
        }
    }
    for kind in ["static", "next"] {
        let args = [
            "ship",
            "source",
            "--out",
            kind,
            "--image",
            kind,
            "--tag",
            "example/docs:v1",
            "--assets",
            "skip",
        ];
        let mut dry = args.to_vec();
        dry.push("--dry-run");
        run(root, &dry, 0);
        assert!(!root.join(kind).exists());
        let result = run(root, &args, 4);
        assert!(String::from_utf8_lossy(&result.stderr).contains("Import completed"));
        assert!(!root.join(kind).join("content/assets").exists());
        let config = fs::read_to_string(root.join(kind).join("zel.yaml")).unwrap();
        assert!(config.contains(&format!("image: {kind}")), "{config}");
        assert!(!config.contains("assets:"));
    }
    assert_eq!(tree(&root.join("source")), original);
}

#[cfg(unix)]
#[test]
fn optional_validation_checks_the_written_layout_once_and_preserves_failures() {
    use std::os::unix::fs::PermissionsExt;
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    fixture(root, "mkdocs");
    write(
        root,
        "bin/docker",
        r#"#!/bin/sh
printf '%s\n' "$1" >> "$FIXTURE_ROOT/calls"
case "$1" in
  info) exit 0;;
  --version) echo fixture;;
  run)
    printf '%s\n' "$@" > "$FIXTURE_ROOT/arguments"
    test -f "$EXPECTED_CONTENT/site.yaml" || exit 99
    test -f "$EXPECTED_CONTENT/01-guide/index.mdx" || exit 99
    exit "$CHECK_EXIT";;
  *) exit 99;;
esac
"#,
    );
    fs::set_permissions(root.join("bin/docker"), fs::Permissions::from_mode(0o755)).unwrap();
    run(root, &["cv", "source", "--out", "unchecked"], 0);
    run(
        root,
        &["cv", "source", "--out", "plan", "--validate", "--dry-run"],
        0,
    );
    assert!(!root.join("calls").exists());
    for layout in ["content", "site"] {
        for (engine_exit, code) in [("0", 0), ("1", 3)] {
            let destination = format!("checked-{layout}-{engine_exit}");
            let content =
                root.join(&destination)
                    .join(if layout == "site" { "content" } else { "" });
            let _ = fs::remove_file(root.join("calls"));
            let output = command(root)
                .args([
                    "cv",
                    "source",
                    "--out",
                    &destination,
                    "--layout",
                    layout,
                    "--validate",
                ])
                .env("DOCKER_HOST", "unix:///fixture-docker.sock")
                .env("ZELLIJ_IMAGE", "fixture/engine:test")
                .env("FIXTURE_ROOT", root)
                .env("EXPECTED_CONTENT", &content)
                .env("CHECK_EXIT", engine_exit)
                .output()
                .unwrap();
            assert_eq!(output.status.code(), Some(code), "{output:?}");
            let calls = fs::read_to_string(root.join("calls")).unwrap();
            assert_eq!(
                calls.lines().filter(|line| *line == "run").count(),
                1,
                "{calls}"
            );
            let arguments = fs::read_to_string(root.join("arguments")).unwrap();
            let mount = format!(
                "{}:/content:ro\n",
                content.canonicalize().unwrap().display()
            );
            assert!(
                arguments.contains(&mount) && arguments.contains("check /content"),
                "{arguments}"
            );
            assert!(
                content
                    .join("assets/imported/docs/media/report.pdf")
                    .is_file()
            );
            assert_eq!(
                String::from_utf8_lossy(&output.stderr).contains("Import completed"),
                code != 0
            );
        }
    }
}
#[test]
fn ambiguous_sources_require_explicit_selection_before_any_write() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    fixture(root, "mkdocs");
    fixture(root, "docusaurus");
    for name in ["convert", "migrate", "ship"] {
        let mut args = vec![name, "source", "--out", "out"];
        if name == "ship" {
            args.extend(["--provider", "netlify", "--site-id", "fixture"]);
        }
        let result = run(root, &args, 2);
        assert!(
            String::from_utf8_lossy(&result.stderr)
                .contains("ambiguous source (docusaurus, mkdocs)")
        );
        assert!(!root.join("out").exists());
    }
    run(
        root,
        &[
            "cv", "source", "--out", "out", "--from", "mkdocs", "--layout", "site",
        ],
        0,
    );
}
#[test]
fn invalid_imports_and_delivery_options_never_create_destinations() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    fixture(root, "mkdocs");
    let original = tree(&root.join("source"));
    for args in [
        vec!["cv", "source", "--out", "source/child", "--layout", "site"],
        vec!["mig", "source", "--out", "source"],
        vec!["cv", "source", "--out", "out", "--from", "unknown"],
        vec!["cv", "source", "--out", "out", "--json"],
        vec!["ship", "source", "--out", "out"],
        vec!["ship", "source", "--out", "out", "--provider", "netlify"],
        vec![
            "ship",
            "source",
            "--out",
            "out",
            "--provider",
            "fly",
            "--site-id",
            "fixture",
        ],
        vec![
            "ship",
            "source",
            "--out",
            "out",
            "--provider",
            "netlify",
            "--site-id",
            "https://bad",
        ],
        vec!["ship", "source", "--out", "out", "--image", "static"],
        vec![
            "ship",
            "source",
            "--out",
            "out",
            "--image",
            "static",
            "--tag",
            "acme/docs:v1",
            "--prod",
        ],
        vec![
            "ship",
            "source",
            "--out",
            "out",
            "--image",
            "next",
            "--tag",
            "acme/docs:v1",
            "--provider",
            "netlify",
        ],
        vec![
            "ship",
            "source",
            "--out",
            "out",
            "--image",
            "static",
            "--tag",
            "https://bad",
        ],
        vec![
            "ship",
            "source",
            "--out",
            "out",
            "--provider",
            "netlify",
            "--site",
            "fixture",
            "--json",
        ],
    ] {
        run(root, &args, 2);
        assert!(!root.join("out").exists());
    }
    write(root, "occupied/keep", "authored");
    run(
        root,
        &[
            "cv",
            "source",
            "--out",
            "occupied",
            "--layout",
            "site",
            "--dry-run",
        ],
        2,
    );
    assert_eq!(
        fs::read_to_string(root.join("occupied/keep")).unwrap(),
        "authored"
    );
    assert_eq!(tree(&root.join("source")), original);
    write(
        root,
        "source/docs/index.md",
        "# Bad\n\n[Outside](../../private.txt)\n",
    );
    write(root, "private.txt", "private bytes");
    run(
        root,
        &["cv", "source", "--out", "out", "--layout", "site"],
        2,
    );
    assert!(!root.join("out").exists());
}
#[test]
fn ship_plans_are_inert_and_failures_retain_configured_imports_for_review() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    fixture(root, "mkdocs");
    let original = tree(&root.join("source"));
    for kind in ["static", "next"] {
        let result = run(
            root,
            &[
                "ship",
                "source",
                "--out",
                "plan",
                "--image",
                kind,
                "--tag",
                "acme/docs:v1",
                "--dry-run",
                "--features",
                "none",
            ],
            0,
        );
        let stdout = String::from_utf8_lossy(&result.stdout);
        assert!(stdout.contains("Features none (this invocation only)"));
        assert!(
            stdout.contains("acme/docs:v1")
                && stdout.contains("orphan.md")
                && stdout.contains("zel.yaml")
        );
        assert!(!root.join("plan").exists());
    }
    let result = command(root)
        .args([
            "ship",
            "source",
            "--out",
            "plan",
            "--provider",
            "netlify",
            "--site",
            "override",
            "--prod",
            "--dry-run",
        ])
        .env("NETLIFY_SITE_ID", "environment")
        .env("NETLIFY_AUTH_TOKEN", "must-not-leak")
        .output()
        .unwrap();
    assert!(result.status.success(), "{result:?}");
    let stdout = String::from_utf8_lossy(&result.stdout);
    assert!(
        stdout.contains("override")
            && stdout.contains("production")
            && !stdout.contains("environment")
            && !stdout.contains("must-not-leak")
    );
    assert!(!root.join("plan").exists());
    let result = run(
        root,
        &[
            "ship",
            "source",
            "--out",
            "netlify",
            "--provider",
            "netlify",
            "--site-id",
            "fixture",
            "--prod",
        ],
        2,
    );
    assert!(String::from_utf8_lossy(&result.stderr).contains("Import completed"));
    let config = fs::read_to_string(root.join("netlify/zel.yaml")).unwrap();
    assert!(
        config.contains("site: \"fixture\"")
            && !config.contains("prod")
            && !config.contains("token")
    );
    let result = run(root, &["pub", "netlify", "--dry-run"], 0);
    assert!(String::from_utf8_lossy(&result.stdout).contains("draft preview"));
    for name in ["migrate", "mig"] {
        let result = run(
            root,
            &[name, "source", "--out", name, "--layout", "site"],
            4,
        );
        assert!(String::from_utf8_lossy(&result.stderr).contains("Import completed"));
        assert!(root.join(name).join("content/site.yaml").is_file());
    }
    let result = run(
        root,
        &[
            "ship",
            "source",
            "--out",
            "image",
            "--image",
            "next",
            "--tag",
            "acme/docs:v1",
        ],
        4,
    );
    assert!(String::from_utf8_lossy(&result.stderr).contains("Import completed"));
    run(root, &["pub", "image", "--dry-run"], 0);
    assert_eq!(tree(&root.join("source")), original);
}
#[cfg(unix)]
#[test]
fn ship_builds_once_pushes_exactly_once_and_stops_after_failed_build() {
    use std::os::unix::fs::PermissionsExt;
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    fixture(root, "mkdocs");
    let original = tree(&root.join("source"));
    write(
        root,
        "bin/docker",
        r#"#!/bin/sh
printf '%s\n' "$1" >> "$FIXTURE_ROOT/calls"
case "$1" in
  info) exit 0;;
  --version) echo fixture;;
  build) cat > "$FIXTURE_ROOT/recipe"; printf '%s\n' "$@" > "$FIXTURE_ROOT/build"; exit "${FAIL_BUILD:-0}";;
  push) printf '%s\n' "$@" > "$FIXTURE_ROOT/push"; exit "${FAIL_PUSH:-0}";;
  *) exit 99;;
esac
"#,
    );
    fs::set_permissions(root.join("bin/docker"), fs::Permissions::from_mode(0o755)).unwrap();
    for (out, kind, build_exit, push_exit) in [
        ("static", "static", "0", "0"),
        ("next", "next", "0", "0"),
        ("bad-build", "static", "1", "0"),
        ("bad-push", "next", "0", "1"),
    ] {
        let _ = fs::remove_file(root.join("calls"));
        let result = command(root)
            .args([
                "ship",
                "source",
                "--out",
                out,
                "--image",
                kind,
                "--tag",
                "localhost:5000/acme/docs:v1",
                "--features",
                if kind == "static" { "all" } else { "none" },
            ])
            .env(
                "PATH",
                format!("{}:/usr/bin:/bin", root.join("bin").display()),
            )
            .env("FIXTURE_ROOT", root)
            .env("ZELLIJ_IMAGE", "fixture/base:test")
            .env("FAIL_BUILD", build_exit)
            .env("FAIL_PUSH", push_exit)
            .output()
            .unwrap();
        assert_eq!(
            result.status.code(),
            Some(if build_exit == "0" && push_exit == "0" {
                0
            } else {
                3
            }),
            "{result:?}"
        );
        let calls = fs::read_to_string(root.join("calls")).unwrap();
        assert_eq!(
            calls.lines().filter(|line| *line == "build").count(),
            1,
            "{calls}"
        );
        assert_eq!(
            calls.lines().filter(|line| *line == "push").count(),
            usize::from(build_exit == "0"),
            "{calls}"
        );
        assert!(
            !calls.lines().any(|line| line == "run" || line == "tag"),
            "{calls}"
        );
        assert!(
            fs::read_to_string(root.join("build"))
                .unwrap()
                .contains("-t\nlocalhost:5000/acme/docs:v1\n")
        );
        if build_exit == "0" {
            assert_eq!(
                fs::read_to_string(root.join("push")).unwrap(),
                "push\nlocalhost:5000/acme/docs:v1\n"
            );
        }
        assert_eq!(
            fs::read_to_string(root.join("recipe"))
                .unwrap()
                .contains(".next/standalone"),
            kind == "next"
        );
        assert!(root.join(out).join("content/site.yaml").is_file());
        let selection = if kind == "static" {
            "maths,diagrams"
        } else {
            "none"
        };
        assert!(
            fs::read_to_string(root.join("recipe"))
                .unwrap()
                .contains(&format!("features \"{selection}\" /app"))
        );
        assert!(
            !fs::read_to_string(root.join(out).join("zel.yaml"))
                .unwrap()
                .contains("features")
        );
        assert!(!root.join(out).join("site").exists());
        run(root, &["pub", out, "--dry-run"], 0);
        assert_eq!(tree(&root.join("source")), original);
    }
}

#[cfg(any(unix, windows))]
#[test]
fn imports_reject_linked_document_directories_before_writing() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    fixture(root, "mkdocs");
    fs::rename(root.join("source/docs"), root.join("outside")).unwrap();
    #[cfg(unix)]
    std::os::unix::fs::symlink(root.join("outside"), root.join("source/docs")).unwrap();
    #[cfg(windows)]
    {
        let status = Command::new("cmd")
            .args(["/C", "mklink", "/J"])
            .arg(root.join("source").join("docs"))
            .arg(root.join("outside"))
            .status()
            .unwrap();
        assert!(status.success());
    }
    let before = tree(&root.join("outside"));
    let result = run(
        root,
        &["cv", "source", "--out", "out", "--layout", "site"],
        2,
    );
    assert!(String::from_utf8_lossy(&result.stderr).contains("symbolic links and junctions"));
    assert!(!root.join("out").exists());
    assert_eq!(tree(&root.join("outside")), before);
    // Remove the link itself before TempDir cleanup on Windows.
    #[cfg(windows)]
    fs::remove_dir(root.join("source/docs")).unwrap();
}

#[test]
fn ship_saves_the_platform_before_a_missing_runtime_and_dry_run_writes_nothing() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    fixture(root, "mkdocs");
    let args = [
        "ship",
        "source",
        "--out",
        "out",
        "--image",
        "static",
        "--tag",
        "acme/docs:v1",
        "--platform",
        "linux/arm64",
    ];
    let plan = command(root).args(args).arg("--dry-run").output().unwrap();
    assert!(plan.status.success(), "{plan:?}");
    assert!(String::from_utf8_lossy(&plan.stdout).contains("linux/arm64"));
    assert!(!root.join("out").exists());
    let result = command(root).args(args).output().unwrap();
    assert!(!result.status.success());
    assert!(
        String::from_utf8_lossy(&result.stderr).contains("Import completed"),
        "{result:?}"
    );
    let config = fs::read_to_string(root.join("out/zel.yaml")).unwrap();
    assert!(config.contains("platform: linux/arm64"), "{config}");
    let result = command(root)
        .args(["pub", "out", "--dry-run"])
        .output()
        .unwrap();
    assert!(result.status.success(), "{result:?}");
    assert!(String::from_utf8_lossy(&result.stdout).contains("linux/arm64"));
}

#[test]
fn cloudflare_ship_plans_without_credentials_and_retains_site_on_credential_failure() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    fixture(root, "mkdocs");
    let original = tree(&root.join("source"));
    let args = [
        "ship",
        "source",
        "--out",
        "converted",
        "--provider",
        "cloudflare",
        "--site",
        "manual",
    ];
    let output = command(root)
        .env_remove("CLOUDFLARE_API_TOKEN")
        .env_remove("CLOUDFLARE_ACCOUNT_ID")
        .args(args)
        .arg("--dry-run")
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    assert!(!root.join("converted").exists());
    let output = command(root)
        .env_remove("CLOUDFLARE_API_TOKEN")
        .env_remove("CLOUDFLARE_ACCOUNT_ID")
        .args(args)
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("CLOUDFLARE_API_TOKEN"));
    let config = fs::read_to_string(root.join("converted/zel.yaml")).unwrap();
    assert!(config.contains("provider: cloudflare") && config.contains("manual"));
    assert!(!config.contains("token"));
    assert!(root.join("converted/content/site.yaml").is_file());
    assert_eq!(tree(&root.join("source")), original);
}

#[test]
fn vercel_ship_plans_without_credentials_and_retains_site_on_credential_failure() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    fixture(root, "mkdocs");
    let original = tree(&root.join("source"));
    let args = [
        "ship",
        "source",
        "--out",
        "converted",
        "--provider",
        "vercel",
        "--site",
        "manual",
    ];
    let output = command(root)
        .env_remove("VERCEL_TOKEN")
        .env_remove("VERCEL_TEAM_ID")
        .args(args)
        .arg("--dry-run")
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    assert!(!root.join("converted").exists());
    let output = command(root)
        .env_remove("VERCEL_TOKEN")
        .env_remove("VERCEL_TEAM_ID")
        .args(args)
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("VERCEL_TOKEN"));
    let config = fs::read_to_string(root.join("converted/zel.yaml")).unwrap();
    assert!(config.contains("provider: vercel") && config.contains("manual"));
    assert!(!config.contains("token"));
    assert!(root.join("converted/content/site.yaml").is_file());
    assert_eq!(tree(&root.join("source")), original);
}

#[test]
fn github_pages_ship_plans_without_credentials_and_retains_site_on_credential_failure() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    fixture(root, "mkdocs");
    let original = tree(&root.join("source"));
    let args = [
        "ship",
        "source",
        "--out",
        "converted",
        "--provider",
        "github-pages",
        "--site",
        "acme/manual",
        "--prod",
    ];
    let denied = command(root)
        .args(&args[..args.len() - 1])
        .output()
        .unwrap();
    assert_eq!(denied.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&denied.stderr).contains("--prod"));
    assert!(!root.join("converted").exists());
    let output = command(root)
        .env_remove("GITHUB_PAGES_TOKEN")
        .args(args)
        .arg("--dry-run")
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    assert!(!root.join("converted").exists());
    let output = command(root)
        .env_remove("GITHUB_PAGES_TOKEN")
        .args(args)
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("GITHUB_PAGES_TOKEN"));
    let config = fs::read_to_string(root.join("converted/zel.yaml")).unwrap();
    assert!(config.contains("provider: github-pages") && config.contains("manual"));
    assert!(!config.contains("token") && !config.contains("production"));
    assert!(root.join("converted/content/site.yaml").is_file());
    assert_eq!(tree(&root.join("source")), original);
}

#[test]
fn s3_ship_plans_without_credentials_and_retains_site_on_credential_failure() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    fixture(root, "mkdocs");
    let original = tree(&root.join("source"));
    let args = [
        "ship",
        "source",
        "--out",
        "converted",
        "--provider",
        "s3",
        "--site",
        "acme-docs/manual",
        "--prod",
    ];
    let denied = command(root)
        .args(&args[..args.len() - 1])
        .output()
        .unwrap();
    assert_eq!(denied.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&denied.stderr).contains("--prod"));
    assert!(!root.join("converted").exists());
    let output = command(root)
        .env_remove("AWS_ACCESS_KEY_ID")
        .args(args)
        .arg("--dry-run")
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    assert!(!root.join("converted").exists());
    let output = command(root)
        .env_remove("AWS_ACCESS_KEY_ID")
        .args(args)
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("AWS_ACCESS_KEY_ID"));
    let config = fs::read_to_string(root.join("converted/zel.yaml")).unwrap();
    assert!(config.contains("provider: s3") && config.contains("manual"));
    assert!(!config.contains("token") && !config.contains("production"));
    assert!(root.join("converted/content/site.yaml").is_file());
    assert_eq!(tree(&root.join("source")), original);
}

#[test]
fn fly_ship_preserves_source_and_saves_selected_kind_after_credential_failure() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    fixture(root, "mkdocs");
    let original = tree(&root.join("source"));
    let args = [
        "ship",
        "source",
        "--out",
        "converted",
        "--provider",
        "fly",
        "--site",
        "manual",
        "--kind",
        "next",
    ];
    let denied = command(root).args(args).output().unwrap();
    assert_eq!(denied.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&denied.stderr).contains("--prod"));
    assert!(!root.join("converted").exists());
    let denied = command(root)
        .args(args)
        .args(["--prod", "--platform", "all"])
        .output()
        .unwrap();
    assert_eq!(denied.status.code(), Some(2));
    assert!(!root.join("converted").exists());
    let plan = command(root)
        .args(args)
        .args(["--prod", "--dry-run"])
        .output()
        .unwrap();
    assert!(plan.status.success(), "{plan:?}");
    assert!(!root.join("converted").exists());
    let output = command(root)
        .env_remove("FLY_API_TOKEN")
        .env_remove("FLY_ORG")
        .args(args)
        .arg("--prod")
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("FLY_API_TOKEN"));
    let config = fs::read_to_string(root.join("converted/zel.yaml")).unwrap();
    assert!(
        config.contains("provider: fly")
            && config.contains("image: next")
            && config.contains("manual")
    );
    assert!(!config.contains("token") && !config.contains("production"));
    assert_eq!(tree(&root.join("source")), original);
}

#[test]
fn railway_ship_validates_before_import_and_retains_import_on_credential_failure() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    fixture(root, "mkdocs");
    let original = tree(&root.join("source"));
    let args = [
        "ship",
        "source",
        "--out",
        "converted",
        "--provider",
        "railway",
        "--site",
        "33333333-3333-4333-8333-333333333333",
        "--kind",
        "next",
        "--prod",
    ];
    let output = command(root).args(args).output().unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert!(!root.join("converted").exists());
    let output = command(root)
        .args(args)
        .args(["--tag", "ghcr.io/acme/docs:v1", "--dry-run"])
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    assert!(!root.join("converted").exists());
    let output = command(root)
        .env_remove("RAILWAY_TOKEN")
        .args(args)
        .args(["--tag", "ghcr.io/acme/docs:v1"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("RAILWAY_TOKEN"));
    let saved = fs::read_to_string(root.join("converted/zel.yaml")).unwrap();
    assert!(
        saved.contains("provider: railway")
            && saved.contains("image: next")
            && saved.contains("ghcr.io/acme/docs:v1")
    );
    assert!(!saved.contains("token") && !saved.contains("production"));
    assert_eq!(tree(&root.join("source")), original);
}

#[test]
fn linked_assets_use_a_public_prefix_in_every_converter_without_copying_files() {
    for format in [
        "mkdocs",
        "docusaurus",
        "vitepress",
        "jekyll",
        "hugo",
        "sphinx",
    ] {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path();
        fixture(root, format);
        let original = tree(&root.join("source"));
        for (command, layout) in [("convert", "content"), ("cv", "site")] {
            let result = run(
                root,
                &[
                    command,
                    "source",
                    "--out",
                    layout,
                    "--layout",
                    layout,
                    "--assets",
                    "link",
                    "--asset-base-url",
                    "https://cdn.example.test/manual",
                ],
                0,
            );
            let content = root
                .join(layout)
                .join(if layout == "site" { "content" } else { "" });
            let page = fs::read_to_string(content.join("01-guide/index.mdx")).unwrap();
            assert!(
                page.contains("https://cdn.example.test/manual/media/report.pdf"),
                "{format}: {page}"
            );
            assert!(page.contains("/guide/next"), "{page}");
            assert!(!content.join("assets").exists());
            let report = String::from_utf8_lossy(&result.stdout);
            assert!(
                report.contains("1 linked asset")
                    && report.contains("https://cdn.example.test/manual/media/report.pdf"),
                "{report}"
            );
            assert!(!report.contains("asset copying is disabled"), "{report}");
        }
        for command in ["migrate", "mig", "ship"] {
            let mut args = vec![
                command,
                "source",
                "--out",
                "plan",
                "--assets",
                "link",
                "--asset-base-url",
                "https://cdn.example.test/manual/",
                "--dry-run",
            ];
            if command == "ship" {
                args.extend(["--image", "static", "--tag", "example/docs:v1"]);
            }
            let result = run(root, &args, 0);
            assert!(
                String::from_utf8_lossy(&result.stdout)
                    .contains("https://cdn.example.test/manual/media/report.pdf")
            );
            assert!(!root.join("plan").exists());
        }
        assert_eq!(tree(&root.join("source")), original);
    }
}

#[test]
fn linked_assets_preserve_encoded_names_queries_fragments_and_external_links() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    fixture(root, "mkdocs");
    write(root, "source/docs/media/café %20#.svg", "<svg/>");
    write(
        root,
        "source/docs/next.md",
        r#"# Next

![Diagram](media/caf%C3%A9%20%2520%23.svg?v=2#part)
[Again][asset]

[asset]: media/caf%C3%A9%20%2520%23.svg?download=1#part "Caption"

[External](https://other.example/logo.svg)

```md
![Example](media/caf%C3%A9%20%2520%23.svg)
```
"#,
    );
    let result = run(
        root,
        &[
            "cv",
            "source",
            "--out",
            "out",
            "--assets",
            "link",
            "--asset-base-url",
            "https://cdn.example.test/base%20dir/日本/",
        ],
        0,
    );
    let page = fs::read_to_string(root.join("out/01-guide/next.mdx")).unwrap();
    let prefix =
        "https://cdn.example.test/base%20dir/%E6%97%A5%E6%9C%AC/media/caf%C3%A9%20%2520%23.svg";
    assert!(page.contains(&format!("{prefix}?v=2#part")), "{page}");
    assert!(
        page.contains(&format!("{prefix}?download=1#part")),
        "{page}"
    );
    assert!(
        page.contains("[External](https://other.example/logo.svg)"),
        "{page}"
    );
    assert!(
        page.contains("![Example](media/caf%C3%A9%20%2520%23.svg)"),
        "{page}"
    );
    assert!(String::from_utf8_lossy(&result.stdout).contains("2 linked assets"));
    assert!(!root.join("out/assets").exists());
}

#[test]
fn linked_assets_require_a_valid_explicit_base_and_reject_unused_base_options_before_writes() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    fixture(root, "mkdocs");
    for command in ["convert", "cv", "migrate", "mig", "ship"] {
        let mut args = vec![command, "source", "--out", "out"];
        if command == "ship" {
            args.extend(["--image", "static", "--tag", "example/docs:v1"]);
        }
        let mut missing = args.clone();
        missing.extend(["--assets", "link"]);
        let result = run(root, &missing, 2);
        assert!(String::from_utf8_lossy(&result.stderr).contains("--asset-base-url"));
        for policy in ["copy", "skip"] {
            let mut unused = args.clone();
            unused.extend([
                "--assets",
                policy,
                "--asset-base-url",
                "https://cdn.example.test",
            ]);
            let result = run(root, &unused, 2);
            assert!(String::from_utf8_lossy(&result.stderr).contains("--assets link"));
        }
        for base in [
            "",
            "cdn.example.test",
            "//cdn.example.test",
            "https:cdn.example.test",
            "file:///tmp/assets",
            "ftp://cdn.example.test",
            "https://user:password@cdn.example.test",
            "https://@cdn.example.test",
            "https://cdn.example.test/?signed=secret",
            "https://cdn.example.test/#fragment",
            "https://cdn.example.test/?",
            "https://cdn.example.test/#",
            " https://cdn.example.test/",
            "https://cdn.example.test/line\nbreak",
            "https://cdn.example.test/a\\b",
            "https://cdn.example.test/%ZZ",
            "https://cdn.example.test/a/../b",
            "https://cdn.example.test/a/%2e%2e/b",
        ] {
            let mut invalid = args.clone();
            invalid.extend(["--assets", "link", "--asset-base-url", base, "--dry-run"]);
            run(root, &invalid, 2);
            assert!(!root.join("out").exists());
        }
    }
}

#[test]
fn linked_assets_keep_path_containment_and_reject_conflicting_source_roots() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    fixture(root, "docusaurus");
    write(root, "source/static/media/report.pdf", "a different asset");
    write(
        root,
        "source/docs/next.md",
        "# Next\n\n[Static](/media/report.pdf)\n",
    );
    for extra in [vec![], vec!["--dry-run"]] {
        let mut args = vec![
            "cv",
            "source",
            "--out",
            "out",
            "--assets",
            "link",
            "--asset-base-url",
            "https://cdn.example.test/",
        ];
        args.extend(extra);
        let result = run(root, &args, 2);
        assert!(
            String::from_utf8_lossy(&result.stderr).contains("same public asset URL"),
            "{result:?}"
        );
        assert!(!root.join("out").exists());
    }
    for href in ["../../private.txt", "../.git/config"] {
        write(
            root,
            "source/docs/index.md",
            format!("# Unsafe\n\n[File]({href})\n"),
        );
        run(
            root,
            &[
                "cv",
                "source",
                "--out",
                "out",
                "--assets",
                "link",
                "--asset-base-url",
                "https://cdn.example.test/",
            ],
            2,
        );
        assert!(!root.join("out").exists());
    }
}

#[test]
fn linked_public_and_static_assets_use_their_asset_root_without_namespace_prefixes() {
    for format in ["docusaurus", "vitepress", "hugo"] {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path();
        fixture(root, format);
        let folder = if format == "vitepress" {
            "source/docs/public"
        } else {
            "source/static"
        };
        write(root, &format!("{folder}/logo.svg"), "<svg/>");
        write(
            root,
            "source/docs/next.md",
            "# Next\n\n![Logo](/logo.svg)\n",
        );
        run(
            root,
            &[
                "cv",
                "source",
                "--out",
                "out",
                "--assets",
                "link",
                "--asset-base-url",
                "http://cdn.example.test:8080/assets",
            ],
            0,
        );
        let page = fs::read_to_string(root.join("out/01-guide/next.mdx")).unwrap();
        assert!(
            page.contains("http://cdn.example.test:8080/assets/logo.svg"),
            "{format}: {page}"
        );
        assert!(!root.join("out/assets").exists());
    }
}

#[test]
fn linked_assets_do_not_invent_urls_for_missing_files_or_unpublished_documents() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    fixture(root, "mkdocs");
    write(
        root,
        "source/docs/next.md",
        "# Next\n\n![Missing](missing.svg)\n\n[Unlisted](orphan.md)\n",
    );
    let result = run(
        root,
        &[
            "cv",
            "source",
            "--out",
            "out",
            "--assets",
            "link",
            "--asset-base-url",
            "https://cdn.example.test/",
        ],
        0,
    );
    let page = fs::read_to_string(root.join("out/01-guide/next.mdx")).unwrap();
    assert!(page.contains("![Missing](missing.svg)"), "{page}");
    assert!(page.contains("[Unlisted](orphan.md)"), "{page}");
    assert!(!page.contains("cdn.example.test"), "{page}");
    let report = String::from_utf8_lossy(&result.stdout);
    assert!(
        report.contains("no converted document or regular asset file")
            && report.contains("was not included"),
        "{report}"
    );
}

#[cfg(unix)]
#[test]
fn linked_assets_reject_symlinks_without_reading_regular_asset_bytes() {
    use std::os::unix::fs::{PermissionsExt, symlink};
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    fixture(root, "mkdocs");
    let asset = root.join("source/docs/media/report.pdf");
    let permissions = fs::metadata(&asset).unwrap().permissions();
    fs::set_permissions(&asset, fs::Permissions::from_mode(0o000)).unwrap();
    let result = command(root)
        .args([
            "cv",
            "source",
            "--out",
            "out",
            "--assets",
            "link",
            "--asset-base-url",
            "https://cdn.example.test/",
        ])
        .output()
        .unwrap();
    fs::set_permissions(&asset, permissions).unwrap();
    assert!(result.status.success(), "{result:?}");
    assert!(!root.join("out/assets").exists());
    fs::remove_file(&asset).unwrap();
    write(root, "private.pdf", "private bytes");
    symlink(root.join("private.pdf"), &asset).unwrap();
    run(
        root,
        &[
            "cv",
            "source",
            "--out",
            "bad",
            "--assets",
            "link",
            "--asset-base-url",
            "https://cdn.example.test/",
        ],
        2,
    );
    assert!(!root.join("bad").exists());
}

#[test]
fn linked_ship_and_migration_retain_external_urls_if_runtime_is_missing() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    fixture(root, "mkdocs");
    for command in ["migrate", "mig", "ship"] {
        let mut args = vec![
            command,
            "source",
            "--out",
            command,
            "--assets",
            "link",
            "--asset-base-url",
            "https://cdn.example.test/",
        ];
        if command == "ship" {
            args.extend(["--image", "next", "--tag", "example/docs:v1"]);
        } else {
            args.extend(["--layout", "site"]);
        }
        let result = run(root, &args, 4);
        assert!(String::from_utf8_lossy(&result.stderr).contains("Import completed"));
        let content = root.join(command).join("content");
        assert!(!content.join("assets").exists());
        assert!(
            fs::read_to_string(content.join("01-guide/index.mdx"))
                .unwrap()
                .contains("https://cdn.example.test/media/report.pdf")
        );
        assert!(
            !fs::read_to_string(root.join(command).join("zel.yaml"))
                .unwrap()
                .contains("asset-base-url")
        );
    }
}
