// Native runtime argument capture. Real container behavior is tested separately.
use std::{env, fs, io::{Read, Write}, path::PathBuf};
fn main() {
    let args: Vec<_> = env::args().skip(1).collect();
    let root = PathBuf::from(env::var_os("ZELLIJ_BUILD_FIXTURE").unwrap());
    writeln!(fs::OpenOptions::new().create(true).append(true).open(root.join("calls")).unwrap(), "{}", args.join("\0")).unwrap();
    let normalized = if args.first().map(String::as_str) == Some("buildx") { &args[1..] } else { &args[..] };
    match normalized.first().map(String::as_str) {
        Some("info") => { fs::write(root.join("probed"), "yes").unwrap(); return; }
        Some("version" | "--version") => {
            if args.first().map(String::as_str) == Some("buildx") && env::var("ZELLIJ_NO_BUILDX").as_deref() == Ok("yes") { std::process::exit(1); }
            println!("27.0.0"); return;
        }
        Some("image") => { println!("sha256:0123456789abcdef"); return; }
        Some("build") => {
            fs::write(root.join("build-args"), args.join("\0")).unwrap();
            let mut recipe = String::new();
            std::io::stdin().read_to_string(&mut recipe).unwrap();
            fs::write(root.join("recipe"), recipe).unwrap();
            if env::var("ZELLIJ_BUILD_FAIL").as_deref() == Ok("yes") { std::process::exit(1); }
        }
        Some("push") => { fs::write(root.join("pushed"), args.join("\n")).unwrap(); }
        Some("manifest") => {
            if normalized[1] == "push" && env::var("ZELLIJ_PUSH_FAIL").as_deref() == Ok("yes") { std::process::exit(1); }
        }
        Some("run") => {
            fs::write(root.join("args"), args.join("\n")).unwrap();
            if env::var("ZELLIJ_BUILD_FAIL").as_deref() == Ok("yes") { std::process::exit(96); }
            let output = args.iter().find_map(|arg| arg.strip_suffix(":/out")).unwrap();
            fs::write(PathBuf::from(output).join("index.html"), "<!doctype html><title>Built</title>").unwrap();
        }
        _ => panic!("unexpected runtime command: {args:?}"),
    }
}
