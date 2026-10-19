//! Exercise the shipped flags and child-output policy without Docker or network.
use std::{
    fs,
    path::Path,
    process::{Command, Output},
};

fn command(root: &Path) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_zel"));
    command.current_dir(root).env("PATH", root.join("no-tools"));
    for key in ["NO_COLOR", "CLICOLOR_FORCE", "FORCE_COLOR"] {
        command.env_remove(key);
    }
    command
}
fn run(root: &Path, args: &[&str]) -> Output {
    command(root).args(args).output().unwrap()
}
fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}
fn ok(output: &Output) {
    assert!(output.status.success(), "{}", text(&output.stderr));
}
fn site(root: &Path) {
    fs::create_dir_all(root.join("docs/zellij/content")).unwrap();
    fs::write(
        root.join("docs/zellij/zel.yaml"),
        "version: 1\nrenderer: portable\n",
    )
    .unwrap();
    fs::write(root.join("docs/zellij/content/index.md"), "# Home\n").unwrap();
}

#[test]
fn quiet_executes_builds_and_preserves_warnings_errors_and_strict_exits() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    site(root);
    let output = run(root, &["-q", "b"]);
    ok(&output);
    assert!(output.stdout.is_empty() && output.stderr.is_empty());
    assert!(root.join("docs/zellij/site/index.html").is_file());
    fs::write(
        root.join("docs/zellij/content/index.md"),
        "# Home\n\n<span>Literal</span>\n",
    )
    .unwrap();
    let output = run(root, &["c", "--quiet"]);
    ok(&output);
    assert!(output.stdout.is_empty());
    assert!(text(&output.stderr).contains("warning"));
    let output = run(root, &["c", "--quiet", "--strict"]);
    assert_eq!(output.status.code(), Some(3));
    assert!(text(&output.stderr).contains("warnings are rejected"));
    let before = fs::read(root.join("docs/zellij/site/index.html")).unwrap();
    fs::write(
        root.join("docs/zellij/content/index.md"),
        "# Home\n[Broken](missing.md)\n",
    )
    .unwrap();
    let normal = run(root, &["b"]);
    let output = run(root, &["b", "--quiet"]);
    assert!(!output.status.success());
    assert_eq!(output.status.code(), normal.status.code());
    assert!(!output.stderr.is_empty());
    assert_eq!(
        before,
        fs::read(root.join("docs/zellij/site/index.html")).unwrap()
    );
}

#[test]
fn quiet_and_verbose_leave_data_payloads_intact() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    site(root);
    for args in [
        vec!["ctx"],
        vec!["context", "--json"],
        vec!["list", "sections", "--json"],
        vec!["themes"],
        vec!["version", "--json"],
        vec!["ex", "hero"],
        vec!["ns", "hero"],
        vec!["idf", "--kind", "static"],
        vec!["completions", "bash"],
        vec!["check", "--json"],
    ] {
        let normal = run(root, &args);
        ok(&normal);
        for flags in [
            vec!["--quiet", "--no-color"],
            vec!["--verbose", "--no-color"],
        ] {
            let mut selected = args.clone();
            selected.extend(&flags);
            let output = run(root, &selected);
            ok(&output);
            assert_eq!(normal.stdout, output.stdout, "{selected:?}");
            if flags[0] == "--verbose" {
                assert!(text(&output.stderr).contains("[verbose]"));
            } else {
                assert!(
                    output.stderr.is_empty(),
                    "{selected:?}: {}",
                    text(&output.stderr)
                );
            }
        }
    }
    let output = run(root, &["ctx", "schema.json", "--quiet", "--json"]);
    ok(&output);
    assert!(output.stdout.is_empty() && output.stderr.is_empty());
    assert_eq!(
        fs::read(root.join("schema.json")).unwrap(),
        zellij_core::schema::SCHEMA_JSON.as_bytes()
    );
}

#[test]
fn conflicts_fail_before_work_even_across_nested_commands() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    for args in [
        vec!["--quiet", "new", "portable", "created", "--verbose"],
        vec!["--verbose", "new", "--quiet", "portable", "created"],
        vec!["init", "created", "--quiet", "--verbose"],
        vec!["--quiet", "init", "created", "--json"],
    ] {
        let output = run(root, &args);
        assert_eq!(output.status.code(), Some(2), "{args:?}");
        assert!(!root.join("created").exists());
        assert!(!output.stderr.is_empty());
    }
}

#[test]
fn no_color_applies_to_help_and_all_parser_errors_even_with_forced_color() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    for args in [
        vec!["--help"],
        vec!["new", "--help"],
        vec!["not-a-command"],
        vec!["--site-root", "a", "build", "b"],
        vec!["--cwd", "a", "build", "--cwd", "b"],
    ] {
        for environment in [false, true] {
            let mut cmd = command(root);
            cmd.env("CLICOLOR_FORCE", "1").args(&args);
            if environment {
                cmd.env("NO_COLOR", "1");
            } else {
                cmd.arg("--no-color");
            }
            let output = cmd.output().unwrap();
            assert!(
                !output.stdout.contains(&27) && !output.stderr.contains(&27),
                "{args:?}: {output:?}"
            );
            assert!(!output.stdout.is_empty() || !output.stderr.is_empty());
        }
    }
}

#[test]
fn quiet_keeps_dry_run_plans_and_formatting_findings() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    site(root);
    let output = run(
        root,
        &[
            "netlify",
            "--site-id",
            "existing-site",
            "--dry-run",
            "--quiet",
        ],
    );
    ok(&output);
    assert!(text(&output.stdout).contains("Plan only"));
    assert!(text(&output.stdout).contains("existing-site"));
    assert!(!root.join("docs/zellij/site").exists());
    fs::write(
        root.join("docs/zellij/zel.yaml"),
        "{version: 1, renderer: portable}\n",
    )
    .unwrap();
    let output = run(root, &["fmt", "--check", "--quiet"]);
    assert!(!output.status.success());
    assert!(text(&output.stdout).contains("Would format"));
}

// Re-enter this test binary in isolated processes, including on Windows, without
// modifying the test runner's environment or process-wide reporting policy.
#[test]
fn child_output() {
    if std::env::var_os("ZEL_OUTPUT_CHILD").is_none() {
        return;
    }
    use std::io::Write;
    if std::env::var_os("ZEL_OUTPUT_ENV_CHECK").is_some() {
        assert_eq!(std::env::var("NO_COLOR").unwrap(), "1");
        assert!(std::env::var_os("FORCE_COLOR").is_none());
    }
    for index in 0..10000 {
        println!("noisy child progress on stdout");
        if index % 1000 == 0 {
            std::io::stdout().flush().unwrap();
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
    }
    print!("\x1b[31");
    std::io::stdout().flush().unwrap();
    std::thread::sleep(std::time::Duration::from_millis(40));
    println!("mcolored child output\x1b[0m");
    eprintln!("\x1b[31mchild diagnostic\x1b[0m");
    if std::env::var_os("ZEL_OUTPUT_FAIL").is_some() {
        std::process::exit(7);
    }
}

#[test]
fn subprocess_driver() {
    let Ok(mode) = std::env::var("ZEL_OUTPUT_DRIVER") else {
        return;
    };
    use zellij_core::{
        process::CommandExt,
        reporting::{self, Options},
    };
    reporting::initialize(Options {
        quiet: mode.contains("quiet"),
        no_color: mode.contains("plain"),
        verbose: false,
    });
    let mut cmd = Command::new(std::env::current_exe().unwrap());
    cmd.args(["--exact", "child_output", "--nocapture"])
        .env("ZEL_OUTPUT_CHILD", "1");
    if mode.contains("plain") {
        cmd.env("ZEL_OUTPUT_ENV_CHECK", "1").env("FORCE_COLOR", "1");
    }
    let status = if mode.contains("silent") {
        cmd.zel_spawn_silent().unwrap().wait().unwrap()
    } else {
        cmd.zel_status().unwrap()
    };
    std::process::exit(status.code().unwrap_or(1));
}

#[test]
fn subprocess_progress_is_suppressed_filtered_and_retained_on_failure() {
    for (mode, fail) in [
        ("quiet", false),
        ("quiet-plain", true),
        ("plain", false),
        ("silent-plain", false),
    ] {
        let mut cmd = Command::new(std::env::current_exe().unwrap());
        cmd.args(["--exact", "subprocess_driver", "--nocapture"])
            .env("ZEL_OUTPUT_DRIVER", mode);
        if fail {
            cmd.env("ZEL_OUTPUT_FAIL", "1");
        }
        let output = cmd.output().unwrap();
        assert_eq!(
            output.status.code(),
            Some(if fail { 7 } else { 0 }),
            "{output:?}"
        );
        if mode == "quiet" || mode == "silent-plain" {
            assert!(!text(&output.stdout).contains("noisy child"));
            assert!(output.stderr.is_empty());
        } else {
            assert!(!output.stdout.contains(&27) && !output.stderr.contains(&27));
            assert!(text(&output.stderr).contains("child diagnostic"));
            if fail {
                assert!(text(&output.stderr).contains("earlier subprocess output omitted"));
                assert!(output.stderr.len() < 67_000);
                assert!(!text(&output.stdout).contains("colored child output"));
            } else {
                assert!(text(&output.stdout).contains("colored child output"));
                assert_eq!(
                    text(&output.stdout)
                        .matches("noisy child progress on stdout")
                        .count(),
                    10000
                );
            }
        }
    }
}

#[cfg(unix)]
#[test]
fn actual_container_commands_honor_controls_without_a_daemon_or_secret_logging() {
    use std::os::unix::fs::PermissionsExt;
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    site(root);
    fs::create_dir(root.join("bin")).unwrap();
    let docker = root.join("bin/docker");
    fs::write(
        &docker,
        r#"#!/bin/sh
case "$1" in
  info) exit 0;;
  --version) echo 'fixture docker'; exit 0;;
  build) cat > recipe;;
  pull|push) :;;
  *) exit 99;;
esac
printf '\033[31mchild progress\033[0m\n'
printf '\033[31mchild detail\033[0m\n' >&2
exit "${FAKE_EXIT:-0}"
"#,
    )
    .unwrap();
    fs::set_permissions(&docker, fs::Permissions::from_mode(0o755)).unwrap();
    for args in [vec!["pull"], vec!["si"], vec!["ip", "fixture/site:test"]] {
        let invoke = |flag: &str, fail: bool| {
            let mut cmd = command(root);
            cmd.env(
                "PATH",
                format!("{}:/usr/bin:/bin", root.join("bin").display()),
            )
            .env("DOCKER_HOST", "https://user:secret-marker@example.invalid")
            .env("ZELLIJ_IMAGE", "fixture/base:test")
            .env("FAKE_EXIT", if fail { "9" } else { "0" });
            for key in [
                "DOCKER_CONTEXT",
                "CONTAINER_HOST",
                "CONTAINER_CONNECTION",
                "CONTAINERD_ADDRESS",
                "CONTAINERD_NAMESPACE",
            ] {
                cmd.env_remove(key);
            }
            cmd.args(&args).args([flag, "--no-color"]).output().unwrap()
        };
        let quiet = invoke("--quiet", false);
        ok(&quiet);
        assert!(
            quiet.stdout.is_empty() && quiet.stderr.is_empty(),
            "{args:?}: {quiet:?}"
        );
        let verbose = invoke("--verbose", false);
        ok(&verbose);
        assert!(text(&verbose.stderr).contains("Selected container runtime"));
        assert!(!text(&verbose.stderr).contains("secret-marker"));
        assert!(text(&verbose.stdout).contains("child progress"));
        assert!(!verbose.stdout.contains(&27) && !verbose.stderr.contains(&27));
        let failure = invoke("--quiet", true);
        assert!(!failure.status.success());
        assert!(failure.stdout.is_empty());
        assert!(text(&failure.stderr).contains("child detail"));
        assert!(!failure.stderr.contains(&27));
    }
    assert!(
        fs::read_to_string(root.join("recipe"))
            .unwrap()
            .contains("FROM")
    );
}
