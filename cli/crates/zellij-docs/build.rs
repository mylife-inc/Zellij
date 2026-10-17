// Embed the already pinned, licensed engine font snapshot. Cargo builds remain
// independent of Node/network access; the engine verifier checks provenance in CI.
use std::{env, fs, path::PathBuf};

fn main() {
    let root = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap())
        .join("../../../engine/packages/zellij/src/themes/fonts")
        .canonicalize()
        .expect("engine font snapshot must be present in the source checkout");
    println!("cargo::rerun-if-changed={}", root.display());
    let mut paths = vec![root.join("fonts.css"), root.join("licenses.txt")];
    paths.extend(
        fs::read_dir(root.join("files"))
            .unwrap()
            .map(|e| e.unwrap().path()),
    );
    paths.sort();
    let mut code = String::from("pub static FONTS: &[(&str, &[u8])] = &[\n");
    for path in paths {
        assert!(path.is_file());
        let name = path
            .strip_prefix(&root)
            .unwrap()
            .to_str()
            .unwrap()
            .replace('\\', "/");
        code.push_str(&format!(
            "({name:?}, include_bytes!({:?})),\n",
            path.to_str().unwrap()
        ));
    }
    code.push_str("];\n");
    fs::write(
        PathBuf::from(env::var_os("OUT_DIR").unwrap()).join("fonts.rs"),
        code,
    )
    .unwrap();
}
