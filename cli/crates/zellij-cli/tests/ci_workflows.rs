//! Local-only generation in disposable repositories; no real provisioning.
use serde_json::Value;
use std::{
    fs,
    path::Path,
    process::{Command, Output},
};
fn write(root: &Path, path: &str, value: &str) {
    let target = root.join(path);
    fs::create_dir_all(target.parent().unwrap()).unwrap();
    fs::write(target, value).unwrap();
}
fn repo(root: &Path) {
    assert!(
        Command::new("git")
            .args(["init", "--quiet"])
            .current_dir(root)
            .status()
            .unwrap()
            .success()
    );
}
fn site(root: &Path, dir: &str, settings: &str) {
    write(
        root,
        &format!("{dir}/zel.yaml"),
        &format!("version: 1\nrenderer: portable\nname: Manual\n{settings}"),
    );
    write(
        root,
        &format!("{dir}/content/index.md"),
        "# Guide\n\n[Next](next.md)\n",
    );
    write(root, &format!("{dir}/content/next.md"), "# Next\n");
}
fn command(root: &Path, args: &[&str]) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_zel"));
    command
        .current_dir(root)
        .args(args)
        .env_remove("NETLIFY_AUTH_TOKEN")
        .env_remove("NETLIFY_SITE_ID");
    command
}
fn run(root: &Path, args: &[&str], code: i32) -> Output {
    let result = command(root, args).output().unwrap();
    assert_eq!(result.status.code(), Some(code), "{args:?}: {result:?}");
    result
}
fn workflow(root: &Path) -> Value {
    serde_yaml_ng::from_str(&fs::read_to_string(root.join(".github/workflows/docs.yml")).unwrap())
        .unwrap()
}
#[test]
fn shorthand_full_form_defaults_nested_selection_and_reserved_names() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    repo(root);
    site(root, "docs/zellij", "ci: {zel-version: 0.2.0-rc.1}\n");
    run(root, &["ci"], 0);
    let original = fs::read(root.join(".github/workflows/docs.yml")).unwrap();
    run(root, &["ci", "init", "github", "--force"], 0);
    assert_eq!(
        fs::read(root.join(".github/workflows/docs.yml")).unwrap(),
        original
    );
    run(&root.join("docs/zellij"), &["ci", "--force"], 0);
    assert_eq!(
        fs::read(root.join(".github/workflows/docs.yml")).unwrap(),
        original
    );
    assert!(!root.join("docs/zellij/.github").exists());
    for dir in ["run", "publish", "init"] {
        site(root, dir, "ci: {zel-version: 0.2.0}\n");
        run(root, &["ci", &format!("./{dir}"), "--force"], 0);
        let a = workflow(root);
        run(root, &["ci", "--site-root", dir, "--force"], 0);
        assert_eq!(workflow(root), a);
    }
    run(
        root,
        &[
            "ci",
            "init",
            "github",
            "--content",
            "docs/zellij",
            "--force",
            "--no-provision",
        ],
        0,
    );
    assert_eq!(
        fs::read(root.join(".github/workflows/docs.yml")).unwrap(),
        original
    );
    run(root, &["ci", "docs/zellij", "--site-root", "run"], 2);
    run(root, &["ci", "--provision", "--no-provision"], 2);
    run(root, &["ci", "init", "gitlab", "--force"], 2);
}
#[test]
fn missing_incompatible_and_invalid_settings_do_not_replace_a_workflow() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    repo(root);
    site(root, "docs/zellij", "");
    run(root, &["ci"], 2);
    assert!(!root.join(".github").exists());
    run(root, &["ci", "--zel-version", "0.2.0"], 0);
    let original = fs::read(root.join(".github/workflows/docs.yml")).unwrap();
    for args in [
        vec!["ci", "--zel-version", "0.1.3", "--force"],
        vec!["ci", "--zel-version", "0.2.0"],
        vec!["ci", "--zel-version", "0.2.0", "--force", "--json"],
        vec![
            "ci",
            "--zel-version",
            "0.2.0",
            "--site-id",
            "unused",
            "--force",
        ],
    ] {
        run(root, &args, 2);
        assert_eq!(
            fs::read(root.join(".github/workflows/docs.yml")).unwrap(),
            original
        );
    }
    for settings in [
        "ci: {triggers: [pull_request_target]}",
        "ci: {branch: 'main*'}",
        "ci: {token: secret}",
        "ci: {workflow: ../other.yml}",
        "ci: {production: true}",
    ] {
        site(root, "docs/zellij", settings);
        run(root, &["ci", "--zel-version", "0.2.0", "--force"], 2);
        assert_eq!(
            fs::read(root.join(".github/workflows/docs.yml")).unwrap(),
            original
        );
    }
    site(root, "docs/expr ${{ secrets.VALUE }}", "");
    run(
        root,
        &[
            "ci",
            "docs/expr ${{ secrets.VALUE }}",
            "--zel-version",
            "0.2.0",
            "--force",
        ],
        2,
    );
}
#[test]
fn saved_delivery_and_legacy_overrides_produce_distinct_artifacts() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    repo(root);
    for kind in ["static", "next"] {
        site(
            root,
            "docs/zellij",
            &format!(
                "image: {{name: registry.example/acme/docs:v2}}\npublish: {{action: push, image: {kind}}}\nci: {{zel-version: 0.2.0}}\n"
            ),
        );
        run(root, &["ci", "--force"], 0);
        let doc = workflow(root);
        let preview = doc["jobs"]["verify"]["steps"]
            .as_array()
            .unwrap()
            .last()
            .unwrap();
        assert_eq!(
            preview["run"],
            format!("zel image {kind} \"$SITE_ROOT\" --tag zel-ci-check:local")
        );
        let publish = doc["jobs"]["publish"]["steps"].as_array().unwrap();
        assert_eq!(
            publish.last().unwrap()["run"],
            "zel publish \"$SITE_ROOT\" --tag \"$IMAGE_TAG\""
        );
        assert_eq!(
            publish.last().unwrap()["env"]["IMAGE_TAG"],
            "registry.example/acme/docs:v2"
        );
        assert!(doc.to_string().contains("secrets.REGISTRY_PASSWORD"));
        run(root, &["ci", "--target", "static", "--force"], 0);
        assert!(workflow(root)["jobs"].get("publish").is_none());
    }
    site(
        root,
        "docs/zellij",
        "publish: {action: deploy, provider: netlify, site: saved-id}\nci: {zel-version: 0.2.0}\n",
    );
    let result = command(root, &["ci", "--force"])
        .env("NETLIFY_AUTH_TOKEN", "must-not-be-written")
        .output()
        .unwrap();
    assert!(result.status.success(), "{result:?}");
    let doc = workflow(root);
    let steps = doc["jobs"]["publish"]["steps"].as_array().unwrap();
    assert_eq!(steps.last().unwrap()["env"]["NETLIFY_SITE_ID"], "saved-id");
    assert!(!doc.to_string().contains("must-not-be-written"));
    assert!(
        !steps.last().unwrap()["run"]
            .as_str()
            .unwrap()
            .contains("--prod")
    );
    let result = command(root, &["ci", "--force", "--site", "flag-id", "--prod"])
        .env("NETLIFY_SITE_ID", "environment-id")
        .output()
        .unwrap();
    assert!(result.status.success(), "{result:?}");
    let doc = workflow(root);
    let steps = doc["jobs"]["publish"]["steps"].as_array().unwrap();
    assert_eq!(steps.last().unwrap()["env"]["NETLIFY_SITE_ID"], "flag-id");
    assert!(
        steps.last().unwrap()["run"]
            .as_str()
            .unwrap()
            .ends_with("--prod")
    );
    run(root, &["ci", "--target", "image", "--force"], 0);
    assert!(workflow(root).to_string().contains("REPOSITORY,,"));
    run(
        root,
        &["ci", "--deploy", "netlify", "--target", "image", "--force"],
        2,
    );
}
#[cfg(unix)]
#[test]
fn generated_portable_steps_execute_with_spaces_quotes_and_shell_characters() {
    use std::os::unix::fs::symlink;
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    repo(root);
    let dir = "docs/Team's [manual] 日本 $(touch ESCAPED)";
    site(root, dir, "ci: {zel-version: 0.2.0}\n");
    run(root, &["ci", dir], 0);
    let doc = workflow(root);
    fs::create_dir(root.join("bin")).unwrap();
    symlink(env!("CARGO_BIN_EXE_zel"), root.join("bin/zel")).unwrap();
    let output = tempfile::tempdir().unwrap();
    for step in doc["jobs"]["verify"]["steps"].as_array().unwrap() {
        if let Some(script) = step["run"].as_str().filter(|s| s.starts_with("zel ")) {
            let result = Command::new("/bin/bash")
                .args(["-e", "-c", script])
                .current_dir(root)
                .env("SITE_ROOT", doc["env"]["SITE_ROOT"].as_str().unwrap())
                .env("RUNNER_TEMP", output.path())
                .env("PATH", root.join("bin"))
                .output()
                .unwrap();
            assert!(result.status.success(), "{script}: {result:?}");
        }
    }
    assert!(output.path().join("zellij-site/index.html").is_file());
    assert!(!root.join("ESCAPED").exists());
    assert!(!root.join(dir).join("site").exists());
}
#[cfg(unix)]
#[test]
fn provisioning_is_explicit_stdin_only_and_reports_partial_failure() {
    use std::os::unix::fs::PermissionsExt;
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    repo(root);
    site(
        root,
        "docs/zellij",
        "publish: {action: deploy, provider: netlify, site: site-id}\nci: {zel-version: 0.2.0}\n",
    );
    write(
        root,
        "bin/gh",
        "#!/bin/sh\n[ -f .github/workflows/docs.yml ] || exit 9\nprintf '%s\\n' \"$@\" > gh-args\ncat > gh-stdin\nexit 7\n",
    );
    fs::set_permissions(root.join("bin/gh"), fs::Permissions::from_mode(0o755)).unwrap();
    let path = std::env::join_paths(
        std::iter::once(root.join("bin"))
            .chain(std::env::split_paths(&std::env::var_os("PATH").unwrap())),
    )
    .unwrap();
    let invoke = |provision: bool| {
        let mut command = command(root, &["ci", "--force"]);
        command
            .env("PATH", &path)
            .env("NETLIFY_AUTH_TOKEN", "private-token");
        if provision {
            command.arg("--provision");
        }
        command.output().unwrap()
    };
    assert!(invoke(false).status.success());
    assert!(!root.join("gh-args").exists());
    let failure = invoke(true);
    assert_eq!(failure.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&failure.stderr).contains("workflow written"));
    assert!(!String::from_utf8_lossy(&failure.stderr).contains("private-token"));
    assert_eq!(
        fs::read_to_string(root.join("gh-args")).unwrap(),
        "secret\nset\nNETLIFY_AUTH_TOKEN\n"
    );
    assert_eq!(
        fs::read_to_string(root.join("gh-stdin")).unwrap(),
        "private-token"
    );
}
#[cfg(unix)]
#[test]
fn workflow_paths_cannot_follow_links_even_with_force() {
    use std::os::unix::fs::symlink;
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    repo(root);
    site(root, "docs/zellij", "ci: {zel-version: 0.2.0}\n");
    let outside = tempfile::tempdir().unwrap();
    symlink(outside.path(), root.join(".github")).unwrap();
    run(root, &["ci", "--force"], 2);
    assert!(fs::read_dir(outside.path()).unwrap().next().is_none());
}

#[test]
fn cwd_selects_the_repository_and_global_site_root_keeps_legacy_content_semantics() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    let project = root.join("Project 日本");
    fs::create_dir(&project).unwrap();
    repo(root);
    repo(&project);
    site(&project, "docs/manual", "ci: {zel-version: 0.2.0}\n");
    run(
        root,
        &[
            "--cwd",
            "Project 日本",
            "--site-root",
            "docs/manual",
            "ci",
            "init",
            "github",
        ],
        0,
    );
    let expected = fs::read(project.join(".github/workflows/docs.yml")).unwrap();
    assert!(!root.join(".github").exists());
    run(
        root,
        &[
            "ci",
            "--cwd",
            "Project 日本",
            "--content",
            "docs/manual",
            "--force",
        ],
        0,
    );
    assert_eq!(
        fs::read(project.join(".github/workflows/docs.yml")).unwrap(),
        expected
    );
    run(
        root,
        &[
            "--cwd",
            "Project 日本",
            "ci",
            "docs/manual",
            "--site-root",
            "docs/manual",
            "--force",
        ],
        2,
    );
    assert_eq!(
        fs::read(project.join(".github/workflows/docs.yml")).unwrap(),
        expected
    );
}

#[test]
fn workflows_keep_explicit_config_in_validation_build_and_delivery() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    repo(root);
    site(root, "docs/Manual 日本", "ci: {zel-version: 0.2.0}\n");
    let config = "docs/Manual 日本/preview's [site].yaml";
    fs::rename(root.join("docs/Manual 日本/zel.yaml"), root.join(config)).unwrap();
    write(root, "docs/Manual 日本/zel.yaml", "invalid: [");
    for delivery in [
        "",
        "publish: {action: deploy, provider: netlify, site: documentation}\n",
        "image: {name: ghcr.io/acme/manual:preview}\npublish: {action: push, image: static}\n",
        "image: {name: ghcr.io/acme/manual:preview}\npublish: {action: push, image: next}\n",
    ] {
        write(
            root,
            config,
            &format!("version: 1\nrenderer: portable\nci: {{zel-version: 0.2.0}}\n{delivery}"),
        );
        run(root, &["ci", "--config", config, "--force"], 0);
        let doc = workflow(root);
        assert_eq!(doc["env"]["SITE_CONFIG"], format!("./{config}"));
        for job in doc["jobs"].as_object().unwrap().values() {
            for step in job["steps"].as_array().unwrap() {
                if let Some(command) = step["run"].as_str().filter(|r| r.starts_with("zel ")) {
                    assert!(command.contains("--config \"$SITE_CONFIG\""), "{command}");
                    assert!(!command.contains("\"$SITE_ROOT\""));
                    assert!(
                        !command.contains("preview's"),
                        "path must be passed as data"
                    );
                }
            }
        }
        assert!(
            doc["on"]["pull_request"]["paths"]
                .as_array()
                .unwrap()
                .iter()
                .any(|path| path == "docs/Manual 日本/**")
        );
    }
    let previous = fs::read(root.join(".github/workflows/docs.yml")).unwrap();
    run(
        root,
        &[
            "ci",
            "--config",
            config,
            "--content",
            "docs/Manual 日本",
            "--force",
        ],
        2,
    );
    assert_eq!(
        fs::read(root.join(".github/workflows/docs.yml")).unwrap(),
        previous
    );
}

#[test]
fn image_platform_is_frozen_in_both_jobs_and_emulation_is_conditional() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    repo(root);
    site(
        root,
        "docs/zellij",
        "ci: {zel-version: 0.2.0}\nimage: {name: 'ghcr.io/acme/docs:v1', platform: linux/arm64}\npublish: {action: push, image: static}\n",
    );
    for (extra, expected) in [
        (vec![], "linux/arm64"),
        (vec!["--platform", "linux/amd64"], "linux/amd64"),
        (vec!["--target", "image"], "linux/arm64"),
    ] {
        let mut args = vec!["ci", "--force"];
        args.extend(extra);
        run(root, &args, 0);
        let doc = workflow(root);
        for job in ["verify", "publish"] {
            let steps = doc["jobs"][job]["steps"].as_array().unwrap();
            assert_eq!(
                steps
                    .iter()
                    .any(|s| s["uses"] == "docker/setup-qemu-action@v4"),
                expected == "linux/arm64"
            );
            let build = steps
                .iter()
                .find(|s| s["name"].as_str().is_some_and(|n| n.starts_with("Build")))
                .unwrap();
            assert!(
                build["run"]
                    .as_str()
                    .unwrap()
                    .contains(&format!("--platform {expected}")),
                "{build}"
            );
        }
    }
    let previous = fs::read(root.join(".github/workflows/docs.yml")).unwrap();
    for args in [
        vec![
            "ci",
            "--force",
            "--target",
            "static",
            "--platform",
            "linux/amd64",
        ],
        vec![
            "ci",
            "--force",
            "--deploy",
            "netlify",
            "--platform",
            "linux/arm64",
        ],
    ] {
        run(root, &args, 2);
        assert_eq!(
            fs::read(root.join(".github/workflows/docs.yml")).unwrap(),
            previous
        );
    }
    run(root, &["ci", "--force", "--target", "static"], 0);
    let yaml = fs::read_to_string(root.join(".github/workflows/docs.yml")).unwrap();
    assert!(!yaml.contains("--platform") && !yaml.contains("qemu"));
}
