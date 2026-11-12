use std::{fs, process::Command};

#[test]
fn conventional_book_commands_work_without_external_tools_and_publish_one_record() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    let site = root.join("docs/zellij");
    fs::create_dir_all(site.join("content")).unwrap();
    fs::write(site.join("zel.yaml"), "version: 1\nrenderer: portable\nbook:\n  version: 1\n  title: My book\n  authors: [Author]\n  chapters: [{id: chapter, source: chapter.md}]\n").unwrap();
    fs::write(site.join("content/chapter.md"), "# First chapter\n").unwrap();
    let run = |args: &[&str]| {
        Command::new(env!("CARGO_BIN_EXE_zel"))
            .current_dir(root)
            .args(args)
            .env("PATH", root.join("no-tools"))
            .env("DOCKER_HOST", "unix:///fixture-docker.sock")
            .env("CONTAINER_HOST", "unix:///fixture-podman.sock")
            .output()
            .unwrap()
    };
    let checked = run(&["check", "--json"]);
    assert!(
        checked.status.success(),
        "{}",
        String::from_utf8_lossy(&checked.stderr)
    );
    let report: serde_json::Value = serde_json::from_slice(&checked.stdout).unwrap();
    assert_eq!(report["book"]["entries"][1]["number"], "1");
    assert!(!site.join("site").exists());
    let built = run(&["build"]);
    assert!(
        built.status.success(),
        "{}",
        String::from_utf8_lossy(&built.stderr)
    );
    let publication: serde_json::Value =
        serde_json::from_slice(&fs::read(site.join("site/_zellij/book.json")).unwrap()).unwrap();
    assert_eq!(report["book"], publication);
    fs::write(site.join("content/draft.md"), "# Unlisted draft").unwrap();
    assert!(!run(&["build"]).status.success());
    assert_eq!(
        publication,
        serde_json::from_slice::<serde_json::Value>(
            &fs::read(site.join("site/_zellij/book.json")).unwrap()
        )
        .unwrap()
    );
}
