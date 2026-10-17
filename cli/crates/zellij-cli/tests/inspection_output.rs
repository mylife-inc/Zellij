//! Exercise the shipped process contract, including stdout/stderr and refusal
//! before filesystem/container actions. These commands need only the binary.
use serde_json::{Value, json};
use std::{
    fs,
    path::Path,
    process::{Command, Output},
};

fn run(root: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_zel"))
        .args(args)
        .current_dir(root)
        .env("PATH", root.join("no-external-tools"))
        .output()
        .unwrap()
}

fn success(output: Output) -> Vec<u8> {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        output.stderr.is_empty(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    output.stdout
}

fn json_output(root: &Path, args: &[&str]) -> Value {
    serde_json::from_slice(&success(run(root, args))).unwrap()
}

fn embedded() -> Value {
    serde_json::from_str(zellij_core::schema::SCHEMA_JSON).unwrap()
}

#[test]
fn context_emits_the_exact_embedded_reference_or_schemas_without_external_tools() {
    let temp = tempfile::tempdir().unwrap();
    assert_eq!(
        success(run(temp.path(), &["context"])),
        zellij_core::schema::CONTEXT_MARKDOWN.as_bytes()
    );
    for args in [["--json", "context"], ["context", "--json"]] {
        assert_eq!(
            success(run(temp.path(), &args)),
            zellij_core::schema::SCHEMA_JSON.as_bytes()
        );
    }
}

#[test]
fn context_file_uses_the_selected_format_and_json_keeps_stdout_empty() {
    let temp = tempfile::tempdir().unwrap();
    let output = run(
        temp.path(),
        &["context", "schema with spaces 日本.json", "--json"],
    );
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("Wrote"));
    assert_eq!(
        fs::read(temp.path().join("schema with spaces 日本.json")).unwrap(),
        zellij_core::schema::SCHEMA_JSON.as_bytes()
    );
    success(run(temp.path(), &["context", "reference.md"]));
    assert_eq!(
        fs::read(temp.path().join("reference.md")).unwrap(),
        zellij_core::schema::CONTEXT_MARKDOWN.as_bytes()
    );

    let failure = run(temp.path(), &["context", "missing/schema.json", "--json"]);
    assert_eq!(failure.status.code(), Some(1));
    assert!(failure.stdout.is_empty());
    assert!(!failure.stderr.is_empty());
}

#[test]
fn theme_catalog_matches_valid_config_names_and_has_human_descriptions() {
    let temp = tempfile::tempdir().unwrap();
    let schema = embedded();
    let output = json_output(temp.path(), &["list", "themes", "--json"]);
    assert_eq!(output, json!({ "themes": schema["themes"] }));
    let themes = output["themes"].as_array().unwrap();
    let names: Vec<_> = themes.iter().map(|theme| theme["name"].clone()).collect();
    assert_eq!(
        json!(names),
        schema["files"]["site.yaml"]["properties"]["theme"]["enum"]
    );
    let human = String::from_utf8(success(run(temp.path(), &["list", "themes"]))).unwrap();
    assert_eq!(human.lines().count(), themes.len());
    for theme in themes {
        for key in ["name", "label", "description"] {
            let text = theme[key].as_str().unwrap();
            assert!(!text.is_empty());
            assert!(human.contains(text));
        }
    }
}

#[test]
fn section_inventory_and_explanations_agree_with_complete_engine_schemas() {
    let temp = tempfile::tempdir().unwrap();
    let schema = embedded();
    let sections = json_output(temp.path(), &["list", "--json", "sections"]);
    let entries = sections["sections"].as_array().unwrap();
    assert_eq!(
        entries.len(),
        schema["sectionTypes"].as_array().unwrap().len()
    );
    for (entry, section_type) in entries
        .iter()
        .zip(schema["sectionTypes"].as_array().unwrap())
    {
        assert_eq!(&entry["type"], section_type);
        let name = section_type.as_str().unwrap();
        let definition = &schema["sections"][name];
        assert_eq!(
            json_output(temp.path(), &["explain", name, "--json"]),
            *definition
        );
        let shared = [
            "type",
            "id",
            "background",
            "padding",
            "motion",
            "titleGradient",
        ];
        let fields = definition["properties"]
            .as_object()
            .unwrap()
            .keys()
            .filter(|name| !shared.contains(&name.as_str()))
            .count();
        let required = definition["required"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|name| !shared.contains(&name.as_str().unwrap()))
            .count();
        assert_eq!(entry["fields"], fields);
        assert_eq!(entry["required"], required);
    }
    let failure = run(temp.path(), &["explain", "not-a-section", "--json"]);
    assert_eq!(failure.status.code(), Some(2));
    assert!(failure.stdout.is_empty());
    assert!(String::from_utf8_lossy(&failure.stderr).contains("zel list sections"));
}

#[test]
fn formats_and_targets_have_stable_identifiers_and_descriptions() {
    let temp = tempfile::tempdir().unwrap();
    let formats = json_output(temp.path(), &["list", "formats", "--json"]);
    let registered: std::collections::BTreeMap<_, _> =
        zellij_convert::all().map(|c| (c.id(), c.label())).collect();
    let expected: Vec<_> = registered
        .iter()
        .map(|(id, label)| json!({"id": id, "label": label}))
        .collect();
    assert_eq!(formats, json!({"formats": expected}));
    let human = String::from_utf8(success(run(temp.path(), &["list", "formats"]))).unwrap();
    for (id, label) in registered {
        assert!(human.contains(id));
        assert!(human.contains(label));
    }

    let targets = json_output(temp.path(), &["list", "targets", "--json"]);
    let entries = targets["targets"].as_array().unwrap();
    assert_eq!(
        entries
            .iter()
            .map(|t| t["id"].as_str().unwrap())
            .collect::<Vec<_>>(),
        ["static", "image", "portable"]
    );
    let human = String::from_utf8(success(run(temp.path(), &["list", "targets"]))).unwrap();
    for entry in entries {
        assert!(human.contains(entry["description"].as_str().unwrap()));
    }
}

#[test]
fn detection_distinguishes_no_match_multiple_matches_and_bad_paths() {
    let temp = tempfile::tempdir().unwrap();
    assert_eq!(
        json_output(temp.path(), &["detect", ".", "--json"]),
        json!({"source": ".", "matches": []})
    );
    fs::create_dir(temp.path().join("docs")).unwrap();
    fs::write(temp.path().join("docs/index.md"), "# Docs\n").unwrap();
    let inferred = json_output(temp.path(), &["detect", ".", "--json"]);
    assert_eq!(inferred["matches"][0]["id"], "mkdocs");
    assert_eq!(inferred["matches"][0]["strength"], "likely");
    assert!((inferred["matches"][0]["confidence"].as_f64().unwrap() - 0.6).abs() < 0.00001);
    fs::write(temp.path().join("mkdocs.yml"), "site_name: Test\n").unwrap();
    fs::write(
        temp.path().join("docusaurus.config.js"),
        "module.exports = {};\n",
    )
    .unwrap();
    let output = json_output(temp.path(), &["--json", "detect", "."]);
    let matches = output["matches"].as_array().unwrap();
    assert_eq!(
        matches
            .iter()
            .map(|m| m["id"].as_str().unwrap())
            .collect::<Vec<_>>(),
        ["docusaurus", "mkdocs"]
    );
    for found in matches {
        assert_eq!(found["confidence"], 1.0);
        assert_eq!(found["strength"], "certain");
        assert!(found["label"].is_string());
    }
    for (path, code) in [("missing", 1), ("mkdocs.yml", 2)] {
        let failure = run(temp.path(), &["detect", path, "--json"]);
        assert_eq!(failure.status.code(), Some(code));
        assert!(failure.stdout.is_empty());
        assert!(!failure.stderr.is_empty());
    }
}

#[test]
fn unsupported_json_is_rejected_before_any_command_action() {
    let temp = tempfile::tempdir().unwrap();
    let commands: &[&[&str]] = &[
        &["new", "docs", "content"],
        &["new", "section", "hero", "--out", "hero.yaml"],
        &["convert", ".", "--out", "converted"],
        &["migrate", ".", "--out", "migrated"],
        &["check"],
        &["dev"],
        &["build"],
        &["doctor"],
        &["serve"],
        &["fmt"],
        &["image", "push", "example:latest"],
        &["ci", "init", "github"],
        &["ci", "publish"],
        &["deploy", "netlify"],
        &["publish"],
        &["ship", "."],
    ];
    for command in commands {
        for leading in [true, false] {
            let mut args = command.to_vec();
            args.insert(if leading { 0 } else { args.len() }, "--json");
            let output = run(temp.path(), &args);
            assert_eq!(output.status.code(), Some(2), "{args:?}");
            assert!(output.stdout.is_empty(), "{args:?}");
            assert!(
                String::from_utf8_lossy(&output.stderr).contains("no command was run"),
                "{args:?}: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
    }
    assert_eq!(fs::read_dir(temp.path()).unwrap().count(), 0);
}

#[test]
fn verbose_is_rejected_instead_of_silently_ignored_and_help_states_json_scope() {
    let temp = tempfile::tempdir().unwrap();
    for flag in ["--verbose", "-v"] {
        for args in [
            vec![flag, "new", "docs", "content"],
            vec!["new", "docs", "content", flag],
        ] {
            let output = run(temp.path(), &args);
            assert_eq!(output.status.code(), Some(2));
            assert!(output.stdout.is_empty());
            assert!(String::from_utf8_lossy(&output.stderr).contains("unexpected argument"));
        }
    }
    assert_eq!(fs::read_dir(temp.path()).unwrap().count(), 0);
    let help = String::from_utf8(success(run(temp.path(), &["--help"]))).unwrap();
    assert!(help.contains("context, list, explain and detect"));
    assert!(!help.contains("--verbose"));
}
