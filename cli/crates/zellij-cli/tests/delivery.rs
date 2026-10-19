//! Saved publication must resolve explicitly, build once and push only after a
//! successful build. No test contacts a registry or reads local credentials.
use std::{
    fs,
    path::Path,
    process::{Command, Output},
};

fn write(root: &Path, path: &str, body: &str) {
    let path = root.join(path);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, body).unwrap();
}
fn config(root: &Path, folder: &str, renderer: &str, settings: &str) {
    write(
        root,
        &format!("{folder}/zel.yaml"),
        &format!("version: 1\nrenderer: {renderer}\n{settings}"),
    );
    write(
        root,
        &format!("{folder}/content/index.md"),
        "# Publication\n",
    );
    write(
        root,
        &format!("{folder}/content/site.yaml"),
        "name: Hosted\n",
    );
}
fn command(root: &Path) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_zel"));
    command
        .current_dir(root)
        .env("PATH", root.join("no-tools"))
        .env("DOCKER_HOST", "unix:///fixture-zellij.sock");
    for key in [
        "DOCKER_CONTEXT",
        "CONTAINER_HOST",
        "CONTAINER_CONNECTION",
        "CONTAINERD_ADDRESS",
        "CONTAINERD_NAMESPACE",
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
const SETTINGS: &str =
    "image: {name: 'ghcr.io/acme/docs:v1'}\npublish: {action: push, image: static}\n";

#[test]
fn plans_use_conventions_custom_roots_and_explicit_overrides_without_tools_or_writes() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    config(root, "docs/zellij", "portable", SETTINGS);
    config(
        root,
        "docs/Manual 日本",
        "hosted",
        &SETTINGS.replace("image: static", "image: next"),
    );
    // Dry run is deliberately resolution only, not a content-validation claim.
    write(root, "docs/zellij/content/index.md", "[Broken](missing.md)");
    for args in [vec!["publish", "--dry-run"], vec!["pub", "--dry-run"]] {
        let result = run(root, &args, 0);
        let stdout = String::from_utf8_lossy(&result.stdout);
        assert!(
            stdout.contains("static") && stdout.contains("ghcr.io/acme/docs:v1"),
            "{stdout}"
        );
        assert!(stdout.contains("No content validation"));
        assert!(!root.join("docs/zellij/site").exists());
    }
    let result = run(
        root,
        &[
            "pub",
            "docs/Manual 日本",
            "--tag",
            "localhost:5000/team/docs:v2",
            "--dry-run",
        ],
        0,
    );
    let stdout = String::from_utf8_lossy(&result.stdout);
    assert!(stdout.contains("next") && stdout.contains("localhost:5000/team/docs:v2"));
    assert!(!stdout.contains("ghcr.io/acme/docs:v1"));
    run(&root.join("docs/zellij"), &["pub", "--dry-run"], 0);
    config(
        root,
        ".",
        "portable",
        &SETTINGS.replace("ghcr.io/acme/docs:v1", "acme/current:latest"),
    );
    let result = run(root, &["pub", "--dry-run"], 0);
    assert!(String::from_utf8_lossy(&result.stdout).contains("acme/current:latest"));
    assert_eq!(
        fs::read_to_string(root.join("docs/zellij/zel.yaml")).unwrap(),
        format!("version: 1\nrenderer: portable\n{SETTINGS}")
    );
}

#[test]
fn publication_requires_intent_and_destination_and_keeps_unsupported_options_inert() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    config(root, "docs/zellij", "portable", "");
    run(root, &["pub", "--tag", "acme/docs:v1", "--dry-run"], 2);
    config(
        root,
        "docs/zellij",
        "portable",
        "publish: {action: push, image: next}\n",
    );
    let result = run(root, &["pub", "--dry-run"], 2);
    assert!(String::from_utf8_lossy(&result.stderr).contains("explicit destination"));
    run(root, &["pub", "--tag", "acme/docs:v1", "--dry-run"], 0);
    config(root, "docs/zellij", "portable", SETTINGS);
    for args in [
        vec!["pub", "--to", "netlify"],
        vec!["pub", "--json"],
        vec!["pub", "--tag", "https://bad/reference"],
        vec!["pub", "--tag", "-malformed"],
    ] {
        run(root, &args, 2);
    }
    run(root, &["pub"], 4); // normal execution must preflight the runtime
    assert!(!root.join("docs/zellij/site").exists());
    run(root, &["pub", "docs/zellij/content", "--dry-run"], 2); // no legacy implicit intent
}

#[test]
fn delivery_configuration_is_typed_strict_and_versioned_for_both_renderers() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    for renderer in ["portable", "hosted"] {
        for settings in [
            "image: null",
            "image: {}",
            "image: {name: 123}",
            "image: {name: true}",
            "image: {name: ''}",
            "image: {name: 'acme/Bad'}",
            "image: {name: 'acme/docs', token: secret}",
            "publish: null",
            "publish: {}",
            "publish: {action: push}",
            "publish: {action: push, image: docker}",
            "publish: {action: push, image: true}",
            "publish: {action: push, image: static, provider: netlify}",
            "publish: {action: push, image: static, site: other}",
            "publish: {action: push, image: static, token: secret}",
            "publish: {action: deploy, provider: cloudflare, site: other}",
            "ci: {system: github}",
        ] {
            config(root, "manual", renderer, settings);
            run(root, &["pub", "manual", "--dry-run"], 2);
        }
    }
    write(
        root,
        "legacy/zel.yaml",
        "name: Legacy\nimage: {name: acme/docs}",
    );
    write(root, "legacy/index.md", "# Legacy\n");
    run(root, &["build", "legacy"], 2);
    // Portable rendering must not mistake shared delivery settings for its own.
    config(root, "docs/zellij", "portable", SETTINGS);
    fs::remove_file(root.join("docs/zellij/content/site.yaml")).unwrap();
    run(root, &["build"], 0);
    assert!(root.join("docs/zellij/site/index.html").is_file());
}

#[cfg(unix)]
mod execution {
    use super::*;
    use std::os::unix::fs::PermissionsExt;

    fn fixture(root: &Path) {
        config(root, "docs/zellij", "portable", SETTINGS);
        write(
            root,
            "docs/zellij/site/keep.txt",
            "authored existing output",
        );
        write(
            root,
            "bin/docker",
            r#"#!/bin/sh
printf '%s\n' "$1" >> "$FIXTURE_ROOT/calls"
case "$1" in
  info) exit 0;;
  --version) echo 'fixture docker'; exit 0;;
  build)
    printf '%s\n' "$@" > "$FIXTURE_ROOT/build-args"
    cat > "$FIXTURE_ROOT/recipe"
    for last do :; done
    printf '%s' "$last" > "$FIXTURE_ROOT/context"
    find "$last" -type f | sed "s|$last/||" | sort > "$FIXTURE_ROOT/context-files"
    exit "${FAKE_BUILD_EXIT:-0}";;
  push)
    printf '%s\n' "$@" > "$FIXTURE_ROOT/push-args"
    exit "${FAKE_PUSH_EXIT:-0}";;
  *) exit 99;;
esac
"#,
        );
        fs::set_permissions(root.join("bin/docker"), fs::Permissions::from_mode(0o755)).unwrap();
    }
    fn runtime(root: &Path) -> Command {
        let mut cmd = command(root);
        cmd.env(
            "PATH",
            format!("{}:/usr/bin:/bin", root.join("bin").display()),
        )
        .env("FIXTURE_ROOT", root)
        .env("ZELLIJ_IMAGE", "fixture/base:test");
        cmd
    }
    fn calls(root: &Path) -> String {
        fs::read_to_string(root.join("calls")).unwrap_or_default()
    }
    fn clear(root: &Path) {
        for file in [
            "calls",
            "build-args",
            "push-args",
            "context-files",
            "context",
        ] {
            let _ = fs::remove_file(root.join(file));
        }
    }
    fn preserved(root: &Path) {
        assert_eq!(
            fs::read_to_string(root.join("docs/zellij/site/keep.txt")).unwrap(),
            "authored existing output"
        );
        assert_eq!(
            fs::read_to_string(root.join("docs/zellij/content/index.md")).unwrap(),
            "# Publication\n"
        );
    }

    #[test]
    fn publishes_each_renderer_and_image_kind_once_with_the_exact_destination() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path();
        fixture(root);
        for renderer in ["portable", "hosted"] {
            for kind in ["static", "next"] {
                clear(root);
                config(
                    root,
                    "docs/zellij",
                    renderer,
                    &SETTINGS.replace("image: static", &format!("image: {kind}")),
                );
                let result = runtime(root)
                    .args(["pub", "--platform", "linux/arm64"])
                    .output()
                    .unwrap();
                assert!(result.status.success(), "{result:?}");
                assert!(
                    fs::read_to_string(root.join("build-args"))
                        .unwrap()
                        .contains("--platform\nlinux/arm64\n")
                );
                let calls = calls(root);
                for expected in ["info", "build", "push"] {
                    assert_eq!(
                        calls.lines().filter(|line| *line == expected).count(),
                        1,
                        "{calls}"
                    );
                }
                assert!(!calls.lines().any(|line| line == "tag"));
                assert_eq!(
                    fs::read_to_string(root.join("push-args")).unwrap(),
                    "push\nghcr.io/acme/docs:v1\n"
                );
                assert!(
                    fs::read_to_string(root.join("build-args"))
                        .unwrap()
                        .contains("-t\nghcr.io/acme/docs:v1\n")
                );
                assert!(
                    String::from_utf8_lossy(&result.stdout)
                        .contains("Published ghcr.io/acme/docs:v1")
                );
                let context = fs::read_to_string(root.join("context")).unwrap();
                assert_eq!(Path::new(&context).exists(), renderer == "hosted");
                let recipe = fs::read_to_string(root.join("recipe")).unwrap();
                assert_eq!(recipe.contains(".next/standalone"), kind == "next");
                if renderer == "portable" {
                    let files = fs::read_to_string(root.join("context-files")).unwrap();
                    assert!(
                        !files.contains("index.md")
                            && !files.contains("zel.yaml")
                            && !files.contains("keep.txt")
                    );
                }
                preserved(root);
            }
        }
    }

    #[test]
    fn local_images_honor_saved_names_and_flags_but_never_publish_implicitly() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path();
        fixture(root);
        for args in [
            vec!["si"],
            vec!["ni"],
            vec!["si", "--tag", "acme/override:v2"],
            vec!["ni", "--tag", "acme/override:v2"],
        ] {
            clear(root);
            let result = runtime(root).args(&args).output().unwrap();
            assert!(result.status.success(), "{result:?}");
            let expected = if args.len() == 1 {
                "ghcr.io/acme/docs:v1"
            } else {
                "acme/override:v2"
            };
            assert!(
                fs::read_to_string(root.join("build-args"))
                    .unwrap()
                    .contains(&format!("-t\n{expected}\n"))
            );
            assert!(!calls(root).lines().any(|line| line == "push"));
            preserved(root);
        }
        // Saved new-command defaults do not change compatibility spellings.
        config(root, "docs/zellij", "hosted", SETTINGS);
        for args in [vec!["image", "build"], vec!["build", "--target", "image"]] {
            clear(root);
            let result = runtime(root).args(args).output().unwrap();
            assert!(result.status.success(), "{result:?}");
            assert!(
                fs::read_to_string(root.join("build-args"))
                    .unwrap()
                    .contains("-t\nzellij:latest\n")
            );
            assert!(!calls(root).lines().any(|line| line == "push"));
        }
        config(root, "docs/zellij", "portable", SETTINGS);
        clear(root);
        let result = runtime(root)
            .args(["publish", "--tag", "acme/override:v2"])
            .output()
            .unwrap();
        assert!(result.status.success(), "{result:?}");
        assert_eq!(
            fs::read_to_string(root.join("push-args")).unwrap(),
            "push\nacme/override:v2\n"
        );
        assert!(
            fs::read_to_string(root.join("build-args"))
                .unwrap()
                .contains("-t\nacme/override:v2\n")
        );
    }

    #[test]
    fn failed_build_never_pushes_and_failed_push_reports_the_completed_stage() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path();
        fixture(root);
        for stage in ["BUILD", "PUSH"] {
            clear(root);
            let result = runtime(root)
                .args(["pub", "--platform", "linux/arm64"])
                .env(format!("FAKE_{stage}_EXIT"), "1")
                .output()
                .unwrap();
            assert_eq!(result.status.code(), Some(3), "{result:?}");
            assert!(!String::from_utf8_lossy(&result.stdout).contains("Published"));
            let stderr = String::from_utf8_lossy(&result.stderr);
            if stage == "BUILD" {
                assert!(!root.join("push-args").exists());
                assert!(stderr.contains("no push was attempted"), "{stderr}");
                assert!(
                    stderr.contains("Image build for linux/arm64 failed")
                        && stderr.contains("emulation"),
                    "{stderr}"
                );
            } else {
                assert!(
                    stderr.contains("local image remains")
                        && stderr.contains("zel image push \"ghcr.io/acme/docs:v1\""),
                    "{stderr}"
                );
                assert!(
                    stderr.contains("container image push failed")
                        && !stderr.contains("content folder is not valid"),
                    "{stderr}"
                );
            }
            let context = fs::read_to_string(root.join("context")).unwrap();
            assert!(!Path::new(&context).exists());
            preserved(root);
        }
        clear(root);
        write(
            root,
            "docs/zellij/content/index.md",
            "[Missing](missing.md)",
        );
        let result = runtime(root)
            .args(["pub", "--platform", "linux/arm64"])
            .output()
            .unwrap();
        assert!(!result.status.success());
        assert!(!root.join("build-args").exists() && !root.join("push-args").exists());
    }

    #[test]
    fn rejected_arguments_and_dry_runs_do_not_invoke_any_runtime_command() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path();
        fixture(root);
        for (args, code) in [
            (vec!["pub", "--dry-run"], 0),
            (vec!["pub", "--json"], 2),
            (vec!["pub", "--to", "netlify"], 2),
            (vec!["si", "--tag", "x\nINJECTED"], 2),
            (vec!["ni", "--tag", "user:secret@registry/docs"], 2),
        ] {
            let result = runtime(root).args(args).output().unwrap();
            assert_eq!(result.status.code(), Some(code), "{result:?}");
            assert!(calls(root).is_empty());
            assert!(!String::from_utf8_lossy(&result.stderr).contains("secret@"));
            preserved(root);
        }
    }
}

#[test]
fn platform_plans_validate_override_and_reject_irrelevant_targets_without_tools() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    config(
        root,
        "docs/zellij",
        "portable",
        &SETTINGS.replace("name:", "platform: linux/arm64, name:"),
    );
    let saved = run(root, &["pub", "--dry-run"], 0);
    assert!(String::from_utf8_lossy(&saved.stdout).contains("linux/arm64"));
    let changed = run(
        root,
        &["deploy", "--platform", "linux/amd64", "--dry-run"],
        0,
    );
    assert!(String::from_utf8_lossy(&changed.stdout).contains("linux/amd64"));
    // Saved image architecture does not affect an explicit HTML deployment.
    run(root, &["netlify", "--site-id", "fixture", "--dry-run"], 0);
    for args in [
        vec![
            "netlify",
            "--site-id",
            "fixture",
            "--platform",
            "linux/arm64",
            "--dry-run",
        ],
        vec!["build", "--platform", "linux/amd64"],
        vec!["si", "--platform", "linux/amd64,linux/arm64"],
        vec!["ni", "--platform", "windows/amd64"],
        vec!["pub", "--platform", "arm64", "--dry-run"],
    ] {
        run(root, &args, 2);
        assert!(!root.join("docs/zellij/site").exists());
    }
    for invalid in [
        "linux/amd64,linux/arm64",
        "windows/amd64",
        "true",
        "42",
        "[]",
    ] {
        config(
            root,
            "docs/zellij",
            "portable",
            &format!("image: {{platform: {invalid}}}\n"),
        );
        run(root, &["si"], 2);
    }
    config(
        root,
        "docs/zellij",
        "portable",
        "image: {platform: linux/arm64}\npublish: {action: push, image: static}\n",
    );
    let missing_tag = run(root, &["pub", "--dry-run"], 2);
    assert!(String::from_utf8_lossy(&missing_tag.stderr).contains("explicit destination"));
    run(root, &["pub", "--tag", "acme/docs:v1", "--dry-run"], 0);
}
