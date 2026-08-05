//! The files themselves.
//!
//! Written as format strings rather than through a template engine: there are
//! a dozen of them, they interpolate a name and a theme, and a dependency that
//! turned `{name}` into a runtime error class would be a poor trade.

use crate::{DocsOptions, Scaffold, section};

pub(crate) fn docs_into(scaffold: &mut Scaffold, options: &DocsOptions) {
    let DocsOptions {
        name,
        theme,
        guide: with_guide,
        minimal,
    } = options;

    scaffold.add(
        "site.yaml",
        site_yaml(name, *theme == *"paper", theme, *with_guide),
    );
    scaffold.add("pages.yaml", pages_yaml(name, *minimal));
    scaffold.add("sections/home/hero.yaml", hero_yaml(name));

    if !minimal {
        scaffold.add("sections/home/features.yaml", FEATURES_YAML);
    }

    if *with_guide {
        scaffold.add("menu.yaml", "- { name: Guide, page: 01-guide }\n");
        guide_into(scaffold, "guide", "Guide", 1);
    } else {
        // A menu with nothing in it is still the file somebody edits first, so
        // it is written with the landing page in it rather than left out.
        scaffold.add("menu.yaml", "- { name: Home, page: home }\n");
    }

    // Git will not track an empty directory, and an assets folder that vanishes
    // on clone is a confusing first bug.
    scaffold.add("assets/.gitkeep", "");
}

fn site_yaml(name: &str, default_theme: bool, theme: &str, with_guide: bool) -> String {
    let theme_line = if default_theme {
        format!("theme: {theme}          # zel list themes — there are fourteen")
    } else {
        format!("theme: {theme}")
    };

    // `legal` is a key of `footer`, not of the site. Emitting it at the top
    // level parsed fine and was silently ignored — unknown top-level keys warn
    // rather than fail, so the copyright line simply never appeared. Found by
    // running the validator over the output, which is the only way this class
    // of mistake surfaces.
    let columns = if with_guide {
        "  columns:\n    - heading: Docs\n      links:\n        - { label: Guide, page: 01-guide }\n"
    } else {
        ""
    };

    format!(
        "# Everything about the site rather than about a page.\n\
         # Every key is optional; see https://zellij.shebka.net/guide/site-config\n\
         name: {name}\n\
         tagline: Built with Zellij.\n\
         \n\
         {theme_line}\n\
         mode: system\n\
         motion: subtle\n\
         \n\
         home: home\n\
         \n\
         # The absolute origin. Canonical URLs, Open Graph tags and the sitemap\n\
         # all need it, and without it the sitemap is generated empty.\n\
         # siteUrl: https://example.com\n\
         \n\
         nav:\n\
         \x20 sticky: true\n\
         \x20 search: true\n\
         \x20 themeToggle: true\n\
         \n\
         footer:\n\
         {columns}\
         \x20 legal: \"© {name}\"\n"
    )
}

fn pages_yaml(name: &str, minimal: bool) -> String {
    let sections = if minimal {
        "    - home/hero\n"
    } else {
        "    - home/hero\n    - home/features\n"
    };

    format!(
        "# Each entry is a page. `sections` names files under sections/.\n\
         - name: home\n\
         \x20 title: {name}\n\
         \x20 description: A site built from a folder of content.\n\
         \x20 layout: landing\n\
         \x20 sections:\n\
         {sections}"
    )
}

fn hero_yaml(name: &str) -> String {
    format!(
        "type: hero\n\
         headline: {name}\n\
         subhead: Edit sections/home/hero.yaml and this line changes.\n\
         cta:\n\
         \x20 - {{ label: Read the guide, href: /guide, style: primary }}\n"
    )
}

const FEATURES_YAML: &str = "type: feature-grid\n\
title: Why this folder is the whole site\n\
columns: 3\n\
items:\n\
\x20 - title: Text, not code\n\
\x20   description: YAML and Markdown. No framework to upgrade.\n\
\x20   icon: file-text\n\
\x20 - title: Checked before it ships\n\
\x20   description: A broken link fails the build, not a reader.\n\
\x20   icon: shield-check\n\
\x20 - title: Nineteen section types\n\
\x20   description: Run `zel list sections` to see them all.\n\
\x20   icon: layout-grid\n";

pub(crate) fn guide_into(scaffold: &mut Scaffold, name: &str, title: &str, order: usize) {
    let dir = format!("{order:02}-{name}");

    scaffold.add(
        format!("{dir}/_sidebar.yaml"),
        format!(
            "title: {title}\n\
             description: How this site is put together.\n\
             \n\
             # Every page in this folder, in the order a reader should meet\n\
             # them. A page missing from here still builds, with a warning.\n\
             nav:\n\
             \x20 - index\n"
        ),
    );

    scaffold.add(
        format!("{dir}/index.mdx"),
        format!(
            "---\n\
             title: {title}\n\
             description: Start here.\n\
             ---\n\
             \n\
             ## You are editing content, not code\n\
             \n\
             This page is `{dir}/index.mdx`. Add another `.mdx` file beside it and\n\
             list it in `_sidebar.yaml` to make it appear in the sidebar.\n\
             \n\
             <Callout type=\"tip\" title=\"Check before you build\">\n\
             `zel check` validates the whole content folder and names the file behind\n\
             any problem it finds.\n\
             </Callout>\n"
        ),
    );
}

pub(crate) fn page_into(name: &str, section_types: &[String]) -> (Scaffold, String) {
    let mut scaffold = Scaffold::new();
    let types: Vec<&str> = if section_types.is_empty() {
        vec!["hero"]
    } else {
        section_types.iter().map(String::as_str).collect()
    };

    let mut listed = String::new();

    for section_type in &types {
        scaffold.add(
            format!("sections/{name}/{section_type}.yaml"),
            section::section_file(section_type),
        );
        listed.push_str(&format!("    - {name}/{section_type}\n"));
    }

    let title = title_case(name);
    let entry = format!(
        "- name: {name}\n\
         \x20 title: {title}\n\
         \x20 description: TODO\n\
         \x20 sections:\n\
         {listed}"
    );

    (scaffold, entry)
}

fn title_case(name: &str) -> String {
    let mut chars = name.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}
