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
        "- parent\n\n  - nested",
        "+---+---+\n| a | b |\n+---+---+",
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
