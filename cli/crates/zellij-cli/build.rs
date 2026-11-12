// Bundle the authored catalog so first use and offline creation need no network.
use std::{env, fs, path::Path};
fn visit(root: &Path, dir: &Path, output: &mut String) {
    let mut entries = fs::read_dir(dir)
        .unwrap()
        .map(|e| e.unwrap())
        .collect::<Vec<_>>();
    entries.sort_by_key(|e| e.file_name());
    for entry in entries {
        let path = entry.path();
        let kind = entry.file_type().unwrap();
        assert!(
            !kind.is_symlink(),
            "template source must not contain symlinks"
        );
        if kind.is_dir() {
            visit(root, &path, output);
        } else {
            assert!(kind.is_file(), "template source must contain regular files");
            let name = path
                .strip_prefix(root)
                .unwrap()
                .to_str()
                .unwrap()
                .replace('\\', "/");
            output.push_str(&format!(
                "({name:?}, include_bytes!({:?})),\n",
                path.to_str().unwrap()
            ));
        }
    }
}
fn main() {
    println!(
        "cargo:rustc-env=ZEL_TARGET={}",
        env::var("TARGET").unwrap_or_else(|_| "unknown".into())
    );
    println!("cargo:rerun-if-changed=build.rs");
    let root = Path::new(&env::var("CARGO_MANIFEST_DIR").unwrap())
        .join("../../../templates")
        .canonicalize()
        .unwrap();
    println!("cargo:rerun-if-changed={}", root.display());
    let mut output = String::from("const BUNDLED: &[(&str, &[u8])] = &[\n");
    output.push_str(&format!(
        "(\"catalog.yaml\", include_bytes!({:?})),\n",
        root.join("catalog.yaml").to_str().unwrap()
    ));
    let mut directories = fs::read_dir(&root)
        .unwrap()
        .map(|e| e.unwrap())
        .filter(|e| e.file_type().unwrap().is_dir())
        .collect::<Vec<_>>();
    directories.sort_by_key(|e| e.file_name());
    for entry in directories {
        let dir = entry.path();
        let manifest = dir.join("template.yaml");
        assert!(
            fs::symlink_metadata(&manifest).unwrap().is_file(),
            "template.yaml must be a regular file"
        );
        let name = manifest
            .strip_prefix(&root)
            .unwrap()
            .to_str()
            .unwrap()
            .replace(std::path::MAIN_SEPARATOR, "/");
        output.push_str(&format!(
            "({name:?}, include_bytes!({:?})),\n",
            manifest.to_str().unwrap()
        ));
        assert!(
            fs::symlink_metadata(dir.join("files")).unwrap().is_dir(),
            "template files must be a directory"
        );
        visit(&root, &dir.join("files"), &mut output);
    }
    output.push_str("];\n");
    fs::write(
        Path::new(&env::var("OUT_DIR").unwrap()).join("templates.rs"),
        output,
    )
    .unwrap();
}
