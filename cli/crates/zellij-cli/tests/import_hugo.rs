//! Actual CLI acceptance without Hugo execution, containers or browsers.
use std::{
    fs,
    path::Path,
    process::{Command, Output},
};
fn write(root: &Path, path: &str, body: impl AsRef<[u8]>) {
    let path = root.join(path);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, body).unwrap();
}
fn read(root: &Path, path: &str) -> String {
    fs::read_to_string(root.join(path)).unwrap()
}
fn run(source: &Path, out: &Path, args: &[&str], code: i32) -> Output {
    let o = Command::new(env!("CARGO_BIN_EXE_zel"))
        .arg("cv")
        .arg(source)
        .arg("--out")
        .arg(out)
        .args(args)
        .output()
        .unwrap();
    assert_eq!(o.status.code(), Some(code), "{o:?}");
    o
}
fn report(o: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&o.stdout),
        String::from_utf8_lossy(&o.stderr)
    )
}
fn basic(root: &Path) {
    write(
        root,
        "hugo.toml",
        "title = 'Manual'\nbaseURL = 'https://old.example.test/manual/'\n",
    );
    write(root, "content/_index.md", "# Home\n");
}

#[test]
fn config_formats_and_frontmatter_preserve_sections_links_assets_and_source() {
    for (config, body) in [
        (
            "hugo.toml",
            "title='Manual'\nbaseURL='https://old.example.test/manual/'\ncontentDir='pages'\n",
        ),
        (
            "hugo.yaml",
            "title: Manual\nbaseURL: https://old.example.test/manual/\ncontentDir: pages\n",
        ),
        (
            "config.json",
            r#"{"title":"Manual","baseURL":"https://old.example.test/manual/","contentDir":"pages"}"#,
        ),
    ] {
        let t = tempfile::tempdir().unwrap();
        let source = t.path().join("source");
        let out = t.path().join("out");
        write(&source, config, body);
        write(
            &source,
            "pages/_index.md",
            "+++\nTitle='Authored home'\ndescription='Description'\ndate=2020-01-02T03:04:05Z\n+++\n# Home {#home}\n\n[File](guide/_index.md#guide)\n\n[Section](/manual/guide/)\n\n[Leaf](https://old.example.test/manual/guide/install/#install)\n\n[Slug](/manual/custom/topic/?q=1#topic)\n\n![Static](/manual/logo.svg)\n\n[Download](files/Q3%20Report.pdf#page=2)\n",
        );
        write(
            &source,
            "pages/guide/_index.md",
            "---\ntitle: Guide\nweight: 2\n---\n# Guide\n\n[Home](../_index.md#home)\n\n[Leaf](install/index.md)\n",
        );
        write(
            &source,
            "pages/guide/install/index.md",
            "{\"title\":\"Install\",\"weight\":1}\n# Install\n\n![Local](diagram.svg)\n\n[Section](../_index.md)\n\n[Website home](../../#home)\n",
        );
        write(
            &source,
            "pages/guide/install/fragment.md",
            "# A page resource\n",
        );
        write(
            &source,
            "pages/guide/install/diagram.svg",
            "<svg>local</svg>",
        );
        write(
            &source,
            "pages/guide/topic.markdown",
            "---\ntitle: Topic\nurl: /custom/topic/\n---\n# Topic\n",
        );
        write(&source, "pages/files/Q3 Report.pdf", [0, 255, 254, 10]);
        write(&source, "static/logo.svg", "<svg>static</svg>");
        let o = run(&source, &out, &["--layout", "site"], 0);
        let c = out.join("content");
        let home = read(&c, "01-guide/index.mdx");
        for expected in [
            "title: \"Authored home\"",
            "description: \"Description\"",
            "[File](</guide/guide#guide>)",
            "[Section](</guide/guide>)",
            "[Leaf](</guide/guide/install#install>)",
            "[Slug](</guide/guide/topic?q=1#topic>)",
            "![Static](</_zellij/assets/imported/static/logo.svg>)",
            "[Download](</_zellij/assets/imported/docs/files/Q3%20Report.pdf#page=2>)",
            "Original Hugo front matter",
        ] {
            assert!(home.contains(expected), "{expected}: {home}");
        }
        assert!(!c.join("01-guide/_index.mdx").exists());
        assert!(read(&c, "01-guide/guide/index.mdx").contains("[Home](</guide#home>)"));
        let leaf = read(&c, "01-guide/guide/install/index.mdx");
        assert!(
            leaf.contains("![Local](</_zellij/assets/imported/docs/guide/install/diagram.svg>)"),
            "{leaf}"
        );
        assert!(leaf.contains("[Website home](</guide#home>)"), "{leaf}");
        assert!(!c.join("01-guide/guide/install/fragment.mdx").exists());
        assert!(report(&o).contains("page resource"));
        let nav = read(&c, "01-guide/_sidebar.yaml");
        assert!(!nav.contains("_index"));
        assert!(nav.contains("Guide"));
        assert!(nav.find("guide/install/index").unwrap() < nav.find("guide/topic").unwrap());
        assert_eq!(
            fs::read(c.join("assets/imported/docs/files/Q3 Report.pdf")).unwrap(),
            [0, 255, 254, 10]
        );
        assert_eq!(read(&source, config), body);
    }
}

#[test]
fn draft_scheduling_headless_build_rules_and_resources_do_not_publish_hidden_content() {
    let t = tempfile::tempdir().unwrap();
    let s = t.path().join("source");
    let o = t.path().join("out");
    basic(&s);
    for (name, meta) in [
        ("draft", "draft: true"),
        ("future", "publishDate: 2999-01-01"),
        ("expired", "expiryDate: 2000-01-01"),
        ("headless", "headless: true"),
        ("unlisted", "build: {list: never}"),
        ("unrendered", "_build: {render: link}"),
    ] {
        write(
            &s,
            &format!("content/{name}/index.md"),
            format!("---\n{meta}\n---\n# Hidden\n"),
        );
        write(&s, &format!("content/{name}/secret.bin"), b"secret");
    }
    write(
        &s,
        "content/visible.md",
        "---\ndate: 2020-01-01\n---\n# Visible\n",
    );
    write(
        &s,
        "content/_index.md",
        "# Home\n\n[Hidden](draft/index.md)\n\n[Bytes](draft/secret.bin)\n\n[Website](/manual/draft/)\n",
    );
    let result = run(&s, &o, &[], 0);
    assert!(o.join("01-guide/visible.mdx").exists());
    assert!(!o.join("01-guide/draft/index.mdx").exists());
    assert!(!o.join("assets/imported/docs/draft/secret.bin").exists());
    assert!(report(&result).contains("excluded"));
    assert!(read(&o, "01-guide/index.mdx").contains("[Website](/manual/draft/)"));
}

#[test]
fn configured_publication_and_weight_date_title_order_are_explicit() {
    let t = tempfile::tempdir().unwrap();
    let s = t.path().join("source");
    let o = t.path().join("out");
    basic(&s);
    write(
        &s,
        "hugo.yaml",
        "buildDrafts: true\nbuildFuture: true\nbuildExpired: true\n",
    );
    fs::remove_file(s.join("hugo.toml")).unwrap();
    for (name, meta) in [
        ("weighted", "weight: 1"),
        ("negative", "weight: -1"),
        ("older", "date: 2020-01-01"),
        ("newer", "date: 2021-01-01"),
        ("draft", "draft: true"),
        ("future", "publishDate: 2999-01-01"),
        ("expired", "expiryDate: 2000-01-01"),
    ] {
        write(
            &s,
            &format!("content/{name}.md"),
            format!("---\n{meta}\n---\n# {name}\n"),
        );
    }
    run(&s, &o, &[], 0);
    let nav = read(&o, "01-guide/_sidebar.yaml");
    for (a, b) in [
        ("negative", "weighted"),
        ("weighted", "future"),
        ("newer", "older"),
    ] {
        assert!(nav.find(a).unwrap() < nav.find(b).unwrap(), "{nav}");
    }
    for name in ["draft", "future", "expired"] {
        assert!(o.join(format!("01-guide/{name}.mdx")).exists());
    }
}

#[test]
fn supported_metadata_delimiters_dates_and_aliases_are_retained() {
    let t = tempfile::tempdir().unwrap();
    let s = t.path().join("source");
    let o = t.path().join("out");
    basic(&s);
    write(
        &s,
        "content/bom.md",
        "\u{feff}---\r\ntitle: BOM\r\ndate: 2020-01-01\r\naliases: [/old/]\r\n---\r\n# BOM\r\n",
    );
    write(
        &s,
        "content/toml.md",
        "+++\ntitle = 'TOML'\ndate=2020-01-01T01:02:03\n+++\n# TOML\n",
    );
    write(
        &s,
        "content/json.md",
        "{\"title\":\"JSON\",\"date\":\"2020-01-01T03:00:00+02:00\"}\n# JSON\n",
    );
    let r = run(&s, &o, &[], 0);
    assert!(report(&r).contains("redirect aliases"));
    for (name, title) in [("bom", "BOM"), ("toml", "TOML"), ("json", "JSON")] {
        assert!(read(&o, &format!("01-guide/{name}.mdx")).contains(&format!("title: \"{title}\"")));
    }
}

#[test]
fn absolute_bundle_assets_and_static_conflicts_are_checked() {
    let t = tempfile::tempdir().unwrap();
    let s = t.path().join("source");
    let o = t.path().join("out");
    basic(&s);
    write(
        &s,
        "content/topic/index.md",
        "# Topic\n\n![A](/manual/topic/photo.svg)\n",
    );
    write(&s, "content/topic/photo.svg", "<svg>A</svg>");
    run(&s, &o, &[], 0);
    assert_eq!(
        read(&o, "assets/imported/docs/topic/photo.svg"),
        "<svg>A</svg>"
    );
    write(&s, "static/topic/photo.svg", "<svg>B</svg>");
    let fail = run(&s, &t.path().join("conflict"), &[], 2);
    assert!(report(&fail).contains("share a website URL"));
    assert!(!t.path().join("conflict").exists());
}

#[test]
fn implicit_sections_preserve_resource_filename_case() {
    let t = tempfile::tempdir().unwrap();
    let s = t.path().join("source");
    let o = t.path().join("out");
    basic(&s);
    write(
        &s,
        "content/Guide/topic.md",
        "# Topic\n\n![Photo](/manual/guide/Photo.svg)\n\n![Root](/manual/Logo.svg)\n",
    );
    write(&s, "content/Guide/Photo.svg", "<svg>Photo</svg>");
    write(&s, "content/Logo.svg", "<svg>Logo</svg>");
    run(&s, &o, &[], 0);
    let page = read(&o, "01-guide/Guide/topic.mdx");
    assert!(
        page.contains("/_zellij/assets/imported/docs/Guide/Photo.svg"),
        "{page}"
    );
    assert!(
        page.contains("/_zellij/assets/imported/docs/Logo.svg"),
        "{page}"
    );
}

#[test]
fn assets_skip_still_checks_traversal_and_does_not_write_assets() {
    let t = tempfile::tempdir().unwrap();
    let s = t.path().join("source");
    let o = t.path().join("out");
    basic(&s);
    write(&s, "content/_index.md", "# Home\n\n![A](a.svg)\n");
    write(&s, "content/a.svg", "<svg/>");
    run(&s, &o, &["--assets", "skip"], 0);
    assert!(!o.join("assets").exists());
    write(
        &s,
        "content/_index.md",
        "# Home\n\n[Escape](../../secret.txt)\n",
    );
    run(&s, &t.path().join("escape"), &["--assets", "skip"], 2);
    assert!(!t.path().join("escape").exists());
}

#[test]
fn unsupported_configuration_fails_before_output() {
    for config in [
        "languages: {en: {}}",
        "module: {mounts: []}",
        "menus: {main: []}",
        "cascade: {draft: true}",
        "permalinks: {docs: '/:title/'}",
        "frontmatter: {date: [':filename']}",
        "uglyURLs: true",
        "ignoreFiles: ['secret']",
        "timeZone: Europe/Oslo",
        "contentDir: ../outside",
        "staticDir: content",
        "publishDir: content/public",
        "outputs: {home: [JSON]}",
        "defaultContentLanguageInSubdir: true",
        "weight: !ruby value",
        "title: A\nTitle: B",
    ] {
        let t = tempfile::tempdir().unwrap();
        let s = t.path().join("source");
        let o = t.path().join("out");
        basic(&s);
        fs::remove_file(s.join("hugo.toml")).unwrap();
        write(&s, "hugo.yaml", config);
        run(&s, &o, &[], 2);
        assert!(!o.exists(), "{config}");
    }
}

#[test]
fn unsupported_page_metadata_and_invalid_data_are_rejected_atomically() {
    for body in [
        "---\ncascade: {draft: true}\n---\n# X",
        "---\nmenus: main\n---\n# X",
        "---\ndraft: 'false'\n---\n# X",
        "---\nweight: 1.5\n---\n# X",
        "---\ndate: yesterday\n---\n# X",
        "---\nbuild: {publishResources: false}\n---\n# X",
        "---\nurl: /../../outside\n---\n# X",
        "---\nurl: https://evil.test\n---\n# X",
        "+++\ntitle='missing end'",
        "{\"title\":\"one\",\"title\":\"two\"}\n# X",
        "---\ntitle: !custom x\n---\n# X",
    ] {
        let t = tempfile::tempdir().unwrap();
        let s = t.path().join("source");
        let o = t.path().join("out");
        basic(&s);
        write(&s, "content/topic.md", body);
        run(&s, &o, &[], 2);
        assert!(!o.exists(), "{body}");
    }
}

#[test]
fn split_configuration_conflicting_indexes_adapters_and_routes_are_refused() {
    for (path, body) in [
        ("config/_default/hugo.toml", "title='split'"),
        ("config.yaml", "title: Duplicate"),
        ("content/index.md", "# Conflicting home"),
        ("content/topic/index.html", "<h1>HTML bundle</h1>"),
        ("content/_content.gotmpl", "{{ code }}"),
        ("content/home.en.md", "# Localized"),
        (
            "content/topic/_index.md",
            "---\nslug: different\n---\n# Section",
        ),
    ] {
        let t = tempfile::tempdir().unwrap();
        let s = t.path().join("source");
        let o = t.path().join("out");
        basic(&s);
        write(&s, path, body);
        run(&s, &o, &[], 2);
        assert!(!o.exists(), "{path}");
    }
    let t = tempfile::tempdir().unwrap();
    let s = t.path().join("source");
    let o = t.path().join("out");
    basic(&s);
    write(&s, "content/topic.md", "# One");
    write(&s, "content/topic/index.md", "# Two");
    run(&s, &o, &[], 2);
    assert!(!o.exists());
}

#[test]
fn detection_dry_runs_and_ship_plans_remain_inert() {
    let t = tempfile::tempdir().unwrap();
    let s = t.path().join("source");
    let o = t.path().join("out");
    basic(&s);
    let detected = Command::new(env!("CARGO_BIN_EXE_zel"))
        .args(["detect", s.to_str().unwrap(), "--json"])
        .output()
        .unwrap();
    let v: serde_json::Value = serde_json::from_slice(&detected.stdout).unwrap();
    assert_eq!(v["matches"][0]["id"], "hugo");
    for command in ["convert", "cv", "migrate", "mig"] {
        let r = Command::new(env!("CARGO_BIN_EXE_zel"))
            .arg(command)
            .arg(&s)
            .arg("--out")
            .arg(&o)
            .arg("--dry-run")
            .output()
            .unwrap();
        assert!(r.status.success(), "{r:?}");
        assert!(!o.exists());
    }
    let r = Command::new(env!("CARGO_BIN_EXE_zel"))
        .args([
            "ship",
            s.to_str().unwrap(),
            "--out",
            o.to_str().unwrap(),
            "--provider",
            "netlify",
            "--site-id",
            "00000000-0000-0000-0000-000000000001",
            "--dry-run",
        ])
        .output()
        .unwrap();
    assert!(r.status.success(), "{r:?}");
    assert!(report(&r).to_lowercase().contains("hugo"), "{r:?}");
    assert!(!o.exists());
}

#[test]
fn leading_whitespace_and_inline_delimiters_cannot_bypass_draft_visibility() {
    for body in [
        " \n\t---\ndraft: true\n---\n# Hidden",
        "\u{feff}\n+++draft=true\n+++\n# Hidden",
        " \r\n{\"draft\":true}\n# Hidden",
        "---draft: true\n---\n# Hidden",
    ] {
        let t = tempfile::tempdir().unwrap();
        let s = t.path().join("source");
        let o = t.path().join("out");
        basic(&s);
        write(&s, "content/hidden.md", body);
        run(&s, &o, &[], 0);
        assert!(!o.join("01-guide/hidden.mdx").exists());
    }
    for body in ["#+DRAFT: true\n# Hidden", "---\ndraft: true\n...\n# Hidden"] {
        let t = tempfile::tempdir().unwrap();
        let s = t.path().join("source");
        let o = t.path().join("out");
        basic(&s);
        write(&s, "content/hidden.md", body);
        run(&s, &o, &[], 2);
        assert!(!o.exists());
    }
}

#[test]
fn shortcodes_stay_literal_and_are_reported_without_execution() {
    let t = tempfile::tempdir().unwrap();
    let s = t.path().join("source");
    let o = t.path().join("out");
    basic(&s);
    write(
        &s,
        "content/_index.md",
        "# Home\n\n{{< include \"secret.txt\" >}}\n\n```md\n{{< example >}}\n```\n",
    );
    let r = run(&s, &o, &[], 0);
    assert!(report(&r).contains("shortcodes"));
    assert!(read(&o, "01-guide/index.mdx").contains("secret.txt"));
}

#[cfg(unix)]
#[test]
fn source_and_asset_symlinks_are_refused() {
    use std::os::unix::fs::symlink;
    for path in ["content/linked.md", "content/photo.svg", "static/logo.svg"] {
        let t = tempfile::tempdir().unwrap();
        let s = t.path().join("source");
        let o = t.path().join("out");
        basic(&s);
        write(t.path(), "outside", "secret");
        fs::create_dir_all(s.join(path).parent().unwrap()).unwrap();
        symlink(t.path().join("outside"), s.join(path)).unwrap();
        run(&s, &o, &[], 2);
        assert!(!o.exists());
    }
}
