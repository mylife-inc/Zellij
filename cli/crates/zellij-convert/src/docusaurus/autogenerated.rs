//! Expand data-only Docusaurus sidebar slices from the validated inventory.
//! Source paths select folders; document IDs never become filesystem paths.

use std::collections::BTreeMap;

use camino::{Utf8Path, Utf8PathBuf};
use serde_yaml_ng::Value;
use zellij_core::{Error, Result, files::contained_path};

use super::{DocIndex, Document, number_prefix};
use crate::{Unmapped, model::Entry};

#[derive(Default)]
enum Link {
    #[default]
    Convention,
    None,
    Doc(String),
    Generated,
}

#[derive(Default)]
struct Category {
    label: Option<String>,
    position: Option<f64>,
    link: Link,
}

fn invalid(path: &Utf8Path, message: impl std::fmt::Display) -> Error {
    Error::InvalidArgument(format!("{path}: invalid category metadata: {message}"))
}

// Import paths use '/' on every OS; native Path::join uses '\\' on Windows,
// which the shared containment boundary intentionally refuses in input paths.
fn child(dir: &Utf8Path, name: &str) -> Utf8PathBuf {
    if dir.as_str().is_empty() {
        name.into()
    } else {
        format!("{dir}/{name}").into()
    }
}

fn string(value: Option<&Value>, path: &Utf8Path, field: &str) -> Result<Option<String>> {
    match value {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(text)) if !text.trim().is_empty() => Ok(Some(text.clone())),
        _ => Err(invalid(path, format!("{field} must be a nonempty string"))),
    }
}

fn category(root: &Utf8Path, dir: &Utf8Path, unmapped: &mut Vec<Unmapped>) -> Result<Category> {
    let mut found = None;
    for name in ["_category_.json", "_category_.yml", "_category_.yaml"] {
        let relative = child(dir, name);
        let full = contained_path(root, &relative)?;
        if full.exists() {
            if let Some((previous, _)) = &found {
                return Err(invalid(
                    &relative,
                    format!("multiple category files; also found {previous}"),
                ));
            }
            found = Some((relative, full));
        }
    }
    let Some((relative, full)) = found else {
        return Ok(Category::default());
    };
    let raw = std::fs::read_to_string(full).map_err(|e| invalid(&relative, e))?;
    let raw = raw.strip_prefix('\u{feff}').unwrap_or(&raw);
    let value: Value = if relative.extension() == Some("json") {
        let json: serde_json::Value =
            serde_json::from_str(raw).map_err(|e| invalid(&relative, e))?;
        serde_yaml_ng::to_value(json).map_err(|e| invalid(&relative, e))?
    } else {
        serde_yaml_ng::from_str(raw).map_err(|e| invalid(&relative, e))?
    };
    let fields = value
        .as_mapping()
        .ok_or_else(|| invalid(&relative, "expected an object"))?;
    let label = string(value.get("label"), &relative, "label")?;
    let position = match value.get("position") {
        None | Some(Value::Null) => None,
        Some(Value::Number(n)) if n.as_f64().is_some_and(f64::is_finite) => n.as_f64(),
        _ => return Err(invalid(&relative, "position must be a finite number")),
    };
    let link = match value.get("link") {
        None => Link::Convention,
        Some(Value::Null) => Link::None,
        Some(link) => match link.get("type").and_then(Value::as_str) {
            Some("doc") => Link::Doc(
                string(link.get("id"), &relative, "link.id")?
                    .ok_or_else(|| invalid(&relative, "link.id is required"))?,
            ),
            Some("generated-index") => {
                unmapped.push(Unmapped { source: relative.clone(), reason: "generated category landing page is not created; add a Zellij page and review its source title, description and slug".into() });
                Link::Generated
            }
            _ => {
                return Err(invalid(
                    &relative,
                    "link must be null, a doc link, or a generated-index link",
                ));
            }
        },
    };
    let mut unsupported = Vec::new();
    for key in fields.keys() {
        let key = key
            .as_str()
            .ok_or_else(|| invalid(&relative, "keys must be strings"))?;
        if !matches!(key, "label" | "position" | "link") {
            unsupported.push(key);
        }
    }
    if !unsupported.is_empty() {
        unsupported.sort_unstable();
        unmapped.push(Unmapped {
            source: relative,
            reason: format!(
                "category fields not mapped: {}; review the original metadata",
                unsupported.join(", ")
            ),
        });
    }
    Ok(Category {
        label,
        position,
        link,
    })
}

type Docs<'a> = Vec<(&'a Utf8PathBuf, &'a Document)>;

fn linked_document<'a>(
    dir: &Utf8Path,
    docs: &Docs<'a>,
    link: &Link,
    index: &DocIndex<'a>,
) -> Result<Option<&'a Utf8PathBuf>> {
    match link {
        Link::None | Link::Generated => Ok(None),
        Link::Convention => Ok(docs
            .iter()
            .find(|(path, _)| {
                let name = path.file_stem().unwrap();
                name.eq_ignore_ascii_case("index")
                    || name.eq_ignore_ascii_case("readme")
                    || name.eq_ignore_ascii_case(dir.file_name().unwrap())
            })
            .map(|(path, _)| *path)),
        Link::Doc(id) => {
            // Docusaurus first searches immediate children by local ID, then
            // the full document ID. Folder names alone cannot reconstruct IDs
            // because documents may disable numeric-prefix parsing.
            let matches: Vec<_> = index
                .documents
                .iter()
                .filter(|(path, doc)| {
                    path.parent() == Some(dir) && doc.id.rsplit('/').next() == Some(id)
                })
                .collect();
            if matches.len() > 1 {
                return Err(invalid(
                    dir,
                    format!("ambiguous local category document ID {id:?}"),
                ));
            }
            let path = matches
                .first()
                .map(|(path, _)| *path)
                .or_else(|| index.ids.get(id.as_str()).copied())
                .ok_or_else(|| {
                    invalid(dir, format!("category document ID {id:?} does not exist"))
                })?;
            Ok((!index.documents[path].excluded).then_some(path))
        }
    }
}

struct Ordered {
    position: Option<f64>,
    source: String,
    entry: Entry,
}

fn slice(
    root: &Utf8Path,
    dir: &Utf8Path,
    docs: Docs<'_>,
    index: &DocIndex<'_>,
    unmapped: &mut Vec<Unmapped>,
) -> Result<Vec<Entry>> {
    let mut direct = Vec::new();
    let mut folders: BTreeMap<String, Docs<'_>> = BTreeMap::new();
    for (path, doc) in docs {
        let relative = path
            .strip_prefix(dir)
            .expect("slice contains its documents");
        if relative.parent().is_none_or(|p| p.as_str().is_empty()) {
            direct.push((path, doc));
        } else {
            folders
                .entry(relative.components().next().unwrap().as_str().to_owned())
                .or_default()
                .push((path, doc));
        }
    }
    let mut items: Vec<_> = direct
        .iter()
        .map(|(path, doc)| Ordered {
            position: doc.position,
            source: path.file_name().unwrap().to_owned(),
            entry: index
                .page(path, None)
                .expect("excluded documents were filtered"),
        })
        .collect();
    for (name, children) in folders {
        let child_dir = child(dir, &name);
        let metadata = category(root, &child_dir, unmapped)?;
        let immediate: Docs<'_> = children
            .iter()
            .copied()
            .filter(|(path, _)| path.parent() == Some(&child_dir))
            .collect();
        let linked = linked_document(&child_dir, &immediate, &metadata.link, index)?;
        let linked_doc = linked.map(|path| &index.documents[path]);
        let (prefix, fallback_label) = number_prefix(&name);
        let label = metadata
            .label
            .or_else(|| {
                linked_doc.map(|doc| doc.sidebar_label.as_ref().unwrap_or(&doc.title).clone())
            })
            .unwrap_or_else(|| fallback_label.to_owned());
        let position = metadata
            .position
            .or_else(|| linked_doc.and_then(|doc| doc.position))
            .or(prefix);
        let mut children = slice(root, &child_dir, children, index, unmapped)?;
        let entry = if let Some(linked) = linked {
            children.retain(|item| !matches!(item, Entry::Page { source, .. } if source == linked));
            if children.is_empty() {
                Entry::Page {
                    label: Some(label),
                    source: linked.clone(),
                }
            } else {
                children.insert(0, index.page(linked, None).unwrap());
                Entry::Group {
                    label,
                    items: children,
                }
            }
        } else {
            Entry::Group {
                label,
                items: children,
            }
        };
        items.push(Ordered {
            position,
            source: name,
            entry,
        });
    }
    items.sort_by(|a, b| {
        match (a.position, b.position) {
            (Some(a), Some(b)) => a.partial_cmp(&b).unwrap(),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => std::cmp::Ordering::Equal,
        }
        .then_with(|| a.source.encode_utf16().cmp(b.source.encode_utf16()))
    });
    Ok(items.into_iter().map(|item| item.entry).collect())
}

pub(super) fn generate(
    root: &Utf8Path,
    directory: &str,
    index: &DocIndex<'_>,
    unmapped: &mut Vec<Unmapped>,
) -> Result<Vec<Entry>> {
    let dir = if directory == "." {
        Utf8Path::new("")
    } else {
        Utf8Path::new(directory)
    };
    let full = if dir.as_str().is_empty() {
        root.to_owned()
    } else {
        contained_path(root, dir)?
    };
    if !full.is_dir() {
        return Err(Error::InvalidArgument(format!(
            "autogenerated sidebar dirName {directory:?} is not a documentation directory"
        )));
    }
    let docs: Docs<'_> = index
        .documents
        .iter()
        .filter(|(path, doc)| !doc.excluded && path.starts_with(dir))
        .collect();
    if docs.is_empty() {
        unmapped.push(Unmapped {
            source: directory.into(),
            reason: "autogenerated sidebar slice has no eligible documents".into(),
        });
    }
    slice(root, dir, docs, index, unmapped)
}
