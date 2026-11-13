use camino::Utf8Path;
use std::fs;
use zellij_core::site::SiteInput;
use zellij_docs::{BuildOptions, pdf};
fn fixture(root: &Utf8Path) {
    fs::create_dir_all(root.join("content")).unwrap();
    fs::write(root.join("zel.yaml"),"version: 1\nrenderer: portable\nmarkdown: {math: true}\nbook:\n  version: 1\n  title: 'Text & proofs'\n  authors: [Author]\n  chapters: [{id: first, source: first.md}, {id: second, source: second.md}]\n").unwrap();
    fs::write(root.join("content/first.md"),"# First\n\nText with **weight**, `code` and $x^2$.\n\n## Detail\n\n[Next](second.md#answer).\n\nA note[^a].\n\n[^a]: Footnote body.\n\n| A | B |\n|---|---|\n| 1 | 2 |\n\n```text\n\\end{document}\n```\n").unwrap();
    fs::write(
        root.join("content/second.md"),
        "# Second\n\n## Answer\n\n[Back](first.md#detail).\n",
    )
    .unwrap();
}
#[test]
fn print_uses_validated_content_order_links_math_and_escaped_text() {
    let temp = tempfile::tempdir().unwrap();
    let root = Utf8Path::from_path(temp.path()).unwrap();
    fixture(root);
    let prepared = pdf::prepare(
        &SiteInput::from(root),
        &root.join("site"),
        BuildOptions::default(),
        true,
    )
    .unwrap();
    let tex = String::from_utf8(prepared.source.files[Utf8Path::new("book.tex")].clone()).unwrap();
    assert!(tex.contains("Text \\& proofs"));
    assert!(tex.find("\\chapter{First}").unwrap() < tex.find("\\chapter{Second}").unwrap());
    assert!(tex.contains("\\(x^2\\)"));
    assert!(tex.contains("\\footnote{\\hypertarget{"));
    assert!(tex.contains("Footnote body."));
    assert!(tex.contains("\\begin{longtable}"));
    let target = tex
        .split("\\hyperlink{")
        .nth(1)
        .unwrap()
        .split('}')
        .next()
        .unwrap();
    assert!(target.ends_with("616e73776572"));
    assert!(
        tex[tex.find("\\chapter{Second}").unwrap()..]
            .contains(&format!("\\hypertarget{{{target}}}"))
    );
    assert!(tex.contains("\\textbackslash{}end\\{document\\}"));
    assert_eq!(tex.matches("\\end{document}").count(), 1);
    let report = zellij_docs::check(root, true);
    assert_eq!(prepared.source.revision, report.book.unwrap().revision);
    assert!(!root.join("site").exists());
    assert!(prepared.publish().is_err());
    assert!(!root.join("site").exists());
}
#[test]
fn unsupported_print_features_and_failed_export_preserve_web_output() {
    let temp = tempfile::tempdir().unwrap();
    let root = Utf8Path::from_path(temp.path()).unwrap();
    fixture(root);
    zellij_docs::build(root, &root.join("site"), BuildOptions::default()).unwrap();
    let before = fs::read(root.join("site/first.html")).unwrap();
    for bad in [
        "# First\n![Remote](https://example.com/a.png)",
        "# First\n```mindmap\nRoot\n```",
        "# First\n[Download](asset.txt)",
    ] {
        fs::write(root.join("content/asset.txt"), "data").unwrap();
        fs::write(root.join("content/first.md"), bad).unwrap();
        // Keep the other page's backlink valid for the intended print error.
        fs::write(root.join("content/second.md"), "# Second\n\n## Answer\n").unwrap();
        assert!(
            pdf::prepare(
                &SiteInput::from(root),
                &root.join("site"),
                BuildOptions::default(),
                true
            )
            .is_err()
        );
        assert_eq!(before, fs::read(root.join("site/first.html")).unwrap());
    }
}

#[test]
fn generated_headings_and_repeated_footnotes_keep_resolvable_print_targets() {
    let temp = tempfile::tempdir().unwrap();
    let root = Utf8Path::from_path(temp.path()).unwrap();
    fixture(root);
    fs::write(root.join("content/first.md"),"---\ntitle: No authored heading\n---\n[Home](index.html#_zel-title).\n\nA note[^n] and the same note[^n]. [Its text](#n).\n\n[^n]: Unique footnote body.\n\n| A | B | C | D | E | F |\n|---|---|---|---|---|---|\n| a | b | c | d | e | f |\n").unwrap();
    fs::write(
        root.join("content/second.md"),
        "# Second\n\n[First](first.md#_zel-title).\n",
    )
    .unwrap();
    let prepared = pdf::prepare(
        &SiteInput::from(root),
        &root.join("printed"),
        BuildOptions::default(),
        false,
    )
    .unwrap();
    let tex = String::from_utf8(prepared.source.files[Utf8Path::new("book.tex")].clone()).unwrap();
    assert_eq!(tex.matches("Unique footnote body.").count(), 1);
    assert!(tex.contains("\\textsuperscript{\\ref{"));
    for part in tex.split("\\hyperlink{").skip(1) {
        let target = part.split('}').next().unwrap();
        assert!(
            tex.contains(&format!("\\hypertarget{{{target}}}")),
            "unresolved target {target}"
        );
    }
    assert!(tex.contains("\\linewidth/6-2\\tabcolsep"));
    fs::write(
        root.join("content/first.md"),
        "# First\n\n[^n]: Never referenced.\n",
    )
    .unwrap();
    fs::write(root.join("content/second.md"), "# Second\n").unwrap();
    let error = pdf::check(&SiteInput::from(root)).unwrap_err().to_string();
    assert!(error.contains("has no footnote reference"), "{error}");
}
