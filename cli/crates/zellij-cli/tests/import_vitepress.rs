//! Native VitePress acceptance, without Node, source execution or containers.
use std::{
    fs,
    path::Path,
    process::{Command, Output},
};

fn write(root: &Path, file: &str, bytes: impl AsRef<[u8]>) {
    let target = root.join(file);
    fs::create_dir_all(target.parent().unwrap()).unwrap();
    fs::write(target, bytes).unwrap();
}
fn read(root: &Path, file: &str) -> String {
    fs::read_to_string(root.join(file)).unwrap()
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
fn report(output: &Output) -> String {
    format!(
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

#[test]
fn literal_configs_detect_and_import_titles_navigation_urls_and_asset_bytes() {
    for (extension, clean) in [("ts", true), ("js", false), ("mjs", true), ("mts", false)] {
        let temp = tempfile::tempdir().unwrap();
        let source = temp.path().join("source");
        let out = temp.path().join("out");
        let config = format!(
            "\u{feff}// Config\nimport {{ defineConfig }} from 'vitepress'\nexport default defineConfig({{title:'A: handbook', description:'Product docs', srcDir:'./pages/', base:'/old/manual/', cleanUrls:{clean}, themeConfig:{{sidebar:[{{text:'Home label',link:'/'}},{{text:'Guides',items:[{{text:'Install label',link:'/02-Guides/Install'}},{{link:'/Caf%C3%A9.md'}}]}},{{text:'External',link:'https://example.test'}}]}}}}); // done\n"
        );
        write(&source, &format!(".vitepress/config.{extension}"), &config);
        write(
            &source,
            "pages/index.md",
            "---\ntitle: Authored title\ndescription: Authored description\nroute: /must-not-activate\n---\n# Source title {#home}\n\n[File](02-Guides/Install.md?from=file#install)\n\n[Website](/old/manual/02-Guides/Install.html?from=web#install)\n\n[Authored](/02-Guides/Install)\n\n[Unicode](/Caf%C3%A9.html)\n\n![Relative](media/logo.svg)\n\n![Root](/media/logo.svg)\n\n![Public](/logo.svg)\n\n[Download](/old/manual/report.pdf#page=2)\n\n[Omitted](/orphan.html)\n\n[External](https://example.test/02-Guides/Install.html)\n\n`[Example](/02-Guides/Install)`\n",
        );
        write(
            &source,
            "pages/02-Guides/Install.md",
            "# Install\n\n## Install {#install}\n\n[Home](../index.html#home)\n",
        );
        write(&source, "pages/Café.md", "# Café\n");
        write(&source, "pages/orphan.md", "# Not in navigation\n");
        write(&source, "pages/media/logo.svg", "<svg>source</svg>");
        write(&source, "pages/public/logo.svg", "<svg>public</svg>");
        write(&source, "pages/public/report.pdf", [0, 255, 254, 9]);
        let detected = Command::new(env!("CARGO_BIN_EXE_zel"))
            .arg("detect")
            .arg(&source)
            .arg("--json")
            .output()
            .unwrap();
        let detection: serde_json::Value = serde_json::from_slice(&detected.stdout).unwrap();
        assert_eq!(detection["matches"][0]["id"], "vitepress");
        assert_eq!(detection["matches"][0]["strength"], "certain");
        let output = run(&source, &out, &["--layout", "site"], 0);
        let content = out.join("content");
        let page = read(&content, "01-guide/index.mdx");
        for expected in [
            "title: \"Authored title\"",
            "description: \"Authored description\"",
            "[File](</guide/guides/install?from=file#install>)",
            "[Website](</guide/guides/install?from=web#install>)",
            "[Authored](</guide/guides/install>)",
            "[Unicode](</guide/cafe>)",
            "![Relative](</_zellij/assets/imported/docs/media/logo.svg>)",
            "![Root](</_zellij/assets/imported/docs/media/logo.svg>)",
            "![Public](</_zellij/assets/imported/public/logo.svg>)",
            "[Download](</_zellij/assets/imported/public/report.pdf#page=2>)",
            "[Omitted](/orphan.html)",
            "`[Example](/02-Guides/Install)`",
            "[External](https://example.test/02-Guides/Install.html)",
        ] {
            assert!(page.contains(expected), "{expected}: {page}");
        }
        assert!(!page.lines().any(|line| line.starts_with("route:")));
        assert!(read(&content, "01-guide/02-Guides/Install.mdx").contains("[Home](</guide#home>)"));
        assert!(read(&content, "01-guide/_sidebar.yaml").contains("Install label"));
        assert!(report(&output).contains("orphan.md"));
        assert!(!content.join("01-guide/orphan.mdx").exists());
        assert_eq!(
            fs::read(content.join("assets/imported/public/report.pdf")).unwrap(),
            [0, 255, 254, 9]
        );
        assert_eq!(
            read(&source, &format!(".vitepress/config.{extension}")),
            config
        );
        assert!(read(&out, "zel.yaml").contains("renderer: hosted"));
    }
}

#[test]
fn multiple_sidebars_bases_linked_groups_and_repeated_pages_preserve_order() {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("source");
    let out = temp.path().join("out");
    write(
        &source,
        ".vitepress/config.js",
        "export default {themeConfig:{sidebar:{'/guide/':{base:'/guide/',items:[{text:'Guide',link:'',items:[]}]}}}};",
    );
    write(&source, "guide/index.md", "# Guide\n");
    write(&source, "guide/next.md", "# Next\n");
    write(&source, "api/start.md", "# API\n");
    // An empty link is rejected; correct it without losing any source files.
    run(&source, &out, &[], 2);
    assert!(!out.exists());
    write(
        &source,
        ".vitepress/config.js",
        "export default {themeConfig:{sidebar:{'/guide/':{base:'/guide/',items:[{text:'Guide',link:'index',collapsed:true,items:[{text:'Next',link:'next'},{text:'API',base:'/api/',link:'start'}]},{link:'next'}]},'/api/':[{text:'Reference',link:'/api/start.md'}]}}};",
    );
    let output = run(&source, &out, &[], 0);
    let sidebar: serde_yaml_ng::Value =
        serde_yaml_ng::from_str(&read(&out, "01-guide/_sidebar.yaml")).unwrap();
    let nav = sidebar["nav"].as_sequence().unwrap();
    assert_eq!(nav[0]["label"].as_str(), Some("/api/"));
    assert_eq!(nav[1]["label"].as_str(), Some("/guide/"));
    let children = nav[1]["items"][0]["items"].as_sequence().unwrap();
    assert_eq!(
        children
            .iter()
            .map(|v| v["page"].as_str().unwrap())
            .collect::<Vec<_>>(),
        ["guide/index", "guide/next", "api/start"]
    );
    assert!(out.join("01-guide/guide/next.mdx").is_file());
    assert!(report(&output).contains("collapsed"));
    assert!(report(&output).contains("combined into one guide"));
}

#[test]
fn missing_sidebar_derives_visible_pages_and_explicit_empty_sidebars_do_not() {
    for setting in [
        "",
        "themeConfig:{sidebar:[]}",
        "themeConfig:{sidebar:{}}",
        "themeConfig:{sidebar:false}",
    ] {
        let temp = tempfile::tempdir().unwrap();
        let source = temp.path().join("source");
        let out = temp.path().join("out");
        write(
            &source,
            ".vitepress/config.mjs",
            format!("export default {{srcDir:'./',{setting}}};"),
        );
        for file in [
            "index.md",
            "deep/next.md",
            "public/download.md",
            ".vitepress/private.md",
            "node_modules/private.md",
            "_partials/private.md",
            ".hidden.md",
        ] {
            write(&source, file, "# Page\n");
        }
        let output = run(&source, &out, &[], 0);
        assert_eq!(out.join("01-guide/index.mdx").exists(), setting.is_empty());
        assert_eq!(
            out.join("01-guide/deep/next.mdx").exists(),
            setting.is_empty()
        );
        for file in [
            "public/download",
            ".vitepress/private",
            "node_modules/private",
            "_partials/private",
            ".hidden",
        ] {
            assert!(!out.join(format!("01-guide/{file}.mdx")).exists());
        }
        assert!(report(&output).contains(if setting.is_empty() {
            "derived from visible"
        } else {
            "absent from the navigation"
        }));
    }
}

#[test]
fn assets_skip_dry_run_and_ship_plans_are_inert() {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("source");
    let out = temp.path().join("out");
    write(&source, ".vitepress/config.ts", "export default {};");
    write(
        &source,
        "index.md",
        "# Home\n\n[Next](next.md)\n\n![Logo](/logo.svg)\n",
    );
    write(&source, "next.md", "# Next\n");
    write(&source, "public/logo.svg", "<svg/>");
    run(&source, &out, &["--dry-run", "--validate"], 0);
    assert!(!out.exists());
    let result = run(&source, &out, &["--assets", "skip"], 0);
    assert!(read(&out, "01-guide/index.mdx").contains("[Next](</guide/next>)"));
    assert!(read(&out, "01-guide/index.mdx").contains("![Logo](/logo.svg)"));
    assert!(!out.join("assets").exists());
    assert!(report(&result).contains("asset copying is disabled"));
    let planned = temp.path().join("planned");
    let result = Command::new(env!("CARGO_BIN_EXE_zel"))
        .arg("ship")
        .arg(&source)
        .arg("--out")
        .arg(&planned)
        .args([
            "--from",
            "vitepress",
            "--image",
            "static",
            "--tag",
            "example/docs:test",
            "--dry-run",
        ])
        .output()
        .unwrap();
    assert!(result.status.success(), "{result:?}");
    assert!(!planned.exists());
    assert!(report(&result).contains("content/assets/imported/public/logo.svg"));
}

#[test]
fn dynamic_configs_and_unsupported_routing_fail_without_source_execution_or_writes() {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("source");
    let out = temp.path().join("out");
    write(&source, "index.md", "# Home\n");
    for raw in [
        "import fs from 'node:fs'; fs.writeFileSync('executed','bad'); export default {}",
        "export default {title: process.env.TITLE}",
        "export default () => ({})",
        "export default {...config}",
        "export default {rewrites:{'index.md':'new.md'}}",
        "export default {srcExclude:['private/**']}",
        "export default {locales:{fr:{label:'French'}}}",
        "export default {vite:{publicDir:'other'}}",
        "export default {extends:{}}",
        "export default {outDir:'custom'}",
        "export default {cacheDir:'custom'}",
        "export default {cleanUrls:'true'}",
        "export default {base:'relative'}",
        "export default {base:'/no-trailing-slash'}",
        "export default {themeConfig:{sidebar:'auto'}}",
        "export default {themeConfig:[]}",
    ] {
        write(&source, ".vitepress/config.ts", raw);
        run(&source, &out, &["--dry-run"], 2);
        assert!(!out.exists());
        assert!(!source.join("executed").exists());
        assert!(!temp.path().join("executed").exists());
        assert_eq!(read(&source, ".vitepress/config.ts"), raw);
    }
}

#[test]
fn invalid_paths_missing_pages_and_output_collisions_fail_before_writes() {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("source");
    let out = temp.path().join("out");
    write(&source, "index.md", "# Home\n");
    for settings in [
        "srcDir:'../private'",
        "srcDir:'/private'",
        "srcDir:'missing'",
        "srcDir:''",
        "srcDir:42",
        "themeConfig:{sidebar:[{link:'/missing'}]}",
        "themeConfig:{sidebar:[{link:'/../index'}]}",
        "themeConfig:{sidebar:[{link:'/%2findex'}]}",
        "themeConfig:{sidebar:[{link:'/index%00'}]}",
        "themeConfig:{sidebar:[{link:42}]}",
        "themeConfig:{sidebar:[{items:[{link:'/'}]}]}",
    ] {
        write(
            &source,
            ".vitepress/config.js",
            format!("export default {{{settings}}};"),
        );
        run(&source, &out, &[], 2);
        assert!(!out.exists());
    }
    write(&source, ".vitepress/config.js", "export default {};");
    write(&source, "01-Page.md", "# One\n");
    write(&source, "Page.md", "# Two\n");
    assert!(report(&run(&source, &out, &[], 2)).contains("would render at"));
    assert!(!out.exists());
}

#[test]
fn ambiguous_configs_and_format_detection_require_explicit_resolution() {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("source");
    let out = temp.path().join("out");
    write(&source, ".vitepress/config.js", "export default {};");
    write(&source, ".vitepress/config.ts", "export default {};");
    write(&source, "index.md", "# Home\n");
    assert!(report(&run(&source, &out, &[], 2)).contains("exactly one"));
    fs::remove_file(source.join(".vitepress/config.ts")).unwrap();
    write(&source, "mkdocs.yml", "site_name: Other\n");
    assert!(report(&run(&source, &out, &[], 2)).contains("ambiguous source (mkdocs, vitepress)"));
    assert!(!out.exists());
    run(&source, &out, &["--from", "vitepress"], 0);
    assert!(out.join("01-guide/index.mdx").is_file());
}

#[test]
fn public_asset_collisions_and_traversal_are_rejected_even_when_copying_is_skipped() {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("source");
    let out = temp.path().join("out");
    write(&source, ".vitepress/config.js", "export default {};");
    write(&source, "logo.svg", "source");
    write(&source, "public/logo.svg", "public");
    write(&source, "index.md", "# Home\n\n![Logo](/logo.svg)\n");
    assert!(
        report(&run(&source, &out, &["--assets", "skip"], 2))
            .contains("both the source and public")
    );
    for path in [
        "../private.svg",
        "/../../private.svg",
        "/.git/config",
        "/bad%5cpath",
        "/bad%FF",
    ] {
        write(
            &source,
            "index.md",
            format!("# Home\n\n![Unsafe]({path})\n"),
        );
        run(&source, &out, &["--assets", "skip"], 2);
        assert!(!out.exists());
    }
}

#[test]
fn unsupported_page_features_are_reported_and_dynamic_route_files_are_rejected() {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("source");
    let out = temp.path().join("out");
    write(&source, ".vitepress/config.js", "export default {};");
    write(
        &source,
        "index.md",
        "---\nlayout: false\nhero: {name: Original hero}\n---\n# Home\n\n::: tip\nOriginal text\n:::\n",
    );
    let result = run(&source, &out, &[], 0);
    assert!(report(&result).contains("extensions are retained"));
    assert!(read(&out, "01-guide/index.mdx").contains("Original hero"));
    assert!(read(&out, "01-guide/index.mdx").contains("Original text"));
    write(&source, "[page].md", "# Dynamic\n");
    let out2 = temp.path().join("dynamic");
    assert!(report(&run(&source, &out2, &[], 2)).contains("dynamic route parameters"));
    assert!(!out2.exists());
}

#[cfg(unix)]
#[test]
fn linked_config_source_directories_and_public_assets_are_never_followed() {
    use std::os::unix::fs::symlink;
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("source");
    let out = temp.path().join("out");
    write(temp.path(), "outside/config.js", "export default {};");
    write(&source, "index.md", "# Home\n");
    symlink(temp.path().join("outside"), source.join(".vitepress")).unwrap();
    run(&source, &out, &[], 2);
    fs::remove_file(source.join(".vitepress")).unwrap();
    write(
        &source,
        ".vitepress/config.js",
        "export default {srcDir:'linked'};",
    );
    symlink(temp.path().join("outside"), source.join("linked")).unwrap();
    run(&source, &out, &[], 2);
    fs::remove_file(source.join("linked")).unwrap();
    write(&source, ".vitepress/config.js", "export default {};");
    symlink(temp.path().join("outside"), source.join("public")).unwrap();
    write(&source, "index.md", "# Home\n\n[Asset](/private.txt)\n");
    run(&source, &out, &["--assets", "skip"], 2);
    assert!(!out.exists());
}
