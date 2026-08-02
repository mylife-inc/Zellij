use camino::{Utf8Path, Utf8PathBuf};

use crate::error::{Error, Result};

/// The files that mark a directory as a Zellij content folder.
///
/// A guide folder alone is enough to build a site, so the absence of
/// `site.yaml` cannot be the test — the loader treats every file as optional
/// and so must this.
const MARKERS: &[&str] = &["site.yaml", "pages.yaml", "menu.yaml", "sections"];

/// A content folder on disk, and where it was found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Project {
    pub content: Utf8PathBuf,
}

impl Project {
    /// Looks for a content folder at `start`, then in the conventional places
    /// beneath it.
    ///
    /// Searching down rather than up: a documentation folder usually sits
    /// beside the source it documents, and walking upwards from a deep
    /// directory would find a sibling project's content as readily as this
    /// one's.
    pub fn discover(start: &Utf8Path) -> Result<Self> {
        if Self::looks_like_content(start) {
            return Ok(Self {
                content: start.to_owned(),
            });
        }

        for candidate in ["content", "docs", "documentation"] {
            let path = start.join(candidate);
            if Self::looks_like_content(&path) {
                return Ok(Self { content: path });
            }
        }

        Err(Error::NoContentFolder {
            path: start.to_owned(),
        })
    }

    fn looks_like_content(path: &Utf8Path) -> bool {
        if !path.is_dir() {
            return false;
        }

        if MARKERS.iter().any(|marker| path.join(marker).exists()) {
            return true;
        }

        // A folder holding nothing but one guide folder is a complete site, and
        // a guide folder is anything with a `_sidebar.yaml` in it.
        std::fs::read_dir(path)
            .into_iter()
            .flatten()
            .flatten()
            .any(|entry| entry.path().join("_sidebar.yaml").exists())
    }
}
