//! Bounded local text resources, resolved only for selected published documents.
use super::{
    invalid,
    rst::{self, Block, Document},
};
use camino::{Utf8Path, Utf8PathBuf};
use std::{
    collections::{BTreeMap, BTreeSet},
    io::Read,
};
use zellij_core::{Result, files::contained_path};
const FILE_BYTES: usize = 8 * 1024 * 1024;
const TOTAL_BYTES: usize = 32 * 1024 * 1024;

pub struct Resolver<'a> {
    root: &'a Utf8Path,
    files: &'a BTreeSet<Utf8PathBuf>,
    encoding: &'a str,
    cache: BTreeMap<Utf8PathBuf, String>,
    cached_bytes: usize,
    expanded_bytes: usize,
    include_count: usize,
    pub included_files: BTreeSet<Utf8PathBuf>,
}

impl<'a> Resolver<'a> {
    pub fn new(root: &'a Utf8Path, files: &'a BTreeSet<Utf8PathBuf>, encoding: &'a str) -> Self {
        Self {
            root,
            files,
            encoding,
            cache: BTreeMap::new(),
            cached_bytes: 0,
            expanded_bytes: 0,
            include_count: 0,
            included_files: BTreeSet::new(),
        }
    }

    fn path(&self, source: &Utf8Path, target: &str) -> Result<Utf8PathBuf> {
        if target.starts_with("//")
            || target.contains(['\\', ':', '<', '>'])
            || target.chars().any(char::is_control)
        {
            return Err(invalid("local resource requires a local portable filename"));
        }
        let mut parts: Vec<_> = if target.starts_with('/') {
            Vec::new()
        } else {
            source
                .parent()
                .unwrap()
                .as_str()
                .split('/')
                .filter(|p| !p.is_empty())
                .collect()
        };
        for part in target.split('/') {
            match part {
                "" | "." => {}
                ".." => {
                    if parts.pop().is_none() {
                        return Err(invalid("local resource path escapes the source folder"));
                    }
                }
                _ => parts.push(part),
            }
        }
        let rel = Utf8PathBuf::from(parts.join("/"));
        if rel.as_str().is_empty()
            || parts.iter().any(|p| p.starts_with('.'))
            || !self.files.contains(&rel)
        {
            return Err(invalid(
                "local resource target is not an eligible source file; check its exact case, existence and exclusions",
            ));
        }
        Ok(rel)
    }

    fn read(&mut self, source: &Utf8Path, target: &str) -> Result<&str> {
        let rel = self.path(source, target)?;
        if !self.cache.contains_key(&rel) {
            let path = contained_path(self.root, &rel)?;
            let metadata = std::fs::metadata(&path)
                .map_err(|e| invalid(format!("local resource {rel}: {e}")))?;
            if !metadata.is_file() || metadata.len() > FILE_BYTES as u64 {
                return Err(invalid(format!(
                    "local resource {rel}: requires a regular file of at most 8 MiB"
                )));
            }
            let mut bytes = Vec::new();
            std::fs::File::open(&path)?
                .take((FILE_BYTES + 1) as u64)
                .read_to_end(&mut bytes)?;
            if bytes.len() > FILE_BYTES || bytes.len() > TOTAL_BYTES - self.cached_bytes {
                return Err(invalid(
                    "local resource input exceeds the 8 MiB file or 32 MiB total limit",
                ));
            }
            self.cached_bytes += bytes.len();
            let raw = String::from_utf8(bytes)
                .map_err(|_| invalid(format!("local resource {rel}: file is not UTF-8")))?;
            if raw.chars().any(|c| {
                (c.is_control() && !matches!(c, '\t' | '\r' | '\n'))
                    || matches!(c, '\u{2028}' | '\u{2029}')
            }) {
                return Err(invalid(format!(
                    "local resource {rel}: unsupported text control character"
                )));
            }
            self.cache
                .insert(rel.clone(), raw.replace("\r\n", "\n").replace('\r', "\n"));
        }
        Ok(&self.cache[&rel])
    }

    fn charge(&mut self, bytes: usize) -> Result<()> {
        if bytes > TOTAL_BYTES - self.expanded_bytes {
            return Err(invalid(
                "local resources total expanded text exceeds 32 MiB",
            ));
        }
        self.expanded_bytes += bytes;
        Ok(())
    }

    pub fn include(
        &mut self,
        source: &Utf8Path,
        spec: &super::include::Spec,
    ) -> Result<(Utf8PathBuf, String)> {
        self.include_count += 1;
        if self.include_count > 10000 {
            return Err(invalid("include expansion exceeds 10000 directives"));
        }
        let path = self.path(source, &spec.target)?;
        let encoding = spec.encoding.as_deref().unwrap_or(self.encoding);
        let body = spec.excerpt(self.read(source, &spec.target)?, encoding)?;
        self.charge(body.len())?;
        self.included_files.insert(path.clone());
        Ok((path, body))
    }

    pub fn resolve_document(&mut self, source: &Utf8Path, doc: &mut Document) -> Result<()> {
        let mut blocks = std::mem::take(&mut doc.blocks);
        self.resolve(source, &mut blocks, doc)?;
        doc.blocks = blocks;
        Ok(())
    }

    fn resolve(
        &mut self,
        source: &Utf8Path,
        blocks: &mut [Block],
        doc: &mut Document,
    ) -> Result<()> {
        for block in blocks {
            match block {
                Block::LiteralInclude(spec) => {
                    let encoding = spec.encoding.as_deref().unwrap_or(self.encoding);
                    let body = spec.excerpt(self.read(source, &spec.target)?, encoding)?;
                    self.charge(body.len())?;
                    *block = Block::Code {
                        language: spec.language.clone(),
                        body,
                    };
                }
                Block::CsvTable(spec) => {
                    let encoding = spec.encoding.as_deref().unwrap_or(self.encoding);
                    let csv_path = self.path(&spec.source, &spec.target)?;
                    let raw = self.read(&spec.source, &spec.target)?;
                    let raw = if encoding == "utf-8-sig" {
                        raw.strip_prefix('\u{feff}').unwrap_or(raw)
                    } else {
                        raw
                    };
                    let bytes = raw.len();
                    let table = super::table_directives::parse("csv-table", &spec.options, raw)?;
                    self.charge(bytes)?;
                    *block = rst::parse_external_table(table, spec, doc, source, &csv_path, self)?;
                    self.resolve(source, std::slice::from_mut(block), doc)?;
                }
                Block::Note { body, .. } => self.resolve(source, body, doc)?,
                Block::List { items, .. } => {
                    for item in items {
                        self.resolve(source, item, doc)?;
                    }
                }
                Block::Table { rows, .. } => {
                    for row in rows {
                        for cell in row {
                            self.resolve(source, cell, doc)?;
                        }
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }
}
