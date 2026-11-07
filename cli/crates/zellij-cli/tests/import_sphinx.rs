//! Real CLI import acceptance; no Python, containers or browser is executed.
use std::{
    fs,
    path::Path,
    process::{Command, Output},
};
fn write(root: &Path, name: &str, text: impl AsRef<[u8]>) {
    let p = root.join(name);
    fs::create_dir_all(p.parent().unwrap()).unwrap();
    fs::write(p, text).unwrap();
}
fn read(root: &Path, name: &str) -> String {
    fs::read_to_string(root.join(name)).unwrap()
}
fn run(source: &Path, out: &Path, args: &[&str], code: i32) -> Output {
    let r = Command::new(env!("CARGO_BIN_EXE_zel"))
        .arg("cv")
        .arg(source)
        .arg("--out")
        .arg(out)
        .args(args)
        .output()
        .unwrap();
    assert_eq!(r.status.code(), Some(code), "{r:?}");
    r
}
fn report(r: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&r.stdout),
        String::from_utf8_lossy(&r.stderr)
    )
}
fn basic(s: &Path) {
    write(s, "conf.py", "project = 'Manual'\n");
    write(s, "index.rst", "Welcome\n=======\n");
}

#[test]
fn tables_preserve_cells_headers_blocks_unicode_references_and_assets() {
    let t = tempfile::tempdir().unwrap();
    let s = t.path().join("source");
    let o = t.path().join("out");
    basic(&s);
    let source = include_str!("../../../../scripts/fixtures/sphinx-tables.rst");
    write(&s, "index.rst", source);
    write(
        &s,
        "details.rst",
        ".. _install:\n\nInstallation\n============\n",
    );
    write(&s, "notes.txt", "preserved download");
    write(&s, "media/logo.svg", "<svg>preserved image</svg>");
    let r = run(&s, &o, &["--layout", "site"], 0);
    let c = o.join("content");
    let body = read(&c, "01-guide/index.mdx");
    assert_eq!(body.matches("<table>").count(), 8);
    assert_eq!(body.matches("<thead>").count(), 4);
    assert_eq!(body.matches("<th>").count(), 17);
    assert_eq!(body.matches("<caption>").count(), 4);
    assert_eq!(body.matches("<th scope=\"row\">").count(), 7);
    for expected in [
        "**Feature** matrix",
        "Café 漢字 e\u{301}",
        "[Details](</guide/details>)",
        "[Installation](</guide/details#install>)",
        "[Notes](</_zellij/assets/imported/docs/notes.txt>)",
        "![Table logo](</_zellij/assets/imported/docs/media/logo.svg>)",
        "**Bold**",
        "*emphasis*",
        "` a|{b} `",
        "&lt;script&gt;literal&lt;/script&gt;",
        "First paragraph.\n\nSecond paragraph.",
        "First item",
        "> **note**",
        ".. include:: /not/executed",
        "First line extends beyond the border.",
        "Continuation with **strong** text.",
        "Empty first cell.",
        "No invented header.",
        "**List** matrix",
        "List first paragraph.\n\nList second paragraph",
        "[List details](</guide/details>)",
        "[List download](</_zellij/assets/imported/docs/notes.txt>)",
        "![List logo](</_zellij/assets/imported/docs/media/logo.svg>)",
        ".. csv-table:: <script>{code}|literal</script>",
        ":header: literal field in code",
        "Empty list row heading.",
        "Quoted \"word\", comma and **bold**.",
        "Second CSV paragraph.",
        "[CSV notes](</_zellij/assets/imported/docs/notes.txt>)",
        "Padded CSV row",
        "漢字│quoted 'word'",
    ] {
        assert!(body.contains(expected), "{expected}: {body}");
    }
    assert_eq!(
        read(&c, "assets/imported/docs/notes.txt"),
        "preserved download"
    );
    assert_eq!(
        read(&c, "assets/imported/docs/media/logo.svg"),
        "<svg>preserved image</svg>"
    );
    assert_eq!(read(&s, "index.rst"), source);
    assert!(!report(&r).contains("HTML/JSX asset and link attributes"));
    let dry = t.path().join("dry");
    run(&s, &dry, &["--dry-run"], 0);
    assert!(!dry.exists());
    let skipped = t.path().join("skip");
    run(&s, &skipped, &["--assets", "skip"], 0);
    assert!(!skipped.join("assets/imported").exists());
}

#[test]
fn directive_separator_preserves_option_looking_literal_code_and_csv_content() {
    let t = tempfile::tempdir().unwrap();
    let source = t.path().join("source");
    let out = t.path().join("out");
    basic(&source);
    write(
        &source,
        "index.rst",
        "Home\n====\n\n.. code-block:: text\n\n   :header: literal\n   :file: /not/read\n   .. include:: /not/read\n\n.. csv-table::\n   :header: First\n            Second\n\n   :strong:`Cell`\n\n.. csv-table::\n\n   :strong:`Body`\n",
    );
    run(&source, &out, &[], 0);
    let body = read(&out, "01-guide/index.mdx");
    assert!(body.contains(":header: literal\n:file: /not/read\n.. include:: /not/read"));
    assert!(body.contains("**Cell**") && body.contains("**Body**"));
    assert_eq!(body.matches("<thead>").count(), 1);
    assert_eq!(body.matches("<th>").count(), 2);
}

#[test]
fn invalid_table_directives_and_external_csv_sources_fail_without_writes() {
    for table in [
        ".. list-table::\n\n   * plain item",
        ".. list-table::\n   :header-rows: 1\n\n   * - only header",
        ".. list-table::\n   :stub-columns: 1\n\n   * - only stub",
        ".. list-table::\n   :header-rows: -1\n\n   * - cell",
        ".. list-table::\n   :header: invalid CSV option\n\n   * - cell",
        ".. csv-table::\n\n   key,\"unclosed",
        ".. csv-table::\n\n   \"closed\"suffix,value",
        ".. csv-table::\n   :header-rows: 2\n\n   key,value",
        ".. csv-table:: Long\n   continued caption\n\n   key,value",
        ".. csv-table::\n   :header: Key,Value\n   key,value",
        ".. csv-table::\n   :header-rows: 0\n   :header-rows: 0\n\n   key,value",
        ".. csv-table::\n   :keepspace: true\n\n   key,value",
        ".. csv-table::\n   :file: ../secret.csv",
        ".. csv-table::\n   :url: http://127.0.0.1:9/private.csv",
        ".. csv-table::\n   :encoding: utf-8\n\n   key,value",
        ".. csv-table::\n   :escape: \\\n\n   key,value",
        ".. csv-table::\n   :widths: 10 20\n\n   key,value",
        ".. csv-table::\n\n   key,.. include:: ../secret.csv",
        ".. list-table::\n\n   * - key\n     - .. raw:: html\n\n          <script>bad</script>",
    ] {
        let t = tempfile::tempdir().unwrap();
        let source = t.path().join("source");
        basic(&source);
        write(t.path(), "secret.csv", "outside-file-sentinel");
        let text = format!("Home\n====\n\n{table}\n");
        write(&source, "index.rst", &text);
        for (index, args) in [vec![], vec!["--dry-run"], vec!["--assets", "skip"]]
            .iter()
            .enumerate()
        {
            let out = t.path().join(format!("out-{index}"));
            let result = run(&source, &out, args, 2);
            assert!(report(&result).contains("index.rst"), "{result:?}");
            assert!(!report(&result).contains("outside-file-sentinel"));
            assert!(!out.exists(), "{table}");
        }
        assert_eq!(read(&source, "index.rst"), text);
    }
}

#[test]
fn list_and_csv_cells_apply_asset_containment_before_all_output_modes() {
    for table in [
        ".. list-table::\n\n   * - :download:`outside <../secret>`",
        ".. csv-table::\n\n   :download:`outside <../secret>`",
    ] {
        for args in [vec![], vec!["--assets", "skip"], vec!["--dry-run"]] {
            let t = tempfile::tempdir().unwrap();
            let source = t.path().join("source");
            let out = t.path().join("out");
            basic(&source);
            write(t.path(), "secret", "outside-file-sentinel");
            write(&source, "index.rst", format!("Home\n====\n\n{table}\n"));
            let result = run(&source, &out, &args, 2);
            assert!(report(&result).contains("escapes"), "{result:?}");
            assert!(!report(&result).contains("outside-file-sentinel"));
            assert!(!out.exists());
        }
    }
}

#[test]
fn nested_list_table_directives_share_the_existing_depth_limit() {
    let t = tempfile::tempdir().unwrap();
    let source = t.path().join("source");
    basic(&source);
    let mut table = "cell".to_string();
    for depth in 1..=17 {
        table = format!(
            ".. list-table::\n\n   * - {}",
            table.replace('\n', "\n       ")
        );
        if depth >= 16 {
            write(&source, "index.rst", format!("Home\n====\n\n{table}\n"));
            let out = t.path().join(format!("out-{depth}"));
            let result = run(&source, &out, &[], if depth == 16 { 0 } else { 2 });
            if depth == 17 {
                assert!(report(&result).contains("nesting exceeds 16"));
                assert!(!out.exists());
            }
        }
    }
}

#[test]
fn unsupported_or_malformed_tables_fail_before_creating_output() {
    for table in [
        "+---+---+\n| a     |\n+---+---+", // column span
        "+---+---+\n| a | b |\n+---+   |\n| c |   |\n+---+---+", // row span
        "+---+---+\n| a | b |",            // missing border
        "+---+---+\n| a | b |\n+===+---+", // mixed divider
        "+---+---+\n| a | b | trailing\n+---+---+", // content outside grid
        "===  ===\nspanning\n--------\n===  ===",
        "===  ===\na    b\n---  ---", // missing equals footer
        "===  ===\n     no preceding row\n===  ===",
        "===  ===\na   xb\n===  ===", // text in gutter
        ".. table:: Caption\n   :align: left\n\n   ===  ===\n   a    b\n   ===  ===",
        ".. table:: Caption\n\n   ===  ===\n   a    b\n   ===  ===\n\n   extra content",
        ".. list-table::\n\n   * - a\n     - b\n   * - missing column",
        ".. csv-table::\n   :file: /private/data",
        "===  ===\na    .. include:: file\n===  ===",
        "+-+\n|\\|\n+-+",
    ] {
        let t = tempfile::tempdir().unwrap();
        let s = t.path().join("s");
        let o = t.path().join("o");
        basic(&s);
        let text = format!("Home\n====\n\n{table}\n");
        write(&s, "index.rst", &text);
        let r = run(&s, &o, &[], 2);
        assert!(report(&r).contains("index.rst"), "{r:?}");
        assert!(!o.exists(), "{table}");
        assert_eq!(read(&s, "index.rst"), text);
    }
}

#[test]
fn table_asset_containment_applies_to_copy_skip_and_dry_runs() {
    for args in [vec![], vec!["--assets", "skip"], vec!["--dry-run"]] {
        let t = tempfile::tempdir().unwrap();
        let s = t.path().join("s");
        let o = t.path().join("o");
        basic(&s);
        write(t.path(), "secret", "outside-file-sentinel");
        write(
            &s,
            "index.rst",
            "Home\n====\n\n===  ===\na    :download:`outside <../secret>`\n===  ===\n",
        );
        let output = run(&s, &o, &args, 2);
        assert!(report(&output).contains("escapes"), "{output:?}");
        assert!(!report(&output).contains("outside-file-sentinel"));
        assert!(!o.exists());
        assert_eq!(read(t.path(), "secret"), "outside-file-sentinel");
    }
}

#[test]
fn nested_tables_enforce_the_depth_limit_even_with_empty_cells() {
    let mut table = "\\ ".to_string();
    for depth in 1..=17 {
        let width = table.lines().map(str::len).max().unwrap();
        let border = format!("+{}+", "-".repeat(width + 2));
        table = format!(
            "{border}\n{}\n{border}",
            table
                .lines()
                .map(|line| format!("| {line:width$} |"))
                .collect::<Vec<_>>()
                .join("\n")
        );
        if depth < 16 {
            continue;
        }
        let t = tempfile::tempdir().unwrap();
        let s = t.path().join("s");
        let o = t.path().join("o");
        basic(&s);
        write(&s, "index.rst", format!("Home\n====\n\n{table}\n"));
        let output = run(&s, &o, &[], if depth == 16 { 0 } else { 2 });
        if depth == 16 {
            assert_eq!(
                read(&o, "01-guide/index.mdx").matches("<table>").count(),
                16
            );
        } else {
            assert!(report(&output).contains("nesting exceeds 16"));
            assert!(!o.exists());
        }
    }
}

#[test]
fn nested_lists_preserve_numbering_blocks_references_assets_and_empty_items() {
    let t = tempfile::tempdir().unwrap();
    let source = t.path().join("source");
    let out = t.path().join("out");
    basic(&source);
    write(
        &source,
        "index.rst",
        "Home\n====\n\n.. toctree::\n\n   lists\n   details\n",
    );
    let fixture = include_str!("../../../../scripts/fixtures/sphinx-lists.rst");
    write(&source, "lists.rst", fixture);
    write(
        &source,
        "details.rst",
        ".. _install:\n\nInstallation\n============\n",
    );
    write(&source, "notes.txt", "Nested download bytes.\n");
    write(&source, "media/logo.svg", "<svg>nested image</svg>");
    run(&source, &out, &[], 0);
    let body = read(&out, "01-guide/lists.mdx");
    for text in [
        "<ol start=\"9\">",
        "<ol start=\"7\">",
        "<ol start=\"3\">",
        "<ol start=\"4\">",
        "<ol start=\"2\">",
        "Prepare **workspace**.\nWrapped parent text.",
        "Nested searchable paragraph.",
        "[Nested details](</guide/details>)",
        "[Installation](</guide/details#install>)",
        "[Nested notes](</_zellij/assets/imported/docs/notes.txt>)",
        "![Nested logo](</_zellij/assets/imported/docs/media/logo.svg>)",
        "> **note**",
        "Keep **original** content.",
        ":header: literal option\n- literal bullet\n#. literal number\n.. include:: /not/executed",
        "Unicode parent Café 漢字.",
        "Unicode leaf.",
        "- literal block bullet\n9. literal block number",
        "Table child **strong**.",
        "<li>\n\n\n</li>",
    ] {
        assert!(body.contains(text), "{text}: {body}");
    }
    assert_eq!(body.matches("<ul>").count(), 6);
    assert_eq!(body.matches("<ol start=").count(), 5);
    assert_eq!(body.matches("<table>").count(), 2);
    assert_eq!(
        read(&out, "assets/imported/docs/notes.txt"),
        "Nested download bytes.\n"
    );
    assert_eq!(
        read(&out, "assets/imported/docs/media/logo.svg"),
        "<svg>nested image</svg>"
    );
    assert_eq!(read(&source, "lists.rst"), fixture);
    let dry = t.path().join("dry");
    run(&source, &dry, &["--dry-run"], 0);
    assert!(!dry.exists());
    let skip = t.path().join("skip");
    run(&source, &skip, &["--assets", "skip"], 0);
    assert!(!skip.join("assets/imported").exists());
}

#[test]
fn malformed_or_unsupported_list_content_fails_before_output() {
    for list in [
        "paragraph\n- missing blank",
        "- parent\n  - missing nested blank",
        "-   parent\n  too little indentation",
        "- parent\nnot separated",
        "- parent\n\n   misaligned continuation",
        "2147483648. too large",
        "- parent\n\n  a. alphabetic",
        "- parent\n\n  (iv) roman",
        "2147483647. last\n#. overflow",
        "- parent\n\n  .. include:: ../secret",
        "- parent\n\n  .. toctree::\n\n     page",
        "- parent\n\n  .. _label:\n\n  Heading\n  =======",
        "- parent\n\n  .. raw:: html\n\n     <script>run()</script>",
    ] {
        let t = tempfile::tempdir().unwrap();
        let source = t.path().join("s");
        let out = t.path().join("o");
        basic(&source);
        let text = format!("Home\n====\n\n{list}\n");
        write(&source, "index.rst", &text);
        run(&source, &out, &[], 2);
        assert!(!out.exists(), "{list}");
        assert_eq!(read(&source, "index.rst"), text);
    }
}

#[test]
fn nested_list_assets_obey_containment_in_every_output_mode() {
    for args in [vec![], vec!["--assets", "skip"], vec!["--dry-run"]] {
        let t = tempfile::tempdir().unwrap();
        let source = t.path().join("s");
        let out = t.path().join("o");
        basic(&source);
        write(t.path(), "secret", "outside-list-sentinel");
        write(
            &source,
            "index.rst",
            "Home\n====\n\n- parent\n\n  1. child\n\n     * :download:`outside <../secret>`\n",
        );
        let result = run(&source, &out, &args, 2);
        assert!(report(&result).contains("escapes"), "{result:?}");
        assert!(!report(&result).contains("outside-list-sentinel"));
        assert!(!out.exists());
        assert_eq!(read(t.path(), "secret"), "outside-list-sentinel");
    }
}

#[test]
fn lists_share_the_table_and_admonition_depth_limit() {
    for outer_note in [false, true] {
        let limit = if outer_note { 15 } else { 16 };
        let mut body = String::new(); // Empty innermost items still consume depth.
        for depth in 1..=limit + 1 {
            body = if depth == 1 {
                "-\n".into()
            } else {
                format!(
                    "- parent\n\n{}",
                    body.lines().map(|l| format!("  {l}\n")).collect::<String>()
                )
            };
            if depth < limit {
                continue;
            }
            let t = tempfile::tempdir().unwrap();
            let source = t.path().join("s");
            let out = t.path().join("o");
            basic(&source);
            let content = if outer_note {
                format!(
                    ".. note::\n\n{}",
                    body.lines()
                        .map(|l| format!("   {l}\n"))
                        .collect::<String>()
                )
            } else {
                body.clone()
            };
            write(&source, "index.rst", format!("Home\n====\n\n{content}\n"));
            let result = run(&source, &out, &[], if depth == limit { 0 } else { 2 });
            if depth == limit {
                assert_eq!(
                    read(&out, "01-guide/index.mdx").matches("<ul>").count(),
                    depth
                );
            } else {
                assert!(report(&result).contains("nesting exceeds 16"));
                assert!(!out.exists());
            }
        }
    }
}

#[test]
fn converts_navigation_labels_links_code_images_and_source_urls() {
    let t = tempfile::tempdir().unwrap();
    let s = t.path().join("source");
    let o = t.path().join("out");
    write(
        &s,
        "conf.py",
        "project='Café: Manual'\nhtml_baseurl='https://old.example.test/manual/'\nextensions=[]\nexclude_patterns=['_build', 'private']\nhtml_static_path=['_static']\n",
    );
    let home = ".. _welcome:\n\nWelcome\n=======\n\n.. toctree::\n   :caption: Learn\n   :maxdepth: 2\n\n   API navigation <guide/index>\n   self\n   External <https://example.test>\n\nRead :doc:`guide/index` or :ref:`setup`.\n\n`File <guide/index.rst#setup>`_ and `Website <https://old.example.test/manual/guide/index.html?q=1#setup>`_.\n\n.. image:: /_static/logo.svg\n   :alt: Site logo\n\n:download:`Report <media/Q3 Report.pdf>`\n";
    write(&s, "index.rst", home);
    write(
        &s,
        "guide/index.rst",
        ".. _guide:\n\nAPI Reference\n=============\n\n.. toctree::\n   :hidden:\n\n   next\n\n.. _setup:\n\nInstallation\n------------\n\nUse **strong**, *emphasis* and ``a_{b}``.\n\n:ref:`Back <welcome>` and :doc:`/index`.\n\n.. note:: Keep the source.\n\n   Use :doc:`next`.\n\n.. code-block:: python\n\n   print('{safe}')\n   .. include:: /private/secret\n   :ref:`literal`\n\nExample::\n\n   ```\n   # literal heading\n\n.. image:: /_static/logo.svg\n\n:download:`Root report </media/Q3 Report.pdf>`\n\n`Home <../index.html#welcome>`_\n",
    );
    write(
        &s,
        "guide/next.rst",
        "Next steps\n==========\n\n- First\n- Second\n\n#. One\n#. Two\n",
    );
    write(&s, "orphan.rst", "Not published\n=============\n");
    write(&s, "private/hidden.rst", ".. unsupported::\n");
    write(&s, "_static/logo.svg", "<svg>logo</svg>");
    write(&s, "media/Q3 Report.pdf", [0, 255, 254, 10]);
    let r = run(&s, &o, &["--layout", "site"], 0);
    let c = o.join("content");
    let h = read(&c, "01-guide/index.mdx");
    for text in [
        "<div id=\"welcome\"",
        "[API Reference](</guide/guide>)",
        "[Installation](</guide/guide#setup>)",
        "[File](</guide/guide#setup>)",
        "[Website](</guide/guide?q=1#setup>)",
        "![Site logo](</_zellij/assets/imported/docs/_static/logo.svg>)",
        "[Report](</_zellij/assets/imported/docs/media/Q3%20Report.pdf>)",
    ] {
        assert!(h.contains(text), "{text}: {h}");
    }
    let g = read(&c, "01-guide/guide/index.mdx");
    for text in [
        "<h2 id=\"setup\">Installation</h2>",
        "[Back](</guide#welcome>)",
        "[Welcome](</guide>)",
        "[Next steps](</guide/guide/next>)",
        ".. include:: /private/secret",
        ":ref:`literal`",
        "````text\n```",
        "[Home](</guide#welcome>)",
        "![](</_zellij/assets/imported/docs/_static/logo.svg>)",
        "[Root report](</_zellij/assets/imported/docs/media/Q3%20Report.pdf>)",
    ] {
        assert!(g.contains(text), "{text}: {g}");
    }
    assert!(
        !g.contains("- [Next steps]"),
        "hidden toctree must not render inline"
    );
    let nav = read(&c, "01-guide/_sidebar.yaml");
    assert!(
        nav.contains("Learn") && nav.contains("API navigation") && nav.contains("guide/next"),
        "{nav}"
    );
    assert!(!c.join("01-guide/orphan.mdx").exists());
    assert!(!c.join("01-guide/private").exists());
    assert_eq!(
        fs::read(c.join("assets/imported/docs/media/Q3 Report.pdf")).unwrap(),
        [0, 255, 254, 10]
    );
    assert_eq!(read(&s, "index.rst"), home);
    assert!(report(&r).contains("orphan.rst"));
    assert!(report(&r).contains("external link"));
}

#[test]
fn accepts_master_document_suffix_forms_bom_crlf_and_named_links() {
    for suffix in ["'.rst'", "['.rst']", "{'.rst':'restructuredtext'}"] {
        let t = tempfile::tempdir().unwrap();
        let s = t.path().join("s");
        let o = t.path().join("o");
        write(
            &s,
            "conf.py",
            format!(
                "\u{feff}project='Docs'\r\nmaster_doc='contents'\r\nsource_suffix={suffix}\r\n"
            ),
        );
        write(
            &s,
            "contents.rst",
            "\u{feff}===========\r\nCafé Manual\r\n===========\r\n\r\nRead Website_ and `Café Manual`_. Visit Website_.\r\n\r\n.. _Website: https://example.test\r\n",
        );
        run(&s, &o, &[], 0);
        let page = read(&o, "01-guide/contents.mdx");
        assert!(
            page.contains("Visit [Website](<https://example.test>)."),
            "{page}"
        );
        assert!(page.contains("[Café Manual](<#cafe-manual>)"), "{page}");
        assert!(!o.join("01-guide/contents.rst.mdx").exists());
    }
}

#[test]
fn config_python_is_never_executed_and_unsupported_settings_fail_atomically() {
    for conf in [
        "import os\nos.system('touch marker')",
        "project = __import__('os').getcwd()",
        "project = 'x'\nexec('anything')",
        "extensions=['sphinx.ext.autodoc']",
        "source_suffix={'.md':'markdown'}",
        "exclude_patterns=['private/*']",
        "include_patterns=[]",
        "rst_prolog='hidden'",
        "html_file_suffix='.htm'",
        "root_doc='a'\nmaster_doc='b'",
        "templates_path=['../outside']",
        "source_encoding='latin1'",
        "language='en'",
        "language='fr'",
    ] {
        let t = tempfile::tempdir().unwrap();
        let s = t.path().join("s");
        let o = t.path().join("o");
        basic(&s);
        write(&s, "conf.py", conf);
        run(&s, &o, &[], 2);
        assert!(!o.exists(), "{conf}");
        assert!(!s.join("marker").exists());
    }
}

#[test]
fn unsupported_rst_fails_before_any_output_without_interpreting_literal_code() {
    for text in [
        ".. include:: ../secret",
        ".. raw:: html\n\n   <script>run()</script>",
        ".. automodule:: app",
        ".. toctree::\n   :glob:\n\n   *",
        ".. toctree::\n   :numbered:\n\n   self",
        ".. image:: logo.svg\n   :width: 20",
        ".. |replace| replace:: secret",
        ":orphan:",
        ":py:func:`run`",
        "|substitution|",
        "Term\n   definition",
        "+---+---+\n| a     |\n+---+---+",
        ".. _dangling:\n\nParagraph",
        ":ref:`missing`",
        "`default role`",
        ".. code-block:: python\n   :linenos:\n\n   pass",
        ".. note::\n\n   .. toctree::\n\n      hidden",
    ] {
        let t = tempfile::tempdir().unwrap();
        let s = t.path().join("s");
        let o = t.path().join("o");
        basic(&s);
        write(&s, "index.rst", format!("Home\n====\n\n{text}\n"));
        run(&s, &o, &[], 2);
        assert!(!o.exists(), "{text}");
    }
}

#[test]
fn cycles_missing_excluded_unlisted_and_escaping_documents_are_rejected() {
    for target in [
        "index",
        "missing",
        "../outside",
        "/../../outside",
        "private/page",
        "https:bad",
        "file?x",
        "file%2fsecret",
    ] {
        let t = tempfile::tempdir().unwrap();
        let s = t.path().join("s");
        let o = t.path().join("o");
        basic(&s);
        write(
            &s,
            "conf.py",
            "project='Docs'\nexclude_patterns=['private']\n",
        );
        write(&s, "private/page.rst", "Hidden\n======\n");
        write(
            &s,
            "index.rst",
            format!("Home\n====\n\n.. toctree::\n\n   {target}\n"),
        );
        run(&s, &o, &[], 2);
        assert!(!o.exists(), "{target}");
    }
    let t = tempfile::tempdir().unwrap();
    let s = t.path().join("s");
    let o = t.path().join("o");
    basic(&s);
    write(&s, "unlisted.rst", "Not published\n=============\n");
    write(&s, "index.rst", "Home\n====\n\n:doc:`unlisted`\n");
    run(&s, &o, &[], 2);
    assert!(!o.exists());
}

#[test]
fn unsafe_assets_fail_even_when_copying_is_disabled_and_sources_are_not_downloaded() {
    for args in [vec![], vec!["--assets", "skip"]] {
        let t = tempfile::tempdir().unwrap();
        let s = t.path().join("s");
        let o = t.path().join("o");
        basic(&s);
        write(&s, "index.rst", "Home\n====\n\n.. image:: ../outside.svg\n");
        run(&s, &o, &args, 2);
        assert!(!o.exists());
        write(
            &s,
            "index.rst",
            "Home\n====\n\n:download:`Config <conf.py>`\n\n:download:`Orphan <orphan.rst>`\n\n.. image:: logo.svg\n",
        );
        write(&s, "orphan.rst", "Orphan\n======\n");
        write(&s, "logo.svg", "<svg/>");
        let r = run(&s, &o, &args, 0);
        assert!(!o.join("assets/imported/docs/conf.py").exists());
        assert!(!o.join("assets/imported/docs/orphan.rst").exists());
        assert_eq!(
            o.join("assets/imported/docs/logo.svg").exists(),
            args.is_empty()
        );
        assert!(report(&r).contains("not copied"));
    }
}

#[test]
fn source_html_is_literal_and_code_fences_cannot_escape() {
    let t = tempfile::tempdir().unwrap();
    let s = t.path().join("s");
    let o = t.path().join("o");
    basic(&s);
    write(
        &s,
        "index.rst",
        "Home\n====\n\n<script>alert(1)</script> and {expression()}\n\n.. code-block:: text\n\n   ````\n   .. toctree::\n\n      hidden\n   ````\n",
    );
    run(&s, &o, &[], 0);
    let page = read(&o, "01-guide/index.mdx");
    assert!(!page.contains("<script>"), "{page}");
    assert!(page.contains("`````text\n````"), "{page}");
    assert!(page.contains("hidden"));
}

#[test]
fn resource_names_are_literal_not_percent_decoded_as_source_urls() {
    let t = tempfile::tempdir().unwrap();
    let s = t.path().join("s");
    let o = t.path().join("o");
    basic(&s);
    write(
        &s,
        "index.rst",
        "Home\n====\n\n:download:`Percent <Report%20.pdf>`\n\n:download:`Space <Report .pdf>`\n\n.. image:: chart%20.svg\n",
    );
    write(&s, "Report%20.pdf", b"literal percent");
    write(&s, "Report .pdf", b"space");
    write(&s, "chart%20.svg", b"<svg>percent</svg>");
    write(&s, "chart .svg", b"<svg>space</svg>");
    run(&s, &o, &[], 0);
    assert_eq!(
        read(&o, "assets/imported/docs/Report%20.pdf"),
        "literal percent"
    );
    assert_eq!(read(&o, "assets/imported/docs/Report .pdf"), "space");
    assert_eq!(
        read(&o, "assets/imported/docs/chart%20.svg"),
        "<svg>percent</svg>"
    );
    assert!(!o.join("assets/imported/docs/chart .svg").exists());
    let page = read(&o, "01-guide/index.mdx");
    assert!(
        page.contains("Report%2520.pdf")
            && page.contains("Report%20.pdf")
            && page.contains("chart%2520.svg"),
        "{page}"
    );
}

#[test]
fn detect_alias_dry_runs_and_ship_share_the_converter() {
    let t = tempfile::tempdir().unwrap();
    let s = t.path().join("s");
    let o = t.path().join("o");
    basic(&s);
    let r = Command::new(env!("CARGO_BIN_EXE_zel"))
        .args(["detect", "--json"])
        .arg(&s)
        .output()
        .unwrap();
    assert!(r.status.success());
    let v: serde_json::Value = serde_json::from_slice(&r.stdout).unwrap();
    assert_eq!(v["matches"][0]["id"], "sphinx");
    for command in ["convert", "cv", "migrate", "mig", "ship"] {
        let mut c = Command::new(env!("CARGO_BIN_EXE_zel"));
        c.arg(command)
            .arg(&s)
            .arg("--out")
            .arg(&o)
            .args(["--dry-run", "--from", "sphinx"]);
        if command == "ship" {
            c.args(["--provider", "netlify", "--site-id", "fake-site"]);
        }
        let r = c.output().unwrap();
        assert!(r.status.success(), "{command}: {r:?}");
        assert!(!o.exists());
    }
}

#[test]
fn normalized_route_and_global_label_collisions_fail() {
    for second in ["my-page", "different"] {
        let t = tempfile::tempdir().unwrap();
        let s = t.path().join("s");
        let o = t.path().join("o");
        basic(&s);
        write(
            &s,
            "index.rst",
            format!("Home\n====\n\n.. toctree::\n\n   MyPage\n   {second}\n"),
        );
        write(&s, "MyPage.rst", ".. _shared:\n\nFirst\n=====\n");
        write(
            &s,
            &format!("{second}.rst"),
            if second == "different" {
                ".. _shared:\n\nSecond\n======\n"
            } else {
                "Second\n======\n"
            },
        );
        run(&s, &o, &[], 2);
        assert!(!o.exists());
    }
}

#[cfg(unix)]
#[test]
fn symlinked_configs_documents_and_assets_are_rejected() {
    for name in ["conf.py", "index.rst", "logo.svg"] {
        let t = tempfile::tempdir().unwrap();
        let s = t.path().join("s");
        let o = t.path().join("o");
        basic(&s);
        write(&s, "index.rst", "Home\n====\n\n.. image:: logo.svg\n");
        write(t.path(), "outside", "<svg/>");
        if s.join(name).exists() {
            fs::remove_file(s.join(name)).unwrap();
        }
        std::os::unix::fs::symlink(t.path().join("outside"), s.join(name)).unwrap();
        run(&s, &o, &[], 2);
        assert!(!o.exists());
    }
}
