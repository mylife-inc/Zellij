//! Import a declarative Hugo documentation tree without executing Go templates.
use crate::{
    Confidence, Conversion, ConvertOptions, Converter, Unmapped,
    emit::emit,
    model::DocTree,
    references::{AssetSource, route},
    source_urls::SourceUrls,
};
use camino::{Utf8Path, Utf8PathBuf};
use serde_yaml_ng::Value;
use std::collections::{BTreeMap, BTreeSet};
use walkdir::WalkDir;
use zellij_core::{Result, files};
mod config;
mod data;
mod navigation;
mod urls;
use data::{active, boolean, invalid, string};

pub(crate) struct Assets {
    pub static_root: Utf8PathBuf,
    pub content: BTreeSet<Utf8PathBuf>,
    pub static_files: BTreeSet<Utf8PathBuf>,
    pub website: BTreeMap<Utf8PathBuf, Utf8PathBuf>,
}
struct Page {
    raw: String,
    address: String,
    title: String,
    weight: i64,
    date: i64,
    published: bool,
    branch: bool,
    leaf: bool,
}
fn note(notes: &mut Vec<Unmapped>, source: &Utf8Path, reason: impl Into<String>) {
    notes.push(Unmapped {
        source: source.to_owned(),
        reason: reason.into(),
    });
}
fn inventory(root: &Utf8Path) -> Result<Vec<Utf8PathBuf>> {
    if !root.exists() {
        return Ok(Vec::new());
    }
    if !root.is_dir() {
        return Err(invalid(format!("{root} must be a directory")));
    }
    let mut paths = Vec::new();
    for e in WalkDir::new(root)
        .sort_by_file_name()
        .into_iter()
        .filter_entry(|e| {
            e.depth() == 0
                || (!e.file_name().to_string_lossy().starts_with('.')
                    && !["node_modules", "vendor"]
                        .contains(&e.file_name().to_string_lossy().as_ref()))
        })
    {
        let e = e.map_err(|e| invalid(e.to_string()))?;
        if e.depth() == 0 {
            continue;
        }
        let path =
            Utf8Path::from_path(e.path()).ok_or_else(|| invalid("source paths must be UTF-8"))?;
        let relative = Utf8PathBuf::from(
            path.strip_prefix(root)
                .unwrap()
                .components()
                .map(|c| c.as_str())
                .collect::<Vec<_>>()
                .join("/"),
        );
        files::contained_path(root, &relative)?;
        if e.file_type().is_file() {
            paths.push(relative);
        }
    }
    Ok(paths)
}
fn is_content(path: &Utf8Path) -> bool {
    matches!(
        path.extension(),
        Some("md" | "markdown" | "mdx" | "html" | "adoc" | "asciidoc" | "rst" | "org" | "pandoc")
    )
}
fn hidden(path: &Utf8Path) -> bool {
    path.components().any(|c| {
        c.as_str().starts_with('_') && !matches!(c.as_str(), "_index.md" | "_index.markdown")
    })
}
pub struct Hugo;
impl Converter for Hugo {
    fn id(&self) -> &'static str {
        "hugo"
    }
    fn label(&self) -> &'static str {
        "Hugo — Markdown sections and page bundles"
    }
    fn detect(&self, source: &Utf8Path) -> Confidence {
        if config::markers(source).is_empty() {
            Confidence::NONE
        } else {
            Confidence::CERTAIN
        }
    }
    fn convert(&self, source: &Utf8Path, options: &ConvertOptions) -> Result<Conversion> {
        let config = config::read(source)?;
        let root = data::directory(source, string(&config, "contentdir")?.unwrap_or("content"))?;
        let static_root =
            data::directory(source, string(&config, "staticdir")?.unwrap_or("static"))?;
        let publish = data::directory(source, string(&config, "publishdir")?.unwrap_or("public"))?;
        if !root.is_dir() {
            return Err(invalid(
                "contentDir must name an existing contained directory",
            ));
        }
        for (a, b) in [
            (&root, &static_root),
            (&root, &publish),
            (&static_root, &publish),
        ] {
            if a.starts_with(b) || b.starts_with(a) {
                return Err(invalid(
                    "content, static and publish directories must not overlap",
                ));
            }
        }
        let paths = inventory(&root)?;
        let mut unmapped = Vec::new();
        let mut pages = BTreeMap::new();
        let mut leaves = BTreeMap::new();
        let mut indexes = BTreeSet::new();
        for path in &paths {
            if path.file_name().is_some_and(|s| s.starts_with("_content.")) {
                return Err(invalid("content adapters require manual migration"));
            }
            if is_content(path) && matches!(path.file_stem(), Some("index" | "_index")) {
                if !matches!(path.extension(), Some("md" | "markdown")) {
                    return Err(invalid(format!("{path}: unsupported bundle index format")));
                }
                let parent = path.parent().unwrap().to_owned();
                if !indexes.insert(parent.clone()) {
                    return Err(invalid("conflicting index/_index files in a bundle"));
                }
                if path.file_stem() == Some("index") {
                    if parent.as_str().is_empty() {
                        return Err(invalid(
                            "use _index.md for the Hugo home page, not a root leaf bundle",
                        ));
                    }
                    leaves.insert(parent, path.clone());
                }
            }
        }
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| invalid(e.to_string()))?
            .as_secs() as i64;
        for path in &paths {
            if hidden(path) {
                note(
                    &mut unmapped,
                    path,
                    "internal or underscore-prefixed content requires manual migration",
                );
                continue;
            }
            if !is_content(path) {
                continue;
            }
            if !matches!(path.extension(), Some("md" | "markdown")) {
                note(&mut unmapped, path, "only Markdown content is imported");
                continue;
            }
            if leaves
                .iter()
                .any(|(dir, index)| path.starts_with(dir) && path != index)
            {
                note(
                    &mut unmapped,
                    path,
                    "Markdown inside a leaf bundle is a page resource, not a standalone page; not imported",
                );
                continue;
            }
            let lang = string(&config, "defaultcontentlanguage")?.unwrap_or("en");
            if path
                .file_stem()
                .is_some_and(|s| s.ends_with(&format!(".{lang}")))
            {
                return Err(invalid(
                    "language-suffixed content requires manual consolidation",
                ));
            }
            let raw = std::fs::read_to_string(files::contained_path(&root, path)?)?;
            let (meta, body, original) = data::frontmatter(&raw)?;
            for key in [
                "cascade",
                "menus",
                "menu",
                "outputs",
                "path",
                "kind",
                "lang",
                "translationkey",
                "resources",
            ] {
                if active(&meta, key) {
                    return Err(invalid(format!("{path}: {key} requires manual migration")));
                }
            }
            if string(&meta, "markup")?
                .is_some_and(|v| !matches!(v, "markdown" | "md" | "goldmark"))
            {
                return Err(invalid("only Markdown markup is supported"));
            }
            let date = data::date(
                &meta,
                &[
                    "date",
                    "publishdate",
                    "pubdate",
                    "published",
                    "lastmod",
                    "modified",
                ],
            )?;
            let published_date =
                data::date(&meta, &["publishdate", "pubdate", "published", "date"])?;
            let expiry = data::date(&meta, &["expirydate", "unpublishdate"])?;
            let mut published = (!boolean(&meta, "draft")? || boolean(&config, "builddrafts")?)
                && (published_date.is_none_or(|d| d <= now) || boolean(&config, "buildfuture")?)
                && (expiry.is_none_or(|d| d > now) || boolean(&config, "buildexpired")?)
                && !boolean(&meta, "headless")?;
            if meta.get("build").is_some() && meta.get("_build").is_some() {
                return Err(invalid("use only build or _build, not both"));
            }
            if let Some(build) = meta.get("build").or_else(|| meta.get("_build")) {
                let m = build
                    .as_mapping()
                    .ok_or_else(|| invalid("page build options must be a mapping"))?;
                if m.keys()
                    .any(|k| !matches!(k.as_str(), Some("list" | "render" | "publishresources")))
                {
                    return Err(invalid("unsupported page build option"));
                }
                let list = string(build, "list")?.unwrap_or("always");
                let render = string(build, "render")?.unwrap_or("always");
                if !matches!(list, "always" | "local" | "never")
                    || !matches!(render, "always" | "link" | "never")
                {
                    return Err(invalid("invalid page build list/render option"));
                }
                published &= list != "never" && render == "always";
                if build.get("publishresources").is_some() && !boolean(build, "publishresources")? {
                    return Err(invalid(
                        "publishResources: false needs template evaluation; migrate resources manually",
                    ));
                }
            }
            let weight = match meta.get("weight") {
                None | Some(Value::Null) => 0,
                Some(v) => v
                    .as_i64()
                    .ok_or_else(|| invalid("weight must be an integer"))?,
            };
            let title = string(&meta, "linktitle")?
                .filter(|s| !s.is_empty())
                .or(string(&meta, "title")?.filter(|s| !s.is_empty()))
                .map(str::to_owned)
                .or_else(|| crate::headings::first_title(body))
                .unwrap_or_else(|| path.file_stem().unwrap().into());
            if !published {
                note(
                    &mut unmapped,
                    path,
                    "draft, scheduled, expired, headless or unlisted page; not imported",
                );
            }
            if active(&meta, "aliases") {
                note(
                    &mut unmapped,
                    path,
                    "redirect aliases are preserved as metadata only; recreate redirects manually",
                );
            }
            let mut fence = crate::fences::Fence::default();
            if body
                .lines()
                .any(|l| !fence.contains(l) && (l.contains("{{") || l.contains("{%")))
            {
                note(
                    &mut unmapped,
                    path,
                    "shortcodes and template expressions remain literal text and require manual migration",
                );
            }
            pages.insert(
                path.clone(),
                Page {
                    raw: data::document(&meta, body, original)?,
                    address: urls::canonical(path, &meta, &config)?,
                    title,
                    weight,
                    date: date.unwrap_or(i64::MIN),
                    published,
                    branch: path.file_stem() == Some("_index"),
                    leaf: path.file_stem() == Some("index"),
                },
            );
        }
        let mut assets = Assets {
            static_root,
            content: BTreeSet::new(),
            static_files: BTreeSet::new(),
            website: BTreeMap::new(),
        };
        for path in inventory(&assets.static_root)? {
            if !hidden(&path) {
                assets.static_files.insert(path);
            }
        }
        for path in &paths {
            if hidden(path) || is_content(path) {
                continue;
            }
            let owner = pages
                .iter()
                .filter(|(p, page)| {
                    (page.branch || page.leaf) && path.starts_with(p.parent().unwrap())
                })
                .max_by_key(|(p, _)| p.components().count())
                // An implicit top-level section owns its resources even when
                // the home page is draft; home metadata does not cascade.
                .filter(|(p, _)| {
                    !p.parent().unwrap().as_str().is_empty()
                        || path.parent().unwrap().as_str().is_empty()
                });
            if owner.is_some_and(|(_, p)| !p.published) {
                continue;
            }
            // Assets belong to their closest bundle. Without an explicit index,
            // the implicit home/section bundle uses the source directory path.
            let address = if let Some((p, page)) =
                owner.filter(|(p, _)| !p.parent().unwrap().as_str().is_empty())
            {
                let tail = path.strip_prefix(p.parent().unwrap()).unwrap();
                format!("{}{}", page.address, tail)
            } else {
                // Top-level directories are implicit branch bundles. Their
                // page URL is sanitized; resource filenames retain their case.
                let mut parts = path.components();
                let first = parts.next().unwrap().as_str();
                let tail = parts.map(|c| c.as_str()).collect::<Vec<_>>().join("/");
                if tail.is_empty() {
                    format!("/{first}")
                } else {
                    format!(
                        "/{}/{tail}",
                        urls::segment(first, boolean(&config, "disablepathtolower")?)?
                    )
                }
            };
            let key = Utf8PathBuf::from(address.trim_start_matches('/'));
            if assets.website.insert(key.clone(), path.clone()).is_some() {
                return Err(invalid(format!("asset URL collision at {key}")));
            }
            assets.content.insert(path.clone());
        }
        let nav = navigation::derive(&pages)?;
        let output_sources: BTreeMap<_, _> = pages
            .iter()
            .filter(|(_, p)| p.branch)
            .map(|(p, _)| {
                // Native PathBuf filename changes insert backslashes on
                // Windows; emitted names and reference keys use portable '/'.
                let output = p.with_file_name("index.md");
                let output = Utf8PathBuf::from(
                    output
                        .components()
                        .map(|c| c.as_str())
                        .collect::<Vec<_>>()
                        .join("/"),
                );
                (p.clone(), output)
            })
            .collect();
        let documents: BTreeMap<_, _> = pages
            .iter()
            .filter(|(_, p)| p.published)
            .map(|(p, v)| (p.clone(), v.raw.clone()))
            .collect();
        let tree = DocTree {
            root,
            site_name: string(&config, "title")?.map(str::to_owned),
            description: string(&config, "description")?
                .or(config
                    .get("params")
                    .map(|v| string(v, "description"))
                    .transpose()?
                    .flatten())
                .map(str::to_owned),
            nav,
            output_sources,
            orphans: pages
                .iter()
                .filter(|(_, p)| !p.published)
                .map(|(p, _)| p.clone())
                .collect(),
            ..Default::default()
        };
        let mut website = SourceUrls::from_site_url(string(&config, "baseurl")?)?;
        for (path, page) in &pages {
            let target = page.published.then(|| route(tree.output_source(path)));
            website.add(&page.address, target.clone())?;
            if page.address.ends_with('/') {
                website.add(&format!("{}index.html", page.address), target)?;
            }
            website.add_page(path, &page.address)?;
        }
        let mut conversion = emit(&tree, |p| documents.get(p).cloned())?;
        crate::references::rewrite(
            &tree,
            &documents,
            AssetSource::Hugo(&assets),
            Some(&website),
            options,
            &mut conversion,
        )?;
        conversion.unmapped.extend(unmapped);
        conversion.notes.push("Hugo was read as data. No Go templates, shortcodes, modules or themes were executed. Navigation is derived from Markdown sections and bundles: weight, date descending, then case-folded link title and source path; theme-specific ordering and locale collation require review. Eligible referenced assets follow the selected import asset policy. Source URL overrides map links to filename-derived Zellij routes; aliases do not generate redirects.".into());
        Ok(conversion)
    }
}
inventory::submit! { &Hugo as &dyn Converter }
