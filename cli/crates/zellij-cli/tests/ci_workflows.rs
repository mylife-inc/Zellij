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
fn independent_site_workflows_do_not_share_a_concurrency_queue() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    repo(root);
    let mut groups = Vec::new();
    let mut names = Vec::new();
    for name in ["manual", "api"] {
        let dir = format!("docs/{name}");
        site(root, &dir, &format!("ci: {{workflow: {name}.yml}}\n"));
        run(root, &["ci", &dir], 0);
        let path = root.join(format!(".github/workflows/{name}.yml"));
        let original = fs::read(&path).unwrap();
        let workflow: Value = serde_yaml_ng::from_slice(&original).unwrap();
        let group = workflow["concurrency"]["group"].as_str().unwrap();
        assert!(
            workflow["name"]
                .as_str()
                .unwrap()
                .contains(&format!("{name}.yml"))
        );
        assert!(group.contains("${{ github.ref }}"));
        assert!(!group.contains("github.workflow"));
        assert_eq!(workflow["concurrency"]["cancel-in-progress"], false);
        groups.push(group.to_lowercase());
        names.push(workflow["name"].as_str().unwrap().to_owned());
        run(root, &["ci", &dir, "--force"], 0);
        assert_eq!(fs::read(path).unwrap(), original);
    }
    assert_ne!(groups[0], groups[1]);
    assert_ne!(names[0], names[1]);
}

#[test]
fn compatible_binary_pins_itself_and_invalid_settings_do_not_replace_a_workflow() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    repo(root);
    site(root, "docs/zellij", "");
    run(root, &["ci"], 0);
    let original = fs::read(root.join(".github/workflows/docs.yml")).unwrap();
    let workflow: serde_yaml_ng::Value = serde_yaml_ng::from_slice(&original).unwrap();
    assert_eq!(
        workflow["jobs"]["verify"]["steps"][1]["env"]["ZEL_VERSION"],
        format!("v{}", env!("CARGO_PKG_VERSION"))
    );
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
    write(
        root,
        &format!("{dir}/content/index.md"),
        "# Guide\n\n$x^2$\n\n```mermaid\ngraph TD; A-->B;\n```\n",
    );
    run(root, &["ci", dir, "--features", "all"], 0);
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
    let html = fs::read_to_string(output.path().join("zellij-site/index.html")).unwrap();
    assert!(html.contains("<math") && html.contains("class=\"mermaid-diagram\""));
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
fn image_platform_is_frozen_in_both_jobs_and_runner_matches_architecture() {
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
                doc["jobs"][job]["runs-on"],
                if expected == "linux/arm64" {
                    "ubuntu-24.04-arm"
                } else {
                    "ubuntu-latest"
                }
            );
            assert!(!steps.iter().any(|s| {
                s["uses"]
                    .as_str()
                    .is_some_and(|action| action.contains("qemu"))
            }));
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

#[test]
fn features_are_frozen_in_all_build_and_delivery_commands_but_not_checks_or_pushes() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    repo(root);
    let dir = "docs/Named manual";
    let filename = format!("{dir}/delivery.yaml");
    for renderer in ["portable", "hosted"] {
        for (settings, extra) in [
            ("", vec![]),
            (
                "publish: {action: deploy, provider: netlify, site: saved-id}\n",
                vec![],
            ),
            (
                "image: {name: ghcr.io/acme/docs:v1, platform: linux/arm64}\npublish: {action: push, image: static}\n",
                vec![],
            ),
            (
                "image: {name: ghcr.io/acme/docs:v1}\npublish: {action: push, image: next}\n",
                vec![],
            ),
            ("", vec!["--target", "image"]),
        ] {
            site(root, dir, "");
            let config =
                format!("version: 1\nrenderer: {renderer}\nci: {{zel-version: 0.2.0}}\n{settings}");
            write(root, &filename, &config);
            write(root, &format!("{dir}/zel.yaml"), "invalid neighbor: [");
            for (selection, canonical) in [
                (Some("all"), "maths,diagrams"),
                (Some("none"), "none"),
                (Some("diagrams,maths"), "maths,diagrams"),
                (None, ""),
            ] {
                let mut args = vec!["ci", "--config", &filename, "--force"];
                args.extend_from_slice(&extra);
                if let Some(value) = selection {
                    args.extend(["--features", value]);
                }
                run(root, &args, 0);
                let doc = workflow(root);
                let mut builds = 0;
                for job in doc["jobs"].as_object().unwrap().values() {
                    for step in job["steps"].as_array().unwrap() {
                        for line in step["run"].as_str().unwrap_or("").lines() {
                            if !line.starts_with("zel ") {
                                continue;
                            }
                            if line.starts_with("zel check") || line.starts_with("zel image push") {
                                assert!(!line.contains("--features"), "{line}");
                            } else {
                                builds += 1;
                                assert!(line.contains("--config \"$SITE_CONFIG\""), "{line}");
                                assert_eq!(
                                    line.contains("--features"),
                                    selection.is_some(),
                                    "{line}"
                                );
                                if selection.is_some() {
                                    assert!(
                                        line.ends_with(&format!("--features {canonical}")),
                                        "{line}"
                                    );
                                }
                            }
                        }
                    }
                }
                assert_eq!(
                    builds,
                    if settings.is_empty() && extra.is_empty() {
                        1
                    } else {
                        2
                    }
                );
                assert_eq!(fs::read_to_string(root.join(&filename)).unwrap(), config);
                let previous = fs::read(root.join(".github/workflows/docs.yml")).unwrap();
                run(
                    root,
                    &[
                        "ci",
                        "--config",
                        &filename,
                        "--force",
                        "--features",
                        "none,all",
                    ],
                    2,
                );
                assert_eq!(
                    fs::read(root.join(".github/workflows/docs.yml")).unwrap(),
                    previous
                );
            }
        }
    }
}

#[test]
fn build_overrides_are_data_and_skip_only_the_requested_preflight() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    repo(root);
    let dir = "docs/Named manual";
    let filename = format!("{dir}/delivery.yaml");
    let prefix = "/日本語/'$(touch${IFS}INJECTED)`touch${IFS}INJECTED2`";
    for (renderer, settings, extra, allowed) in [
        ("hosted", "", vec![], true),
        (
            "hosted",
            "publish: {action: deploy, provider: netlify, site: saved-id}\n",
            vec![],
            true,
        ),
        (
            "hosted",
            "image: {name: ghcr.io/acme/docs:v1}\npublish: {action: push, image: static}\n",
            vec![],
            true,
        ),
        (
            "portable",
            "image: {name: ghcr.io/acme/docs:v1}\npublish: {action: push, image: next}\n",
            vec![],
            true,
        ),
        ("portable", "", vec!["--target", "image"], true),
        ("portable", "", vec![], false),
        (
            "portable",
            "publish: {action: deploy, provider: netlify, site: saved-id}\n",
            vec![],
            false,
        ),
        (
            "portable",
            "image: {name: ghcr.io/acme/docs:v1}\npublish: {action: push, image: static}\n",
            vec![],
            false,
        ),
    ] {
        site(root, dir, "");
        let config = format!(
            "version: 1\nrenderer: {renderer}\nci: {{zel-version: 0.2.0-rc.2}}\n{settings}"
        );
        write(root, &filename, &config);
        write(root, &format!("{dir}/zel.yaml"), "invalid neighbor: [");
        for no_check in [false, true] {
            let mut args = vec![
                "ci",
                "--config",
                &filename,
                "--force",
                "--base-path",
                prefix,
                "--site-url",
                "https://DOCS.example:443/",
                "--features",
                "none",
            ];
            args.extend_from_slice(&extra);
            if no_check {
                args.push("--no-check");
            }
            let previous = fs::read(root.join(".github/workflows/docs.yml")).ok();
            run(root, &args, if allowed { 0 } else { 2 });
            if !allowed {
                assert_eq!(
                    fs::read(root.join(".github/workflows/docs.yml")).ok(),
                    previous
                );
                continue;
            }
            let doc = workflow(root);
            assert_eq!(doc["env"]["ZEL_BUILD_BASE_PATH"], prefix);
            assert_eq!(doc["env"]["ZEL_BUILD_SITE_URL"], "https://docs.example");
            for job in doc["jobs"].as_object().unwrap().values() {
                let commands: Vec<_> = job["steps"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .filter_map(|s| s["run"].as_str())
                    .collect();
                assert_eq!(
                    commands
                        .iter()
                        .filter(|c| c.starts_with("zel check"))
                        .count(),
                    usize::from(!no_check)
                );
                for line in commands
                    .iter()
                    .flat_map(|command| command.lines())
                    .filter(|line| line.starts_with("zel "))
                {
                    let is_build =
                        !line.starts_with("zel check") && !line.starts_with("zel image push");
                    assert_eq!(
                        line.contains("--base-path \"$ZEL_BUILD_BASE_PATH\""),
                        is_build,
                        "{line}"
                    );
                    assert_eq!(
                        line.contains("--site-url \"$ZEL_BUILD_SITE_URL\""),
                        is_build,
                        "{line}"
                    );
                    assert_eq!(line.contains("--no-check"), is_build && no_check, "{line}");
                    assert!(!line.contains(prefix));
                }
                #[cfg(unix)]
                {
                    let capture = root.join("argv");
                    let _ = fs::remove_file(&capture);
                    let script = format!(
                        "zel() {{ printf '%s\\0' \"$@\" >> \"$CAPTURE\"; }}\n{}",
                        commands
                            .iter()
                            .filter(|c| c.starts_with("zel ") || c.starts_with("TAG="))
                            .copied()
                            .collect::<Vec<_>>()
                            .join("\n")
                    );
                    let result = Command::new("bash")
                        .args(["-eu", "-c", &script])
                        .current_dir(root)
                        .env("CAPTURE", &capture)
                        .env("SITE_ROOT", dir)
                        .env("SITE_CONFIG", &filename)
                        .env("RUNNER_TEMP", root)
                        .env("IMAGE_TAG", "ghcr.io/acme/docs:v1")
                        .env("REPOSITORY", "acme/docs")
                        .env("REVISION", "abcdef")
                        .env("ZEL_BUILD_BASE_PATH", prefix)
                        .env("ZEL_BUILD_SITE_URL", "https://docs.example")
                        .output()
                        .unwrap();
                    assert!(result.status.success(), "{result:?}");
                    let values = fs::read(capture).unwrap();
                    assert_eq!(
                        values
                            .split(|b| *b == 0)
                            .filter(|value| *value == prefix.as_bytes())
                            .count(),
                        1
                    );
                    assert!(!root.join("INJECTED").exists() && !root.join("INJECTED2").exists());
                }
            }
            assert_eq!(fs::read_to_string(root.join(&filename)).unwrap(), config);
        }
    }
    write(root, &filename, "version: 1\nrenderer: hosted\n");
    run(
        root,
        &[
            "ci",
            "init",
            "github",
            "--config",
            &filename,
            "--force",
            "--base-path",
            "/",
        ],
        0,
    );
    assert_eq!(workflow(root)["env"]["ZEL_BUILD_BASE_PATH"], "/");
    let previous = fs::read(root.join(".github/workflows/docs.yml")).unwrap();
    for flags in [
        vec!["--base-path", "/${{github.ref}}"],
        vec!["--site-url", "https://docs.example/path"],
        vec!["--no-check", "--zel-version", "0.2.0-rc.1"],
        vec!["--no-check", "--zel-version", "0.2.0-beta.3"],
    ] {
        let mut args = vec!["ci", "--config", &filename, "--force"];
        args.extend(flags);
        run(root, &args, 2);
        assert_eq!(
            fs::read(root.join(".github/workflows/docs.yml")).unwrap(),
            previous
        );
    }
    run(root, &["ci", "--config", &filename, "--force"], 0);
    let doc = workflow(root);
    assert!(doc["env"].get("ZEL_BUILD_BASE_PATH").is_none());
    assert!(
        doc["jobs"]["verify"]["steps"]
            .as_array()
            .unwrap()
            .iter()
            .any(|s| s["name"] == "Validate documentation")
    );
}
