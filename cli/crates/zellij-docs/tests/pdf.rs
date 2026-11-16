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

#[test]
fn next_print_uses_one_publication_and_publishes_only_verified_downloads() {
    use sha2::{Digest, Sha256};
    let temp = tempfile::tempdir().unwrap();
    let root = Utf8Path::from_path(temp.path()).unwrap();
    fixture(root);
    let source = SiteInput::from(root);
    let native =
        pdf::prepare(&source, &root.join("native"), BuildOptions::default(), true).unwrap();
    let output = root.join("next");
    let prepared = pdf::prepare_hosted(&source, &output, BuildOptions::default()).unwrap();
    assert_eq!(prepared.source.files, native.source.files);
    assert_eq!(prepared.source.revision, native.source.revision);
    assert_eq!(prepared.features, vec!["maths"]);
    let pdf_bytes = b"%PDF-1.7\nfixture\n%%EOF\n";
    fs::write(prepared.destination.join("book.pdf"), pdf_bytes).unwrap();
    fs::write(
        prepared.destination.join("book.tex"),
        b"private print source",
    )
    .unwrap();
    fs::write(
        prepared.destination.join("build.log"),
        b"private diagnostic",
    )
    .unwrap();
    fs::create_dir(prepared.destination.join("assets")).unwrap();
    fs::write(
        prepared.destination.join("assets/a.png"),
        b"private print input",
    )
    .unwrap();
    fs::write(
        prepared.destination.join("pdf.json"),
        serde_json::to_vec(&serde_json::json!({
            "revision":prepared.source.revision,
            "pdf_sha256":format!("{:x}", Sha256::digest(pdf_bytes)),
        }))
        .unwrap(),
    )
    .unwrap();
    prepared.publish().unwrap();
    let record: serde_json::Value =
        serde_json::from_slice(&fs::read(output.join("assets/book.json")).unwrap()).unwrap();
    assert_eq!(record["publication"]["revision"], native.source.revision);
    assert_eq!(
        fs::read(output.join("assets/pdf/book.pdf")).unwrap(),
        pdf_bytes
    );
    assert_eq!(fs::read_dir(output.join("assets/pdf")).unwrap().count(), 2);
    assert!(!output.join(".zel-pdf-work").exists());
    for index in 0..3 {
        let body = fs::read_to_string(output.join(format!("manual/p{index}.zeldoc.json"))).unwrap();
        assert!(body.contains("/_zellij/assets/pdf/book.pdf"));
        assert!(body.contains("\"download\": true"));
    }
    let before = fs::read(output.join("assets/book.json")).unwrap();
    for bad in ["missing", "revision", "hash"] {
        let prepared = pdf::prepare_hosted(&source, &output, BuildOptions::default()).unwrap();
        fs::write(prepared.destination.join("book.pdf"), pdf_bytes).unwrap();
        if bad != "missing" {
            fs::write(prepared.destination.join("pdf.json"),serde_json::to_vec(&serde_json::json!({
                "revision":if bad=="revision" {"stale"} else {&prepared.source.revision},
                "pdf_sha256":if bad=="hash" {"invalid".into()} else {format!("{:x}",Sha256::digest(pdf_bytes))},
            })).unwrap()).unwrap();
        }
        assert!(prepared.publish().is_err());
        assert_eq!(fs::read(output.join("assets/book.json")).unwrap(), before);
        assert_eq!(
            fs::read(output.join("assets/pdf/book.pdf")).unwrap(),
            pdf_bytes
        );
    }
}

#[test]
fn print_overrides_preserve_web_and_content_revision_but_customize_both_print_paths() {
    let temp = tempfile::tempdir().unwrap();
    let root = Utf8Path::from_path(temp.path()).unwrap();
    fixture(root);
    let base = fs::read_to_string(root.join("zel.yaml")).unwrap();
    for edition in ["classic", "technical", "study"] {
        let config = base.replace(
            "  version: 1",
            &format!("  version: 1\n  edition: {edition}"),
        );
        fs::write(root.join("zel.yaml"), &config).unwrap();
        let source = SiteInput::from(root);
        let original = pdf::prepare(
            &source,
            &root.join("original"),
            BuildOptions::default(),
            false,
        )
        .unwrap();
        zellij_docs::build(root, &root.join("site"), BuildOptions::default()).unwrap();
        let html = fs::read(root.join("site/first.html")).unwrap();
        assert!(
            !serde_json::to_value(&original.source.settings)
                .unwrap()
                .as_object()
                .unwrap()
                .contains_key("style")
        );
        fs::write(
            root.join("zel.yaml"),
            format!("{config}pdf: {{style: {{}}}}\n"),
        )
        .unwrap();
        let empty =
            pdf::prepare(&source, &root.join("empty"), BuildOptions::default(), false).unwrap();
        assert_eq!(
            original.source.files, empty.source.files,
            "empty style must preserve every print source byte"
        );
        assert!(
            !String::from_utf8_lossy(&empty.source.files[Utf8Path::new("book.tex")])
                .contains("@TITLE_FONT@")
        );
        for chapter in ["classic", "technical", "study"] {
            let custom = format!(
                "{config}pdf:\n  style:\n    accent: '#493B72'\n    body-font: sans\n    title-font: serif\n    chapter-style: {chapter}\n    line-spacing: relaxed\n    margins: {{left: 24, right: 22}}\n"
            );
            fs::write(root.join("zel.yaml"), &custom).unwrap();
            let native = pdf::prepare(
                &source,
                &root.join("native"),
                BuildOptions::default(),
                false,
            )
            .unwrap();
            let hosted =
                pdf::prepare_hosted(&source, &root.join("hosted"), BuildOptions::default())
                    .unwrap();
            assert_eq!(native.source.files, hosted.source.files);
            assert_eq!(native.source.revision, original.source.revision);
            assert_eq!(native.source.template, original.source.template);
            let tex =
                String::from_utf8(native.source.files[Utf8Path::new("book.tex")].clone()).unwrap();
            assert!(tex.contains("\\definecolor{zelaccent}{HTML}{493B72}"));
            assert!(tex.contains("\\renewcommand{\\familydefault}{\\sfdefault}"));
            assert!(tex.contains("\\linespread{1.15}"));
            assert!(tex.contains("\\geometry{left=24mm,right=22mm,top=20mm,bottom=22mm}"));
            let preamble = tex.split("\\begin{document}").next().unwrap();
            for title in preamble.split("\\Huge").skip(1) {
                assert!(title.split('#').next().unwrap().contains("\\rmfamily"));
            }
            let opening = preamble.rsplit("\\def\\@makechapterhead").next().unwrap();
            assert!(opening.contains(match chapter {
                "technical" => "\\hrule height 2pt",
                "study" => "\\rule{.25\\linewidth}",
                _ => "\\hrule height 0.6pt",
            }));
            assert_eq!(
                tex.split("\\begin{document}")
                    .nth(1)
                    .unwrap()
                    .matches("\\end{document}")
                    .count(),
                1
            );
            zellij_docs::build(root, &root.join("site"), BuildOptions::default()).unwrap();
            assert_eq!(html, fs::read(root.join("site/first.html")).unwrap());
        }
    }
}
