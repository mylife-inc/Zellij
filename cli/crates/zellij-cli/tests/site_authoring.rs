use serde_json::Value;
use std::{
    collections::BTreeMap,
    fs,
    io::Write,
    path::Path,
    process::{Command, Stdio},
};

fn run(root: &Path, args: &[&str], input: &str, code: i32) -> std::process::Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_zel"))
        .args(args)
        .current_dir(root)
        .env("PATH", root.join("no-tools"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let _ = child.stdin.take().unwrap().write_all(input.as_bytes());
    let output = child.wait_with_output().unwrap();
    assert_eq!(output.status.code(), Some(code), "{args:?}: {output:?}");
    output
}
fn yaml(path: impl AsRef<Path>) -> Value {
    serde_yaml_ng::from_slice(&fs::read(path).unwrap()).unwrap()
}
fn tree(root: &Path) -> BTreeMap<String, Vec<u8>> {
    walkdir::WalkDir::new(root)
        .into_iter()
        .map(Result::unwrap)
        .filter(|entry| entry.file_type().is_file())
        .map(|entry| {
            (
                entry
                    .path()
                    .strip_prefix(root)
                    .unwrap()
                    .to_string_lossy()
                    .replace('\\', "/"),
                fs::read(entry.path()).unwrap(),
            )
        })
        .collect()
}
#[test]
fn noninteractive_defaults_and_aliases_keep_their_existing_starters() {
    for (args, theme, portable) in [
        (vec!["init"], "cupertino", true),
        (vec!["new", "portable"], "cupertino", true),
        (vec!["nd"], "paper", false),
        (vec!["new", "docs"], "paper", false),
    ] {
        let temp = tempfile::tempdir().unwrap();
        let out = run(temp.path(), &args, "", 0);
        assert!(out.stderr.is_empty());
        let root = temp.path().join("docs/zellij");
        let site = yaml(root.join(if portable {
            "zel.yaml"
        } else {
            "content/site.yaml"
        }));
        assert_eq!(site["name"], "zellij");
        assert_eq!(site["theme"], theme);
        assert_eq!(
            root.join("content/guide/getting-started.md").exists(),
            portable
        );
        assert_eq!(
            root.join("content/sections/home/features.yaml").exists(),
            !portable
        );
        assert!(!root.join("content/01-guide").exists());
    }
}
#[test]
fn portable_wizard_matches_flags_and_builds_without_external_tools() {
    for minimal in [false, true] {
        let temp = tempfile::tempdir().unwrap();
        let name = "Acme: \"Docs\" # 日本";
        let choice = if minimal { "minimal" } else { "standard" };
        run(
            temp.path(),
            &["init", "wizard", "-i"],
            &format!("{name}\nforest\n{choice}\n"),
            0,
        );
        let mut flags = vec![
            "new", "portable", "flags", "--name", name, "--theme", "forest",
        ];
        if minimal {
            flags.push("--minimal");
        }
        run(temp.path(), &flags, "", 0);
        assert_eq!(
            tree(&temp.path().join("wizard")),
            tree(&temp.path().join("flags"))
        );
        run(temp.path(), &["check", "wizard", "--strict"], "", 0);
        run(temp.path(), &["build", "wizard"], "", 0);
        let html = fs::read_to_string(temp.path().join("wizard/site/index.html")).unwrap();
        assert!(html.contains("data-zellij-theme=\"forest\""));
        assert_eq!(
            temp.path()
                .join("wizard/site/guide/getting-started.html")
                .exists(),
            !minimal
        );
    }
}
#[test]
fn hosted_wizard_matches_every_starter_and_guide_combination() {
    for mode in ["standard", "minimal", "full"] {
        for guide in [false, true] {
            let temp = tempfile::tempdir().unwrap();
            run(
                temp.path(),
                &["nd", "wizard", "-i"],
                &format!(
                    "Acme 日本\nforest\n{mode}\n{}\n",
                    if guide { "yes" } else { "no" }
                ),
                0,
            );
            let mut flags = vec![
                "new",
                "docs",
                "flags",
                "--name",
                "Acme 日本",
                "--theme",
                "forest",
            ];
            if mode == "minimal" {
                flags.push("--minimal");
            }
            if mode == "full" {
                flags.push("--full");
            }
            if guide {
                flags.push("--guide");
            }
            run(temp.path(), &flags, "", 0);
            assert_eq!(
                tree(&temp.path().join("wizard")),
                tree(&temp.path().join("flags"))
            );
            assert_eq!(
                temp.path()
                    .join("wizard/content/01-guide/index.mdx")
                    .exists(),
                guide
            );
            assert_eq!(
                temp.path()
                    .join("wizard/content/sections/examples/catalogue.yaml")
                    .exists(),
                mode == "full"
            );
        }
    }
}
#[test]
fn explicit_values_skip_questions_and_quiet_retains_required_prompts() {
    let temp = tempfile::tempdir().unwrap();
    for (command, extra, portable) in [
        ("init", vec!["--minimal"], true),
        ("nd", vec!["--full", "--guide"], false),
    ] {
        let mut args = vec![
            command, command, "-i", "--name", "Preset", "--theme", "forest",
        ];
        args.extend(extra);
        let out = run(temp.path(), &args, "", 0);
        let prompts = String::from_utf8(out.stderr).unwrap();
        assert!(!prompts.contains("Site name ["));
        assert!(!prompts.contains("Theme (name or number)"));
        assert!(!prompts.contains("Starter (name or number)"));
        assert!(!prompts.contains("Include a guide?"));
        let site = yaml(temp.path().join(command).join(if portable {
            "zel.yaml"
        } else {
            "content/site.yaml"
        }));
        assert_eq!(site["name"], "Preset");
        assert_eq!(site["theme"], "forest");
    }
    let out = run(
        temp.path(),
        &["--quiet", "init", "quiet", "-i"],
        "\n\n\n",
        0,
    );
    assert!(out.stdout.is_empty());
    assert!(
        String::from_utf8(out.stderr)
            .unwrap()
            .contains("Site name [quiet]")
    );
    assert_eq!(
        yaml(temp.path().join("quiet/zel.yaml"))["theme"],
        "cupertino"
    );
}
#[test]
fn invalid_answers_reprompt_and_numeric_choices_accept_crlf_and_cwd() {
    let temp = tempfile::tempdir().unwrap();
    fs::create_dir(temp.path().join("product")).unwrap();
    let index = zellij_core::schema::themes()
        .iter()
        .position(|t| t.name == "forest")
        .unwrap()
        + 1;
    let out = run(
        temp.path(),
        &["--cwd", "product", "init", "-i"],
        &format!(" \r\n\u{1b}bad\r\nAcme 日本\r\nmissing\r\n0\r\n{index}\r\n99\r\n2\r\n"),
        0,
    );
    let root = temp.path().join("product/docs/zellij");
    assert_eq!(yaml(root.join("zel.yaml"))["theme"], "forest");
    assert_eq!(yaml(root.join("zel.yaml"))["name"], "Acme 日本");
    assert!(!root.join("content/guide").exists());
    let prompts = String::from_utf8(out.stderr).unwrap();
    assert!(prompts.contains("site name must be nonempty"));
    assert!(prompts.contains("choose a name or number"));
}
#[test]
fn cancellation_at_every_question_never_writes_a_site_or_overwrites_with_force() {
    let temp = tempfile::tempdir().unwrap();
    let inputs = [
        "",
        ":q\n",
        "Acme\n",
        "Acme\n:q\n",
        "Acme\nforest\n:q\n",
        "Acme\nforest\nfull\n:q\n",
    ];
    for input in inputs {
        let out = run(temp.path(), &["nd", "missing/site", "-i"], input, 2);
        assert!(
            String::from_utf8(out.stderr)
                .unwrap()
                .contains("site creation cancelled")
        );
        assert!(out.stdout.is_empty());
        assert!(!temp.path().join("missing").exists());
    }
    run(
        temp.path(),
        &["nd", "existing", "--name", "Original"],
        "",
        0,
    );
    let before = tree(&temp.path().join("existing"));
    for input in inputs {
        run(temp.path(), &["nd", "existing", "-i", "--force"], input, 2);
        assert_eq!(tree(&temp.path().join("existing")), before);
    }
    run(
        temp.path(),
        &["init", "portable", "-i"],
        "Acme\nforest\n:q\n",
        2,
    );
    assert!(!temp.path().join("portable").exists());
}
#[test]
fn invalid_flags_and_destinations_fail_without_prompting_or_writes() {
    let temp = tempfile::tempdir().unwrap();
    for command in ["init", "nd"] {
        for flags in [
            vec!["--theme", "missing"],
            vec!["--name", " "],
            vec!["--name", "bad\nname"],
        ] {
            let mut args = vec![command, "new", "-i"];
            args.extend(flags);
            let out = run(temp.path(), &args, "", 2);
            assert!(
                !String::from_utf8(out.stderr)
                    .unwrap()
                    .contains("Site name [")
            );
            assert!(!temp.path().join("new").exists());
        }
    }
    fs::create_dir(temp.path().join("existing")).unwrap();
    fs::write(temp.path().join("existing/authored.txt"), "retain").unwrap();
    for command in ["init", "nd"] {
        let out = run(temp.path(), &[command, "existing", "-i"], "", 2);
        assert!(
            !String::from_utf8(out.stderr)
                .unwrap()
                .contains("Site name [")
        );
        assert_eq!(tree(&temp.path().join("existing")).len(), 1);
    }
    run(
        temp.path(),
        &["nd", "existing", "-i", "--force"],
        "Replaced\nforest\nminimal\nno\n",
        0,
    );
    assert_eq!(
        fs::read_to_string(temp.path().join("existing/authored.txt")).unwrap(),
        "retain"
    );
    run(
        temp.path(),
        &["nd", "conflict", "-i", "--minimal", "--full"],
        "",
        2,
    );
    assert!(!temp.path().join("conflict").exists());
}
#[cfg(unix)]
#[test]
fn final_plan_rejects_symlink_collisions_before_writing_any_earlier_files() {
    let temp = tempfile::tempdir().unwrap();
    fs::create_dir_all(temp.path().join("existing/content")).unwrap();
    fs::create_dir(temp.path().join("outside")).unwrap();
    fs::write(temp.path().join("outside/sentinel.txt"), "retain").unwrap();
    std::os::unix::fs::symlink(
        temp.path().join("outside"),
        temp.path().join("existing/content/sections"),
    )
    .unwrap();
    run(
        temp.path(),
        &["nd", "existing", "-i", "--force"],
        "Acme\nforest\nfull\nyes\n",
        2,
    );
    assert!(!temp.path().join("existing/zel.yaml").exists());
    assert!(!temp.path().join("existing/content/site.yaml").exists());
    assert_eq!(tree(&temp.path().join("outside")).len(), 1);
}
