use camino::{Utf8Path, Utf8PathBuf};
use serde_json::{Value, json};
use std::fs;
use zellij_core::site::SiteInput;
use zellij_docs::{BuildOptions, adapt_hosted, build, check, pdf};
fn fixture(root: &Utf8Path) -> Utf8PathBuf {
    let input = Utf8Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../scripts/fixtures/terms");
    let dest = root.join("book");
    for e in walkdir::WalkDir::new(&input) {
        let e = e.unwrap();
        let p = Utf8Path::from_path(e.path()).unwrap();
        let d = dest.join(p.strip_prefix(&input).unwrap());
        if e.file_type().is_dir() {
            fs::create_dir_all(d).unwrap()
        } else {
            fs::copy(p, d).unwrap();
        }
    }
    dest
}
fn read(p: &Utf8Path) -> String {
    fs::read_to_string(p).unwrap()
}
fn record(source: &Utf8Path) -> Value {
    let r = check(source, true);
    assert!(r.ok, "{:?}", r.diagnostics);
    serde_json::to_value(r.book.unwrap()).unwrap()
}
#[test]
fn catalogs_links_occurrences_and_print_share_one_publication() {
    let tmp = tempfile::tempdir().unwrap();
    let root = Utf8Path::from_path(tmp.path()).unwrap();
    let source = fixture(root);
    let book = record(&source);
    let kinds: Vec<_> = book["entries"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e["kind"].as_str().unwrap())
        .collect();
    assert_eq!(
        &kinds[kinds.len() - 3..],
        ["bibliography", "glossary", "index"]
    );
    let glossary = &book["terms"][0];
    let index = &book["terms"][1];
    assert_eq!(
        index["terms"]
            .as_array()
            .unwrap()
            .iter()
            .map(|t| t["id"].as_str().unwrap())
            .collect::<Vec<_>>(),
        ["mechanics", "energy", "sum"]
    );
    assert_eq!(
        index["terms"][1]["occurrences"].as_array().unwrap().len(),
        4
    );
    assert_eq!(
        glossary["terms"][0]["occurrences"][0]["href"],
        "index.html#_zel-use-glossary-energy-1"
    );
    assert_eq!(
        glossary["terms"][1]["occurrences"][0]["source"],
        "chapters/energy.md"
    );
    let site = root.join("site");
    build(&source, &site, BuildOptions::default()).unwrap();
    let energy = read(&site.join("chapters/energy.html"));
    assert!(energy.contains("id=\"_zel-use-index-energy-2\""));
    assert!(energy.contains("href=\"../glossary.html#_zel-glossary-energy\""));
    assert!(
        read(&site.join("book-index.html"))
            .contains("chapters/energy.html#_zel-use-index-energy-2")
    );
    assert!(read(&site.join("glossary.html")).contains("No occurrences."));
    assert!(read(&site.join("_zellij/search.js")).contains("The capacity to do work"));
    assert!(!site.join("glossary.json").exists());
    assert!(!site.join("index.json").exists());
    let hosted = root.join("next");
    adapt_hosted(&source, &hosted).unwrap();
    let trees: Vec<Value> = fs::read_dir(hosted.join("manual"))
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.to_string_lossy().ends_with(".zeldoc.json"))
        .map(|p| serde_json::from_slice(&fs::read(p).unwrap()).unwrap())
        .collect();
    let tree = trees
        .iter()
        .find(|t| t["route"] == "/glossary.html")
        .unwrap()
        .to_string();
    assert!(tree.contains("/#_zel-use-glossary-energy-1"));
    assert!(
        trees
            .iter()
            .find(|t| t["route"] == "/chapters/energy.html")
            .unwrap()
            .to_string()
            .contains("_zel-use-index-energy-2")
    );
    let prepared = pdf::prepare(
        &SiteInput::from(&source),
        &root.join("pdf"),
        BuildOptions::default(),
        false,
    )
    .unwrap();
    assert_eq!(prepared.source.revision, book["revision"]);
    let tex = String::from_utf8(prepared.source.files[Utf8Path::new("book.tex")].clone()).unwrap();
    assert!(tex.contains("\\chapter{Glossary}"));
    assert!(tex.contains("\\chapter{Index}"));
    assert!(tex.contains("\\getpagerefnumber"));
    assert!(tex.contains("\\pageref*"));
    assert!(!tex.contains("glossary:energy"));
    for c in book["terms"].as_array().unwrap() {
        for t in c["terms"].as_array().unwrap() {
            for o in t["occurrences"].as_array().unwrap() {
                let anchor = o["anchor"]
                    .as_str()
                    .unwrap()
                    .bytes()
                    .map(|b| format!("{b:02x}"))
                    .collect::<String>();
                assert!(tex.contains(&anchor));
            }
        }
    }
}
#[test]
fn ordering_revision_and_occurrence_identity_are_stable() {
    let tmp = tempfile::tempdir().unwrap();
    let root = Utf8Path::from_path(tmp.path()).unwrap();
    let source = fixture(root);
    let before = record(&source);
    let p = source.join("content/index.json");
    let mut catalog: Value = serde_json::from_str(&read(&p)).unwrap();
    catalog.as_array_mut().unwrap().reverse();
    fs::write(&p, serde_json::to_vec(&catalog).unwrap()).unwrap();
    let reordered = record(&source);
    assert_eq!(before["terms"], reordered["terms"]);
    assert_ne!(before["revision"], reordered["revision"]);
    let p = source.join("content/chapters/energy.md");
    fs::write(&p, format!("{}\nAnother [sum](index:sum).\n", read(&p))).unwrap();
    let after = record(&source);
    assert_eq!(
        before["terms"][1]["terms"][1]["occurrences"],
        after["terms"][1]["terms"][1]["occurrences"]
    );
}
#[test]
fn invalid_catalogs_and_markers_preserve_previous_export() {
    let tmp = tempfile::tempdir().unwrap();
    let root = Utf8Path::from_path(tmp.path()).unwrap();
    let source = fixture(root);
    let site = root.join("site");
    build(&source, &site, BuildOptions::default()).unwrap();
    let previous = fs::read(site.join("glossary.html")).unwrap();
    let p = source.join("content/glossary.json");
    let original = read(&p);
    for bad in [
        json!([]),
        json!([{"id":"x","term":"X"}]),
        json!([{"id":"bad/id","term":"X","definition":"Y"}]),
        json!([{"id":"x","term":"X","definition":"Y","unknown":1}]),
        json!([{"id":"x","term":"X","definition":"Y","see-also":["missing"]}]),
        json!([{"id":"x","term":"X","definition":"Y","parent":"y"}]),
    ] {
        fs::write(&p, serde_json::to_vec(&bad).unwrap()).unwrap();
        assert!(build(&source, &site, BuildOptions::default()).is_err());
        assert_eq!(fs::read(site.join("glossary.html")).unwrap(), previous);
    }
    fs::write(&p, original).unwrap();
    let p = source.join("content/series.md");
    let original = read(&p);
    for bad in [
        "[missing](glossary:missing)",
        "![image](index:energy)",
        "## [heading](glossary:energy)",
        "[**rich**](index:sum)",
        "[](index:sum)",
        "[bad](index:sum?query)",
        "[bad](glossary:energy#frag)",
        "[x][_zel-use-fake]\n\n[_zel-use-fake]: https://example.com",
    ] {
        fs::write(&p, format!("{original}\n{bad}\n")).unwrap();
        let r = check(&source, true);
        assert!(!r.ok, "{bad}");
        assert!(build(&source, &site, BuildOptions::default()).is_err());
        assert_eq!(fs::read(site.join("glossary.html")).unwrap(), previous);
    }
}
#[test]
fn catalog_paths_reserved_routes_and_subentry_cycles_are_rejected() {
    let tmp = tempfile::tempdir().unwrap();
    let root = Utf8Path::from_path(tmp.path()).unwrap();
    let source = fixture(root);
    let p = source.join("zel.yaml");
    let original = read(&p);
    for bad in [
        "../index.json",
        ".hidden/index.json",
        "_zellij/index.json",
        "index.txt",
    ] {
        fs::write(
            &p,
            original.replace("source: index.json", &format!("source: {bad}")),
        )
        .unwrap();
        assert!(!check(&source, true).ok, "{bad}");
    }
    fs::write(&p, &original).unwrap();
    fs::write(source.join("content/book-index.md"), "# Collision\n").unwrap();
    assert!(!check(&source, true).ok);
    fs::remove_file(source.join("content/book-index.md")).unwrap();
    let p = source.join("content/index.json");
    for bad in [
        json!([{"id":"a","term":"A","parent":"a"}]),
        json!([{"id":"a","term":"A","parent":"b"},{"id":"b","term":"B","parent":"a"}]),
        json!([{"id":"a","term":"A","see-also":["a"]}]),
        json!([{"id":"a","term":"A"},{"id":"a","term":"Again"}]),
    ] {
        fs::write(&p, serde_json::to_vec(&bad).unwrap()).unwrap();
        assert!(!check(&source, true).ok);
    }
}
#[test]
fn catalogs_are_plain_data_and_terms_need_no_math_or_javascript() {
    let tmp = tempfile::tempdir().unwrap();
    let root = Utf8Path::from_path(tmp.path()).unwrap();
    let source = root.join("simple");
    fs::create_dir_all(source.join("content")).unwrap();
    fs::write(source.join("zel.yaml"),"version: 1\nrenderer: portable\nbook:\n  version: 1\n  title: Terms\n  authors: [Author]\n  chapters: [{id: chapter, source: chapter.md}]\n  glossary: {source: glossary.json}\n").unwrap();
    fs::write(
        source.join("content/chapter.md"),
        "# Chapter\n\nA [term](glossary:a).\n",
    )
    .unwrap();
    fs::write(
        source.join("content/glossary.json"),
        r#"[{"id":"a","term":"<script> & title","definition":"\\input{secret} <img> & text"}]"#,
    )
    .unwrap();
    let site = root.join("site");
    build(&source, &site, BuildOptions::default()).unwrap();
    let html = read(&site.join("glossary.html"));
    assert!(html.contains("&lt;script&gt; &amp; title"));
    assert!(!html.contains("<img>"));
    assert!(!html.contains("katex"));
    pdf::check(&SiteInput::from(&source)).unwrap();
    fs::write(
        source.join("content/index.html.md"),
        "# Dotted\n\nA [term](glossary:a).\n",
    )
    .unwrap();
    let p = source.join("zel.yaml");
    fs::write(
        &p,
        read(&p).replace(
            "source: chapter.md}]",
            "source: chapter.md}, {id: dotted, source: index.html.md}]",
        ),
    )
    .unwrap();
    let hosted = root.join("hosted");
    adapt_hosted(&source, &hosted).unwrap();
    let all = fs::read_dir(hosted.join("manual"))
        .unwrap()
        .map(|e| fs::read_to_string(e.unwrap().path()).unwrap_or_default())
        .collect::<String>();
    assert!(all.contains("/index.html.html#_zel-use-glossary-a-1"));
}
#[test]
fn ordinary_nonbook_links_still_reject_private_schemes() {
    let tmp = tempfile::tempdir().unwrap();
    let root = Utf8Path::from_path(tmp.path()).unwrap();
    let source = root.join("plain");
    fs::create_dir_all(&source).unwrap();
    let p = source.join("index.md");
    for scheme in ["index:energy", "glossary:energy"] {
        fs::write(&p, format!("# Plain\n\n[term]({scheme})\n")).unwrap();
        assert!(!check(&source, true).ok);
    }
    fs::write(&p, "# Plain\n\n`[term](glossary:energy)`\n").unwrap();
    assert!(check(&source, true).ok);
}
#[test]
fn footnote_definitions_follow_reading_order_without_changing_anchor_allocation() {
    let tmp = tempfile::tempdir().unwrap();
    let root = Utf8Path::from_path(tmp.path()).unwrap();
    let source = fixture(root);
    let p = source.join("content/series.md");
    let original = read(&p);
    fs::write(&p,format!("{original}\n[^late]: A [late term](index:sum).\n\nA [main term](index:sum) before the note[^late].\n")).unwrap();
    let book = record(&source);
    let occurrences = book["terms"][1]["terms"][2]["occurrences"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|o| o["source"] == "series.md")
        .map(|o| o["anchor"].as_str().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(
        occurrences,
        [
            "_zel-use-index-sum-1",
            "_zel-use-index-sum-3",
            "_zel-use-index-sum-2"
        ]
    );
    pdf::check(&SiteInput::from(&source)).unwrap();
}
