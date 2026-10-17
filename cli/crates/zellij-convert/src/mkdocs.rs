//! MkDocs.
//!
//! The easiest of the formats and so the one that proves the plugin boundary:
//! everything here is about reading `mkdocs.yml`, and nothing about Zellij.

use camino::{Utf8Path, Utf8PathBuf};
use serde_yaml_ng::Value;
use walkdir::WalkDir;

use crate::model::{DocTree, Entry};
use crate::{Confidence, Conversion, ConvertOptions, Converter, emit::emit};

/// MkDocs configuration is YAML with Python in it.
///
/// `!!python/name:material.extensions.emoji.twemoji` and friends are perfectly
/// ordinary in a Material theme config, and a YAML parser that has never heard
/// of Python rejects the whole document over them. Since none of it is
/// navigation, the tags are stripped before parsing rather than supported.
fn strip_python_tags(source: &str) -> String {
    source
        .lines()
        .map(|line| match line.find("!!python/") {
            Some(index) => {
                let (before, _) = line.split_at(index);
                format!("{before}\"\"")
            }
            None => line.to_owned(),
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Reads one `nav:` entry.
///
/// MkDocs allows three shapes, and a real config mixes them:
///
/// ```yaml
/// nav:
///   - index.md                    # a bare path
///   - Home: index.md              # a labelled page
///   - Guides:                     # a group
///     - guides/install.md
/// ```
fn parse_entry(value: &Value) -> Option<Entry> {
    match value {
        Value::String(path) => Some(Entry::Page {
            label: None,
            source: Utf8PathBuf::from(path),
        }),

        Value::Mapping(mapping) => {
            let (key, child) = mapping.iter().next()?;
            let label = key.as_str()?.to_owned();

            match child {
                // `Label: path.md`, or `Label: https://…` for a link out.
                Value::String(target) => {
                    if target.starts_with("http://") || target.starts_with("https://") {
                        Some(Entry::Foreign {
                            label,
                            target: target.clone(),
                        })
                    } else {
                        Some(Entry::Page {
                            label: Some(label),
                            source: Utf8PathBuf::from(target),
                        })
                    }
                }

                // `Label:` followed by a list is a group.
                Value::Sequence(items) => Some(Entry::Group {
                    label,
                    items: items.iter().filter_map(parse_entry).collect(),
                }),

                _ => None,
            }
        }

        _ => None,
    }
}

fn config_path(source: &Utf8Path) -> Option<Utf8PathBuf> {
    ["mkdocs.yml", "mkdocs.yaml"]
        .iter()
        .map(|name| source.join(name))
        .find(|path| path.exists())
}

pub struct MkDocs;

impl Converter for MkDocs {
    fn id(&self) -> &'static str {
        "mkdocs"
    }

    fn label(&self) -> &'static str {
        "MkDocs — mkdocs.yml with a nav tree"
    }

    fn detect(&self, source: &Utf8Path) -> Confidence {
        if config_path(source).is_some() {
            // Its own configuration file, by name. Nothing else looks like this.
            Confidence::CERTAIN
        } else if source.join("docs").join("index.md").exists() {
            // A `docs/` folder with an index is the MkDocs default layout, but
            // it is also what half the repositories on the internet look like.
            Confidence::LIKELY
        } else {
            Confidence::NONE
        }
    }

    fn convert(
        &self,
        source: &Utf8Path,
        options: &ConvertOptions,
    ) -> zellij_core::Result<Conversion> {
        let config = config_path(source).ok_or_else(|| zellij_core::Error::UnknownFormat {
            path: source.to_owned(),
        })?;

        let config =
            zellij_core::files::contained_path(source, Utf8Path::new(config.file_name().unwrap()))?;
        let raw = std::fs::read_to_string(&config)?;
        let parsed: Value = serde_yaml_ng::from_str(&strip_python_tags(&raw)).map_err(|err| {
            zellij_core::Error::NoEngine {
                reason: format!("{config} is not valid YAML: {err}"),
            }
        })?;

        // `docs_dir` is configurable; `docs` is the default.
        let docs_dir = parsed
            .get("docs_dir")
            .and_then(Value::as_str)
            .unwrap_or("docs");
        let root = if docs_dir == "." {
            source.to_owned()
        } else {
            zellij_core::files::contained_path(source, Utf8Path::new(docs_dir))?
        };

        let nav: Vec<Entry> = parsed
            .get("nav")
            .and_then(Value::as_sequence)
            .map(|entries| entries.iter().filter_map(parse_entry).collect())
            .unwrap_or_default();

        let mut tree = DocTree {
            site_name: parsed
                .get("site_name")
                .and_then(Value::as_str)
                .map(str::to_owned),
            description: parsed
                .get("site_description")
                .and_then(Value::as_str)
                .map(str::to_owned),
            root: root.clone(),
            nav,
            orphans: Vec::new(),
        };

        // A config with no `nav:` is legal — MkDocs then derives one from the
        // folder. Deriving the same way is better than converting nothing.
        let derived = tree.nav.is_empty();
        let listed: Vec<String> = tree
            .pages()
            .iter()
            .map(|path| path.as_str().to_owned())
            .collect();

        for entry in WalkDir::new(&root)
            .sort_by_file_name()
            .into_iter()
            .flatten()
        {
            let path = match Utf8PathBuf::from_path_buf(entry.path().to_path_buf()) {
                Ok(path) => path,
                Err(_) => continue,
            };

            if !path.as_str().ends_with(".md") && !path.as_str().ends_with(".markdown") {
                continue;
            }

            let Ok(relative) = path.strip_prefix(&root) else {
                continue;
            };
            let relative = relative.to_owned();

            if listed.contains(&relative.as_str().to_owned()) {
                continue;
            }

            if derived {
                tree.nav.push(Entry::Page {
                    label: None,
                    source: relative,
                });
            } else {
                tree.orphans.push(relative);
            }
        }

        let documents = crate::read_documents(&tree, false)?;
        let mut conversion = emit(&tree, |relative| documents.get(relative).cloned());
        crate::references::rewrite(&tree, &documents, None, options, &mut conversion)?;

        if derived {
            conversion.notes.push(
                "mkdocs.yml declared no `nav:`, so the sidebar was derived from the folder, \
                 in the order MkDocs itself would have used."
                    .into(),
            );
        }

        Ok(conversion)
    }
}

inventory::submit! {
    &MkDocs as &'static dyn Converter
}

#[cfg(test)]
mod tests {
    use super::*;

    fn value(yaml: &str) -> Value {
        serde_yaml_ng::from_str(yaml).unwrap()
    }

    #[test]
    fn reads_a_bare_path() {
        let entry = parse_entry(&value("index.md")).unwrap();
        assert_eq!(
            entry,
            Entry::Page {
                label: None,
                source: Utf8PathBuf::from("index.md")
            }
        );
    }

    #[test]
    fn reads_a_labelled_page() {
        let entry = parse_entry(&value("Home: index.md")).unwrap();
        assert_eq!(
            entry,
            Entry::Page {
                label: Some("Home".into()),
                source: Utf8PathBuf::from("index.md")
            }
        );
    }

    #[test]
    fn reads_a_group_and_keeps_its_nesting() {
        let entry = parse_entry(&value("Guides:\n  - a.md\n  - Deep:\n    - b.md")).unwrap();
        match entry {
            Entry::Group { label, items } => {
                assert_eq!(label, "Guides");
                assert_eq!(items.len(), 2);
                assert!(matches!(items[1], Entry::Group { .. }));
            }
            other => panic!("expected a group, got {other:?}"),
        }
    }

    #[test]
    fn treats_a_url_as_a_link_out_rather_than_a_page() {
        let entry = parse_entry(&value("Blog: https://example.com")).unwrap();
        assert!(matches!(entry, Entry::Foreign { .. }));
    }

    #[test]
    fn survives_the_python_tags_a_material_config_carries() {
        // Without stripping, one of these fails the whole document — and the
        // navigation, which is what the conversion is actually after, is
        // nowhere near them.
        let raw = "site_name: Acme\nmarkdown_extensions:\n  - pymdownx.emoji:\n      emoji_index: !!python/name:material.extensions.emoji.twemoji\nnav:\n  - index.md\n";
        let parsed: Value = serde_yaml_ng::from_str(&strip_python_tags(raw)).unwrap();
        assert_eq!(parsed.get("site_name").unwrap().as_str(), Some("Acme"));
        assert!(parsed.get("nav").is_some());
    }
}
