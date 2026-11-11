//! Resolve navigation patterns once, before document discovery and rendering.
use super::{document_path, document_target, external, invalid, rst};
use camino::{Utf8Path, Utf8PathBuf};
use regex::Regex;
use std::collections::BTreeSet;
use zellij_core::Result;

// Sphinx's patfilter semantics, not filesystem globbing: * and ? cannot
// cross '/', ** can, and **/ still requires a slash. Names omit .rst.
fn matcher(pattern: &str) -> Result<Regex> {
    if pattern.len() > 4096 {
        return Err(invalid("toctree glob exceeds 4096 bytes"));
    }
    let chars: Vec<_> = pattern.chars().collect();
    let mut out = String::from(r"\A(?:");
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        i += 1;
        match c {
            '*' if chars.get(i) == Some(&'*') => {
                i += 1;
                out.push_str(".*");
            }
            '*' => out.push_str("[^/]*"),
            '?' => out.push_str("[^/]"),
            '[' => {
                let start = i;
                let negative = chars.get(i) == Some(&'!');
                if negative {
                    i += 1;
                }
                if chars.get(i) == Some(&']') {
                    i += 1;
                }
                while i < chars.len() && chars[i] != ']' {
                    i += 1;
                }
                if i == chars.len() {
                    out.push_str(r"\[");
                    i = start;
                    continue;
                }
                out.push('[');
                if negative {
                    out.push_str("^/");
                }
                let contents = &chars[start + usize::from(negative)..i];
                for (offset, c) in contents.iter().enumerate() {
                    // Preserve ranges, but escape Rust regex class operators
                    // and literal leading/trailing hyphens.
                    if *c == '-' && offset > 0 && offset + 1 < contents.len() {
                        out.push('-');
                    } else {
                        out.push_str(&regex::escape(&c.to_string()));
                    }
                }
                out.push(']');
                i += 1;
            }
            _ => out.push_str(&regex::escape(&c.to_string())),
        }
    }
    out.push_str(r")\z");
    Regex::new(&out).map_err(|_| invalid(format!("invalid toctree glob {pattern:?}")))
}

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
    fn sphinx_patterns_are_case_sensitive_and_separator_aware() {
        for (pattern, yes, no) in [
            ("guide/*", "guide/intro", "guide/deep/intro"),
            ("guide/**", "guide/deep/intro", "Guide/intro"),
            ("**/intro", "guide/intro", "intro"),
            ("p?", "pé", "p/"),
            ("[a-c]", "b", "d"),
            ("[!a-c]", "z", "/"),
            ("[^a]", "^", "b"),
            ("[]a]", "]", "b"),
            ("[-a]", "-", "b"),
            ("[a-]", "-", "b"),
            ("[a&&b]", "&", "c"),
            ("[open", "[open", "o"),
            ("a.b", "a.b", "axb"),
        ] {
            let regex = matcher(pattern).unwrap();
            assert!(regex.is_match(yes), "{pattern}: {yes}");
            assert!(!regex.is_match(no), "{pattern}: {no}");
        }
        assert!(matcher("[z-a]").is_err());
        assert!(matcher(&"*".repeat(4097)).is_err());
    }

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
