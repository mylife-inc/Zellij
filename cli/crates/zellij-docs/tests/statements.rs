use camino::{Utf8Path, Utf8PathBuf};
use serde_json::Value;
use std::fs;
use zellij_core::site::SiteInput;
use zellij_docs::{BuildOptions, adapt_hosted, build, check, pdf};
fn fixture(root: &Utf8Path) -> Utf8PathBuf {
    let input =
        Utf8Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../scripts/fixtures/statements");
    let source = root.join("book");
    for entry in walkdir::WalkDir::new(&input) {
        let entry = entry.unwrap();
        let path = Utf8Path::from_path(entry.path()).unwrap();
        let dest = source.join(path.strip_prefix(&input).unwrap());
        if entry.file_type().is_dir() {
            fs::create_dir_all(dest).unwrap();
        } else {
            fs::copy(path, dest).unwrap();
        }
    }
    source
}
fn json(path: &Utf8Path) -> Value {
    serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
}
fn print(source: &Utf8Path, root: &Utf8Path) -> pdf::Prepared {
    pdf::prepare(
        &SiteInput::from(source),
        &root.join("pdf"),
        BuildOptions::default(),
        false,
    )
    .unwrap()
}
#[test]
fn all_adapters_share_statement_identity_numbers_and_resolved_references() {
    let tmp = tempfile::tempdir().unwrap();
    let root = Utf8Path::from_path(tmp.path()).unwrap();
    let source = fixture(root);
    let report = check(&source, true);
    assert!(report.ok, "{:?}", report.diagnostics);
    let publication = serde_json::to_value(report.book.unwrap()).unwrap();
    let statements = publication["statements"].as_array().unwrap();
    assert_eq!(
        statements
            .iter()
            .map(|s| s["number"].as_str())
            .collect::<Vec<_>>(),
        [
            Some("T.1"),
            Some("F1.1"),
            Some("1.1"),
            Some("1.2"),
            Some("1.3"),
            None,
            Some("1.4"),
            Some("2.1"),
            Some("2.2"),
            Some("A.1")
        ]
    );
    assert_eq!(statements[4]["kind"], "theorem");
    assert_eq!(
        statements[4]["href"],
        "chapters/energy.html#_zel-stmt-thm%3Aenergy"
    );
    assert!(statements[4].get("page").is_none());
    assert!(statements[5].get("number").is_none());
    let out = root.join("site");
    build(&source, &out, BuildOptions::default()).unwrap();
    assert_eq!(json(&out.join("_zellij/book.json")), publication);
    let html = fs::read_to_string(out.join("chapters/energy.html")).unwrap();
    assert!(html.contains("<strong>Theorem 1.3 (Energy &amp; equality)</strong>"));
    assert!(html.contains("href=\"../series.html#_zel-stmt-prop%3Asum\" title=\"Proposition 2.1 (Adding nonnegative values)\">Proposition 2.1</a>"));
    assert!(html.contains("title=\"Theorem 1.3 (Energy &amp; equality)\">1.3</a>"));
    assert_eq!(html.matches("aria-hidden=\"true\"").count(), 1);
    assert!(html.contains("<blockquote>"));
    assert!(html.contains("<ul>"));
    assert!(html.contains("<strong>nonnegative</strong>"));
    assert!(!html.contains("[!THEOREM]"));
    assert!(!html.contains("\\autoref"));
    let adapted = root.join("adapted");
    adapt_hosted(&source, &adapted).unwrap();
    assert_eq!(
        json(&adapted.join("assets/book.json"))["publication"],
        publication
    );
    let doc = fs::read_dir(adapted.join("manual"))
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.to_string_lossy().ends_with(".zeldoc.json"))
        .map(|p| json(Utf8Path::from_path(&p).unwrap()))
        .find(|v| v["title"] == "Energy")
        .unwrap();
    let search = doc["text"].as_str().unwrap();
    assert!(search.contains("Theorem 1.3 (Energy & equality)"));
    assert!(!search.contains("[!THEOREM]"));
    assert!(!search.contains("\\autoref"));
    assert!(!search.contains("\\label"));
    let native_search = fs::read_to_string(out.join("_zellij/search.js")).unwrap();
    assert!(native_search.contains("Theorem 1.3 (Energy"));
    assert!(!native_search.contains("[!THEOREM]"));
    let body = doc["body"].to_string();
    assert!(body.contains("Theorem 1.3 (Energy & equality)"));
    assert!(body.contains("/series.html#_zel-stmt-prop%3Asum"));
    assert!(body.contains("zellij-statement-proof"));
    assert!(!body.contains("[!THEOREM]"));
    let prepared = print(&source, root);
    assert_eq!(prepared.source.revision, publication["revision"]);
    let tex = String::from_utf8(prepared.source.files[Utf8Path::new("book.tex")].clone()).unwrap();
    let after_contents = tex.split_once("\\tableofcontents").unwrap().1;
    assert!(after_contents.trim_start().starts_with("\\clearpage\\markboth{Equations and arguments}{Equations and arguments}\\thispagestyle{plain}"));
    assert!(tex.contains("Theorem 1.3 (Energy \\& equality)"));
    assert!(tex.contains("}{Proposition 2.1}"));
    assert!(tex.contains("}{Equation 2.1}"));
    assert!(tex.contains("\\rule{0.45em}{0.45em}"));
    assert!(!tex.contains("\\autoref"));
    assert!(!tex.contains("[!THEOREM]"));
}
#[test]
fn invalid_statements_fail_checks_and_preserve_existing_output() {
    let tmp = tempfile::tempdir().unwrap();
    let root = Utf8Path::from_path(tmp.path()).unwrap();
    let source = fixture(root);
    let out = root.join("site");
    build(&source, &out, BuildOptions::default()).unwrap();
    let before = fs::read(out.join("_zellij/book.json")).unwrap();
    let html = fs::read(out.join("chapters/energy.html")).unwrap();
    let page = source.join("content/chapters/energy.md");
    let original = fs::read_to_string(&page).unwrap();
    for (extra, diagnostic) in [
        (
            "> [!THEOREM] {#eq:sum}\n>\n> Duplicate equation.",
            "duplicate label",
        ),
        (
            "> [!LEMMA] {#prop:sum}\n>\n> Duplicate statement.",
            "duplicate label",
        ),
        ("> [!PROOF]\n>\n> No label.", "needs a stable"),
        ("> [!THOREM] {#typo}\n>\n> Typo.", "unknown statement kind"),
        ("> [!THEOREM] {#bad label}\n>\n> Bad label.", "labels must"),
        ("> [!THEOREM] {#empty}", "must contain a body"),
        (
            "> [!THEOREM] {#title} **Rich title**\n>\n> Body.",
            "plain-text title",
        ),
        (
            "> [!THEOREM] {#title}\n> Body without blank line.",
            "blank quoted line",
        ),
        (
            "> [!THEOREM] {#outside}\n>\n> > [!LEMMA] {#inside}\n> >\n> > Nested.",
            "top-level",
        ),
        (
            "- A list\n\n  > [!THEOREM] {#listed}\n  >\n  > Nested.",
            "top-level",
        ),
        (
            "> [!THEOREM] {#heading}\n>\n> ## Heading",
            "cannot contain headings",
        ),
        (
            "Note.[^bad]\n\n[^bad]:\n    > [!THEOREM] {#note}\n    >\n    > Nested.",
            "top-level",
        ),
        (r"$\eqref{thm:energy}$", "is a statement"),
        (r"$\ref{proof:energy}$", "unnumbered proof"),
        (r"$\autoref{missing}$", "undefined book label"),
        (
            r"## Heading $\autoref{thm:energy}$",
            "not headings or links",
        ),
        ("[Missing](#_zel-stmt-absent)", "missing heading/footnote"),
    ] {
        fs::write(&page, format!("{original}\n\n{extra}\n")).unwrap();
        let report = check(&source, false);
        assert!(!report.ok, "{extra}");
        assert!(
            report
                .diagnostics
                .iter()
                .any(|d| d.message.contains(diagnostic)),
            "{extra}: {:?}",
            report.diagnostics
        );
        assert!(
            report.diagnostics.iter().any(|d| d.file.is_some()),
            "{:?}",
            report.diagnostics
        );
        assert!(build(&source, &out, BuildOptions::default()).is_err());
        assert!(
            pdf::prepare(
                &SiteInput::from(source.as_path()),
                &root.join("pdf"),
                BuildOptions::default(),
                false
            )
            .is_err()
        );
        assert!(adapt_hosted(&source, &root.join("adapted")).is_err());
        assert_eq!(before, fs::read(out.join("_zellij/book.json")).unwrap());
        assert_eq!(html, fs::read(out.join("chapters/energy.html")).unwrap());
    }
}
#[test]
fn numbering_is_shared_across_kinds_but_separate_from_equations_and_stable_after_reorder() {
    let tmp = tempfile::tempdir().unwrap();
    let root = Utf8Path::from_path(tmp.path()).unwrap();
    let source = fixture(root);
    let before = check(&source, true).book.unwrap();
    let page = source.join("content/chapters/energy.md");
    fs::write(&page,format!("> [!PROOF] {{#new:proof}}\n>\n> Unnumbered.\n\n> [!EXAMPLE] {{#new:example}}\n>\n> A new example.\n\n{}",fs::read_to_string(&page).unwrap())).unwrap();
    let after = check(&source, true).book.unwrap();
    let a = before
        .statements
        .iter()
        .find(|s| s.id == "thm:energy")
        .unwrap();
    let b = after
        .statements
        .iter()
        .find(|s| s.id == "thm:energy")
        .unwrap();
    assert_eq!(a.number.as_deref(), Some("1.3"));
    assert_eq!(b.number.as_deref(), Some("1.4"));
    assert_eq!(a.href, b.href);
    assert_eq!(before.equations[1].number, after.equations[1].number);
    assert_ne!(before.revision, after.revision);
    let settings = source.join("zel.yaml");
    let original = fs::read_to_string(&settings).unwrap().replace("\r\n", "\n");
    let reordered = original.replace(
        "    - {id: energy, source: chapters/energy.md}\n    - {id: series, source: series.md}",
        "    - {id: series, source: series.md}\n    - {id: energy, source: chapters/energy.md}",
    );
    assert_ne!(original, reordered);
    fs::write(&settings, reordered.replace('\n', "\r\n")).unwrap();
    let reordered = check(&source, true).book.unwrap();
    let c = reordered
        .statements
        .iter()
        .find(|s| s.id == "thm:energy")
        .unwrap();
    assert_eq!(c.number.as_deref(), Some("2.4"));
    assert_eq!(a.href, c.href);
}
#[test]
fn markers_are_book_only_math_is_optional_and_title_text_cannot_inject_code() {
    let tmp = tempfile::tempdir().unwrap();
    let root = Utf8Path::from_path(tmp.path()).unwrap();
    let source = root.join("source");
    fs::create_dir_all(source.join("content")).unwrap();
    let content = "# Plain\n\n> [!THEOREM] {#safe} <script>alert(1)</script> & \\input{secret}\n>\n> Ordinary body.\n\n> [!NOTE]\n> An ordinary alert.\n";
    fs::write(
        source.join("content/index.md"),
        content.replace('\n', "\r\n"),
    )
    .unwrap();
    fs::write(source.join("content/chapter.md"), "# A chapter\n").unwrap();
    fs::write(source.join("zel.yaml"), "version: 1\nrenderer: portable\n").unwrap();
    let out = root.join("site");
    build(&source, &out, BuildOptions::default()).unwrap();
    let html = fs::read_to_string(out.join("index.html")).unwrap();
    assert!(html.contains("[!THEOREM]"));
    assert!(!html.contains("id=\"_zel-stmt-safe\""));
    fs::write(
        source.join("zel.yaml"),
        "version: 1\nrenderer: portable\nbook:\n  version: 1\n  title: Plain\n  authors: [Writer]\n  chapters: [{id: chapter, source: chapter.md}]\n",
    )
    .unwrap();
    build(&source, &out, BuildOptions::default()).unwrap();
    let html = fs::read_to_string(out.join("index.html")).unwrap();
    assert!(html.contains("id=\"_zel-stmt-safe\""));
    assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
    assert!(!html.contains("<script>alert(1)"));
    assert!(html.contains("markdown-alert-note"));
    let prepared = print(&source, root);
    let tex = String::from_utf8(prepared.source.files[Utf8Path::new("book.tex")].clone()).unwrap();
    assert!(!tex.contains("\\input{secret}"));
    assert!(tex.contains("\\textbackslash{}input\\{secret\\}"));
}
