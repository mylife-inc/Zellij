use camino::Utf8Path;
use serde_json::Value;
use std::fs;

#[test]
fn adapter_preserves_pages_settings_literal_text_and_references() {
    let tmp = tempfile::tempdir().unwrap();
    let root = Utf8Path::from_path(tmp.path()).unwrap();
    let source = root.join("manual");
    fs::create_dir_all(source.join("content/nested")).unwrap();
    fs::write(source.join("zel.yaml"), "version: 1\nrenderer: portable\nname: Acme\ntheme: forest\nbrand:\n  logo: {src: 'logo #1%.svg', alt: Acme, href: 'https://example.com'}\nnav:\n  - {page: index.md, label: Start}\n  - label: Learn\n    children:\n      - {href: 'https://example.com/help', label: Help}\n      - {page: nested/readme.md, label: Guide}\n").unwrap();
    fs::write(source.join("content/index.md"), "---\ntitle: Welcome\ndescription: Description\n---\n# Authored {#exact}\n\n[Guide](nested/#there) [File](report.pdf?dl=1) ![Mark](logo%20%231%25.svg)\n\n<a>{process.exit(9)}</a>\n\nexport const value = 1\n\n| A | B |\n| :- | -: |\n| x | y |\n\n- [x] Done\n\nNote[^ref].\n\n[^ref]: A footnote.\n\n```mermaid\ngraph TD; A-->B\n```\n\n$x$ stays literal.\n").unwrap();
    fs::write(
        source.join("content/nested/readme.md"),
        "# Details\n\n## There {#there}\n\n[Home](../index.md#exact)\n",
    )
    .unwrap();
    fs::write(source.join("content/unlisted.markdown"), "# Kept\n").unwrap();
    fs::write(source.join("content/logo #1%.svg"), "<svg/>").unwrap();
    fs::write(source.join("content/report.pdf"), b"%PDF-1.4\0bytes").unwrap();
    fs::write(source.join("content/unused.txt"), "private").unwrap();
    let out = root.join("adapted");
    let report = zellij_docs::adapt_hosted(&source, &out).unwrap();
    assert_eq!(report.pages, 3);
    let read = |file: &str| -> Value {
        serde_json::from_slice(&fs::read(out.join(file)).unwrap()).unwrap()
    };
    let site = read("site.yaml");
    assert_eq!(site["theme"], "forest");
    assert_eq!(site["brand"]["logo"]["src"], "assets/portable-brand.svg");
    let sidebar = read("manual/_sidebar.yaml");
    assert_eq!(sidebar["nav"][0]["label"], "Start");
    assert_eq!(
        sidebar["nav"][1]["items"][0]["href"],
        "https://example.com/help"
    );
    let page = read("manual/p0.zeldoc.json");
    assert_eq!(page["route"], "/");
    assert_eq!(page["title"], "Welcome");
    assert_eq!(page["headings"][0]["id"], "exact");
    let body = page["body"].to_string();
    for expected in [
        "/nested/index.html#there",
        "/_zellij/assets/portable/report.pdf?dl=1",
        "process.exit(9)",
        "export const value = 1",
        "language-text",
        "$x$ stays literal",
        "checkbox",
        "footnote-definition",
        "tbody",
    ] {
        assert!(body.contains(expected), "missing {expected}: {body}");
    }
    assert!(!body.contains("language-mermaid"));
    assert_eq!(
        fs::read(out.join("assets/portable/report.pdf")).unwrap(),
        b"%PDF-1.4\0bytes"
    );
    assert!(!out.join("assets/portable/unused.txt").exists());
    assert_eq!(read("manual/p2.zeldoc.json")["title"], "Kept");
    assert!(source.join("content/index.md").is_file());
    assert!(!source.join("site").exists());
    // A broken fragment must fail before creating the destination.
    fs::write(
        source.join("content/unlisted.markdown"),
        "[broken](index.md#missing)",
    )
    .unwrap();
    let bad = root.join("bad");
    assert!(zellij_docs::adapt_hosted(&source, &bad).is_err());
    assert!(!bad.exists());
}

#[test]
fn adapter_generates_home_and_retains_enabled_math_and_diagrams() {
    let tmp = tempfile::tempdir().unwrap();
    let root = Utf8Path::from_path(tmp.path()).unwrap();
    let source = root.join("source");
    fs::create_dir(&source).unwrap();
    fs::write(
        source.join("zel.yaml"),
        "name: Features\nmarkdown: {math: true, mermaid: true}\n",
    )
    .unwrap();
    fs::write(source.join("page.md"), "## Maths {#math}\n\n$x^2$\n\n```mermaid\ngraph TD; A-->B\n```\n\n```mindmap\nRoot: [A, B]\n```\n").unwrap();
    let out = root.join("output");
    let report = zellij_docs::adapt_hosted(&source, &out).unwrap();
    assert_eq!(report.pages, 2);
    assert_eq!(report.features, ["maths", "diagrams"]);
    let page = fs::read_to_string(out.join("manual/p1.zeldoc.json")).unwrap();
    for value in [
        "language-math",
        "math-inline",
        "language-mermaid",
        "language-mindmap",
        "_zel-title",
    ] {
        assert!(page.contains(value), "{value}");
    }
    let home = fs::read_to_string(out.join("manual/p0.zeldoc.json")).unwrap();
    assert!(home.contains("/page.html"));
}
