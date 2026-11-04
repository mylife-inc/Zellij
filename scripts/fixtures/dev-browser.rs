// Record a browser launch without opening a GUI. Used by native and container CI.
use std::{env, fs::OpenOptions, io::Write};
fn main() {
    let args: Vec<_> = env::args().skip(1).collect();
    assert_eq!(
        args.len(),
        1,
        "the launcher receives one URL, not shell syntax"
    );
    writeln!(
        OpenOptions::new()
            .create(true)
            .append(true)
            .open(env::var_os("ZELLIJ_DEV_BROWSER_LOG").unwrap())
            .unwrap(),
        "{}",
        args[0]
    )
    .unwrap();
}
