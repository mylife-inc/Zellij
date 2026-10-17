use camino::Utf8Path;
use std::fs;
use zellij_docs::watch::Snapshot;

#[test]
fn snapshots_detect_bytes_additions_deletions_and_ignore_metadata_only_changes() {
    let temp = tempfile::tempdir().unwrap();
    let root = Utf8Path::from_path(temp.path()).unwrap();
    let file = root.join("README.md");
    fs::write(&file, "first").unwrap();
    let time = fs::metadata(&file).unwrap().modified().unwrap();
    let initial = Snapshot::read(root).unwrap();
    fs::write(&file, "other").unwrap();
    fs::File::options()
        .write(true)
        .open(&file)
        .unwrap()
        .set_modified(time)
        .unwrap();
    assert_ne!(Snapshot::read(root).unwrap(), initial);
    fs::write(&file, "first").unwrap();
    assert_eq!(Snapshot::read(root).unwrap(), initial);
    fs::create_dir(root.join("assets")).unwrap();
    fs::write(root.join("assets/image.bin"), [0, 255]).unwrap();
    assert_ne!(Snapshot::read(root).unwrap(), initial);
    fs::remove_file(root.join("assets/image.bin")).unwrap();
    assert_eq!(Snapshot::read(root).unwrap(), initial);
    fs::remove_file(&file).unwrap();
    assert_ne!(Snapshot::read(root).unwrap(), initial);
}

#[test]
fn hidden_and_dependency_trees_are_ignored_but_missing_roots_fail() {
    let temp = tempfile::tempdir().unwrap();
    let root = Utf8Path::from_path(temp.path()).unwrap();
    fs::write(root.join("README.md"), "# Home").unwrap();
    let initial = Snapshot::read(root).unwrap();
    for name in [".git", "node_modules"] {
        fs::create_dir(root.join(name)).unwrap();
        fs::write(root.join(name).join("nested.md"), "ignored").unwrap();
    }
    fs::write(root.join(".edit.tmp"), "ignored").unwrap();
    assert_eq!(Snapshot::read(root).unwrap(), initial);
    assert!(Snapshot::read(&root.join("missing")).is_err());
}

#[cfg(unix)]
#[test]
fn snapshot_refuses_nested_links_without_reading_their_targets() {
    let temp = tempfile::tempdir().unwrap();
    let root = Utf8Path::from_path(temp.path()).unwrap();
    std::os::unix::fs::symlink("/nonexistent-target", root.join("linked")).unwrap();
    assert!(
        Snapshot::read(root)
            .unwrap_err()
            .to_string()
            .contains("symbolic links")
    );
    let other = tempfile::tempdir().unwrap();
    let alias = root.join("root-alias");
    std::os::unix::fs::symlink(other.path(), &alias).unwrap();
    assert!(
        Snapshot::read(&alias)
            .unwrap_err()
            .to_string()
            .contains("symbolic links")
    );
}
