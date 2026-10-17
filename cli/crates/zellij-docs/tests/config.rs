use camino::{Utf8Path, Utf8PathBuf};
use std::fs;
use zellij_docs::{BuildOptions, build};

fn fixture() -> (tempfile::TempDir, Utf8PathBuf, Utf8PathBuf) {
    let temp = tempfile::tempdir().unwrap();
    let root = Utf8Path::from_path(temp.path()).unwrap();
    let source = root.join("docs");
    fs::create_dir_all(source.join("guide")).unwrap();
    for (path, text) in [
        ("README.md", "# Home\n"),
        ("guide/a.md", "# Alpha\n"),
        ("guide/z.md", "# Zulu\n"),
        ("unlisted.md", "# Unlisted\n\nFind me with search.\n"),
        (
            "company logo.svg",
            "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"80\" height=\"40\"/>",
        ),
    ] {
        fs::write(source.join(path), text).unwrap();
    }
    let output = root.join("site");
    (temp, source, output)
}
const CONFIG: &str = r#"
name: Company & docs
theme: forest
brand:
  logo:
    src: company logo.svg
    alt: Company logo
    href: https://example.com/company?one=1&two=2#about
nav:
  - page: README.md
    label: Overview
  - label: 'Guide <group>'
    children:
      - page: guide/z.md
        label: Start here
      - label: More
        children:
          - page: guide/a.md
  - label: Company website
    href: https://example.com/
"#;

#[test]
fn configured_navigation_branding_and_paging_survive_nested_pages() {
    let (_temp, source, out) = fixture();
    fs::write(source.join("zel.yaml"), CONFIG).unwrap();
    let report = build(&source, &out, BuildOptions::default()).unwrap();
    assert_eq!(report.pages, 4);
    assert!(report.warnings.is_empty());
    let home = fs::read_to_string(out.join("index.html")).unwrap();
    assert!(home.contains("Company &amp; docs"));
    assert!(home.contains("data-zellij-theme=\"forest\""));
    assert!(home.contains("href=\"guide/z.html\"><small>Next</small><span>Start here"));
    assert!(home.contains("Guide &lt;group&gt;"));
    assert!(!out.join("zel.yaml").exists());
    assert_eq!(
        fs::read(source.join("company logo.svg")).unwrap(),
        fs::read(out.join("company logo.svg")).unwrap()
    );
    let zulu = fs::read_to_string(out.join("guide/z.html")).unwrap();
    assert!(zulu.contains("href=\"../index.html\"><small>Previous</small><span>Overview"));
    assert!(zulu.contains("href=\"../guide/a.html\"><small>Next</small><span>Alpha"));
    assert!(zulu.contains("src=\"../company%20logo.svg\" alt=\"Company logo\""));
    assert!(zulu.contains("href=\"https://example.com/company?one=1&amp;two=2#about\""));
    assert!(zulu.contains("href=\"../guide/z.html\" aria-current=\"page\">Start here"));
    let unlisted = fs::read_to_string(out.join("unlisted.html")).unwrap();
    assert!(!unlisted.contains("rel=\"next\"") && !unlisted.contains("rel=\"prev\""));
    assert!(!unlisted.contains("href=\"unlisted.html\""));
    assert!(
        fs::read_to_string(out.join("_zellij/search.js"))
            .unwrap()
            .contains("Find me with search")
    );
    assert_eq!(fs::read_to_string(source.join("zel.yaml")).unwrap(), CONFIG);
}

#[test]
fn cli_overrides_and_removing_configuration_restore_defaults() {
    let (_temp, source, out) = fixture();
    fs::write(source.join("zel.yaml"), CONFIG).unwrap();
    build(
        &source,
        &out,
        BuildOptions {
            name: Some("Override"),
            theme: Some("cupertino"),
        },
    )
    .unwrap();
    let home = fs::read_to_string(out.join("index.html")).unwrap();
    assert!(home.contains("Home · Override"));
    assert!(home.contains("data-zellij-theme=\"cupertino\""));
    fs::remove_file(source.join("zel.yaml")).unwrap();
    build(&source, &out, BuildOptions::default()).unwrap();
    let home = fs::read_to_string(out.join("index.html")).unwrap();
    assert!(home.contains("Home · docs"));
    assert!(home.contains("rel=\"next\" href=\"guide/a.html\""));
    assert!(!home.contains("brand-logo"));
    assert!(!out.join("company logo.svg").exists());
}

#[test]
fn invalid_configuration_preserves_the_previous_export() {
    let (_temp, source, out) = fixture();
    fs::create_dir_all(source.join("assets/node_modules")).unwrap();
    fs::copy(
        source.join("company logo.svg"),
        source.join("assets/node_modules/logo.svg"),
    )
    .unwrap();
    build(&source, &out, BuildOptions::default()).unwrap();
    let previous = fs::read(out.join("index.html")).unwrap();
    for config in [
        "unknown: true",
        "name: []",
        "name: 42",
        "name: true",
        "name: ' '",
        "name: One\nname: Two",
        "theme: missing",
        "nav: nope",
        "nav: [README.md]",
        "nav: [{page: missing.md}]",
        "nav: [{page: ../README.md}]",
        "nav: [{page: README.md}, {page: README.md}]",
        "nav: [{page: README.md, href: 'https://example.com'}]",
        "nav: [{label: ' ', page: README.md}]",
        "nav: [{page: 12}]",
        "nav: [{label: Group, children: []}]",
        "nav: [{children: [{page: README.md}]}]",
        "nav: [{label: Empty}]",
        "nav: [{href: 'https://example.com'}]",
        "nav: [{label: Bad, href: 'javascript:alert(1)'}]",
        "nav: [{label: Bad, href: '//example.com'}]",
        "nav: [{label: Bad, href: 'https:'}]",
        "nav: [{label: Bad, href: 'https://example.com/ bad'}]",
        "nav: [{page: README.md, typo: true}]",
        "brand: {logo: {src: 'https://example.com/logo.png', alt: Logo}}",
        "brand: {logo: {src: '../logo.svg', alt: Logo}}",
        "brand: {logo: {src: 'missing.svg', alt: Logo}}",
        "brand: {logo: {src: 'assets/node_modules/logo.svg', alt: Logo}}",
        "brand: {logo: {src: 'README.md', alt: Logo}}",
        "brand: {logo: {src: 'company logo.svg', alt: ''}}",
        "brand: {logo: {src: 'company logo.svg', alt: Logo, href: 'file:///tmp/a'}}",
    ] {
        fs::write(source.join("zel.yaml"), config).unwrap();
        let result = build(&source, &out, BuildOptions::default());
        assert!(result.is_err(), "unexpectedly accepted {config}");
        assert_eq!(
            fs::read(out.join("index.html")).unwrap(),
            previous,
            "{config}"
        );
        assert_eq!(fs::read_to_string(source.join("zel.yaml")).unwrap(), config);
    }
}

#[test]
fn generated_home_follows_explicit_navigation_and_empty_navigation_is_allowed() {
    let (_temp, source, out) = fixture();
    fs::remove_file(source.join("README.md")).unwrap();
    fs::write(
        source.join("zel.yaml"),
        "nav:\n  - page: guide/z.md\n  - page: guide/a.md\n",
    )
    .unwrap();
    build(&source, &out, BuildOptions::default()).unwrap();
    let home = fs::read_to_string(out.join("index.html")).unwrap();
    let list = home
        .split("<ul class=\"document-list\">")
        .nth(1)
        .unwrap()
        .split("</ul>")
        .next()
        .unwrap();
    assert!(list.find("Zulu").unwrap() < list.find("Alpha").unwrap());
    assert!(!list.contains("Unlisted"));
    fs::write(source.join("zel.yaml"), "nav: []").unwrap();
    build(&source, &out, BuildOptions::default()).unwrap();
    let page = fs::read_to_string(out.join("guide/a.html")).unwrap();
    assert!(!page.contains("aria-current=\"page\">Alpha</a>"));
    assert!(!page.contains("rel=\"next\"") && !page.contains("rel=\"prev\""));
    assert!(out.join("guide/z.html").is_file());
}

#[cfg(unix)]
#[test]
fn configuration_and_logo_links_are_rejected() {
    let (_temp, source, out) = fixture();
    let external = source.parent().unwrap().join("external.yaml");
    fs::write(&external, CONFIG).unwrap();
    std::os::unix::fs::symlink(&external, source.join("zel.yaml")).unwrap();
    assert!(zellij_docs::has_config(&source));
    assert!(build(&source, &out, BuildOptions::default()).is_err());
    assert!(!out.exists());
    fs::remove_file(source.join("zel.yaml")).unwrap();
    fs::write(
        source.join("zel.yaml"),
        "brand: {logo: {src: linked.svg, alt: Logo}}",
    )
    .unwrap();
    std::os::unix::fs::symlink(source.join("company logo.svg"), source.join("linked.svg")).unwrap();
    assert!(build(&source, &out, BuildOptions::default()).is_err());
    assert!(!out.exists());
}
