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
fn tree(root: &Path) -> BTreeMap<String, Vec<u8>> {
    walkdir::WalkDir::new(root)
        .into_iter()
        .map(Result::unwrap)
        .filter(|e| e.file_type().is_file())
        .map(|e| {
            (
                e.path()
                    .strip_prefix(root)
                    .unwrap()
                    .to_string_lossy()
                    .replace('\\', "/"),
                fs::read(e.path()).unwrap(),
            )
        })
        .collect()
}
fn entry(output: &std::process::Output) -> Value {
    let text = String::from_utf8_lossy(&output.stdout);
    let yaml = text
        .split("Add this to pages.yaml:\n\n")
        .nth(1)
        .unwrap()
        .lines()
        .map(|line| line.strip_prefix("    ").unwrap_or(line))
        .collect::<Vec<_>>()
        .join("\n");
    serde_yaml_ng::from_str(&yaml).unwrap()
}
#[test]
fn ordinary_commands_keep_defaults_aliases_and_required_name_without_prompting() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    run(root, &["nd", "--minimal"], "", 0);
    for (args, name) in [
        (vec!["np", "true"], "true"),
        (vec!["new", "page", "42"], "42"),
    ] {
        let output = run(root, &args, "", 0);
        assert!(output.stderr.is_empty());
        assert_eq!(entry(&output)[0]["name"], name);
        assert_eq!(
            entry(&output)[0]["sections"],
            serde_json::json!([format!("{name}/hero")])
        );
        assert!(
            root.join(format!("docs/zellij/content/sections/{name}/hero.yaml"))
                .is_file()
        );
    }
    let out = run(root, &["np"], "ignored\n", 2);
    assert!(String::from_utf8(out.stderr).unwrap().contains("required"));
}
#[test]
fn wizard_and_flags_have_identical_files_and_ordered_registration_reports() {
    let temp = tempfile::tempdir().unwrap();
    for folder in ["wizard", "flags"] {
        fs::create_dir(temp.path().join(folder)).unwrap();
        fs::write(
            temp.path().join(folder).join("pages.yaml"),
            "# authored order\n[]\n",
        )
        .unwrap();
        fs::write(
            temp.path().join(folder).join("menu.yaml"),
            "# authored menu\n[]\n",
        )
        .unwrap();
    }
    let a = run(
        temp.path(),
        &["np", "-i", "--dir", "wizard"],
        "日本 docs\nfaq,hero,feature-grid\n",
        0,
    );
    let b = run(
        temp.path(),
        &[
            "new",
            "page",
            "日本 docs",
            "--sections",
            "faq,hero,feature-grid",
            "--dir",
            "flags",
        ],
        "",
        0,
    );
    assert_eq!(a.stdout, b.stdout);
    assert_eq!(
        tree(&temp.path().join("wizard")),
        tree(&temp.path().join("flags"))
    );
    let page = entry(&a);
    assert_eq!(page[0]["name"], "日本 docs");
    assert_eq!(
        page[0]["sections"],
        serde_json::json!(["日本 docs/faq", "日本 docs/hero", "日本 docs/feature-grid"])
    );
    assert_eq!(
        fs::read_to_string(temp.path().join("wizard/pages.yaml")).unwrap(),
        "# authored order\n[]\n"
    );
    assert_eq!(
        fs::read_to_string(temp.path().join("wizard/menu.yaml")).unwrap(),
        "# authored menu\n[]\n"
    );
}
#[test]
fn supplied_values_skip_questions_and_quiet_preserves_prompts_and_registration() {
    let temp = tempfile::tempdir().unwrap();
    for (folder, flags, input, name_question, section_question) in [
        (
            "both",
            vec!["preset", "--sections", "faq"],
            "",
            false,
            false,
        ),
        ("name", vec!["preset"], "\r\n", false, true),
        (
            "sections",
            vec!["--sections", "faq"],
            "preset\r\n",
            true,
            false,
        ),
    ] {
        let mut args = vec!["--quiet", "np", "-i", "--dir", folder];
        args.extend(flags);
        let out = run(temp.path(), &args, input, 0);
        let prompts = String::from_utf8_lossy(&out.stderr);
        assert_eq!(prompts.contains("Page name:"), name_question);
        assert_eq!(prompts.contains("Sections ("), section_question);
        assert!(!String::from_utf8_lossy(&out.stdout).contains('✓'));
        assert_eq!(entry(&out)[0]["name"], "preset");
        assert!(
            temp.path()
                .join(folder)
                .join(if folder == "name" {
                    "sections/preset/hero.yaml"
                } else {
                    "sections/preset/faq.yaml"
                })
                .is_file()
        );
    }
}
#[test]
fn invalid_answers_retry_and_ordered_numeric_choices_accept_crlf() {
    let temp = tempfile::tempdir().unwrap();
    let kinds = zellij_core::schema::section_types();
    let hero = kinds.iter().position(|k| k == "hero").unwrap() + 1;
    let faq = kinds.iter().position(|k| k == "faq").unwrap() + 1;
    let input = format!(
        "\r\n \r\nbad\tname\r\n../escape\r\nPlans 日本\r\nmissing\r\n0\r\n999\r\nhero,,faq\r\nhero,{hero}\r\n{faq}, {hero}\r\n"
    );
    let out = run(temp.path(), &["np", "-i", "--dir", "content"], &input, 0);
    let prompts = String::from_utf8_lossy(&out.stderr);
    assert!(prompts.contains("nonempty text"));
    assert!(prompts.contains("no section type"));
    assert!(prompts.contains("is repeated"));
    assert_eq!(
        entry(&out)[0]["sections"],
        serde_json::json!(["Plans 日本/faq", "Plans 日本/hero"])
    );
    assert!(!temp.path().join("escape").exists());
}
#[test]
fn cancellation_at_each_question_leaves_existing_and_missing_destinations_untouched() {
    let temp = tempfile::tempdir().unwrap();
    fs::create_dir(temp.path().join("existing")).unwrap();
    fs::write(temp.path().join("existing/pages.yaml"), "# retain\n[]\n").unwrap();
    let before = tree(&temp.path().join("existing"));
    for input in ["", ":q\n", "plans\n", "plans\n:q\n"] {
        for destination in ["missing/content", "existing"] {
            let out = run(temp.path(), &["np", "-i", "--dir", destination], input, 2);
            assert!(String::from_utf8_lossy(&out.stderr).contains("page creation cancelled"));
            assert!(out.stdout.is_empty());
            assert!(!temp.path().join("missing").exists());
            assert_eq!(tree(&temp.path().join("existing")), before);
        }
    }
}
#[test]
fn invalid_flags_and_late_collisions_never_leave_partial_sections() {
    let temp = tempfile::tempdir().unwrap();
    for flags in [
        vec!["--sections", "hero,missing"],
        vec!["--sections", "hero,hero"],
        vec!["../escape"],
        vec!["/absolute"],
        vec![".git/config"],
    ] {
        let mut args = vec!["np", "-i", "--dir", "missing"];
        args.extend(flags);
        let out = run(temp.path(), &args, "", 2);
        assert!(!String::from_utf8_lossy(&out.stderr).contains("Page name:"));
        assert!(!temp.path().join("missing").exists());
    }
    fs::create_dir_all(temp.path().join("content/sections/plans")).unwrap();
    fs::write(
        temp.path().join("content/sections/plans/faq.yaml"),
        "authored",
    )
    .unwrap();
    let before = tree(&temp.path().join("content"));
    run(
        temp.path(),
        &["np", "plans", "-i", "--dir", "content"],
        "hero,faq\n",
        2,
    );
    assert_eq!(tree(&temp.path().join("content")), before);
    fs::write(temp.path().join("file"), "retain").unwrap();
    run(temp.path(), &["np", "-i", "--dir", "file"], "", 2);
    assert_eq!(
        fs::read_to_string(temp.path().join("file")).unwrap(),
        "retain"
    );
}
#[test]
fn global_site_config_and_working_directory_selection_work_with_the_wizard() {
    let temp = tempfile::tempdir().unwrap();
    fs::create_dir_all(temp.path().join("product/manual/content")).unwrap();
    fs::write(
        temp.path().join("product/manual/zel.yaml"),
        "version: 1\nrenderer: hosted\n",
    )
    .unwrap();
    for (name, selection) in [
        ("from-root", vec!["--site-root", "manual"]),
        ("from-config", vec!["--config", "manual/zel.yaml"]),
    ] {
        let mut args = vec!["--cwd", "product"];
        args.extend(selection);
        args.extend(["np", name, "-i"]);
        run(temp.path(), &args, "hero\n", 0);
        assert!(
            temp.path()
                .join(format!("product/manual/content/sections/{name}/hero.yaml"))
                .is_file()
        );
    }
}
#[cfg(unix)]
#[test]
fn symlinked_section_destinations_are_rejected_before_questions() {
    let temp = tempfile::tempdir().unwrap();
    fs::create_dir(temp.path().join("content")).unwrap();
    fs::create_dir(temp.path().join("outside")).unwrap();
    fs::write(temp.path().join("outside/sentinel"), "retain").unwrap();
    std::os::unix::fs::symlink(
        temp.path().join("outside"),
        temp.path().join("content/sections"),
    )
    .unwrap();
    let out = run(temp.path(), &["np", "-i", "--dir", "content"], "", 2);
    assert!(!String::from_utf8_lossy(&out.stderr).contains("Page name:"));
    assert_eq!(tree(&temp.path().join("outside")).len(), 1);
}
