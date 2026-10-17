use crate::Page;
use camino::{Utf8Path, Utf8PathBuf};
use std::collections::{BTreeMap, BTreeSet};
use zellij_core::{Result, files};

pub fn encode(value: &str) -> String {
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

pub fn decode(value: &str) -> Option<String> {
    let mut bytes = Vec::new();
    let mut iter = value.bytes();
    while let Some(b) = iter.next() {
        if b == b'%' {
            bytes.push(
                ((iter.next()? as char).to_digit(16)? * 16 + (iter.next()? as char).to_digit(16)?)
                    as u8,
            );
        } else {
            bytes.push(b);
        }
    }
    String::from_utf8(bytes).ok()
}

pub fn prefix(output: &Utf8Path) -> String {
    "../".repeat(output.parent().map_or(0, |p| p.components().count()))
}

pub fn href(from: &Utf8Path, to: &Utf8Path) -> String {
    format!("{}{}", prefix(from), encode(to.as_str()))
}

/// URL dot segments are resolved within the Markdown root, never the host FS.
pub fn resolve(parent: &Utf8Path, url: &str) -> Option<Utf8PathBuf> {
    let decoded = decode(url)?;
    if decoded.contains(['\\', ':']) || decoded.chars().any(char::is_control) {
        return None;
    }
    let mut parts: Vec<_> = if decoded.starts_with('/') {
        vec![]
    } else {
        parent
            .as_str()
            .split('/')
            .filter(|s| !s.is_empty())
            .collect()
    };
    for part in decoded.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                parts.pop()?;
            }
            other => parts.push(other),
        }
    }
    Some(parts.join("/").into())
}

pub struct Resolver<'a> {
    pub root: &'a Utf8Path,
    pub pages: &'a [Page],
    pub assets: BTreeMap<Utf8PathBuf, Vec<u8>>,
}

impl Resolver<'_> {
    pub fn rewrite(&mut self, page: &Page, url: &str, image: bool) -> Result<String> {
        let fail =
            |reason: &str| files::unsafe_path(&page.source, format!("reference {url:?}: {reason}"));
        if url.starts_with("//") {
            return Ok(format!("https:{url}"));
        }
        if let Some((scheme, _)) = url.split_once(':').filter(|(s, _)| {
            s.starts_with(|c: char| c.is_ascii_alphabetic())
                && s.chars()
                    .all(|c| c.is_ascii_alphanumeric() || "+-.".contains(c))
        }) {
            if ["https", "http", "mailto", "tel"].contains(&scheme.to_ascii_lowercase().as_str()) {
                return Ok(url.into());
            }
            return Err(fail("unsupported URL scheme"));
        }
        let end = url.find(['?', '#']).unwrap_or(url.len());
        let (path, suffix) = url.split_at(end);
        let fragment = url
            .split_once('#')
            .map(|(_, f)| decode(f).ok_or_else(|| fail("invalid fragment encoding")))
            .transpose()?;
        if path.is_empty() {
            self.check_fragment(page, fragment.as_deref(), &fail)?;
            return Ok(url.into());
        }
        let relative = resolve(page.source.parent().unwrap_or(Utf8Path::new("")), path)
            .ok_or_else(|| fail("path escapes the documentation root or has invalid encoding"))?;
        let candidate = if relative.as_str().is_empty() {
            self.root.to_owned()
        } else {
            files::contained_path(self.root, &relative)?
        };
        let mut target = None;
        if candidate.is_dir() {
            let candidates = self.pages.iter().filter(|p| {
                p.output.parent() == Some(relative.as_path())
                    && p.output.file_name() == Some("index.html")
            });
            target = candidates.into_iter().next();
        } else if candidate.is_file() {
            let canonical = candidate.canonicalize_utf8()?;
            target = self
                .pages
                .iter()
                .find(|p| !p.source.as_str().is_empty() && self.root.join(&p.source) == canonical);
        } else {
            // Extensionless authored links are unambiguous only when exactly
            // one discovered document has that source stem.
            let candidates: Vec<_> = self
                .pages
                .iter()
                .filter(|p| p.source.with_extension("") == relative || p.output == relative)
                .collect();
            if candidates.len() == 1 {
                target = Some(candidates[0]);
            }
        }
        if let Some(target) = target {
            if image {
                return Err(fail("an image points to a Markdown document"));
            }
            self.check_fragment(target, fragment.as_deref(), &fail)?;
            return Ok(format!("{}{suffix}", href(&page.output, &target.output)));
        }
        if !candidate.is_file() {
            return Err(fail("no matching document or local asset"));
        }
        let canonical = candidate.canonicalize_utf8()?;
        let relative = crate::url_path(
            canonical
                .strip_prefix(self.root)
                .map_err(|_| fail("asset escapes the documentation root"))?,
        );
        crate::portable_path(&relative)?;
        if relative
            .components()
            .next()
            .is_some_and(|p| p.as_str().eq_ignore_ascii_case("_zellij"))
        {
            return Err(fail("_zellij is reserved for generated assets"));
        }
        if !self.assets.contains_key(&relative) {
            self.assets
                .insert(relative.clone(), std::fs::read(canonical)?);
        }
        Ok(format!("{}{suffix}", href(&page.output, &relative)))
    }

    fn check_fragment(
        &self,
        target: &Page,
        fragment: Option<&str>,
        fail: &impl Fn(&str) -> zellij_core::Error,
    ) -> Result<()> {
        if let Some(id) = fragment.filter(|f| !f.is_empty()) {
            let mut ids: BTreeSet<&str> = target
                .markdown
                .headings
                .iter()
                .map(|h| h.id.as_str())
                .collect();
            if !target.markdown.has_h1 {
                ids.insert("_zel-title");
            }
            for event in &target.markdown.events {
                if let pulldown_cmark::Event::Start(pulldown_cmark::Tag::FootnoteDefinition(name)) =
                    event
                {
                    ids.insert(name);
                }
            }
            if !ids.contains(id) {
                return Err(fail(&format!(
                    "missing heading/footnote #{id} in {}",
                    target.source
                )));
            }
        }
        Ok(())
    }
}
