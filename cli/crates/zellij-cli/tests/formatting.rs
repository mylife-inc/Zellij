use std::{
    fs,
    path::Path,
    process::{Command, Output},
};
fn write(root: &Path, file: &str, body: &str) {
    let file = root.join(file);
    fs::create_dir_all(file.parent().unwrap()).unwrap();
    fs::write(file, body).unwrap();
}
fn run(root: &Path, args: &[&str], code: i32) -> Output {
    let result = Command::new(env!("CARGO_BIN_EXE_zel"))
        .args(args)
        .current_dir(root)
        .env("PATH", root.join("no-tools"))
        .env("DOCKER_HOST", "fixture-invalid")
        .output()
        .unwrap();
    assert_eq!(result.status.code(), Some(code), "{args:?}: {result:?}");
    result
}
#[test]
fn formats_only_selected_yaml_and_check_never_writes() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    write(
        root,
        "docs/zellij/zel.yaml",
        "version: 1\nrenderer: hosted\n",
    );
    let source = "headline: 'Hello'\ntype: hero\ncta:\n    - label: Read\n      href: /guide\n";
    write(root, "docs/zellij/content/sections/hero.yaml", source);
    for file in [
        "docs/specs/spec.yaml",
        "docs/zellij/content/assets/download.yaml",
        "docs/zellij/content/guide.mdx",
        "docs/zellij/site/export.yaml",
        "docs/zellij/content/.draft.yaml",
    ] {
        write(root, file, "keep: [  original  ]\n");
    }
    run(root, &["fmt", "--check"], 1);
    assert_eq!(
        fs::read_to_string(root.join("docs/zellij/content/sections/hero.yaml")).unwrap(),
        source
    );
    run(root, &["fmt"], 0);
    run(root, &["fmt", "--check"], 0);
    let formatted =
        fs::read_to_string(root.join("docs/zellij/content/sections/hero.yaml")).unwrap();
    assert!(formatted.starts_with("type: hero\nheadline:"));
    assert_eq!(
        serde_yaml_ng::from_str::<serde_yaml_ng::Value>(source).unwrap(),
        serde_yaml_ng::from_str::<serde_yaml_ng::Value>(&formatted).unwrap()
    );
    for file in [
        "docs/specs/spec.yaml",
        "docs/zellij/content/assets/download.yaml",
        "docs/zellij/content/guide.mdx",
        "docs/zellij/site/export.yaml",
        "docs/zellij/content/.draft.yaml",
    ] {
        assert_eq!(
            fs::read_to_string(root.join(file)).unwrap(),
            "keep: [  original  ]\n"
        );
    }
    // A custom site and legacy raw hosted content remain explicit selections.
    write(
        root,
        "Manual 日本/zel.yaml",
        "version: 1\nrenderer: portable\nname:    Manual\n",
    );
    write(root, "Manual 日本/content/attachment.yaml", "invalid: [");
    run(root, &["fmt", "Manual 日本"], 0);
    run(&root.join("Manual 日本"), &["fmt", "--check"], 0);
    assert_eq!(
        fs::read_to_string(root.join("Manual 日本/content/attachment.yaml")).unwrap(),
        "invalid: ["
    );
    write(root, "raw/site.yaml", "name:   Raw\n");
    run(root, &["fmt", "raw"], 0);
    assert_eq!(
        fs::read_to_string(root.join("raw/site.yaml")).unwrap(),
        "name: Raw\n"
    );
}
#[test]
fn malformed_late_file_and_invalid_options_leave_every_source_unchanged() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    write(root, "zel.yaml", "version: 1\nrenderer: hosted\n");
    write(root, "content/a.yaml", "value:   untouched\n");
    write(root, "content/z.yaml", "invalid: [");
    run(root, &["fmt"], 2);
    assert_eq!(
        fs::read_to_string(root.join("content/a.yaml")).unwrap(),
        "value:   untouched\n"
    );
    run(root, &["fmt", "--json"], 2);
    run(root, &["fmt", "--force"], 2);
    run(root, &["watch", "--json"], 2);
}
#[cfg(unix)]
#[test]
fn links_are_rejected_and_permissions_survive_replacement() {
    use std::os::unix::fs::{PermissionsExt, symlink};
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    write(root, "zel.yaml", "version: 1\nrenderer: hosted\n");
    write(root, "content/site.yaml", "name:   Source\n");
    write(root, "outside.yml", "name:   Keep\n");
    symlink(root.join("outside.yml"), root.join("content/z.yaml")).unwrap();
    run(root, &["fmt"], 2);
    assert_eq!(
        fs::read_to_string(root.join("content/site.yaml")).unwrap(),
        "name:   Source\n"
    );
    fs::remove_file(root.join("content/z.yaml")).unwrap();
    fs::set_permissions(
        root.join("content/site.yaml"),
        fs::Permissions::from_mode(0o640),
    )
    .unwrap();
    run(root, &["fmt"], 0);
    assert_eq!(
        fs::metadata(root.join("content/site.yaml"))
            .unwrap()
            .permissions()
            .mode()
            & 0o777,
        0o640
    );
    assert_eq!(
        fs::read_to_string(root.join("outside.yml")).unwrap(),
        "name:   Keep\n"
    );
}

#[cfg(windows)]
#[test]
fn junctions_are_rejected_before_any_write() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    write(root, "zel.yaml", "version: 1\nrenderer: hosted\n");
    write(root, "content/a.yaml", "name:   Source\n");
    write(root, "outside/keep.yaml", "name:   Keep\n");
    let link = root.join("content").join("linked");
    assert!(
        Command::new("cmd")
            .args(["/C", "mklink", "/J"])
            .arg(&link)
            .arg(root.join("outside"))
            .status()
            .unwrap()
            .success()
    );
    let output = run(root, &["fmt"], 2);
    // Unlink before temporary-directory cleanup even if later assertions fail.
    fs::remove_dir(link).unwrap();
    assert!(String::from_utf8_lossy(&output.stderr).contains("junctions"));
    assert_eq!(
        fs::read_to_string(root.join("content/a.yaml")).unwrap(),
        "name:   Source\n"
    );
    assert_eq!(
        fs::read_to_string(root.join("outside/keep.yaml")).unwrap(),
        "name:   Keep\n"
    );
}

#[cfg(unix)]
#[test]
fn large_sites_do_not_keep_every_staged_file_open() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    write(root, "zel.yaml", "version: 1\nrenderer: hosted\n");
    for i in 0..160 {
        write(root, &format!("content/page-{i}.yaml"), "name:   Source\n");
    }
    let output = Command::new("/bin/sh")
        .args(["-c", "ulimit -n 64 && exec \"$1\" fmt", "format-limit"])
        .arg(env!("CARGO_BIN_EXE_zel"))
        .current_dir(root)
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    run(root, &["fmt", "--check"], 0);
}
