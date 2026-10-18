use camino::{Utf8Path, Utf8PathBuf};
use std::fs;
use zellij_docs::{BuildOptions, Code, build, check};

fn fixture() -> (tempfile::TempDir, Utf8PathBuf, Utf8PathBuf) {
    let temp = tempfile::tempdir().unwrap();
    let root = Utf8Path::from_path(temp.path()).unwrap();
    let source = root.join("Diagrams 日本");
    fs::create_dir_all(source.join("guide")).unwrap();
    fs::write(source.join("zel.yaml"), "markdown: {mermaid: true}\n").unwrap();
    let out = root.join("site");
    (temp, source, out)
}

fn fence(source: &str) -> String {
    format!("```mermaid\n{source}\n```\n")
}

#[test]
fn mermaid_is_opt_in_with_page_local_scripts_escaped_source_and_search() {
    let (_temp, source, out) = fixture();
    fs::write(source.join("index.md"), "# Home\n[Graphs](guide/graphs.md)").unwrap();
    fs::write(
        source.join("guide/graphs.md"),
        fence("flowchart LR\nA[日本 & <script>alert(1)</script>] --> B")
            + &fence("sequenceDiagram\nAlice->>Bob: Hello"),
    )
    .unwrap();
    let report = check(&source, true);
    assert!(report.ok, "{:?}", report.diagnostics);
    assert!(!out.exists());
    build(&source, &out, BuildOptions::default()).unwrap();
    let graph = fs::read_to_string(out.join("guide/graphs.html")).unwrap();
    assert_eq!(graph.matches("class=\"mermaid-diagram\"").count(), 2);
    assert!(graph.contains("aria-label=\"Mermaid diagram 2\""));
    assert!(graph.contains("日本 &amp; &lt;script&gt;"));
    assert!(!graph.contains("<script>alert"));
    assert_eq!(graph.matches("<details open>").count(), 2);
    assert!(graph.contains("../_zellij/mermaid/mermaid.js"));
    assert!(graph.contains("../_zellij/math/math.css"));
    assert!(
        !fs::read_to_string(out.join("index.html"))
            .unwrap()
            .contains("mermaid.js")
    );
    assert!(
        fs::read_to_string(out.join("_zellij/search.js"))
            .unwrap()
            .contains("Alice")
    );
    assert!(out.join("_zellij/mermaid/licenses.txt").exists());
    assert!(out.join("_zellij/mermaid/manifest.json").exists());

    for config in ["", "markdown: {mermaid: false}"] {
        fs::write(source.join("zel.yaml"), config).unwrap();
        build(&source, &out, BuildOptions::default()).unwrap();
        let graph = fs::read_to_string(out.join("guide/graphs.html")).unwrap();
        assert!(graph.contains("data-language=\"mermaid\""));
        assert!(!graph.contains("mermaid-diagram"));
        assert!(!out.join("_zellij/mermaid").exists());
        assert!(!out.join("_zellij/math").exists());
    }
}

#[test]
fn native_validation_bounds_source_but_leaves_mermaid_grammar_to_the_browser() {
    let (_temp, source, out) = fixture();
    fs::write(source.join("index.md"), fence("not valid Mermaid syntax")).unwrap();
    assert!(check(&source, true).ok);
    build(&source, &out, BuildOptions::default()).unwrap();
    let original = fs::read(out.join("index.html")).unwrap();
    for (body, ordinal, message) in [
        (fence(" "), "1", "must not be empty"),
        (fence(&"x".repeat(65_536)), "1", "64 KiB"),
        (fence("flowchart LR\nA-->B").repeat(33), "33", "32 Mermaid"),
    ] {
        fs::write(source.join("guide/large.md"), &body).unwrap();
        let report = check(&source, false);
        assert!(!report.ok);
        assert_eq!(report.diagnostics[0].code, Code::Rendering);
        assert_eq!(
            report.diagnostics[0].file.as_deref(),
            Some("guide/large.md")
        );
        assert!(
            report.diagnostics[0]
                .message
                .contains(&format!("mermaid {ordinal}"))
        );
        assert!(report.diagnostics[0].message.contains(message));
        assert!(build(&source, &out, BuildOptions::default()).is_err());
        assert_eq!(fs::read(out.join("index.html")).unwrap(), original);
        assert_eq!(
            fs::read_to_string(source.join("guide/large.md")).unwrap(),
            body
        );
    }
    fs::remove_file(source.join("guide/large.md")).unwrap();
    fs::write(source.join("zel.yaml"), "markdown: {mermaid: perhaps}").unwrap();
    assert_eq!(
        check(&source, false).diagnostics[0].code,
        Code::Configuration
    );
}

#[test]
fn removing_the_last_diagram_removes_runtime_but_keeps_enabled_math_assets() {
    let (_temp, source, out) = fixture();
    fs::write(
        source.join("zel.yaml"),
        "markdown: {mermaid: true, math: true}",
    )
    .unwrap();
    fs::write(
        source.join("index.md"),
        fence("flowchart LR\nA-->B") + "\n$x^2$\n",
    )
    .unwrap();
    build(&source, &out, BuildOptions::default()).unwrap();
    fs::write(source.join("index.md"), "# Math only\n$x^2$\n").unwrap();
    build(&source, &out, BuildOptions::default()).unwrap();
    assert!(!out.join("_zellij/mermaid").exists());
    assert!(out.join("_zellij/math/math.css").exists());
}
