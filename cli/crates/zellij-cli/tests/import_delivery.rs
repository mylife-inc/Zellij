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
    if format == "mkdocs" {
        write(
            root,
            "source/mkdocs.yml",
            "site_name: Import\nnav:\n  - index.md\n  - next.md\n  - External: https://example.com\n",
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
    command.current_dir(root).env("PATH", root.join("bin"));
    for key in [
        "DOCKER_HOST",
        "DOCKER_CONTEXT",
        "CONTAINER_HOST",
        "CONTAINER_CONNECTION",
        "CONTAINERD_ADDRESS",
        "CONTAINERD_NAMESPACE",
        "NETLIFY_AUTH_TOKEN",
        "NETLIFY_SITE_ID",
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
    for format in ["mkdocs", "docusaurus"] {
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
            "vercel",
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
            ],
            0,
        );
        let stdout = String::from_utf8_lossy(&result.stdout);
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
            .arg(root.join("source/docs"))
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
