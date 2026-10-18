//! Exercise the actual scaffold → strict check → build flow without any tools on PATH.
use serde_json::{Value, json};
use std::{collections::BTreeMap, fs, path::Path, process::Command};

fn run(root: &Path, args: &[&str], code: i32) -> std::process::Output {
    let output = Command::new(env!("CARGO_BIN_EXE_zel"))
        .current_dir(root)
        .args(args)
        .env("PATH", root.join("no-external-tools"))
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(code), "{args:?}: {output:?}");
    if code == 0 {
        assert!(output.stderr.is_empty(), "{output:?}");
    }
    output
}

fn snapshot(root: &Path) -> BTreeMap<std::path::PathBuf, Vec<u8>> {
    fn visit(root: &Path, dir: &Path, files: &mut BTreeMap<std::path::PathBuf, Vec<u8>>) {
        for entry in fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                visit(root, &path, files);
            } else {
                files.insert(
                    path.strip_prefix(root).unwrap().to_owned(),
                    fs::read(path).unwrap(),
                );
            }
        }
    }
    let mut files = BTreeMap::new();
    visit(root, root, &mut files);
    files
}

fn check(root: &Path, dir: &str, pages: usize) {
    let output = run(root, &["check", dir, "--strict", "--json"], 0);
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["target"], "portable");
    assert_eq!(report["pages"], pages);
    assert_eq!(report["diagnostics"], json!([]));
}

#[test]
fn default_and_minimal_starters_build_without_changing_sources() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    for (dir, minimal, pages, files) in [("docs", false, 2, 4), ("Minimal 日本", true, 1, 2)] {
        let args = if minimal {
            vec!["new", "portable", dir, "--minimal"]
        } else {
            vec!["new", "portable"]
        };
        run(root, &args, 0);
        let source = root.join(dir);
        let before = snapshot(&source);
        assert_eq!(before.len(), files);
        check(root, dir, pages);
        run(root, &["build", dir], 0);
        let site = root.join(format!("{dir}-site"));
        let html = fs::read_to_string(site.join("index.html")).unwrap();
        assert!(html.contains(&format!("<h1 id=\"_zel-title\">{dir}</h1>")));
        assert!(html.contains("data-zellij-theme=\"cupertino\""));
        if !minimal {
            let guide = fs::read_to_string(site.join("guide/getting-started.html")).unwrap();
            assert!(guide.contains("../index.html#build-and-share"));
            assert!(site.join("assets/workflow.svg").is_file());
        }
        assert_eq!(snapshot(&source), before);
    }
}

#[test]
fn names_round_trip_and_every_theme_generates_valid_configuration() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    let names = [
        "Acme: Docs # 1",
        "TRUE",
        "null",
        "42",
        "[title](missing.md) <tag> & \"quotes\" \\ café",
        "العربية 日本",
        "line\u{2028}next",
    ];
    for (i, theme) in zellij_core::schema::themes().iter().enumerate() {
        let name = names[i % names.len()];
        let dir = format!("starter-{i}");
        run(
            root,
            &[
                "new",
                "portable",
                &dir,
                "--name",
                name,
                "--theme",
                &theme.name,
            ],
            0,
        );
        check(root, &dir, 2);
        let config = fs::read_to_string(root.join(&dir).join("zel.yaml")).unwrap();
        assert!(config.contains(&format!("name: {}", zellij_core::yaml::string(name))));
        assert!(config.contains(&format!(
            "theme: {}",
            zellij_core::yaml::string(&theme.name)
        )));
    }
    // The name cannot inject a link/HTML node or a second configuration key.
    run(root, &["build", "starter-4", "--out", "site"], 0);
    let html = fs::read_to_string(root.join("site/index.html")).unwrap();
    assert!(html.contains("[title](missing.md) &lt;tag&gt; &amp; &quot;quotes&quot; \\ café"));
    assert!(!html.contains("href=\"missing.md\""));
}

#[test]
fn current_directory_name_and_existing_dotfiles_are_preserved() {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("New project 日本");
    fs::create_dir_all(source.join(".git")).unwrap();
    fs::write(source.join(".git/config"), "keep git config").unwrap();
    fs::write(source.join(".gitignore"), "local-only\n").unwrap();
    run(&source, &["new", "portable", ".", "--minimal"], 0);
    check(&source, ".", 1);
    assert!(
        fs::read_to_string(source.join("zel.yaml"))
            .unwrap()
            .contains("name: \"New project 日本\"")
    );
    assert_eq!(
        fs::read_to_string(source.join(".git/config")).unwrap(),
        "keep git config"
    );
    assert_eq!(
        fs::read_to_string(source.join(".gitignore")).unwrap(),
        "local-only\n"
    );
}

#[test]
fn invalid_options_and_existing_content_fail_before_writes() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    for extra in [
        vec!["--name", " "],
        vec!["--name", "line\nnext"],
        vec!["--name", "tab\tname"],
        vec!["--theme", "missing"],
        vec!["--force"],
        vec!["--json"],
    ] {
        let mut args = vec!["new", "portable", "output"];
        args.extend(extra);
        run(root, &args, 2);
        assert!(!root.join("output").exists());
    }
    fs::create_dir(root.join("output")).unwrap();
    fs::write(root.join("output/notes.md"), "authored").unwrap();
    let before = snapshot(root);
    let failure = run(root, &["new", "portable", "output"], 2);
    assert!(!String::from_utf8_lossy(&failure.stderr).contains("--force"));
    assert_eq!(snapshot(root), before);
    run(root, &["new", "portable", "output/notes.md"], 1);
    assert_eq!(snapshot(root), before);
    run(root, &["new", "portable", "fresh"], 0);
    let before = snapshot(root);
    run(root, &["new", "portable", "fresh", "--minimal"], 2);
    assert_eq!(snapshot(root), before);
}

#[cfg(unix)]
#[test]
fn dangling_generated_path_is_not_followed_or_replaced() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    fs::create_dir(root.join("docs")).unwrap();
    std::os::unix::fs::symlink(root.join("outside"), root.join("docs/index.md")).unwrap();
    run(root, &["new", "portable"], 2);
    assert!(
        fs::symlink_metadata(root.join("docs/index.md"))
            .unwrap()
            .file_type()
            .is_symlink()
    );
    assert!(!root.join("outside").exists());
    assert!(!root.join("docs/zel.yaml").exists());
}
