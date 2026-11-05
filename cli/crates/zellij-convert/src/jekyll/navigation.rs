use super::{
    Collection, Page,
    config::{self, invalid, string},
    report,
};
use crate::{Unmapped, model::Entry, source_urls::SourceUrls};
use camino::{Utf8Path, Utf8PathBuf};
use serde_yaml_ng::Value;
use std::collections::BTreeMap;
use zellij_core::{Result, files};

fn items(
    value: &Value,
    lookup: &SourceUrls,
    unmapped: &mut Vec<Unmapped>,
    depth: usize,
) -> Result<Vec<Entry>> {
    if depth > 32 {
        return Err(invalid("navigation nesting is too deep"));
    }
    let mut result = Vec::new();
    for item in value
        .as_sequence()
        .ok_or_else(|| invalid("navigation must be an array"))?
    {
        let mapping = item
            .as_mapping()
            .ok_or_else(|| invalid("navigation entries must be mappings"))?;
        for (a, b) in [("title", "name"), ("url", "link")] {
            if mapping.contains_key(a) && mapping.contains_key(b) {
                return Err(invalid(format!(
                    "navigation must not contain both {a} and {b}"
                )));
            }
        }
        if mapping.keys().any(|k| {
            !matches!(
                k.as_str(),
                Some("title" | "name" | "url" | "link" | "children")
            )
        }) {
            return Err(invalid(
                "unsupported navigation fields; use name/link or title/url with optional children",
            ));
        }
        let label = string(item, "title")?
            .or(string(item, "name")?)
            .filter(|v| !v.is_empty())
            .map(str::to_owned);
        let url = string(item, "url")?.or(string(item, "link")?);
        let mut children = Vec::new();
        if let Some(url) = url {
            let end = url.find(['?', '#']).unwrap_or(url.len());
            let path = &url[..end];
            if path.contains(['{', '}']) {
                return Err(invalid("Liquid navigation URLs require manual migration"));
            }
            if let Some(key) = lookup.path(path)? {
                match lookup.document(&key) {
                    Some(Some(source)) => {
                        if end < url.len() {
                            report(
                                unmapped,
                                Utf8Path::new(source),
                                "navigation query/fragment is not retained in page navigation",
                            );
                        }
                        children.push(Entry::Page {
                            source: source.into(),
                            label: label.clone(),
                        });
                    }
                    Some(None) => report(
                        unmapped,
                        Utf8Path::new("_data/navigation"),
                        format!("{url}: excluded or ambiguous document; not added to navigation"),
                    ),
                    None => {
                        return Err(invalid(format!(
                            "navigation URL {url:?} does not match a supported Markdown page"
                        )));
                    }
                }
            } else if path.starts_with("//") || path.contains(':') {
                children.push(Entry::Foreign {
                    label: label.clone().unwrap_or_else(|| url.into()),
                    target: url.into(),
                });
            } else {
                return Err(invalid(
                    "navigation links must be absolute local URLs or external URLs",
                ));
            }
        }
        if let Some(nested) = item.get("children") {
            children.extend(items(nested, lookup, unmapped, depth + 1)?);
            let label =
                label.ok_or_else(|| invalid("navigation groups require a title or name"))?;
            if !children.is_empty() {
                result.push(Entry::Group {
                    label,
                    items: children,
                });
            }
        } else if children.is_empty() && url.is_none() {
            return Err(invalid("navigation entry needs a URL or children"));
        } else {
            result.extend(children);
        }
    }
    Ok(result)
}
pub(super) fn read(
    root: &Utf8Path,
    config: &Value,
    lookup: &SourceUrls,
    all: &BTreeMap<Utf8PathBuf, Page>,
    collections: &BTreeMap<String, Collection>,
    notes: &mut Vec<String>,
    unmapped: &mut Vec<Unmapped>,
) -> Result<Vec<Entry>> {
    let data = config::directory(root, string(config, "data_dir")?.unwrap_or("_data"))?;
    let paths: Vec<_> = ["navigation.yml", "navigation.yaml", "navigation.json"]
        .into_iter()
        .filter(|f| std::fs::symlink_metadata(data.join(f)).is_ok())
        .collect();
    if paths.len() > 1 {
        return Err(invalid("multiple navigation data files; keep exactly one"));
    }
    if let Some(path) = paths.first() {
        let value = config::yaml(&std::fs::read_to_string(files::contained_path(
            &data,
            Utf8Path::new(path),
        )?)?)?;
        let value = if let Some(map) = value.as_mapping() {
            if map.len() != 1 || !map.contains_key("main") {
                return Err(invalid(
                    "navigation mappings must contain only main; consolidate theme-specific navigation in a source copy",
                ));
            }
            &map["main"]
        } else {
            &value
        };
        notes.push("Navigation came from the explicit _data/navigation array (or main array). Unlisted Markdown pages are reported rather than added automatically. Linked groups place their page first.".into());
        return items(value, lookup, unmapped, 1);
    }
    notes.push("No supported navigation data file was present. Navigation was derived from published Markdown pages in filename order, then enabled collections in name order. Collection order lists place their named pages first; other pages follow in filename order.".into());
    let mut result: Vec<_> = all
        .iter()
        .filter(|(_, p)| p.published && p.collection.is_none())
        .map(|(source, _)| Entry::Page {
            label: None,
            source: source.clone(),
        })
        .collect();
    for (name, collection) in collections {
        if !collection.output || ["posts", "drafts"].contains(&name.as_str()) {
            continue;
        }
        let mut pages: Vec<_> = all
            .iter()
            .filter(|(_, p)| p.published && p.collection.as_deref() == Some(name))
            .map(|(p, _)| p.clone())
            .collect();
        let listed: Vec<_> = collection
            .order
            .iter()
            .map(|p| Utf8PathBuf::from(format!("_{name}/{p}")))
            .collect();
        for p in &listed {
            if !all.contains_key(p) {
                return Err(invalid(format!(
                    "collection order references missing page {p}"
                )));
            }
        }
        pages.sort_by_key(|p| listed.iter().position(|v| v == p).unwrap_or(usize::MAX));
        if !pages.is_empty() {
            result.push(Entry::Group {
                label: name.clone(),
                items: pages
                    .into_iter()
                    .map(|source| Entry::Page {
                        label: None,
                        source,
                    })
                    .collect(),
            });
        }
    }
    Ok(result)
}
