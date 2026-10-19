// Isolated runtime protocol fixture; never invokes an actual container engine.
use std::{env, fs, io::Write, path::Path, thread, time::Duration};
fn copy(from: &Path, to: &Path) {
    for entry in fs::read_dir(from).unwrap() {
        let entry = entry.unwrap(); let dest = to.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() { fs::create_dir_all(&dest).unwrap(); copy(&entry.path(), &dest); }
        else { fs::copy(entry.path(), dest).unwrap(); }
    }
}
fn main() {
    let args: Vec<_> = env::args().skip(1).collect();
    let root = env::var("ZEL_EJECT_FIXTURE").unwrap(); let root = Path::new(&root);
    let mut log = fs::OpenOptions::new().append(true).create(true).open(root.join("log")).unwrap();
    writeln!(log,"{}",args.join("|")).unwrap();
    match args[0].as_str() {
        "info" | "version" => println!("fixture runtime"),
        "create" => {
            assert_eq!(&args[1..5], ["--name", &args[2], "--entrypoint", "node"]);
            assert_eq!(args.last().unwrap(), "--version");
            assert_eq!(args[5],env::var("ZEL_EJECT_EXPECT_IMAGE").unwrap());
            fs::write(root.join("container"), &args[2]).unwrap();
        },
        "cp" => {
            assert!(!args[2].starts_with(r"\\?\"));
            let name = fs::read_to_string(root.join("container")).unwrap();
            assert!(args[1].starts_with(&format!("{name}:")));
            let mode = env::var("ZEL_EJECT_MODE").unwrap_or_default();
            if mode == "slow" { fs::write(root.join("copying"), "").unwrap(); thread::sleep(Duration::from_secs(30)); }
            if mode == "fail" { std::process::exit(96); }
            let source = if args[1].ends_with(":/app/.") {"app"} else {"engine"};
            copy(&root.join(source), Path::new(&args[2]));
            #[cfg(unix)]
            if source == "app" && mode == "link" {
                std::os::unix::fs::symlink(root.join("keep"), Path::new(&args[2]).join("linked")).unwrap();
            }
        },
        "rm" => { assert_eq!(args[1],"-f"); fs::remove_file(root.join("container")).unwrap(); },
        _ => panic!("unexpected runtime operation: {args:?}"),
    }
}
