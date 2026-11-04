// Native runtime protocol fixture: isolated from every real container daemon.
use std::{
    env, fs,
    io::{Read, Write},
    net::TcpListener,
    path::PathBuf,
    thread,
    time::Duration,
};
fn main() {
    let args: Vec<_> = env::args().skip(1).collect();
    let root = PathBuf::from(env::var_os("ZELLIJ_DEV_FIXTURE").unwrap());
    let mode = env::var("ZELLIJ_DEV_MODE").unwrap_or_default();
    match args.first().map(String::as_str) {
        Some("info" | "image") => return,
        Some("--version" | "version") => {
            println!("podman version 5.0.0");
            return;
        }
        Some("exec") => {
            assert_eq!(args[1], fs::read_to_string(root.join("name")).unwrap());
            if !root.join("allow-ready").exists() {
                std::process::exit(1);
            }
            print!("{}", fs::read_to_string(root.join("metadata")).unwrap());
            return;
        }
        Some("stop" | "rm") => {
            assert_eq!(
                args.last().unwrap(),
                &fs::read_to_string(root.join("name")).unwrap()
            );
            if args[0] == "stop" && mode == "stop-fails" {
                std::process::exit(1);
            }
            fs::write(root.join("stopped"), &args[0]).unwrap();
            return;
        }
        Some("run") => {}
        _ => panic!("unexpected runtime invocation: {args:?}"),
    }
    let option = |key| args.windows(2).find(|p| p[0] == key).unwrap()[1].clone();
    let name = option("--name");
    assert!(
        args.iter()
            .any(|a| a == &format!("ZELLIJ_DEV_SESSION={name}"))
    );
    assert!(args.iter().any(|a| a == "--init"));
    let binding = option("-p").strip_suffix(":3000").unwrap().to_owned();
    fs::write(root.join("args"), args.join("\n")).unwrap();
    fs::write(root.join("name"), &name).unwrap();
    assert!(
        env::temp_dir().join(&name).is_dir(),
        "session identity exists during startup"
    );
    println!("fixture runtime is starting development");
    eprintln!("fixture runtime startup diagnostic");
    if mode == "fail" {
        std::process::exit(96);
    }
    if mode == "empty-mount" {
        std::process::exit(97);
    }
    let prefix = "/products/manual";
    let metadata = format!(
        "{{\"version\":1,\"session\":\"{name}\",\"basePath\":\"{prefix}\",\"revision\":\"first\"}}"
    );
    fs::write(root.join("metadata"), &metadata).unwrap();
    let listener = TcpListener::bind(binding).unwrap();
    listener.set_nonblocking(true).unwrap();
    fs::write(root.join("listening"), "ready").unwrap();
    while !root.join("stopped").exists() {
        if let Ok((mut socket, _)) = listener.accept() {
            // Windows accepted sockets can inherit the listener's nonblocking
            // mode. Consume the complete request headers before closing, or a
            // partial read can reset the client's connection instead of sending
            // a usable response.
            socket.set_nonblocking(false).unwrap();
            socket
                .set_read_timeout(Some(Duration::from_secs(1)))
                .unwrap();
            let mut request = Vec::new();
            let mut chunk = [0; 4096];
            while !request.windows(4).any(|part| part == b"\r\n\r\n") && request.len() < 16384 {
                match socket.read(&mut chunk) {
                    Ok(0) | Err(_) => break,
                    Ok(count) => request.extend_from_slice(&chunk[..count]),
                }
            }
            if !request.windows(4).any(|part| part == b"\r\n\r\n") { continue; }
            let request = String::from_utf8_lossy(&request);
            let (code, body) =
                if request.starts_with(&format!("GET {prefix}/_zellij/dev-revision.json ")) {
                    let body = if mode == "wrong-session" {
                        metadata.replace(&name, "another-session")
                    } else {
                        metadata.clone()
                    };
                    (200, body)
                } else {
                    (404, String::new())
                };
            let _ = write!(
                socket,
                "HTTP/1.1 {code} Fixture\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            );
        }
        thread::sleep(Duration::from_millis(10));
    }
}
