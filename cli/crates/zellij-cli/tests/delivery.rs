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
        "RAILWAY_TOKEN",
        "RAILWAY_PROJECT_ID",
        "RAILWAY_ENVIRONMENT_ID",
        "RAILWAY_SERVICE_ID",
        "FLY_APP",
        "FLY_ORG",
        "FLY_API_TOKEN",
        "DOCKER_CONTEXT",
        "CONTAINER_HOST",
        "CONTAINER_CONNECTION",
        "CONTAINERD_ADDRESS",
        "CONTAINERD_NAMESPACE",
        "VERCEL_TOKEN",
        "VERCEL_PROJECT_ID",
        "VERCEL_TEAM_ID",
        "GITHUB_PAGES_TOKEN",
        "GITHUB_PAGES_REPOSITORY",
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
            "publish: {action: deploy, provider: fly, site: other}",
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
    if [ -f "$last/index.html" ]; then cat "$last/index.html" > "$FIXTURE_ROOT/built.html"; fi
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
    fn feature_selection_reaches_each_image_build_without_changing_saved_settings() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path();
        fixture(root);
        let markdown = "# Publication\n\n$x^2$\n\n```mermaid\ngraph TD; A-->B;\n```\n";
        for renderer in ["portable", "hosted"] {
            for kind in ["static", "next"] {
                for selection in ["all", "none"] {
                    clear(root);
                    let saved = selection == "none";
                    let settings = format!(
                        "{}{}",
                        SETTINGS.replace("image: static", &format!("image: {kind}")),
                        if renderer == "portable" {
                            format!("markdown: {{math: {saved}, mermaid: {saved}}}\n")
                        } else {
                            String::new()
                        }
                    );
                    config(root, "docs/zellij", renderer, &settings);
                    write(root, "docs/zellij/content/index.md", markdown);
                    let config_before = fs::read(root.join("docs/zellij/zel.yaml")).unwrap();
                    let result = runtime(root)
                        .args(["pub", "--features", selection])
                        .output()
                        .unwrap();
                    assert!(result.status.success(), "{result:?}");
                    let recipe = fs::read_to_string(root.join("recipe")).unwrap();
                    if renderer == "portable" && kind == "static" {
                        let html = fs::read_to_string(root.join("built.html")).unwrap();
                        assert_eq!(html.contains("<math"), !saved);
                        assert_eq!(html.contains("class=\"mermaid-diagram\""), !saved);
                    } else {
                        let canonical = if saved { "none" } else { "maths,diagrams" };
                        assert!(
                            recipe.contains(&format!("features \"{canonical}\" /app")),
                            "{recipe}"
                        );
                    }
                    assert_eq!(
                        calls(root).lines().filter(|line| *line == "build").count(),
                        1
                    );
                    assert_eq!(
                        calls(root).lines().filter(|line| *line == "push").count(),
                        1
                    );
                    assert_eq!(
                        fs::read(root.join("docs/zellij/zel.yaml")).unwrap(),
                        config_before
                    );
                    assert_eq!(
                        fs::read_to_string(root.join("docs/zellij/content/index.md")).unwrap(),
                        markdown
                    );
                    assert_eq!(
                        fs::read_to_string(root.join("docs/zellij/site/keep.txt")).unwrap(),
                        "authored existing output"
                    );
                }
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

#[test]
fn cloudflare_defaults_aliases_overrides_and_provider_isolation() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    config(
        root,
        "docs/zellij",
        "portable",
        "publish: {action: deploy, provider: cloudflare, site: saved-project}\n",
    );
    for route in [
        vec!["publish"],
        vec!["deploy"],
        vec!["deploy", "cloudflare"],
        vec!["cloudflare"],
        vec!["cf"],
    ] {
        let mut cmd = command(root);
        cmd.env_remove("CLOUDFLARE_PAGES_PROJECT_NAME")
            .env_remove("CLOUDFLARE_API_TOKEN")
            .env_remove("CLOUDFLARE_ACCOUNT_ID");
        let output = cmd.args(route).arg("--dry-run").output().unwrap();
        assert!(output.status.success(), "{output:?}");
        let text = String::from_utf8_lossy(&output.stdout);
        assert!(text.contains("saved-project") && text.contains("zel-preview"));
        assert!(!root.join("docs/zellij/site").exists());
    }
    let output = command(root)
        .env("CLOUDFLARE_PAGES_PROJECT_NAME", "environment-project")
        .args(["cf", "--site", "explicit-project", "--prod", "--dry-run"])
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("explicit-project"));
    assert!(!String::from_utf8_lossy(&output.stdout).contains("environment-project"));
    for flags in [
        vec!["--platform", "linux/amd64"],
        vec!["--tag", "acme/image:v1"],
        vec!["--to", "netlify"],
        vec!["--site", "https://manual.pages.dev"],
    ] {
        let mut cmd = command(root);
        cmd.args(["cf", "--dry-run"]).args(flags);
        assert_eq!(cmd.output().unwrap().status.code(), Some(2));
    }
    config(
        root,
        "docs/zellij",
        "portable",
        "publish: {action: deploy, provider: netlify, site: old-site}\n",
    );
    let output = command(root)
        .env_remove("CLOUDFLARE_PAGES_PROJECT_NAME")
        .env("NETLIFY_SITE_ID", "old-env-site")
        .args(["cf", "--dry-run"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("CLOUDFLARE_PAGES_PROJECT_NAME"));
    config(root, "docs/zellij", "portable", SETTINGS);
    assert_eq!(
        command(root)
            .args(["pub", "--to", "cloudflare", "--site", "manual", "--dry-run"])
            .output()
            .unwrap()
            .status
            .code(),
        Some(2)
    );
}

#[test]
fn cloudflare_missing_credentials_stop_before_build_or_tools() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    config(root, "docs/zellij", "hosted", "");
    let result = command(root)
        .env_remove("CLOUDFLARE_API_TOKEN")
        .env_remove("CLOUDFLARE_ACCOUNT_ID")
        .args(["cf", "--site", "manual"])
        .output()
        .unwrap();
    assert_eq!(result.status.code(), Some(2));
    let error = String::from_utf8_lossy(&result.stderr);
    assert!(error.contains("CLOUDFLARE_API_TOKEN"), "{error}");
    assert!(!root.join("docs/zellij/site").exists());
}

#[test]
fn vercel_defaults_aliases_overrides_and_provider_isolation() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    config(
        root,
        "docs/zellij",
        "portable",
        "publish: {action: deploy, provider: vercel, site: prj_Saved123}\n",
    );
    for route in [
        vec!["publish"],
        vec!["deploy"],
        vec!["deploy", "vercel"],
        vec!["vercel"],
        vec!["vc"],
    ] {
        let mut cmd = command(root);
        cmd.env_remove("VERCEL_PROJECT_ID")
            .env_remove("VERCEL_TOKEN")
            .env_remove("VERCEL_TEAM_ID");
        let output = cmd.args(route).arg("--dry-run").output().unwrap();
        assert!(output.status.success(), "{output:?}");
        let text = String::from_utf8_lossy(&output.stdout);
        assert!(text.contains("prj_Saved123") && text.contains("preview"));
        assert!(!root.join("docs/zellij/site").exists());
    }
    let output = command(root)
        .env("VERCEL_PROJECT_ID", "environment-project")
        .args(["vc", "--site", "explicit-project", "--prod", "--dry-run"])
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("explicit-project"));
    assert!(!String::from_utf8_lossy(&output.stdout).contains("environment-project"));
    for flags in [
        vec!["--platform", "linux/amd64"],
        vec!["--tag", "acme/image:v1"],
        vec!["--to", "netlify"],
        vec!["--site", "https://manual.vercel.app"],
    ] {
        let mut cmd = command(root);
        cmd.args(["vc", "--dry-run"]).args(flags);
        assert_eq!(cmd.output().unwrap().status.code(), Some(2));
    }
    config(
        root,
        "docs/zellij",
        "portable",
        "publish: {action: deploy, provider: netlify, site: old-site}\n",
    );
    let output = command(root)
        .env_remove("VERCEL_PROJECT_ID")
        .env("NETLIFY_SITE_ID", "old-env-site")
        .args(["vc", "--dry-run"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("VERCEL_PROJECT_ID"));
    config(root, "docs/zellij", "portable", SETTINGS);
    assert_eq!(
        command(root)
            .args(["pub", "--to", "vercel", "--site", "manual", "--dry-run"])
            .output()
            .unwrap()
            .status
            .code(),
        Some(2)
    );
}

#[test]
fn vercel_missing_credentials_stop_before_build_or_tools() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    config(root, "docs/zellij", "hosted", "");
    let result = command(root)
        .env_remove("VERCEL_TOKEN")
        .env_remove("VERCEL_TEAM_ID")
        .args(["vc", "--site", "manual"])
        .output()
        .unwrap();
    assert_eq!(result.status.code(), Some(2));
    let error = String::from_utf8_lossy(&result.stderr);
    assert!(error.contains("VERCEL_TOKEN"), "{error}");
    assert!(!root.join("docs/zellij/site").exists());
}

#[test]
fn github_pages_defaults_aliases_overrides_and_provider_isolation() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    config(
        root,
        "docs/zellij",
        "portable",
        "publish: {action: deploy, provider: github-pages, site: acme/saved}\n",
    );
    for route in [
        vec!["publish"],
        vec!["deploy"],
        vec!["deploy", "github-pages"],
        vec!["github-pages"],
        vec!["ghp"],
    ] {
        let mut cmd = command(root);
        cmd.env_remove("GITHUB_PAGES_REPOSITORY")
            .env_remove("AWS_ACCESS_KEY_ID")
            .env_remove("AWS_SECRET_ACCESS_KEY")
            .env_remove("AWS_SESSION_TOKEN")
            .env_remove("AWS_REGION")
            .env_remove("S3_EXPECTED_BUCKET_OWNER")
            .env_remove("S3_DESTINATION")
            .env_remove("AWS_ENDPOINT_URL")
            .env_remove("AWS_ENDPOINT_URL_S3")
            .env_remove("GITHUB_PAGES_TOKEN");
        let output = cmd
            .args(route)
            .args(["--prod", "--dry-run"])
            .output()
            .unwrap();
        assert!(output.status.success(), "{output:?}");
        let text = String::from_utf8_lossy(&output.stdout);
        assert!(text.contains("acme/saved") && text.contains("production"));
        assert!(!root.join("docs/zellij/site").exists());
    }
    let output = command(root)
        .env("GITHUB_PAGES_REPOSITORY", "acme/environment")
        .args(["ghp", "--site", "acme/explicit", "--prod", "--dry-run"])
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("acme/explicit"));
    assert!(!String::from_utf8_lossy(&output.stdout).contains("acme/environment"));
    for flags in [
        vec!["--platform", "linux/amd64"],
        vec!["--tag", "acme/image:v1"],
        vec!["--to", "netlify"],
        vec!["--site", "https://github.com/acme/manual"],
    ] {
        let mut cmd = command(root);
        cmd.args(["ghp", "--prod", "--dry-run"]).args(flags);
        assert_eq!(cmd.output().unwrap().status.code(), Some(2));
    }
    config(
        root,
        "docs/zellij",
        "portable",
        "publish: {action: deploy, provider: netlify, site: old-site}\n",
    );
    let output = command(root)
        .env_remove("GITHUB_PAGES_REPOSITORY")
        .env("NETLIFY_SITE_ID", "old-env-site")
        .args(["ghp", "--prod", "--dry-run"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("GITHUB_PAGES_REPOSITORY"));
    config(root, "docs/zellij", "portable", SETTINGS);
    assert_eq!(
        command(root)
            .args([
                "pub",
                "--to",
                "github-pages",
                "--site",
                "acme/manual",
                "--dry-run"
            ])
            .output()
            .unwrap()
            .status
            .code(),
        Some(2)
    );
}

#[test]
fn github_pages_missing_credentials_stop_before_build_or_tools() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    config(root, "docs/zellij", "hosted", "");
    let result = command(root)
        .env_remove("GITHUB_PAGES_TOKEN")
        .args(["ghp", "--site", "acme/manual", "--prod"])
        .output()
        .unwrap();
    assert_eq!(result.status.code(), Some(2));
    let error = String::from_utf8_lossy(&result.stderr);
    assert!(error.contains("GITHUB_PAGES_TOKEN"), "{error}");
    assert!(!root.join("docs/zellij/site").exists());
}

#[test]
fn github_pages_requires_production_even_for_saved_publication_and_dry_runs() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    config(
        root,
        "docs/zellij",
        "portable",
        "publish: {action: deploy, provider: github-pages, site: acme/manual}\n",
    );
    for route in [vec!["ghp"], vec!["pub"], vec!["deploy", "github-pages"]] {
        let output = command(root).args(route).arg("--dry-run").output().unwrap();
        assert_eq!(output.status.code(), Some(2));
        assert!(String::from_utf8_lossy(&output.stderr).contains("--prod"));
    }
    for repository in [
        "manual",
        "acme/manual/extra",
        "../manual",
        "-acme/manual",
        "acme/..",
        "acme/manual.git",
        "acme/bad repo",
    ] {
        let output = command(root)
            .args(["ghp", "--prod", "--dry-run", "--site", repository])
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(2), "{repository}");
    }
    assert!(!root.join("docs/zellij/site").exists());
}

#[test]
fn s3_defaults_aliases_overrides_and_provider_isolation() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    config(
        root,
        "docs/zellij",
        "portable",
        "publish: {action: deploy, provider: s3, site: acme-docs/saved}\n",
    );
    for route in [
        vec!["publish"],
        vec!["deploy"],
        vec!["deploy", "s3"],
        vec!["s3"],
    ] {
        let mut cmd = command(root);
        cmd.env_remove("S3_DESTINATION")
            .env_remove("AWS_ACCESS_KEY_ID")
            .env_remove("AWS_SECRET_ACCESS_KEY")
            .env_remove("AWS_SESSION_TOKEN")
            .env_remove("AWS_REGION")
            .env_remove("S3_EXPECTED_BUCKET_OWNER")
            .env_remove("S3_DESTINATION")
            .env_remove("AWS_ENDPOINT_URL")
            .env_remove("AWS_ENDPOINT_URL_S3")
            .env_remove("AWS_ACCESS_KEY_ID");
        let output = cmd
            .args(route)
            .args(["--prod", "--dry-run"])
            .output()
            .unwrap();
        assert!(output.status.success(), "{output:?}");
        let text = String::from_utf8_lossy(&output.stdout);
        assert!(text.contains("acme-docs/saved") && text.contains("production"));
        assert!(!root.join("docs/zellij/site").exists());
    }
    let output = command(root)
        .env("S3_DESTINATION", "acme-docs/environment")
        .args(["s3", "--site", "acme-docs/explicit", "--prod", "--dry-run"])
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("acme-docs/explicit"));
    assert!(!String::from_utf8_lossy(&output.stdout).contains("acme-docs/environment"));
    for flags in [
        vec!["--platform", "linux/amd64"],
        vec!["--tag", "acme-docs/image:v1"],
        vec!["--to", "netlify"],
        vec!["--site", "https://github.com/acme-docs/manual"],
    ] {
        let mut cmd = command(root);
        cmd.args(["s3", "--prod", "--dry-run"]).args(flags);
        assert_eq!(cmd.output().unwrap().status.code(), Some(2));
    }
    config(
        root,
        "docs/zellij",
        "portable",
        "publish: {action: deploy, provider: netlify, site: old-site}\n",
    );
    let output = command(root)
        .env_remove("S3_DESTINATION")
        .env("NETLIFY_SITE_ID", "old-env-site")
        .args(["s3", "--prod", "--dry-run"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("S3_DESTINATION"));
    config(root, "docs/zellij", "portable", SETTINGS);
    assert_eq!(
        command(root)
            .args([
                "pub",
                "--to",
                "s3",
                "--site",
                "acme-docs/manual",
                "--dry-run"
            ])
            .output()
            .unwrap()
            .status
            .code(),
        Some(2)
    );
}

#[test]
fn s3_missing_credentials_stop_before_build_or_tools() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    config(root, "docs/zellij", "hosted", "");
    let result = command(root)
        .env_remove("AWS_ACCESS_KEY_ID")
        .args(["s3", "--site", "acme-docs/manual", "--prod"])
        .output()
        .unwrap();
    assert_eq!(result.status.code(), Some(2));
    let error = String::from_utf8_lossy(&result.stderr);
    assert!(error.contains("AWS_ACCESS_KEY_ID"), "{error}");
    assert!(!root.join("docs/zellij/site").exists());
}

#[test]
fn s3_requires_production_even_for_saved_publication_and_dry_runs() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    config(
        root,
        "docs/zellij",
        "portable",
        "publish: {action: deploy, provider: s3, site: acme-docs/manual}\n",
    );
    for route in [vec!["s3"], vec!["pub"], vec!["deploy", "s3"]] {
        let output = command(root).args(route).arg("--dry-run").output().unwrap();
        assert_eq!(output.status.code(), Some(2));
        assert!(String::from_utf8_lossy(&output.stderr).contains("--prod"));
    }
    for repository in [
        "UPPER",
        "acme-docs/../extra",
        "../manual",
        "-acme-docs/manual",
        "acme-docs/..",
        "acme-docs/",
        "acme-docs/bad repo",
    ] {
        let output = command(root)
            .args(["s3", "--prod", "--dry-run", "--site", repository])
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(2), "{repository}");
    }
    assert!(!root.join("docs/zellij/site").exists());
}

#[test]
fn s3_environment_validation_never_starts_a_build_or_uses_custom_endpoints() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    config(root, "docs/zellij", "hosted", "");
    for (key, value, expected) in [
        (
            "AWS_ENDPOINT_URL",
            "https://private.invalid",
            "AWS_ENDPOINT_URL",
        ),
        (
            "AWS_ENDPOINT_URL_S3",
            "https://private.invalid",
            "AWS_ENDPOINT_URL_S3",
        ),
        ("AWS_REGION", "cn-north-1", "AWS_REGION"),
        (
            "S3_EXPECTED_BUCKET_OWNER",
            "bad-owner",
            "S3_EXPECTED_BUCKET_OWNER",
        ),
    ] {
        let result = command(root)
            .env("AWS_ACCESS_KEY_ID", "AKIAIOSFODNN7EXAMPLE")
            .env("AWS_SECRET_ACCESS_KEY", "fixture-secret")
            .env("AWS_REGION", "us-east-1")
            .env("S3_EXPECTED_BUCKET_OWNER", "123456789012")
            .env(key, value)
            .args(["s3", "--site", "acme-docs/manual", "--prod"])
            .output()
            .unwrap();
        assert_eq!(result.status.code(), Some(2), "{result:?}");
        let e = String::from_utf8_lossy(&result.stderr);
        assert!(
            e.contains(expected) && !e.contains("fixture-secret") && !e.contains("private.invalid"),
            "{e}"
        );
        assert!(!root.join("docs/zellij/site").exists());
    }
}

#[test]
fn fly_plans_select_image_kind_and_require_explicit_production_without_tools() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    config(
        root,
        "docs/zellij",
        "portable",
        "publish: {action: deploy, provider: fly, site: saved-app, image: next}\nimage: {platform: all}\n",
    );
    for route in [vec!["fly"], vec!["pub"], vec!["deploy", "fly"]] {
        let mut args = route.clone();
        args.push("--dry-run");
        let output = run(root, &args, 2);
        assert!(String::from_utf8_lossy(&output.stderr).contains("--prod"));
        args.push("--prod");
        let output = run(root, &args, 0);
        let out = String::from_utf8_lossy(&output.stdout);
        assert!(
            out.contains("saved-app") && out.contains("next") && out.contains("linux/amd64"),
            "{out}"
        );
    }
    let output = command(root)
        .env("FLY_APP", "env-app")
        .args([
            "fly",
            "--site",
            "explicit-app",
            "--kind",
            "static",
            "--prod",
            "--dry-run",
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let out = String::from_utf8_lossy(&output.stdout);
    assert!(
        out.contains("explicit-app")
            && out.contains("static")
            && !out.contains("saved-app")
            && !out.contains("env-app")
    );
    for flags in [
        vec!["--platform", "all"],
        vec!["--platform", "linux/arm64"],
        vec!["--tag", "my-image"],
        vec!["--site", "https://manual.fly.dev"],
    ] {
        let mut args = vec!["fly", "--prod", "--dry-run"];
        args.extend(flags);
        run(root, &args, 2);
    }
    run(
        root,
        &[
            "fly",
            "--prod",
            "--dry-run",
            "--platform",
            "linux/amd64",
            "--kind",
            "next",
            "--base-path",
            "/manual",
        ],
        0,
    );
    run(
        root,
        &[
            "fly",
            "--prod",
            "--dry-run",
            "--kind",
            "static",
            "--base-path",
            "/manual",
        ],
        2,
    );
    run(
        root,
        &[
            "deploy",
            "netlify",
            "--site",
            "other-site",
            "--kind",
            "next",
            "--dry-run",
        ],
        2,
    );
    let output = run(root, &["fly", "--prod"], 2);
    assert!(String::from_utf8_lossy(&output.stderr).contains("FLY_API_TOKEN"));
    assert!(!root.join("docs/zellij/site").exists());
}
#[test]
fn deployment_image_kind_is_only_valid_for_fly() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    config(
        root,
        "docs/zellij",
        "portable",
        "publish: {action: deploy, provider: netlify, site: manual, image: next}\n",
    );
    let output = run(root, &["pub", "--dry-run"], 2);
    assert!(String::from_utf8_lossy(&output.stderr).contains("publish.image"));
}

#[test]
fn railway_delivery_plans_resolve_kind_registry_and_scope_without_tools() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    let service = "33333333-3333-4333-8333-333333333333";
    config(
        root,
        "docs/zellij",
        "portable",
        &format!(
            "publish: {{action: deploy, provider: railway, site: {service}, image: next}}\nimage: {{name: 'ghcr.io/acme/docs:v1', platform: all}}\n"
        ),
    );
    for route in [
        vec!["railway"],
        vec!["rw"],
        vec!["deploy", "railway"],
        vec!["pub"],
    ] {
        let mut args = route;
        args.push("--dry-run");
        run(root, &args, 2);
        args.push("--prod");
        let out = run(root, &args, 0);
        let text = String::from_utf8_lossy(&out.stdout);
        assert!(
            text.contains(service)
                && text.contains("next")
                && text.contains("ghcr.io/acme/docs:v1")
                && text.contains("linux/amd64"),
            "{text}"
        );
    }
    let out = run(
        root,
        &[
            "rw",
            "--prod",
            "--dry-run",
            "--kind",
            "static",
            "--tag",
            "docker.io/acme/docs:v2",
        ],
        0,
    );
    assert!(String::from_utf8_lossy(&out.stdout).contains("docker.io/acme/docs:v2"));
    for flags in [
        vec!["--platform", "all"],
        vec!["--platform", "linux/arm64"],
        vec!["--site", "name"],
        vec!["--tag", "local-image"],
        vec!["--tag", "localhost:5000/acme/docs:v1"],
        vec!["--tag", "ghcr.io/acme/docs"],
    ] {
        let mut args = vec!["rw", "--prod", "--dry-run"];
        args.extend(flags);
        run(root, &args, 2);
    }
    run(
        root,
        &[
            "rw",
            "--prod",
            "--dry-run",
            "--kind",
            "next",
            "--base-path",
            "/manual",
        ],
        0,
    );
    run(
        root,
        &[
            "rw",
            "--prod",
            "--dry-run",
            "--kind",
            "static",
            "--base-path",
            "/manual",
        ],
        2,
    );
    let output = command(root)
        .env_remove("RAILWAY_TOKEN")
        .args(["rw", "--prod"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("RAILWAY_TOKEN"));
    assert!(!root.join("docs/zellij/site").exists());
    config(
        root,
        "docs/zellij",
        "portable",
        &format!("publish: {{action: deploy, provider: railway, site: {service}}}\n"),
    );
    run(root, &["rw", "--prod", "--dry-run"], 2);
    run(
        root,
        &["rw", "--prod", "--dry-run", "--tag", "ghcr.io/acme/docs:v3"],
        0,
    );
}
