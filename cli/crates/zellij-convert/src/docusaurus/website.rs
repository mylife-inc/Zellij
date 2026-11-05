//! Read URL settings only from inert configuration, never by executing JS.
use std::collections::{BTreeMap, BTreeSet};

use camino::{Utf8Path, Utf8PathBuf};
use serde_json::Value;
use zellij_core::{Error, Result};

use super::{Document, category_links};
use crate::{model::DocTree, references::route, source_urls::SourceUrls};

fn literal_config(raw: &str) -> Option<Value> {
    let mut text = raw.trim_start_matches('\u{feff}').trim();
    loop {
        if let Some(comment) = text.strip_prefix("//") {
            text = comment.split_once('\n')?.1.trim_start();
        } else if let Some(comment) = text.strip_prefix("/*") {
            text = comment.split_once("*/")?.1.trim_start();
        } else {
            break;
        }
    }
    let body = if let Some(rest) = text.strip_prefix("module.exports") {
        rest.trim_start().strip_prefix('=')?
    } else {
        text.strip_prefix("export default")?
    };
    json5::from_str(body.trim().trim_end_matches(';').trim()).ok()
}

fn field<'a>(value: &'a Value, key: &str, default: &'a str) -> Result<&'a str> {
    match value.get(key) {
        None => Ok(default),
        Some(Value::String(value)) => Ok(value),
        _ => Err(Error::InvalidArgument(format!(
            "Docusaurus {key} must be a string"
        ))),
    }
}

fn unsupported(notes: &mut Vec<String>, why: &str) -> Option<SourceUrls> {
    notes.push(format!("Source website URL mapping was skipped: {why}. Source JavaScript is never executed; review original website links manually. Document-file links still convert."));
    None
}

pub(super) fn read(
    project: &Utf8Path,
    raw: Option<&str>,
    tree: &DocTree,
    documents: &BTreeMap<Utf8PathBuf, Document>,
    pages: &category_links::Pages,
    notes: &mut Vec<String>,
) -> Result<Option<SourceUrls>> {
    let config = match raw {
        None => serde_json::json!({}),
        Some(raw) => match literal_config(raw) {
            Some(value @ Value::Object(_)) => value,
            _ => {
                return Ok(unsupported(
                    notes,
                    "configuration is not a direct literal module.exports/export default object",
                ));
            }
        },
    };
    if project.join("versions.json").exists() || config.get("i18n").is_some() {
        return Ok(unsupported(
            notes,
            "versioned or localized source routing is not supported",
        ));
    }
    if config
        .get("staticDirectories")
        .is_some_and(|value| value != &serde_json::json!(["static"]))
    {
        return Ok(unsupported(
            notes,
            "custom staticDirectories are not supported",
        ));
    }

    let mut selections = Vec::new();
    for (key, names) in [
        ("presets", ["classic", "@docusaurus/preset-classic"]),
        (
            "plugins",
            ["content-docs", "@docusaurus/plugin-content-docs"],
        ),
    ] {
        let Some(entries) = config.get(key) else {
            continue;
        };
        let Some(entries) = entries.as_array() else {
            return Ok(unsupported(notes, "presets/plugins are not literal arrays"));
        };
        for entry in entries {
            let (name, options) = match entry {
                Value::String(name) => (name.as_str(), serde_json::json!({})),
                Value::Array(pair) if pair.len() == 2 && pair[1].is_object() => {
                    (pair[0].as_str().unwrap_or(""), pair[1].clone())
                }
                _ => {
                    return Ok(unsupported(
                        notes,
                        "presets/plugins are not supported literal entries",
                    ));
                }
            };
            if !names.contains(&name) {
                return Ok(unsupported(
                    notes,
                    "custom presets/plugins may change source routing",
                ));
            }
            let docs = if key == "presets" {
                options
                    .get("docs")
                    .cloned()
                    .unwrap_or(serde_json::json!({}))
            } else {
                options
            };
            if !docs.is_object() {
                return Ok(unsupported(
                    notes,
                    "the docs plugin is disabled or has non-object options",
                ));
            }
            selections.push(docs);
        }
    }
    if selections.len() > 1 {
        return Ok(unsupported(
            notes,
            "multiple docs instances are not supported",
        ));
    }
    let options = selections.pop().unwrap_or(serde_json::json!({}));
    if field(&options, "path", "docs")? != "docs"
        || field(&options, "id", "default")? != "default"
        || [
            "versions",
            "lastVersion",
            "onlyIncludeVersions",
            "includeCurrentVersion",
            "numberPrefixParser",
        ]
        .iter()
        .any(|key| options.get(key).is_some())
    {
        return Ok(unsupported(
            notes,
            "custom docs paths, instances, versions or prefix parsers require review",
        ));
    }
    let origin = config
        .get("url")
        .map(|_| field(&config, "url", ""))
        .transpose()?;
    let mut urls = SourceUrls::new(origin, field(&config, "baseUrl", "/")?)?;
    let docs_base = field(&options, "routeBasePath", "docs")?;
    if docs_base.starts_with("//") {
        return Err(Error::InvalidArgument(
            "Docusaurus routeBasePath must be a local path".into(),
        ));
    }
    // Validate the route prefix as a local URL path before joining document URLs.
    let docs_base = super::slugs::category(
        &format!("/{}", docs_base.trim_matches('/')),
        Utf8Path::new("docusaurus.config"),
    )?;
    let docs_base = docs_base.strip_prefix("/guide").unwrap();
    let listed: BTreeSet<_> = tree.pages().into_iter().collect();
    for (source, doc) in documents {
        let slug = match &doc.route {
            Some(route) => route.strip_prefix("/guide").unwrap().to_owned(),
            None => {
                // Upstream uses the physical filename for index detection,
                // even when front matter supplies a different document ID.
                let stem = source.file_stem().unwrap();
                let parent = source.parent().and_then(Utf8Path::file_name).unwrap_or("");
                let index = stem.eq_ignore_ascii_case("index")
                    || stem.eq_ignore_ascii_case("readme")
                    || stem.eq_ignore_ascii_case(parent);
                let path = if index {
                    doc.id.rsplit_once('/').map_or("", |(parent, _)| parent)
                } else {
                    &doc.id
                };
                format!("/{path}")
            }
        };
        let target = (!doc.excluded && listed.contains(source)).then(|| {
            tree.routes
                .get(source)
                .cloned()
                .unwrap_or_else(|| route(source))
        });
        urls.add(&format!("{docs_base}{slug}"), target)?;
    }
    for (source, slug) in pages.explicit_routes() {
        urls.add(
            &format!("{docs_base}{}", slug.strip_prefix("/guide").unwrap()),
            tree.routes.get(source).cloned(),
        )?;
    }
    Ok(Some(urls))
}
