//! Import declarative Sphinx/reStructuredText documentation without Python.
mod config;
mod rst;

use crate::{
    Confidence, Conversion, ConvertOptions, Converter, Unmapped,
    emit::emit,
    model::{DocTree, Entry},
    references::{AssetSource, encode_path, route},
    source_urls::SourceUrls,
};
use camino::{Utf8Path, Utf8PathBuf};
use std::collections::{BTreeMap, BTreeSet};
use zellij_core::{
    Error, Result,
    files::{contained_path, relative_path},
};

pub struct Sphinx;
fn invalid(message: impl std::fmt::Display) -> Error {
    Error::InvalidArgument(format!("Sphinx import: {message}"))
}
fn output(path: &Utf8Path) -> Utf8PathBuf {
    format!("{}.md", path.as_str().strip_suffix(".rst").unwrap()).into()
}
fn external(target: &str) -> bool {
    target.starts_with("https://") || target.starts_with("http://")
}
fn document_path(source: &Utf8Path, target: &str) -> Result<Utf8PathBuf> {
    if target.is_empty()
        || target.contains(['\\', ':', '#', '?', '%', '*', '[', ']'])
        || target.starts_with("//")
    {
        return Err(invalid(format!("unsupported document target {target:?}")));
    }
    let mut parts: Vec<_> = if target.starts_with('/') {
        Vec::new()
    } else {
        source
            .parent()
            .unwrap()
            .as_str()
            .split('/')
            .filter(|s| !s.is_empty())
            .collect()
    };
    for part in target.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                parts
                    .pop()
                    .ok_or_else(|| invalid("document target escapes source root"))?;
            }
            _ => parts.push(part),
        }
    }
    let path = parts.join("/");
    let path = Utf8PathBuf::from(if path.ends_with(".rst") {
        path
    } else {
        format!("{path}.rst")
    });
    relative_path(&path)?;
    Ok(path)
}
fn relative_link(source: &Utf8Path, target: &Utf8Path) -> String {
    let from: Vec<_> = source
        .parent()
        .unwrap()
        .as_str()
        .split('/')
        .filter(|s| !s.is_empty())
        .collect();
    let to: Vec<_> = target.as_str().split('/').collect();
    let common = from.iter().zip(&to).take_while(|(a, b)| a == b).count();
    format!(
        "{}{}",
        "../".repeat(from.len() - common),
        encode_path(&to[common..].join("/"))
    )
}
fn asset_reference(source: &Utf8Path, target: &str) -> String {
    // Root-relative source resources do not inherit html_baseurl. The shared
    // resolver still enforces containment and the eligible-resource allowlist.
    if let Some(path) = target.strip_prefix('/')
        && !target.starts_with("//")
    {
        let depth = source.parent().unwrap().components().count();
        format!("{}{path}", "../".repeat(depth))
    } else {
        target.into()
    }
}
fn navigation(
    path: &Utf8PathBuf,
    label: Option<String>,
    docs: &BTreeMap<Utf8PathBuf, rst::Document>,
    stack: &mut BTreeSet<Utf8PathBuf>,
    budget: &mut usize,
) -> Result<Vec<Entry>> {
    if stack.len() >= 32 || *budget >= 10000 {
        return Err(invalid("toctree exceeds 32 levels or 10000 entries"));
    }
    *budget += 1;
    if !stack.insert(path.clone()) {
        return Err(invalid(format!("cyclic toctree at {path}")));
    }
    let doc = docs
        .get(path)
        .ok_or_else(|| invalid(format!("toctree document {path} is missing or excluded")))?;
    let mut entries = vec![Entry::Page {
        label,
        source: path.clone(),
    }];
    for block in &doc.blocks {
        let rst::Block::Toc(toc) = block else {
            continue;
        };
        let mut children = Vec::new();
        for (label, target) in &toc.entries {
            if target == "self" {
                continue;
            }
            if external(target) {
                children.push(Entry::Foreign {
                    label: label.clone().unwrap_or(target.clone()),
                    target: target.clone(),
                });
                continue;
            }
            let next = document_path(path, target)?;
            let child = docs.get(&next).ok_or_else(|| {
                invalid(format!("toctree document {next} is missing or excluded"))
            })?;
            let mut nested = navigation(&next, label.clone(), docs, stack, budget)?;
            if nested.len() == 1 {
                children.append(&mut nested);
            } else {
                children.push(Entry::Group {
                    label: label.clone().unwrap_or(child.title.clone()),
                    items: nested,
                });
            }
        }
        if let Some(caption) = &toc.caption {
            entries.push(Entry::Group {
                label: caption.clone(),
                items: children,
            });
        } else {
            entries.append(&mut children);
        }
    }
    stack.remove(path);
    Ok(entries)
}

impl Converter for Sphinx {
    fn id(&self) -> &'static str {
        "sphinx"
    }
    fn label(&self) -> &'static str {
        "Sphinx (declarative reStructuredText)"
    }
    fn detect(&self, source: &Utf8Path) -> Confidence {
        if source.join("conf.py").is_file() {
            Confidence::CERTAIN
        } else {
            Confidence::NONE
        }
    }
    fn convert(&self, source: &Utf8Path, options: &ConvertOptions) -> Result<Conversion> {
        let conf = contained_path(source, Utf8Path::new("conf.py"))?;
        let config = config::parse(&std::fs::read_to_string(conf)?)?;
        config::validate(&config)?;
        let root = Utf8PathBuf::from_path_buf(std::fs::canonicalize(source)?)
            .map_err(|_| invalid("source path is not UTF-8"))?;
        let mut excluded = config::strings(&config, "exclude_patterns")?;
        for key in [
            "templates_path",
            "html_theme_path",
            "html_static_path",
            "html_extra_path",
        ] {
            for path in config::strings(&config, key)? {
                contained_path(&root, Utf8Path::new(&path))?;
                // Static resources remain available to explicit image/download
                // references, but templates/themes never enter the asset set.
                if matches!(key, "templates_path" | "html_theme_path") {
                    excluded.push(path);
                }
            }
        }
        excluded.extend(["_build".into(), "_templates".into(), "node_modules".into()]);
        let excluded: Vec<_> = excluded.into_iter().map(Utf8PathBuf::from).collect();
        for path in &excluded {
            contained_path(&root, path)?;
        }
        let mut docs = BTreeMap::new();
        let mut assets = BTreeSet::new();
        let mut unmapped = Vec::new();
        for entry in walkdir::WalkDir::new(&root)
            .min_depth(1)
            .sort_by_file_name()
            .into_iter()
            .filter_entry(|e| {
                !e.file_type().is_dir() || !e.file_name().to_string_lossy().starts_with('.')
            })
        {
            let entry = entry.map_err(|e| invalid(e.to_string()))?;
            let path = Utf8Path::from_path(entry.path())
                .ok_or_else(|| invalid("source path is not UTF-8"))?;
            let rel = Utf8PathBuf::from(
                path.strip_prefix(&root)
                    .unwrap()
                    .components()
                    .map(|c| c.as_str())
                    .collect::<Vec<_>>()
                    .join("/"),
            );
            contained_path(&root, &rel)?;
            if !entry.file_type().is_file() {
                continue;
            }
            if excluded.iter().any(|p| rel.starts_with(p))
                || rel.components().any(|p| p.as_str().starts_with('.'))
            {
                if rel.extension() == Some("rst") {
                    unmapped.push(Unmapped {
                        source: rel,
                        reason: "excluded Sphinx source; not imported".into(),
                    });
                }
                continue;
            }
            if rel.extension() == Some("rst") {
                if rel.components().any(|p| p.as_str().starts_with('_'))
                    || matches!(rel.file_stem(), Some("genindex" | "modindex" | "search"))
                {
                    return Err(invalid(format!("reserved Sphinx document name {rel}")));
                }
                let doc = rst::parse(
                    &std::fs::read_to_string(path)?,
                    config::string(&config, "highlight_language")?.unwrap_or("text"),
                )
                .map_err(|e| invalid(format!("{rel}: {e}")))?;
                docs.insert(rel, doc);
            } else if !matches!(
                rel.extension().unwrap_or("").to_ascii_lowercase().as_str(),
                "py" | "pyc"
                    | "pyo"
                    | "rst"
                    | "md"
                    | "mdx"
                    | "markdown"
                    | "html"
                    | "htm"
                    | "yaml"
                    | "yml"
                    | "toml"
                    | "json"
                    | "js"
                    | "ts"
                    | "sh"
                    | "bat"
            ) && !matches!(rel.file_name(), Some("Makefile" | "makefile"))
            {
                assets.insert(rel);
            }
        }
        let main = config::string(&config, "root_doc")?
            .or(config::string(&config, "master_doc")?)
            .unwrap_or("index");
        let main = document_path(Utf8Path::new("index.rst"), main)?;
        let nav = navigation(&main, None, &docs, &mut BTreeSet::new(), &mut 0)?;
        let mut tree = DocTree {
            root,
            site_name: config::string(&config, "project")?.map(str::to_owned),
            nav,
            ..Default::default()
        };
        let published: BTreeSet<_> = tree.pages().into_iter().cloned().collect();
        tree.orphans = docs
            .keys()
            .filter(|p| !published.contains(*p))
            .cloned()
            .collect();
        tree.output_sources = docs.keys().map(|p| (p.clone(), output(p))).collect();
        let mut labels = BTreeMap::new();
        for (path, doc) in &docs {
            for (label, (anchor, title)) in &doc.labels {
                if labels
                    .insert(label.clone(), (path, anchor, title))
                    .is_some()
                {
                    return Err(invalid(format!("duplicate global Sphinx label {label:?}")));
                }
            }
        }
        let mut documents = BTreeMap::new();
        for path in &published {
            let doc = &docs[path];
            let resolve = |kind: &str, text: &str| -> Result<String> {
                if kind == "asset" {
                    return Ok(asset_reference(path, text));
                }
                let (label, target) = rst::target(text)?;
                if kind == "download" {
                    return rst::link(
                        label.as_deref().unwrap_or(&target),
                        &asset_reference(path, &target),
                    );
                }
                if matches!(kind, "doc" | "toc") && !external(&target) {
                    let target = if target == "self" && kind == "toc" {
                        path.clone()
                    } else {
                        document_path(path, &target)?
                    };
                    if !published.contains(&target) {
                        return Err(invalid(format!(
                            "{path}: reference to missing, excluded or unlisted document {target}"
                        )));
                    }
                    return rst::link(
                        label.as_deref().unwrap_or(&docs[&target].title),
                        &relative_link(path, &target),
                    );
                }
                if kind == "ref" {
                    let (target, anchor, title) =
                        labels.get(&rst::name(&target)).ok_or_else(|| {
                            invalid(format!("{path}: unresolved section label {target:?}"))
                        })?;
                    if !published.contains(*target) {
                        return Err(invalid(format!(
                            "{path}: section label belongs to an unlisted document"
                        )));
                    }
                    return rst::link(
                        label.as_deref().unwrap_or(title),
                        &format!("{}#{anchor}", relative_link(path, target)),
                    );
                }
                if kind == "link" && label.is_none() {
                    let href = doc.links.get(&rst::name(&target)).ok_or_else(|| {
                        invalid(format!("{path}: unresolved named hyperlink {target:?}"))
                    })?;
                    if href.ends_with('_') {
                        return Err(invalid(
                            "indirect named hyperlinks require manual migration",
                        ));
                    }
                    return rst::link(&target, href);
                }
                rst::link(label.as_deref().unwrap_or(&target), &target)
            };
            documents.insert(
                path.clone(),
                rst::render(&doc.blocks, &resolve).map_err(|e| invalid(format!("{path}: {e}")))?,
            );
            for reason in &doc.notes {
                unmapped.push(Unmapped {
                    source: path.clone(),
                    reason: reason.clone(),
                });
            }
        }
        let mut website = SourceUrls::from_site_url(config::string(&config, "html_baseurl")?)?;
        for path in docs.keys() {
            let address = format!("/{}.html", path.as_str().strip_suffix(".rst").unwrap());
            let target = published.contains(path).then(|| route(&output(path)));
            website.add(&address, target.clone())?;
            website.add_page(path, &address)?;
            if path.as_str() == "index.rst" {
                website.add("/", target)?;
            }
        }
        let mut result = emit(&tree, |p| documents.get(p).cloned())?;
        crate::references::rewrite(
            &tree,
            &documents,
            AssetSource::Sphinx(&assets),
            Some(&website),
            options,
            &mut result,
        )?;
        result.unmapped.extend(unmapped);
        result.notes.push("Sphinx was read without running Python, Docutils, extensions, templates or a source build. Imports follow the root document's explicit toctree (including hidden trees); unlisted documents are not published. HTML-builder .html URLs are mapped; dirhtml/custom builders, automatic numbering, alternate heading anchors and theme presentation require review. Only referenced eligible resources are copied; html_static_path/html_extra_path are not copied wholesale. The original source is unchanged.".into());
        Ok(result)
    }
}
inventory::submit! { &Sphinx as &dyn Converter }
