use serde_json::Value;
use std::{
    fs,
    io::Write,
    path::Path,
    process::{Command, Stdio},
};
fn run(root: &Path, args: &[&str], input: &str, code: i32) -> std::process::Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_zel"))
        .args(args)
        .current_dir(root)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    // Invalid flags may fail before reading; that is the desired behavior.
    let _ = child.stdin.take().unwrap().write_all(input.as_bytes());
    let output = child.wait_with_output().unwrap();
    assert_eq!(output.status.code(), Some(code), "{args:?}: {output:?}");
    output
}
fn yaml(output: &std::process::Output) -> Value {
    serde_yaml_ng::from_slice(&output.stdout).unwrap()
}
#[test]
fn all_templates_remain_identical_and_no_arguments_do_not_wait_in_pipelines() {
    let temp = tempfile::tempdir().unwrap();
    for kind in zellij_core::schema::section_types() {
        let out = run(temp.path(), &["new", "section", &kind], "", 0);
        assert_eq!(
            String::from_utf8(out.stdout).unwrap(),
            zellij_core::schema::section_template(&kind).unwrap()
        );
        assert!(out.stderr.is_empty());
    }
    run(temp.path(), &["ns"], "", 2);
    run(temp.path(), &["ns", "unknown"], "", 2);
}
#[test]
fn wizard_and_flags_produce_the_same_pricing_section() {
    let temp = tempfile::tempdir().unwrap();
    let interactive = run(
        temp.path(),
        &["ns", "pricing-table", "-i"],
        "3\nFree\n$0\nTeam\n$20\nBusiness\nContact us\nTeam\n\n",
        0,
    );
    let flags = run(
        temp.path(),
        &[
            "ns",
            "pricing-table",
            "--tiers",
            "3",
            "--set",
            "tiers.0.name=Free",
            "--set",
            "tiers.0.price=$0",
            "--set",
            "tiers.1.name=Team",
            "--set",
            "tiers.1.price=$20",
            "--set",
            "tiers.2.name=Business",
            "--set",
            "tiers.2.price=Contact us",
            "--highlight",
            "Team",
        ],
        "",
        0,
    );
    assert_eq!(yaml(&interactive), yaml(&flags));
    assert!(
        String::from_utf8(interactive.stderr)
            .unwrap()
            .contains("Highlight tier")
    );
    assert!(flags.stderr.is_empty());
}
#[test]
fn selection_invalid_answers_and_quiet_prompts_keep_stdout_as_yaml() {
    let temp = tempfile::tempdir().unwrap();
    let out = run(
        temp.path(),
        &["--quiet", "ns", "--interactive"],
        "unknown\nhero\nHello 日本\n\n",
        0,
    );
    assert_eq!(yaml(&out)["headline"], "Hello 日本");
    assert!(
        String::from_utf8(out.stderr)
            .unwrap()
            .contains("choose a section type")
    );
    let out = run(
        temp.path(),
        &[
            "ns",
            "pricing-table",
            "-i",
            "--set",
            "tiers.0.name=Team",
            "--set",
            "tiers.0.price=42",
            "--highlight",
            "Team",
        ],
        "9\n1\n\n",
        0,
    );
    assert_eq!(yaml(&out)["tiers"][0]["price"], "42");
    assert!(
        String::from_utf8(out.stderr)
            .unwrap()
            .contains("between 1 and 4")
    );
}
#[test]
fn cancelled_or_invalid_authoring_never_creates_or_overwrites_files() {
    let temp = tempfile::tempdir().unwrap();
    for input in ["", ":q\n", "Hello\n"] {
        let args = if input == "Hello\n" {
            vec!["ns", "hero", "-i"]
        } else {
            vec!["ns", "hero", "-i", "--out", "new/hero.yaml"]
        };
        let out = run(temp.path(), &args, input, 2);
        assert!(out.stdout.is_empty());
        assert!(!temp.path().join("new").exists());
    }
    for args in [
        vec!["ns", "hero", "--tiers", "2"],
        vec!["ns", "hero", "--highlight", "1"],
        vec!["ns", "hero", "--set", "headline="],
        vec!["ns", "hero", "--set", "bad=value"],
        vec![
            "ns",
            "hero",
            "--set",
            "headline=one",
            "--set",
            "headline=two",
        ],
        vec!["ns", "pricing-table", "--tiers", "2", "--count", "tiers=3"],
        vec![
            "ns",
            "hero-split",
            "--set",
            "visual=one.svg",
            "--set",
            "terminal.body=hello",
        ],
        vec![
            "ns",
            "pricing-table",
            "--highlight",
            "1",
            "--set",
            "tiers.0.highlighted=true",
        ],
    ] {
        let mut args = args;
        args.extend(["--out", "new/test.yaml"]);
        run(temp.path(), &args, "", 2);
        assert!(!temp.path().join("new").exists());
    }
    run(
        temp.path(),
        &["ns", "hero", "section.yaml", "--set", "headline=First"],
        "",
        0,
    );
    let before = fs::read(temp.path().join("section.yaml")).unwrap();
    run(
        temp.path(),
        &["ns", "hero", "--out", "section.yaml", "--interactive"],
        "replacement\n",
        2,
    );
    assert_eq!(fs::read(temp.path().join("section.yaml")).unwrap(), before);
    run(
        temp.path(),
        &["ns", "hero", "one.yaml", "--out", "two.yaml"],
        "",
        2,
    );
}
#[test]
fn supplied_flags_survive_wizard_resizing_and_matrix_cells_follow_columns() {
    let temp = tempfile::tempdir().unwrap();
    let out = run(
        temp.path(),
        &["ns", "code-cards", "-i", "--set", "cards.1.title=Second"],
        "1\n2\n\n\n\n\n",
        0,
    );
    assert_eq!(yaml(&out)["cards"][1]["title"], "Second");
    let out = run(
        temp.path(),
        &[
            "ns",
            "matrix",
            "--count",
            "columns=3",
            "--count",
            "rows=4",
            "--set",
            "rows.0.cells.2=Yes",
        ],
        "",
        0,
    );
    assert_eq!(yaml(&out)["rows"][0]["cells"][2], "Yes");
    assert!(
        yaml(&out)["rows"]
            .as_array()
            .unwrap()
            .iter()
            .all(|r| r["cells"].as_array().unwrap().len() == 3)
    );
}
#[cfg(unix)]
#[test]
fn existing_and_dangling_output_links_are_not_replaced() {
    use std::os::unix::fs::symlink;
    let temp = tempfile::tempdir().unwrap();
    symlink("missing", temp.path().join("section.yaml")).unwrap();
    run(temp.path(), &["ns", "hero", "--out", "section.yaml"], "", 2);
    assert!(
        fs::symlink_metadata(temp.path().join("section.yaml"))
            .unwrap()
            .file_type()
            .is_symlink()
    );
}
