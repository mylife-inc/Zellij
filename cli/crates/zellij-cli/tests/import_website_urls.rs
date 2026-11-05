//! Real CLI imports: original website addresses resolve to emitted routes.
use std::{
    fs,
    path::Path,
    process::{Command, Output},
};

fn write(root: &Path, name: &str, body: impl AsRef<[u8]>) {
    let file = root.join(name);
    fs::create_dir_all(file.parent().unwrap()).unwrap();
    fs::write(file, body).unwrap();
}
fn convert(source: &Path, out: &Path, extra: &[&str], code: i32) -> Output {
    let output = Command::new(env!("CARGO_BIN_EXE_zel"))
        .args([
            "cv",
            source.to_str().unwrap(),
            "--out",
            out.to_str().unwrap(),
        ])
        .args(extra)
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(code), "{output:?}");
    output
}
fn text(root: &Path, name: &str) -> String {
    fs::read_to_string(root.join(name)).unwrap()
}
fn docusaurus(root: &Path, config: &str, sidebar: &str) {
    write(root, "docusaurus.config.js", config);
    write(
        root,
        "sidebars.js",
        format!("module.exports = {{docs: {sidebar}}};"),
    );
}

#[test]
fn mkdocs_directory_and_html_urls_preserve_prefix_origin_suffixes_and_asset_bytes() {
    for directories in [true, false] {
        let temp = tempfile::tempdir().unwrap();
        let source = temp.path().join("source");
        let out = temp.path().join("out");
        write(
            &source,
            "mkdocs.yml",
            format!(
                "site_name: Website links\nsite_url: https://docs.example.test/old/manual/\nuse_directory_urls: {directories}\nnav: [index.md, 02-Guides/03-Start.md, Café.md, nested/README.md]\n"
            ),
        );
        let end = if directories { "/" } else { ".html" };
        let home = if directories {
            "/old/manual/"
        } else {
            "/old/manual/index.html"
        };
        let index = format!(
            "# Home\n\n[Start](/old/manual/02-Guides/03-Start{end}?v=2#install)\n\n[Absolute](https://DOCS.example.test:443/old/manual/Caf%C3%A9{end}#intro)\n\n[Readme](/old/manual/nested/{})\n\n[Unlisted](/old/manual/orphan{end})\n\n[Foreign](https://elsewhere.test/old/manual/Caf%C3%A9{end})\n\n[Outside](https://docs.example.test/old/manual-extra/Caf%C3%A9{end})\n\n![Logo](/old/manual/media/logo.svg?raw=1)\n\n[Download](https://docs.example.test/old/manual/media/report.pdf#page=2)\n\n`[Literal](/old/manual/Caf%C3%A9{end})`\n",
            if directories { "" } else { "index.html" }
        );
        write(&source, "docs/index.md", &index);
        write(
            &source,
            "docs/02-Guides/03-Start.md",
            format!("# Start\n\n## Install\n\n[Home]({home})\n"),
        );
        write(&source, "docs/Café.md", "# Café\n\n## Intro\n");
        write(&source, "docs/nested/README.md", "# Nested\n");
        write(&source, "docs/orphan.md", "# Do not publish\n");
        write(&source, "docs/media/logo.svg", "<svg/>");
        write(&source, "docs/media/report.pdf", [0, 255, 3, 4]);
        let output = convert(&source, &out, &[], 0);
        let page = text(&out, "01-guide/index.mdx");
        for expected in [
            "[Start](</guide/guides/start?v=2#install>)",
            "[Absolute](</guide/cafe#intro>)",
            "[Readme](</guide/nested/readme>)",
            "![Logo](</_zellij/assets/imported/docs/media/logo.svg?raw=1>)",
            "[Download](</_zellij/assets/imported/docs/media/report.pdf#page=2>)",
        ] {
            assert!(page.contains(expected), "{expected}: {page}");
        }
        assert!(page.contains(&format!("[Unlisted](/old/manual/orphan{end})")));
        assert!(page.contains("[Foreign](https://elsewhere.test/"));
        assert!(page.contains("[Outside](https://docs.example.test/old/manual-extra/"));
        assert!(page.contains(&format!("`[Literal](/old/manual/Caf%C3%A9{end})`")));
        assert!(text(&out, "01-guide/02-Guides/03-Start.mdx").contains("[Home](</guide>)"));
        assert!(String::from_utf8_lossy(&output.stdout).contains("its document was not imported"));
        assert_eq!(
            fs::read(out.join("assets/imported/docs/media/report.pdf")).unwrap(),
            [0, 255, 3, 4]
        );
        assert_eq!(text(&source, "docs/index.md"), index);
    }
}

#[test]
fn mkdocs_default_null_and_empty_site_urls_use_their_supported_url_style() {
    for (setting, url) in [
        ("", "/next/"),
        ("site_url: null\n", "/next/"),
        ("site_url: ''\n", "/next.html"),
    ] {
        let temp = tempfile::tempdir().unwrap();
        let source = temp.path().join("source");
        let out = temp.path().join("out");
        write(
            &source,
            "mkdocs.yml",
            format!("site_name: Links\n{setting}nav: [index.md, next.md]\n"),
        );
        write(
            &source,
            "docs/index.md",
            format!("# Home\n\n[Next]({url})\n"),
        );
        write(&source, "docs/next.md", "# Next");
        convert(&source, &out, &[], 0);
        assert!(text(&out, "01-guide/index.mdx").contains("[Next](</guide/next>)"));
    }
}

#[test]
fn docusaurus_website_urls_follow_literal_prefixes_ids_slugs_indexes_and_categories() {
    for (selection, prefix) in [
        (
            "presets: [['classic', {docs: {routeBasePath: 'api'}}]]",
            "/api",
        ),
        (
            "plugins: [['@docusaurus/plugin-content-docs', {routeBasePath: '/' }]]",
            "",
        ),
        ("", "/docs"),
    ] {
        let temp = tempfile::tempdir().unwrap();
        let source = temp.path().join("source");
        let out = temp.path().join("out");
        docusaurus(
            &source,
            &format!(
                "/* Config */\nexport default {{title: 'Links', url: 'https://docs.example.test', baseUrl: '/old/', {selection}}};"
            ),
            "[{type:'category',label:'Overview',link:{type:'generated-index',slug:'/Overview'},items:[{type:'autogenerated',dirName:'.'}]}]",
        );
        let original = format!(
            "# Home\n\n[Custom](/old{prefix}/Guides/custom?q=a%2Fb#install)\n\n[Explicit](https://docs.example.test/old{prefix}/Public/API)\n\n[Unicode](/old{prefix}/Caf%C3%A9/Release%20Notes)\n\n[Readme](/old{prefix}/Nested)\n\n[Folder index](/old{prefix}/Same)\n\n[Overview](/old{prefix}/Overview)\n\n[Draft](/old{prefix}/draft)\n\n![Prefixed](/old/img/logo.svg)\n\n![Legacy](/img/logo.svg)\n\n[Asset](https://docs.example.test/old/img/logo.svg?download=1)\n\n[Foreign](https://docs.example.test.evil/old{prefix}/Overview)\n"
        );
        write(&source, "docs/index.md", &original);
        write(
            &source,
            "docs/01-Guides/02-Start.md",
            "---\nid: custom\n---\n# Start\n\n## Install\n",
        );
        write(
            &source,
            "docs/explicit.md",
            "---\nslug: /Public/API\n---\n# API\n",
        );
        write(
            &source,
            "docs/cafe.md",
            "---\nslug: /Caf%C3%A9/Release%20Notes\n---\n# Café\n",
        );
        write(
            &source,
            "docs/Nested/README.md",
            "---\nid: changed\n---\n# Nested\n",
        );
        write(&source, "docs/Same/Same.md", "# Same\n");
        write(&source, "docs/draft.md", "---\ndraft: true\n---\n# Draft\n");
        write(&source, "static/img/logo.svg", "<svg/>");
        let output = convert(&source, &out, &["--layout", "site"], 0);
        let content = out.join("content");
        let page = text(&content, "01-guide/index.mdx");
        for expected in [
            "[Custom](</guide/guides/start?q=a%2Fb#install>)",
            "[Explicit](</guide/Public/API>)",
            "[Unicode](</guide/Caf%C3%A9/Release%20Notes>)",
            "[Readme](</guide/nested/readme>)",
            "[Folder index](</guide/same/same>)",
            "[Overview](</guide/Overview>)",
            "![Prefixed](</_zellij/assets/imported/static/img/logo.svg>)",
            "![Legacy](</_zellij/assets/imported/static/img/logo.svg>)",
            "[Asset](</_zellij/assets/imported/static/img/logo.svg?download=1>)",
        ] {
            assert!(page.contains(expected), "{expected}: {page}");
        }
        assert!(page.contains(&format!("[Draft](/old{prefix}/draft)")));
        assert!(page.contains("[Foreign](https://docs.example.test.evil/"));
        assert!(String::from_utf8_lossy(&output.stdout).contains("its document was not imported"));
        assert_eq!(
            text(&content, "assets/imported/static/img/logo.svg"),
            "<svg/>"
        );
        assert_eq!(text(&source, "docs/index.md"), original);
    }
}

#[test]
fn ambiguous_website_addresses_are_reported_instead_of_choosing_a_page() {
    for docusaurus_source in [false, true] {
        let temp = tempfile::tempdir().unwrap();
        let source = temp.path().join("source");
        let out = temp.path().join("out");
        let url = if docusaurus_source { "/docs/" } else { "/" };
        if docusaurus_source {
            docusaurus(&source, "module.exports = {};", "['index','README']");
        } else {
            write(
                &source,
                "mkdocs.yml",
                "site_name: Collision\nnav: [index.md, README.md]\n",
            );
        }
        write(
            &source,
            "docs/index.md",
            format!("# Index\n\n[Ambiguous]({url})\n"),
        );
        write(&source, "docs/README.md", "# Readme");
        let result = convert(&source, &out, &[], 0);
        assert!(text(&out, "01-guide/index.mdx").contains(&format!("[Ambiguous]({url})")));
        assert!(
            String::from_utf8_lossy(&result.stdout).contains("source website URL is ambiguous")
        );
    }
}

#[test]
fn computed_custom_and_versioned_docusaurus_configs_are_reported_without_execution() {
    for config in [
        "// module.exports = {baseUrl: '/wrong/'};\nmodule.exports = makeConfig();",
        "module.exports = {url: process.env.URL, baseUrl: '/old/'};",
        "module.exports = {presets: [['custom', {}]]};",
        "module.exports = {i18n: {locales:['en','fr']}};",
        "module.exports = {presets: [['classic',{docs:{routeBasePath:'api',path:'other'}}]]};",
        "module.exports = {presets: [['classic',{docs:{lastVersion:'1.0'}}]]};",
        "module.exports = {plugins: [['content-docs',{}],['content-docs',{id:'other'}]]};",
    ] {
        let temp = tempfile::tempdir().unwrap();
        let source = temp.path().join("source");
        let out = temp.path().join("out");
        docusaurus(&source, config, "['index','next']");
        write(
            &source,
            "docs/index.md",
            "# Home\n\n[Website](/docs/next)\n\n[File](next.md)\n",
        );
        write(&source, "docs/next.md", "# Next\n");
        let result = convert(&source, &out, &[], 0);
        let page = text(&out, "01-guide/index.mdx");
        assert!(page.contains("[Website](/docs/next)"));
        assert!(page.contains("[File](</guide/next>)"));
        assert!(
            String::from_utf8_lossy(&result.stdout)
                .contains("Source website URL mapping was skipped")
        );
    }
}

#[test]
fn website_matching_decodes_once_keeps_case_and_never_claims_foreign_origins() {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("source");
    let out = temp.path().join("out");
    write(
        &source,
        "mkdocs.yml",
        "site_name: Exact URLs\nsite_url: http://127.0.0.1:9/old/\nnav: [index.md, Café.md]\n",
    );
    write(&source, "docs/Café.md", "# Café");
    write(
        &source,
        "docs/literal%20name.txt",
        "literal percent filename",
    );
    write(
        &source,
        "docs/index.md",
        "# Home\n\n[Unicode](/old/Cafe%CC%81/)\n\n[Literal](/old/literal%2520name.txt)\n\n[Case](/old/café/)\n\n[Port](http://127.0.0.1:10/old/Caf%C3%A9/)\n\n[Scheme](https://127.0.0.1:9/old/Caf%C3%A9/)\n\n[Relative host](//127.0.0.1:9/old/Caf%C3%A9/)\n\n[Noncanonical](http:127.0.0.1/old/Caf%C3%A9/)\n\n[Local HTTP](http://127.0.0.1:9/old/Caf%C3%A9/)\n",
    );
    convert(&source, &out, &[], 0);
    let page = text(&out, "01-guide/index.mdx");
    assert!(page.contains("[Unicode](</guide/cafe>)"));
    assert!(page.contains("[Local HTTP](</guide/cafe>)"));
    assert!(page.contains("[Literal](</_zellij/assets/imported/docs/literal%2520name.txt>)"));
    assert_eq!(
        text(&out, "assets/imported/docs/literal%20name.txt"),
        "literal percent filename"
    );
    for unchanged in [
        "[Case](/old/café/)",
        "[Port](http://127.0.0.1:10/",
        "[Scheme](https://127.0.0.1:9/",
        "[Relative host](//127.0.0.1:9/",
        "[Noncanonical](http:127.0.0.1/",
    ] {
        assert!(page.contains(unchanged), "{unchanged}: {page}");
    }
}

#[test]
fn docusaurus_invalid_literal_routing_fails_and_versions_file_disables_url_inference() {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("source");
    let out = temp.path().join("out");
    write(&source, "docs/index.md", "# Home\n\n[Next](/docs/next)\n");
    write(&source, "docs/next.md", "# Next");
    for config in [
        "module.exports = {url: 'https://example.test/path'};",
        "module.exports = {baseUrl: 'relative'};",
        "module.exports = {baseUrl: 42};",
        "module.exports = {presets:[['classic',{docs:{routeBasePath:42}}]]};",
        "module.exports = {presets:[['classic',{docs:{routeBasePath:'//host'}}]]};",
    ] {
        docusaurus(&source, config, "['index','next']");
        convert(&source, &out, &[], 2);
        assert!(!out.exists());
    }
    docusaurus(&source, "module.exports = {};", "['index','next']");
    write(&source, "versions.json", "[\"1.0\"]");
    let result = convert(&source, &out, &[], 0);
    assert!(
        String::from_utf8_lossy(&result.stdout).contains("versioned or localized source routing")
    );
    assert!(text(&out, "01-guide/index.mdx").contains("[Next](/docs/next)"));
}

#[test]
fn website_asset_skip_and_dry_run_preserve_the_same_import_contract() {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("source");
    let out = temp.path().join("out");
    write(
        &source,
        "mkdocs.yml",
        "site_name: Skip\nsite_url: https://docs.example.test/old/\nnav: [index.md, next.md]\n",
    );
    write(
        &source,
        "docs/index.md",
        "# Home\n\n[Next](/old/next/)\n\n![Logo](https://docs.example.test/old/logo.svg)\n",
    );
    write(&source, "docs/next.md", "# Next");
    write(&source, "docs/logo.svg", "<svg/>");
    let output = convert(&source, &out, &["--assets", "skip", "--dry-run"], 0);
    assert!(!out.exists());
    assert!(String::from_utf8_lossy(&output.stdout).contains("asset copying is disabled"));
    convert(&source, &out, &["--assets", "skip"], 0);
    let page = text(&out, "01-guide/index.mdx");
    assert!(page.contains("[Next](</guide/next>)"));
    assert!(page.contains("![Logo](https://docs.example.test/old/logo.svg)"));
    assert!(!out.join("assets").exists());
}

#[test]
fn unsafe_website_asset_paths_and_invalid_routing_config_fail_before_writes() {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("source");
    let out = temp.path().join("out");
    write(
        &source,
        "mkdocs.yml",
        "site_name: Safe\nsite_url: https://docs.example.test/old/\nnav: [index.md]\n",
    );
    for reference in [
        "/../../secret",
        "/old/%2Fetc/passwd",
        "/old/%5csecret",
        "/old/.git/config",
        "/old/bad%FF",
        "/old/file%00",
    ] {
        write(
            &source,
            "docs/index.md",
            format!("# Home\n\n[Bad]({reference})\n"),
        );
        convert(&source, &out, &["--assets", "skip"], 2);
        assert!(!out.exists());
    }
    write(&source, "docs/index.md", "# Home");
    for config in [
        "site_url: ftp://example.test/",
        "site_url: https://user:secret@example.test/",
        "site_url: https://example.test/?bad=1",
        "site_url: 42",
        "use_directory_urls: string",
    ] {
        write(
            &source,
            "mkdocs.yml",
            format!("site_name: Safe\nnav: [index.md]\n{config}\n"),
        );
        convert(&source, &out, &[], 2);
        assert!(!out.exists());
    }
    #[cfg(unix)]
    {
        write(&source, "mkdocs.yml", "site_name: Safe\nnav: [index.md]\n");
        write(&source, "docs/index.md", "# Home\n\n[Bad](/secret.txt)\n");
        write(temp.path(), "secret.txt", "private");
        std::os::unix::fs::symlink(
            temp.path().join("secret.txt"),
            source.join("docs/secret.txt"),
        )
        .unwrap();
        convert(&source, &out, &[], 2);
        assert!(!out.exists());
    }
}
