use std::{fs, path::Path, process::Command};

fn zel(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_zel"))
        .args(args)
        .output()
        .unwrap()
}

fn mkdocs(root: &Path, nav: &str) {
    fs::create_dir_all(root.join("docs")).unwrap();
    fs::write(
        root.join("mkdocs.yml"),
        format!("site_name: Test\nnav:\n  - {nav}\n"),
    )
    .unwrap();
    fs::write(root.join("docs/index.md"), "# Original\n").unwrap();
}

fn refused(output: std::process::Output) {
    assert!(
        !output.status.success(),
        "unexpected success: {}",
        String::from_utf8_lossy(&output.stdout)
    );
    assert!(String::from_utf8_lossy(&output.stderr).contains("error:"));
}

#[test]
fn conversion_preserves_existing_destination() {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("source");
    let out = temp.path().join("out");
    mkdocs(&source, "index.md");
    fs::create_dir(&out).unwrap();
    fs::write(out.join("site.yaml"), "authored").unwrap();
    refused(zel(&[
        "convert",
        source.to_str().unwrap(),
        "--out",
        out.to_str().unwrap(),
    ]));
    assert_eq!(
        fs::read_to_string(out.join("site.yaml")).unwrap(),
        "authored"
    );
    assert!(!out.join("01-guide").exists());
}

#[test]
fn conversion_refuses_overlapping_roots_even_in_dry_run() {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("source");
    mkdocs(&source, "index.md");
    for out in [
        source.clone(),
        source.join("converted"),
        temp.path().to_owned(),
    ] {
        refused(zel(&[
            "convert",
            source.to_str().unwrap(),
            "--out",
            out.to_str().unwrap(),
            "--dry-run",
        ]));
    }
    assert!(!source.join("site.yaml").exists());
    assert!(!source.join("converted").exists());
}

#[test]
fn conversion_refuses_parent_paths_before_writing_anything() {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("source");
    let out = temp.path().join("out");
    mkdocs(&source, "../../escape.md");
    fs::write(temp.path().join("escape.md"), "# Do not import").unwrap();
    refused(zel(&[
        "convert",
        source.to_str().unwrap(),
        "--out",
        out.to_str().unwrap(),
    ]));
    assert!(!out.exists());
    assert!(!temp.path().join("escape.mdx").exists());
}

#[test]
fn docusaurus_refuses_parent_document_ids() {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("source");
    fs::create_dir_all(source.join("docs")).unwrap();
    fs::write(
        source.join("sidebars.js"),
        "module.exports = { docs: ['../../escape'] };",
    )
    .unwrap();
    fs::write(temp.path().join("escape.md"), "# Private").unwrap();
    let out = temp.path().join("out");
    refused(zel(&[
        "convert",
        source.to_str().unwrap(),
        "--from",
        "docusaurus",
        "--out",
        out.to_str().unwrap(),
    ]));
    assert!(!out.exists());
}

#[test]
fn page_collision_does_not_write_earlier_sections() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    fs::create_dir_all(root.join("sections/home")).unwrap();
    fs::write(root.join("sections/home/faq.yaml"), "authored").unwrap();
    refused(zel(&[
        "new",
        "page",
        "home",
        "--sections",
        "hero,faq",
        "--dir",
        root.to_str().unwrap(),
    ]));
    assert_eq!(
        fs::read_to_string(root.join("sections/home/faq.yaml")).unwrap(),
        "authored"
    );
    assert!(!root.join("sections/home/hero.yaml").exists());
}

#[test]
fn page_names_cannot_escape_the_content_root() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("content");
    refused(zel(&[
        "new",
        "page",
        "../../escape",
        "--dir",
        root.to_str().unwrap(),
    ]));
    assert!(!temp.path().join("escape").exists());
    assert!(!root.exists());
}

#[test]
fn conversion_and_new_page_still_write_valid_noncolliding_paths() {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("source");
    let out = temp.path().join("out");
    mkdocs(&source, "index.md");
    assert!(
        zel(&[
            "convert",
            source.to_str().unwrap(),
            "--out",
            out.to_str().unwrap()
        ])
        .status
        .success()
    );
    assert!(out.join("01-guide/index.mdx").is_file());
    assert!(
        zel(&["new", "page", "pricing", "--dir", out.to_str().unwrap()])
            .status
            .success()
    );
    assert!(out.join("sections/pricing/hero.yaml").is_file());
}

#[cfg(unix)]
#[test]
fn converter_refuses_symlinked_documents() {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("source");
    mkdocs(&source, "linked.md");
    fs::write(temp.path().join("private.md"), "# Private").unwrap();
    std::os::unix::fs::symlink(
        temp.path().join("private.md"),
        source.join("docs/linked.md"),
    )
    .unwrap();
    let out = temp.path().join("out");
    refused(zel(&[
        "convert",
        source.to_str().unwrap(),
        "--out",
        out.to_str().unwrap(),
    ]));
    assert!(!out.exists());
}

#[cfg(unix)]
#[test]
fn new_page_refuses_symlinked_parent_without_modifying_target() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("content");
    let outside = temp.path().join("outside");
    fs::create_dir_all(&root).unwrap();
    fs::create_dir(&outside).unwrap();
    std::os::unix::fs::symlink(&outside, root.join("sections")).unwrap();
    refused(zel(&[
        "new",
        "page",
        "home",
        "--dir",
        root.to_str().unwrap(),
    ]));
    assert_eq!(fs::read_dir(outside).unwrap().count(), 0);
}
