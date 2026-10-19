//! Build-context and failure tests without a daemon. Actual image HTTP behavior
//! is tested by scripts/check-static-images.mjs in container acceptance.
#![cfg(unix)]
use std::{fs, os::unix::fs::PermissionsExt, path::Path, process::Command};

fn command(root: &Path) -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_zel"));
    cmd.current_dir(root)
        .env(
            "PATH",
            format!("{}:/usr/bin:/bin", root.join("bin").display()),
        )
        .env("DOCKER_HOST", "unix:///fake-zellij.sock")
        .env("ZELLIJ_IMAGE", "fixture/base:test");
    for key in [
        "DOCKER_CONTEXT",
        "CONTAINER_HOST",
        "CONTAINER_CONNECTION",
        "CONTAINERD_ADDRESS",
        "CONTAINERD_NAMESPACE",
    ] {
        cmd.env_remove(key);
    }
    cmd
}

fn fixture(root: &Path) {
    fs::create_dir_all(root.join("bin")).unwrap();
    fs::write(
        root.join("bin/docker"),
        r#"#!/bin/sh
case "$1" in
  info) exit 0;;
  --version) echo 'fixture docker'; exit 0;;
  build)
    printf '%s\n' "$@" > arguments
    cat > recipe
    for last do :; done
    printf '%s' "$last" > context-path
    find "$last" -type f | sed "s|$last/||" | sort > context-files
    if [ -f "$last/index.html" ]; then cp "$last/index.html" captured.html; fi
    exit "${FAKE_BUILD_EXIT:-0}";;
  *) exit 99;;
esac
"#,
    )
    .unwrap();
    fs::set_permissions(root.join("bin/docker"), fs::Permissions::from_mode(0o755)).unwrap();
    fs::create_dir_all(root.join("docs/zellij/content/nested")).unwrap();
    fs::write(
        root.join("docs/zellij/zel.yaml"),
        "version: 1\nrenderer: portable\nname: Image fixture\n",
    )
    .unwrap();
    fs::write(
        root.join("docs/zellij/content/index.md"),
        "# Image fixture\n\n[Page](nested/page.md#details)\n\n[Download](manual.pdf)\n",
    )
    .unwrap();
    fs::write(
        root.join("docs/zellij/content/nested/page.md"),
        "# Page\n\n## Details\n\n[Home](../index.md)\n",
    )
    .unwrap();
    fs::write(
        root.join("docs/zellij/content/manual.pdf"),
        "fixture download",
    )
    .unwrap();
    fs::write(
        root.join("docs/zellij/content/unused.txt"),
        "not referenced",
    )
    .unwrap();
    fs::create_dir(root.join("docs/zellij/site")).unwrap();
    fs::write(root.join("docs/zellij/site/index.html"), "previous").unwrap();
}

#[test]
fn static_alias_packages_only_compiled_files_and_cleans_up_after_success_and_failure() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    fixture(root);
    for (args, code, tag) in [
        (vec!["si"], 0, "zellij-static:latest"),
        (
            vec!["image", "static", "docs/zellij", "--tag", "acme/manual:v2"],
            1,
            "acme/manual:v2",
        ),
    ] {
        let output = command(root)
            .args(&args)
            .env("FAKE_BUILD_EXIT", code.to_string())
            .output()
            .unwrap();
        assert_eq!(
            output.status.code(),
            Some(if code == 0 { 0 } else { 3 }),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert_eq!(stdout.contains('✓'), code == 0);
        if code != 0 {
            assert!(String::from_utf8_lossy(&output.stderr).contains("site build failed"));
        }
        let arguments = fs::read_to_string(root.join("arguments")).unwrap();
        assert!(arguments.contains(&format!("-t\n{tag}\n")));
        let files = fs::read_to_string(root.join("context-files")).unwrap();
        for expected in ["index.html", "nested/page.html", "manual.pdf", "_zellij/"] {
            assert!(files.contains(expected), "{files}");
        }
        for forbidden in [".md", "zel.yaml", "unused.txt", ".zellij-output"] {
            assert!(!files.contains(forbidden), "{files}");
        }
        let context = fs::read_to_string(root.join("context-path")).unwrap();
        assert!(
            !Path::new(&context).exists(),
            "temporary export leaked: {context}"
        );
        assert_eq!(
            fs::read_to_string(root.join("docs/zellij/site/index.html")).unwrap(),
            "previous"
        );
        assert!(
            fs::read_to_string(root.join("captured.html"))
                .unwrap()
                .contains("nested/page.html#details")
        );
        let recipe = fs::read_to_string(root.join("recipe")).unwrap();
        assert!(recipe.contains("USER nginx"));
        assert!(!recipe.contains("npm"));
    }
}

#[test]
fn hosted_static_selects_content_context_and_legacy_next_keeps_its_recipe_and_tag() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    fixture(root);
    fs::write(
        root.join("docs/zellij/zel.yaml"),
        "version: 1\nrenderer: hosted\n",
    )
    .unwrap();
    fs::write(root.join("docs/zellij/content/site.yaml"), "name: Hosted\n").unwrap();
    for args in [
        vec!["si"],
        vec!["image", "build"],
        vec!["ni"],
        vec!["build", "--target", "image"],
    ] {
        let output = command(root).args(&args).output().unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let is_static = args[0] == "si";
        let recipe = fs::read_to_string(root.join("recipe")).unwrap();
        assert_eq!(recipe.contains("ENV ZELLIJ_OUTPUT=export"), is_static);
        assert_eq!(recipe.contains(".next/standalone"), !is_static);
        assert!(recipe.contains("FROM fixture/base:test AS build"));
        let path = fs::read_to_string(root.join("context-path")).unwrap();
        assert_eq!(
            Path::new(&path),
            root.join("docs/zellij/content").canonicalize().unwrap()
        );
        let tag = if is_static {
            "zellij-static:latest"
        } else if args[0] == "ni" {
            "zellij-next:latest"
        } else {
            "zellij:latest"
        };
        assert!(
            fs::read_to_string(root.join("arguments"))
                .unwrap()
                .contains(&format!("-t\n{tag}\n"))
        );
    }
}

#[test]
fn invalid_markdown_and_unsupported_json_never_run_an_image_build() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    fixture(root);
    fs::write(
        root.join("docs/zellij/content/index.md"),
        "# Bad\n\n[Missing](absent.md)\n",
    )
    .unwrap();
    for args in [
        vec!["si"],
        vec!["si", "--json"],
        vec!["ni"],
        vec!["ni", "--json"],
    ] {
        let output = command(root).args(args).output().unwrap();
        assert!(!output.status.success());
        assert!(!root.join("arguments").exists());
        assert_eq!(
            fs::read_to_string(root.join("docs/zellij/site/index.html")).unwrap(),
            "previous"
        );
    }
}

#[test]
fn next_alias_adapts_markdown_and_cleans_context_on_failure() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    fixture(root);
    for (args, code, tag) in [
        (vec!["ni"], 0, "zellij-next:latest"),
        (
            vec!["image", "next", "docs/zellij", "--tag", "acme/docs:v2"],
            1,
            "acme/docs:v2",
        ),
    ] {
        let out = command(root)
            .args(args)
            .env("FAKE_BUILD_EXIT", code.to_string())
            .output()
            .unwrap();
        assert_eq!(
            out.status.code(),
            Some(if code == 0 { 0 } else { 3 }),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        assert_eq!(
            String::from_utf8_lossy(&out.stdout).contains('✓'),
            code == 0
        );
        let args = fs::read_to_string(root.join("arguments")).unwrap();
        assert!(args.contains(&format!("-t\n{tag}\n")));
        let files = fs::read_to_string(root.join("context-files")).unwrap();
        for expected in [
            "manual/p0.zeldoc.json",
            "manual/p1.zeldoc.json",
            "manual/_sidebar.yaml",
            "site.yaml",
            "assets/portable/manual.pdf",
        ] {
            assert!(files.contains(expected), "{files}");
        }
        for private in [
            "index.md",
            "page.md",
            "unused.txt",
            "zel.yaml",
            "site/index.html",
        ] {
            assert!(!files.contains(private), "{files}");
        }
        assert!(!Path::new(&fs::read_to_string(root.join("context-path")).unwrap()).exists());
        assert_eq!(
            fs::read_to_string(root.join("docs/zellij/site/index.html")).unwrap(),
            "previous"
        );
        let recipe = fs::read_to_string(root.join("recipe")).unwrap();
        assert!(recipe.contains(".next/standalone"));
        assert!(!recipe.contains("nginx"));
    }
}

#[test]
fn platforms_reach_every_image_build_and_cli_overrides_saved_defaults() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    fixture(root);
    for renderer in ["portable", "hosted"] {
        fs::write(
            root.join("docs/zellij/zel.yaml"),
            format!("version: 1\nrenderer: {renderer}\nimage: {{platform: linux/arm64}}\n"),
        )
        .unwrap();
        fs::write(root.join("docs/zellij/content/site.yaml"), "name: Hosted\n").unwrap();
        let mut forms = vec![
            vec!["si"],
            vec!["ni"],
            vec!["image", "static"],
            vec!["image", "next"],
        ];
        if renderer == "hosted" {
            forms.extend([vec!["image", "build"], vec!["b", "--target", "image"]]);
        }
        for form in forms {
            for (extra, expected) in [
                (vec![], "linux/arm64"),
                (vec!["--platform", "linux/amd64"], "linux/amd64"),
            ] {
                let result = command(root).args(&form).args(extra).output().unwrap();
                assert!(result.status.success(), "{form:?}: {result:?}");
                let arguments = fs::read_to_string(root.join("arguments")).unwrap();
                assert!(
                    arguments.contains(&format!("--platform\n{expected}\n")),
                    "{arguments}"
                );
                assert_eq!(arguments.matches("--platform").count(), 1);
            }
        }
    }
}

#[test]
fn runtime_build_and_push_arguments_preserve_foreign_platforms() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    fixture(root);
    let script = fs::read_to_string(root.join("bin/docker"))
        .unwrap()
        .replace(
            "  *) exit 99;;",
            "  push) printf '%s\\n' \"$@\" > pushed; exit 0;;\n  *) exit 99;;",
        );
    for (program, endpoint) in [
        ("docker", "DOCKER_HOST"),
        ("podman", "CONTAINER_HOST"),
        ("nerdctl", "CONTAINERD_ADDRESS"),
    ] {
        fs::write(root.join(format!("bin/{program}")), &script).unwrap();
        fs::set_permissions(
            root.join(format!("bin/{program}")),
            fs::Permissions::from_mode(0o755),
        )
        .unwrap();
        for args in [
            vec!["si", "--platform", "linux/arm64"],
            vec!["ip", "fixture:latest"],
        ] {
            let out = command(root)
                .env_remove("DOCKER_HOST")
                .env(endpoint, "fixture")
                .args(&args)
                .output()
                .unwrap();
            assert!(out.status.success(), "{program} {args:?}: {out:?}");
        }
        let args = fs::read_to_string(root.join("arguments")).unwrap();
        assert_eq!(args.contains("--progress"), program != "podman", "{args}");
        assert!(args.contains("--platform\nlinux/arm64\n"));
        let push = fs::read_to_string(root.join("pushed")).unwrap();
        assert_eq!(
            push.contains("--all-platforms"),
            program == "nerdctl",
            "{push}"
        );
    }
}
