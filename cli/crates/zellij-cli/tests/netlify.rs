//! CLI planning/prerequisite behavior never uses user credentials or the network.
use std::{
    fs,
    path::Path,
    process::{Command, Output},
};
fn fixture(root: &Path, folder: &str, settings: &str) {
    let root = root.join(folder);
    fs::create_dir_all(root.join("content")).unwrap();
    fs::write(
        root.join("zel.yaml"),
        format!("version: 1\nrenderer: portable\n{settings}\n"),
    )
    .unwrap();
    fs::write(root.join("content/index.md"), "# Manual\n").unwrap();
}
fn command(root: &Path) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_zel"));
    command
        .current_dir(root)
        .env("PATH", root.join("no-tools"))
        .env_remove("NETLIFY_AUTH_TOKEN")
        .env_remove("NETLIFY_SITE_ID");
    command
}
fn run(root: &Path, args: &[&str], code: i32) -> Output {
    let output = command(root).args(args).output().unwrap();
    assert_eq!(output.status.code(), Some(code), "{args:?}: {output:?}");
    output
}
const SETTINGS: &str = "publish: {action: deploy, provider: netlify, site: saved.netlify.app}";
#[test]
fn canonical_short_and_saved_forms_resolve_the_same_plan_without_tools_or_credentials() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    fixture(root, "docs/zellij", SETTINGS);
    let expected = run(root, &["deploy", "netlify", "--dry-run"], 0);
    for args in [
        vec!["netlify", "--dry-run"],
        vec!["publish", "--dry-run"],
        vec!["pub", "--dry-run"],
        vec!["deploy", "--dry-run"],
        vec!["pub", "--to", "netlify", "--dry-run"],
    ] {
        let actual = run(root, &args, 0);
        assert_eq!(actual.stdout, expected.stdout);
        assert_eq!(actual.stderr, expected.stderr);
    }
    assert!(String::from_utf8_lossy(&expected.stdout).contains("draft preview"));
    fixture(root, "docs/日本 Manual", SETTINGS);
    let result = run(
        root,
        &[
            "netlify",
            "docs/日本 Manual",
            "--site",
            "override.netlify.app",
            "--prod",
            "--dry-run",
        ],
        0,
    );
    let text = String::from_utf8_lossy(&result.stdout);
    assert!(
        text.contains("production")
            && text.contains("override.netlify.app")
            && !text.contains("saved.netlify.app")
    );
    run(&root.join("docs/日本 Manual"), &["pub", "--dry-run"], 0);
    assert!(!root.join("docs/zellij/site").exists());
    assert!(!root.join("docs/日本 Manual/site").exists());
}
#[test]
fn site_precedence_and_explicit_intent_work_without_saved_publication() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    fixture(root, ".", SETTINGS);
    for (args, expected) in [
        (vec!["pub", "--dry-run"], "environment.netlify.app"),
        (
            vec!["netlify", "--site-id", "explicit.netlify.app", "--dry-run"],
            "explicit.netlify.app",
        ),
    ] {
        let result = command(root)
            .env("NETLIFY_SITE_ID", "environment.netlify.app")
            .args(args)
            .output()
            .unwrap();
        assert!(result.status.success(), "{result:?}");
        assert!(String::from_utf8_lossy(&result.stdout).contains(expected));
    }
    fixture(root, ".", "");
    run(
        root,
        &["netlify", "--site-id", "explicit.netlify.app", "--dry-run"],
        0,
    );
    run(
        root,
        &[
            "pub",
            "--to",
            "netlify",
            "--site",
            "explicit.netlify.app",
            "--dry-run",
        ],
        0,
    );
    run(
        root,
        &["pub", "--site", "explicit.netlify.app", "--dry-run"],
        2,
    );
    run(root, &["netlify", "--dry-run"], 2);
    fixture(root, ".", "publish: {action: deploy, provider: netlify}");
    run(
        root,
        &["pub", "--site", "explicit.netlify.app", "--dry-run"],
        0,
    );
}
#[test]
fn invalid_settings_options_and_missing_credentials_stop_before_any_build() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    fixture(root, ".", SETTINGS);
    for args in [
        vec!["netlify", "--json"],
        vec!["netlify", "--tag", "acme/image"],
        vec!["deploy", "vercel"],
        vec!["pub", "--to", "cloudflare"],
        vec!["deploy", "netlify", "--to", "fly"],
        vec!["netlify", "--site", "https://user:secret@site.test"],
    ] {
        let result = run(root, &args, 2);
        assert!(!String::from_utf8_lossy(&result.stdout).contains("Building"));
        assert!(!String::from_utf8_lossy(&result.stderr).contains("secret"));
    }
    let result = run(root, &["netlify"], 2);
    assert!(String::from_utf8_lossy(&result.stderr).contains("NETLIFY_AUTH_TOKEN"));
    assert!(!root.join("site").exists());
    for settings in [
        "publish: {action: deploy, provider: netlify, site: 123}",
        "publish: {action: deploy, provider: netlify, site: true}",
        "publish: {action: deploy, provider: netlify, site: null}",
        "publish: {action: deploy, provider: netlify, site: ''}",
        "publish: {action: deploy, provider: netlify, site: 'https://site.test'}",
        "publish: {action: deploy, provider: netlify, image: static}",
        "publish: {action: deploy, provider: netlify, token: secret}",
        "publish: {action: deploy, provider: netlify, prod: true}",
    ] {
        fixture(root, ".", settings);
        run(root, &["pub", "--dry-run"], 2);
    }
    // Loading shared publication config also remains valid for ordinary builds.
    fixture(root, ".", SETTINGS);
    run(root, &["build"], 0);
    assert!(root.join("site/index.html").is_file());
}
