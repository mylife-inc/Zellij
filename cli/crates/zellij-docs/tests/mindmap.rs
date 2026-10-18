use camino::{Utf8Path, Utf8PathBuf};
use std::fs;
use zellij_docs::{BuildOptions, Code, build, check};

fn fixture() -> (tempfile::TempDir, Utf8PathBuf, Utf8PathBuf) {
    let temp = tempfile::tempdir().unwrap();
    let root = Utf8Path::from_path(temp.path()).unwrap();
    let source = root.join("Maps 日本");
    fs::create_dir_all(source.join("guide")).unwrap();
    let out = root.join("site");
    (temp, source, out)
}

fn document(map: &str) -> String {
    format!("# Maps\n\n```mindmap\n{map}\n```\n")
}

#[test]
fn native_mindmaps_keep_the_hierarchy_search_text_and_unique_accessible_names() {
    let (_temp, source, out) = fixture();
    let map = "Zellij:\n  Content: [site.yaml, menu.yaml]\n  Output:\n    HTML: null\n    Versions: [1, true]\n  Leaf:\n";
    fs::write(source.join("index.md"), "# Home\n\n[Maps](guide/maps.md)\n").unwrap();
    fs::write(
        source.join("guide/maps.md"),
        format!("{}\n```mindmap\nSolo:\n```\n", document(map)),
    )
    .unwrap();
    let report = check(&source, true);
    assert!(report.ok, "{:?}", report.diagnostics);
    assert!(!out.exists());
    build(&source, &out, BuildOptions::default()).unwrap();
    let html = fs::read_to_string(out.join("guide/maps.html")).unwrap();
    assert_eq!(html.matches("class=\"mindmap-svg\"").count(), 2);
    assert!(html.contains("aria-labelledby=\"_zel-mindmap-1-title\""));
    assert!(html.contains("aria-labelledby=\"_zel-mindmap-2-title\""));
    assert!(html.contains("Mindmap: Zellij"));
    assert!(html.contains("<li>Content<ul><li>site.yaml</li><li>menu.yaml</li></ul></li>"));
    assert!(html.contains("<li>Versions<ul><li>1</li><li>true</li></ul></li>"));
    assert!(html.contains("<li>Leaf</li>"));
    assert!(html.contains("../_zellij/mindmap.css"));
    assert!(
        fs::read_to_string(out.join("_zellij/search.js"))
            .unwrap()
            .contains("site.yaml")
    );
    assert!(!html.contains("data-language=\"mindmap\""));
    assert_eq!(html.matches("<script ").count(), 2, "No diagram runtime");
}

#[test]
fn labels_are_plain_text_and_cannot_inject_svg_or_html() {
    let (_temp, source, out) = fixture();
    fs::write(
        source.join("index.md"),
        document("'日本 café 👩‍💻 & <script>alert(1)</script>':\n  - '</text><image href=\"https://example.com/tracker\" onload=\"alert(1)\"/>'\n  - 'javascript:alert(1)'\n  - '<style>body{display:none}</style>'"),
    ).unwrap();
    assert!(check(&source, true).ok);
    build(&source, &out, BuildOptions::default()).unwrap();
    let html = fs::read_to_string(out.join("index.html")).unwrap();
    assert!(html.contains("日本 café 👩‍💻 &amp; &lt;script&gt;"));
    assert!(!html.contains("<image"));
    assert!(!html.contains("<script>alert"));
    assert!(!html.contains("<style>body"));
    assert!(!html.contains("href=\"https://example.com/tracker\""));
}

#[test]
fn invalid_mindmaps_name_the_source_and_preserve_the_export() {
    let (_temp, source, out) = fixture();
    fs::write(source.join("index.md"), document("Original: [Good]")).unwrap();
    build(&source, &out, BuildOptions::default()).unwrap();
    let before = fs::read(out.join("index.html")).unwrap();
    for map in [
        "",
        "- no\n- root",
        "A: null\nB: null",
        "Root: [unfinished",
        "Root: {duplicate: 1, duplicate: 2}",
        "Root: !custom tagged",
        "'': null",
        "Root: {' ': null}",
        "Root: {42: value}",
        "Root: [[nested]]",
        "Root: [null]",
        "Root: &loop [*loop]",
        "Root: .nan",
        "Root: null\n---\nSecond: null",
        "Root: {\"\\0\": null}",
    ] {
        let body = document(map);
        fs::write(source.join("index.md"), &body).unwrap();
        let report = check(&source, false);
        assert!(!report.ok, "{map}");
        assert_eq!(report.exit_code, 3);
        assert_eq!(report.diagnostics[0].code, Code::Rendering);
        assert_eq!(report.diagnostics[0].file.as_deref(), Some("index.md"));
        assert!(report.diagnostics[0].message.contains("mindmap 1"));
        assert!(build(&source, &out, BuildOptions::default()).is_err());
        assert_eq!(fs::read(out.join("index.html")).unwrap(), before);
        assert_eq!(fs::read_to_string(source.join("index.md")).unwrap(), body);
    }
}

#[test]
fn mindmaps_bound_source_nodes_depth_and_labels() {
    let (_temp, source, _out) = fixture();
    let cases = [
        (format!("Root: #{}", "x".repeat(65_536)), "64 KiB"),
        (
            format!("Root: [{}]", vec!["leaf"; 512].join(",")),
            "512 nodes",
        ),
        (
            (0..34)
                .map(|i| format!("{}level{i}:\n", "  ".repeat(i)))
                .collect(),
            "32 levels",
        ),
        (format!("Root: '{}'", "x".repeat(257)), "256 characters"),
        (
            "Root:\n  A: &a [leaf, leaf, leaf, leaf]\n  B: &b [*a, *a, *a, *a]\n  C: &c [*b, *b, *b, *b]\n  D: &d [*c, *c, *c, *c]\n  E: &e [*d, *d, *d, *d]\n  F: [*e, *e]\n".into(),
            "YAML expansion limit",
        ),
    ];
    for (map, expected) in cases {
        fs::write(source.join("index.md"), document(&map)).unwrap();
        let report = check(&source, false);
        assert!(!report.ok);
        assert!(
            report.diagnostics[0].message.contains(expected),
            "{:?}",
            report.diagnostics
        );
    }
    // A wide but supported tree, including repeated scalar values, is valid.
    fs::write(
        source.join("index.md"),
        document(&format!("Root: [{}]", vec!["leaf"; 511].join(","))),
    )
    .unwrap();
    assert!(check(&source, true).ok);
}

#[test]
fn ordinary_fences_stay_literal_and_unused_diagram_styles_are_removed() {
    let (_temp, source, out) = fixture();
    fs::write(source.join("index.md"), document("Root: [Child]")).unwrap();
    build(&source, &out, BuildOptions::default()).unwrap();
    assert!(out.join("_zellij/mindmap.css").is_file());
    fs::write(source.join("index.md"), "# Code\n\n```mermaid\nflowchart LR\n A --> B\n```\n\n```yaml\nRoot: [Child]\n```\n\n````markdown\n```mindmap\nA: 1\nB: 2\n```\n````\n").unwrap();
    build(&source, &out, BuildOptions::default()).unwrap();
    let html = fs::read_to_string(out.join("index.html")).unwrap();
    assert!(!html.contains("mindmap-svg"));
    assert!(!html.contains("_zellij/mindmap.css"));
    assert!(!out.join("_zellij/mindmap.css").exists());
    assert!(html.contains("data-language=\"mermaid\""));
    assert!(check(&source, true).ok);
}
