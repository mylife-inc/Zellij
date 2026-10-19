// An isolated container protocol and future-release executable for native CI.
use std::{env, fs};
fn main() {
    let args: Vec<_> = env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("--version") => println!("zel 999.0.0"),
        Some("info") => {
            if env::var_os("ZEL_FIXTURE_UNAVAILABLE").is_some() { std::process::exit(1); }
        }
        Some("pull") => {
            assert_eq!(args.len(), 2);
            fs::write(env::var_os("ZEL_FIXTURE_LOG").unwrap(), &args[1]).unwrap();
            std::process::exit(env::var("ZEL_FIXTURE_EXIT").unwrap_or_default().parse().unwrap_or(0));
        }
        _ => std::process::exit(96),
    }
}
