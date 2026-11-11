//! Resolve navigation patterns once, before document discovery and rendering.
use super::{document_path, document_target, external, invalid, patterns::matcher, rst};
use camino::{Utf8Path, Utf8PathBuf};
use std::collections::BTreeSet;
use zellij_core::Result;

pub(super) fn expand(
    owner: &Utf8Path,
    doc: &mut rst::Document,
    files: &BTreeSet<Utf8PathBuf>,
    budget: &mut usize,
) -> Result<()> {
    for block in &mut doc.blocks {
        let rst::Block::Toc(toc) = block else {
            continue;
        };
        if toc.glob {
            // Never introduce the owning document, generated pages, hidden
            // or internal documents. Exclusions/symlinks are checked by the
            // source inventory before this point. Each tree has its own set.
            let mut remaining: BTreeSet<_> = files
                .iter()
                .filter(|p| {
                    p.as_path() != owner
                        && !p.components().any(|c| c.as_str().starts_with(['.', '_']))
                        && !matches!(p.file_stem(), Some("genindex" | "modindex" | "search"))
                })
                .collect();
            let mut entries = Vec::new();
            for (label, target) in std::mem::take(&mut toc.entries) {
                if label.is_none() && !external(&target) && target.contains(['*', '?', '[']) {
                    let pattern = matcher(&document_target(owner, &target, true)?)?;
                    let matched: Vec<_> = remaining
                        .iter()
                        .copied()
                        .filter(|p| pattern.is_match(p.as_str().strip_suffix(".rst").unwrap()))
                        .take(budget.saturating_add(1))
                        .collect();
                    if matched.is_empty() {
                        doc.notes.push(format!(
                            "{owner}: toctree glob {target:?} matched no remaining documents"
                        ));
                    }
                    if matched.len() > *budget {
                        return Err(invalid("toctree globs exceed 10000 expanded entries"));
                    }
                    *budget -= matched.len();
                    for file in matched {
                        remaining.remove(file);
                        let target = format!("/{file}");
                        // Glob expansion retains the same filename/containment
                        // contract as explicitly named documents.
                        document_path(owner, &target)?;
                        entries.push((None, target));
                    }
                } else {
                    if target != "self" && !external(&target) {
                        remaining.remove(&document_path(owner, &target)?);
                    }
                    entries.push((label, target));
                }
            }
            toc.entries = entries;
        }
        if toc.reversed {
            toc.entries.reverse();
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn glob_expansion_has_a_shared_budget_and_per_tree_candidates() {
        let files = ["index.rst", "a.rst", "b.rst"]
            .map(Utf8PathBuf::from)
            .into();
        let make_doc = || rst::Document {
            title: "Home".into(),
            blocks: (0..2)
                .map(|_| {
                    rst::Block::Toc(rst::Toc {
                        glob: true,
                        entries: vec![(None, "*".into())],
                        ..Default::default()
                    })
                })
                .collect(),
            labels: Default::default(),
            links: Default::default(),
            notes: Vec::new(),
            substitutions: Default::default(),
        };
        let mut doc = make_doc();
        let mut budget = 4;
        expand(Utf8Path::new("index.rst"), &mut doc, &files, &mut budget).unwrap();
        assert_eq!(budget, 0);
        for block in doc.blocks {
            let rst::Block::Toc(toc) = block else {
                panic!()
            };
            assert_eq!(
                toc.entries,
                vec![(None, "/a.rst".into()), (None, "/b.rst".into())]
            );
        }
        let err = expand(Utf8Path::new("index.rst"), &mut make_doc(), &files, &mut 3).unwrap_err();
        assert!(err.to_string().contains("10000 expanded entries"));
    }
}
