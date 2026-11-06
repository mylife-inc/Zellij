use serde_json::Value;
use std::{fs, path::Path, process::Command};

fn run(root: &Path, args: &[&str], code: i32) -> std::process::Output {
    let output = Command::new(env!("CARGO_BIN_EXE_zel"))
        .args(args)
        .current_dir(root)
        .env("PATH", root.join("no-tools"))
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(code), "{args:?}: {output:?}");
    output
}
fn yaml(path: impl AsRef<Path>) -> Value {
    serde_yaml_ng::from_slice(&fs::read(path).unwrap()).unwrap()
}
#[test]
fn full_starters_include_every_section_media_and_optional_guide_without_external_tools() {
    let temp = tempfile::tempdir().unwrap();
    let types = zellij_core::schema::section_types();
    for guide in [false, true] {
        let directory = if guide {
            "Manual 日本"
        } else {
            "docs/zellij"
        };
        let name = "Acme: \"Docs\" # 日本";
        let mut args = vec!["nd"];
        if guide {
            args.push(directory);
        }
        args.extend(["--full", "--name", name, "--theme", "cupertino"]);
        if guide {
            args.push("--guide");
        }
        let output = run(temp.path(), &args, 0);
        assert!(
            String::from_utf8(output.stdout)
                .unwrap()
                .contains("section-example.mp4")
        );
        let root = temp.path().join(directory);
        assert_eq!(yaml(root.join("zel.yaml"))["renderer"], "hosted");
        let content = root.join("content");
        assert_eq!(yaml(content.join("site.yaml"))["name"], name);
        assert_eq!(yaml(content.join("site.yaml"))["theme"], "cupertino");
        let pages = yaml(content.join("pages.yaml"));
        assert_eq!(pages.as_array().unwrap().len(), types.len() + 2);
        let catalogue = yaml(content.join("sections/examples/catalogue.yaml"));
        for kind in &types {
            let section = yaml(content.join(format!("sections/examples/{kind}.yaml")));
            assert_eq!(section["type"], kind.as_str());
            assert!(!section.to_string().contains("TODO"));
            assert!(!section.to_string().contains("replace-me"));
            assert!(
                pages
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|p| p["name"] == format!("example-{kind}"))
            );
            assert!(
                catalogue["body"]
                    .as_str()
                    .unwrap()
                    .contains(&format!("](/example-{kind})"))
            );
        }
        let menu = yaml(content.join("menu.yaml"));
        assert_eq!(menu.as_array().unwrap().len(), if guide { 3 } else { 2 });
        assert_eq!(content.join("01-guide/index.mdx").exists(), guide);
        assert_eq!(
            fs::read(content.join("assets/section-example.mp4")).unwrap(),
            include_bytes!("../../zellij-scaffold/assets/section-example.mp4")
        );
        assert_eq!(
            fs::read(content.join("assets/section-example.svg")).unwrap(),
            include_bytes!("../../zellij-scaffold/assets/section-example.svg")
        );
    }
}
#[test]
fn invalid_combinations_and_nonempty_destinations_are_preserved() {
    let temp = tempfile::tempdir().unwrap();
    run(temp.path(), &["nd", "invalid", "--full", "--minimal"], 2);
    assert!(!temp.path().join("invalid").exists());
    fs::create_dir(temp.path().join("existing")).unwrap();
    fs::write(temp.path().join("existing/authored.md"), "keep").unwrap();
    run(temp.path(), &["nd", "existing", "--full"], 2);
    assert_eq!(
        fs::read_to_string(temp.path().join("existing/authored.md")).unwrap(),
        "keep"
    );
    assert_eq!(
        fs::read_dir(temp.path().join("existing")).unwrap().count(),
        1
    );
    run(temp.path(), &["nd", "existing", "--full", "--force"], 0);
    assert_eq!(
        fs::read_to_string(temp.path().join("existing/authored.md")).unwrap(),
        "keep"
    );
    fs::write(
        temp.path()
            .join("existing/content/assets/section-example.mp4"),
        "custom media",
    )
    .unwrap();
    run(temp.path(), &["nd", "existing", "--full"], 2);
    assert_eq!(
        fs::read_to_string(
            temp.path()
                .join("existing/content/assets/section-example.mp4")
        )
        .unwrap(),
        "custom media"
    );
    run(temp.path(), &["nd", "existing", "--full", "--force"], 0);
    assert_eq!(
        fs::read(
            temp.path()
                .join("existing/content/assets/section-example.mp4")
        )
        .unwrap(),
        include_bytes!("../../zellij-scaffold/assets/section-example.mp4")
    );
}
#[test]
fn binary_collisions_are_preflighted_before_text_files_are_written() {
    use camino::Utf8Path;
    use zellij_scaffold::Scaffold;
    let temp = tempfile::tempdir().unwrap();
    fs::write(temp.path().join("media.mp4"), b"authored").unwrap();
    let mut scaffold = Scaffold::new();
    scaffold
        .add("first.yaml", "type: hero")
        .add_bytes("media.mp4", b"new video".as_slice());
    assert!(
        scaffold
            .write_new(Utf8Path::from_path(temp.path()).unwrap())
            .is_err()
    );
    assert!(!temp.path().join("first.yaml").exists());
    assert_eq!(
        fs::read(temp.path().join("media.mp4")).unwrap(),
        b"authored"
    );
}
#[cfg(unix)]
#[test]
fn force_does_not_follow_media_links() {
    let temp = tempfile::tempdir().unwrap();
    fs::create_dir_all(temp.path().join("site/content/assets")).unwrap();
    let target = temp.path().join("site/content/assets/section-example.mp4");
    std::os::unix::fs::symlink("missing", &target).unwrap();
    run(temp.path(), &["nd", "site", "--full", "--force"], 2);
    assert!(
        fs::symlink_metadata(target)
            .unwrap()
            .file_type()
            .is_symlink()
    );
    assert!(!temp.path().join("site/zel.yaml").exists());
}
