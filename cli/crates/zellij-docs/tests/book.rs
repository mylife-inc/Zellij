use camino::{Utf8Path, Utf8PathBuf};
use serde_json::Value;
use std::fs;
use zellij_docs::{BuildOptions, adapt_hosted, build, check};

fn fixture(root: &Utf8Path) -> Utf8PathBuf {
    let input = Utf8Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../scripts/fixtures/book");
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
fn revision(source: &Utf8Path) -> String {
    let report = check(source, true);
    assert!(report.ok, "{:?}", report.diagnostics);
    report.book.unwrap().revision
}

#[test]
fn one_publication_drives_check_export_and_hosted_order() {
    let tmp = tempfile::tempdir().unwrap();
    let root = Utf8Path::from_path(tmp.path()).unwrap();
    let source = fixture(root);
    let report = check(&source, true);
    assert!(report.ok, "{:?}", report.diagnostics);
    assert_eq!(report.pages, Some(6));
    assert!(!source.join("site").exists());
    let publication = serde_json::to_value(report.book.unwrap()).unwrap();
    let entries = publication["entries"].as_array().unwrap();
    assert_eq!(
        entries
            .iter()
            .map(|e| e["id"].as_str().unwrap())
            .collect::<Vec<_>>(),
        [
            "title-page",
            "preface",
            "noticing",
            "questions",
            "practice",
            "field-notes"
        ]
    );
    assert_eq!(entries[3]["number"], "2");
    assert_eq!(entries[3]["part"], "attention");
    assert_eq!(entries[3]["title"], "Better questions");
    assert_eq!(entries[5]["number"], "A");
    assert!(entries[0]["source"].is_null());
    assert!(entries[0].get("index").is_none());
    let out = root.join("site");
    build(&source, &out, BuildOptions::default()).unwrap();
    assert_eq!(json(&out.join("_zellij/book.json")), publication);
    let home = fs::read_to_string(out.join("index.html")).unwrap();
    for expected in [
        "An atlas of quiet ideas",
        "Mira Vale",
        "assets/cover.svg",
        "data-book-id=\"title-page\"",
        "data-zellij-theme=\"paper\"",
        "<h2>Contents</h2>",
    ] {
        assert!(home.contains(expected), "{expected}");
    }
    let chapter = fs::read_to_string(out.join("chapters/z-noticing.html")).unwrap();
    assert!(chapter.contains("Chapter 1"));
    assert!(chapter.contains("rel=\"next\" href=\"../chapters/a-questions.html\""));
    assert!(chapter.contains("rel=\"prev\" href=\"../preface.html\""));
    let question = fs::read_to_string(out.join("chapters/a-questions.html")).unwrap();
    assert!(question.contains(">A question worth keeping</h1>"));
    let adapted = root.join("adapted");
    adapt_hosted(&source, &adapted).unwrap();
    assert_eq!(json(&adapted.join("site.yaml"))["language"], "en");
    assert_eq!(
        json(&adapted.join("assets/book.json"))["publication"],
        publication
    );
    assert_eq!(
        json(&adapted.join("assets/book.json"))["routes"][0]["route"],
        "/"
    );
    let sidebar = json(&adapted.join("manual/_sidebar.yaml"));
    assert_eq!(
        sidebar["nav"][2]["items"][0]["label"],
        "1. The art of noticing"
    );
    let home = json(&adapted.join("manual/p0.zeldoc.json")).to_string();
    assert!(home.contains("/_zellij/assets/portable/assets/cover.svg"));
    assert!(home.contains("Mira Vale"));
}

#[test]
fn invalid_manifests_preserve_published_output() {
    let tmp = tempfile::tempdir().unwrap();
    let root = Utf8Path::from_path(tmp.path()).unwrap();
    let source = fixture(root);
    let settings = source.join("zel.yaml");
    let original = fs::read_to_string(&settings).unwrap();
    let out = root.join("site");
    build(&source, &out, BuildOptions::default()).unwrap();
    let before = fs::read(out.join("_zellij/book.json")).unwrap();
    for changed in [
        original.replace("chapters/z-noticing.md", "missing.md"),
        original.replace("id: questions", "id: noticing"),
        original.replace("chapters/a-questions.md", "chapters/z-noticing.md"),
        original.replace("assets/cover.svg", "../cover.svg"),
        original.replace("assets/cover.svg", "notes.md"),
        original.replace("    - {id: practice, source: practice.md}\n", ""),
        original.replace("language: en", "language: 'en\" onload=\"alert(1)'"),
        format!("{original}\nnav: []\n"),
    ] {
        fs::write(&settings, changed).unwrap();
        let report = check(&source, false);
        assert!(!report.ok, "invalid book succeeded");
        assert!(report.book.is_none());
        assert!(build(&source, &out, BuildOptions::default()).is_err());
        assert_eq!(fs::read(out.join("_zellij/book.json")).unwrap(), before);
    }
    fs::write(&settings, original).unwrap();
    fs::write(source.join("content/draft.md"), "# A draft").unwrap();
    assert!(
        check(&source, false)
            .diagnostics
            .iter()
            .any(|d| d.message.contains("unlisted Markdown"))
    );
}

#[test]
fn revision_is_location_and_presentation_independent_but_tracks_content() {
    let tmp = tempfile::tempdir().unwrap();
    let root = Utf8Path::from_path(tmp.path()).unwrap();
    let source = fixture(root);
    let initial = revision(&source);
    let settings = source.join("zel.yaml");
    let original = fs::read_to_string(&settings).unwrap();
    fs::write(
        &settings,
        format!("# Formatting is not content\n{original}\ntheme: forest\nname: My site\n"),
    )
    .unwrap();
    assert_eq!(revision(&source), initial);
    build(
        &source,
        &root.join("override"),
        BuildOptions {
            name: Some("Another name"),
            theme: Some("cupertino"),
            features: None,
        },
    )
    .unwrap();
    assert_eq!(
        json(&root.join("override/_zellij/book.json"))["revision"],
        initial
    );
    for (path, replacement) in [
        ("zel.yaml", original.replace("Mira Vale", "Mira & Vale")),
        ("content/notes.md", "# Revised field notes\n".into()),
        (
            "content/assets/cover.svg",
            "<svg xmlns=\"http://www.w3.org/2000/svg\"/>".into(),
        ),
    ] {
        let path = source.join(path);
        let before = fs::read(&path).unwrap();
        fs::write(&path, replacement).unwrap();
        assert_ne!(revision(&source), initial);
        fs::write(&path, before).unwrap();
    }
    let moved = root.join("Moved 日本");
    fs::rename(&source, &moved).unwrap();
    assert_eq!(revision(&moved), initial);
}

#[test]
fn authored_title_page_and_rtl_metadata_remain_authored() {
    let tmp = tempfile::tempdir().unwrap();
    let root = Utf8Path::from_path(tmp.path()).unwrap();
    let source = fixture(root);
    let settings = source.join("zel.yaml");
    let original = fs::read_to_string(&settings).unwrap();
    fs::write(
        &settings,
        original
            .replace("language: en", "language: ar")
            .replace("direction: ltr", "direction: rtl"),
    )
    .unwrap();
    fs::write(
        source.join("content/README.md"),
        "# My own title page\n\nMy own introduction.",
    )
    .unwrap();
    let out = root.join("site");
    build(&source, &out, BuildOptions::default()).unwrap();
    let home = fs::read_to_string(out.join("index.html")).unwrap();
    assert!(home.contains("lang=\"ar\" dir=\"rtl\""));
    assert!(home.contains("My own introduction."));
    assert!(!home.contains("class=\"book-cover\""));
    assert_eq!(
        json(&out.join("_zellij/book.json"))["entries"][0]["source"],
        "README.md"
    );
    let adapted = root.join("adapted");
    adapt_hosted(&source, &adapted).unwrap();
    assert_eq!(json(&adapted.join("site.yaml"))["direction"], "rtl");
    fs::write(
        &settings,
        original.replace("source: preface.md", "source: README.md"),
    )
    .unwrap();
    assert!(
        check(&source, false)
            .diagnostics
            .iter()
            .any(|d| d.message.contains("title page is automatic"))
    );
}

#[test]
fn appendix_numbers_continue_after_z_and_metadata_is_escaped() {
    let temp = tempfile::tempdir().unwrap();
    let root = Utf8Path::from_path(temp.path()).unwrap();
    let source = root.join("book");
    fs::create_dir_all(source.join("content")).unwrap();
    let mut manifest = String::from(
        "version: 1\nrenderer: portable\nbook:\n  version: 1\n  title: '<b>A book</b>'\n  authors: ['A & B']\n  subtitle: '<script>literal</script>'\n  chapters: [{id: first, source: first.md}]\n  appendices:\n",
    );
    fs::write(source.join("content/first.md"), "# First").unwrap();
    for i in 1..=27 {
        manifest.push_str(&format!("    - {{id: notes-{i}, source: note-{i}.md}}\n"));
        fs::write(source.join(format!("content/note-{i}.md")), "# Notes").unwrap();
    }
    fs::write(source.join("zel.yaml"), manifest).unwrap();
    let book = check(&source, true).book.unwrap();
    assert_eq!(book.entries.last().unwrap().number.as_deref(), Some("AA"));
    build(&source, &root.join("site"), BuildOptions::default()).unwrap();
    let home = fs::read_to_string(root.join("site/index.html")).unwrap();
    assert!(home.contains("&lt;script&gt;literal&lt;/script&gt;"));
    assert!(home.contains("A &amp; B"));
    assert!(!home.contains("<b>A book</b>"));
    assert!(!home.contains("<ul class=\"document-list\"><li><a href=\"index.html\""));
}
