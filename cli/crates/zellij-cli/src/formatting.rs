//! Native YAML formatting. Parse and prepare the entire set before any write.
use zellij_core::site::SiteInput;
mod syntax;

use camino::{Utf8Path, Utf8PathBuf};
use std::{fs, io::Write};
use zellij_core::{
    Error, Result, files,
    site::{Renderer, Site, is_link},
};

pub fn run(dir: Option<&SiteInput>, check: bool) -> Result<()> {
    let site = Site::load(crate::select_site(dir)?)?;
    let paths = inputs(&site)?;
    let mut changes = Vec::new();
    for path in &paths {
        let original = fs::read_to_string(path)?;
        let formatted = syntax::format(&original)
            .map_err(|e| Error::InvalidArgument(format!("{path}: {e}; no files were changed")))?;
        if formatted != original {
            changes.push((path.clone(), original, formatted));
        }
    }
    if check {
        for (path, _, _) in &changes {
            println!("Would format {path}");
        }
        if !changes.is_empty() {
            return Err(Error::FormattingRequired);
        }
    } else {
        // Stage beside each target for atomic replacement on the same volume.
        // All syntax/semantic checks complete before staging or publication.
        let mut staged = Vec::new();
        for (path, _, formatted) in &changes {
            let mut file = tempfile::NamedTempFile::new_in(path.parent().unwrap())?;
            file.as_file()
                .set_permissions(fs::metadata(path)?.permissions())?;
            file.write_all(formatted.as_bytes())?;
            file.flush()?;
            // Keep the paths, not hundreds of open handles, until publication.
            staged.push(file.into_temp_path());
        }
        // Refuse concurrent edits/replaced links before publishing the first file.
        for (path, original, _) in &changes {
            let relative = path
                .strip_prefix(&site.root)
                .map_err(|e| Error::InvalidArgument(e.to_string()))?;
            // The write-plan validator uses portable separators. Split actual
            // OS components so Windows paths work without treating a Unix
            // filename containing a backslash as a different destination.
            let relative = relative.iter().collect::<Vec<_>>().join("/");
            files::contained_path(&site.root, Utf8Path::new(&relative))?;
            if fs::read_to_string(path)? != *original {
                return Err(files::unsafe_path(
                    path,
                    "file changed during formatting; retry after saving",
                ));
            }
        }
        for ((path, _, _), file) in changes.iter().zip(staged) {
            file.persist(path).map_err(|e| e.error)?;
            println!("Formatted {path}");
        }
    }
    println!(
        "{} YAML files checked; {} {}.",
        paths.len(),
        changes.len(),
        if check {
            "need formatting"
        } else {
            "formatted"
        }
    );
    Ok(())
}

fn inputs(site: &Site) -> Result<Vec<Utf8PathBuf>> {
    let mut result = Vec::new();
    let config = site.config_path.clone();
    if config.exists() {
        result.push(config);
    }
    if site.renderer == Renderer::Portable {
        return Ok(result);
    }
    let content = if site.configured {
        site.content.clone()
    } else {
        crate::resolve_content(&site.input)?
    };
    let output = files::absolute_path(&site.output)?;
    let walker = walkdir::WalkDir::new(&content)
        .follow_links(false)
        .sort_by_file_name()
        .into_iter();
    for entry in walker.filter_entry(|entry| {
        if entry.depth() == 0 {
            return true;
        }
        let name = entry.file_name().to_string_lossy();
        !name.starts_with('.')
            && !matches!(name.as_ref(), "node_modules" | "assets")
            && entry.path() != output.as_std_path()
            && !(entry.file_type().is_dir() && entry.path().join(".zellij-output").is_file())
    }) {
        let entry = entry.map_err(|e| files::unsafe_path(&content, e.to_string()))?;
        let path = Utf8Path::from_path(entry.path())
            .ok_or_else(|| files::unsafe_path(&content, "non-UTF-8 filename"))?;
        let metadata = fs::symlink_metadata(path)?;
        if is_link(&metadata) {
            return Err(files::unsafe_path(
                path,
                "formatting does not follow symlinks or junctions",
            ));
        }
        if metadata.is_dir() {
            continue;
        }
        if !metadata.is_file() {
            return Err(files::unsafe_path(path, "expected a regular file"));
        }
        if matches!(path.extension(), Some("yaml" | "yml")) {
            result.push(path.to_owned());
        }
    }
    result.sort();
    result.dedup();
    Ok(result)
}
