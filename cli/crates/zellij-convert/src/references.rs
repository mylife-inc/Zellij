//! Resolve references against the complete import, before writing any files.
//! Markdown offsets keep code examples and link labels out of the rewrite.

use crate::{
    Conversion, ConvertOptions, OutputAsset, Unmapped,
    emit::{slug_for, split_frontmatter},
    material::demote_material,
    mdx::make_mdx_safe,
    model::DocTree,
};
use camino::{Utf8Path, Utf8PathBuf};
use pulldown_cmark::{Event, LinkType, Options, Parser, Tag, TagEnd};
use regex::Regex;
use std::{
    collections::{BTreeMap, BTreeSet},
    ops::Range,
    sync::LazyLock,
};
use unicode_normalization::UnicodeNormalization;
use zellij_core::{
    Result,
    files::{contained_path, unsafe_path},
};

// Keep these rules aligned with content/naming.ts and guide-folder.ts. The CI
// fixture compares converted links to routes loaded by the actual engine.
fn route(source: &Utf8PathBuf) -> String {
    static PREFIX: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"^[0-9]+\s*[-._]\s*(\S.*)$").unwrap());
    static ACRONYM: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"([A-Z]+)([A-Z][a-z])").unwrap());
    static CAMEL: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"([a-z0-9])([A-Z])").unwrap());
    static SEPARATORS: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"[^a-z0-9]+").unwrap());
    let slug = slug_for(source);
    let parts: Vec<_> = slug.split('/').collect();
    let count = parts.len() - usize::from(parts.last() == Some(&"index"));
    let mut out = String::from("/guide");
    for part in &parts[..count] {
        let name = PREFIX.replace(part, "$1");
        let name = if name.contains(|c: char| c.is_whitespace() || "-_.".contains(c)) {
            name.into_owned()
        } else {
            CAMEL
                .replace_all(&ACRONYM.replace_all(&name, "$1 $2"), "$1 $2")
                .into_owned()
        };
        let folded: String = name
            .nfkd()
            .filter(|c| !(('\u{0300}'..='\u{036f}').contains(c)))
            .collect();
        let lower = folded.to_lowercase();
        let segment = SEPARATORS.replace_all(&lower, "-");
        let segment = segment.trim_matches('-');
        if !segment.is_empty() {
            out.push('/');
            out.push_str(segment);
        }
    }
    out
}

fn decode(value: &str) -> Option<String> {
    let mut bytes = Vec::new();
    let mut iter = value.bytes();
    while let Some(b) = iter.next() {
        if b == b'%' {
            let hi = (iter.next()? as char).to_digit(16)?;
            let lo = (iter.next()? as char).to_digit(16)?;
            bytes.push((hi * 16 + lo) as u8);
        } else {
            bytes.push(b);
        }
    }
    String::from_utf8(bytes).ok()
}

fn encode_path(value: &str) -> String {
    let mut out = String::new();
    for b in value.bytes() {
        if b.is_ascii_alphanumeric() || b"/-._~".contains(&b) {
            out.push(b as char);
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}

/// URL dot segments are legal within the source root. The resulting filesystem
/// path still goes through the shared portable-path and symlink checks.
fn resolve(parent: &Utf8Path, url: &str) -> Option<Utf8PathBuf> {
    let decoded = decode(url)?;
    if decoded.contains(['\\', ':', '\0']) || decoded.starts_with('/') {
        return None;
    }
    let mut parts: Vec<_> = parent
        .as_str()
        .split('/')
        .filter(|s| !s.is_empty())
        .collect();
    for part in decoded.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                parts.pop()?;
            }
            other => parts.push(other),
        }
    }
    let path = Utf8PathBuf::from(parts.join("/"));
    if !path.as_str().is_empty() {
        zellij_core::files::relative_path(&path).ok()?;
    }
    Some(path)
}

fn external(url: &str) -> bool {
    url.starts_with("//")
        || url.split_once(':').is_some_and(|(scheme, _)| {
            scheme.starts_with(|c: char| c.is_ascii_alphabetic())
                && scheme
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || "+-.".contains(c))
        })
}

struct Resolver<'a> {
    tree: &'a DocTree,
    project: Option<&'a Utf8Path>,
    options: &'a ConvertOptions,
    routes: BTreeMap<Utf8PathBuf, String>,
    assets: BTreeMap<Utf8PathBuf, Vec<u8>>,
    unmapped: Vec<Unmapped>,
    fragments: bool,
}

impl Resolver<'_> {
    fn report(&mut self, source: &Utf8Path, url: &str, reason: &str) {
        self.unmapped.push(Unmapped {
            source: source.to_owned(),
            reason: format!("reference {url:?}: {reason}; left unchanged"),
        });
    }

    fn href(&mut self, source: &Utf8Path, url: &str, image: bool) -> Result<String> {
        if external(url) || url.is_empty() {
            return Ok(url.to_owned());
        }
        if url.contains('#') {
            self.fragments = true;
        }
        if url.starts_with(['#', '?']) {
            return Ok(url.to_owned());
        }
        let end = url.find(['?', '#']).unwrap_or(url.len());
        let (path, suffix) = url.split_at(end);
        // Docusaurus documents static-folder assets as /img/x and
        // @site/static/img/x. Neither authorizes arbitrary project-root reads.
        let static_path = self.project.and_then(|_| {
            path.strip_prefix("@site/static/")
                .or_else(|| path.strip_prefix('/'))
        });
        let (root, relative, namespace) = if let Some(path) = static_path {
            let Some(relative) = resolve(Utf8Path::new(""), path) else {
                return Err(unsafe_path(
                    Utf8Path::new(path),
                    "asset reference escapes its static root or is not portable",
                ));
            };
            let root = contained_path(self.project.unwrap(), Utf8Path::new("static"))?;
            (root, relative, "static")
        } else {
            if path.starts_with('/') {
                self.report(source, url, "root-relative source URLs cannot be mapped without the source deployment configuration");
                return Ok(url.to_owned());
            }
            let Some(relative) = resolve(source.parent().unwrap_or(Utf8Path::new("")), path) else {
                return Err(unsafe_path(
                    Utf8Path::new(path),
                    "reference escapes the documentation root or is not a portable UTF-8 path",
                ));
            };
            if !image && let Some(route) = self.routes.get(&relative) {
                return Ok(format!("{route}{suffix}"));
            }
            if !image && relative.extension().is_none() {
                // A unique extensionless document or directory index is safe;
                // competing candidates are reported instead of guessed.
                let candidates = [
                    relative.with_extension("md"),
                    relative.with_extension("mdx"),
                    relative.with_extension("markdown"),
                    relative.join("index.md"),
                    relative.join("index.mdx"),
                    relative.join("index.markdown"),
                ];
                let routes: BTreeSet<_> = candidates
                    .iter()
                    .filter_map(|p| self.routes.get(p))
                    .collect();
                if routes.len() == 1 {
                    return Ok(format!("{}{suffix}", routes.first().unwrap()));
                }
                if routes.len() > 1 {
                    self.report(source, url, "ambiguous document target");
                    return Ok(url.to_owned());
                }
            }
            (self.tree.root.clone(), relative, "docs")
        };
        if relative.as_str().is_empty() {
            self.report(
                source,
                url,
                "no converted index document or asset at this root",
            );
            return Ok(url.to_owned());
        }
        // Do not publish unlisted documents as downloadable assets.
        if relative.extension().is_some_and(|extension| {
            ["md", "mdx", "markdown"]
                .iter()
                .any(|known| extension.eq_ignore_ascii_case(known))
        }) {
            self.report(
                source,
                url,
                "document is missing or was not included in the converted navigation",
            );
            return Ok(url.to_owned());
        }
        let full = contained_path(&root, &relative)?;
        if !full.is_file() {
            self.report(
                source,
                url,
                "no converted document or regular asset file at this path",
            );
            return Ok(url.to_owned());
        }
        if !self.options.copy_assets {
            self.report(source, url, "asset copying is disabled");
            return Ok(url.to_owned());
        }
        let destination = Utf8PathBuf::from(format!("assets/imported/{namespace}/{relative}"));
        if !self.assets.contains_key(&destination) {
            self.assets
                .insert(destination.clone(), std::fs::read(&full)?);
        }
        let public = destination.as_str().strip_prefix("assets/").unwrap();
        Ok(format!("/_zellij/assets/{}{suffix}", encode_path(public)))
    }

    fn markdown(&mut self, source: &Utf8Path, body: &str) -> Result<String> {
        let parser = Parser::new_ext(
            body,
            Options::ENABLE_TABLES | Options::ENABLE_STRIKETHROUGH | Options::ENABLE_FOOTNOTES,
        );
        let definitions: Vec<_> = parser
            .reference_definitions()
            .iter()
            .map(|(_, def)| def.span.clone())
            .collect();
        let mut edits: Vec<(Range<usize>, String)> = Vec::new();
        // The parser gives a link's full span. Child spans locate the closing
        // label bracket without guessing at escaped brackets or inline code.
        struct Link {
            span: Range<usize>,
            label_end: usize,
            url: String,
            title: String,
            kind: LinkType,
            image: bool,
        }
        let mut stack: Vec<Link> = Vec::new();
        for (event, span) in parser.into_offset_iter() {
            let image = matches!(&event, Event::Start(Tag::Image { .. }));
            match event {
                Event::Start(Tag::Link {
                    link_type,
                    dest_url,
                    title,
                    ..
                })
                | Event::Start(Tag::Image {
                    link_type,
                    dest_url,
                    title,
                    ..
                }) => {
                    if let Some(parent) = stack.last_mut() {
                        parent.label_end = parent.label_end.max(span.end);
                    }
                    stack.push(Link {
                        label_end: span.start + if image { 2 } else { 1 },
                        span,
                        url: dest_url.to_string(),
                        title: title.to_string(),
                        kind: link_type,
                        image,
                    });
                }
                Event::End(TagEnd::Link | TagEnd::Image) => {
                    let link = stack.pop().expect("balanced Markdown links");
                    if matches!(link.kind, LinkType::Autolink | LinkType::Email) {
                        continue;
                    }
                    let url = self.href(source, &link.url, link.image)?;
                    // Expand reference uses to inline links; definitions can
                    // then be removed, including multiline/angle destinations
                    // which the MDX safety pass otherwise treats as markup.
                    if url == link.url && link.kind == LinkType::Inline {
                        continue;
                    }
                    let tail = &body[link.label_end..link.span.end];
                    let Some(close) = tail.find(']') else {
                        continue;
                    };
                    let title = if link.title.is_empty() {
                        String::new()
                    } else {
                        format!(
                            " \"{}\"",
                            link.title.replace('\\', "\\\\").replace('"', "\\\"")
                        )
                    };
                    edits.push((
                        link.label_end + close..link.span.end,
                        format!("](<{url}>{title})"),
                    ));
                }
                Event::Html(ref html) | Event::InlineHtml(ref html) => {
                    static ATTRIBUTE: LazyLock<Regex> =
                        LazyLock::new(|| Regex::new(r"(?i)\b(?:src|srcset|href)\s*=").unwrap());
                    if ATTRIBUTE.is_match(html) {
                        self.report(
                            source,
                            html,
                            "HTML/JSX asset and link attributes require manual migration",
                        );
                    }
                    if let Some(parent) = stack.last_mut() {
                        parent.label_end = parent.label_end.max(span.end);
                    }
                }
                _ => {
                    if let Some(parent) = stack.last_mut() {
                        parent.label_end = parent.label_end.max(span.end);
                    }
                }
            }
        }
        for span in definitions {
            edits.push((span, String::new()));
        }
        edits.sort_by_key(|(span, _)| span.start);
        let mut out = body.to_owned();
        for (span, replacement) in edits.into_iter().rev() {
            out.replace_range(span, &replacement);
        }
        Ok(out)
    }
}

pub(crate) fn rewrite(
    tree: &DocTree,
    documents: &BTreeMap<Utf8PathBuf, String>,
    project: Option<&Utf8Path>,
    options: &ConvertOptions,
    conversion: &mut Conversion,
) -> Result<()> {
    let mut resolver = Resolver {
        tree,
        project,
        options,
        routes: BTreeMap::new(),
        assets: BTreeMap::new(),
        unmapped: Vec::new(),
        fragments: false,
    };
    let mut occupied = BTreeMap::new();
    for source in documents.keys() {
        if source.as_str().split('/').any(|part| part.starts_with('.'))
            || source.file_name().is_some_and(|name| name.starts_with('_'))
        {
            return Err(unsafe_path(
                source,
                "document name is hidden from the engine; rename it before importing",
            ));
        }
        let target = route(source);
        if let Some(previous) = occupied.insert(target.clone(), source) {
            return Err(unsafe_path(
                source,
                format!("both {previous} and {source} would render at {target}"),
            ));
        }
        resolver.routes.insert(source.clone(), target.clone());
        if project.is_some() && source.extension() == Some("md") && !tree.root.join(source).exists()
        {
            resolver.routes.insert(source.with_extension("mdx"), target);
        }
    }
    for (source, raw) in documents {
        let (_, body) = split_frontmatter(raw);
        let body = demote_material(body);
        let rewritten = resolver.markdown(source, &body)?;
        let output = Utf8PathBuf::from(format!("01-guide/{}.mdx", slug_for(source)));
        if let Some(file) = conversion.files.iter_mut().find(|f| f.path == output) {
            let (_, old_body) = split_frontmatter(&file.body);
            let prefix_len = file.body.len() - old_body.len();
            file.body.truncate(prefix_len);
            file.body
                .push_str(make_mdx_safe(&rewritten).trim_start_matches('\n'));
        }
    }
    conversion.assets = resolver
        .assets
        .into_iter()
        .map(|(path, body)| OutputAsset { path, body })
        .collect();
    conversion.unmapped.extend(resolver.unmapped);
    if resolver.fragments {
        conversion.notes.push("Link fragments were retained. Review source-specific heading IDs and links to a leading H1 (Zellij renders that title outside the Markdown body); those anchors are not translated automatically.".into());
    }
    Ok(())
}
