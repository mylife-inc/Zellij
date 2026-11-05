//! Declarative VitePress imports, without evaluating source JavaScript or Vue.
use std::collections::{BTreeMap, BTreeSet};

use camino::{Utf8Path, Utf8PathBuf};
use serde_json::Value;
use walkdir::WalkDir;
use zellij_core::{Result, files};

use crate::{
    Confidence, Conversion, ConvertOptions, Converter, Unmapped,
    emit::emit,
    model::DocTree,
    references::{AssetSource, route},
    source_urls::SourceUrls,
};

mod config;
mod navigation;

use config::{invalid, string};

fn configs(source: &Utf8Path) -> Vec<Utf8PathBuf> {
    ["js", "ts", "mjs", "mts"]
        .iter()
        .map(|extension| Utf8PathBuf::from(format!(".vitepress/config.{extension}")))
        .filter(|path| std::fs::symlink_metadata(source.join(path)).is_ok())
        .collect()
}

fn inventory(
    root: &Utf8Path,
    unmapped: &mut Vec<Unmapped>,
) -> Result<BTreeMap<Utf8PathBuf, String>> {
    let mut documents = BTreeMap::new();
    let entries = WalkDir::new(root)
        .sort_by_file_name()
        .into_iter()
        .filter_entry(|entry| {
            if entry.depth() == 0 {
                return true;
            }
            let name = entry.file_name().to_string_lossy();
            // These trees contain configuration, dependencies, partials or public
            // downloads, not documentation to promote into navigation.
            !name.starts_with(['.', '_']) && !["node_modules", "public"].contains(&name.as_ref())
        });
    for entry in entries {
        let entry = entry.map_err(|e| invalid(format!("cannot inspect source files: {e}")))?;
        let path = Utf8Path::from_path(entry.path())
            .ok_or_else(|| invalid("source paths must be UTF-8"))?;
        if path == root {
            continue;
        }
        let relative = Utf8PathBuf::from(
            path.strip_prefix(root)
                .unwrap()
                .components()
                .map(|p| p.as_str())
                .collect::<Vec<_>>()
                .join("/"),
        );
        let full = files::contained_path(root, &relative)?;
        if !entry.file_type().is_file() {
            continue;
        }
        if relative.extension() == Some("mdx") {
            unmapped.push(Unmapped {
                source: relative,
                reason: "VitePress Markdown import supports .md files, not MDX; not converted"
                    .into(),
            });
            continue;
        }
        if relative.extension() != Some("md") {
            continue;
        }
        if relative.as_str().contains(['[', ']']) {
            return Err(invalid(format!(
                "{relative}: dynamic route parameters require manual migration"
            )));
        }
        let raw = std::fs::read_to_string(full)?;
        crate::frontmatter::parse(&raw, &relative)?;
        documents.insert(relative, raw);
    }
    Ok(documents)
}

fn website(
    config: &Value,
    all: &BTreeMap<Utf8PathBuf, String>,
    imported: &BTreeSet<Utf8PathBuf>,
) -> Result<SourceUrls> {
    let base = string(config, "base")?.unwrap_or("/");
    if !base.starts_with('/') || !base.ends_with('/') {
        return Err(invalid("base must start and end with /"));
    }
    let mut urls = SourceUrls::new(None, base)?;
    let clean = match config.get("cleanUrls") {
        None => false,
        Some(Value::Bool(value)) => *value,
        _ => return Err(invalid("cleanUrls must be a boolean")),
    };
    for source in all.keys() {
        let stem = source.as_str().strip_suffix(".md").unwrap();
        let html = format!("{stem}.html");
        let index = stem == "index" || stem.ends_with("/index");
        let canonical = if index {
            stem.strip_suffix("index").unwrap()
        } else if clean {
            stem
        } else {
            &html
        };
        let target = imported.contains(source).then(|| route(source));
        // VitePress compiles authored /page.md and /page links and prepends
        // base. Also recognize addresses copied from an already-built site.
        for path in [source.as_str(), stem, html.as_str(), canonical] {
            urls.add(path, target.clone())?;
            urls.add_unprefixed(path, target.clone())?;
        }
        urls.add_page(source, canonical)?;
    }
    Ok(urls)
}

pub struct VitePress;

impl Converter for VitePress {
    fn id(&self) -> &'static str {
        "vitepress"
    }
    fn label(&self) -> &'static str {
        "VitePress — declarative .vitepress/config and Markdown"
    }
    fn detect(&self, source: &Utf8Path) -> Confidence {
        if configs(source).is_empty() {
            Confidence::NONE
        } else {
            Confidence::CERTAIN
        }
    }
    fn convert(&self, source: &Utf8Path, options: &ConvertOptions) -> Result<Conversion> {
        let paths = configs(source);
        if paths.len() != 1 {
            return Err(invalid(
                "expected exactly one .vitepress/config.js, .ts, .mjs or .mts file",
            ));
        }
        let config_path = files::contained_path(source, &paths[0])?;
        let settings = config::read(&std::fs::read_to_string(config_path)?)?;
        let src = string(&settings, "srcDir")?.unwrap_or(".");
        let src = if src == "./" {
            "."
        } else {
            src.strip_prefix("./").unwrap_or(src).trim_end_matches('/')
        };
        let root = if src == "." {
            source.to_owned()
        } else {
            files::contained_path(source, Utf8Path::new(src))?
        };
        if !root.is_dir() {
            return Err(invalid("srcDir must name an existing contained directory"));
        }
        let mut notes = vec!["VitePress configuration was read as data; no JavaScript, imports, helpers or Vue code were executed. Theme components, top navigation, home-page front matter and custom Markdown presentation require manual migration. Hidden/underscore trees, node_modules and public are not scanned as documentation. Only referenced assets are copied.".into()];
        let mut unmapped = Vec::new();
        let all = inventory(&root, &mut unmapped)?;
        let mut tree = DocTree {
            root,
            site_name: string(&settings, "title")?.map(str::to_owned),
            description: string(&settings, "description")?.map(str::to_owned),
            nav: navigation::read(&settings, &all, &mut notes, &mut unmapped)?,
            ..Default::default()
        };
        let imported: BTreeSet<_> = tree.pages().into_iter().cloned().collect();
        tree.orphans = all
            .keys()
            .filter(|p| !imported.contains(*p))
            .cloned()
            .collect();
        let documents: BTreeMap<_, _> = all
            .iter()
            .filter(|(p, _)| imported.contains(*p))
            .map(|(p, raw)| (p.clone(), raw.clone()))
            .collect();
        for (source, raw) in &documents {
            let metadata = crate::frontmatter::parse(raw, source)?;
            let mut fence = crate::fences::Fence::default();
            if metadata.contains("layout")
                || metadata.body.lines().any(|line| {
                    !fence.contains(line)
                        && [
                            ":::",
                            "<<<",
                            "<!--@include",
                            "<script",
                            "<style",
                            "<template",
                            "{{",
                        ]
                        .iter()
                        .any(|marker| line.contains(marker))
                })
            {
                unmapped.push(Unmapped { source: source.clone(), reason: "VitePress layout, Vue or Markdown extensions are retained as source text/metadata for manual migration, not converted into Zellij components".into() });
            }
        }
        let urls = website(&settings, &all, &imported)?;
        let mut conversion = emit(&tree, |p| documents.get(p).cloned())?;
        crate::references::rewrite(
            &tree,
            &documents,
            AssetSource::VitePress,
            Some(&urls),
            options,
            &mut conversion,
        )?;
        conversion.unmapped.extend(unmapped);
        conversion.notes.extend(notes);
        Ok(conversion)
    }
}

inventory::submit! { &VitePress as &dyn Converter }
