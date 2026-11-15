use camino::Utf8Path;
use std::fs;
use zellij_core::site::SiteInput;
use zellij_docs::{BuildOptions, adapt_hosted, build, pdf};

#[test]
fn editions_drive_web_print_and_revision_while_classic_is_unchanged() {
    let temp = tempfile::tempdir().unwrap();
    let root = Utf8Path::from_path(temp.path()).unwrap();
    fs::create_dir(root.join("content")).unwrap();
    fs::write(
        root.join("content/one.md"),
        "# First chapter\n\nA useful idea.\n",
    )
    .unwrap();
    let base = "version: 1\nrenderer: portable\nbook:\n  version: 1\n  title: A book\n  authors: [Author]\n  chapters: [{id: first, source: one.md}]\n";
    let mut classic = None;
    let mut revisions = std::collections::BTreeSet::new();
    for edition in ["", "classic", "technical", "study"] {
        fs::write(
            root.join("zel.yaml"),
            format!(
                "{base}{}",
                if edition.is_empty() {
                    String::new()
                } else {
                    format!("  edition: {edition}\n")
                }
            ),
        )
        .unwrap();
        let prepared = pdf::prepare(
            &SiteInput::from(root),
            &root.join("print"),
            BuildOptions::default(),
            false,
        )
        .unwrap();
        let tex =
            String::from_utf8(prepared.source.files[Utf8Path::new("book.tex")].clone()).unwrap();
        build(root, &root.join("site"), BuildOptions::default()).unwrap();
        let html = fs::read_to_string(root.join("site/one.html")).unwrap();
        let publication = fs::read(root.join("site/_zellij/book.json")).unwrap();
        let adapted = root.join(format!("adapted-{edition}"));
        adapt_hosted(root, &adapted).unwrap();
        let tree: serde_json::Value =
            serde_json::from_slice(&fs::read(adapted.join("manual/p0.zeldoc.json")).unwrap())
                .unwrap();
        let snapshot = (
            tex.clone(),
            html.clone(),
            publication,
            prepared.source.revision.clone(),
            tree.clone(),
        );
        match edition {
            "" => {
                classic = Some(snapshot);
            }
            "classic" => assert_eq!(
                snapshot,
                classic.clone().unwrap(),
                "Explicit classic preserves HTML, TeX, tree, record and revision"
            ),
            _ => {
                assert!(html.contains(&format!("zellij-book-{edition}")));
                assert!(html.contains("_zellij/book-editions.css"));
                assert_eq!(
                    tree["body"]["children"][0]["properties"]["className"][1],
                    format!("zellij-book-{edition}")
                );
                assert!(tex.contains(&format!("{} EDITION", edition.to_uppercase())));
                assert!(tex.contains("\\zelcover{A book}"));
                assert_eq!(prepared.source.template, format!("{edition}-v1"));
                assert_ne!(prepared.source.revision, classic.as_ref().unwrap().3);
                assert!(revisions.insert(prepared.source.revision.clone()));
                let css = fs::read(root.join("site/_zellij/book-editions.css")).unwrap();
                assert_eq!(
                    css,
                    include_bytes!(
                        "../../../../engine/packages/zellij/src/styles/book-editions.css"
                    )
                );
            }
        }
    }
    let before = fs::read(root.join("site/index.html")).unwrap();
    for bad in ["novel", "true", "{}", "[technical]"] {
        fs::write(root.join("zel.yaml"), format!("{base}  edition: {bad}\n")).unwrap();
        assert!(build(root, &root.join("site"), BuildOptions::default()).is_err());
        assert_eq!(fs::read(root.join("site/index.html")).unwrap(), before);
    }
}
