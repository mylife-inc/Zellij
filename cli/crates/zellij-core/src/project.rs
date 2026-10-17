use camino::{Utf8Path, Utf8PathBuf};

use crate::error::{Error, Result};

/// The files that mark a directory as a Zellij content folder.
///
/// A guide folder alone is enough to build a site, so the absence of
/// `site.yaml` cannot be the test — the loader treats every file as optional
/// and so must this.
const MARKERS: &[&str] = &["site.yaml", "pages.yaml", "menu.yaml"];
const RESERVED_DIRS: &[&str] = &["sections", "examples", "assets", "public", "node_modules"];

/// A content folder on disk, and where it was found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Project {
    pub content: Utf8PathBuf,
}

impl Project {
    /// Search `content/`, the current root, then `docs/` and `documentation/`.
    /// The content-first order preserves the CLI's established default. Never
    /// walk upwards and accidentally select another project's content.
    pub fn discover(start: &Utf8Path) -> Result<Self> {
        for path in [
            start.join("content"),
            start.to_owned(),
            start.join("docs"),
            start.join("documentation"),
        ] {
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

        if MARKERS.iter().any(|marker| path.join(marker).is_file())
            || path.join("sections").is_dir()
        {
            return true;
        }

        // A folder holding nothing but one guide folder is a complete site, and
        // a guide folder is anything with a `_sidebar.yaml` in it.
        std::fs::read_dir(path)
            .into_iter()
            .flatten()
            .flatten()
            .any(|entry| {
                let name = entry.file_name();
                let name = name.to_string_lossy();
                !name.starts_with('.')
                    && !RESERVED_DIRS.contains(&name.as_ref())
                    && entry.file_type().is_ok_and(|kind| kind.is_dir())
                    && entry.path().join("_sidebar.yaml").is_file()
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> (tempfile::TempDir, Utf8PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        let root = Utf8PathBuf::from_path_buf(dir.path().to_owned()).unwrap();
        (dir, root)
    }

    fn write(root: &Utf8Path, file: &str) {
        let file = root.join(file);
        std::fs::create_dir_all(file.parent().unwrap()).unwrap();
        std::fs::write(file, "").unwrap();
    }

    #[test]
    fn discovers_every_supported_marker_in_each_conventional_location() {
        for location in ["", "content", "docs", "documentation"] {
            for marker in [
                "site.yaml",
                "pages.yaml",
                "menu.yaml",
                "sections/hero.yaml",
                "01-guide/_sidebar.yaml",
            ] {
                let (_guard, root) = fixture();
                let content = root.join(location);
                write(&content, marker);
                assert_eq!(Project::discover(&root).unwrap().content, content);
            }
        }
    }

    #[test]
    fn content_precedes_root_then_docs_then_documentation() {
        let (_guard, root) = fixture();
        for location in ["documentation", "docs", "", "content"] {
            let content = root.join(location);
            write(&content, "menu.yaml");
            assert_eq!(Project::discover(&root).unwrap().content, content);
        }
    }

    #[test]
    fn ignores_wrong_file_types_hidden_and_reserved_guides() {
        let (_guard, root) = fixture();
        std::fs::create_dir(root.join("site.yaml")).unwrap();
        std::fs::create_dir_all(root.join("guide/_sidebar.yaml")).unwrap();
        write(&root, "sections"); // A file is not the sections directory.
        for dir in [".hidden", "assets", "examples", "public", "node_modules"] {
            write(&root, &format!("{dir}/_sidebar.yaml"));
        }
        assert!(Project::discover(&root).is_err());
    }

    #[test]
    fn does_not_search_ancestors() {
        let (_guard, root) = fixture();
        write(&root, "site.yaml");
        let child = root.join("unrelated");
        std::fs::create_dir(&child).unwrap();
        assert!(Project::discover(&child).is_err());
    }
}
