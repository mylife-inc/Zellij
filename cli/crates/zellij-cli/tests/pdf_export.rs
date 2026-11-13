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
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    site(root);
    runtime(root);
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
    // Cancel our own fixture child only after its runtime invocation starts.
    let mut child = command(root)
        .env_remove("CONTAINER_HOST")
        .env_remove("DOCKER_CONTEXT")
        .env_remove("CONTAINER_CONNECTION")
        .env_remove("CONTAINERD_ADDRESS")
        .env_remove("CONTAINERD_NAMESPACE")
        .env("PDF_WAIT", "1")
        .args(["build", "--pdf"])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    while !root.join("waiting").exists() {
        assert!(
            std::time::Instant::now() < deadline,
            "runtime never started"
        );
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    assert!(
        Command::new("/bin/kill")
            .args(["-INT", &child.id().to_string()])
            .status()
            .unwrap()
            .success()
    );
    let status = loop {
        if let Some(status) = child.try_wait().unwrap() {
            break status;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "cancelled child did not exit"
        );
        std::thread::sleep(std::time::Duration::from_millis(20));
    };
    assert_eq!(status.code(), Some(3));
    assert_eq!(before, fs::read(root.join("site/index.html")).unwrap());
    assert_eq!(
        fs::read_to_string(root.join("cleanup"))
            .unwrap()
            .lines()
            .count(),
        6
    );
    // Each supported rootless runtime maps the caller to container UID/GID 0.
    // Explicit userns=host prevents custom per-container remapping of bind mounts.
    for (program, selector, info) in [
        ("docker", "DOCKER_HOST", r#"["name=rootless"]"#),
        ("podman", "CONTAINER_HOST", "true"),
        ("nerdctl", "CONTAINERD_ADDRESS", r#"["name=rootless"]"#),
    ] {
        if program != "docker" {
            fs::copy(root.join("bin/docker"), root.join("bin").join(program)).unwrap();
        }
        let result = command(root)
            .env_remove("DOCKER_HOST")
            .env_remove("CONTAINER_HOST")
            .env_remove("DOCKER_CONTEXT")
            .env_remove("CONTAINER_CONNECTION")
            .env_remove("CONTAINERD_ADDRESS")
            .env_remove("CONTAINERD_NAMESPACE")
            .env(selector, "unix:///fixture.sock")
            .env("PDF_INFO", info)
            .args(["--quiet", "--no-color", "pdf"])
            .output()
            .unwrap();
        assert!(result.status.success(), "{program}: {result:?}");
        let argv = fs::read_to_string(root.join("arguments")).unwrap();
        assert!(argv.contains("--user\n0:0\n"));
        assert!(argv.contains("--userns\nhost\n"));
    }
    let result = command(root)
        .env_remove("CONTAINER_HOST")
        .env("PDF_INFO", "unknown")
        .args(["build", "--pdf"])
        .output()
        .unwrap();
    assert_eq!(result.status.code(), Some(4));
    assert!(String::from_utf8_lossy(&result.stderr).contains("user mapping"));
    assert_eq!(before, fs::read(root.join("site/index.html")).unwrap());
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

#[cfg(unix)]
fn runtime(root: &Path) {
    use std::os::unix::fs::PermissionsExt;
    fs::create_dir(root.join("bin")).unwrap();
    fs::write(
        root.join("bin/docker"),
        r#"#!/bin/sh
case "$1" in
 info) if [ "$2" = --format ]; then printf '%s\n' "${PDF_INFO:-[]}"; fi; exit 0;;
 --version) exit 0;;
 rm) echo cleaned >> cleanup; exit 0;;
 run)
  printf '%s\n' "$@" > arguments
  if [ -f hold ]; then echo holding > holding; while [ -f hold ]; do /bin/sleep 0.05; done; fi
  for arg in "$@"; do
   case "$arg" in *:/out) out=${arg%:/out};; esac
  done
  [ -n "$out" ] || exit 90
  printf '%%PDF-1.7\nfixture\n%%%%EOF\n' > "$out/book.pdf"
  printf '%s' "${PDF_LOG:-clean typesetting}" > "$out/book.log"
  if [ "${PDF_WAIT:-0}" = 1 ]; then echo waiting > waiting; while :; do :; done; fi
  exit "${PDF_EXIT:-0}";;
 *) exit 99;;
esac
"#,
    )
    .unwrap();
    fs::set_permissions(root.join("bin/docker"), fs::Permissions::from_mode(0o755)).unwrap();
}

#[test]
fn configured_pdf_validation_overrides_and_doctor_remain_explicit() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    site(root);
    let base = fs::read_to_string(root.join("zel.yaml")).unwrap();
    fs::write(
        root.join("zel.yaml"),
        format!("{base}pdf: {{enabled: true, paper: a4, font-size: 12}}\n"),
    )
    .unwrap();
    let build = command(root).args(["build", "--no-pdf"]).output().unwrap();
    assert!(build.status.success(), "{build:?}");
    let before = fs::read(root.join("site/index.html")).unwrap();
    for args in [vec!["build"], vec!["--json", "doctor"]] {
        let result = command(root).args(args).output().unwrap();
        assert_eq!(result.status.code(), Some(4), "{result:?}");
    }
    for args in [vec!["pdf", "--check"], vec!["check", "--strict"]] {
        let result = command(root).args(args).output().unwrap();
        assert!(result.status.success(), "{result:?}");
    }
    assert_eq!(before, fs::read(root.join("site/index.html")).unwrap());
    for value in [
        "{enabled: yes}",
        "{paper: legal}",
        "{font-size: 9}",
        "{font-size: 13}",
        "{font-size: 11.5}",
        "{font-size: '12'}",
        "{paper: 'a4,escape'}",
        "{enabld: true}",
        "null",
    ] {
        fs::write(root.join("zel.yaml"), format!("{base}pdf: {value}\n")).unwrap();
        let result = command(root).args(["build", "--no-pdf"]).output().unwrap();
        assert!(!result.status.success(), "{value}: {result:?}");
        assert_eq!(before, fs::read(root.join("site/index.html")).unwrap());
    }
    fs::write(
        root.join("zel.yaml"),
        format!("{base}pdf: {{enabled: false}}\n"),
    )
    .unwrap();
    assert!(
        command(root)
            .arg("build")
            .output()
            .unwrap()
            .status
            .success()
    );
    assert_eq!(
        command(root)
            .args(["build", "--pdf"])
            .output()
            .unwrap()
            .status
            .code(),
        Some(4)
    );
    assert_eq!(
        command(root)
            .args(["build", "--pdf", "--no-pdf"])
            .output()
            .unwrap()
            .status
            .code(),
        Some(2)
    );
    fs::write(
        root.join("zel.yaml"),
        "version: 1\nrenderer: portable\npdf: {enabled: true}\n",
    )
    .unwrap();
    assert!(
        !command(root)
            .arg("build")
            .output()
            .unwrap()
            .status
            .success()
    );
    fs::write(
        root.join("zel.yaml"),
        "version: 1\nrenderer: portable\nbook: null\npdf: {enabled: true}\n",
    )
    .unwrap();
    let result = command(root).arg("doctor").output().unwrap();
    assert_eq!(result.status.code(), Some(2), "{result:?}");
    assert!(String::from_utf8_lossy(&result.stderr).contains("book manifest"));
}

#[cfg(unix)]
#[test]
fn configured_print_settings_reach_artifacts_and_next_delivery_fails_before_runtime() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    site(root);
    runtime(root);
    let base = fs::read_to_string(root.join("zel.yaml")).unwrap();
    fs::write(
        root.join("zel.yaml"),
        format!("{base}pdf: {{enabled: true, paper: letter, font-size: 10}}\n"),
    )
    .unwrap();
    let run = |args: &[&str]| {
        command(root)
            .env_remove("CONTAINER_HOST")
            .env_remove("DOCKER_CONTEXT")
            .env_remove("CONTAINER_CONNECTION")
            .env_remove("CONTAINERD_ADDRESS")
            .env_remove("CONTAINERD_NAMESPACE")
            .args(args)
            .output()
            .unwrap()
    };
    let result = run(&["build"]);
    assert!(result.status.success(), "{result:?}");
    let tex = fs::read_to_string(root.join("site/_zellij/pdf/book.tex")).unwrap();
    assert!(tex.contains("10pt,letterpaper"));
    assert!(!tex.contains("@PAPER@"));
    let receipt: serde_json::Value =
        serde_json::from_slice(&fs::read(root.join("site/_zellij/pdf/pdf.json")).unwrap()).unwrap();
    assert_eq!(receipt["settings"]["paper"], "letter");
    assert_eq!(receipt["settings"]["font-size"], 10);
    fs::remove_file(root.join("arguments")).unwrap();
    let result = run(&["ni"]);
    assert_eq!(result.status.code(), Some(2), "{result:?}");
    assert!(String::from_utf8_lossy(&result.stderr).contains("static image"));
    assert!(!root.join("arguments").exists());
    let result = run(&["build", "--no-pdf"]);
    assert!(result.status.success(), "{result:?}");
    assert!(!root.join("site/_zellij/pdf").exists());
    assert!(!root.join("arguments").exists());
}

#[cfg(unix)]
#[test]
fn preview_rebuilds_pdf_and_web_together_and_recovers_without_stale_publication() {
    use std::time::{Duration, Instant};
    struct Child(std::process::Child);
    impl Drop for Child {
        fn drop(&mut self) {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }
    fn wait(mut condition: impl FnMut() -> bool) {
        let end = Instant::now() + Duration::from_secs(20);
        while !condition() {
            assert!(Instant::now() < end, "preview condition timed out");
            std::thread::sleep(Duration::from_millis(50));
        }
    }
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    site(root);
    runtime(root);
    let base = fs::read_to_string(root.join("zel.yaml")).unwrap();
    let enabled = format!("{base}pdf: {{enabled: true}}\n");
    fs::write(root.join("zel.yaml"), &enabled).unwrap();
    let log = fs::File::create(root.join("preview.log")).unwrap();
    let mut child = Child(
        command(root)
            .env_remove("CONTAINER_HOST")
            .env_remove("DOCKER_CONTEXT")
            .env_remove("CONTAINER_CONNECTION")
            .env_remove("CONTAINERD_ADDRESS")
            .env_remove("CONTAINERD_NAMESPACE")
            .args(["view", "--watch", "--no-open", "--port", "0"])
            .stdout(log.try_clone().unwrap())
            .stderr(log)
            .spawn()
            .unwrap(),
    );
    let mut origin = String::new();
    wait(|| {
        let log = fs::read_to_string(root.join("preview.log")).unwrap();
        if let Some(url) = log
            .lines()
            .find_map(|line| line.strip_prefix("Serving at "))
        {
            origin = url.trim_end_matches('/').into();
            true
        } else {
            false
        }
    });
    let client = reqwest::blocking::Client::builder()
        .no_proxy()
        .timeout(Duration::from_secs(2))
        .build()
        .unwrap();
    let get = |path: &str| client.get(format!("{origin}/{path}")).send().unwrap();
    let json = |path: &str| get(path).json::<serde_json::Value>().unwrap();
    assert!(
        get("_zellij/pdf/book.pdf")
            .bytes()
            .unwrap()
            .starts_with(b"%PDF-")
    );
    let first = json("_zellij/book.json")["revision"].clone();
    let initial_web = get("one.html").text().unwrap();
    fs::write(
        root.join("content/one.md"),
        "# One\n\n[Broken](missing.md)\n",
    )
    .unwrap();
    wait(|| !json("_zellij/preview.json")["error"].is_null());
    assert_eq!(json("_zellij/book.json")["revision"], first);
    assert_eq!(json("_zellij/pdf/pdf.json")["revision"], first);
    // Ignore the changing preview-error banner; the successful chapter stays served.
    assert!(get("one.html").text().unwrap().contains("A short chapter."));
    assert!(initial_web.contains("Download"));
    fs::write(root.join("hold"), "hold").unwrap();
    fs::write(
        root.join("content/one.md"),
        "# One\n\nIntermediate manuscript.\n",
    )
    .unwrap();
    wait(|| root.join("holding").exists());
    fs::write(root.join("content/one.md"), "# One\n\nLatest manuscript.\n").unwrap();
    fs::remove_file(root.join("hold")).unwrap();
    wait(|| {
        fs::read_to_string(root.join("preview.log"))
            .unwrap()
            .contains("content changed during the build")
    });
    wait(|| {
        json("_zellij/preview.json")["error"].is_null()
            && get("one.html")
                .text()
                .unwrap()
                .contains("Latest manuscript.")
    });
    let latest = json("_zellij/book.json")["revision"].clone();
    assert_ne!(latest, first);
    assert_eq!(latest, json("_zellij/pdf/pdf.json")["revision"]);
    fs::write(
        root.join("zel.yaml"),
        format!("{base}pdf: {{enabled: false}}\n"),
    )
    .unwrap();
    wait(|| get("_zellij/pdf/book.pdf").status().as_u16() == 404);
    assert!(
        !get("one.html")
            .text()
            .unwrap()
            .contains("_zellij/pdf/book.pdf")
    );
    fs::write(root.join("zel.yaml"), &enabled).unwrap();
    wait(|| get("_zellij/pdf/book.pdf").status().is_success());
    assert_eq!(
        json("_zellij/book.json")["revision"],
        json("_zellij/pdf/pdf.json")["revision"]
    );
    // Cancellation while typesetting removes the container and retains the site.
    let before = fs::read(root.join("site/one.html")).unwrap();
    fs::remove_file(root.join("holding")).unwrap();
    fs::write(root.join("hold"), "hold").unwrap();
    fs::write(
        root.join("content/one.md"),
        "# One\n\nCancelled manuscript.\n",
    )
    .unwrap();
    wait(|| root.join("holding").exists());
    assert!(
        Command::new("/bin/kill")
            .args(["-INT", &child.0.id().to_string()])
            .status()
            .unwrap()
            .success()
    );
    wait(|| child.0.try_wait().unwrap().is_some());
    assert_eq!(before, fs::read(root.join("site/one.html")).unwrap());
    assert!(
        fs::read_to_string(root.join("cleanup"))
            .unwrap()
            .lines()
            .count()
            >= 5
    );
}
