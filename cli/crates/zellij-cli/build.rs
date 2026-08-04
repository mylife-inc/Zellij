// The target triple, so `zel doctor` can say which build somebody is running.
// Cargo tells a build script its target and tells the crate nothing, so this
// is the only way to get it in without a dependency.
fn main() {
    println!(
        "cargo:rustc-env=ZEL_TARGET={}",
        std::env::var("TARGET").unwrap_or_else(|_| "unknown".into())
    );
    println!("cargo:rerun-if-changed=build.rs");
}
