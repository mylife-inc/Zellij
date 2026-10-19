// A real executable browser substitute: prove the supplied URL responds before
// recording launch. No GUI, shell parsing, Node, or external tools are needed.
use std::{
    env, fs,
    io::{Read, Write},
    net::TcpStream,
    time::Duration,
};
fn main() {
    if env::var_os("ZELLIJ_PREVIEW_FAIL").is_some() {
        std::process::exit(9);
    }
    let args: Vec<_> = env::args().skip(1).collect();
    assert_eq!(args.len(), 1);
    let url = &args[0];
    let rest = url.strip_prefix("http://").unwrap();
    let (host, path) = rest.split_once('/').unwrap();
    let mut stream = TcpStream::connect(host).unwrap();
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .unwrap();
    write!(
        stream,
        "GET /{path} HTTP/1.1\r\nHost: {host}\r\nConnection: close\r\n\r\n"
    )
    .unwrap();
    let mut response = String::new();
    stream.read_to_string(&mut response).unwrap();
    assert!(response.starts_with("HTTP/1.1 200"), "{response}");
    let expected = env::var("ZELLIJ_PREVIEW_EXPECT").unwrap_or_else(|_| "Preview handbook".into());
    assert!(response.contains(&expected));
    fs::write(env::var_os("ZELLIJ_PREVIEW_RECORD").unwrap(), url).unwrap();
}
