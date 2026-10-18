//! A Markdown starter for the native renderer. No hosted engine is required.
use crate::Scaffold;
use zellij_core::{Error, Result, schema, yaml};

pub struct PortableOptions {
    pub name: String,
    pub theme: String,
    pub minimal: bool,
}

impl Default for PortableOptions {
    fn default() -> Self {
        Self {
            name: "Documentation".into(),
            theme: "cupertino".into(),
            minimal: false,
        }
    }
}

/// Validate options before any filesystem writes. Display text lives in quoted
/// metadata, never interpolated into Markdown links, HTML or shell commands.
pub fn portable(options: &PortableOptions) -> Result<Scaffold> {
    if options.name.trim().is_empty() || options.name.chars().any(char::is_control) {
        return Err(Error::InvalidArgument(
            "site name must be nonempty text without control characters".into(),
        ));
    }
    if !schema::themes().iter().any(|t| t.name == options.theme) {
        return Err(Error::InvalidArgument(format!(
            "unknown theme {:?}; run zel list themes",
            options.theme
        )));
    }
    let name = yaml::string(&options.name);
    let theme = yaml::string(&options.theme);
    let mut config = format!(
        "# Native Markdown site. Presentation paths are relative to content/.\nversion: 1\nrenderer: portable\nname: {name}\ntheme: {theme}\n\n# Remove nav to discover and order pages automatically.\nnav:\n  - page: index.md\n    label: Home\n"
    );
    let mut home = format!("---\ntitle: {name}\n---\n\n");
    let mut scaffold = Scaffold::new();
    if options.minimal {
        home.push_str("Write your documentation here.\n\n## Add a page\n\nCreate another Markdown file and add it to `nav` in `zel.yaml`.\n\nRun `zel check --strict` to validate your changes and `zel view --watch`\nto preview them while you write.\n");
    } else {
        config.push_str("  - page: guide/getting-started.md\n    label: Getting started\n");
        home.push_str(HOME);
        scaffold.add("content/guide/getting-started.md", GUIDE);
        scaffold.add("content/assets/workflow.svg", WORKFLOW);
    }
    scaffold.add("zel.yaml", config);
    scaffold.add("content/index.md", home);
    Ok(scaffold)
}

const HOME: &str = r#"Your documentation starts here. Edit this page, then follow the
[getting started guide](guide/getting-started.md#write-your-first-page).

## Build and share

Run these commands from the site root (the folder containing `zel.yaml`):

```sh
zel check --strict
zel build
```

The build prints the output path. Open its `index.html` directly in a browser;
navigation, local images and search work without an HTTP server. Share the whole
output folder so its pages, fonts and assets stay together.

## Preview as you write

```sh
zel view --watch
```

This builds the site, serves it locally, opens a browser and reloads after
successful edits. Press Ctrl-C to stop. To serve an existing export, use
`zel serve` followed by its output folder.
"#;

const GUIDE: &str = r#"# Getting started

## Write your first page

Create a Markdown file beside `index.md`, or in a subfolder. Add its relative
filename to `nav` in `zel.yaml`, or remove `nav` to use automatic navigation.
Pages omitted from `nav` are still built and included in search.

Use Markdown links to source filenames, such as [Home](../index.md#build-and-share).
The builder rewrites them to the generated HTML paths and checks heading anchors.

## Include local assets

Put images inside this source folder and reference them relative to the page:

![Markdown becomes a portable HTML site](../assets/workflow.svg)

The builder copies referenced local files into the output. Remote images and
links still need a network connection. Raw HTML is shown as text; MDX requires the hosted renderer. Math and Mermaid
rendering can be enabled with `markdown.math` and `markdown.mermaid` in `zel.yaml`.

## Make it yours

Set `name` and `theme` in `zel.yaml`. Run `zel list themes` for available themes.
Optional `brand.logo` settings can supply a local company logo and its website:

```yaml
brand:
  logo:
    src: assets/company.svg
    alt: Company
    href: https://example.com
```

Add your actual image before enabling that configuration. Keep generated output
outside this source folder, and run `zel check --strict` before sharing it.
"#;

const WORKFLOW: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" width="640" height="120" viewBox="0 0 640 120">
  <rect width="640" height="120" rx="12" fill="#194b3b"/>
  <text x="32" y="70" fill="white" font-family="sans-serif" font-size="26">Markdown → portable HTML</text>
</svg>
"##;
