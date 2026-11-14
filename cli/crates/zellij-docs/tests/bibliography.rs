use camino::{Utf8Path, Utf8PathBuf};
use serde_json::{Value, json};
use std::fs;
use zellij_core::site::SiteInput;
use zellij_docs::{BuildOptions, adapt_hosted, build, check, pdf};
fn fixture(root: &Utf8Path) -> Utf8PathBuf {
    let input =
        Utf8Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../scripts/fixtures/bibliography");
    let source = root.join("book");
    for e in walkdir::WalkDir::new(&input) {
        let e = e.unwrap();
        let path = Utf8Path::from_path(e.path()).unwrap();
        let dest = source.join(path.strip_prefix(&input).unwrap());
        if e.file_type().is_dir() {
            fs::create_dir_all(dest).unwrap();
        } else {
            fs::copy(path, dest).unwrap();
        }
    }
    source
}
fn read(path: &Utf8Path) -> Value {
    serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
}
fn publication(source: &Utf8Path) -> Value {
    let r = check(source, true);
    assert!(r.ok, "{:?}", r.diagnostics);
    serde_json::to_value(r.book.unwrap()).unwrap()
}
#[test]
fn one_registry_drives_reading_order_links_search_hosted_and_print() {
    let tmp = tempfile::tempdir().unwrap();
    let root = Utf8Path::from_path(tmp.path()).unwrap();
    let source = fixture(root);
    let book = publication(&source);
    let refs = book["bibliography"]["references"].as_array().unwrap();
    assert_eq!(
        refs.iter()
            .map(|r| r["id"].as_str().unwrap())
            .collect::<Vec<_>>(),
        ["vale2024", "river2021", "shebka2026", "uncited"]
    );
    assert_eq!(
        refs.iter()
            .map(|r| r["number"].as_u64().unwrap())
            .collect::<Vec<_>>(),
        [1, 2, 3, 4]
    );
    assert_eq!(
        book["entries"].as_array().unwrap().last().unwrap()["kind"],
        "bibliography"
    );
    assert!(book["entries"].as_array().unwrap().last().unwrap()["source"].is_null());
    let output = root.join("site");
    build(&source, &output, BuildOptions::default()).unwrap();
    assert_eq!(read(&output.join("_zellij/book.json")), book);
    let html = fs::read_to_string(output.join("chapters/energy.html")).unwrap();
    assert!(
        html.contains("href=\"../bibliography.html#_zel-bib-river2021\""),
        "{html}"
    );
    assert!(html.contains("pp. 12–14"));
    assert!(html.contains("<code>[@not-a-citation]</code>"));
    let bib = fs::read_to_string(output.join("bibliography.html")).unwrap();
    for r in refs {
        assert!(bib.contains(r["anchor"].as_str().unwrap()));
        assert!(bib.contains(r["title"].as_str().unwrap()));
    }
    assert!(bib.contains("https://doi.org/10.1234/example.sum"));
    assert!(!output.join("references.json").exists());
    let search = fs::read_to_string(output.join("_zellij/search.js")).unwrap();
    assert!(search.contains("Further reading for curious minds"));
    let hosted = root.join("hosted");
    adapt_hosted(&source, &hosted).unwrap();
    assert_eq!(read(&hosted.join("assets/book.json"))["publication"], book);
    let docs: Vec<Value> = fs::read_dir(hosted.join("manual"))
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.to_string_lossy().ends_with(".zeldoc.json"))
        .map(|p| serde_json::from_slice(&fs::read(p).unwrap()).unwrap())
        .collect();
    let body = docs
        .iter()
        .find(|d| d["route"] == "/bibliography.html")
        .unwrap()
        .to_string();
    assert!(body.contains("_zel-bib-river2021"));
    assert!(body.contains("zellij-bibliography"));
    let print = pdf::prepare(
        &SiteInput::from(&source),
        &root.join("pdf"),
        BuildOptions::default(),
        false,
    )
    .unwrap();
    assert_eq!(print.source.revision, book["revision"]);
    let tex = String::from_utf8(print.source.files[Utf8Path::new("book.tex")].clone()).unwrap();
    assert!(tex.contains("\\backmatter\n\\chapter{Bibliography}"));
    assert!(tex.contains("Finite sums and careful arguments"));
    for r in refs {
        let anchor = r["anchor"]
            .as_str()
            .unwrap()
            .bytes()
            .map(|b| format!("{b:02x}"))
            .collect::<String>();
        assert!(tex.contains(&anchor));
    }
    assert!(tex.contains("\\textbf{[4]}"));
    assert!(!tex.contains("[@river2021]"));
}
#[test]
fn numbering_is_deterministic_and_catalog_changes_invalidate_revision() {
    let tmp = tempfile::tempdir().unwrap();
    let root = Utf8Path::from_path(tmp.path()).unwrap();
    let source = fixture(root);
    let before = publication(&source);
    let path = source.join("content/references.json");
    let mut catalog = read(&path);
    catalog.as_array_mut().unwrap().reverse();
    fs::write(&path, serde_json::to_vec(&catalog).unwrap()).unwrap();
    let reordered = publication(&source);
    assert_eq!(
        before["bibliography"]["references"],
        reordered["bibliography"]["references"]
    );
    assert_ne!(before["revision"], reordered["revision"]);
    catalog[0]["title"] = json!("Changed title");
    fs::write(&path, serde_json::to_vec(&catalog).unwrap()).unwrap();
    assert_ne!(reordered["revision"], publication(&source)["revision"]);
    let config = source.join("zel.yaml");
    let yaml = fs::read_to_string(&config).unwrap();
    fs::write(
        &config,
        yaml.replace("include-uncited: true", "include-uncited: false"),
    )
    .unwrap();
    assert_eq!(
        publication(&source)["bibliography"]["references"]
            .as_array()
            .unwrap()
            .len(),
        3
    );
    let home = source.join("content/index.md");
    let original = fs::read_to_string(&home).unwrap();
    fs::write(home, original.replace("[@vale2024]", "[@river2021]")).unwrap();
    let changed = publication(&source);
    assert_eq!(changed["bibliography"]["references"][0]["id"], "river2021");
    assert_eq!(
        changed["bibliography"]["references"][1]["anchor"],
        "_zel-bib-vale2024"
    );
}
#[test]
fn invalid_catalogs_and_citations_preserve_the_previous_export() {
    let tmp = tempfile::tempdir().unwrap();
    let root = Utf8Path::from_path(tmp.path()).unwrap();
    let source = fixture(root);
    let output = root.join("site");
    build(&source, &output, BuildOptions::default()).unwrap();
    let before = fs::read(output.join("bibliography.html")).unwrap();
    let path = source.join("content/references.json");
    let original = fs::read_to_string(&path).unwrap();
    let valid: Value = serde_json::from_str(&original).unwrap();
    let mut cases = vec!["{}".to_string(), "[]".into(), "[".into()];
    for (key, value) in [
        ("id", json!("bad id")),
        ("id", json!("vale2024")),
        ("title", json!(12)),
        ("URL", json!("javascript:alert(1)")),
        ("URL", json!("https://user:pass@example.com")),
        ("DOI", json!("https://doi.org/10.1234/example")),
        ("author", json!(["Name"])),
        ("issued", json!({"date-parts":[[2024,99]]})),
    ] {
        let mut value_catalog = valid.clone();
        value_catalog[0][key] = value;
        cases.push(value_catalog.to_string());
    }
    let mut duplicate_source = valid.clone();
    duplicate_source[0]["id"] = json!("vale2024");
    duplicate_source[0]["citation-key"] = json!("different-alias");
    cases.push(duplicate_source.to_string());
    for value in cases {
        fs::write(&path, value).unwrap();
        assert!(build(&source, &output, BuildOptions::default()).is_err());
        assert_eq!(fs::read(output.join("bibliography.html")).unwrap(), before);
    }
    fs::write(&path, original).unwrap();
    let chapter = source.join("content/chapters/energy.md");
    let original = fs::read_to_string(&chapter).unwrap();
    for extra in [
        "Unknown [@missing].",
        "Bad [@river2021; missing].",
        "Bad [@river2021, ].",
        "# Heading [@river2021]",
        "![\u{0040}river2021]",
        "[text][@river2021]",
        "![@river2021]\n\n[@river2021]: https://example.com/image.png",
        "[bad](zellij-cite:@river2021)",
        "[@river2021]\n\n[@river2021]: https://example.com",
    ] {
        fs::write(&chapter, format!("{original}\n\n{extra}\n")).unwrap();
        let report = check(&source, true);
        assert!(!report.ok, "accepted {extra}");
        assert!(build(&source, &output, BuildOptions::default()).is_err());
        assert_eq!(fs::read(output.join("bibliography.html")).unwrap(), before);
    }
}
#[test]
fn citation_opt_in_escaping_and_missing_configuration_are_explicit() {
    let tmp = tempfile::tempdir().unwrap();
    let root = Utf8Path::from_path(tmp.path()).unwrap();
    let source = fixture(root);
    let chapter = source.join("content/chapters/energy.md");
    let original = fs::read_to_string(&chapter).unwrap();
    fs::write(&chapter,format!("{original}\n\nEscaped \\[@missing] and `[@missing]`.\n\n```text\n[@missing]\n```\n\n[ordinary @text](https://example.com)\n\n[A literal [@missing] marker](https://example.com)\n")).unwrap();
    assert!(check(&source, true).ok);
    let config = source.join("zel.yaml");
    let yaml = fs::read_to_string(&config).unwrap();
    fs::write(&config, yaml.split("  bibliography:").next().unwrap()).unwrap();
    let report = check(&source, true);
    assert!(!report.ok);
    assert!(
        format!("{:?}", report.diagnostics).contains("citations require book.bibliography.source")
    );
    let plain = root.join("plain");
    fs::create_dir(&plain).unwrap();
    fs::write(plain.join("index.md"), "# Plain\n\n[@not-a-citation]\n").unwrap();
    assert!(check(&plain, true).ok);
    let out = root.join("plain-site");
    build(&plain, &out, BuildOptions::default()).unwrap();
    assert!(
        fs::read_to_string(out.join("index.html"))
            .unwrap()
            .contains("[@not-a-citation]")
    );
    fs::write(
        plain.join("index.md"),
        "# Plain\n\n[invalid](zellij-cite:@id)",
    )
    .unwrap();
    assert!(!check(&plain, true).ok);
}
#[test]
fn generated_routes_and_catalog_paths_are_guarded_and_metadata_is_literal() {
    let tmp = tempfile::tempdir().unwrap();
    let root = Utf8Path::from_path(tmp.path()).unwrap();
    let source = fixture(root);
    let path = source.join("content/references.json");
    let mut catalog = read(&path);
    catalog[0]["title"] = json!("A <script> & \\input{secret}");
    fs::write(&path, catalog.to_string()).unwrap();
    let out = root.join("site");
    build(&source, &out, BuildOptions::default()).unwrap();
    let html = fs::read_to_string(out.join("bibliography.html")).unwrap();
    assert!(html.contains("A &lt;script&gt; &amp; \\input{secret}"));
    let print = pdf::prepare(
        &SiteInput::from(&source),
        &root.join("pdf"),
        BuildOptions::default(),
        false,
    )
    .unwrap();
    let tex = String::from_utf8(print.source.files[Utf8Path::new("book.tex")].clone()).unwrap();
    assert!(!tex.contains("\\input{secret}"));
    fs::write(source.join("content/bibliography.md"), "# Collision\n").unwrap();
    assert!(!check(&source, true).ok);
    fs::remove_file(source.join("content/bibliography.md")).unwrap();
    let config = source.join("zel.yaml");
    let yaml = fs::read_to_string(&config).unwrap();
    for path in [
        "../references.json",
        "/tmp/references.json",
        "references.bib",
        ".secret.json",
    ] {
        fs::write(
            &config,
            yaml.replace("source: references.json", &format!("source: {path}")),
        )
        .unwrap();
        assert!(!check(&source, true).ok, "{path}");
    }
}

#[test]
fn footnotes_follow_their_first_call_site_and_empty_bibliographies_are_consistent() {
    let tmp = tempfile::tempdir().unwrap();
    let root = Utf8Path::from_path(tmp.path()).unwrap();
    let source = fixture(root);
    let home = source.join("content/index.md");
    let original = fs::read_to_string(&home).unwrap();
    fs::write(&home,original.replace("Start with [@vale2024].", "[^early]: [@shebka2026].\n\nStart with [@vale2024], then a note.[^early] [@river2021].")).unwrap();
    let p = publication(&source);
    assert_eq!(p["bibliography"]["references"][1]["id"], "shebka2026");
    let simple = root.join("empty");
    fs::create_dir_all(simple.join("content")).unwrap();
    fs::write(simple.join("zel.yaml"),"version: 1\nrenderer: portable\nbook:\n  version: 1\n  title: No citations\n  authors: [Author]\n  chapters: [{id: start, source: start.md}]\n  bibliography: {source: refs.json}\n").unwrap();
    fs::write(
        simple.join("content/start.md"),
        "# Start\n\nNo citations here.",
    )
    .unwrap();
    fs::write(
        simple.join("content/refs.json"),
        r#"[{"id":"one","type":"book","title":"Unused"}]"#,
    )
    .unwrap();
    assert_eq!(
        publication(&simple)["bibliography"]["references"],
        json!([])
    );
    let out = root.join("empty-site");
    build(&simple, &out, BuildOptions::default()).unwrap();
    assert!(
        fs::read_to_string(out.join("bibliography.html"))
            .unwrap()
            .contains("No references cited.")
    );
    fs::write(
        simple.join("content/start.md"),
        "# Start\n\nA citation without math [@one].",
    )
    .unwrap();
    assert_eq!(
        publication(&simple)["bibliography"]["references"][0]["number"],
        1
    );
    fs::write(
        simple.join("content/start.md"),
        "# Start\n\nNo citations here.",
    )
    .unwrap();
    let print = pdf::prepare(
        &SiteInput::from(&simple),
        &root.join("empty-pdf"),
        BuildOptions::default(),
        false,
    )
    .unwrap();
    assert!(
        String::from_utf8(print.source.files[Utf8Path::new("book.tex")].clone())
            .unwrap()
            .contains("No references cited.")
    );
}

#[test]
fn csl_aliases_numeric_ids_and_bounded_citation_clusters_work() {
    let tmp = tempfile::tempdir().unwrap();
    let root = Utf8Path::from_path(tmp.path()).unwrap();
    let source = fixture(root);
    let path = source.join("content/references.json");
    let mut catalog = read(&path);
    catalog[0]["id"] = json!("https://example.com/reference/river");
    catalog[0]["citation-key"] = json!("river2021");
    catalog[0]["volume"] = json!(4);
    catalog[0]["issue"] = json!(2);
    catalog
        .as_array_mut()
        .unwrap()
        .push(json!({"id":123,"type":"book","title":"A numbered catalog record"}));
    fs::write(&path, catalog.to_string()).unwrap();
    let home = source.join("content/index.md");
    let original = fs::read_to_string(&home).unwrap();
    fs::write(&home, format!("{original}\n\nAnother source [@123].")).unwrap();
    let p = publication(&source);
    assert_eq!(p["bibliography"]["references"][1]["id"], "123");
    let refs = p["bibliography"]["references"].as_array().unwrap();
    assert_eq!(
        refs.iter().find(|r| r["id"] == "river2021").unwrap()["source_id"],
        "https://example.com/reference/river"
    );
    for bad in [
        "[@123,\n p. 1]".to_owned(),
        format!(
            "[{}]",
            std::iter::repeat_n("@123", 33)
                .collect::<Vec<_>>()
                .join("; ")
        ),
    ] {
        fs::write(&home, format!("{original}\n\n{bad}")).unwrap();
        assert!(!check(&source, true).ok, "{bad}");
    }
}
