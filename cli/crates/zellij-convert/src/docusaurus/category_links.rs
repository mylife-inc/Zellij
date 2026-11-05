//! Adapt category links to ordinary, editable Zellij pages and navigation.
use std::collections::{BTreeMap, BTreeSet};

use camino::{Utf8Path, Utf8PathBuf};
use serde_yaml_ng::Value;
use zellij_core::{Error, Result, yaml::string as yaml_string};

use super::{DocIndex, Document};
use crate::{
    Unmapped,
    model::Entry,
    references::{encode_path, route},
};

#[derive(Default)]
pub(super) enum Link {
    #[default]
    Convention,
    None,
    Doc(String),
    Generated(GeneratedIndex),
}

pub(super) struct GeneratedIndex {
    title: Option<String>,
    description: Option<String>,
}

fn invalid(source: &Utf8Path, message: impl std::fmt::Display) -> Error {
    Error::InvalidArgument(format!("{source}: invalid category link: {message}"))
}

fn text(value: Option<&Value>, source: &Utf8Path, key: &str) -> Result<Option<String>> {
    match value {
        None => Ok(None),
        Some(Value::String(value)) => Ok(Some(value.clone())),
        _ => Err(invalid(source, format!("{key} must be a string"))),
    }
}

pub(super) fn parse(
    value: Option<&Value>,
    source: &Utf8Path,
    unmapped: &mut Vec<Unmapped>,
) -> Result<Link> {
    let Some(value) = value else {
        return Ok(Link::Convention);
    };
    if value.is_null() {
        return Ok(Link::None);
    }
    let fields = value
        .as_mapping()
        .ok_or_else(|| invalid(source, "expected an object or null"))?;
    let (link, supported): (_, &[&str]) = match value.get("type").and_then(Value::as_str) {
        Some("doc") => {
            let id = text(value.get("id"), source, "id")?
                .filter(|id| !id.trim().is_empty())
                .ok_or_else(|| invalid(source, "id must be a nonempty string"))?;
            (Link::Doc(id), &["type", "id"])
        }
        Some("generated-index") => {
            let title = text(value.get("title"), source, "title")?;
            if title.as_ref().is_some_and(|title| title.trim().is_empty()) {
                return Err(invalid(source, "title must be a nonempty string"));
            }
            let description = text(value.get("description"), source, "description")?;
            (
                Link::Generated(GeneratedIndex { title, description }),
                &["type", "title", "description"],
            )
        }
        _ => return Err(invalid(source, "type must be doc or generated-index")),
    };
    let mut unsupported = Vec::new();
    for key in fields.keys() {
        let key = key
            .as_str()
            .ok_or_else(|| invalid(source, "keys must be strings"))?;
        if !supported.contains(&key) {
            unsupported.push(key)
        }
    }
    if !unsupported.is_empty() {
        unsupported.sort_unstable();
        unmapped.push(Unmapped {
            source: source.to_owned(),
            reason: format!(
                "category link fields not mapped: {}; review the original configuration",
                unsupported.join(", ")
            ),
        });
    }
    Ok(link)
}

/// Group headings aren't clickable in Zellij. Put the linked page first and
/// avoid repeating that same direct child. A lone index becomes one link.
pub(super) fn with_document(label: String, mut items: Vec<Entry>, linked: Entry) -> Entry {
    let Entry::Page { source, .. } = &linked else {
        unreachable!("category link is a page")
    };
    items.retain(|item| !matches!(item, Entry::Page { source: other, .. } if other == source));
    if items.is_empty() {
        Entry::Page {
            label: Some(label),
            source: source.clone(),
        }
    } else {
        items.insert(0, linked);
        Entry::Group { label, items }
    }
}

struct Page {
    title: String,
    description: Option<String>,
    items: Vec<Entry>,
}

pub(super) struct Pages {
    occupied: BTreeSet<String>,
    keys: BTreeMap<String, Utf8PathBuf>,
    pages: BTreeMap<Utf8PathBuf, Page>,
}

impl Pages {
    pub fn new(documents: &BTreeMap<Utf8PathBuf, Document>) -> Self {
        // Reserve even excluded/orphan sources, so generated pages cannot take
        // over their routes if the user later adds those documents to the nav.
        Self {
            occupied: documents.keys().map(route).collect(),
            keys: BTreeMap::new(),
            pages: BTreeMap::new(),
        }
    }

    pub fn category(
        &mut self,
        key: &str,
        label: String,
        items: Vec<Entry>,
        metadata: GeneratedIndex,
        source: &Utf8Path,
        unmapped: &mut Vec<Unmapped>,
    ) -> Entry {
        fn has_page(items: &[Entry]) -> bool {
            items.iter().any(|item| match item {
                Entry::Page { .. } => true,
                Entry::Group { items, .. } => has_page(items),
                Entry::Foreign { .. } => false,
            })
        }
        if !has_page(&items) {
            unmapped.push(Unmapped {
                source: source.to_owned(),
                reason:
                    "generated category page omitted because it has no eligible child documents"
                        .into(),
            });
            return Entry::Group { label, items };
        }
        let path = if let Some(path) = self.keys.get(key) {
            path.clone()
        } else {
            // Use the same normalization as the actual engine routes. Labels
            // are never used as filesystem paths; the allocated stem is ASCII.
            let normalized = route(&Utf8PathBuf::from(format!("{label}.md")));
            let mut stem = normalized
                .strip_prefix("/guide")
                .expect("guide route")
                .replace('/', "-")
                .trim_matches('-')
                .to_owned();
            if stem.is_empty() {
                stem = "overview".into();
            }
            if stem.starts_with(|c: char| c.is_ascii_digit()) {
                stem = format!("section-{stem}");
            }
            stem.truncate(80);
            let mut suffix = 1;
            let path = loop {
                let name = if suffix == 1 {
                    stem.clone()
                } else {
                    format!("{stem}-{suffix}")
                };
                let candidate = Utf8PathBuf::from(format!("category/{name}/index.md"));
                if self.occupied.insert(route(&candidate)) {
                    break candidate;
                }
                suffix += 1;
            };
            self.keys.insert(key.to_owned(), path.clone());
            path
        };
        let title = metadata.title.unwrap_or_else(|| label.clone());
        self.pages.entry(path.clone()).or_insert_with(|| Page {
            title: title.clone(),
            description: metadata.description,
            items: items.clone(),
        });
        let linked = Entry::Page {
            label: Some(title),
            source: path,
        };
        with_document(label, items, linked)
    }

    pub fn documents(&self, index: &DocIndex<'_>) -> BTreeMap<Utf8PathBuf, String> {
        self.pages
            .iter()
            .map(|(source, page)| {
                let mut raw = format!("---\ntitle: {}\n", yaml_string(&page.title));
                if let Some(description) = &page.description {
                    raw.push_str(&format!("description: {}\n", yaml_string(description)));
                }
                raw.push_str("---\n\n");
                if let Some(description) = &page.description {
                    raw.push_str(&format!("{}\n\n", literal(description)));
                }
                self.list(&page.items, source, index, 0, &mut raw);
                (source.clone(), raw)
            })
            .collect()
    }

    fn list(
        &self,
        items: &[Entry],
        source: &Utf8Path,
        index: &DocIndex<'_>,
        depth: usize,
        out: &mut String,
    ) {
        let indent = "  ".repeat(depth);
        for item in items {
            match item {
                Entry::Page {
                    label,
                    source: target,
                } => {
                    let title = label.as_deref().unwrap_or_else(|| {
                        self.pages
                            .get(target)
                            .map(|page| page.title.as_str())
                            .unwrap_or_else(|| index.documents[target].title.as_str())
                    });
                    out.push_str(&format!(
                        "{indent}- [{}](<{}>)\n",
                        literal(title),
                        relative_link(source, target)
                    ));
                }
                Entry::Group { label, items } => {
                    out.push_str(&format!("{indent}- {}\n", literal(label)));
                    self.list(items, source, index, depth + 1, out);
                }
                // Existing import policy reports foreign links for menu.yaml.
                Entry::Foreign { .. } => {}
            }
        }
    }
}

// Source metadata is plain text, not trusted Markdown/JSX. Numeric character
// references preserve punctuation and line breaks without creating MDX code.
fn literal(text: &str) -> String {
    let module_start = text.trim_start().starts_with("import ")
        || text.trim_start().starts_with("export ")
        || text.starts_with(' ');
    text.chars()
        .enumerate()
        .map(|(at, c)| {
            if (module_start && at == 0)
                || c.is_ascii_punctuation()
                || c.is_control()
                || matches!(c, '\u{85}' | '\u{2028}' | '\u{2029}')
            {
                format!("&#{};", c as u32)
            } else {
                c.to_string()
            }
        })
        .collect()
}

fn relative_link(source: &Utf8Path, target: &Utf8Path) -> String {
    let parent: Vec<_> = source.parent().unwrap().as_str().split('/').collect();
    let parts: Vec<_> = target.as_str().split('/').collect();
    let shared = parent
        .iter()
        .zip(&parts)
        .take_while(|(a, b)| a == b)
        .count();
    let mut path = "../".repeat(parent.len() - shared);
    path.push_str(&parts[shared..].join("/"));
    encode_path(&path)
}
