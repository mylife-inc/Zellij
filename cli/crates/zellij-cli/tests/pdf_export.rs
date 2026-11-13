use std::{fs, path::Path, process::Command};
fn site(root: &Path) {
    fs::create_dir_all(root.join("content")).unwrap();
    fs::write(root.join("zel.yaml"),"version: 1\nrenderer: portable\nbook:\n  version: 1\n  title: Book\n  authors: [Author]\n  chapters: [{id: one, source: one.md}]\n").unwrap();
    fs::write(root.join("content/one.md"), "# One\n\nA short chapter.\n").unwrap();
}
fn command(root: &Path) -> Command {
    let mut c = Command::new(env!("CARGO_BIN_EXE_zel"));
    c.current_dir(root)
        .env("PATH", root.join("bin"))
        .env("DOCKER_HOST", "unix:///fixture.sock")
        .env("CONTAINER_HOST", "unix:///fixture-other.sock");
    c
}
#[test]
fn pdf_is_explicit_check_is_native_and_missing_runtime_keeps_previous_output() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    site(root);
    assert!(
        command(root)
            .args(["build"])
            .output()
            .unwrap()
            .status
            .success()
    );
    let before = fs::read(root.join("site/index.html")).unwrap();
    for args in [
        vec!["pdf", "--check"],
        vec!["export", "pdf", "--check"],
        vec!["--config", "zel.yaml", "pdf", "--check"],
    ] {
        let result = command(root).args(args).output().unwrap();
        assert!(result.status.success(), "{result:?}");
    }
    assert!(!root.join("pdf").exists());
    let result = command(root).args(["build", "--pdf"]).output().unwrap();
    assert_eq!(result.status.code(), Some(4), "{result:?}");
    assert!(String::from_utf8_lossy(&result.stderr).contains("PDF requires"));
    assert_eq!(before, fs::read(root.join("site/index.html")).unwrap());
    assert!(!root.join("site/_zellij/pdf").exists());
    assert!(
        !command(root)
            .args(["build", "--target", "static", "--pdf"])
            .output()
            .unwrap()
            .status
            .success()
    );
}
#[cfg(unix)]
#[test]
fn container_contract_receipt_cleanup_and_failure_preservation() {
    use std::os::unix::fs::PermissionsExt;
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    site(root);
    fs::create_dir(root.join("bin")).unwrap();
    fs::write(
        root.join("bin/docker"),
        r#"#!/bin/sh
case "$1" in
 info|--version) exit 0;;
 rm) echo cleaned >> cleanup; exit 0;;
 run)
  printf '%s\n' "$@" > arguments
  for arg in "$@"; do
   case "$arg" in *:/out) out=${arg%:/out};; esac
  done
  [ -n "$out" ] || exit 90
  printf '%%PDF-1.7\nfixture\n%%%%EOF\n' > "$out/book.pdf"
  printf '%s' "${PDF_LOG:-clean typesetting}" > "$out/book.log"
  exit "${PDF_EXIT:-0}";;
 *) exit 99;;
esac
"#,
    )
    .unwrap();
    fs::set_permissions(root.join("bin/docker"), fs::Permissions::from_mode(0o755)).unwrap();
    let run = |args: &[&str], exit: &str, log: &str| {
        command(root)
            .env_remove("CONTAINER_HOST")
            .env_remove("DOCKER_CONTEXT")
            .env_remove("CONTAINER_CONNECTION")
            .env_remove("CONTAINERD_ADDRESS")
            .env_remove("CONTAINERD_NAMESPACE")
            .env("PDF_EXIT", exit)
            .env("PDF_LOG", log)
            .args(args)
            .output()
            .unwrap()
    };
    let result = run(&["build", "--pdf"], "0", "clean");
    assert!(result.status.success(), "{result:?}");
    let argv = fs::read_to_string(root.join("arguments")).unwrap();
    for required in [
        "--network\nnone",
        "--read-only",
        "--cap-drop\nALL",
        "no-new-privileges",
        "-no-shell-escape",
        ":/input:ro",
        ":/out",
        "sha256:",
    ] {
        assert!(argv.contains(required), "{required}");
    }
    assert!(!argv.contains(&format!("{}:/", root.join("content").display())));
    let before = fs::read(root.join("site/index.html")).unwrap();
    let pdf = fs::read(root.join("site/_zellij/pdf/book.pdf")).unwrap();
    let record: serde_json::Value =
        serde_json::from_slice(&fs::read(root.join("site/_zellij/pdf/pdf.json")).unwrap()).unwrap();
    let book: serde_json::Value =
        serde_json::from_slice(&fs::read(root.join("site/_zellij/book.json")).unwrap()).unwrap();
    assert_eq!(record["revision"], book["revision"]);
    for (exit, log) in [
        ("1", "clean"),
        ("0", "Missing character: lost glyph"),
        ("0", "Overfull \\hbox"),
    ] {
        let result = run(&["build", "--pdf"], exit, log);
        assert_eq!(result.status.code(), Some(3), "{result:?}");
        assert_eq!(before, fs::read(root.join("site/index.html")).unwrap());
        assert_eq!(
            pdf,
            fs::read(root.join("site/_zellij/pdf/book.pdf")).unwrap()
        );
    }
    assert!(run(&["pdf"], "0", "clean").status.success());
    assert!(root.join("pdf/book.tex").is_file());
    assert_eq!(before, fs::read(root.join("site/index.html")).unwrap());
    assert_eq!(
        fs::read_to_string(root.join("cleanup"))
            .unwrap()
            .lines()
            .count(),
        5
    );
    assert!(
        command(root)
            .arg("build")
            .output()
            .unwrap()
            .status
            .success()
    );
    assert!(!root.join("site/_zellij/pdf").exists());
}
