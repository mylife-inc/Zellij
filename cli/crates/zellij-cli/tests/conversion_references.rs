use std::{
    fs,
    path::Path,
    process::{Command, Output},
};

fn write(root: &Path, path: &str, body: impl AsRef<[u8]>) {
    let target = root.join(path);
    fs::create_dir_all(target.parent().unwrap()).unwrap();
    fs::write(target, body).unwrap();
}
fn convert(source: &Path, out: &Path, extra: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_zel"))
        .arg("convert")
        .arg(source)
        .arg("--out")
        .arg(out)
        .args(extra)
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
fn fixture(source: &Path, docusaurus: bool) {
    if docusaurus {
        write(
            source,
            "docusaurus.config.js",
            "module.exports = {title: 'Conversion'};",
        );
        write(
            source,
            "sidebars.js",
            "module.exports = {docs: ['index', '02-Setup/APIReference', 'nested/index']};",
        );
    } else {
        write(
            source,
            "mkdocs.yml",
            "site_name: Conversion\nnav:\n  - index.md\n  - 02-Setup/APIReference.md\n  - nested/index.md\n",
        );
    }
    write(
        source,
        if docusaurus {
            "docs/02-Setup/APIReference.mdx"
        } else {
            "docs/02-Setup/APIReference.md"
        },
        "# Reference\n\n## Install\n\n[Home](../index.md)\n\n![Nested](../media/logo.svg)\n",
    );
    write(source, "docs/nested/index.md", "# Nested\n");
    write(
        source,
        "docs/media/logo.svg",
        b"<svg xmlns=\"http://www.w3.org/2000/svg\"/>\n",
    );
    write(
        source,
        "docs/media/Q3 Report (final).pdf",
        [0x25, 0x50, 0x44, 0x46, 0, 255, 254],
    );
    write(
        source,
        "docs/index.md",
        "# Welcome\n\n[Install **now**](02-Setup/APIReference.md?view=all#install \"Go\")\n\n[Folder](nested/)\n\n[Report](<media/Q3 Report (final).pdf>)\n\n[Reference][guide]\n\n![Logo][logo]\n\n[![Logo](media/logo.svg)](02-Setup/APIReference.md)\n\n[`a]b`](02-Setup/APIReference.md)\n\n[External](https://example.com/a.md?q=1#section)\n\n[guide]: 02-Setup/APIReference.md#install \"Reference title\"\n[logo]: <media/logo.svg>\n\n`[code](../../private.md)`\n\n```md\n[example](../../private.md)\n![example](../../secret.png)\n```\n",
    );
}

#[test]
fn both_converters_rewrite_document_and_reference_links_and_copy_binary_assets() {
    for docusaurus in [false, true] {
        let temp = tempfile::tempdir().unwrap();
        let source = temp.path().join("source");
        let out = temp.path().join("out");
        fixture(&source, docusaurus);
        success(convert(&source, &out, &[]));
        let body = fs::read_to_string(out.join("01-guide/index.mdx")).unwrap();
        for expected in [
            "[Install **now**](</guide/setup/api-reference?view=all#install> \"Go\")",
            "[Folder](</guide/nested>)",
            "[Report](</_zellij/assets/imported/docs/media/Q3%20Report%20%28final%29.pdf>)",
            "[Reference](</guide/setup/api-reference#install> \"Reference title\")",
            "![Logo](</_zellij/assets/imported/docs/media/logo.svg>)",
            "[![Logo](</_zellij/assets/imported/docs/media/logo.svg>)](</guide/setup/api-reference>)",
            "[`a]b`](</guide/setup/api-reference>)",
            "[External](https://example.com/a.md?q=1#section)",
            "`[code](../../private.md)`",
            "[example](../../private.md)",
        ] {
            assert!(body.contains(expected), "missing {expected:?}:\n{body}");
        }
        assert!(!body.contains("[guide]:"));
        assert_eq!(
            fs::read(out.join("assets/imported/docs/media/Q3 Report (final).pdf")).unwrap(),
            [0x25, 0x50, 0x44, 0x46, 0, 255, 254]
        );
        let nested = fs::read_to_string(out.join("01-guide/02-Setup/APIReference.mdx")).unwrap();
        assert!(nested.contains("[Home](</guide>)"), "{nested}");
    }
}

#[test]
fn docusaurus_static_assets_and_mdx_source_links_are_supported() {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("source");
    let out = temp.path().join("out");
    fixture(&source, true);
    write(&source, "static/img/logo.svg", "static bytes");
    write(
        &source,
        "docs/index.md",
        "# Home\n\n![A](/img/logo.svg)\n![B](@site/static/img/logo.svg)\n[Page](02-Setup/APIReference.mdx)\n",
    );
    success(convert(&source, &out, &[]));
    let body = fs::read_to_string(out.join("01-guide/index.mdx")).unwrap();
    assert_eq!(
        body.matches("/_zellij/assets/imported/static/img/logo.svg")
            .count(),
        2
    );
    assert!(body.contains("[Page](</guide/setup/api-reference>)"));
    assert_eq!(
        fs::read(out.join("assets/imported/static/img/logo.svg")).unwrap(),
        b"static bytes"
    );
}

#[test]
fn dry_run_reports_assets_without_creating_destination() {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("source");
    let out = temp.path().join("out");
    fixture(&source, false);
    let report = success(convert(&source, &out, &["--dry-run"]));
    assert!(report.contains("assets/imported/docs/media/logo.svg"));
    assert!(!out.exists());
}

#[test]
fn missing_unlisted_and_html_references_are_reported_without_copying_orphans() {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("source");
    let out = temp.path().join("out");
    fixture(&source, false);
    write(&source, "docs/hidden.md", "# Draft");
    write(&source, "docs/secret.MD", "# Draft");
    write(
        &source,
        "docs/index.md",
        "# Home\n\n[Missing](missing.md)\n[Hidden](hidden.md)\n[Secret](secret.MD)\n![Missing](media/no.png)\n[Root](/old/docs)\n\n<img src=\"media/logo.svg\" />\n",
    );
    let report = success(convert(&source, &out, &[]));
    for expected in [
        "missing.md",
        "hidden.md",
        "media/no.png",
        "root-relative",
        "HTML/JSX",
    ] {
        assert!(report.contains(expected), "{report}");
    }
    assert!(!out.join("assets/imported/docs/hidden.md").exists());
    assert!(!out.join("assets/imported/docs/secret.MD").exists());
    assert!(!out.join("01-guide/hidden.mdx").exists());
}

#[test]
fn asset_traversal_and_git_metadata_are_rejected_before_any_output() {
    for url in [
        "../../private.png",
        "%2e%2e/%2e%2e/private.png",
        "media/%2e%2e/%2e%2e/private.png",
        "media%5csecret.png",
        ".git/config",
    ] {
        let temp = tempfile::tempdir().unwrap();
        let source = temp.path().join("source");
        let out = temp.path().join("out");
        fixture(&source, false);
        write(
            &source,
            "docs/index.md",
            format!("# Home\n\n![Escape]({url})\n"),
        );
        let output = convert(&source, &out, &[]);
        assert!(!output.status.success(), "accepted {url}");
        assert!(!out.exists());
    }
}

#[test]
fn duplicate_public_routes_and_case_only_assets_fail_before_writing() {
    for collision in [false, true] {
        let temp = tempfile::tempdir().unwrap();
        let source = temp.path().join("source");
        let out = temp.path().join("out");
        fixture(&source, false);
        if collision {
            write(
                &source,
                "mkdocs.yml",
                "site_name: Test\nnav:\n  - 01-intro.md\n  - intro.md\n",
            );
            write(&source, "docs/01-intro.md", "# One\n");
            write(&source, "docs/intro.md", "# Two\n");
        } else {
            // Both URL spellings must be rejected even on a case-insensitive source FS.
            write(&source, "docs/a.svg", "one");
            write(
                &source,
                "docs/index.md",
                "# Home\n\n![a](a.svg)\n![b](A.svg)\n",
            );
            write(&source, "docs/A.svg", "two");
        }
        assert!(!convert(&source, &out, &[]).status.success());
        assert!(!out.exists());
    }
}

#[cfg(unix)]
#[test]
fn symlinked_assets_are_refused_before_any_output() {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("source");
    let out = temp.path().join("out");
    fixture(&source, false);
    write(temp.path(), "private.png", "private");
    std::os::unix::fs::symlink(
        temp.path().join("private.png"),
        source.join("docs/media/linked.png"),
    )
    .unwrap();
    write(
        &source,
        "docs/index.md",
        "# Home\n\n![Link](media/linked.png)\n",
    );
    assert!(!convert(&source, &out, &[]).status.success());
    assert!(!out.exists());
}

#[test]
fn shortcut_collapsed_escaped_and_multiline_reference_labels_survive() {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("source");
    let out = temp.path().join("out");
    fixture(&source, false);
    write(
        &source,
        "docs/index.md",
        "# Home\n\n[Guide] and [Guide][] and [escaped \\]][Guide].\n\n> [Quoted][Guide]\n\nFootnote[^note].\n\n[^note]: [Inside](02-Setup/APIReference.md)\n\n[Guide]:\n  02-Setup/APIReference.md\n  \"Title\"\n",
    );
    success(convert(&source, &out, &[]));
    let body = fs::read_to_string(out.join("01-guide/index.mdx")).unwrap();
    assert_eq!(
        body.matches("</guide/setup/api-reference>").count(),
        5,
        "{body}"
    );
    assert!(
        body.contains("[escaped \\]](</guide/setup/api-reference> \"Title\")"),
        "{body}"
    );
    assert!(
        body.contains("[^note]: [Inside](</guide/setup/api-reference>)"),
        "{body}"
    );
}

#[test]
fn engine_hidden_documents_are_rejected_instead_of_creating_broken_routes() {
    for page in ["_private.md", ".hidden/page.md"] {
        let temp = tempfile::tempdir().unwrap();
        let source = temp.path().join("source");
        let out = temp.path().join("out");
        write(&source, "mkdocs.yml", format!("nav:\n  - {page}\n"));
        write(&source, &format!("docs/{page}"), "# Hidden\n");
        assert!(!convert(&source, &out, &[]).status.success());
        assert!(!out.exists());
    }
}

#[test]
fn relative_directory_links_can_return_to_the_documentation_root() {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("source");
    let out = temp.path().join("out");
    fixture(&source, false);
    write(
        &source,
        "docs/02-Setup/APIReference.md",
        "# Reference\n\n[Up](../)\n[Root](.././)\n",
    );
    write(&source, "docs/index.md", "# Home\n\n[Self](./)\n");
    success(convert(&source, &out, &[]));
    let body = fs::read_to_string(out.join("01-guide/02-Setup/APIReference.mdx")).unwrap();
    assert_eq!(body.matches("</guide>").count(), 2, "{body}");
    assert!(
        fs::read_to_string(out.join("01-guide/index.mdx"))
            .unwrap()
            .contains("[Self](</guide>)")
    );
}
