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
        .env("ZEL_PDF_BACKEND", "container")
        .env("DOCKER_HOST", "unix:///fixture.sock")
        .env("CONTAINER_HOST", "unix:///fixture-other.sock");
    c
}
#[test]
fn default_cached_pdf_preflight_is_read_only_and_offline_never_falls_back_to_container() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    site(root);
    let base = fs::read_to_string(root.join("zel.yaml")).unwrap();
    fs::write(
        root.join("zel.yaml"),
        format!("{base}pdf: {{enabled: true}}\n"),
    )
    .unwrap();
    let home = root.join("zel home");
    let run = |args: &[&str]| {
        command(root)
            .env("ZELLIJ_HOME", &home)
            .env_remove("ZEL_ENGINE_CACHE")
            .env_remove("ZEL_PDF_BACKEND")
            .args(args)
            .output()
            .unwrap()
    };
    let diagnostic = run(&["doctor", "--json"]);
    assert!(diagnostic.status.success(), "{diagnostic:?}");
    let report: serde_json::Value = serde_json::from_slice(&diagnostic.stdout).unwrap();
    assert_eq!(report["runtime"]["status"], "not_required");
    assert_eq!(report["pdf_engine"]["status"], "download_required");
    assert_eq!(report["rich_engine"]["status"], "download_required");
    assert!(report["image"].is_null());
    assert!(run(&["pdf", "--check", "--offline"]).status.success());
    assert!(run(&["build", "--no-pdf", "--offline"]).status.success());
    let before = fs::read(root.join("site/index.html")).unwrap();
    for args in [
        vec!["pdf", "--offline"],
        vec!["build", "--offline"],
        vec!["bp", "--offline"],
    ] {
        let result = run(&args);
        assert!(!result.status.success(), "{result:?}");
        assert!(String::from_utf8_lossy(&result.stderr).contains("compiler is not cached"));
    }
    let result = run(&["pdf", "--offline", "--pdf-backend", "container"]);
    assert!(!result.status.success());
    assert!(
        String::from_utf8_lossy(&result.stderr)
            .contains("--offline PDFs require --pdf-backend cached")
    );
    assert_eq!(before, fs::read(root.join("site/index.html")).unwrap());
    assert!(!root.join("pdf").exists());
    assert!(!home.exists());
    let cache = home.join("cache/pdf").join(env!("CARGO_PKG_VERSION"));
    fs::create_dir_all(&cache).unwrap();
    fs::write(cache.join("manifest.json"), "broken").unwrap();
    let diagnosis = run(&["doctor"]);
    assert!(!diagnosis.status.success());
    let text = String::from_utf8_lossy(&diagnosis.stdout);
    assert!(text.contains("not required for cached PDF builds"));
    assert!(!text.contains("portable web-only"));
    assert_eq!(
        fs::read_to_string(cache.join("manifest.json")).unwrap(),
        "broken"
    );
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
#[test]
fn pdf_inputs_share_the_output_filesystem_and_are_removed_after_execution() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    site(root);
    runtime(root);
    let shared = root.join("shared");
    let unshared = root.join("unshared-system-temp");
    fs::create_dir(&shared).unwrap();
    fs::create_dir(&unshared).unwrap();
    for exit in ["0", "1"] {
        let result = command(root)
            .env_remove("CONTAINER_HOST")
            .env("TMPDIR", &unshared)
            .env("PDF_EXIT", exit)
            .args(["pdf", "--out", "shared/pdf"])
            .output()
            .unwrap();
        assert_eq!(result.status.success(), exit == "0", "{result:?}");
        let argv = fs::read_to_string(root.join("arguments")).unwrap();
        let input = Path::new(
            argv.lines()
                .find_map(|line| line.strip_suffix(":/input:ro"))
                .expect("read-only generated inputs"),
        );
        assert!(
            input.starts_with(shared.canonicalize().unwrap()),
            "PDF inputs must use the output's shared filesystem: {}",
            input.display()
        );
        assert!(!input.exists(), "generated inputs retained after execution");
        assert_eq!(fs::read_dir(&shared).unwrap().count(), 1);
        assert!(shared.join("pdf/book.pdf").is_file());
    }
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
 build)
  /bin/cat > recipe
  for arg in "$@"; do context=$arg; done
  /bin/cp -R "$context" captured
  echo built >> builds; exit 0;;
 push) echo pushed >> pushes; exit 0;;
 run)
  printf '%s\n' "$@" > arguments
  if [ -f hold ]; then echo holding > holding; while [ -f hold ]; do /bin/sleep 0.05; done; fi
  for arg in "$@"; do
   case "$arg" in *:/out) out=${arg%:/out};; esac
  done
  [ -n "$out" ] || exit 90
  printf '%%PDF-1.7\nfixture\n%%%%EOF\n' > "$out/book.pdf"
  printf '%s' "${PDF_LOG:-clean typesetting}" > "$out/book.log"
  if [ -n "${PDF_EDIT:-}" ]; then printf '\nEdited while typesetting.\n' >> "$PDF_EDIT"; fi
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
fn configured_print_settings_reach_artifacts_and_no_pdf_remains_native() {
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

#[cfg(unix)]
#[test]
fn next_pdf_builds_and_publishes_once_and_stops_on_failure_or_changed_input() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    site(root);
    runtime(root);
    let base = fs::read_to_string(root.join("zel.yaml")).unwrap();
    let config = format!(
        "{base}pdf: {{enabled: true}}\nimage: {{name: ghcr.io/mylife-inc/books:next}}\npublish: {{action: push, image: next}}\n"
    );
    fs::write(root.join("zel.yaml"), &config).unwrap();
    fs::create_dir(root.join("site")).unwrap();
    fs::write(root.join("site/index.html"), "previous export").unwrap();
    let run = |args: &[&str], failure: bool, edit: bool| {
        command(root)
            .env_remove("CONTAINER_HOST")
            .env_remove("DOCKER_CONTEXT")
            .env_remove("CONTAINER_CONNECTION")
            .env_remove("CONTAINERD_ADDRESS")
            .env_remove("CONTAINERD_NAMESPACE")
            .env("PDF_EXIT", if failure { "1" } else { "0" })
            .env(
                "PDF_EDIT",
                if edit {
                    root.join("content/one.md")
                } else {
                    std::path::PathBuf::new()
                },
            )
            .args(args)
            .output()
            .unwrap()
    };
    for args in [
        vec!["pub", "--dry-run"],
        vec![
            "deploy",
            "fly",
            "--prod",
            "--kind",
            "next",
            "--site-id",
            "book-test",
            "--dry-run",
        ],
        vec![
            "deploy",
            "railway",
            "--prod",
            "--kind",
            "next",
            "--site-id",
            "00000000-0000-4000-8000-000000000001",
            "--dry-run",
        ],
    ] {
        let output = run(&args, false, false);
        assert!(output.status.success(), "{output:?}");
        assert!(String::from_utf8_lossy(&output.stdout).contains("PDF"));
        assert!(!root.join("arguments").exists());
    }
    for args in [vec!["ni"], vec!["pub"]] {
        let output = run(&args, false, false);
        assert!(output.status.success(), "{output:?}");
        let captured = root.join("captured");
        assert!(captured.join("assets/pdf/book.pdf").is_file());
        assert_eq!(
            fs::read_dir(captured.join("assets/pdf")).unwrap().count(),
            2
        );
        assert!(!captured.join(".zel-pdf-work").exists());
        assert!(!captured.join(".zellij-output").exists());
        let receipt: serde_json::Value =
            serde_json::from_slice(&fs::read(captured.join("assets/pdf/pdf.json")).unwrap())
                .unwrap();
        let book: serde_json::Value =
            serde_json::from_slice(&fs::read(captured.join("assets/book.json")).unwrap()).unwrap();
        assert_eq!(receipt["revision"], book["publication"]["revision"]);
        fs::remove_dir_all(captured).unwrap();
    }
    assert_eq!(
        fs::read_to_string(root.join("builds"))
            .unwrap()
            .lines()
            .count(),
        2
    );
    assert_eq!(
        fs::read_to_string(root.join("pushes"))
            .unwrap()
            .lines()
            .count(),
        1
    );
    for (failure, edit) in [(true, false), (false, true)] {
        let output = run(&["pub"], failure, edit);
        assert_eq!(output.status.code(), Some(3), "{output:?}");
        assert!(!root.join("captured").exists());
        assert_eq!(
            fs::read_to_string(root.join("builds"))
                .unwrap()
                .lines()
                .count(),
            2
        );
        assert_eq!(
            fs::read_to_string(root.join("pushes"))
                .unwrap()
                .lines()
                .count(),
            1
        );
    }
    assert_eq!(
        fs::read_to_string(root.join("site/index.html")).unwrap(),
        "previous export"
    );
    assert_eq!(fs::read_to_string(root.join("zel.yaml")).unwrap(), config);
}

#[test]
fn invalid_print_styles_fail_natively_and_preserve_existing_output() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    site(root);
    let base = fs::read_to_string(root.join("zel.yaml")).unwrap();
    assert!(
        command(root)
            .arg("build")
            .output()
            .unwrap()
            .status
            .success()
    );
    let before = fs::read(root.join("site/index.html")).unwrap();
    for style in [
        "null",
        "[]",
        "{accent: null}",
        "{accent: '#fff'}",
        "{accent: '#GG1234'}",
        "{accent: '123456'}",
        "{accent: '#123456}\\\\input{secret}'}",
        "{body-font: Arial}",
        "{body-font: null}",
        "{title-font: 12}",
        "{chapter-style: novel}",
        "{line-spacing: 1.15}",
        "{margins: null}",
        "{margins: []}",
        "{margins: {left: 9}}",
        "{margins: {right: 41}}",
        "{margins: {top: 12.5}}",
        "{margins: {bottom: '20'}}",
        "{margins: {left: 0, right: 255}}",
        "{margins: {inside: 20}}",
        "{font: serif}",
    ] {
        fs::write(
            root.join("zel.yaml"),
            format!("{base}pdf: {{enabled: false, style: {style}}}\n"),
        )
        .unwrap();
        for args in [["build", "--no-pdf"], ["pdf", "--check"]] {
            let result = command(root).args(args).output().unwrap();
            assert!(!result.status.success(), "{style}: {result:?}");
            assert!(!String::from_utf8_lossy(&result.stderr).contains("PDF requires a running"));
            assert_eq!(before, fs::read(root.join("site/index.html")).unwrap());
            assert!(!root.join("pdf").exists());
        }
    }
    for style in [
        "{}",
        "{accent: '#493b72', body-font: sans, title-font: serif, chapter-style: study, line-spacing: relaxed, margins: {left: 15, right: 40, top: 15, bottom: 40}}",
        "{body-font: serif, title-font: sans, chapter-style: technical, line-spacing: spacious}",
        "{line-spacing: normal}",
    ] {
        fs::write(
            root.join("zel.yaml"),
            format!("{base}pdf: {{style: {style}}}\n"),
        )
        .unwrap();
        let result = command(root).args(["pdf", "--check"]).output().unwrap();
        assert!(result.status.success(), "{style}: {result:?}");
    }
}

#[cfg(unix)]
#[test]
fn print_style_receipt_survives_static_and_next_delivery() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    site(root);
    runtime(root);
    let base = fs::read_to_string(root.join("zel.yaml")).unwrap();
    fs::write(root.join("zel.yaml"),format!("{base}pdf: {{enabled: true, style: {{accent: '#493B72', title-font: sans, chapter-style: study, line-spacing: spacious, margins: {{left: 24}}}}}}\n")).unwrap();
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
    let record: serde_json::Value =
        serde_json::from_slice(&fs::read(root.join("site/_zellij/pdf/pdf.json")).unwrap()).unwrap();
    assert_eq!(record["settings"]["style"]["title-font"], "sans");
    assert_eq!(record["settings"]["style"]["margins"]["left"], 24);
    assert_eq!(record["settings"]["style"]["margins"]["right"], 18);
    let result = run(&["ni", "--tag", "book:next"]);
    assert!(result.status.success(), "{result:?}");
    let next: serde_json::Value =
        serde_json::from_slice(&fs::read(root.join("captured/assets/pdf/pdf.json")).unwrap())
            .unwrap();
    assert_eq!(record["settings"], next["settings"]);
}

#[test]
fn unsupported_print_locales_and_font_combinations_preserve_existing_site() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    site(root);
    let original = fs::read_to_string(root.join("zel.yaml")).unwrap();
    assert!(
        command(root)
            .arg("build")
            .output()
            .unwrap()
            .status
            .success()
    );
    let before = fs::read(root.join("site/index.html")).unwrap();
    for (language, direction, font) in [
        ("ja", "ltr", "serif"),
        ("ar", "ltr", "serif"),
        ("ar", "rtl", "libertinus"),
        ("fr", "rtl", "serif"),
    ] {
        let mut yaml: serde_yaml_ng::Value = serde_yaml_ng::from_str(&original).unwrap();
        yaml["book"]["language"] = language.into();
        yaml["book"]["direction"] = direction.into();
        yaml["pdf"] = serde_yaml_ng::from_str::<serde_yaml_ng::Value>(&format!(
            "{{enabled: true, style: {{body-font: {font}}}}}"
        ))
        .unwrap();
        fs::write(
            root.join("zel.yaml"),
            serde_yaml_ng::to_string(&yaml).unwrap(),
        )
        .unwrap();
        for args in [["build", "--pdf"], ["pdf", "--check"]] {
            let result = command(root).args(args).output().unwrap();
            assert!(!result.status.success(), "{result:?}");
            assert!(!String::from_utf8_lossy(&result.stderr).contains("PDF requires a running"));
            assert_eq!(before, fs::read(root.join("site/index.html")).unwrap());
        }
    }
}
