// A deterministic runtime protocol fixture, not a replacement for real-container CI.
use std::{env, fs, io::Write, path::PathBuf, thread, time::Duration};
fn main() {
    let args: Vec<_> = env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("info") => return,
        Some("version") => {
            println!("27.0.0");
            return;
        }
        Some("image") => {
            println!("sha256:0123456789abcdef");
            return;
        }
        Some("run") => {}
        _ => panic!("unexpected runtime command: {args:?}"),
    }
    let mounts: Vec<_> = args
        .windows(2)
        .filter(|p| p[0] == "-v")
        .map(|p| p[1].as_str())
        .collect();
    let content = PathBuf::from(
        mounts
            .iter()
            .find_map(|p| p.strip_suffix(":/app/content:ro"))
            .unwrap(),
    );
    let output = PathBuf::from(mounts.iter().find_map(|p| p.strip_suffix(":/out")).unwrap());
    let control = fs::read_to_string(content.join("preview.txt")).unwrap();
    let mut lines = control.lines();
    let title = lines.next().unwrap();
    let prefix = lines.next().unwrap();
    let mode = lines.next().unwrap();
    if let Some(log) = env::var_os("ZELLIJ_PREVIEW_ENGINE_LOG") {
        writeln!(
            fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(log)
                .unwrap(),
            "{title}"
        )
        .unwrap();
    }
    if mode == "slow" {
        thread::sleep(Duration::from_secs(2));
    }
    if mode == "fail" {
        std::process::exit(96);
    }
    if mode == "empty" {
        return;
    }
    if mode != "missing-index" {
        fs::write(output.join("index.html"), format!("<!doctype html><html><head><title>{title}</title></head><body><h1>{title}</h1><a href=\"{prefix}/guide\">Guide</a></body></html>")).unwrap();
        fs::write(output.join("guide.html"), "<!doctype html><html><head><title>Guide</title></head><body><h1>Guide</h1></body></html>").unwrap();
    }
    if mode != "missing-metadata" {
        fs::write(
            output.join(".zellij-deployment.json"),
            format!("\"{prefix}\""),
        )
        .unwrap();
    }
}
