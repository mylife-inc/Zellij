use std::{
    fs,
    path::Path,
    process::{Command, Output},
};

fn write(root: &Path, path: &str, body: &str) {
    let file = root.join(path);
    fs::create_dir_all(file.parent().unwrap()).unwrap();
    fs::write(file, body).unwrap();
}

fn convert(source: &Path, out: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_zel"))
        .arg("convert")
        .arg(source)
        .arg("--out")
        .arg(out)
        .output()
        .unwrap()
}

fn success(output: Output) -> String {
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap()
}

fn docusaurus(source: &Path, sidebar: &str) {
    write(
        source,
        "docusaurus.config.js",
        "module.exports = {title: 'Metadata'};",
    );
    write(source, "sidebars.json", sidebar);
}

fn metadata(out: &Path, file: &str) -> serde_yaml_ng::Value {
    let raw = fs::read_to_string(out.join(file)).unwrap();
    serde_yaml_ng::from_str(
        raw.strip_prefix("---\n")
            .unwrap()
            .split_once("\n---\n")
            .unwrap()
            .0,
    )
    .unwrap()
}

#[test]
fn both_importers_preserve_authored_metadata_and_distinct_sidebar_labels() {
    for is_docusaurus in [false, true] {
        let temp = tempfile::tempdir().unwrap();
        let source = temp.path().join("source");
        let out = temp.path().join("out");
        if is_docusaurus {
            docusaurus(
                &source,
                r#"{"docs":[{"type":"doc","id":"guide/custom","label":"Sidebar label"}]}"#,
            );
        } else {
            write(
                &source,
                "mkdocs.yml",
                "site_name: Metadata\nnav:\n  - Sidebar label: 01-guide/source.md\n",
            );
        }
        let raw = "\u{feff}---\r\nid: custom\r\ntitle: 'Page: title'\r\ndescription: |\r\n  First line\r\n  Second line\r\nslug: /old-route\r\nunknown: !plugin value\r\n...\r\n# Body heading {#body}\r\n\r\nA paragraph.\r\n";
        write(&source, "docs/01-guide/source.md", raw);
        success(convert(&source, &out));
        let page = metadata(&out, "01-guide/01-guide/source.mdx");
        assert_eq!(page["title"].as_str(), Some("Page: title"));
        assert_eq!(
            page["description"].as_str(),
            Some("First line\nSecond line\n")
        );
        assert!(page.get("slug").is_none());
        let body = fs::read_to_string(out.join("01-guide/01-guide/source.mdx")).unwrap();
        assert!(body.contains("#   unknown: !plugin value"));
        assert!(body.contains("<div id=\"body\" />"));
        let sidebar: serde_yaml_ng::Value = serde_yaml_ng::from_str(
            &fs::read_to_string(out.join("01-guide/_sidebar.yaml")).unwrap(),
        )
        .unwrap();
        assert_eq!(sidebar["nav"][0]["page"].as_str(), Some("01-guide/source"));
        assert_eq!(sidebar["nav"][0]["label"].as_str(), Some("Sidebar label"));
        assert_eq!(
            fs::read_to_string(source.join("docs/01-guide/source.md")).unwrap(),
            raw
        );
    }
}

#[test]
fn custom_ids_prefix_rules_and_mdx_links_resolve_to_physical_files() {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("source");
    let out = temp.path().join("out");
    docusaurus(
        &source,
        r#"{"docs":["intro",{"type":"category","label":"Guides","items":[{"type":"doc","id":"Guides/custom","label":"Overridden"},"Guides/Install","02-Exact/03-Keep","2026-11-04"]}]}"#,
    );
    write(
        &source,
        "docs/01-intro.md",
        "# Welcome\n\n[Page](02-Guides/04-APIReference.md)\n",
    );
    write(
        &source,
        "docs/02-Guides/04-APIReference.mdx",
        "---\nid: custom\nsidebar_label: Short label\n---\n# Reference title\n\n[Home](../01-intro.md)\n\n![Logo](logo.svg)\n",
    );
    write(&source, "docs/02-Guides/05-Install.md", "# Install\n");
    write(&source, "docs/02-Guides/logo.svg", "<svg/>\n");
    write(
        &source,
        "docs/02-Exact/03-Keep.md",
        "---\nparse_number_prefixes: false\n---\n# Keep\n",
    );
    write(&source, "docs/2026-11-04.md", "# Date\n");
    success(convert(&source, &out));
    let body = fs::read_to_string(out.join("01-guide/01-intro.mdx")).unwrap();
    assert!(
        body.contains("[Page](</guide/guides/api-reference>)"),
        "{body}"
    );
    let body = fs::read_to_string(out.join("01-guide/02-Guides/04-APIReference.mdx")).unwrap();
    assert!(body.contains("[Home](</guide/intro>)"), "{body}");
    assert!(body.contains("/_zellij/assets/imported/docs/02-Guides/logo.svg"));
    assert_eq!(
        fs::read(out.join("assets/imported/docs/02-Guides/logo.svg")).unwrap(),
        b"<svg/>\n"
    );
    assert_eq!(
        metadata(&out, "01-guide/02-Guides/04-APIReference.mdx")["title"],
        "Reference title"
    );
    let sidebar = fs::read_to_string(out.join("01-guide/_sidebar.yaml")).unwrap();
    assert!(sidebar.contains("label: \"Short label\""));
    assert!(!sidebar.contains("Overridden"));
    assert!(out.join("01-guide/02-Exact/03-Keep.mdx").is_file());
    assert!(out.join("01-guide/2026-11-04.mdx").is_file());
}

#[test]
fn filename_sidebar_ids_remain_accepted_when_unambiguous() {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("source");
    let out = temp.path().join("out");
    docusaurus(&source, r#"{"docs":["01-Start"]}"#);
    write(
        &source,
        "docs/01-Start.mdx",
        "---\nid: beginning\n---\n# Start\n",
    );
    success(convert(&source, &out));
    assert!(out.join("01-guide/01-Start.mdx").is_file());
}

#[test]
fn drafts_and_unlisted_documents_are_excluded_with_and_without_a_sidebar() {
    for has_sidebar in [false, true] {
        let temp = tempfile::tempdir().unwrap();
        let source = temp.path().join("source");
        let out = temp.path().join("out");
        docusaurus(&source, r#"{"docs":["home","draft","hidden"]}"#);
        if !has_sidebar {
            fs::remove_file(source.join("sidebars.json")).unwrap();
        }
        write(&source, "docs/home.md", "# Home\n\n[Draft](draft.md)\n");
        write(&source, "docs/draft.md", "---\ndraft: true\n---\n# Draft\n");
        write(
            &source,
            "docs/hidden.md",
            "---\nunlisted: true\n---\n# Hidden\n",
        );
        write(&source, "docs/orphan.md", "# Orphan\n");
        let report = success(convert(&source, &out));
        assert!(!out.join("01-guide/draft.mdx").exists());
        assert!(!out.join("01-guide/hidden.mdx").exists());
        assert!(!out.join("assets/imported/docs/draft.md").exists());
        assert!(report.contains("marked draft or unlisted"), "{report}");
        assert_eq!(out.join("01-guide/orphan.mdx").exists(), !has_sidebar);
    }
}

#[test]
fn ambiguous_and_missing_sidebar_ids_fail_before_writing() {
    for (sidebar, files, expected) in [
        (
            r#"{"docs":["same"]}"#,
            vec![
                ("a.md", "---\nid: same\n---\n# A"),
                ("b.md", "---\nid: same\n---\n# B"),
            ],
            "duplicate Docusaurus document ID",
        ),
        (
            r#"{"docs":["a"]}"#,
            vec![
                ("a.md", "---\nid: other\n---\n# A"),
                ("b.md", "---\nid: a\n---\n# B"),
            ],
            "ambiguous Docusaurus sidebar ID",
        ),
        (
            r#"{"docs":["a"]}"#,
            vec![
                ("a.md", "---\nid: first\n---\n# A"),
                ("a.mdx", "---\nid: second\n---\n# B"),
            ],
            "ambiguous Markdown/MDX source files",
        ),
        (
            r#"{"docs":["missing"]}"#,
            vec![("a.md", "# A")],
            "does not match a source document",
        ),
    ] {
        let temp = tempfile::tempdir().unwrap();
        let source = temp.path().join("source");
        let out = temp.path().join("out");
        docusaurus(&source, sidebar);
        for (file, raw) in &files {
            write(&source, &format!("docs/{file}"), raw);
        }
        let result = convert(&source, &out);
        assert!(!result.status.success());
        let error = String::from_utf8_lossy(&result.stderr);
        assert!(error.contains(expected), "{error}");
        assert!(!out.exists());
        for (file, raw) in &files {
            assert_eq!(
                fs::read_to_string(source.join("docs").join(file)).unwrap(),
                *raw
            );
        }
    }
}

#[test]
fn invalid_metadata_fails_with_source_context_without_output() {
    for is_docusaurus in [false, true] {
        for raw in [
            "---\ntitle: [bad]\n---\n# A",
            "---\ndescription: false\n---\n# A",
            "---\ntitle: ''\n---\n# A",
            "---\nx: [\n---\n# A",
            "---\ntitle: unfinished",
        ] {
            let temp = tempfile::tempdir().unwrap();
            let source = temp.path().join("source");
            let out = temp.path().join("out");
            if is_docusaurus {
                docusaurus(&source, r#"{"docs":["page"]}"#);
            } else {
                write(
                    &source,
                    "mkdocs.yml",
                    "site_name: Metadata\nnav:\n  - page.md\n",
                );
            }
            write(&source, "docs/page.md", raw);
            let result = convert(&source, &out);
            assert!(!result.status.success());
            let error = String::from_utf8_lossy(&result.stderr);
            assert!(error.contains("page.md: invalid front matter"), "{error}");
            assert!(!out.exists());
            assert_eq!(
                fs::read_to_string(source.join("docs/page.md")).unwrap(),
                raw
            );
        }
    }
    for field in [
        "id: ../escape",
        "id: a/b",
        "sidebar_label: []",
        "draft: 'false'",
        "parse_number_prefixes: nope",
        "draft: true\nunlisted: 'true'",
    ] {
        let temp = tempfile::tempdir().unwrap();
        let source = temp.path().join("source");
        let out = temp.path().join("out");
        docusaurus(&source, r#"{"docs":["page"]}"#);
        write(
            &source,
            "docs/page.md",
            &format!("---\n{field}\n---\n# Page"),
        );
        let result = convert(&source, &out);
        assert!(!result.status.success(), "{field}");
        assert!(String::from_utf8_lossy(&result.stderr).contains("page.md"));
        assert!(!out.exists());
    }
}

#[test]
fn inventory_refuses_linked_directories_before_following_them() {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("source");
    let out = temp.path().join("out");
    docusaurus(&source, r#"{"docs":["home"]}"#);
    write(&source, "docs/home.md", "# Home\n");
    write(temp.path(), "outside/private.md", "Do not read or import");
    // cmd's mklink requires native separators even though Rust accepts both.
    let link = source.join("docs").join("linked");
    let target = temp.path().join("outside");
    #[cfg(unix)]
    std::os::unix::fs::symlink(&target, &link).unwrap();
    #[cfg(windows)]
    {
        let created = Command::new("cmd")
            .arg("/C")
            .arg("mklink")
            .arg("/J")
            .arg(&link)
            .arg(&target)
            .output()
            .unwrap();
        assert!(
            created.status.success(),
            "{}\n{}",
            String::from_utf8_lossy(&created.stdout),
            String::from_utf8_lossy(&created.stderr)
        );
    }
    let result = convert(&source, &out);
    // Unlink before temporary-directory cleanup, even if an assertion fails.
    #[cfg(windows)]
    fs::remove_dir(&link).unwrap();
    assert!(!result.status.success());
    assert!(String::from_utf8_lossy(&result.stderr).contains("symbolic links and junctions"));
    assert!(!out.exists());
    assert_eq!(
        fs::read_to_string(target.join("private.md")).unwrap(),
        "Do not read or import"
    );
}

#[test]
fn empty_or_filtered_navigation_never_promotes_orphans() {
    for sidebar in [
        r#"{"docs":[]}"#,
        r#"{"docs":["draft"]}"#,
        r#"{"docs":[{"type":"category","label":"Drafts","items":["draft"]}]}"#,
        r#"{"docs":[{"type":"autogenerated","dirName":"."}]}"#,
    ] {
        let temp = tempfile::tempdir().unwrap();
        let source = temp.path().join("source");
        let out = temp.path().join("out");
        docusaurus(&source, sidebar);
        write(&source, "docs/draft.md", "---\ndraft: true\n---\n# Draft");
        write(&source, "docs/orphan.md", "# Orphan");
        let report = success(convert(&source, &out));
        assert!(!out.join("01-guide/draft.mdx").exists());
        assert!(!out.join("01-guide/orphan.mdx").exists());
        assert!(report.contains("absent from the navigation"));
        let sidebar: serde_yaml_ng::Value = serde_yaml_ng::from_str(
            &fs::read_to_string(out.join("01-guide/_sidebar.yaml")).unwrap(),
        )
        .unwrap();
        assert!(sidebar["nav"].is_sequence());
        for entry in sidebar["nav"].as_sequence().unwrap() {
            assert!(entry["items"].is_sequence());
        }
    }
}

#[test]
fn json_sidebar_strings_are_data_not_javascript() {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("source");
    let out = temp.path().join("out");
    docusaurus(
        &source,
        r#"{"docs":[{"type":"doc","id":"home","label":"Details... and `require()`"}]}"#,
    );
    write(&source, "docs/home.md", "# Home");
    write(&source, "docs/orphan.md", "# Orphan");
    success(convert(&source, &out));
    assert!(!out.join("01-guide/orphan.mdx").exists());
    let sidebar: serde_yaml_ng::Value =
        serde_yaml_ng::from_str(&fs::read_to_string(out.join("01-guide/_sidebar.yaml")).unwrap())
            .unwrap();
    assert_eq!(sidebar["nav"][0]["label"], "Details... and `require()`");
}
