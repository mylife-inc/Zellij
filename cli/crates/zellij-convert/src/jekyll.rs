//! Read Jekyll documentation as data, without Ruby, Liquid or theme execution.
use crate::{
    Confidence, Conversion, ConvertOptions, Converter, Unmapped,
    emit::emit,
    model::{DocTree, Entry},
    references::{AssetSource, route},
    source_urls::SourceUrls,
};
use camino::{Utf8Path, Utf8PathBuf};
use serde_yaml_ng::{Mapping, Value};
use std::{
    collections::{BTreeMap, BTreeSet},
    io::Read,
};
use walkdir::WalkDir;
use zellij_core::{Result, files};
mod config;
mod navigation;
mod urls;
use config::{boolean, invalid, string};

struct Page {
    raw: String,
    address: String,
    collection: Option<String>,
    published: bool,
}
struct Collection {
    output: bool,
    order: Vec<Utf8PathBuf>,
    permalink: Option<String>,
}
fn configs(source: &Utf8Path) -> Vec<Utf8PathBuf> {
    ["_config.yml", "_config.yaml", "_config.toml"]
        .into_iter()
        .map(Utf8PathBuf::from)
        .filter(|p| std::fs::symlink_metadata(source.join(p)).is_ok())
        .collect()
}
fn collections(config: &Value) -> Result<BTreeMap<String, Collection>> {
    let mut out = BTreeMap::new();
    let value = config
        .get("collections")
        .cloned()
        .unwrap_or(Value::Mapping(Mapping::new()));
    let mapping = match value {
        Value::Mapping(mapping) => mapping,
        Value::Sequence(values) => values.into_iter().map(|v| (v, Value::Null)).collect(),
        _ => return Err(invalid("collections must be a mapping or a list of names")),
    };
    for (name, settings) in mapping {
        let name = name
            .as_str()
            .ok_or_else(|| invalid("collection names must be strings"))?;
        if name.is_empty()
            || !name
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-'))
        {
            return Err(invalid(
                "collection names must use ASCII letters, numbers, - or _",
            ));
        }
        if ["data", "layouts", "includes", "plugins", "site", "sass"].contains(&name) {
            return Err(invalid(
                "collection name conflicts with an internal Jekyll directory",
            ));
        }
        if !settings.is_null() && !settings.is_mapping() {
            return Err(invalid("collection settings must be a mapping"));
        }
        if settings.get("sort_by").is_some() {
            return Err(invalid(
                "collection sort_by requires manual migration; use a literal order list",
            ));
        }
        out.insert(
            name.to_owned(),
            Collection {
                output: boolean(&settings, "output")?.unwrap_or(false),
                order: config::paths(&settings, "order")?,
                permalink: string(&settings, "permalink")?.map(str::to_owned),
            },
        );
    }
    Ok(out)
}
fn report(notes: &mut Vec<Unmapped>, source: &Utf8Path, reason: impl Into<String>) {
    notes.push(Unmapped {
        source: source.to_owned(),
        reason: reason.into(),
    });
}
fn website(config: &Value) -> Result<SourceUrls> {
    SourceUrls::new(
        string(config, "url")?.filter(|v| !v.is_empty()),
        string(config, "baseurl")?
            .filter(|v| !v.is_empty())
            .unwrap_or("/"),
    )
}
fn supported(config: &Value) -> Result<()> {
    if !config.is_mapping() {
        return Err(invalid("configuration must be a YAML mapping"));
    }
    for key in ["include", "languages", "locales", "permalinks"] {
        if config
            .get(key)
            .is_some_and(|v| !v.is_null() && v.as_sequence().is_none_or(|v| !v.is_empty()))
        {
            return Err(invalid(format!(
                "{key} requires manual migration before importing"
            )));
        }
    }
    for (key, expected) in [
        ("collections_dir", "."),
        ("layouts_dir", "_layouts"),
        ("includes_dir", "_includes"),
        ("plugins_dir", "_plugins"),
        ("encoding", "utf-8"),
        ("markdown_ext", "markdown,mkdown,mkdn,mkd,md"),
    ] {
        if string(config, key)?
            .is_some_and(|v| v != expected && !(key == "collections_dir" && v.is_empty()))
        {
            return Err(invalid(format!("custom {key} is not supported")));
        }
    }
    let style = string(config, "permalink")?.unwrap_or("date");
    if !["date", "none", "ordinal", "pretty"].contains(&style) {
        return Err(invalid(
            "global permalink must be date, none, ordinal or pretty; custom patterns require manual migration",
        ));
    }
    Ok(())
}

pub struct Jekyll;
impl Converter for Jekyll {
    fn id(&self) -> &'static str {
        "jekyll"
    }
    fn label(&self) -> &'static str {
        "Jekyll — YAML configuration, Markdown pages and collections"
    }
    fn detect(&self, source: &Utf8Path) -> Confidence {
        if configs(source).is_empty() {
            Confidence::NONE
        } else {
            Confidence::CERTAIN
        }
    }
    fn convert(&self, source: &Utf8Path, options: &ConvertOptions) -> Result<Conversion> {
        let configs = configs(source);
        if configs.len() != 1 {
            return Err(invalid(
                "expected exactly one _config.yml or _config.yaml; multiple configuration files require manual consolidation",
            ));
        }
        if configs[0].extension() == Some("toml") {
            return Err(invalid(
                "TOML configuration is not supported yet; use a YAML source copy",
            ));
        }
        let mut settings = config::yaml(&std::fs::read_to_string(files::contained_path(
            source,
            &configs[0],
        )?)?)?;
        if settings.is_null() {
            settings = Value::Mapping(Mapping::new());
        }
        supported(&settings)?;
        let defaults = config::Defaults::read(&settings)?;
        let root = config::directory(source, string(&settings, "source")?.unwrap_or("."))?;
        if !root.is_dir() {
            return Err(invalid("source must name an existing contained directory"));
        }
        let destination =
            config::directory(source, string(&settings, "destination")?.unwrap_or("_site"))?;
        if root.starts_with(&destination) {
            return Err(invalid("destination must not contain the source"));
        }
        let collections = collections(&settings)?;
        let data = config::directory(&root, string(&settings, "data_dir")?.unwrap_or("_data"))?;
        if root.starts_with(&data) {
            return Err(invalid(
                "data_dir must not contain the documentation source",
            ));
        }
        let excludes = config::paths(&settings, "exclude")?;
        let mut notes=vec!["Jekyll was read as data; no Ruby, Liquid, plugins or theme code was executed. Layouts, theme-provided configuration, generated pages, posts/drafts, Liquid expressions and custom Markdown presentation require manual migration. Only referenced eligible static assets are copied.".into()];
        let mut unmapped = Vec::new();
        let mut all = BTreeMap::new();
        let mut assets = BTreeSet::new();
        let style = string(&settings, "permalink")?.unwrap_or("date");
        for (name, collection) in &collections {
            let directory = root.join(format!("_{name}"));
            if collection.output
                && (directory.starts_with(&data) || directory.starts_with(&destination))
            {
                return Err(invalid(
                    "collection directory conflicts with data_dir or destination",
                ));
            }
            if !collection.output || ["posts", "drafts"].contains(&name.as_str()) {
                report(
                    &mut unmapped,
                    Utf8Path::new(&format!("_{name}")),
                    "collection is not enabled for documentation output; not imported",
                );
            }
        }
        let entries = WalkDir::new(&root)
            .sort_by_file_name()
            .into_iter()
            .filter_entry(|entry| {
                if entry.depth() == 0 {
                    return true;
                }
                let name = entry.file_name().to_string_lossy();
                let relative = entry.path().strip_prefix(&root).unwrap();
                if entry.path().starts_with(&destination)
                    || entry.path().starts_with(&data)
                    || excludes.iter().any(|e| relative.starts_with(e))
                {
                    return false;
                }
                if name.starts_with('.')
                    || name.ends_with('~')
                    || ["node_modules", "vendor", "Gemfile", "Gemfile.lock"]
                        .contains(&name.as_ref())
                {
                    return false;
                }
                if let Some(name) = name.strip_prefix('_') {
                    return entry.depth() == 1
                        && !["posts", "drafts"].contains(&name)
                        && collections.get(name).is_some_and(|c| c.output);
                }
                true
            });
        for entry in entries {
            let entry = entry.map_err(|e| invalid(format!("cannot inspect source: {e}")))?;
            let path = Utf8Path::from_path(entry.path())
                .ok_or_else(|| invalid("source paths must be UTF-8"))?;
            if path == root {
                continue;
            }
            let relative = Utf8PathBuf::from(
                path.strip_prefix(&root)
                    .unwrap()
                    .components()
                    .map(|p| p.as_str())
                    .collect::<Vec<_>>()
                    .join("/"),
            );
            let full = files::contained_path(&root, &relative)?;
            if !entry.file_type().is_file() {
                continue;
            }
            let markdown = matches!(relative.extension(), Some("md" | "markdown"));
            if !markdown {
                let mut prefix = Vec::new();
                std::fs::File::open(&full)?
                    .take(8)
                    .read_to_end(&mut prefix)?;
                let prefix = prefix.as_slice();
                let prefix = prefix.strip_prefix(&[0xef, 0xbb, 0xbf]).unwrap_or(prefix);
                if prefix.starts_with(b"---\n")
                    || prefix.starts_with(b"---\r\n")
                    || matches!(
                        relative.extension(),
                        Some("mdx" | "mkd" | "mkdn" | "mkdown")
                    )
                {
                    report(
                        &mut unmapped,
                        &relative,
                        "template or unsupported document format; only .md and .markdown pages are imported",
                    );
                } else {
                    assets.insert(relative);
                }
                continue;
            }
            let raw = std::fs::read_to_string(&full)?;
            if crate::frontmatter::parse(&raw, &relative)?.raw.is_none() {
                report(
                    &mut unmapped,
                    &relative,
                    "Markdown without YAML front matter is a Jekyll static file; not imported as a page",
                );
                continue;
            }
            let first = relative.components().next().unwrap().as_str();
            let collection = first
                .strip_prefix('_')
                .filter(|name| collections.contains_key(*name));
            let raw = defaults.apply(&relative, collection.unwrap_or("pages"), &raw)?;
            let metadata = crate::frontmatter::parse(&raw, &relative)?;
            let published = (!metadata.contains("published")
                || metadata.boolean("published")? == Some(true))
                && metadata.boolean("nav_exclude")? != Some(true);
            static DATED: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {
                regex::Regex::new(r"^\d{2,4}-\d{1,2}-\d{1,2}-").unwrap()
            });
            if collection.is_some()
                && (metadata.contains("date") || DATED.is_match(relative.file_name().unwrap()))
            {
                return Err(invalid(format!(
                    "{relative}: dated collections need manual migration; automatic future/date ordering is not implemented"
                )));
            }
            let explicit = metadata.string("permalink")?;
            let collection_url = collection
                .and_then(|name| collections[name].permalink.as_deref())
                .map(|pattern| {
                    let name = collection.unwrap();
                    let stem = relative.with_extension("");
                    let tail = stem.as_str().strip_prefix(&format!("_{name}/")).unwrap();
                    static TOKENS: std::sync::LazyLock<regex::Regex> =
                        std::sync::LazyLock::new(|| regex::Regex::new(r":[A-Za-z_]+").unwrap());
                    TOKENS
                        .replace_all(pattern, |capture: &regex::Captures<'_>| match &capture[0] {
                            ":collection" => name.to_owned(),
                            ":path" => tail.to_owned(),
                            ":output_ext" => ".html".to_owned(),
                            other => other.to_owned(),
                        })
                        .into_owned()
                });
            let address = urls::canonical(
                &relative,
                collection,
                explicit.or(collection_url.as_deref()),
                style,
            )?;
            if !published {
                report(
                    &mut unmapped,
                    &relative,
                    "unpublished or excluded from navigation; not imported",
                );
            }
            let mut fence = crate::fences::Fence::default();
            if metadata
                .body
                .lines()
                .any(|l| !fence.contains(l) && (l.contains("{{") || l.contains("{%")))
            {
                report(
                    &mut unmapped,
                    &relative,
                    "Liquid expressions are retained as literal source text and require manual migration",
                );
            }
            if metadata.contains("parent")
                || metadata.contains("nav_order")
                || metadata.contains("has_children")
            {
                report(
                    &mut unmapped,
                    &relative,
                    "theme-specific front matter navigation requires review; use supported _data/navigation entries for explicit order and nesting",
                );
            }
            let collection = collection.map(str::to_owned);
            all.insert(
                relative,
                Page {
                    raw,
                    address,
                    collection,
                    published,
                },
            );
        }
        let mut lookup = website(&settings)?;
        for (source, page) in &all {
            urls::register(
                &mut lookup,
                &page.address,
                page.published.then(|| source.to_string()),
                true,
            )?;
        }
        let nav = navigation::read(
            &root,
            &settings,
            &lookup,
            &all,
            &collections,
            &mut notes,
            &mut unmapped,
        )?;
        let selected: BTreeSet<_> = nav_pages(&nav);
        let documents: BTreeMap<_, _> = all
            .iter()
            .filter(|(p, _)| selected.contains(*p))
            .map(|(p, page)| (p.clone(), page.raw.clone()))
            .collect();
        let tree = DocTree {
            root,
            site_name: string(&settings, "title")?.map(str::to_owned),
            description: string(&settings, "description")?.map(str::to_owned),
            nav,
            orphans: all
                .keys()
                .filter(|p| !selected.contains(*p))
                .cloned()
                .collect(),
            ..Default::default()
        };
        let mut website = website(&settings)?;
        for (source, page) in &all {
            let target = selected.contains(source).then(|| route(source));
            urls::register(&mut website, &page.address, target, false)?;
            website.add_page(source, &page.address)?;
        }
        let mut conversion = emit(&tree, |p| documents.get(p).cloned())?;
        crate::references::rewrite(
            &tree,
            &documents,
            AssetSource::Jekyll(&assets),
            Some(&website),
            options,
            &mut conversion,
        )?;
        conversion.notes.extend(notes);
        conversion.unmapped.extend(unmapped);
        Ok(conversion)
    }
}
fn nav_pages(nav: &[Entry]) -> BTreeSet<Utf8PathBuf> {
    let mut out = BTreeSet::new();
    for entry in nav {
        match entry {
            Entry::Page { source, .. } => {
                out.insert(source.clone());
            }
            Entry::Group { items, .. } => out.extend(nav_pages(items)),
            _ => {}
        }
    }
    out
}
inventory::submit! { &Jekyll as &dyn Converter }
