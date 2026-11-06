//! A browsable, self-contained section catalogue for `new docs --full`.
use crate::{DocsOptions, Scaffold, authoring::Section};
use serde_json::Value;
use zellij_core::{Error, Result, schema, yaml::string};

fn title(kind: &str) -> String {
    let words = kind.replace('-', " ");
    let mut chars = words.chars();
    chars
        .next()
        .map(|c| c.to_uppercase().collect::<String>() + chars.as_str())
        .unwrap_or_default()
}

fn sample_text(kind: &str, path: &str, original: &str) -> String {
    if original == "assets/replace-me.svg" {
        return "assets/section-example.svg".into();
    }
    if original == "assets/replace-me.mp4" {
        return "assets/section-example.mp4".into();
    }
    if original != "TODO" {
        return original.into();
    }
    let field = path.rsplit('.').next().unwrap();
    let number = path
        .split('.')
        .find_map(|p| p.parse::<usize>().ok())
        .unwrap_or(0)
        + 1;
    match field {
        "headline" => format!("{} example", title(kind)),
        "title" => format!("{} item {number}", title(kind)),
        "question" => "Where do I edit this example?".into(),
        "answer" => format!("Edit `content/sections/examples/{kind}.yaml`, then preview the site again."),
        "body" if kind == "markdown-prose" => "## Content you own\n\nWrite **Markdown** in this section, alongside the other section types.\n\n- Keep the source in your repository.\n- Replace this example with your own explanation.".into(),
        "body" => "zel check --strict --deep".into(),
        "name" if kind == "pricing-table" => "Community".into(),
        "name" => "Example teammate".into(),
        "price" => "$0".into(),
        "value" => number.to_string(),
        "label" => format!("Example {number}"),
        "date" => format!("Step {number}"),
        "quote" => "Good documentation makes a product easier to understand.".into(),
        "author" => "Example customer".into(),
        _ => "Included".into(),
    }
}

fn populate(section: &mut Section, kind: &str, value: &Value, path: &str) -> Result<()> {
    match value {
        Value::Object(fields) => {
            for (key, value) in fields {
                if key != "type" {
                    populate(
                        section,
                        kind,
                        value,
                        &if path.is_empty() {
                            key.clone()
                        } else {
                            format!("{path}.{key}")
                        },
                    )?;
                }
            }
        }
        Value::Array(items) => {
            for (i, value) in items.iter().enumerate() {
                populate(section, kind, value, &format!("{path}.{i}"))?;
            }
        }
        Value::String(text) => section.set(path, &sample_text(kind, path, text))?,
        _ => {}
    }
    Ok(())
}

/// Standard home page, an example catalogue and one page per registered type.
/// Examples come from the same engine-owned templates as `new section`.
pub fn configured_full_docs(options: &DocsOptions) -> Result<Scaffold> {
    if options.minimal {
        return Err(Error::InvalidArgument(
            "--full cannot be combined with --minimal".into(),
        ));
    }
    let types = schema::section_types();
    let mut scaffold = Scaffold::new();
    for (path, body) in crate::docs(options).files() {
        let body = match path.as_str() {
            "pages.yaml" => {
                let mut pages = body.to_owned();
                pages.push_str("\n- name: examples\n  title: Section examples\n  description: Browse editable examples of every section type.\n  sections:\n    - examples/catalogue\n");
                for kind in &types {
                    pages.push_str(&format!("\n- name: example-{kind}\n  title: {}\n  description: {}\n  sections:\n    - examples/{kind}\n", string(&title(kind)), string(&format!("Editable {kind} section example."))));
                }
                pages
            }
            "menu.yaml" => format!(
                "- {{ name: Home, page: home }}\n- {{ name: Examples, page: examples }}\n{}",
                if options.guide {
                    "- { name: Guide, page: 01-guide }\n"
                } else {
                    ""
                }
            ),
            _ => body.to_owned(),
        };
        scaffold.add(format!("content/{path}"), body);
    }
    scaffold.add("zel.yaml", "version: 1\nrenderer: hosted\n");
    let mut catalogue = String::from(
        "type: markdown-prose\nbody: |\n  ## Section examples\n\n  Each page is an editable example. Change its YAML file under `content/sections/examples/`.\n  Sample images and video are included locally; replace them before publishing.\n\n",
    );
    for kind in &types {
        let mut section = Section::new(kind)?;
        let value = section.value().clone();
        populate(&mut section, kind, &value, "")?;
        let body = section.render()?.replace(
            "# Media paths are placeholders. Add your own files under content/assets\n# and update these paths before publishing.",
            "# Sample media is included under content/assets. Replace it before publishing.",
        ).replace("assets/replace-me.svg", "assets/section-example.svg").replace("assets/replace-me.mp4", "assets/section-example.mp4");
        scaffold.add(format!("content/sections/examples/{kind}.yaml"), body);
        catalogue.push_str(&format!(
            "  - [{}](/example-{kind}) — `content/sections/examples/{kind}.yaml`\n",
            title(kind)
        ));
    }
    scaffold.add("content/sections/examples/catalogue.yaml", catalogue);
    scaffold.add(
        "content/assets/section-example.svg",
        include_str!("../assets/section-example.svg"),
    );
    scaffold.add_bytes(
        "content/assets/section-example.mp4",
        include_bytes!("../assets/section-example.mp4").as_slice(),
    );
    Ok(scaffold)
}
