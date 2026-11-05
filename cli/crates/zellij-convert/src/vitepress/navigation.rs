use std::collections::{BTreeMap, BTreeSet};

use camino::{Utf8Path, Utf8PathBuf};
use serde_json::Value;
use zellij_core::Result;

use super::config::{invalid, string};
use crate::{Unmapped, model::Entry, references::decode};

fn external(link: &str) -> bool {
    let path = link.split(['?', '#']).next().unwrap_or(link);
    path.starts_with("//") || path.contains(':')
}

fn page(
    link: &str,
    label: Option<String>,
    documents: &BTreeMap<Utf8PathBuf, String>,
    unmapped: &mut Vec<Unmapped>,
) -> Result<Entry> {
    if external(link) {
        return Ok(Entry::Foreign {
            label: label.unwrap_or_else(|| link.to_owned()),
            target: link.to_owned(),
        });
    }
    let end = link.find(['?', '#']).unwrap_or(link.len());
    let path = link[..end]
        .split('/')
        .map(|part| {
            let part =
                decode(part).ok_or_else(|| invalid("sidebar link contains malformed encoding"))?;
            if part.contains(['/', '\\', ':', '?', '#']) || part.chars().any(char::is_control) {
                return Err(invalid(
                    "sidebar link contains encoded separators or reserved characters",
                ));
            }
            Ok(part)
        })
        .collect::<Result<Vec<_>>>()?
        .join("/");
    let path = path.trim_start_matches('/');
    // Unlike links inside Markdown, sidebar paths always address the source
    // root. No ancestor traversal, drive roots, or hidden source reads.
    if !path.is_empty() {
        zellij_core::files::relative_path(Utf8Path::new(path.trim_end_matches('/')))?;
    }
    let candidates: Vec<Utf8PathBuf> = if path.is_empty() || path.ends_with('/') {
        vec![format!("{path}index.md").into()]
    } else if path.ends_with(".md") {
        vec![path.into()]
    } else if let Some(stem) = path.strip_suffix(".html") {
        vec![format!("{stem}.md").into()]
    } else {
        vec![format!("{path}.md").into()]
    };
    let source = candidates
        .iter()
        .find(|p| documents.contains_key(*p))
        .ok_or_else(|| {
            invalid(format!(
                "sidebar link {link:?} does not match an imported Markdown source file"
            ))
        })?;
    if end < link.len() {
        unmapped.push(Unmapped { source: source.clone(), reason: format!("sidebar query/fragment in {link:?} is not preserved in page navigation; page content is imported") });
    }
    Ok(Entry::Page {
        label,
        source: source.clone(),
    })
}

fn items(
    value: &Value,
    inherited_base: &str,
    documents: &BTreeMap<Utf8PathBuf, String>,
    unmapped: &mut Vec<Unmapped>,
    depth: usize,
) -> Result<Vec<Entry>> {
    if depth > 6 {
        return Err(invalid(
            "sidebar nesting beyond VitePress's six displayed levels is not supported",
        ));
    }
    let entries = value
        .as_array()
        .ok_or_else(|| invalid("sidebar items must be an array"))?;
    let mut out = Vec::new();
    for item in entries {
        let object = item
            .as_object()
            .ok_or_else(|| invalid("each sidebar item must be a literal object"))?;
        let label = string(item, "text")?
            .filter(|v| !v.is_empty())
            .map(str::to_owned);
        let base = string(item, "base")?
            .filter(|v| !v.is_empty())
            .unwrap_or(inherited_base);
        let mut children = Vec::new();
        if let Some(link) = string(item, "link")? {
            if link.is_empty() {
                return Err(invalid("sidebar link must not be empty"));
            }
            children.push(page(
                &format!("{base}{link}"),
                label.clone(),
                documents,
                unmapped,
            )?);
        }
        if let Some(nested) = item.get("items") {
            children.extend(items(nested, base, documents, unmapped, depth + 1)?);
            let label = label.ok_or_else(|| invalid("sidebar groups require text"))?;
            if !children.is_empty() {
                out.push(Entry::Group {
                    label,
                    items: children,
                });
            }
        } else if children.is_empty() {
            unmapped.push(Unmapped {
                source: ".vitepress/config".into(),
                reason: format!(
                    "sidebar label {:?} has no link or children; not imported",
                    label.as_deref().unwrap_or("")
                ),
            });
        } else {
            out.extend(children);
        }
        let extra: BTreeSet<_> = object
            .keys()
            .filter(|k| !["text", "link", "items", "base"].contains(&k.as_str()))
            .collect();
        if !extra.is_empty() {
            unmapped.push(Unmapped {
                source: ".vitepress/config".into(),
                reason: format!("sidebar presentation fields {extra:?} require manual review"),
            });
        }
    }
    Ok(out)
}

pub(super) fn read(
    config: &Value,
    documents: &BTreeMap<Utf8PathBuf, String>,
    notes: &mut Vec<String>,
    unmapped: &mut Vec<Unmapped>,
) -> Result<Vec<Entry>> {
    let theme = config.get("themeConfig");
    if theme.is_some_and(|t| !t.is_object()) {
        return Err(invalid("themeConfig must be an object"));
    }
    let sidebar = theme.and_then(|t| t.get("sidebar"));
    match sidebar {
        None => {
            notes.push("VitePress declares no sidebar; navigation was derived from visible Markdown files in filename order.".into());
            Ok(documents
                .keys()
                .map(|source| Entry::Page {
                    label: None,
                    source: source.clone(),
                })
                .collect())
        }
        Some(Value::Bool(false)) => Ok(Vec::new()),
        Some(value @ Value::Array(_)) => items(value, "", documents, unmapped, 1),
        Some(Value::Object(groups)) => {
            let mut nav = Vec::new();
            for (prefix, slice) in groups {
                if prefix.is_empty() || external(prefix) {
                    return Err(invalid(
                        "multiple-sidebar keys must be local route prefixes",
                    ));
                }
                let entries = if slice.is_array() {
                    items(slice, "", documents, unmapped, 1)?
                } else {
                    let array = slice
                        .get("items")
                        .ok_or_else(|| invalid("multiple-sidebar values need an items array"))?;
                    items(
                        array,
                        string(slice, "base")?.unwrap_or(""),
                        documents,
                        unmapped,
                        1,
                    )?
                };
                if !entries.is_empty() {
                    nav.push(Entry::Group {
                        label: prefix.clone(),
                        items: entries,
                    });
                }
            }
            notes.push("VitePress's route-specific sidebars were combined into one guide, grouped by route prefix in lexical order. Linked groups place their page first; repeated pages are emitted once.".into());
            Ok(nav)
        }
        _ => Err(invalid(
            "sidebar must be an array, a literal multiple-sidebar object, or false",
        )),
    }
}
