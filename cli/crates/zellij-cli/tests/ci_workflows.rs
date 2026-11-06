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
        .env_remove("FLY_APP")
        .env_remove("FLY_ORG")
        .env_remove("FLY_API_TOKEN")
        .env_remove("NETLIFY_AUTH_TOKEN")
        .env_remove("NETLIFY_SITE_ID")
        .env_remove("CLOUDFLARE_PAGES_PROJECT_NAME")
        .env_remove("CLOUDFLARE_ACCOUNT_ID")
        .env_remove("CLOUDFLARE_API_TOKEN")
        .env_remove("VERCEL_TOKEN")
        .env_remove("VERCEL_PROJECT_ID")
        .env_remove("VERCEL_TEAM_ID")
        .env_remove("AWS_ACCESS_KEY_ID")
        .env_remove("AWS_SECRET_ACCESS_KEY")
        .env_remove("AWS_SESSION_TOKEN")
        .env_remove("AWS_REGION")
        .env_remove("S3_EXPECTED_BUCKET_OWNER")
        .env_remove("S3_DESTINATION")
        .env_remove("AWS_ENDPOINT_URL")
        .env_remove("AWS_ENDPOINT_URL_S3")
        .env_remove("GITHUB_PAGES_TOKEN")
        .env_remove("GITHUB_PAGES_REPOSITORY");
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
fn combined_publication_builds_natively_and_joins_only_successful_digest_outputs() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    repo(root);
    for kind in ["static", "next", "legacy"] {
        site(
            root,
            "docs/Named manual",
            &format!(
                "ci: {{zel-version: 0.2.0-rc.2}}\nimage: {{name: 'registry.example:5000/acme/docs:v1', platform: all}}\npublish: {{action: push, image: {}}}\n",
                if kind == "legacy" { "next" } else { kind }
            ),
        );
        let file = "docs/Named manual/delivery.yaml";
        fs::rename(root.join("docs/Named manual/zel.yaml"), root.join(file)).unwrap();
        let mut args = vec![
            "ci",
            "--config",
            file,
            "--force",
            "--features",
            "none",
            "--build-arg",
            "LABEL=日本語 $(literal) =",
        ];
        if kind == "legacy" {
            args.extend(["--target", "image"]);
        }
        run(root, &args, 0);
        let doc = workflow(root);
        assert!(!doc.to_string().contains("qemu"));
        let verify = &doc["jobs"]["verify"];
        assert_eq!(
            verify["strategy"]["matrix"]["include"]
                .as_array()
                .unwrap()
                .len(),
            2
        );
        assert!(!verify.to_string().contains("secrets."));
        assert!(!verify.to_string().contains("zel image push"));
        for (arch, runner) in [("amd64", "ubuntu-latest"), ("arm64", "ubuntu-24.04-arm")] {
            let job = &doc["jobs"][format!("publish_{arch}")];
            assert_eq!(job["runs-on"], runner);
            assert_eq!(job["outputs"]["image"], "${{ steps.image.outputs.image }}");
            let steps = job["steps"].as_array().unwrap();
            let build = steps.last().unwrap();
            let script = build["run"].as_str().unwrap();
            assert!(script.contains(&format!("--platform linux/{arch}")));
            assert!(script.contains("--features none --build-arg \"$ZEL_IMAGE_ARG_0\""));
            assert!(script.contains("--config \"$SITE_CONFIG\""));
            assert!(script.contains("${GITHUB_RUN_ID}-${GITHUB_RUN_ATTEMPT}"));
            assert!(script.contains(".RepoDigests 0"));
            assert!(!script.contains("日本語") && !script.contains(":v1"));
            assert!(
                job["if"]
                    .as_str()
                    .unwrap()
                    .contains("github.ref == 'refs/heads/main'")
            );
            assert_eq!(
                script
                    .lines()
                    .find(|line| line.starts_with("zel image push"))
                    .unwrap(),
                "zel image push \"$BUILD_TAG\""
            );
            if kind != "legacy" {
                assert_eq!(
                    build["env"]["IMAGE_REPOSITORY"],
                    "registry.example:5000/acme/docs"
                );
            }
        }
        let publish = &doc["jobs"]["publish"];
        assert_eq!(
            publish["needs"],
            serde_json::json!(["publish_amd64", "publish_arm64"])
        );
        let final_step = publish["steps"].as_array().unwrap().last().unwrap();
        assert_eq!(
            final_step["env"]["ARM64_IMAGE"],
            "${{ needs.publish_arm64.outputs.image }}"
        );
        let script = final_step["run"].as_str().unwrap();
        assert!(script.contains("--tag \"$IMAGE_TAG\" \"$AMD64_IMAGE\" \"$ARM64_IMAGE\""));
        assert!(!script.contains("--build-arg") && !script.contains("zel "));
        // An older release cannot run these workflows; preserve the written file.
        let before = fs::read(root.join(".github/workflows/docs.yml")).unwrap();
        args.extend(["--zel-version", "0.2.0-rc.1"]);
        run(root, &args, 2);
        assert_eq!(
            fs::read(root.join(".github/workflows/docs.yml")).unwrap(),
            before
        );
    }
}

#[test]
fn image_build_arguments_are_quoted_data_in_both_jobs_and_require_image_delivery() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    repo(root);
    let dir = "docs/Named manual";
    let file = format!("{dir}/delivery.yaml");
    let assignment = "DOCS_LABEL=日本語 '$(touch INJECTED)`echo literal` = ";
    for renderer in ["portable", "hosted"] {
        for kind in ["static", "next", "legacy"] {
            site(root, dir, "");
            let settings = if kind == "legacy" {
                String::new()
            } else {
                format!(
                    "image: {{name: ghcr.io/acme/docs:v1}}\npublish: {{action: push, image: {kind}}}\n"
                )
            };
            let config = format!(
                "version: 1\nrenderer: {renderer}\nci: {{zel-version: 0.2.0-rc.2}}\n{settings}"
            );
            write(root, &file, &config);
            write(root, &format!("{dir}/zel.yaml"), "invalid neighbor: [");
            let mut args = vec![
                "ci",
                "--config",
                &file,
                "--force",
                "--build-arg",
                "DOCS_LABEL=old",
                "--build-arg",
                assignment,
                "--build-arg",
                "EMPTY=",
            ];
            if kind == "legacy" {
                args.extend(["--target", "image"]);
            }
            run(root, &args, 0);
            let doc = workflow(root);
            assert_eq!(doc["env"]["ZEL_IMAGE_ARG_0"], assignment);
            assert_eq!(doc["env"]["ZEL_IMAGE_ARG_1"], "EMPTY=");
            for job in doc["jobs"].as_object().unwrap().values() {
                let commands: Vec<_> = job["steps"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .filter_map(|s| s["run"].as_str())
                    .filter(|s| s.starts_with("zel ") || s.starts_with("TAG="))
                    .collect();
                for line in commands
                    .iter()
                    .flat_map(|s| s.lines())
                    .filter(|s| s.starts_with("zel "))
                {
                    let build =
                        !line.starts_with("zel check") && !line.starts_with("zel image push");
                    assert_eq!(
                        line.contains("--build-arg \"$ZEL_IMAGE_ARG_0\""),
                        build,
                        "{line}"
                    );
                    assert_eq!(
                        line.contains("--build-arg \"$ZEL_IMAGE_ARG_1\""),
                        build,
                        "{line}"
                    );
                    assert!(!line.contains(assignment));
                }
                #[cfg(target_os = "linux")]
                {
                    let capture = root.join("argv");
                    let _ = fs::remove_file(&capture);
                    let script = format!(
                        "zel() {{ printf '%s\\0' \"$@\" >> \"$CAPTURE\"; }}\n{}",
                        commands.join("\n")
                    );
                    let output = Command::new("bash")
                        .args(["-eu", "-c", &script])
                        .current_dir(root)
                        .env("CAPTURE", &capture)
                        .env("SITE_CONFIG", &file)
                        .env("SITE_ROOT", dir)
                        .env("IMAGE_TAG", "ghcr.io/acme/docs:v1")
                        .env("REPOSITORY", "acme/docs")
                        .env("REVISION", "abc")
                        .env("ZEL_IMAGE_ARG_0", assignment)
                        .env("ZEL_IMAGE_ARG_1", "EMPTY=")
                        .output()
                        .unwrap();
                    assert!(output.status.success(), "{output:?}");
                    let values = fs::read(capture).unwrap();
                    for expected in [assignment, "EMPTY="] {
                        assert_eq!(
                            values
                                .split(|b| *b == 0)
                                .filter(|v| *v == expected.as_bytes())
                                .count(),
                            1
                        );
                    }
                    assert!(!root.join("INJECTED").exists());
                }
            }
            assert_eq!(fs::read_to_string(root.join(&file)).unwrap(), config);
            let previous = fs::read(root.join(".github/workflows/docs.yml")).unwrap();
            for extra in [
                vec!["--target", "static"],
                vec!["--deploy", "netlify"],
                vec!["--zel-version", "0.2.0-rc.1"],
                vec!["--build-arg", "EXPRESSION=${{github.ref}}"],
            ] {
                let mut invalid = args.clone();
                if kind == "legacy" {
                    invalid.truncate(invalid.len() - 2);
                }
                invalid.extend(extra);
                run(root, &invalid, 2);
                assert_eq!(
                    fs::read(root.join(".github/workflows/docs.yml")).unwrap(),
                    previous
                );
            }
            let mut defaults = vec!["ci", "init", "github", "--config", &file, "--force"];
            if kind == "legacy" {
                defaults.extend(["--target", "image"]);
            }
            run(root, &defaults, 0);
            assert!(
                !fs::read_to_string(root.join(".github/workflows/docs.yml"))
                    .unwrap()
                    .contains("ZEL_IMAGE_ARG_")
            );
        }
    }
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
    for (provider, token_name) in [
        ("netlify", "NETLIFY_AUTH_TOKEN"),
        ("cloudflare", "CLOUDFLARE_API_TOKEN"),
        ("vercel", "VERCEL_TOKEN"),
        ("github-pages", "GITHUB_PAGES_TOKEN"),
    ] {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path();
        repo(root);
        let destination = if provider == "github-pages" {
            "acme/manual"
        } else {
            "site-id"
        };
        site(
            root,
            "docs/zellij",
            &format!(
                "publish: {{action: deploy, provider: {provider}, site: {destination}}}\nci: {{zel-version: 0.2.0-rc.2, production: true}}\n"
            ),
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
            command.env("PATH", &path).env(token_name, "private-token");
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
            format!("secret\nset\n{token_name}\n")
        );
        assert_eq!(
            fs::read_to_string(root.join("gh-stdin")).unwrap(),
            "private-token"
        );
    }
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
                // Generated jobs target Ubuntu, including its Bash 4+ case
                // conversion in legacy image publication. macOS's system Bash
                // cannot execute that existing workflow syntax. Every platform
                // still checks generated data and native CLI forwarding.
                #[cfg(target_os = "linux")]
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

#[test]
fn cloudflare_workflows_use_preview_by_default_and_isolate_credentials() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    repo(root);
    site(
        root,
        "docs/zellij",
        "ci: {zel-version: 0.2.0-rc.2}\npublish: {action: deploy, provider: cloudflare, site: manual}\n",
    );
    for prod in [false, true] {
        let mut args = vec!["ci", "--force"];
        if prod {
            args.push("--prod");
        }
        let result = command(root, &args)
            .env_remove("CLOUDFLARE_PAGES_PROJECT_NAME")
            .env_remove("CLOUDFLARE_API_TOKEN")
            .output()
            .unwrap();
        assert!(result.status.success(), "{result:?}");
        let doc = workflow(root);
        let publish = &doc["jobs"]["publish"];
        let verify = doc["jobs"]["verify"].to_string();
        assert!(!verify.contains("CLOUDFLARE") && !verify.contains("secrets."));
        assert!(!publish["if"].as_str().unwrap().contains("pull_request"));
        let step = publish["steps"].as_array().unwrap().last().unwrap();
        assert!(
            step["run"]
                .as_str()
                .unwrap()
                .starts_with("zel deploy cloudflare")
        );
        assert_eq!(step["run"].as_str().unwrap().contains("--prod"), prod);
        assert_eq!(step["env"]["CLOUDFLARE_PAGES_PROJECT_NAME"], "manual");
        assert_eq!(
            step["env"]["CLOUDFLARE_API_TOKEN"],
            "${{ secrets.CLOUDFLARE_API_TOKEN }}"
        );
        assert!(
            step["env"]["CLOUDFLARE_ACCOUNT_ID"]
                .as_str()
                .unwrap()
                .contains("vars.CLOUDFLARE_ACCOUNT_ID")
        );
        assert!(!doc.to_string().contains("NETLIFY"));
    }
    let result = command(root, &["ci", "--force", "--zel-version", "0.2.0-rc.1"])
        .output()
        .unwrap();
    assert_eq!(result.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&result.stderr).contains("0.2.0-rc.2"));
}
#[test]
fn cloudflare_workflow_override_does_not_reuse_netlify_destination() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    repo(root);
    site(
        root,
        "docs/zellij",
        "ci: {zel-version: 0.2.0-rc.2}\npublish: {action: deploy, provider: netlify, site: old-site}\n",
    );
    let result = command(root, &["ci", "--deploy", "cloudflare"])
        .env_remove("CLOUDFLARE_PAGES_PROJECT_NAME")
        .output()
        .unwrap();
    assert!(result.status.success(), "{result:?}");
    let doc = workflow(root).to_string();
    assert!(!doc.contains("old-site"));
    assert!(doc.contains("vars.CLOUDFLARE_PAGES_PROJECT_NAME"));
}

#[test]
fn vercel_workflows_use_preview_by_default_and_isolate_credentials() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    repo(root);
    site(
        root,
        "docs/zellij",
        "ci: {zel-version: 0.2.0-rc.2}\npublish: {action: deploy, provider: vercel, site: manual}\n",
    );
    for prod in [false, true] {
        let mut args = vec!["ci", "--force"];
        if prod {
            args.push("--prod");
        }
        let result = command(root, &args)
            .env_remove("VERCEL_PROJECT_ID")
            .env_remove("VERCEL_TOKEN")
            .output()
            .unwrap();
        assert!(result.status.success(), "{result:?}");
        let doc = workflow(root);
        let publish = &doc["jobs"]["publish"];
        let verify = doc["jobs"]["verify"].to_string();
        assert!(!verify.contains("VERCEL") && !verify.contains("secrets."));
        assert!(!publish["if"].as_str().unwrap().contains("pull_request"));
        let step = publish["steps"].as_array().unwrap().last().unwrap();
        assert!(
            step["run"]
                .as_str()
                .unwrap()
                .starts_with("zel deploy vercel")
        );
        assert_eq!(step["run"].as_str().unwrap().contains("--prod"), prod);
        assert_eq!(step["env"]["VERCEL_PROJECT_ID"], "manual");
        assert_eq!(step["env"]["VERCEL_TOKEN"], "${{ secrets.VERCEL_TOKEN }}");
        assert!(
            step["env"]["VERCEL_TEAM_ID"]
                .as_str()
                .unwrap()
                .contains("vars.VERCEL_TEAM_ID")
        );
        assert!(!doc.to_string().contains("NETLIFY"));
    }
    let result = command(root, &["ci", "--force", "--zel-version", "0.2.0-rc.1"])
        .output()
        .unwrap();
    assert_eq!(result.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&result.stderr).contains("0.2.0-rc.2"));
}
#[test]
fn vercel_workflow_override_does_not_reuse_netlify_destination() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    repo(root);
    site(
        root,
        "docs/zellij",
        "ci: {zel-version: 0.2.0-rc.2}\npublish: {action: deploy, provider: netlify, site: old-site}\n",
    );
    let result = command(root, &["ci", "--deploy", "vercel"])
        .env_remove("VERCEL_PROJECT_ID")
        .output()
        .unwrap();
    assert!(result.status.success(), "{result:?}");
    let doc = workflow(root).to_string();
    assert!(!doc.contains("old-site"));
    assert!(doc.contains("vars.VERCEL_PROJECT_ID"));
}

#[test]
fn github_pages_workflow_requires_production_and_uses_dedicated_credentials() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    repo(root);
    site(
        root,
        "docs/zellij",
        "ci: {zel-version: 0.2.0-rc.2}\npublish: {action: deploy, provider: github-pages, site: acme/manual}\n",
    );
    let denied = command(root, &["ci"]).output().unwrap();
    assert_eq!(denied.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&denied.stderr).contains("--prod"));
    assert!(!root.join(".github/workflows/docs.yml").exists());
    run(root, &["ci", "--prod"], 0);
    let original = fs::read(root.join(".github/workflows/docs.yml")).unwrap();
    let doc = workflow(root);
    let publish = &doc["jobs"]["publish"];
    let verify = doc["jobs"]["verify"].to_string();
    assert!(!verify.contains("secrets.") && !verify.contains("GITHUB_PAGES_TOKEN"));
    assert!(!publish["if"].as_str().unwrap().contains("pull_request"));
    let step = publish["steps"].as_array().unwrap().last().unwrap();
    assert!(
        step["run"]
            .as_str()
            .unwrap()
            .starts_with("zel deploy github-pages")
    );
    assert!(step["run"].as_str().unwrap().contains("--prod"));
    assert_eq!(step["env"]["GITHUB_PAGES_REPOSITORY"], "acme/manual");
    assert_eq!(
        step["env"]["GITHUB_PAGES_TOKEN"],
        "${{ secrets.GITHUB_PAGES_TOKEN }}"
    );
    assert!(!doc.to_string().contains("secrets.GITHUB_TOKEN"));
    for args in [
        vec!["ci", "--prod", "--force", "--branch", "gh-pages"],
        vec!["ci", "--prod", "--force", "--zel-version", "0.2.0-rc.1"],
    ] {
        run(root, &args, 2);
        assert_eq!(
            fs::read(root.join(".github/workflows/docs.yml")).unwrap(),
            original
        );
    }
    site(
        root,
        "docs/zellij",
        "ci: {zel-version: 0.2.0-rc.2, production: true}\npublish: {action: deploy, provider: github-pages, site: acme/manual}\n",
    );
    run(root, &["ci", "--force"], 0);
}
#[test]
fn github_pages_workflow_override_does_not_reuse_another_providers_destination() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    repo(root);
    site(
        root,
        "docs/zellij",
        "ci: {zel-version: 0.2.0-rc.2}\npublish: {action: deploy, provider: netlify, site: old-site}\n",
    );
    let result = command(root, &["ci", "--deploy", "github-pages", "--prod"])
        .env_remove("GITHUB_PAGES_REPOSITORY")
        .output()
        .unwrap();
    assert!(result.status.success(), "{result:?}");
    let doc = workflow(root).to_string();
    assert!(!doc.contains("old-site"));
    assert!(doc.contains("vars.GITHUB_PAGES_REPOSITORY"));
}

#[test]
fn s3_workflow_requires_production_and_isolates_aws_credentials() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    repo(root);
    site(
        root,
        "docs/zellij",
        "ci: {zel-version: 0.2.0-rc.2}\npublish: {action: deploy, provider: s3, site: acme-docs/manual}\n",
    );
    run(root, &["ci"], 2);
    assert!(!root.join(".github/workflows/docs.yml").exists());
    run(root, &["ci", "--prod"], 0);
    let doc = workflow(root);
    let publish = &doc["jobs"]["publish"];
    assert!(!doc["jobs"]["verify"].to_string().contains("secrets."));
    assert!(!publish["if"].as_str().unwrap().contains("pull_request"));
    let step = publish["steps"].as_array().unwrap().last().unwrap();
    assert!(step["run"].as_str().unwrap().starts_with("zel deploy s3"));
    assert!(step["run"].as_str().unwrap().contains("--prod"));
    assert_eq!(step["env"]["S3_DESTINATION"], "acme-docs/manual");
    for key in [
        "AWS_ACCESS_KEY_ID",
        "AWS_SECRET_ACCESS_KEY",
        "AWS_SESSION_TOKEN",
    ] {
        assert_eq!(step["env"][key], format!("${{{{ secrets.{key} }}}}"));
    }
    for key in ["AWS_REGION", "S3_EXPECTED_BUCKET_OWNER"] {
        assert!(
            step["env"][key]
                .as_str()
                .unwrap()
                .contains(&format!("vars.{key}"))
        );
    }
    let original = fs::read(root.join(".github/workflows/docs.yml")).unwrap();
    run(
        root,
        &["ci", "--prod", "--force", "--zel-version", "0.2.0-rc.1"],
        2,
    );
    assert_eq!(
        fs::read(root.join(".github/workflows/docs.yml")).unwrap(),
        original
    );
    site(
        root,
        "docs/zellij",
        "ci: {zel-version: 0.2.0-rc.2, production: true}\npublish: {action: deploy, provider: s3, site: acme-docs/manual}\n",
    );
    run(root, &["ci", "--force"], 0);
    site(
        root,
        "docs/zellij",
        "ci: {zel-version: 0.2.0-rc.2}\npublish: {action: deploy, provider: netlify, site: old-site}\n",
    );
    run(root, &["ci", "--deploy", "s3", "--prod", "--force"], 0);
    let text = workflow(root).to_string();
    assert!(text.contains("vars.S3_DESTINATION") && !text.contains("old-site"));
}
#[test]
fn s3_provision_validates_all_credentials_before_writing_workflow() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    repo(root);
    site(
        root,
        "docs/zellij",
        "ci: {zel-version: 0.2.0-rc.2, production: true}\npublish: {action: deploy, provider: s3, site: acme-docs/manual}\n",
    );
    for (access, secret, session) in [
        ("", "", ""),
        ("AKIAIOSFODNN7EXAMPLE", "", ""),
        ("ASIAIOSFODNN7EXAMPLE", "fixture-secret", ""),
        ("ASIAIOSFODNN7EXAMPLE", "fixture-secret", "bad\ntoken"),
    ] {
        let output = command(root, &["ci", "--provision"])
            .env("AWS_ACCESS_KEY_ID", access)
            .env("AWS_SECRET_ACCESS_KEY", secret)
            .env("AWS_SESSION_TOKEN", session)
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(2), "{output:?}");
        assert!(!root.join(".github/workflows/docs.yml").exists());
        assert!(!String::from_utf8_lossy(&output.stderr).contains("fixture-secret"));
    }
}
#[cfg(unix)]
#[test]
fn s3_provision_sends_multiple_secrets_only_on_stdin_and_keeps_partial_workflow() {
    use std::os::unix::fs::PermissionsExt;
    for fail in [
        "none",
        "AWS_ACCESS_KEY_ID",
        "AWS_SECRET_ACCESS_KEY",
        "AWS_SESSION_TOKEN",
    ] {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path();
        repo(root);
        site(
            root,
            "docs/zellij",
            "ci: {zel-version: 0.2.0-rc.2, production: true}\npublish: {action: deploy, provider: s3, site: acme-docs/manual}\n",
        );
        write(
            root,
            "bin/gh",
            "#!/bin/sh\n[ -f .github/workflows/docs.yml ] || exit 9\nprintf '%s\\n' \"$@\" >> gh-args\ncat > \"stdin-$3\"\n[ \"$3\" = \"$FAIL_SECRET\" ] && exit 7\nexit 0\n",
        );
        fs::set_permissions(root.join("bin/gh"), fs::Permissions::from_mode(0o755)).unwrap();
        let path = std::env::join_paths(
            std::iter::once(root.join("bin"))
                .chain(std::env::split_paths(&std::env::var_os("PATH").unwrap())),
        )
        .unwrap();
        let invoke = |provision: bool| {
            let mut cmd = command(root, &["ci", "--force"]);
            cmd.env("PATH", &path)
                .env("FAIL_SECRET", fail)
                .env("AWS_ACCESS_KEY_ID", "ASIAIOSFODNN7EXAMPLE")
                .env("AWS_SECRET_ACCESS_KEY", "fixture-secret")
                .env("AWS_SESSION_TOKEN", "fixture-session");
            if provision {
                cmd.arg("--provision");
            }
            cmd.output().unwrap()
        };
        assert!(invoke(false).status.success());
        assert!(!root.join("gh-args").exists());
        let result = invoke(true);
        assert_eq!(
            result.status.code(),
            Some(if fail == "none" { 0 } else { 1 })
        );
        let diagnostic = String::from_utf8_lossy(&result.stderr);
        assert!(!diagnostic.contains("fixture-secret") && !diagnostic.contains("fixture-session"));
        if fail != "none" {
            assert!(diagnostic.contains("workflow written"));
        }
        let mut expected = String::new();
        for (key, value) in [
            ("AWS_ACCESS_KEY_ID", "ASIAIOSFODNN7EXAMPLE"),
            ("AWS_SECRET_ACCESS_KEY", "fixture-secret"),
            ("AWS_SESSION_TOKEN", "fixture-session"),
        ] {
            expected.push_str(&format!("secret\nset\n{key}\n"));
            assert_eq!(
                fs::read_to_string(root.join(format!("stdin-{key}"))).unwrap(),
                value
            );
            if fail == key {
                break;
            }
        }
        assert_eq!(fs::read_to_string(root.join("gh-args")).unwrap(), expected);
        assert!(
            !fs::read_to_string(root.join(".github/workflows/docs.yml"))
                .unwrap()
                .contains("fixture-secret")
        );
    }
}

#[test]
fn fly_workflows_build_the_selected_kind_without_credentials_on_prs() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    repo(root);
    site(
        root,
        "docs/zellij",
        "ci: {zel-version: 0.2.0-rc.2}\npublish: {action: deploy, provider: fly, site: manual, image: next}\nimage: {platform: all}\n",
    );
    run(root, &["ci"], 2);
    assert!(!root.join(".github/workflows/docs.yml").exists());
    run(root, &["ci", "--prod"], 0);
    let doc = workflow(root);
    let verify = doc["jobs"]["verify"].to_string();
    assert!(
        verify.contains("zel image next")
            && verify.contains("linux/amd64")
            && !verify.contains("FLY_API_TOKEN")
            && !verify.contains("secrets.")
    );
    let step = doc["jobs"]["publish"]["steps"]
        .as_array()
        .unwrap()
        .last()
        .unwrap();
    assert!(
        step["run"].as_str().unwrap().contains("zel deploy fly")
            && step["run"].as_str().unwrap().contains("--kind next --prod")
    );
    assert!(
        step["run"].as_str().unwrap().contains("--password-stdin")
            && step["run"].as_str().unwrap().contains("docker logout")
    );
    assert_eq!(step["env"]["FLY_APP"], "manual");
    assert_eq!(step["env"]["FLY_API_TOKEN"], "${{ secrets.FLY_API_TOKEN }}");
    assert_eq!(doc["jobs"]["publish"]["runs-on"], "ubuntu-latest");
    run(root, &["ci", "--prod", "--kind", "static", "--force"], 0);
    assert!(
        workflow(root)["jobs"]["verify"]
            .to_string()
            .contains("zel image static")
    );
    for flags in [
        vec!["--platform", "all"],
        vec!["--platform", "linux/arm64"],
        vec!["--zel-version", "0.2.0-rc.1"],
        vec!["--target", "static", "--kind", "next"],
    ] {
        let mut args = vec!["ci", "--prod", "--force"];
        args.extend(flags);
        run(root, &args, 2);
    }
    let before = fs::read(root.join(".github/workflows/docs.yml")).unwrap();
    let output = command(root, &["ci", "--prod", "--provision", "--force"])
        .env("FLY_API_TOKEN", "bad\ntoken")
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert_eq!(
        fs::read(root.join(".github/workflows/docs.yml")).unwrap(),
        before
    );
    site(
        root,
        "docs/zellij",
        "ci: {zel-version: 0.2.0-rc.2}\npublish: {action: deploy, provider: netlify, site: other-site}\n",
    );
    run(root, &["ci", "--deploy", "fly", "--prod", "--force"], 0);
    let text = workflow(root).to_string();
    assert!(text.contains("vars.FLY_APP") && !text.contains("other-site"));
}
