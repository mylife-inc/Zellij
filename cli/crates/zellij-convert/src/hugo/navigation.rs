use super::{Page, data::invalid};
use crate::model::Entry;
use camino::{Utf8Path, Utf8PathBuf};
use std::{
    cmp::Reverse,
    collections::{BTreeMap, BTreeSet},
};
use zellij_core::Result;

pub(super) fn derive(pages: &BTreeMap<Utf8PathBuf, Page>) -> Result<Vec<Entry>> {
    fn walk(
        dir: &Utf8Path,
        pages: &BTreeMap<Utf8PathBuf, Page>,
        depth: usize,
    ) -> Result<Vec<Entry>> {
        if depth > 32 {
            return Err(invalid("section nesting exceeds 32 levels"));
        }
        let mut start = Vec::new();
        let mut items = Vec::new();
        let mut dirs = BTreeSet::new();
        for (path, page) in pages.iter().filter(|(_, p)| p.published) {
            if path.parent() == Some(dir) {
                let entry = Entry::Page {
                    source: path.clone(),
                    label: Some(page.title.clone()),
                };
                if page.branch {
                    start.push(entry);
                } else {
                    items.push((Some(page), path.clone(), entry));
                }
            } else if let Ok(tail) = path.strip_prefix(dir) {
                let child = dir.join(tail.components().next().unwrap().as_str());
                dirs.insert(child);
            }
        }
        for child in dirs {
            let landing = pages.iter().find(|(p, page)| {
                p.parent() == Some(&child) && (page.branch || page.leaf) && page.published
            });
            let children = walk(&child, pages, depth + 1)?;
            if children.is_empty() {
                continue;
            }
            let page = landing.map(|(_, p)| p);
            let label = page
                .map(|p| p.title.clone())
                .unwrap_or_else(|| child.file_name().unwrap().to_owned());
            let entry = if page.is_some_and(|p| p.leaf) {
                children.into_iter().next().unwrap()
            } else {
                Entry::Group {
                    label,
                    items: children,
                }
            };
            items.push((page, child, entry));
        }
        items.sort_by_key(|(p, path, _)| {
            let weight = p.map_or(0, |p| p.weight);
            (
                weight == 0,
                weight,
                Reverse(p.map_or(i64::MIN, |p| p.date)),
                p.map(|p| p.title.to_lowercase())
                    .unwrap_or_else(|| path.file_name().unwrap().to_lowercase()),
                path.clone(),
            )
        });
        start.extend(items.into_iter().map(|(_, _, e)| e));
        Ok(start)
    }
    walk(Utf8Path::new(""), pages, 0)
}
