use camino::{Utf8Path, Utf8PathBuf};
use std::fs;
use zellij_docs::{BuildOptions, build};

fn fixture() -> (tempfile::TempDir, Utf8PathBuf, Utf8PathBuf) {
    let temp = tempfile::tempdir().unwrap();
    let root = Utf8PathBuf::from_path_buf(temp.path().to_owned()).unwrap();
    let source = root.join("Markdown 日本");
    fs::create_dir(&source).unwrap();
    let out = root.join("Published site");
    (temp, source, out)
}
fn write(root: &Utf8Path, file: &str, body: impl AsRef<[u8]>) {
    let path = root.join(file);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, body).unwrap();
}

#[test]
fn complete_markdown_site_has_relative_links_assets_anchors_and_embedded_search() {
    let (_temp, source, out) = fixture();
    write(
        &source,
        "README.md",
        "---\ntitle: 'A <safe> title'\n---\n# Welcome\n\n[Install](guide/Install%20Now.md?x=1#setup-1)\n\n![Logo](assets/logo.svg)\n\n[Report](assets/report.pdf)\n\n```rust\nfn main() { println!(\"hi\"); }\n```\n",
    );
    write(
        &source,
        "guide/Install Now.md",
        "# Install\n\n## Setup\n\nText.\n\n## Setup\n\n[Home](../README.md#welcome)\n\n[Custom](#custom:anchor)\n\n## Other {#custom:anchor}\n",
    );
    write(
        &source,
        "assets/logo.svg",
        "<svg xmlns=\"http://www.w3.org/2000/svg\"/>",
    );
    write(&source, "assets/report.pdf", [0, 255, 254]);
    let report = build(&source, &out, BuildOptions::default()).unwrap();
    assert_eq!(report.pages, 2);
    assert!(report.warnings.is_empty());
    let home = fs::read_to_string(out.join("index.html")).unwrap();
    assert!(home.contains("A &lt;safe&gt; title"));
    assert!(home.contains("guide/Install%20Now.html?x=1#setup-1"));
    assert!(
        home.contains("<span style="),
        "syntax highlighting is produced at build time"
    );
    assert!(!home.contains("/_next/"));
    let child = fs::read_to_string(out.join("guide/Install Now.html")).unwrap();
    assert!(child.contains("../index.html#welcome"));
    assert!(child.contains("id=\"setup-1\""));
    assert!(child.contains("../_zellij/site.js"));
    assert_eq!(
        fs::read(out.join("assets/report.pdf")).unwrap(),
        [0, 255, 254]
    );
    assert!(
        fs::read_to_string(out.join("_zellij/search.js"))
            .unwrap()
            .contains("window.ZELLIJ_SEARCH=")
    );
    assert!(out.join("_zellij/fonts/licenses.txt").is_file());
    assert_eq!(
        fs::read_dir(out.join("_zellij/fonts/files"))
            .unwrap()
            .count(),
        75
    );
    assert!(out.join(".zellij-output").is_file());
    assert!(!source.join("site.yaml").exists());
}

#[test]
fn directories_readmes_and_generated_home_resolve_without_redirects() {
    let (_temp, source, out) = fixture();
    write(&source, "guide/README.md", "# Start\n\n[Next](next.md)\n");
    write(
        &source,
        "guide/next.md",
        "# Next\n\n[Folder](./)\n\n[Root](/)\n",
    );
    let report = build(&source, &out, BuildOptions::default()).unwrap();
    assert_eq!(report.pages, 3);
    let home = fs::read_to_string(out.join("index.html")).unwrap();
    assert!(home.contains("guide/index.html"));
    let next = fs::read_to_string(out.join("guide/next.html")).unwrap();
    assert!(next.contains("../guide/index.html"));
    assert!(next.contains("../index.html"));
    assert!(!next.contains("http-equiv=\"refresh\""));
}

#[test]
fn invalid_references_preserve_the_previous_export_and_all_source_bytes() {
    let (_temp, source, out) = fixture();
    write(&source, "index.md", "# Original\n");
    build(&source, &out, BuildOptions::default()).unwrap();
    let original = fs::read(out.join("index.html")).unwrap();
    for body in [
        "[Broken](absent.md)",
        "[Bad anchor](index.md#missing)",
        "[Escape](../secret.pdf)",
        "[Encoded escape](%2e%2e/secret.pdf)",
        "[Script](javascript:alert%281%29)",
        "[Bad percent](bad%ZZ.md)",
        "## Repeated {#x}\n\n## Again {#x}",
        "---\ntitle: [bad]\n---\nText",
        "---\ntitle: No close",
    ] {
        write(&source, "index.md", body);
        assert!(
            build(&source, &out, BuildOptions::default()).is_err(),
            "{body}"
        );
        assert_eq!(fs::read(out.join("index.html")).unwrap(), original);
        assert_eq!(fs::read_to_string(source.join("index.md")).unwrap(), body);
    }
    write(&source, "index.md", "# Replacement\n");
    write(&out, "obsolete.html", "old generated page");
    build(&source, &out, BuildOptions::default()).unwrap();
    assert!(!out.join("obsolete.html").exists());
    assert!(
        fs::read_to_string(out.join("index.html"))
            .unwrap()
            .contains("Replacement")
    );
}

#[test]
fn output_ownership_collisions_and_overlap_are_checked_before_publication() {
    let (_temp, source, out) = fixture();
    write(&source, "index.md", "# Site\n");
    write(&out, "authored.txt", "keep");
    assert!(build(&source, &out, BuildOptions::default()).is_err());
    assert_eq!(
        fs::read_to_string(out.join("authored.txt")).unwrap(),
        "keep"
    );
    for target in [
        source.clone(),
        source.join("output"),
        source.parent().unwrap().to_owned(),
    ] {
        assert!(build(&source, &target, BuildOptions::default()).is_err());
    }
    fs::remove_dir_all(&out).unwrap();
    write(&source, "index.markdown", "# Conflict\n");
    assert!(build(&source, &out, BuildOptions::default()).is_err());
    assert!(!out.exists());
    fs::remove_file(source.join("index.markdown")).unwrap();
    write(&source, "index.md", "# Page\n\n[HTML](index.html)\n");
    write(&source, "index.html", "authored asset");
    assert!(build(&source, &out, BuildOptions::default()).is_err());
    assert!(!out.exists());
}

#[test]
fn raw_html_is_literal_reported_and_cannot_execute_or_break_search_script() {
    let (_temp, source, out) = fixture();
    write(
        &source,
        "index.md",
        "# Title\n\n<script>alert('no')</script>\n\n<img src=\"https://example.com/tracker\">\n",
    );
    let report = build(&source, &out, BuildOptions::default()).unwrap();
    assert_eq!(report.warnings.len(), 1);
    let html = fs::read_to_string(out.join("index.html")).unwrap();
    assert!(!html.contains("<script>alert"));
    assert!(!html.contains("<img src=\"https:"));
    assert!(html.contains("&lt;script&gt;"));
    let search = fs::read_to_string(out.join("_zellij/search.js")).unwrap();
    assert!(!search.contains("</script>"));
}

#[test]
fn themes_metadata_order_and_frontmatter_are_checked() {
    let (_temp, source, out) = fixture();
    write(
        &source,
        "index.md",
        "\u{feff}---\r\ntitle: Home\r\ndescription: 'Quoted \"summary\"'\r\n---\r\n# Home\r\n",
    );
    write(&source, "a.md", "# A\n");
    write(&source, "z.md", "---\norder: -1\n---\n# Z\n");
    build(
        &source,
        &out,
        BuildOptions {
            name: Some("Company & Docs"),
            theme: Some("forest"),
        },
    )
    .unwrap();
    let html = fs::read_to_string(out.join("index.html")).unwrap();
    assert!(html.contains("data-zellij-theme=\"forest\""));
    assert!(html.contains("Company &amp; Docs"));
    assert!(html.contains("Quoted &quot;summary&quot;"));
    assert!(html.contains("rel=\"next\" href=\"z.html\""));
    assert!(
        build(
            &source,
            &out,
            BuildOptions {
                name: None,
                theme: Some("missing")
            }
        )
        .is_err()
    );
    write(&source, "widget.mdx", "<Callout>MDX</Callout>");
    assert!(build(&source, &out, BuildOptions::default()).is_err());
}

#[cfg(unix)]
#[test]
fn symlinked_documents_and_assets_are_rejected() {
    use std::os::unix::fs::symlink;
    let (temp, source, out) = fixture();
    fs::write(temp.path().join("secret"), "private").unwrap();
    write(&source, "index.md", "# Docs\n\n[Asset](linked.pdf)");
    symlink(temp.path().join("secret"), source.join("linked.pdf")).unwrap();
    assert!(build(&source, &out, BuildOptions::default()).is_err());
    assert!(!out.exists());
    fs::remove_file(source.join("linked.pdf")).unwrap();
    symlink(temp.path().join("secret"), source.join("linked.md")).unwrap();
    assert!(build(&source, &out, BuildOptions::default()).is_err());
}

#[test]
fn markdown_extensions_footnotes_and_external_urls_keep_their_meaning() {
    let (_temp, source, out) = fixture();
    write(
        &source,
        "index.md",
        "# Main\n\n[Reference][next]\n\n[next]: sub/next.markdown?return=https://example.com#custom\n\n[Remote](https://example.com/path?q=a:b) and [Email](mailto:docs@example.com).\n\nNote[^note].\n\n[^note]: A footnote.\n\n## Note\n\n```md\n[Not a link](missing.md)\n```\n",
    );
    write(
        &source,
        "sub/next.markdown",
        "# Next\n\n## Custom {#custom}\n\n[Footnote](../index.md#note)\n",
    );
    build(&source, &out, BuildOptions::default()).unwrap();
    let html = fs::read_to_string(out.join("index.html")).unwrap();
    assert!(html.contains("sub/next.html?return=https://example.com#custom"));
    assert!(html.contains("href=\"mailto:docs@example.com\""));
    assert!(html.contains("id=\"note-1\""));
    assert!(html.contains("id=\"note\""));
    assert!(html.contains("missing.md"));
}

#[cfg(unix)]
#[test]
fn invalid_windows_names_and_reserved_asset_names_fail_before_writes() {
    let (_temp, source, out) = fixture();
    write(&source, "index.md", "# Docs\n");
    for name in [
        "CON.md",
        "COM¹.md",
        "a?.md",
        "folder /page.md",
        "_zellij/page.md",
    ] {
        write(&source, name, "# Invalid portable name\n");
        assert!(
            build(&source, &out, BuildOptions::default()).is_err(),
            "{name}"
        );
        assert!(!out.exists());
        fs::remove_file(source.join(name)).unwrap();
    }
    write(
        &source,
        "index.md",
        "# Docs\n\n[Reserved](_ZELLIJ/custom.txt)\n",
    );
    write(&source, "_ZELLIJ/custom.txt", "authored");
    assert!(build(&source, &out, BuildOptions::default()).is_err());
    assert!(!out.exists());
}

#[cfg(target_os = "linux")]
#[test]
fn case_sensitive_sources_cannot_create_case_collisions_on_other_platforms() {
    let (_temp, source, out) = fixture();
    write(&source, "A/one.md", "# One\n");
    write(&source, "a/two.md", "# Two\n");
    assert!(build(&source, &out, BuildOptions::default()).is_err());
    assert!(!out.exists());
}

#[test]
fn references_cannot_publish_files_excluded_from_discovery_and_watching() {
    let (_temp, source, out) = fixture();
    write(&source, "README.md", "# Previous site\n");
    build(&source, &out, BuildOptions::default()).unwrap();
    let previous = fs::read(out.join("index.html")).unwrap();
    for path in [
        ".private/logo.svg",
        "assets/node_modules/logo.svg",
        "assets/.logo.svg",
    ] {
        write(&source, path, "<svg xmlns=\"http://www.w3.org/2000/svg\"/>");
        for reference in [format!("![Logo]({path})"), format!("[Download]({path})")] {
            write(
                &source,
                "README.md",
                format!("# Invalid reference\n\n{reference}\n"),
            );
            let error = build(&source, &out, BuildOptions::default())
                .err()
                .expect("excluded asset must fail");
            assert!(error.to_string().contains("hidden paths or node_modules"));
            assert_eq!(fs::read(out.join("index.html")).unwrap(), previous);
            assert!(!out.join(path).exists());
            assert!(source.join(path).is_file());
        }
    }
}
