use camino::{Utf8Path, Utf8PathBuf};
use std::fs;
use zellij_docs::{BuildOptions, Code, build, check};

fn fixture() -> (tempfile::TempDir, Utf8PathBuf, Utf8PathBuf) {
    let temp = tempfile::tempdir().unwrap();
    let root = Utf8Path::from_path(temp.path()).unwrap();
    let source = root.join("Math 日本");
    fs::create_dir_all(source.join("guide")).unwrap();
    fs::write(source.join("zel.yaml"), "markdown:\n  math: true\n").unwrap();
    let out = root.join("site");
    (temp, source, out)
}

#[test]
fn equations_render_at_build_time_and_keep_heading_search_and_link_text() {
    let (_temp, source, out) = fixture();
    fs::write(
        source.join("index.md"),
        "# Math\n\n[Energy](guide/energy.md#energy-emc2)\n",
    )
    .unwrap();
    fs::write(
        source.join("guide/energy.md"),
        r"# Energy $E=mc^2$

Inline $\frac{1}{2} + \sqrt{x}$.

$$
\begin{pmatrix}a & b \\ c & d\end{pmatrix}
$$

`$\notacommand$` and escaped \$5 remain text.

```latex
$\notacommand$
```

[Home](../index.md)

![An equation $x^2$](../symbol.svg)
",
    )
    .unwrap();
    fs::write(
        source.join("symbol.svg"),
        "<svg xmlns=\"http://www.w3.org/2000/svg\"/>",
    )
    .unwrap();
    let report = check(&source, true);
    assert!(report.ok, "{:?}", report.diagnostics);
    assert!(!out.exists());
    assert!(
        build(&source, &out, BuildOptions::default())
            .unwrap()
            .warnings
            .is_empty()
    );
    let html = fs::read_to_string(out.join("guide/energy.html")).unwrap();
    assert!(html.contains("<mfrac>"));
    assert!(html.contains("<msqrt>"));
    assert!(html.contains("<mtable"));
    assert!(html.contains("id=\"energy-emc2\""));
    assert!(
        html.contains("Energy E=mc^2"),
        "title/TOC retain equation source"
    );
    assert!(html.contains("alt=\"An equation x^2\""));
    assert!(html.contains("../_zellij/math/math.css"));
    assert!(html.contains("<annotation encoding=\"application/x-tex\">"));
    assert!(out.join("_zellij/math/STIXTwoMath-Regular.woff2").is_file());
    assert!(out.join("_zellij/math/OFL.txt").is_file());
    assert!(
        fs::read_to_string(out.join("_zellij/search.js"))
            .unwrap()
            .contains("E=mc^2")
    );
}

#[test]
fn math_is_opt_in_and_unused_sites_have_no_math_assets() {
    let (_temp, source, out) = fixture();
    let body = "# Prices\n\nFrom $5 to $10. $\\unknown$ stays literal.\n";
    fs::write(source.join("index.md"), body).unwrap();
    for config in ["{}", "markdown: {}", "markdown: {math: false}"] {
        fs::write(source.join("zel.yaml"), config).unwrap();
        assert!(check(&source, true).ok);
        build(&source, &out, BuildOptions::default()).unwrap();
        let html = fs::read_to_string(out.join("index.html")).unwrap();
        assert!(html.contains("From $5 to $10."));
        assert!(!html.contains("<math"));
        assert!(!html.contains("_zellij/math/"));
        assert!(!out.join("_zellij/math").exists());
    }
    fs::write(source.join("zel.yaml"), "markdown: {math: true}").unwrap();
    fs::write(source.join("index.md"), "# No equations\n").unwrap();
    build(&source, &out, BuildOptions::default()).unwrap();
    assert!(!out.join("_zellij/math").exists());
}

#[test]
fn invalid_equations_fail_check_and_preserve_the_last_export() {
    let (_temp, source, out) = fixture();
    fs::write(source.join("index.md"), "# Original\n\n$x^2$\n").unwrap();
    build(&source, &out, BuildOptions::default()).unwrap();
    let before = fs::read(out.join("index.html")).unwrap();
    for formula in [
        r"\frac{1}",
        r"\notacommand{x}",
        r"\href{javascript:alert(1)}{x}",
        r"\newcommand{\forever}{\forever}\forever",
        r"\eqref{missing}",
    ] {
        let body = format!("# Broken\n\n${formula}$\n");
        fs::write(source.join("index.md"), &body).unwrap();
        let report = check(&source, false);
        assert!(!report.ok, "{formula}");
        assert_eq!(report.exit_code, 3);
        assert_eq!(report.diagnostics[0].code, Code::Rendering);
        assert_eq!(report.diagnostics[0].file.as_deref(), Some("index.md"));
        assert!(report.diagnostics[0].message.contains("math expression"));
        assert!(build(&source, &out, BuildOptions::default()).is_err());
        assert_eq!(fs::read(out.join("index.html")).unwrap(), before);
        assert_eq!(fs::read_to_string(source.join("index.md")).unwrap(), body);
    }
}

#[test]
fn equation_text_and_annotations_cannot_inject_html() {
    let (_temp, source, out) = fixture();
    fs::write(
        source.join("index.md"),
        r"# Escaping

$\text{</mtext><img src=x onerror=alert(1)>}$

$\operatorname{<script>alert(1)</script>}(x)$

$a < b$
",
    )
    .unwrap();
    build(&source, &out, BuildOptions::default()).unwrap();
    let html = fs::read_to_string(out.join("index.html")).unwrap();
    assert!(!html.contains("<img src=x"));
    assert!(!html.contains("<script>alert"));
    assert!(html.contains("&lt;"));
    assert!(html.contains("<math"));
}

#[test]
fn math_configuration_is_strictly_typed() {
    let (_temp, source, out) = fixture();
    fs::write(source.join("index.md"), "# Home\n").unwrap();
    for config in [
        "markdown: {math: 'true'}",
        "markdown: {math: 1}",
        "markdown: {maths: true}",
        "markdown: false",
    ] {
        fs::write(source.join("zel.yaml"), config).unwrap();
        let report = check(&source, false);
        assert!(!report.ok);
        assert_eq!(report.diagnostics[0].code, Code::Configuration);
        assert!(build(&source, &out, BuildOptions::default()).is_err());
        assert!(!out.exists());
    }
}

#[test]
fn equation_labels_have_distinct_namespaces_and_math_assets_are_removed_when_disabled() {
    let (_temp, source, out) = fixture();
    let expression = r"$$\begin{equation}x=1\label{same}\end{equation}\quad\eqref{same}$$";
    fs::write(
        source.join("index.md"),
        format!("# Labels\n\n{expression}\n\n{expression}\n"),
    )
    .unwrap();
    build(&source, &out, BuildOptions::default()).unwrap();
    let html = fs::read_to_string(out.join("index.html")).unwrap();
    assert!(html.contains("id=\"_zel-math-1-same\""));
    assert!(html.contains("id=\"_zel-math-2-same\""));
    assert!(html.contains("href=\"#_zel-math-1-same\""));
    assert!(html.contains("href=\"#_zel-math-2-same\""));
    assert!(out.join("_zellij/math").is_dir());
    fs::write(source.join("zel.yaml"), "markdown: {math: false}").unwrap();
    build(&source, &out, BuildOptions::default()).unwrap();
    assert!(!out.join("_zellij/math").exists());
    assert!(
        !fs::read_to_string(out.join("index.html"))
            .unwrap()
            .contains("<math")
    );
}
