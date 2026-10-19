//! Export into a sibling staging directory, then replace only an empty or
//! previously owned Zellij output. Failed builds leave the previous site intact.

use std::fs;

use crate::{Error, Result, files};
use camino::{Utf8Path, Utf8PathBuf};

const MARKER: &str = ".zellij-output";
const SIGNATURE: &str = "Zellij static export v1\n";

pub struct StaticOutput {
    target: Utf8PathBuf,
    stage: tempfile::TempDir,
    pub directory: Utf8PathBuf,
}

fn validate_output(path: &Utf8Path) -> Result<()> {
    if !path.exists() {
        return Ok(());
    }
    if !path.is_dir() {
        return Err(files::unsafe_path(path, "build output must be a directory"));
    }
    if fs::read_dir(path)?.next().is_none() {
        return Ok(());
    }
    let marker = path.join(MARKER);
    let owned = fs::symlink_metadata(&marker)
        .is_ok_and(|meta| meta.is_file() && !meta.file_type().is_symlink())
        && fs::read_to_string(&marker).is_ok_and(|body| body == SIGNATURE);
    if !owned || path.join(".git").exists() {
        return Err(files::unsafe_path(
            path,
            "nonempty directory is not a Zellij export; choose a new or empty --out directory",
        ));
    }
    Ok(())
}

impl StaticOutput {
    pub fn prepare(content: &Utf8Path, target: &Utf8Path) -> Result<Self> {
        files::disjoint(content, target)?;
        let target = files::absolute_path(target)?;
        validate_output(&target)?;
        let parent = target
            .parent()
            .ok_or_else(|| files::unsafe_path(&target, "no parent directory"))?;
        fs::create_dir_all(parent)?;
        // Beside the requested output, so the runtime shares the same host
        // filesystem path and rename stays on the same filesystem.
        let stage = tempfile::Builder::new()
            .prefix(".zel-build-")
            .tempdir_in(parent)?;
        let directory = Utf8Path::from_path(stage.path())
            .ok_or_else(|| files::unsafe_path(parent, "staging path is not UTF-8"))?
            .join("site");
        fs::create_dir(&directory)?;
        Ok(Self {
            target,
            stage,
            directory,
        })
    }

    pub fn ensure_nonempty(&self) -> Result<()> {
        // Detect unshared output mounts before touching the previous build.
        if fs::read_dir(&self.directory)?.next().is_none() {
            return Err(Error::MountEmpty {
                path: self.target.clone(),
            });
        }
        Ok(())
    }

    pub fn publish(self) -> Result<()> {
        self.ensure_nonempty()?;
        files::write_files(
            &self.directory,
            &[(Utf8Path::new(MARKER), SIGNATURE)],
            false,
        )?;
        validate_output(&self.target)?;
        let previous = self.stage.path().join("previous");
        let had_previous = self.target.exists();
        if had_previous {
            fs::rename(&self.target, &previous)?;
        }
        if let Err(error) = fs::rename(&self.directory, &self.target) {
            if had_previous && let Err(restore_error) = fs::rename(&previous, &self.target) {
                // Never let TempDir cleanup delete the only surviving copy.
                let retained = self.stage.keep();
                return Err(files::unsafe_path(
                    &self.target,
                    format!(
                        "could not publish ({error}) or restore ({restore_error}); previous export preserved at {}",
                        retained.join("previous").display()
                    ),
                ));
            }
            return Err(error.into());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn refuses_overlap_and_unowned_output_without_writing() {
        let temp = tempfile::tempdir().unwrap();
        let root = Utf8Path::from_path(temp.path()).unwrap();
        let content = root.join("content");
        fs::create_dir(&content).unwrap();
        fs::write(content.join("site.yaml"), "authored").unwrap();
        for out in [root.to_owned(), content.clone(), content.join("dist")] {
            assert!(StaticOutput::prepare(&content, &out).is_err());
        }
        let out = root.join("other-project");
        fs::create_dir(&out).unwrap();
        fs::write(out.join("notes.md"), "authored too").unwrap();
        assert!(StaticOutput::prepare(&content, &out).is_err());
        assert_eq!(
            fs::read_to_string(out.join("notes.md")).unwrap(),
            "authored too"
        );
        assert_eq!(
            fs::read_to_string(content.join("site.yaml")).unwrap(),
            "authored"
        );
    }

    #[test]
    fn publishes_complete_exports_and_preserves_old_export_on_failure() {
        let temp = tempfile::tempdir().unwrap();
        let root = Utf8Path::from_path(temp.path()).unwrap();
        let content = root.join("content");
        fs::create_dir(&content).unwrap();
        let out = root.join("dist");
        let first = StaticOutput::prepare(&content, &out).unwrap();
        fs::write(first.directory.join("index.html"), "old").unwrap();
        fs::write(first.directory.join("obsolete.html"), "old page").unwrap();
        first.publish().unwrap();

        // An empty/unshared mount and an aborted build both retain the site.
        assert!(
            StaticOutput::prepare(&content, &out)
                .unwrap()
                .publish()
                .is_err()
        );
        drop(StaticOutput::prepare(&content, &out).unwrap());
        assert_eq!(fs::read_to_string(out.join("index.html")).unwrap(), "old");

        let next = StaticOutput::prepare(&content, &out).unwrap();
        fs::write(next.directory.join("index.html"), "new").unwrap();
        fs::create_dir(next.directory.join(".well-known")).unwrap();
        fs::write(next.directory.join(".well-known/example"), "hidden asset").unwrap();
        next.publish().unwrap();
        assert_eq!(fs::read_to_string(out.join("index.html")).unwrap(), "new");
        assert!(!out.join("obsolete.html").exists());
        assert!(out.join(".well-known/example").exists());
        assert!(out.join(MARKER).is_file());
        assert!(!fs::read_dir(root).unwrap().any(|entry| {
            entry
                .unwrap()
                .file_name()
                .to_string_lossy()
                .starts_with(".zel-build-")
        }));
    }

    #[test]
    fn rechecks_destination_before_replacing_it() {
        let temp = tempfile::tempdir().unwrap();
        let root = Utf8Path::from_path(temp.path()).unwrap();
        let content = root.join("content");
        fs::create_dir(&content).unwrap();
        let out = root.join("dist");
        let build = StaticOutput::prepare(&content, &out).unwrap();
        fs::write(build.directory.join("index.html"), "new").unwrap();
        fs::create_dir(&out).unwrap();
        fs::write(out.join("notes.md"), "added during build").unwrap();
        assert!(build.publish().is_err());
        assert_eq!(
            fs::read_to_string(out.join("notes.md")).unwrap(),
            "added during build"
        );
    }
}
