use camino::{Utf8Path, Utf8PathBuf};
use serde_json::Value;
use std::fs;
use zellij_core::site::SiteInput;
use zellij_docs::{BuildOptions, adapt_hosted, build, check, pdf};
fn fixture(root: &Utf8Path) -> Utf8PathBuf {
    let input =
        Utf8Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../scripts/fixtures/equations");
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
#[test]
fn publication_numbers_forward_and_backward_references_once_for_all_renderers() {
    let tmp = tempfile::tempdir().unwrap();
    let root = Utf8Path::from_path(tmp.path()).unwrap();
    let source = fixture(root);
    let out = root.join("site");
    let report = check(&source, true);
    assert!(report.ok, "{:?}", report.diagnostics);
    let publication = serde_json::to_value(report.book.unwrap()).unwrap();
    let equations = publication["equations"].as_array().unwrap();
    assert_eq!(
        equations
            .iter()
            .map(|e| e["number"].as_str().unwrap())
            .collect::<Vec<_>>(),
        ["F1.1", "1.1", "1.2", "2.1", "A.1"]
    );
    assert_eq!(equations[1]["id"], "eq:energy");
    assert_eq!(
        equations[1]["href"],
        "chapters/energy.html#_zel-eq-eq%3Aenergy"
    );
    assert!(equations[1].get("page").is_none());
    build(&source, &out, BuildOptions::default()).unwrap();
    assert_eq!(json(&out.join("_zellij/book.json")), publication);
    let html = fs::read_to_string(out.join("chapters/energy.html")).unwrap();
    assert!(html.contains("id=\"_zel-eq-eq:energy\""));
    assert!(html.contains("aria-label=\"Equation 1.1\">(1.1)</a>"));
    assert!(
        html.contains("href=\"../series.html#_zel-eq-eq%3Asum\" title=\"Equation 2.1\">(2.1)</a>")
    );
    assert!(html.contains("title=\"Equation 2.1\">2.1</a>"));
    assert!(!html.contains("\\label"));
    let adapted = root.join("adapted");
    adapt_hosted(&source, &adapted).unwrap();
    assert_eq!(
        json(&adapted.join("assets/book.json"))["publication"],
        publication
    );
    // File indexes are source order, not reading order: locate by authored title.
    let doc = fs::read_dir(adapted.join("manual"))
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.to_string_lossy().ends_with(".zeldoc.json"))
        .map(|p| json(Utf8Path::from_path(&p).unwrap()))
        .find(|v| v["title"] == "Energy")
        .unwrap();
    let body = doc["body"].to_string();
    assert!(body.contains("/series.html#_zel-eq-eq%3Asum"));
    assert!(body.contains("\"href\":\"/series.html#_zel-eq-eq:sum\""));
    assert!(body.contains("zellij-equation-number"));
    assert!(body.contains("Equation 1.1"));
    assert!(!body.contains("label{"));
    let prepared = pdf::prepare(
        &SiteInput::from(source.as_path()),
        &root.join("pdf"),
        BuildOptions::default(),
        false,
    )
    .unwrap();
    assert_eq!(prepared.source.revision, publication["revision"]);
    let tex = String::from_utf8(prepared.source.files[Utf8Path::new("book.tex")].clone()).unwrap();
    for n in ["F1.1", "1.1", "1.2", "2.1", "A.1"] {
        assert!(tex.contains(&format!("\\tag{{{n}}}")));
    }
    assert!(tex.contains("}{(2.1)}"));
    assert!(tex.contains("}{(1.2)}"));
    assert!(!tex.contains("\\eqref{"));
    assert!(!tex.contains("\\label{eq:"));
}
#[test]
fn inserting_an_equation_changes_numbers_and_revision_but_keeps_target_identity() {
    let tmp = tempfile::tempdir().unwrap();
    let root = Utf8Path::from_path(tmp.path()).unwrap();
    let source = fixture(root);
    let before = check(&source, true).book.unwrap();
    let page = source.join("content/chapters/energy.md");
    fs::write(
        &page,
        format!(
            "$$\\alpha\\label{{eq:new}}x=0$$\n\n{}",
            fs::read_to_string(&page).unwrap()
        ),
    )
    .unwrap();
    let report = check(&source, true);
    assert!(report.ok, "{:?}", report.diagnostics);
    let after = report.book.unwrap();
    let old = before
        .equations
        .iter()
        .find(|e| e.id == "eq:energy")
        .unwrap();
    let new = after
        .equations
        .iter()
        .find(|e| e.id == "eq:energy")
        .unwrap();
    assert_eq!(old.number, "1.1");
    assert_eq!(new.number, "1.2");
    assert_eq!(old.href, new.href);
    assert_ne!(before.revision, after.revision);
}
#[test]
fn invalid_semantics_preserve_the_existing_website() {
    let tmp = tempfile::tempdir().unwrap();
    let root = Utf8Path::from_path(tmp.path()).unwrap();
    let source = fixture(root);
    let out = root.join("site");
    build(&source, &out, BuildOptions::default()).unwrap();
    let before = fs::read(out.join("_zellij/book.json")).unwrap();
    let page = source.join("content/chapters/energy.md");
    let original = fs::read_to_string(&page).unwrap();
    for (extra, diagnostic) in [
        (r"$$x\label{eq:sum}$$", "duplicate label"),
        (r"$\eqref{missing}$", "undefined equation"),
        (r"$$x\label{bad label}$$", "labels must"),
        (r"$$x\label{eq:one}\label{eq:two}$$", "exactly one"),
        (r"$$\label{eq:empty}$$", "must contain math"),
        ("$$\\label{eq:comment}% no math\n$$", "must contain math"),
        (
            r"$$\begin{aligned}x&=1\label{eq:nested}\end{aligned}$$",
            "top-level",
        ),
        (r"$x\label{eq:inline}$", "top-level"),
        (r"$$\frac{1\label{eq:nested}}{2}$$", "top-level"),
        (r"$x+\eqref{eq:sum}$", "own inline expression"),
        (r"$$x\tag{99}$$", "manual tags"),
        (
            r"$$\begin{equation}x\end{equation}$$",
            "numbering environment",
        ),
        (r"## Heading $\eqref{eq:sum}$", "not headings or links"),
        (
            r"[Link $\eqref{eq:sum}$](../series.md)",
            "not headings or links",
        ),
    ] {
        fs::write(&page, format!("{original}\n\n{extra}\n")).unwrap();
        let report = check(&source, false);
        assert!(!report.ok, "{extra}");
        assert_eq!(
            report.diagnostics[0].file.as_deref(),
            Some(if extra.contains("label{eq:sum}") {
                "series.md"
            } else {
                "chapters/energy.md"
            })
        );
        assert!(
            report
                .diagnostics
                .iter()
                .any(|d| d.message.contains(diagnostic)),
            "{extra}: {:?}",
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
        assert_eq!(before, fs::read(out.join("_zellij/book.json")).unwrap());
    }
}

#[test]
fn title_and_front_matter_have_distinct_namespaces_and_renumber_with_manifest_order() {
    let tmp = tempfile::tempdir().unwrap();
    let root = Utf8Path::from_path(tmp.path()).unwrap();
    let source = fixture(root);
    fs::write(
        source.join("content/index.md"),
        "# Title\n\n$$x=0\\label{eq:title}$$\n",
    )
    .unwrap();
    let settings = source.join("zel.yaml");
    let original = fs::read_to_string(&settings).unwrap();
    fs::write(
        &settings,
        original.replace("\r\n", "\n").replace(
            "    - {id: preface, source: preface.md}",
            "    - {id: preface, source: preface.md}\n    - {id: afterword, source: afterword.md}",
        ).replace('\n', "\r\n"),
    )
    .unwrap();
    fs::write(
        source.join("content/afterword.md"),
        "# Another opening\n\n$$y=0\\label{eq:afterword}$$\n",
    )
    .unwrap();
    let report = check(&source, true);
    assert!(report.ok, "{:?}", report.diagnostics);
    let book = report.book.unwrap();
    assert_eq!(
        book.equations
            .iter()
            .take(3)
            .map(|e| e.number.as_str())
            .collect::<Vec<_>>(),
        ["T.1", "F1.1", "F2.1"]
    );
    let config = fs::read_to_string(&settings).unwrap().replace("\r\n", "\n");
    let reordered = config.replace(
        "    - {id: energy, source: chapters/energy.md}\n    - {id: series, source: series.md}",
        "    - {id: series, source: series.md}\n    - {id: energy, source: chapters/energy.md}",
    );
    assert_ne!(
        reordered, config,
        "the fixture must actually reorder its chapters"
    );
    fs::write(&settings, reordered).unwrap();
    let after = check(&source, true).book.unwrap();
    assert_eq!(
        after
            .equations
            .iter()
            .find(|e| e.id == "eq:energy")
            .unwrap()
            .number,
        "2.1"
    );
    assert_eq!(
        after
            .equations
            .iter()
            .find(|e| e.id == "eq:sum")
            .unwrap()
            .number,
        "1.1"
    );
    assert_ne!(after.revision, book.revision);
}
