//! Jekyll import acceptance without Ruby, source execution, browsers or containers.
use std::{
    fs,
    path::Path,
    process::{Command, Output},
};
fn write(root: &Path, path: &str, body: impl AsRef<[u8]>) {
    let p = root.join(path);
    fs::create_dir_all(p.parent().unwrap()).unwrap();
    fs::write(p, body).unwrap();
}
fn read(root: &Path, path: &str) -> String {
    fs::read_to_string(root.join(path)).unwrap()
}
fn run(source: &Path, out: &Path, args: &[&str], code: i32) -> Output {
    let output = Command::new(env!("CARGO_BIN_EXE_zel"))
        .arg("cv")
        .arg(source)
        .arg("--out")
        .arg(out)
        .args(args)
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(code), "{output:?}");
    output
}
fn report(o: &Output) -> String {
    format!(
        "{}\n{}",
        String::from_utf8_lossy(&o.stdout),
        String::from_utf8_lossy(&o.stderr)
    )
}
fn page(body: &str) -> String {
    format!("---\n---\n{body}")
}

#[test]
fn configuration_formats_preserve_metadata_navigation_urls_assets_and_source_bytes() {
    for (config, style) in [
        ("_config.yml", "date"),
        ("_config.yaml", "pretty"),
        ("_config.toml", "pretty"),
    ] {
        let t = tempfile::tempdir().unwrap();
        let source = t.path().join("source");
        let out = t.path().join("out");
        let config_body = format!(
            "title: 'A: manual'\ndescription: Site description\nsource: pages\nurl: https://old.example.test\nbaseurl: /manual\npermalink: {style}\ncollections:\n  reference:\n    output: true\n    permalink: /ref/:path/\n"
        );
        let config_body = if config.ends_with(".toml") {
            format!(
                "\u{feff}title = 'A: manual'\r\ndescription = 'Site description'\r\nsource = 'pages'\r\nurl = 'https://old.example.test'\r\nbaseurl = '/manual'\r\npermalink = '{style}'\r\n[collections.reference]\r\noutput = true\r\npermalink = '/ref/:path/'\r\n"
            )
        } else {
            config_body
        };
        write(&source, config, &config_body);
        write(
            &source,
            "pages/index.md",
            "\u{feff}---\r\ntitle: Authored title\r\ndescription: Authored description\r\nroute: /do-not-activate\r\n---\r\n# Welcome {#home}\n\n[File](Guide/Install.md?from=file#install)\n\n[Website](/manual/Custom/Install/?from=website#install)\n\n[Absolute](https://old.example.test/manual/Custom/Install/#install)\n\n[Collection](_reference/API.markdown#api)\n\n[Collection URL](/manual/ref/API/)\n\n[Unicode](/manual/Caf%C3%A9/)\n\n![Logo](media/logo.svg)\n\n[Download](/manual/media/Q3%20Report.pdf#page=2)\n\n[Outside](https://elsewhere.test/manual/Custom/Install/)\n\n`[Example](/manual/Custom/Install/)`\n",
        );
        write(
            &source,
            "pages/Guide/Install.md",
            "---\ntitle: Install title\npermalink: /Custom/Install/\n---\n# Install\n\n## Install {#install}\n\n[Home](../../index.html#home)\n",
        );
        write(
            &source,
            "pages/_reference/API.markdown",
            page("# API {#api}\n\n[Home](../index.md#home)\n\n![Local](diagram.svg)\n"),
        );
        write(&source, "pages/_reference/diagram.svg", "<svg>local</svg>");
        write(
            &source,
            "pages/Café.md",
            "---\npermalink: /Caf%C3%A9/\n---\n# Café\n",
        );
        write(&source, "pages/orphan.md", page("# Orphan"));
        write(&source, "pages/media/logo.svg", "<svg>logo</svg>");
        write(&source, "pages/media/Q3 Report.pdf", [0, 255, 254, 10]);
        write(
            &source,
            "pages/_data/navigation.yml",
            "- name: Home label\n  link: /\n- title: Guides\n  children:\n    - title: Install label\n      url: /Custom/Install/?next=https://example.test#install\n    - title: API label\n      url: /ref/API/\n    - title: Café\n      url: /Caf%C3%A9/\n- title: External\n  url: https://elsewhere.test/\n",
        );
        let detected = Command::new(env!("CARGO_BIN_EXE_zel"))
            .arg("detect")
            .arg(&source)
            .arg("--json")
            .output()
            .unwrap();
        let detected: serde_json::Value = serde_json::from_slice(&detected.stdout).unwrap();
        assert_eq!(detected["matches"][0]["id"], "jekyll");
        let output = run(&source, &out, &["--layout", "site"], 0);
        let content = out.join("content");
        let home = read(&content, "01-guide/index.mdx");
        for expected in [
            "title: \"Authored title\"",
            "description: \"Authored description\"",
            "[File](</guide/guide/install?from=file#install>)",
            "[Website](</guide/guide/install?from=website#install>)",
            "[Absolute](</guide/guide/install#install>)",
            "[Collection](</guide/reference/api#api>)",
            "[Collection URL](</guide/reference/api>)",
            "[Unicode](</guide/cafe>)",
            "![Logo](</_zellij/assets/imported/docs/media/logo.svg>)",
            "[Download](</_zellij/assets/imported/docs/media/Q3%20Report.pdf#page=2>)",
            "[Outside](https://elsewhere.test/manual/Custom/Install/)",
            "`[Example](/manual/Custom/Install/)`",
        ] {
            assert!(home.contains(expected), "{expected}: {home}");
        }
        assert!(!home.lines().any(|l| l.starts_with("route:")));
        assert!(home.contains("Original Jekyll front matter"));
        assert!(read(&content, "01-guide/Guide/Install.mdx").contains("[Home](</guide#home>)"));
        assert!(
            read(&content, "01-guide/_reference/API.mdx")
                .contains("![Local](</_zellij/assets/imported/docs/_reference/diagram.svg>)")
        );
        assert_eq!(
            fs::read(content.join("assets/imported/docs/media/Q3 Report.pdf")).unwrap(),
            [0, 255, 254, 10]
        );
        let sidebar = read(&content, "01-guide/_sidebar.yaml");
        assert!(sidebar.contains("Install label"));
        assert!(sidebar.contains("API label"));
        assert!(!content.join("01-guide/orphan.mdx").exists());
        assert!(report(&output).contains("orphan.md"));
        assert!(report(&output).contains("query/fragment"));
        assert!(report(&output).contains("External"));
        assert_eq!(read(&source, config), config_body);
        assert!(read(&out, "zel.yaml").contains("renderer: hosted"));
    }
}

#[test]
fn derived_navigation_respects_defaults_collection_order_and_visibility() {
    let t = tempfile::tempdir().unwrap();
    let source = t.path().join("source");
    let out = t.path().join("out");
    write(
        &source,
        "_config.yml",
        "title: Manual\ncollections:\n  docs:\n    output: true\n    order: [z-last.md, deep/start.md]\n  private:\n    output: false\ndefaults:\n  - scope: {path: '', type: docs}\n    values: {description: Collection description, published: true}\n  - scope: {path: _docs/deep, type: docs}\n    values: {title: Default title}\n  - scope: {path: _docs/deep}\n    values: {title: Less specific}\n  - scope: {path: hidden, type: pages}\n    values: {published: false}\n  - scope: {path: ''}\n    values: {description: Broad description}\n",
    );
    for p in [
        "index.md",
        "_docs/z-last.md",
        "_docs/a-first.md",
        "_docs/deep/start.md",
        "_private/secret.md",
        "_posts/2099-01-01-post.md",
        "_drafts/idea.md",
        "_site/generated.md",
        ".hidden.md",
        "node_modules/dependency.md",
        "vendor/dependency.md",
        "hidden/a.md",
    ] {
        write(&source, p, page("# Body\n"));
    }
    write(
        &source,
        "_docs/own.md",
        "---\ntitle: Own title\ndescription: Own description\n---\nBody\n",
    );
    write(
        &source,
        "unpublished.md",
        "---\npublished: false\n---\nSecret\n",
    );
    write(&source, "null.md", "---\npublished: null\n---\nSecret\n");
    write(
        &source,
        "nav-hidden.md",
        "---\nnav_exclude: true\n---\nSecret\n",
    );
    write(&source, "README.md", "# Static Markdown\n");
    let output = run(&source, &out, &[], 0);
    let nav: serde_yaml_ng::Value =
        serde_yaml_ng::from_str(&read(&out, "01-guide/_sidebar.yaml")).unwrap();
    let children = nav["nav"][1]["items"].as_sequence().unwrap();
    assert_eq!(
        children
            .iter()
            .map(|v| v.as_str().unwrap())
            .collect::<Vec<_>>(),
        [
            "_docs/z-last",
            "_docs/deep/start",
            "_docs/a-first",
            "_docs/own"
        ]
    );
    let inherited = read(&out, "01-guide/_docs/deep/start.mdx");
    assert!(inherited.contains("title: \"Default title\""));
    assert!(inherited.contains("description: \"Collection description\""));
    assert!(read(&out, "01-guide/_docs/own.mdx").contains("title: \"Own title\""));
    for p in [
        "_private/secret",
        "_posts/2099-01-01-post",
        "_drafts/idea",
        "_site/generated",
        ".hidden",
        "node_modules/dependency",
        "vendor/dependency",
        "hidden/a",
        "unpublished",
        "null",
        "nav-hidden",
        "README",
    ] {
        assert!(!out.join(format!("01-guide/{p}.mdx")).exists(), "{p}");
    }
    assert!(report(&output).contains("static file"));
    assert!(report(&output).contains("not enabled"));
}

#[test]
fn empty_navigation_stays_empty_and_linked_groups_preserve_order() {
    for navigation in [
        "[]",
        "main: []",
        "- title: Start\n  url: /\n  children:\n    - title: Next\n      url: /next.html\n- title: Again\n  url: /next.html\n",
    ] {
        let t = tempfile::tempdir().unwrap();
        let source = t.path().join("source");
        let out = t.path().join("out");
        write(&source, "_config.yml", "");
        write(&source, "index.md", page("# Home"));
        write(&source, "next.md", page("# Next"));
        write(&source, "_data/navigation.yaml", navigation);
        run(&source, &out, &[], 0);
        let nav: serde_yaml_ng::Value =
            serde_yaml_ng::from_str(&read(&out, "01-guide/_sidebar.yaml")).unwrap();
        if navigation.contains("[]") {
            assert_eq!(nav["nav"].as_sequence().unwrap().len(), 0);
            assert!(!out.join("01-guide/index.mdx").exists());
        } else {
            assert_eq!(nav["nav"][0]["items"][0]["page"].as_str(), Some("index"));
            assert_eq!(nav["nav"][0]["items"][1]["page"].as_str(), Some("next"));
            assert!(out.join("01-guide/next.mdx").exists());
        }
    }
}

#[test]
fn excluded_and_template_assets_are_never_copied_even_when_linked() {
    for skip in [false, true] {
        let t = tempfile::tempdir().unwrap();
        let source = t.path().join("source");
        let out = t.path().join("out");
        write(
            &source,
            "_config.yml",
            "exclude: [private, excluded.pdf]\ndata_dir: data\ndestination: output\n",
        );
        write(
            &source,
            "index.md",
            page(
                "# Home\n\n[Next](next.md)\n\n![Logo](logo.svg)\n\n[Secret](private/report.pdf)\n\n[Excluded](/excluded.pdf)\n\n[Config](_config.yml)\n\n[Generated](output/index.html)\n\n[Template](custom.css)\n\n[BOM template](bom.css)\n\n[Data](data/hidden.json)\n",
            ),
        );
        write(&source, "next.md", page("# Next"));
        write(&source, "logo.svg", "<svg/>");
        for path in [
            "private/report.pdf",
            "excluded.pdf",
            "output/index.html",
            "data/hidden.json",
        ] {
            write(&source, path, "Secret");
        }
        write(&source, "custom.css", "---\n---\nbody {color: red}\n");
        write(
            &source,
            "bom.css",
            "\u{feff}---\r\n---\r\nbody {color: blue}\r\n",
        );
        let output = run(
            &source,
            &out,
            if skip { &["--assets", "skip"] } else { &[] },
            0,
        );
        let home = read(&out, "01-guide/index.mdx");
        assert!(home.contains("[Next](</guide/next>)"));
        assert_eq!(out.join("assets/imported/docs/logo.svg").exists(), !skip);
        for path in [
            "private/report.pdf",
            "excluded.pdf",
            "_config.yml",
            "output/index.html",
            "custom.css",
            "bom.css",
            "data/hidden.json",
        ] {
            assert!(
                !out.join(format!("assets/imported/docs/{path}")).exists(),
                "{path}"
            );
        }
        assert!(report(&output).contains("excluded, template-only or unavailable"));
    }
}

#[test]
fn dry_run_and_ship_plans_do_not_write_or_invoke_runtimes() {
    let t = tempfile::tempdir().unwrap();
    let source = t.path().join("source");
    let out = t.path().join("out");
    write(&source, "_config.toml", "title = 'Docs'\n");
    write(&source, "index.md", page("# Home"));
    run(&source, &out, &["--dry-run", "--validate"], 0);
    assert!(!out.exists());
    let o = Command::new(env!("CARGO_BIN_EXE_zel"))
        .arg("ship")
        .arg(&source)
        .arg("--out")
        .arg(&out)
        .args([
            "--from",
            "jekyll",
            "--image",
            "static",
            "--tag",
            "example/docs:test",
            "--dry-run",
        ])
        .output()
        .unwrap();
    assert!(o.status.success(), "{o:?}");
    assert!(!out.exists());
}

#[test]
fn ambiguous_detection_and_config_files_require_explicit_resolution() {
    let t = tempfile::tempdir().unwrap();
    let source = t.path().join("source");
    let out = t.path().join("out");
    write(&source, "_config.yml", "title: Docs\n");
    write(&source, "index.md", page("# Home"));
    write(&source, "mkdocs.yml", "site_name: Other\n");
    assert!(report(&run(&source, &out, &[], 2)).contains("ambiguous source (jekyll, mkdocs)"));
    run(&source, &out, &["--from", "jekyll", "--dry-run"], 0);
    write(&source, "_config.yaml", "{}");
    assert!(report(&run(&source, &out, &["--from", "jekyll"], 2)).contains("exactly one"));
    fs::remove_file(source.join("_config.yaml")).unwrap();
    write(&source, "_config.toml", "title = 'TOML'\n");
    assert!(report(&run(&source, &out, &["--from", "jekyll"], 2)).contains("exactly one"));
    assert!(!out.exists());
}

#[test]
fn unsafe_or_unsupported_configuration_fails_before_writing() {
    let t = tempfile::tempdir().unwrap();
    let source = t.path().join("source");
    let out = t.path().join("out");
    write(&source, "index.md", page("# Home"));
    for config in [
        "source: ../outside",
        "source: /outside",
        "source: missing",
        "source: ''",
        "source: 42",
        "destination: .",
        "data_dir: .",
        "data_dir: ../outside",
        "exclude: ['**/secret.md']",
        "include: [secret.md]",
        "collections_dir: other",
        "collections: {docs: {output: true, sort_by: weight}}",
        "collections: {'../outside': {output: true}}",
        "collections: {docs: {output: yes}}",
        "defaults: [{scope: {path: '**/*.md'}, values: {title: X}}]",
        "defaults: [{scope: {unknown: foo}, values: {title: X}}]",
        "permalink: /:year/:title/",
        "plugins_dir: scripts",
        "title: !ruby/object:Thing {}",
        "title: a\ntitle: b",
    ] {
        write(&source, "_config.yml", config);
        run(&source, &out, &[], 2);
        assert!(!out.exists(), "{config}");
    }
}

#[test]
fn invalid_navigation_dates_and_permalink_values_fail_before_output() {
    let t = tempfile::tempdir().unwrap();
    let source = t.path().join("source");
    let out = t.path().join("out");
    write(
        &source,
        "_config.yml",
        "collections: {docs: {output: true}}\n",
    );
    write(&source, "index.md", page("# Home"));
    for nav in [
        "- name: Missing\n  link: /missing.html",
        "- name: Bad\n  link: /../../private",
        "main: []\nother: []",
        "- name: Both\n  title: Both\n  link: /",
        "- name: Dynamic\n  link: '{{ site.baseurl }}/'",
        "- name: Relative\n  link: index.md",
    ] {
        write(&source, "_data/navigation.yml", nav);
        run(&source, &out, &[], 2);
        assert!(!out.exists());
    }
    fs::remove_file(source.join("_data/navigation.yml")).unwrap();
    for permalink in [
        "../outside",
        "//host/file",
        "/bad%2fpath/",
        "/bad%FF",
        "/a/../b",
        "/:year/:title/",
        "/path?query=1",
        "/bad%00",
    ] {
        write(
            &source,
            "index.md",
            format!("---\npermalink: '{permalink}'\n---\n# Home\n"),
        );
        run(&source, &out, &[], 2);
        assert!(!out.exists());
    }
    write(&source, "index.md", page("# Home"));
    write(
        &source,
        "_docs/dates.md",
        "---\ndate: 2099-01-01\n---\nFuture\n",
    );
    assert!(report(&run(&source, &out, &[], 2)).contains("dated collections"));
    fs::remove_file(source.join("_docs/dates.md")).unwrap();
    write(&source, "_docs/2099-01-01-future.md", page("Future\n"));
    run(&source, &out, &[], 2);
    assert!(!out.exists());
}

#[test]
fn liquid_is_reported_preserved_and_never_executed() {
    let t = tempfile::tempdir().unwrap();
    let source = t.path().join("source");
    let out = t.path().join("out");
    write(
        &source,
        "_config.yml",
        "plugins: [dangerous-plugin]\ntheme: custom-theme\n",
    );
    write(
        &source,
        "index.md",
        page(
            "# Home\n\n{% include secret.html %}\n\n{{ site.title }}\n\n```liquid\n{{ example }}\n```\n",
        ),
    );
    write(
        &source,
        "_plugins/execute.rb",
        "File.write('SOURCE_EXECUTED', 'bad')\n",
    );
    write(&source, "_includes/secret.html", "secret");
    let output = run(&source, &out, &[], 0);
    let imported = read(&out, "01-guide/index.mdx");
    assert!(imported.contains("include secret.html"));
    assert!(imported.contains("site.title"));
    assert!(report(&output).contains("Liquid expressions"));
    assert!(!source.join("SOURCE_EXECUTED").exists());
    assert!(!out.join("assets").exists());
}

#[test]
fn unsafe_asset_paths_and_output_collisions_leave_destination_absent() {
    let t = tempfile::tempdir().unwrap();
    let source = t.path().join("source");
    let out = t.path().join("out");
    write(&source, "_config.yml", "{}");
    for link in ["../outside.pdf", "/.git/config", "/bad%5cpath", "/bad%FF"] {
        write(
            &source,
            "index.md",
            page(&format!("# Home\n\n[Bad]({link})\n")),
        );
        run(&source, &out, &["--assets", "skip"], 2);
        assert!(!out.exists());
    }
    write(&source, "index.md", page("# Home"));
    write(&source, "page.md", page("# First"));
    write(&source, "01-Page.markdown", page("# Duplicate"));
    assert!(report(&run(&source, &out, &[], 2)).contains("would render at"));
    assert!(!out.exists());
}

#[cfg(unix)]
#[test]
fn linked_configuration_documents_and_assets_are_rejected() {
    use std::os::unix::fs::symlink;
    let t = tempfile::tempdir().unwrap();
    let source = t.path().join("source");
    let out = t.path().join("out");
    write(t.path(), "outside/config.yml", "title: Docs\n");
    write(&source, "index.md", page("# Home"));
    symlink(
        t.path().join("outside/config.yml"),
        source.join("_config.yml"),
    )
    .unwrap();
    run(&source, &out, &[], 2);
    fs::remove_file(source.join("_config.yml")).unwrap();
    write(t.path(), "outside/config.toml", "title = 'Docs'\n");
    symlink(
        t.path().join("outside/config.toml"),
        source.join("_config.toml"),
    )
    .unwrap();
    run(&source, &out, &[], 2);
    fs::remove_file(source.join("_config.toml")).unwrap();
    write(&source, "_config.yml", "{}");
    symlink(t.path().join("outside"), source.join("linked")).unwrap();
    run(&source, &out, &[], 2);
    fs::remove_file(source.join("linked")).unwrap();
    write(t.path(), "outside/asset.svg", "secret");
    symlink(t.path().join("outside/asset.svg"), source.join("logo.svg")).unwrap();
    write(&source, "index.md", page("# Home\n\n![Logo](logo.svg)\n"));
    run(&source, &out, &["--assets", "skip"], 2);
    assert!(!out.exists());
}

#[test]
fn default_permalink_styles_resolve_relative_website_links() {
    for style in ["date", "pretty", "ordinal", "none"] {
        let t = tempfile::tempdir().unwrap();
        let source = t.path().join("source");
        let out = t.path().join("out");
        write(
            &source,
            "_config.yml",
            format!("permalink: {style}\nlang: en\n"),
        );
        let url = if style == "pretty" {
            "/topic/"
        } else {
            "/topic.html"
        };
        let home = if style == "pretty" {
            "../index.html"
        } else {
            "index.html"
        };
        write(
            &source,
            "index.md",
            page(&format!("# Home\n\n[Topic]({url}?a=1#topic)\n")),
        );
        write(
            &source,
            "topic.md",
            page(&format!("# Topic {{#topic}}\n\n[Home]({home})\n")),
        );
        run(&source, &out, &[], 0);
        assert!(read(&out, "01-guide/index.mdx").contains("[Topic](</guide/topic?a=1#topic>)"));
        assert!(read(&out, "01-guide/topic.mdx").contains("[Home](</guide>)"));
    }
}

#[test]
fn collection_permalink_tokens_must_match_whole_supported_placeholders() {
    let t = tempfile::tempdir().unwrap();
    let source = t.path().join("source");
    let out = t.path().join("out");
    write(&source, "_docs/page.md", page("# Page\n"));
    for pattern in ["/:pathname/", "/:collection_extra/:path/", "/:title/"] {
        write(
            &source,
            "_config.yml",
            format!("collections: {{docs: {{output: true, permalink: '{pattern}'}}}}\n"),
        );
        run(&source, &out, &[], 2);
        assert!(!out.exists());
    }
}

#[test]
fn toml_tables_and_arrays_preserve_defaults_order_visibility_and_authored_values() {
    let t = tempfile::tempdir().unwrap();
    let source = t.path().join("source");
    let out = t.path().join("out");
    let config = r#"title = 'Café manual'
description = """First line.
Second line."""
source = 'pages'
exclude = ['private']
[collections.reference]
output = true
order = ['z-last.md', 'a-first.md']
[[defaults]]
scope = { path = '', type = 'reference' }
[defaults.values]
title = 'Inherited title'
description = 'Inherited description'
published = true
weight = 7
ratio = 1.5
updated = 2026-11-09T12:30:00Z
Tags = ['one', 'two']
[[defaults]]
scope = { path = 'hidden', type = 'pages' }
values = { published = false }
"#;
    write(&source, "_config.toml", config);
    write(&source, "pages/index.md", page("# Home\n"));
    write(&source, "pages/_reference/z-last.md", page("# Last\n"));
    write(
        &source,
        "pages/_reference/a-first.md",
        "---\ntitle: Authored title\ndescription: Authored description\n---\n# First\n",
    );
    write(&source, "pages/hidden/page.md", page("# Hidden\n"));
    write(&source, "pages/private/page.md", page("# Private\n"));
    run(&source, &out, &["--layout", "site"], 0);
    let content = out.join("content");
    let site: serde_yaml_ng::Value = serde_yaml_ng::from_str(&read(&content, "site.yaml")).unwrap();
    assert_eq!(site["name"].as_str(), Some("Café manual"));
    let nav: serde_yaml_ng::Value =
        serde_yaml_ng::from_str(&read(&content, "01-guide/_sidebar.yaml")).unwrap();
    assert_eq!(
        nav["nav"][1]["items"][0].as_str(),
        Some("_reference/z-last")
    );
    assert_eq!(
        nav["nav"][1]["items"][1].as_str(),
        Some("_reference/a-first")
    );
    let inherited = read(&content, "01-guide/_reference/z-last.mdx");
    assert!(inherited.contains("title: \"Inherited title\""));
    assert!(inherited.contains("description: \"Inherited description\""));
    assert!(inherited.contains("2026-11-09T12:30:00Z"));
    assert!(!inherited.contains("$__toml_private"));
    let own = read(&content, "01-guide/_reference/a-first.mdx");
    assert!(own.contains("title: \"Authored title\""));
    assert!(own.contains("description: \"Authored description\""));
    assert!(!content.join("01-guide/hidden/page.mdx").exists());
    assert!(!content.join("01-guide/private/page.mdx").exists());
    assert_eq!(read(&source, "_config.toml"), config);
}

#[test]
fn malformed_or_unsupported_toml_fails_before_output() {
    let t = tempfile::tempdir().unwrap();
    let source = t.path().join("source");
    let out = t.path().join("out");
    write(&source, "index.md", page("# Home\n"));
    for config in [
        "title = 'A'\ntitle = 'B'",
        "title = { broken",
        "title = ${ENV}",
        "source = '../outside'",
        "source = '/outside'",
        "source = 42",
        "destination = '.'",
        "data_dir = '../private'",
        "exclude = ['**/private.md']",
        "include = ['secret.md']",
        "collections.docs.output = 'yes'",
        "collections.docs.sort_by = 'weight'",
        "defaults = [{scope = {path = '**/*.md'}, values = {title = 'X'}}]",
        "defaults = [{scope = {}, values = {published = 'false'}}]",
        "permalink = '/:year/:title/'",
        "plugins_dir = 'scripts'",
    ] {
        write(&source, "_config.toml", config);
        run(&source, &out, &["--dry-run"], 2);
        run(&source, &out, &[], 2);
        assert!(!out.exists(), "{config}");
        assert_eq!(read(&source, "_config.toml"), config);
    }
    // An inherited date still enters the existing explicit dated-collection
    // diagnostic, rather than hiding in TOML's private datetime map.
    write(&source, "_docs/page.md", page("# Page\n"));
    write(
        &source,
        "_config.toml",
        "collections.docs.output = true\n[[defaults]]\nscope = {type = 'docs'}\nvalues = {date = 2099-01-01}\n",
    );
    assert!(report(&run(&source, &out, &[], 2)).contains("dated collections"));
    assert!(!out.exists());
    // Empty TOML is a valid empty configuration, like an empty YAML config.
    write(&source, "_config.toml", "# Empty config\n");
    run(&source, &out, &[], 0);
    assert!(out.join("01-guide/index.mdx").exists());
}
