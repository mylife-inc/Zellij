//! Shared boundaries for commands that read imported paths or write content.
//!
//! Check the complete plan before creating anything. Paths from content must
//! be portable relative paths; existing symlinks are never followed beneath
//! a root. Explicit output roots are resolved before overlap checks.

use std::{collections::BTreeSet, fs, io::Write};

use camino::{Utf8Path, Utf8PathBuf};

use crate::{Error, Result};

pub fn unsafe_path(path: &Utf8Path, reason: impl Into<String>) -> Error {
    Error::UnsafePath {
        path: path.to_owned(),
        reason: reason.into(),
    }
}

pub fn relative_path(path: &Utf8Path) -> Result<()> {
    if path.as_str().is_empty()
        || path.as_str().contains(['\\', ':', '\0'])
        || path.as_str().split('/').any(|part| {
            part.is_empty() || part == "." || part == ".." || part.eq_ignore_ascii_case(".git")
        })
    {
        return Err(unsafe_path(
            path,
            "expected a relative path without parent segments, drive prefixes, or Git metadata",
        ));
    }
    Ok(())
}

/// Canonicalize the existing prefix, retaining a not-yet-created suffix.
/// Resolving ancestors first catches aliases such as `link-to-source/output`.
pub fn absolute_path(path: &Utf8Path) -> Result<Utf8PathBuf> {
    let absolute = if path.is_absolute() {
        path.to_owned()
    } else {
        Utf8PathBuf::from_path_buf(std::env::current_dir()?)
            .map_err(|_| unsafe_path(path, "working directory is not UTF-8"))?
            .join(path)
    };
    match fs::symlink_metadata(&absolute) {
        Ok(_) => Ok(absolute.canonicalize_utf8()?),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            let parent = absolute
                .parent()
                .ok_or_else(|| unsafe_path(path, "no parent directory"))?;
            let name = absolute
                .file_name()
                .ok_or_else(|| unsafe_path(path, "cannot resolve this path"))?;
            Ok(absolute_path(parent)?.join(name))
        }
        Err(error) => Err(error.into()),
    }
}

/// Neither root may contain the other, even through a symlink alias.
pub fn disjoint(source: &Utf8Path, output: &Utf8Path) -> Result<()> {
    let source = absolute_path(source)?;
    let output = absolute_path(output)?;
    if source.starts_with(&output) || output.starts_with(&source) {
        return Err(unsafe_path(
            &output,
            format!("source and output must not overlap ({source})"),
        ));
    }
    Ok(())
}

/// Return a contained path, rejecting links and non-directory ancestors.
/// A missing leaf is allowed so importers can still report unmapped documents.
pub fn contained_path(root: &Utf8Path, relative: &Utf8Path) -> Result<Utf8PathBuf> {
    relative_path(relative)?;
    let mut path = absolute_path(root)?;
    if path.exists() && !path.is_dir() {
        return Err(unsafe_path(&path, "expected a directory"));
    }
    let parts: Vec<_> = relative.as_str().split('/').collect();
    for (index, part) in parts.iter().enumerate() {
        path.push(part);
        match fs::symlink_metadata(&path) {
            Ok(meta) if meta.file_type().is_symlink() => {
                return Err(unsafe_path(
                    &path,
                    "symbolic links are not allowed beneath the content root",
                ));
            }
            Ok(meta) if index + 1 < parts.len() && !meta.is_dir() => {
                return Err(unsafe_path(&path, "a parent component is not a directory"));
            }
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }
    }
    Ok(path)
}

pub fn empty_directory(root: &Utf8Path) -> Result<()> {
    let root = absolute_path(root)?;
    if root.exists() && fs::read_dir(&root)?.next().is_some() {
        return Err(unsafe_path(
            &root,
            "destination must be empty; choose a new output directory",
        ));
    }
    Ok(())
}

/// Validate every target, including duplicates and file/directory conflicts.
pub fn preflight(root: &Utf8Path, paths: &[&Utf8Path], overwrite: bool) -> Result<()> {
    let mut names = BTreeSet::new();
    for path in paths {
        let target = contained_path(root, path)?;
        let key = path.as_str().to_lowercase();
        if !names.insert(key) {
            return Err(unsafe_path(
                path,
                "duplicate output path (including case-only differences)",
            ));
        }
        if let Ok(meta) = fs::symlink_metadata(&target) {
            if !meta.is_file() || !overwrite {
                return Err(Error::FileExists { path: target });
            }
        }
    }
    for path in paths {
        for parent in path.ancestors().skip(1).filter(|p| !p.as_str().is_empty()) {
            if names.contains(&parent.as_str().to_lowercase()) {
                return Err(unsafe_path(
                    path,
                    "another output file is this path's parent",
                ));
            }
        }
    }
    Ok(())
}

pub fn write_files(root: &Utf8Path, files: &[(&Utf8Path, &str)], overwrite: bool) -> Result<()> {
    let paths: Vec<_> = files.iter().map(|(path, _)| *path).collect();
    preflight(root, &paths, overwrite)?;
    let mut staged = Vec::with_capacity(files.len());
    for (relative, body) in files {
        let target = contained_path(root, relative)?;
        let parent = target.parent().expect("contained file has a parent");
        fs::create_dir_all(parent)?;
        let mut temporary = tempfile::NamedTempFile::new_in(parent)?;
        temporary.write_all(body.as_bytes())?;
        // Keep the temporary path, not an open descriptor for every page.
        // Large documentation imports can exceed the process's file limit.
        staged.push((temporary.into_temp_path(), target));
    }
    for (temporary, target) in staged {
        if overwrite {
            // Replacing a directory entry also avoids modifying a hard-linked
            // file elsewhere, unlike truncating the existing inode.
            temporary.persist(&target).map_err(|error| error.error)?;
        } else {
            temporary
                .persist_noclobber(&target)
                .map_err(|error| error.error)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_nonportable_and_escaping_paths() {
        for path in [
            "",
            "../file",
            "/file",
            "a/../b",
            "C:/file",
            "C:\\file",
            "a\\b",
            ".git/config",
            "a/.GIT/config",
        ] {
            assert!(relative_path(Utf8Path::new(path)).is_err(), "{path}");
        }
        relative_path(Utf8Path::new("guide/nested/file.mdx")).unwrap();
    }

    #[test]
    fn preflight_rejects_conflicts_before_creating_root() {
        let temp = tempfile::tempdir().unwrap();
        let root = Utf8Path::from_path(temp.path()).unwrap().join("output");
        for paths in [["same", "same"], ["Same", "same"], ["dir", "dir/file"]] {
            let files = paths.map(|p| (Utf8Path::new(p), "body"));
            assert!(write_files(&root, &files, false).is_err());
            assert!(!root.exists());
        }
    }

    #[cfg(unix)]
    #[test]
    fn rejects_dangling_links_and_overlap_through_aliases() {
        let temp = tempfile::tempdir().unwrap();
        let root = Utf8Path::from_path(temp.path()).unwrap();
        fs::create_dir(root.join("source")).unwrap();
        std::os::unix::fs::symlink(root.join("source"), root.join("alias")).unwrap();
        assert!(disjoint(&root.join("source"), &root.join("alias/output")).is_err());
        std::os::unix::fs::symlink(root.join("missing"), root.join("dangling")).unwrap();
        assert!(contained_path(root, Utf8Path::new("dangling")).is_err());
    }
}
