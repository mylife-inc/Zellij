//! Docusaurus.
//!
//! The most common of these tools in the wild and the hardest to read, because
//! its sidebar is a JavaScript *module* rather than a data file. Most of them
//! are only data wearing JavaScript's clothes:
//!
//! ```js
//! module.exports = {
//!   docs: [
//!     'intro',
//!     { type: 'category', label: 'Guides', items: ['guides/install'] },
//!   ],
//! };
//! ```
//!
//! Strip the export and that is JSON5. Some are not — they call
//! `require('./partial')`, build entries in a loop, or read the filesystem —
//! and for those this converter declines and says why. Half-evaluating
//! somebody's JavaScript is how you produce a sidebar that is wrong in a way
//! nobody notices for a month.

use camino::{Utf8Path, Utf8PathBuf};
use regex::Regex;
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::LazyLock,
};
use walkdir::WalkDir;
use zellij_core::{Error, Result};

use crate::model::{DocTree, Entry};
use crate::{Confidence, Conversion, ConvertOptions, Converter, Unmapped, emit::emit};

struct Document {
    raw: String,
    id: String,
    sidebar_label: Option<String>,
    excluded: bool,
}

// Docusaurus's default numberPrefixParser preserves date/version-like names.
// Custom JavaScript parsers are not executed by this data-only importer.
fn strip_number_prefix(name: &str) -> &str {
    static DATE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[0-9]+[-_.][0-9]+").unwrap());
    static PREFIX: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"^[0-9]+\s*[-_.]+\s*([^-_.\s].*)$").unwrap());
    if DATE.is_match(name) {
        return name;
    }
    PREFIX
        .captures(name)
        .and_then(|c| c.get(1))
        .map_or(name, |m| m.as_str())
}

fn document_id(path: &Utf8Path, metadata: &crate::frontmatter::Frontmatter<'_>) -> Result<String> {
    let prefixes = metadata.boolean("parse_number_prefixes")?.unwrap_or(true);
    let basename = path.file_stem().expect("document has a filename");
    let custom = metadata.string("id")?;
    let basename = custom.unwrap_or_else(|| {
        if prefixes {
            strip_number_prefix(basename)
        } else {
            basename
        }
    });
    if basename.contains(['/', '\\', '\0']) || matches!(basename, "." | "..") {
        return Err(Error::InvalidArgument(format!(
            "{path}: front matter id must be a single document name, without path separators or parent segments"
        )));
    }
    let mut parts: Vec<_> = path
        .parent()
        .unwrap_or(Utf8Path::new(""))
        .as_str()
        .split('/')
        .filter(|s| !s.is_empty())
        .map(|s| if prefixes { strip_number_prefix(s) } else { s })
        .collect();
    parts.push(basename);
    Ok(parts.join("/"))
}

fn read_inventory(root: &Utf8Path) -> Result<BTreeMap<Utf8PathBuf, Document>> {
    let mut documents = BTreeMap::new();
    for entry in WalkDir::new(root).sort_by_file_name() {
        let entry = entry
            .map_err(|e| Error::InvalidArgument(format!("cannot inspect Docusaurus docs: {e}")))?;
        let path = Utf8Path::from_path(entry.path()).ok_or_else(|| {
            Error::InvalidArgument("Docusaurus document paths must be UTF-8".into())
        })?;
        if path == root {
            continue;
        }
        let relative = path.strip_prefix(root).expect("walk stays under root");
        let relative = Utf8PathBuf::from(
            relative
                .components()
                .map(|part| part.as_str())
                .collect::<Vec<_>>()
                .join("/"),
        );
        // Validate directories as well so junctions cannot be traversed on
        // Windows, even if the OS reports them as ordinary directories.
        let full = zellij_core::files::contained_path(root, &relative)?;
        if !entry.file_type().is_file() || !matches!(relative.extension(), Some("md" | "mdx")) {
            continue;
        }
        let raw = std::fs::read_to_string(full)?;
        let metadata = crate::frontmatter::parse(&raw, &relative)?;
        let id = document_id(&relative, &metadata)?;
        let sidebar_label = metadata.string("sidebar_label")?.map(str::to_owned);
        let draft = metadata.boolean("draft")?.unwrap_or(false);
        let unlisted = metadata.boolean("unlisted")?.unwrap_or(false);
        let excluded = draft || unlisted;
        documents.insert(
            relative.to_owned(),
            Document {
                raw,
                id,
                sidebar_label,
                excluded,
            },
        );
    }
    Ok(documents)
}

fn resolve_navigation(
    tree: &mut DocTree,
    documents: &BTreeMap<Utf8PathBuf, Document>,
    derive_from_folder: bool,
) -> Result<Vec<Unmapped>> {
    let mut ids = BTreeMap::new();
    let mut paths = BTreeMap::new();
    for (path, doc) in documents {
        if let Some(previous) = ids.insert(doc.id.as_str(), path) {
            return Err(Error::InvalidArgument(format!(
                "duplicate Docusaurus document ID {:?}: {previous} and {path}",
                doc.id
            )));
        }
        let stem = path.with_extension("").to_string();
        if let Some(previous) = paths.insert(stem, path) {
            return Err(Error::InvalidArgument(format!(
                "ambiguous Markdown/MDX source files: {previous} and {path}"
            )));
        }
    }
    let mut excluded = BTreeSet::new();
    fn walk(
        entries: &mut Vec<Entry>,
        ids: &BTreeMap<&str, &Utf8PathBuf>,
        paths: &BTreeMap<String, &Utf8PathBuf>,
        documents: &BTreeMap<Utf8PathBuf, Document>,
        excluded: &mut BTreeSet<Utf8PathBuf>,
    ) -> Result<()> {
        let mut resolved = Vec::new();
        for mut entry in std::mem::take(entries) {
            match &mut entry {
                Entry::Page { label, source } => {
                    let id = source
                        .as_str()
                        .strip_suffix(".md")
                        .expect("sidebar parser supplies an ID marker");
                    let canonical = ids.get(id).copied();
                    let legacy = paths.get(id).copied();
                    if let (Some(a), Some(b)) = (canonical, legacy)
                        && a != b
                    {
                        return Err(Error::InvalidArgument(format!(
                            "ambiguous Docusaurus sidebar ID {id:?}: document ID selects {a}, but the legacy filename selects {b}"
                        )));
                    }
                    let path = canonical.or(legacy).ok_or_else(|| {
                        Error::InvalidArgument(format!(
                            "Docusaurus sidebar ID {id:?} does not match a source document"
                        ))
                    })?;
                    let doc = &documents[path];
                    if doc.excluded {
                        excluded.insert(path.clone());
                        continue;
                    }
                    *source = path.clone();
                    if let Some(authored) = &doc.sidebar_label {
                        *label = Some(authored.clone());
                    }
                }
                Entry::Group { items, .. } => walk(items, ids, paths, documents, excluded)?,
                Entry::Foreign { .. } => {}
            }
            resolved.push(entry);
        }
        *entries = resolved;
        Ok(())
    }
    if derive_from_folder {
        for (source, doc) in documents {
            if doc.excluded {
                excluded.insert(source.clone());
                continue;
            }
            tree.nav.push(Entry::Page {
                label: doc.sidebar_label.clone(),
                source: source.clone(),
            });
        }
    } else {
        walk(&mut tree.nav, &ids, &paths, documents, &mut excluded)?;
    }
    let listed: BTreeSet<_> = tree.pages().into_iter().cloned().collect();
    for (path, doc) in documents {
        if doc.excluded {
            excluded.insert(path.clone());
        } else if !listed.contains(path) {
            tree.orphans.push(path.clone());
        }
    }
    Ok(excluded
        .into_iter()
        .map(|source| Unmapped {
            source,
            reason: "marked draft or unlisted in Docusaurus front matter; not converted".into(),
        })
        .collect())
}

/// Strips the module wrapper so the object literal can be parsed as JSON5.
fn unwrap_module(source: &str) -> Option<&str> {
    let body = source
        .split_once("module.exports")
        .or_else(|| source.split_once("export default"))
        .map(|(_, rest)| rest)?
        .trim_start()
        .strip_prefix('=')
        .unwrap_or_else(|| {
            source
                .split_once("export default")
                .map(|(_, rest)| rest)
                .unwrap_or("")
        })
        .trim();

    Some(body.trim_end().trim_end_matches(';').trim())
}

/// Constructs this cannot read without running the file.
///
/// Named individually so the report can say which one stopped it, rather than
/// "could not parse" — the author knows what `require` is doing in there and
/// only needs to be told that it is the problem.
fn dynamic_construct(source: &str) -> Option<&'static str> {
    // A comment mentioning `require` is not a call to it, but distinguishing
    // them needs a tokenizer, and being wrong in the cautious direction costs
    // an unnecessary message rather than a wrong sidebar.
    for (needle, name) in [
        ("require(", "a require() call"),
        ("=>", "an arrow function"),
        ("function ", "a function"),
        ("...", "a spread"),
        ("`", "a template literal"),
    ] {
        if source.contains(needle) {
            return Some(name);
        }
    }
    None
}

fn parse_entry(value: &Value) -> Option<Entry> {
    match value {
        // A bare document id.
        Value::String(id) => Some(Entry::Page {
            label: None,
            source: Utf8PathBuf::from(format!("{id}.md")),
        }),

        Value::Object(map) => match map.get("type").and_then(Value::as_str) {
            Some("category") => Some(Entry::Group {
                label: map.get("label").and_then(Value::as_str)?.to_owned(),
                items: map
                    .get("items")
                    .and_then(Value::as_array)
                    .map(|items| items.iter().filter_map(parse_entry).collect())
                    .unwrap_or_default(),
            }),

            Some("doc") => Some(Entry::Page {
                label: map.get("label").and_then(Value::as_str).map(str::to_owned),
                source: Utf8PathBuf::from(format!("{}.md", map.get("id").and_then(Value::as_str)?)),
            }),

            Some("link") => Some(Entry::Foreign {
                label: map.get("label").and_then(Value::as_str)?.to_owned(),
                target: map.get("href").and_then(Value::as_str)?.to_owned(),
            }),

            // `autogenerated` tells Docusaurus to build this branch from the
            // folder. There is nothing here to read, and the walk below picks
            // the pages up as orphans — which is the honest outcome, since the
            // order came from frontmatter this converter has not read.
            _ => None,
        },

        _ => None,
    }
}

fn sidebar_path(source: &Utf8Path) -> Option<Utf8PathBuf> {
    [
        "sidebars.js",
        "sidebars.ts",
        "sidebars.json",
        "sidebars.cjs",
    ]
    .iter()
    .map(|name| source.join(name))
    .find(|path| path.exists())
}

fn config_path(source: &Utf8Path) -> Option<Utf8PathBuf> {
    ["docusaurus.config.js", "docusaurus.config.ts"]
        .iter()
        .map(|name| source.join(name))
        .find(|path| path.exists())
}

/// Pulls a quoted string field out of a config file without parsing it.
///
/// `docusaurus.config.js` is full of requires and functions, so it is never
/// going to parse as data. The title and tagline are all this needs from it,
/// and they are always string literals.
fn config_string(source: &str, key: &str) -> Option<String> {
    let start = source.find(&format!("{key}:"))? + key.len() + 1;
    let rest = source[start..].trim_start();
    let quote = rest.chars().next().filter(|c| *c == '\'' || *c == '"')?;
    let body = &rest[1..];
    let end = body.find(quote)?;
    Some(body[..end].to_owned())
}

pub struct Docusaurus;

impl Converter for Docusaurus {
    fn id(&self) -> &'static str {
        "docusaurus"
    }

    fn label(&self) -> &'static str {
        "Docusaurus — sidebars.js and a docs folder"
    }

    fn detect(&self, source: &Utf8Path) -> Confidence {
        if config_path(source).is_some() {
            Confidence::CERTAIN
        } else if sidebar_path(source).is_some() {
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
        let root = zellij_core::files::contained_path(source, Utf8Path::new("docs"))?;
        if !root.is_dir() {
            return Err(zellij_core::Error::UnknownFormat {
                path: source.to_owned(),
            });
        }

        let (site_name, description) = match config_path(source) {
            Some(path) => {
                let path = zellij_core::files::contained_path(
                    source,
                    Utf8Path::new(path.file_name().unwrap()),
                )?;
                let raw = std::fs::read_to_string(&path)?;
                (config_string(&raw, "title"), config_string(&raw, "tagline"))
            }
            None => (None, None),
        };

        let mut notes = Vec::new();
        let mut nav: Vec<Entry> = Vec::new();
        let mut derive_from_folder = true;

        if let Some(path) = sidebar_path(source) {
            let path = zellij_core::files::contained_path(
                source,
                Utf8Path::new(path.file_name().unwrap()),
            )?;
            let raw = std::fs::read_to_string(&path)?;

            if let Some(construct) = dynamic_construct(&raw) {
                // Declining is the point. A sidebar half-read from a file that
                // computes its exports is worse than no sidebar, because it
                // looks finished.
                notes.push(format!(
                    "{path} contains {construct}, so it was not read: this converter parses \
                     declarative sidebars and will not guess at ones that compute their exports. \
                     The pages below came from the folder instead, and their order is alphabetical \
                     rather than yours. Reordering `_sidebar.yaml` by hand is usually a few minutes."
                ));
            } else if let Some(body) = if path.extension() == Some("json") {
                Some(raw.as_str())
            } else {
                unwrap_module(&raw)
            } {
                match json5::from_str::<Value>(body) {
                    Ok(Value::Object(sidebars)) => {
                        derive_from_folder = false;
                        // Every named sidebar, in order. Most projects have one.
                        for entries in sidebars.values().filter_map(Value::as_array) {
                            nav.extend(entries.iter().filter_map(parse_entry));
                        }
                    }
                    Ok(Value::Array(entries)) => {
                        derive_from_folder = false;
                        nav.extend(entries.iter().filter_map(parse_entry));
                    }
                    _ => notes.push(format!(
                        "{path} could not be read as data; the pages below came from the folder."
                    )),
                }
            }
        } else {
            notes.push(
                "No sidebars file, so the navigation was derived from the folder.".to_owned(),
            );
        }

        let mut tree = DocTree {
            site_name,
            description,
            root: root.clone(),
            nav,
            orphans: Vec::new(),
        };

        let inventory = read_inventory(&root)?;
        let excluded = resolve_navigation(&mut tree, &inventory, derive_from_folder)?;
        let documents: BTreeMap<_, _> = tree
            .pages()
            .into_iter()
            .map(|path| (path.clone(), inventory[path].raw.clone()))
            .collect();
        let mut conversion = emit(&tree, |relative| documents.get(relative).cloned())?;
        crate::references::rewrite(&tree, &documents, Some(source), options, &mut conversion)?;

        conversion.notes.extend(notes);
        conversion.unmapped.extend(excluded);
        conversion.notes.push("Docusaurus document IDs and sidebar labels use front matter and the default number-prefix rules. Source filenames determine Zellij routes; custom slugs, plugin paths, custom prefix parsers and autogenerated sidebar ordering still require review.".into());
        Ok(conversion)
    }
}

inventory::submit! {
    &Docusaurus as &'static dyn Converter
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn document_ids_follow_default_prefix_and_custom_basename_rules() {
        for (path, fields, expected) in [
            ("01-Guides/02-Start.md", "", "Guides/Start"),
            ("01-Guides/02-Start.mdx", "id: custom", "Guides/custom"),
            (
                "01-Guides/02-Start.mdx",
                "id: 03-custom",
                "Guides/03-custom",
            ),
            (
                "01-Guides/02-Start.md",
                "parse_number_prefixes: false",
                "01-Guides/02-Start",
            ),
            (
                "01-Guides/02-Start.md",
                "id: custom\nparse_number_prefixes: false",
                "01-Guides/custom",
            ),
            ("2026-11-04/v1.2.md", "", "2026-11-04/v1.2"),
            ("1.2/03__ Chapter.md", "", "1.2/Chapter"),
        ] {
            let raw = format!("---\n{fields}\n---\n");
            let path = Utf8Path::new(path);
            let metadata = crate::frontmatter::parse(&raw, path).unwrap();
            assert_eq!(document_id(path, &metadata).unwrap(), expected);
        }
    }

    #[test]
    fn unwraps_common_js() {
        assert_eq!(unwrap_module("module.exports = {a: 1};"), Some("{a: 1}"));
    }

    #[test]
    fn unwraps_es_modules() {
        assert_eq!(unwrap_module("export default {a: 1}\n"), Some("{a: 1}"));
    }

    #[test]
    fn reads_a_category_and_its_nesting() {
        let body = unwrap_module(
            "module.exports = { docs: ['intro', { type: 'category', label: 'Guides', items: ['g/a'] }] };",
        )
        .unwrap();
        let parsed: Value = json5::from_str(body).unwrap();
        let entries = parsed["docs"].as_array().unwrap();

        assert_eq!(
            parse_entry(&entries[0]).unwrap(),
            Entry::Page {
                label: None,
                source: Utf8PathBuf::from("intro.md")
            }
        );
        match parse_entry(&entries[1]).unwrap() {
            Entry::Group { label, items } => {
                assert_eq!(label, "Guides");
                assert_eq!(items.len(), 1);
            }
            other => panic!("expected a group, got {other:?}"),
        }
    }

    #[test]
    fn reads_a_link_as_foreign() {
        let value: Value =
            json5::from_str("{type:'link', label:'Blog', href:'https://example.com'}").unwrap();
        assert!(matches!(parse_entry(&value), Some(Entry::Foreign { .. })));
    }

    #[test]
    fn refuses_a_sidebar_that_computes_itself() {
        assert_eq!(
            dynamic_construct("module.exports = require('./generated');"),
            Some("a require() call")
        );
        assert_eq!(
            dynamic_construct("module.exports = items.map(x => x.id);"),
            Some("an arrow function")
        );
        assert_eq!(dynamic_construct("module.exports = {docs: ['a']};"), None);
    }

    #[test]
    fn lifts_the_title_out_of_a_config_it_cannot_parse() {
        let config = "const x = require('y');\nmodule.exports = {\n  title: 'Acme Docs',\n  tagline: 'Ship it',\n};";
        assert_eq!(config_string(config, "title").as_deref(), Some("Acme Docs"));
        assert_eq!(config_string(config, "tagline").as_deref(), Some("Ship it"));
    }
}
