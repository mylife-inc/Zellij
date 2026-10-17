use camino::{Utf8Path, Utf8PathBuf};
use std::{collections::BTreeMap, fs};
use zellij_docs::{BuildOptions, Code, Severity, build, check};

fn fixture() -> (tempfile::TempDir, Utf8PathBuf, Utf8PathBuf) {
    let temp = tempfile::tempdir().unwrap();
    let root = Utf8Path::from_path(temp.path()).unwrap();
    let source = root.join("Docs 日本");
    fs::create_dir(&source).unwrap();
    fs::write(source.join("README.md"), "# Home\n").unwrap();
    let out = root.join("site");
    (temp, source, out)
}
fn snapshot(root: &Utf8Path) -> BTreeMap<String, Vec<u8>> {
    walkdir::WalkDir::new(root)
        .into_iter()
        .map(|e| {
            let e = e.unwrap();
            let path = Utf8Path::from_path(e.path()).unwrap();
            (
                path.strip_prefix(root).unwrap().to_string(),
                if e.file_type().is_file() {
                    fs::read(path).unwrap()
                } else {
                    vec![]
                },
            )
        })
        .collect()
}

#[test]
fn check_compiles_the_same_site_without_any_filesystem_writes() {
    let (_temp, source, out) = fixture();
    fs::create_dir(source.join("guide")).unwrap();
    fs::write(
        source.join("guide/Install Here.md"),
        "# Install\n\n## Setup\n\n```rust\nfn main() {}\n```\n",
    )
    .unwrap();
    fs::write(
        source.join("logo.svg"),
        "<svg xmlns=\"http://www.w3.org/2000/svg\"/>",
    )
    .unwrap();
    fs::write(
        source.join("README.md"),
        "# Home\n\n[Install](guide/Install%20Here.md#setup)\n\n![Company](logo.svg)\n",
    )
    .unwrap();
    fs::write(source.join("zel.yaml"), "theme: forest\nbrand: {logo: {src: logo.svg, alt: Company}}\nnav: [{page: README.md}, {page: guide/Install Here.md}]\n").unwrap();
    // An unowned existing default destination must be irrelevant to checking.
    let default = zellij_docs::default_output(&source).unwrap();
    fs::create_dir(&default).unwrap();
    fs::write(default.join("authored.txt"), "keep this").unwrap();
    let before = snapshot(source.parent().unwrap());
    let report = check(&source, true);
    assert!(report.ok, "{:?}", report.diagnostics);
    assert_eq!(report.pages, Some(2));
    assert_eq!(report.exit_code, 0);
    assert!(report.diagnostics.is_empty());
    assert_eq!(snapshot(source.parent().unwrap()), before);
    assert_eq!(
        build(&source, &out, BuildOptions::default()).unwrap().pages,
        report.pages.unwrap()
    );
    assert!(
        build(&source, &default, BuildOptions::default()).is_err(),
        "check does not promise destination ownership"
    );
}

#[test]
fn all_reference_errors_are_reported_with_source_files_and_stable_codes() {
    let (_temp, source, out) = fixture();
    fs::write(
        source.join("README.md"),
        "# Home\n\n[Missing](missing.md)\n\n![Missing](absent.png)\n\n<span>Literal</span>\n",
    )
    .unwrap();
    fs::write(
        source.join("next.md"),
        "# Next\n\n[Bad anchor](README.md#absent)\n\n[Other](other.md)\n",
    )
    .unwrap();
    let before = snapshot(source.parent().unwrap());
    let report = check(&source, false);
    assert!(!report.ok);
    assert_eq!(report.exit_code, 3);
    assert_eq!(report.pages, None);
    let errors: Vec<_> = report
        .diagnostics
        .iter()
        .filter(|d| d.severity == Severity::Error)
        .collect();
    assert_eq!(errors.len(), 4);
    assert!(errors.iter().all(|d| d.code == Code::Reference));
    assert_eq!(
        errors.iter().map(|d| d.file.as_deref()).collect::<Vec<_>>(),
        [
            Some("README.md"),
            Some("README.md"),
            Some("next.md"),
            Some("next.md")
        ]
    );
    let json = serde_json::to_value(&report).unwrap();
    assert_eq!(json["schema_version"], 1);
    assert_eq!(json["target"], "portable");
    assert_eq!(json["diagnostics"][0]["code"], "raw-html");
    assert_eq!(json["diagnostics"][1]["severity"], "error");
    assert!(json.get("exit_code").is_none());
    assert!(build(&source, &out, BuildOptions::default()).is_err());
    assert_eq!(snapshot(source.parent().unwrap()), before);
}

#[test]
fn malformed_documents_are_collected_before_reference_resolution() {
    let (_temp, source, _out) = fixture();
    fs::write(
        source.join("README.md"),
        "---\ntitle: []\n---\n[Missing](absent.md)\n",
    )
    .unwrap();
    fs::write(
        source.join("bad.md"),
        "# Duplicate {#same}\n\n## Again {#same}\n",
    )
    .unwrap();
    let report = check(&source, false);
    assert_eq!(report.diagnostics.len(), 2);
    assert!(report.diagnostics.iter().all(|d| d.code == Code::Markdown));
    assert!(report.diagnostics.iter().all(|d| d.file.is_some()));
    assert_eq!(report.exit_code, 3);
}

#[test]
fn strict_warnings_keep_their_severity_and_do_not_replace_existing_output() {
    let (_temp, source, out) = fixture();
    fs::write(
        source.join("README.md"),
        "# Home\n\n<span>Literal HTML</span>\n",
    )
    .unwrap();
    let built = build(&source, &out, BuildOptions::default()).unwrap();
    assert_eq!(built.warnings.len(), 1);
    let before = snapshot(source.parent().unwrap());
    let ordinary = check(&source, false);
    let strict = check(&source, true);
    assert!(ordinary.ok);
    assert!(!strict.ok);
    assert_eq!(strict.exit_code, 3);
    assert_eq!(strict.pages, Some(1));
    assert_eq!(strict.diagnostics[0].severity, Severity::Warning);
    assert_eq!(
        serde_json::to_value(ordinary.diagnostics).unwrap(),
        serde_json::to_value(strict.diagnostics).unwrap()
    );
    assert_eq!(snapshot(source.parent().unwrap()), before);
}

#[test]
fn config_discovery_and_io_failures_remain_structured() {
    let (_temp, source, _out) = fixture();
    fs::write(source.join("zel.yaml"), "nav: [{page: missing.md}]").unwrap();
    let report = check(&source, false);
    assert_eq!(report.diagnostics[0].code, Code::Configuration);
    assert_eq!(report.diagnostics[0].file.as_deref(), Some("zel.yaml"));
    fs::remove_file(source.join("zel.yaml")).unwrap();
    fs::write(source.join("unsupported.mdx"), "# MDX").unwrap();
    let report = check(&source, false);
    assert_eq!(report.diagnostics[0].code, Code::Source);
    assert_eq!(
        report.diagnostics[0].file.as_deref(),
        Some("unsupported.mdx")
    );
    let missing = check(&source.join("absent"), true);
    assert!(!missing.ok);
    assert_eq!(missing.exit_code, 1);
    assert_eq!(missing.diagnostics.len(), 1);
    assert_eq!(missing.diagnostics[0].code, Code::Source);
    assert!(!check(&source.join("README.md"), false).ok);
}

#[test]
fn virtual_output_collisions_fail_check_and_preserve_previous_builds() {
    for files in [
        vec!["index.md", "index.markdown"],
        vec!["guide.md", "guide.html/next.md"],
    ] {
        let (_temp, source, out) = fixture();
        build(&source, &out, BuildOptions::default()).unwrap();
        let previous = snapshot(&out);
        for path in files {
            let path = source.join(path);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, "# Conflict\n").unwrap();
        }
        let report = check(&source, false);
        assert!(!report.ok);
        assert_eq!(report.diagnostics[0].code, Code::OutputPath);
        assert!(build(&source, &out, BuildOptions::default()).is_err());
        assert_eq!(snapshot(&out), previous);
    }
}
