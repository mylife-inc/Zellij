//! What every documentation tool has in common, once its own vocabulary is
//! stripped away.
//!
//! MkDocs calls it `nav`, VitePress calls it `sidebar`, Sphinx calls it a
//! `toctree`. They are the same thing: an ordered tree of labelled entries,
//! some of which are pages and some of which are groups. Converting into this
//! first is what lets the Zellij side be written once rather than once per
//! format — and it is where a second converter costs a parser instead of a
//! rewrite.

use camino::Utf8PathBuf;

/// An entry in a documentation tree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Entry {
    /// A page, and the file it came from.
    Page {
        /// What the navigation calls it. Absent when the tool leaves it to the
        /// page's own first heading.
        label: Option<String>,
        /// Relative to the source documentation root.
        source: Utf8PathBuf,
    },
    /// A named group of entries. Nests to any depth.
    Group { label: String, items: Vec<Entry> },
    /// Something the source navigation pointed at that has no Zellij
    /// equivalent — an external link, most often. Kept rather than dropped so
    /// the report can say what became of it.
    Foreign { label: String, target: String },
}

/// A whole documentation tree, in no particular tool's terms.
#[derive(Debug, Clone, Default)]
pub struct DocTree {
    /// The site's name, where the source knew one.
    pub site_name: Option<String>,
    pub description: Option<String>,
    /// The absolute path the pages were read from.
    pub root: Utf8PathBuf,
    pub nav: Vec<Entry>,
    /// Pages found on disk that the navigation never mentioned.
    ///
    /// Kept separately rather than appended: a tool that hides a page from its
    /// own navigation may have meant to, and silently promoting it into the
    /// sidebar would publish something the author chose not to show.
    pub orphans: Vec<Utf8PathBuf>,
}

impl DocTree {
    /// Every page in the tree, navigation order first.
    pub fn pages(&self) -> Vec<&Utf8PathBuf> {
        fn walk<'a>(entries: &'a [Entry], out: &mut Vec<&'a Utf8PathBuf>) {
            for entry in entries {
                match entry {
                    Entry::Page { source, .. } => out.push(source),
                    Entry::Group { items, .. } => walk(items, out),
                    Entry::Foreign { .. } => {}
                }
            }
        }

        let mut out = Vec::new();
        walk(&self.nav, &mut out);
        out
    }
}
