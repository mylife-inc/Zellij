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
fn json_lists_support_explicit_scalar_refinements_and_optional_field_removal() {
    let temp = tempfile::tempdir().unwrap();
    let cards = serde_json::json!([
        {"title":"One", "featured":true, "code":{"title":"Shell", "body":"$(touch sentinel)\n日本\t42"}},
        {"title":"Two", "code":{"body":"true: literal"}}
    ]);
    let assignment = format!("cards={cards}");
    let refinements = [
        "--set",
        "cards.0.title=Updated 日本",
        "--unset",
        "cards.0.featured",
        "--unset",
        "cards.0.code.title",
    ];
    let mut first = vec!["ns", "code-cards", "--set-json", &assignment];
    first.extend(refinements);
    let out = run(temp.path(), &first, "", 0);
    let mut expected = cards;
    expected[0]["title"] = "Updated 日本".into();
    expected[0].as_object_mut().unwrap().remove("featured");
    expected[0]["code"].as_object_mut().unwrap().remove("title");
    assert_eq!(yaml(&out)["cards"], expected);
    assert!(!temp.path().join("sentinel").exists());
    let mut reversed = vec!["new", "section", "code-cards"];
    reversed.extend(refinements);
    reversed.extend(["--set-json", &assignment]);
    assert_eq!(run(temp.path(), &reversed, "", 0).stdout, out.stdout);
    assert!(out.stderr.is_empty());
}
#[test]
fn typed_json_scalars_and_complete_matrix_values_keep_their_types() {
    let temp = tempfile::tempdir().unwrap();
    let out = run(
        temp.path(),
        &[
            "ns",
            "media-panel",
            "--set-json",
            "scrim=0.5",
            "--set-json",
            r#"video="assets/movie.mp4""#,
        ],
        "",
        0,
    );
    assert_eq!(yaml(&out)["scrim"], 0.5);
    assert!(yaml(&out).get("image").is_none());
    let out = run(
        temp.path(),
        &["ns", "code-cards", "--set-json", "cards.0.featured=true"],
        "",
        0,
    );
    assert_eq!(yaml(&out)["cards"][0]["featured"], true);
    let out = run(
        temp.path(),
        &[
            "ns",
            "hero",
            "--set-json",
            r#"headline="42""#,
            "--set-json",
            "cta=[]",
        ],
        "",
        0,
    );
    assert_eq!(yaml(&out)["headline"], "42");
    assert_eq!(yaml(&out)["cta"], serde_json::json!([]));
    let columns = r#"columns=[{"label":"A"},{"label":"B"},{"label":"C"}]"#;
    let rows =
        r#"rows=[{"label":"One","cells":["1","2","3"]},{"label":"Two","cells":["4","5","6"]}]"#;
    let out = run(
        temp.path(),
        &["ns", "matrix", "--set-json", columns, "--set-json", rows],
        "",
        0,
    );
    assert_eq!(yaml(&out)["rows"][1]["cells"][2], "6");
    run(temp.path(), &["ns", "matrix", "--set-json", columns], "", 2);
}
#[test]
fn invalid_json_or_removal_fails_before_any_prompt_or_output_write() {
    let temp = tempfile::tempdir().unwrap();
    for (kind, flags) in [
        ("hero", vec!["--set-json", "headline=42"]),
        ("hero", vec!["--set-json", "headline=null"]),
        ("hero", vec!["--set-json", "cta=["]),
        ("hero", vec!["--set-json", "cta=[] trailing"]),
        ("hero", vec!["--set-json", r#"cta=[{"label":"Read"}]"#]),
        (
            "hero",
            vec![
                "--set-json",
                r#"cta=[{"label":"Read","href":"/","typo":true}]"#,
            ],
        ),
        ("hero", vec!["--set-json", r#"type="hero""#]),
        ("hero", vec!["--unset", "headline"]),
        ("hero", vec!["--unset", "typo"]),
        ("hero", vec!["--unset", "background.gradient"]),
        ("hero-split", vec!["--unset", "terminal"]),
        ("code-cards", vec!["--unset", "cards.0"]),
        ("code-cards", vec!["--unset", "cards.0.code.body"]),
        ("code-cards", vec!["--unset", "cards.9.featured"]),
    ] {
        let mut args = vec!["ns", kind, "-i", "--out", "missing/section.yaml"];
        args.extend(flags);
        let out = run(temp.path(), &args, "", 2);
        assert!(out.stdout.is_empty());
        assert!(
            !String::from_utf8(out.stderr)
                .unwrap()
                .contains("Press Enter")
        );
        assert!(!temp.path().join("missing").exists());
    }
}
#[test]
fn ambiguous_assignments_and_conflicting_counts_or_highlights_are_rejected() {
    let temp = tempfile::tempdir().unwrap();
    let tiers = r#"tiers=[{"name":"Free","price":"0"}]"#;
    for (kind, flags) in [
        (
            "hero",
            vec![
                "--set-json",
                r#"background="glow""#,
                "--set",
                "background.gradient.0=accent",
            ],
        ),
        ("hero", vec!["--set-json", "cta=[]", "--set-json", "cta=[]"]),
        (
            "hero",
            vec!["--set-json", r#"headline="A""#, "--set", "headline=A"],
        ),
        ("hero", vec!["--set-json", "cta=[]", "--unset", "cta"]),
        ("hero", vec!["--unset", "badge", "--unset", "badge"]),
        ("hero", vec!["--set", "badge=A", "--unset", "badge"]),
        ("hero", vec!["--count", "cta=1", "--set-json", "cta=[]"]),
        ("hero", vec!["--count", "cta=1", "--unset", "cta"]),
        (
            "hero-split",
            vec![
                "--set-json",
                r#"terminal={"body":"Hello"}"#,
                "--set-json",
                r#"terminal.body="Hello""#,
            ],
        ),
        (
            "hero-split",
            vec![
                "--set-json",
                r#"terminal.body="Hello""#,
                "--set-json",
                r#"terminal={"body":"Hello"}"#,
            ],
        ),
        (
            "hero-split",
            vec![
                "--set-json",
                r#"terminal={"body":"Hello"}"#,
                "--set",
                "visual=assets/logo.svg",
            ],
        ),
        ("pricing-table", vec!["--tiers", "2", "--set-json", tiers]),
        (
            "pricing-table",
            vec!["--set-json", tiers, "--highlight", "1"],
        ),
        (
            "pricing-table",
            vec![
                "--set-json",
                r#"tiers.0={"name":"Free","price":"0"}"#,
                "--highlight",
                "none",
            ],
        ),
    ] {
        let mut args = vec!["ns", kind, "--out", "missing/section.yaml"];
        args.extend(flags);
        run(temp.path(), &args, "", 2);
        assert!(!temp.path().join("missing").exists());
    }
    let out = run(
        temp.path(),
        &[
            "ns",
            "pricing-table",
            "--tiers",
            "1",
            "--set-json",
            tiers,
            "--set",
            "tiers.0.highlighted=true",
        ],
        "",
        0,
    );
    assert_eq!(yaml(&out)["tiers"][0]["highlighted"], true);
}
#[test]
fn wizard_preserves_complete_json_subtrees_and_explicit_omissions() {
    let temp = tempfile::tempdir().unwrap();
    let tiers = r#"tiers=[{"name":"Free","price":"0","description":"Remove me"},{"name":"Team","price":"20","highlighted":true}]"#;
    let out = run(
        temp.path(),
        &[
            "--quiet",
            "ns",
            "pricing-table",
            "-i",
            "--set-json",
            tiers,
            "--unset",
            "tiers.0.description",
        ],
        "\n",
        0,
    );
    let text = String::from_utf8_lossy(&out.stderr);
    assert!(
        !text.contains("tiers (items)")
            && !text.contains("tiers.0")
            && !text.contains("Highlight tier")
    );
    assert!(text.contains("Output file"));
    let flags = run(
        temp.path(),
        &[
            "ns",
            "pricing-table",
            "--set-json",
            tiers,
            "--unset",
            "tiers.0.description",
        ],
        "",
        0,
    );
    assert_eq!(out.stdout, flags.stdout);
    assert!(yaml(&flags)["tiers"][0].get("description").is_none());
    assert_eq!(yaml(&flags)["tiers"][1]["highlighted"], true);
}
#[test]
fn structured_authoring_honors_cwd_quiet_cancellation_and_existing_files() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    fs::create_dir(root.join("Product 日本")).unwrap();
    let args = [
        "--cwd",
        "Product 日本",
        "--quiet",
        "ns",
        "hero",
        "section.yaml",
        "--set-json",
        r#"headline="Authored""#,
        "--unset",
        "badge",
    ];
    let out = run(root, &args, "", 0);
    assert!(out.stdout.is_empty() && out.stderr.is_empty());
    let before = fs::read(root.join("Product 日本/section.yaml")).unwrap();
    run(root, &args, "", 2);
    assert_eq!(
        fs::read(root.join("Product 日本/section.yaml")).unwrap(),
        before
    );
    for input in ["", ":q\n"] {
        run(
            root,
            &[
                "ns",
                "hero",
                "-i",
                "--set-json",
                "cta=[]",
                "--out",
                "missing/section.yaml",
            ],
            input,
            2,
        );
        assert!(!root.join("missing").exists());
    }
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

#[test]
fn composition_authors_nested_variants_and_enforces_tree_limits_before_writing() {
    let temp = tempfile::tempdir().unwrap();
    let nodes = serde_json::json!([{ "type":"group", "layout":{"kind":"split"}, "children":[
        {"type":"heading", "level":1, "text":"Original"},
        {"type":"image", "image":"assets/diagram.svg", "alt":"A diagram"}
    ]}]);
    let assignment = format!("children={nodes}");
    let out = run(
        temp.path(),
        &[
            "ns",
            "compose",
            "--set-json",
            &assignment,
            "--set",
            "children.0.children.0.text=Updated 日本",
            "--set",
            "children.0.children.1.alt=Updated diagram",
        ],
        "",
        0,
    );
    assert_eq!(
        yaml(&out)["children"][0]["children"][0]["text"],
        "Updated 日本"
    );
    assert_eq!(
        yaml(&out)["children"][0]["children"][1]["alt"],
        "Updated diagram"
    );
    let text = serde_json::json!({"type":"text", "text":"Readable"});
    let group = |children: Vec<Value>| serde_json::json!({"type":"group", "children":children});
    let deep = group(vec![group(vec![group(vec![group(vec![text.clone()])])])]);
    let large = vec![group(vec![text.clone(); 18]); 7]; // 133 nodes
    let headings = serde_json::json!([{"type":"heading","text":"A","level":1},
        {"type":"group","children":[{"type":"heading","text":"B","level":1}]}]);
    for value in [
        serde_json::json!([deep]),
        serde_json::json!(large),
        headings,
    ] {
        let assignment = format!("children={value}");
        run(
            temp.path(),
            &[
                "ns",
                "compose",
                "--set-json",
                &assignment,
                "--out",
                "blocked.yaml",
            ],
            "",
            2,
        );
        assert!(!temp.path().join("blocked.yaml").exists());
    }
}

#[test]
fn composition_explain_names_every_node_and_layout_variant() {
    let temp = tempfile::tempdir().unwrap();
    let output = run(temp.path(), &["explain", "compose"], "", 0);
    let text = String::from_utf8(output.stdout).unwrap();
    for name in [
        "heading", "text", "image", "button", "stat", "divider", "group",
    ] {
        assert!(text.contains(&format!("`children` ({name})")), "{text}");
    }
    for name in ["stack", "grid", "split"] {
        assert!(text.contains(&format!("`layout` ({name})")), "{text}");
    }
    assert!(text.contains("128 nodes"));
    let output = run(temp.path(), &["explain", "compose", "--json"], "", 0);
    let value: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["properties"]["children"]["maxItems"], 24);
}

#[test]
fn named_presets_are_listed_and_generated_offline_as_editable_yaml() {
    let temp = tempfile::tempdir().unwrap();
    let catalog = run(temp.path(), &["presets", "--json"], "", 0);
    assert_eq!(
        catalog.stdout,
        run(temp.path(), &["list", "presets", "--json"], "", 0).stdout
    );
    let value: Value = serde_json::from_slice(&catalog.stdout).unwrap();
    assert_eq!(value["presets"].as_array().unwrap().len(), 4);
    let context: Value =
        serde_json::from_slice(&run(temp.path(), &["context", "--json"], "", 0).stdout).unwrap();
    for preset in value["presets"].as_array().unwrap() {
        let name = preset["name"].as_str().unwrap();
        let inferred = run(temp.path(), &["ns", "-p", name], "", 0);
        let explicit = run(
            temp.path(),
            &["new", "section", "compose", "--preset", name],
            "",
            0,
        );
        assert_eq!(inferred.stdout, explicit.stdout);
        let interactive = run(
            temp.path(),
            &["ns", "-p", name, "--interactive"],
            &"\n".repeat(512),
            0,
        );
        assert_eq!(inferred.stdout, interactive.stdout);
        assert_eq!(
            String::from_utf8(inferred.stdout.clone()).unwrap(),
            context["sectionPresets"][name]["yaml"].as_str().unwrap()
        );
        assert_eq!(yaml(&inferred)["type"], "compose");
        assert!(yaml(&inferred).get("preset").is_none());
    }
    assert_eq!(fs::read_dir(temp.path()).unwrap().count(), 0);
}

#[test]
fn preset_edits_keep_nested_types_and_apply_normal_output_ownership_rules() {
    let temp = tempfile::tempdir().unwrap();
    let output = run(
        temp.path(),
        &["ns", "-p", "product-hero", "--set", "children.0.text=01"],
        "",
        0,
    );
    assert_eq!(yaml(&output)["children"][0]["text"], "01");
    let output = run(
        temp.path(),
        &[
            "ns",
            "-p",
            "media-accordion",
            "--count",
            "children.1.items=2",
            "--set",
            "children.1.items.0.title=My own detail",
        ],
        "",
        0,
    );
    assert_eq!(
        yaml(&output)["children"][1]["items"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    assert_eq!(
        yaml(&output)["children"][1]["items"][0]["title"],
        "My own detail"
    );
    let args = ["ns", "-p", "split-story", "--out", "story.yaml"];
    run(temp.path(), &args, "", 0);
    let original = fs::read(temp.path().join("story.yaml")).unwrap();
    run(temp.path(), &args, "", 2);
    assert_eq!(fs::read(temp.path().join("story.yaml")).unwrap(), original);
}

#[test]
fn invalid_presets_and_accordion_sizes_fail_before_prompting_or_writing() {
    let temp = tempfile::tempdir().unwrap();
    for options in [
        vec!["-p", "unknown"],
        vec!["hero", "-p", "product-hero"],
        vec!["-p", "media-accordion", "--count", "children.1.items=9"],
        vec!["media-accordion", "--count", "items=1"],
        vec!["media-accordion", "--set", "itemHeadingLevel=1"],
    ] {
        let mut args = vec!["ns", "-i", "--out", "missing/section.yaml"];
        args.extend(options);
        let output = run(temp.path(), &args, "", 2);
        assert!(output.stdout.is_empty());
        assert!(
            !String::from_utf8(output.stderr)
                .unwrap()
                .contains("Press Enter")
        );
        assert!(!temp.path().join("missing").exists());
    }
    let output = run(
        temp.path(),
        &["ns", "media-accordion", "--count", "items=8"],
        "",
        0,
    );
    assert_eq!(yaml(&output)["items"].as_array().unwrap().len(), 8);
    let explain = run(temp.path(), &["explain", "compose"], "", 0);
    let text = String::from_utf8(explain.stdout).unwrap();
    assert!(text.contains("media-accordion") && text.contains("zel ns -p product-hero"));
}
