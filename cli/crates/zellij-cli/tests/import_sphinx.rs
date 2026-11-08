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
        "data/values%20.csv",
        format!(
            "\u{feff}{}",
            include_str!("../../../../scripts/fixtures/sphinx-external-table.csv")
                .replace("\r\n", "\n")
                .replace('\n', "\r\n")
        ),
    );
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
    assert_eq!(body.matches("<table>").count(), 9);
    assert_eq!(body.matches("<thead>").count(), 5);
    assert_eq!(body.matches("<th>").count(), 19);
    assert_eq!(body.matches("<caption>").count(), 5);
    assert_eq!(body.matches("<th scope=\"row\">").count(), 9);
    for expected in [
        "**Feature** matrix",
        "External CSV searchable **value**",
        "[CSV details](</guide/details>)",
        "![CSV file logo](</_zellij/assets/imported/docs/media/logo.svg>)",
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
    assert!(!c.join("assets/imported/docs/data/values%20.csv").exists());
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
        "<ol start=\"8\" type=\"A\">",
        "<ol start=\"8\" type=\"a\">",
        "<ol start=\"4\" type=\"I\">",
        "<ol start=\"4\" type=\"i\">",
        "[Roman reference](</guide/details#install>)",
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
    assert_eq!(body.matches("<ol start=").count(), 9);
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
fn enumerated_prose_and_automatic_restarts_keep_their_meaning() {
    let t = tempfile::tempdir().unwrap();
    let source = t.path().join("s");
    let out = t.path().join("o");
    basic(&source);
    write(
        &source,
        "index.rst",
        concat!(
            "Home\n====\n\nA. Einstein was\na physicist.\n\n",
            "9. nine\n#. ten\n11. explicit eleven\n\n",
            "h. eight\n#. nine\nj. explicit ten\n\n",
            "iv. four\n#. five\nvi. explicit six\n",
        ),
    );
    run(&source, &out, &[], 0);
    let body = read(&out, "01-guide/index.mdx");
    assert!(body.contains("A. Einstein was\na physicist."), "{body}");
    assert_eq!(body.matches("<ol start=").count(), 6, "{body}");
    for start in [
        "9",
        "11",
        "8\" type=\"a",
        "10\" type=\"a",
        "4\" type=\"i",
        "6\" type=\"i",
    ] {
        assert!(body.contains(&format!("<ol start=\"{start}\">")), "{body}");
    }
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
        "iiii. invalid Roman",
        "iV. mixed case Roman",
        "MMMM. too large Roman",
        "z. last alphabetic\n#. overflow",
        "mmmcmxcix. last Roman\n#. overflow",
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

#[test]
fn substitutions_preserve_forward_references_inline_markup_images_links_and_scope() {
    let t = tempfile::tempdir().unwrap();
    let source = t.path().join("s");
    let out = t.path().join("o");
    basic(&source);
    write(
        &source,
        "conf.py",
        "project='Manual'\nversion='2.4'\nrelease='2.4.1'\n",
    );
    let fixture = include_str!("../../../../scripts/fixtures/sphinx-substitutions.rst");
    write(
        &source,
        "index.rst",
        format!("{fixture}\n.. toctree::\n\n   details\n"),
    );
    write(
        &source,
        "details.rst",
        ".. _install:\n\nInstallation\n============\n\n|Product| is local.\n\n.. |Product| replace:: Different product\n",
    );
    write(&source, "media/logo.svg", "<svg>substitution image</svg>");
    write(&source, "notes.txt", "Substitution download bytes.\n");
    run(&source, &out, &[], 0);
    let body = read(&out, "01-guide/index.mdx");
    for text in [
        "**Zellij** is the **Zellij** documentation engine.",
        "[*Read* the guide](</guide/details#install>)",
        "[![Substitution logo](</_zellij/assets/imported/docs/media/logo.svg>)](</guide/details#install>)",
        "The release is 2\\.4\\.1 (2\\.4).",
        "[Substituted section](</guide/details#install>)",
        "[Substituted notes](</_zellij/assets/imported/docs/notes.txt>)",
        "|undefined-in-code|",
        ".. |ignored| replace:: not a definition",
        "` |Product| `",
        "**Zellij** features",
        "<td>",
    ] {
        assert!(body.contains(text), "{text}: {body}");
    }
    assert_eq!(
        read(&out, "assets/imported/docs/media/logo.svg"),
        "<svg>substitution image</svg>"
    );
    assert_eq!(
        read(&out, "assets/imported/docs/notes.txt"),
        "Substitution download bytes.\n"
    );
    assert!(read(&out, "01-guide/details.mdx").contains("Different product is local."));
    assert!(read(&source, "index.rst").starts_with(fixture));
    let dry = t.path().join("dry");
    run(&source, &dry, &["--dry-run"], 0);
    assert!(!dry.exists());
    let skip = t.path().join("skip");
    run(&source, &skip, &["--assets", "skip"], 0);
    assert!(!skip.join("assets/imported").exists());
}

#[test]
fn invalid_substitutions_fail_before_output_even_when_unused() {
    for text in [
        "|missing|",
        ".. |a| replace:: |a|",
        ".. |a| replace:: |b|\n.. |b| replace:: |a|",
        ".. |same| replace:: one\n.. |same| replace:: two",
        ".. |a  b| replace:: one\n.. |a b| replace:: two",
        ".. | bad| replace:: text",
        ".. |a|replace:: text",
        ".. |a| replace::",
        ".. |a| replace:: first\n\n   second paragraph",
        ".. toctree::\n   :caption: |a|\n\n   self\n\n.. |a| replace:: title",
        ".. toctree::\n\n   |a| <self>\n\n.. |a| replace:: title",
        ".. |a| replace:: - block list",
        ".. |a| replace:: .. include:: private",
        ".. |a| replace:: text\n   :trim:",
        ".. |a| date::",
        ".. |a| unicode:: 0x20",
        ".. |a| raw:: html\n\n   <script>bad</script>",
        ".. |a| image:: logo.svg\n   :width: 20",
        ".. |a| image:: logo.svg\n   :target:",
        ".. |a| image:: logo.svg\n\n   unwanted body",
        "|CASE|\n\n.. |Case| replace:: first\n.. |case| replace:: second",
        "|a|__\n\n.. |a| replace:: anonymous",
        "|a|_\n\n.. |a| replace:: `nested <https://example.com>`_\n.. _a: https://example.net",
        "word|a|\n\n.. |a| replace:: adjacent",
        "|a|word\n\n.. |a| replace:: adjacent",
        "|a|_word\n\n.. |a| replace:: adjacent\n.. _a: https://example.com",
        "|a|\n====\n\n.. |a| replace:: complex heading",
    ] {
        let t = tempfile::tempdir().unwrap();
        let source = t.path().join("s");
        let out = t.path().join("o");
        basic(&source);
        let raw = format!("Home\n====\n\n{text}\n");
        write(&source, "index.rst", &raw);
        run(&source, &out, &[], 2);
        assert!(!out.exists(), "{text}");
        assert_eq!(read(&source, "index.rst"), raw);
    }
}

#[test]
fn substitution_assets_enforce_containment_in_copy_skip_and_dry_runs() {
    for definition in [
        "image:: ../secret",
        "replace:: :download:`secret <../secret>`",
    ] {
        for args in [vec![], vec!["--assets", "skip"], vec!["--dry-run"]] {
            let t = tempfile::tempdir().unwrap();
            let source = t.path().join("s");
            let out = t.path().join("o");
            basic(&source);
            write(t.path(), "secret", "substitution-private-sentinel");
            write(
                &source,
                "index.rst",
                format!(
                    "Home\n====\n\n|outer|\n\n.. |outer| replace:: |inner|\n.. |inner| {definition}\n"
                ),
            );
            let result = run(&source, &out, &args, 2);
            assert!(!out.exists());
            assert!(report(&result).contains("escapes"), "{result:?}");
            assert!(!report(&result).contains("substitution-private-sentinel"));
        }
    }
}

#[test]
fn substitution_configuration_stays_literal_and_authored_overrides_are_local() {
    let t = tempfile::tempdir().unwrap();
    let source = t.path().join("s");
    let out = t.path().join("o");
    basic(&source);
    write(
        &source,
        "conf.py",
        "project='Manual'\nversion='1. |missing| **plain** <tag>'\nrelease='configured'\n",
    );
    write(
        &source,
        "index.rst",
        concat!(
            "Home\n====\n\n|VERSION|\n\n|release|. |Mixed name| / |MIXED NAME|.\n\n",
            "|x| / |X|. |body|\n\n",
            ".. |release| replace:: Authored release\n",
            ".. |Mixed   name| replace:: Wrapped\n   continuation\n",
            ".. |x| replace:: lower\n.. |X| replace:: upper\n",
            ".. |body| replace::\n\n   Body paragraph\n",
        ),
    );
    run(&source, &out, &[], 0);
    let body = read(&out, "01-guide/index.mdx");
    assert!(
        body.contains("1\\. \\|missing\\| \\*\\*plain\\*\\* &lt;tag&gt;"),
        "{body}"
    );
    assert!(
        body.contains("Authored release. Wrapped continuation / Wrapped continuation."),
        "{body}"
    );
    assert!(body.contains("lower / upper. Body paragraph"), "{body}");
    assert!(!body.contains("configured"));
}

#[test]
fn literalinclude_preserves_code_and_resolves_nested_blocks_without_asset_copying() {
    let t = tempfile::tempdir().unwrap();
    let source = t.path().join("s");
    basic(&source);
    let rst = include_str!("../../../../scripts/fixtures/sphinx-literalinclude.rst");
    let code =
        include_str!("../../../../scripts/fixtures/sphinx-literal-code.txt").replace("\r\n", "\n");
    // Exercise CRLF input on every host, independently of Git checkout settings.
    let source_code = code.replace('\n', "\r\n");
    write(&source, "index.rst", rst);
    write(&source, "snippets/Sample%20.py", &source_code);
    write(&source, "snippets/Sample .py", "wrong file");
    for (i, args) in [vec![], vec!["--assets", "skip"], vec!["--layout", "site"]]
        .iter()
        .enumerate()
    {
        let out = t.path().join(format!("out-{i}"));
        run(&source, &out, args, 0);
        let content = if i == 2 {
            out.join("content")
        } else {
            out.clone()
        };
        let body = read(&content, "01-guide/index.mdx");
        assert!(body.contains(&format!("`````text\n{code}`````")), "{body}");
        assert!(body.contains("```text\nliteral_beta = \"{not_executed()} <script>literal()</script>\"\nliteral_alpha = \"Café 漢字\"\nliteral_beta ="), "{body}");
        assert!(body.contains("> literal_alpha = \"Café 漢字\""), "{body}");
        assert!(body.contains("<ul>") && body.contains("<table>"), "{body}");
        assert!(!body.contains("wrong file"));
        assert!(
            !content
                .join("assets/imported/docs/snippets/Sample%20.py")
                .exists()
        );
    }
    let dry = t.path().join("dry");
    run(&source, &dry, &["--dry-run"], 0);
    assert!(!dry.exists());
    assert_eq!(read(&source, "snippets/Sample%20.py"), source_code);
    assert_eq!(read(&source, "index.rst"), rst);
}

#[test]
fn literalinclude_paths_are_document_relative_or_source_rooted_and_not_url_decoded() {
    let t = tempfile::tempdir().unwrap();
    let source = t.path().join("s");
    basic(&source);
    write(
        &source,
        "index.rst",
        "Home\n====\n\n.. toctree::\n\n   nested/page\n",
    );
    write(
        &source,
        "nested/page.rst",
        "Page\n====\n\n.. literalinclude:: ../snippets/Café #100%20.py\n\n.. literalinclude:: /snippets/Café #100%20.py\n",
    );
    write(
        &source,
        "snippets/Café #100%20.py",
        "exact_literal_filename\n",
    );
    // Unpublished documents are parsed, but their file targets are not opened.
    write(
        &source,
        "orphan.rst",
        "Orphan\n======\n\n.. literalinclude:: missing.py\n",
    );
    let out = t.path().join("o");
    run(&source, &out, &[], 0);
    assert_eq!(
        read(&out, "01-guide/nested/page.mdx")
            .matches("exact_literal_filename")
            .count(),
        2
    );
}

#[test]
fn literalinclude_encoding_tabs_and_line_endings_are_explicit() {
    for (i, (config, options, expected)) in [
        ("", "", "a\n\tb\nc"),
        ("source_encoding='utf-8'\n", "", "\u{feff}a\n\tb\nc"),
        (
            "source_encoding='utf-8'\n",
            "   :encoding: utf-8-sig\n   :tab-width: 3\n",
            "a\n   b\nc",
        ),
        ("", "   :encoding: utf-8\n", "\u{feff}a\n\tb\nc"),
    ]
    .iter()
    .enumerate()
    {
        let t = tempfile::tempdir().unwrap();
        let s = t.path().join("s");
        basic(&s);
        write(
            &s,
            "conf.py",
            format!("project='Manual'\nhighlight_language='python'\n{config}"),
        );
        write(
            &s,
            "index.rst",
            format!("Home\n====\n\n.. literalinclude:: code.py\n{options}"),
        );
        write(&s, "code.py", "\u{feff}a\r\n\tb\rc");
        let o = t.path().join("o");
        run(&s, &o, &[], 0);
        assert!(
            read(&o, "01-guide/index.mdx").contains(&format!("```python\n{expected}\n```")),
            "encoding case {i}"
        );
    }
}

#[test]
fn literalinclude_selection_order_markers_and_dedent_are_preserved() {
    for (options, expected) in [
        (":lines: 4,1,3-4,2-", "d\na\nc\nd\nb\nc\nd\n"),
        (":lines: -2", "a\nb\n"),
        (":start-after: a\n:end-at: d\n:lines: 2-", "c\nd\n"),
        (":start-at: b\n:end-before: d", "b\nc\n"),
    ] {
        let t = tempfile::tempdir().unwrap();
        let s = t.path().join("s");
        basic(&s);
        write(
            &s,
            "index.rst",
            format!(
                "Home\n====\n\n.. literalinclude:: code.py\n   {}\n",
                options.replace('\n', "\n   ")
            ),
        );
        write(&s, "code.py", "a\nb\nc\nd\n");
        let o = t.path().join("o");
        run(&s, &o, &[], 0);
        assert!(
            read(&o, "01-guide/index.mdx").contains(&format!("```text\n{expected}```")),
            "{options}"
        );
    }
    for (options, code, expected) in [
        (
            ":start-at: MARK\n:end-before: MARK",
            "MARK\nkeep\nMARK\ndrop\n",
            "MARK\nkeep\n",
        ),
        (
            ":dedent:",
            " \tfirst\n \t  second\n \t\n",
            "first\n  second\n\n",
        ),
        (":dedent: 2", "  café\n    漢字\n\n", "café\n  漢字\n\n"),
        (":dedent: 0", "  untouched\n", "  untouched\n"),
        ("", "", "\n"),
    ] {
        let t = tempfile::tempdir().unwrap();
        let s = t.path().join("s");
        basic(&s);
        write(
            &s,
            "index.rst",
            format!(
                "Home\n====\n\n.. literalinclude:: code.py\n   {}\n",
                options.replace('\n', "\n   ")
            ),
        );
        write(&s, "code.py", code);
        let o = t.path().join("o");
        run(&s, &o, &[], 0);
        assert!(
            read(&o, "01-guide/index.mdx").contains(&format!("```text\n{expected}```")),
            "{options}"
        );
    }
}

#[test]
fn invalid_literalinclude_options_and_selections_fail_before_writes() {
    for options in [
        ":lines: 0",
        ":lines: 3-1",
        ":lines: 1,",
        ":lines: -",
        ":lines: 1-2-3",
        ":lines: 999",
        ":lines: 4-",
        ":lines: +1",
        ":lines: 999999999999999999999999",
        ":start-at: absent",
        ":end-before: absent",
        ":start-at: alpha\n:start-after: alpha",
        ":end-at: beta\n:end-before: beta",
        ":start-at:",
        ":end-at:",
        ":dedent: 1",
        ":dedent: -1",
        ":tab-width: 0",
        ":tab-width: 65",
        ":encoding: latin1",
        ":language: js{bad}",
        ":language:",
        ":lines: 1\n:lines: 2",
        ":caption: caption",
        ":linenos:",
        ":lineno-start: 4",
        ":lineno-match:",
        ":emphasize-lines: 1",
        ":pyobject: demo",
        ":diff: other.py",
        ":prepend: text",
        ":append: text",
        ":class: style",
        ":name: code",
        ":force:",
    ] {
        for args in [vec![], vec!["--dry-run"], vec!["--assets", "skip"]] {
            let t = tempfile::tempdir().unwrap();
            let s = t.path().join("s");
            basic(&s);
            let rst = format!(
                "Home\n====\n\n.. literalinclude:: code.py\n   {}\n",
                options.replace('\n', "\n   ")
            );
            write(&s, "index.rst", &rst);
            write(&s, "code.py", "alpha\nbeta\n");
            let o = t.path().join("o");
            let result = run(&s, &o, &args, 2);
            assert!(!o.exists(), "{options}: {result:?}");
            assert_eq!(read(&s, "index.rst"), rst);
        }
    }
}

#[test]
fn literalinclude_rejects_unsafe_missing_excluded_and_nontext_files() {
    for target in [
        "../outside.py",
        "//server/code.py",
        "C:/code.py",
        "https://example.test/code.py",
        "nested\\code.py",
        "<standard.txt>",
        "missing.py",
        "snippets",
        ".hidden.py",
        "excluded.py",
        "EXCLUDED.py",
        "snippets/CODE.py",
        "_build/code.py",
        "templates/code.py",
        "binary.py",
        "control.py",
    ] {
        for args in [vec![], vec!["--dry-run"], vec!["--assets", "skip"]] {
            let t = tempfile::tempdir().unwrap();
            let s = t.path().join("s");
            basic(&s);
            write(
                &s,
                "conf.py",
                "project='Manual'\nexclude_patterns=['excluded.py']\ntemplates_path=['templates']\n",
            );
            write(
                &s,
                "index.rst",
                format!("Home\n====\n\n.. literalinclude:: {target}\n"),
            );
            for file in [
                ".hidden.py",
                "excluded.py",
                "_build/code.py",
                "templates/code.py",
                "snippets/code.py",
            ] {
                write(&s, file, "literal-private-sentinel");
            }
            write(t.path(), "outside.py", "literal-private-sentinel");
            write(&s, "binary.py", [0xff]);
            write(&s, "control.py", [0]);
            let o = t.path().join("o");
            let result = run(&s, &o, &args, 2);
            assert!(!o.exists());
            assert!(
                !report(&result).contains("literal-private-sentinel"),
                "{result:?}"
            );
        }
    }
}

#[test]
fn literalinclude_limits_bound_files_selection_and_repeated_expansion() {
    for (code, options, repeats) in [
        ("x".repeat(8 * 1024 * 1024 + 1), "", 1),
        ("x\n".repeat(100_001), ":lines: 1-", 1),
        ("x".repeat(7 * 1024 * 1024), "", 5),
        ("\t".repeat(1024 * 1024), ":tab-width: 64", 1),
    ] {
        let t = tempfile::tempdir().unwrap();
        let s = t.path().join("s");
        basic(&s);
        write(&s, "code.py", &code);
        write(
            &s,
            "index.rst",
            format!(
                "Home\n====\n\n{}",
                format!(".. literalinclude:: code.py\n   {options}\n\n").repeat(repeats)
            ),
        );
        let o = t.path().join("o");
        run(&s, &o, &[], 2);
        assert!(!o.exists());
    }
}

#[test]
fn literalinclude_requires_target_and_no_body_and_does_not_enable_parsed_includes() {
    for directive in [
        ".. literalinclude::",
        ".. literalinclude:: code.py\n\n   body",
        ".. include:: code.py",
    ] {
        let t = tempfile::tempdir().unwrap();
        let s = t.path().join("s");
        basic(&s);
        write(&s, "code.py", "code\n");
        write(&s, "index.rst", format!("Home\n====\n\n{directive}\n"));
        let o = t.path().join("o");
        run(&s, &o, &[], 2);
        assert!(!o.exists());
    }
}

#[cfg(unix)]
#[test]
fn literalinclude_rejects_symlinked_files_and_directories() {
    for directory in [false, true] {
        let t = tempfile::tempdir().unwrap();
        let s = t.path().join("s");
        basic(&s);
        write(t.path(), "outside/code.py", "literal-private-sentinel");
        let (from, to, target) = if directory {
            ("outside", "linked", "linked/code.py")
        } else {
            ("outside/code.py", "linked.py", "linked.py")
        };
        std::os::unix::fs::symlink(t.path().join(from), s.join(to)).unwrap();
        write(
            &s,
            "index.rst",
            format!("Home\n====\n\n.. literalinclude:: {target}\n"),
        );
        let o = t.path().join("o");
        let result = run(&s, &o, &[], 2);
        assert!(!o.exists());
        assert!(!report(&result).contains("literal-private-sentinel"));
    }
}

#[test]
fn local_csv_files_preserve_tables_references_and_encoding_without_copying_data() {
    let t = tempfile::tempdir().unwrap();
    let s = t.path().join("s");
    basic(&s);
    write(
        &s,
        "index.rst",
        "Home\n====\n\n.. toctree::\n\n   nested/page\n   details\n",
    );
    write(&s, "details.rst", "Details\n=======\n");
    write(
        &s,
        "nested/page.rst",
        "Page\n====\n\n.. csv-table:: **Local** matrix\n   :file: ../data/Café%20.csv\n   :encoding: utf-8-sig\n   :header: Key,Value\n   :header-rows: 1\n   :stub-columns: 1\n\n.. note:: Nested\n\n   .. csv-table::\n      :file: /data/Café%20.csv\n",
    );
    let data = "\u{feff}Label,Description\r\nAlpha,**Searchable file cell**\r\nDocs,:doc:`/details`\r\nFile,:download:`../notes.txt`\r\n";
    write(&s, "data/Café%20.csv", data);
    write(&s, "data/Café .csv", "Wrong filename");
    write(&s, "notes.txt", "CSV-linked download");
    write(
        &s,
        "orphan.rst",
        "Orphan\n======\n\n.. csv-table::\n   :file: absent.csv\n",
    );
    for (i, args) in [vec![], vec!["--assets", "skip"], vec!["--layout", "site"]]
        .iter()
        .enumerate()
    {
        let o = t.path().join(format!("out-{i}"));
        run(&s, &o, args, 0);
        let c = if i == 2 { o.join("content") } else { o };
        let body = read(&c, "01-guide/nested/page.mdx");
        assert_eq!(body.matches("<table>").count(), 2, "{body}");
        assert!(
            body.contains("**Local** matrix") && body.contains("**Searchable file cell**"),
            "{body}"
        );
        assert!(body.contains("[Details](</guide/details>)"), "{body}");
        assert!(
            !body.contains('\u{feff}') && !body.contains("Wrong filename"),
            "{body}"
        );
        assert!(!c.join("assets/imported/docs/data/Café%20.csv").exists());
        if i != 1 {
            assert_eq!(
                read(&c, "assets/imported/docs/notes.txt"),
                "CSV-linked download"
            );
        }
    }
    let dry = t.path().join("dry");
    run(&s, &dry, &["--dry-run"], 0);
    assert!(!dry.exists());
    assert_eq!(read(&s, "data/Café%20.csv"), data);
}

#[test]
fn local_csv_cells_parse_nested_resources_and_document_substitutions() {
    let t = tempfile::tempdir().unwrap();
    let s = t.path().join("s");
    basic(&s);
    write(
        &s,
        "index.rst",
        "Home\n====\n\n.. |product| replace:: Zellij\n\n.. csv-table::\n   :file: data/outer.csv\n",
    );
    write(
        &s,
        "data/outer.csv",
        "\".. csv-table::\n   :file: inner.csv\",\".. literalinclude:: snippets/example.py\"\n",
    );
    write(&s, "data/inner.csv", "|product|,nested table\n");
    write(&s, "snippets/example.py", "literal code from CSV\n");
    let o = t.path().join("o");
    run(&s, &o, &[], 0);
    let body = read(&o, "01-guide/index.mdx");
    assert_eq!(body.matches("<table>").count(), 2, "{body}");
    assert!(
        body.contains("Zellij") && body.contains("```text\nliteral code from CSV\n```"),
        "{body}"
    );
}

#[test]
fn local_csv_supports_tab_delimiters_and_explicit_bom_policy() {
    for (i, (encoding, bom)) in [("utf-8-sig", false), ("utf-8", true)].iter().enumerate() {
        let t = tempfile::tempdir().unwrap();
        let s = t.path().join("s");
        basic(&s);
        write(
            &s,
            "index.rst",
            format!(
                "Home\n====\n\n.. csv-table::\n   :file: data.tsv\n   :encoding: {encoding}\n   :delim: tab\n   :quote: '\n   :header-rows: 1\n"
            ),
        );
        write(
            &s,
            "data.tsv",
            "\u{feff}Key\tValue\rAlpha\t'quoted\tcell'\r",
        );
        let o = t.path().join(format!("o-{i}"));
        run(&s, &o, &[], 0);
        let body = read(&o, "01-guide/index.mdx");
        assert_eq!(body.contains('\u{feff}'), *bom, "{body}");
        assert!(body.contains("quoted\tcell"), "{body}");
        assert_eq!(body.matches("<th>").count(), 2, "{body}");
    }
}

#[test]
fn invalid_local_csv_sources_fail_before_writes_in_every_import_mode() {
    for (file, options, data) in [
        ("../outside.csv", "", "value"),
        ("https://example.invalid/data.csv", "", "value"),
        ("//server/data.csv", "", "value"),
        ("C:\\data.csv", "", "value"),
        (".hidden.csv", "", "value"),
        ("excluded/data.csv", "", "value"),
        ("DATA.csv", "", "value"),
        ("missing.csv", "", "value"),
        ("data.csv", ":encoding: latin-1", "value"),
        (
            "data.csv",
            ":url: https://example.invalid/data.csv",
            "value",
        ),
        ("data.csv", ":header-rows: 1", "single row"),
        ("data.csv", "", "\"unclosed"),
        ("data.csv", "", ".. raw:: html"),
        ("data.csv", "", "\0binary"),
        ("data.csv", "", "\u{2028}unsupported"),
        ("data.csv", "", ".. csv-table::\n   :file: data.csv"),
    ] {
        let t = tempfile::tempdir().unwrap();
        let s = t.path().join("s");
        basic(&s);
        write(
            &s,
            "conf.py",
            "project='CSV'\nexclude_patterns=['excluded']\n",
        );
        for p in ["data.csv", ".hidden.csv", "excluded/data.csv"] {
            write(&s, p, data);
        }
        write(t.path(), "outside.csv", "outside sentinel");
        write(
            &s,
            "index.rst",
            format!("Home\n====\n\n.. csv-table::\n   :file: {file}\n   {options}\n"),
        );
        for (i, args) in [vec![], vec!["--dry-run"], vec!["--assets", "skip"]]
            .iter()
            .enumerate()
        {
            let o = t.path().join(format!("out-{i}"));
            run(&s, &o, args, 2);
            assert!(!o.exists());
        }
    }
    for data in [vec![0xff, 0xfe], vec![b'x'; 8 * 1024 * 1024 + 1]] {
        let t = tempfile::tempdir().unwrap();
        let s = t.path().join("s");
        basic(&s);
        write(&s, "data.csv", data);
        write(
            &s,
            "index.rst",
            "Home\n====\n\n.. csv-table::\n   :file: data.csv\n",
        );
        let o = t.path().join("o");
        run(&s, &o, &[], 2);
        assert!(!o.exists());
    }
}

#[test]
fn local_csv_requires_one_data_source_and_preserves_output_on_failure() {
    for directive in [
        ".. csv-table::\n   :file:\n",
        ".. csv-table::\n   :file: data.csv\n\n   inline,body\n",
        ".. csv-table::\n   :file: data.csv\n   :file: other.csv\n",
    ] {
        let t = tempfile::tempdir().unwrap();
        let s = t.path().join("s");
        basic(&s);
        write(&s, "data.csv", "valid,data");
        write(&s, "index.rst", format!("Home\n====\n\n{directive}"));
        let o = t.path().join("o");
        run(&s, &o, &[], 2);
        assert!(!o.exists());
    }
}

#[cfg(unix)]
#[test]
fn local_csv_rejects_symlink_sources() {
    let t = tempfile::tempdir().unwrap();
    let s = t.path().join("s");
    basic(&s);
    write(t.path(), "outside.csv", "outside,data");
    std::os::unix::fs::symlink(t.path().join("outside.csv"), s.join("data.csv")).unwrap();
    write(
        &s,
        "index.rst",
        "Home\n====\n\n.. csv-table::\n   :file: data.csv\n",
    );
    let o = t.path().join("o");
    run(&s, &o, &["--assets", "skip"], 2);
    assert!(!o.exists());
}

#[test]
fn recursive_csv_files_stop_at_the_shared_nesting_limit() {
    let t = tempfile::tempdir().unwrap();
    let s = t.path().join("s");
    basic(&s);
    write(
        &s,
        "index.rst",
        "Home\n====\n\n.. csv-table::\n   :file: data.csv\n",
    );
    write(&s, "data.csv", "\".. csv-table::\n   :file: data.csv\"\n");
    let o = t.path().join("o");
    let r = run(&s, &o, &[], 2);
    assert!(report(&r).contains("nesting exceeds 16"), "{}", report(&r));
    assert!(!o.exists());
}

#[test]
fn csv_and_literal_includes_share_the_total_expansion_budget() {
    let t = tempfile::tempdir().unwrap();
    let s = t.path().join("s");
    basic(&s);
    write(&s, "large.txt", "x".repeat(8 * 1024 * 1024));
    let mut page = String::from("Home\n====\n");
    for _ in 0..4 {
        page.push_str("\n.. literalinclude:: large.txt\n");
    }
    page.push_str("\n.. csv-table::\n   :file: tiny.csv\n");
    write(&s, "index.rst", page);
    write(&s, "tiny.csv", "small,value\n");
    let o = t.path().join("o");
    let r = run(&s, &o, &[], 2);
    assert!(
        report(&r).contains("total expanded text exceeds 32 MiB"),
        "{}",
        report(&r)
    );
    assert!(!o.exists());
}

#[test]
fn external_csv_headers_and_data_keep_distinct_nested_file_sources() {
    let t = tempfile::tempdir().unwrap();
    let s = t.path().join("s");
    basic(&s);
    write(
        &s,
        "index.rst",
        "Home\n====\n\n.. csv-table::\n   :file: data/outer.csv\n   :header: \".. csv-table::\n               :file: header.csv\"\n",
    );
    write(&s, "header.csv", "HEADER source\n");
    write(&s, "data/header.csv", "WRONG header source\n");
    write(
        &s,
        "data/outer.csv",
        "\".. csv-table::\n   :file: inner.csv\"\n",
    );
    write(&s, "data/inner.csv", "DATA source\n");
    write(&s, "inner.csv", "WRONG data source\n");
    let o = t.path().join("o");
    run(&s, &o, &[], 0);
    let body = read(&o, "01-guide/index.mdx");
    assert_eq!(body.matches("<table>").count(), 3, "{body}");
    assert!(
        body.contains("HEADER source") && body.contains("DATA source"),
        "{body}"
    );
    assert!(!body.contains("WRONG"), "{body}");
}
