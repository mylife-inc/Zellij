use camino::Utf8Path;
use std::{
    fs,
    io::{Read, Write},
    net::TcpStream,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::Duration,
};
use zellij_docs::server::StaticServer;

struct Running {
    stop: Arc<AtomicBool>,
    task: Option<thread::JoinHandle<()>>,
    url: String,
}
impl Running {
    fn start(root: &std::path::Path, prefix: &str) -> Self {
        let server = StaticServer::bind(Utf8Path::from_path(root).unwrap(), 0, prefix).unwrap();
        let url = server.url().to_owned();
        let stop = Arc::new(AtomicBool::new(false));
        let signal = stop.clone();
        let task = thread::spawn(move || server.run(&signal).unwrap());
        Self {
            stop,
            task: Some(task),
            url,
        }
    }
    fn address(&self) -> &str {
        self.url
            .trim_start_matches("http://")
            .split('/')
            .next()
            .unwrap()
    }
    fn request(&self, method: &str, path: &str, host: Option<&str>) -> (u16, String, Vec<u8>) {
        let mut socket = TcpStream::connect(self.address()).unwrap();
        socket
            .set_read_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        write!(socket, "{method} {path} HTTP/1.1\r\nHost: {}\r\nConnection: close\r\nContent-Length: 0\r\n\r\n",host.unwrap_or(self.address())).unwrap();
        let mut bytes = Vec::new();
        socket.read_to_end(&mut bytes).unwrap();
        let end = bytes.windows(4).position(|s| s == b"\r\n\r\n").unwrap();
        let headers = String::from_utf8(bytes[..end].to_vec()).unwrap();
        let status = headers.split_whitespace().nth(1).unwrap().parse().unwrap();
        (status, headers.to_lowercase(), bytes[end + 4..].to_vec())
    }
}
impl Drop for Running {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        self.task.take().unwrap().join().unwrap();
    }
}
fn fixture() -> tempfile::TempDir {
    let root = tempfile::tempdir().unwrap();
    for (path, body) in [
        ("index.html", "<h1>Home</h1>"),
        ("guide/index.html", "<h1>Guide</h1>"),
        ("guide/next.html", "<h1>Next</h1>"),
        ("guide/v1.2.html", "Version"),
        ("日本/with spaces.html", "Unicode"),
        ("404.html", "Custom not found"),
        ("assets/style.css", "body{}"),
        ("assets/search.js", "window.search=[];"),
        (".secret", "private"),
        (".git/config", "private git"),
        (".well-known/example", "well known"),
    ] {
        let file = root.path().join(path);
        fs::create_dir_all(file.parent().unwrap()).unwrap();
        fs::write(file, body).unwrap();
    }
    fs::write(root.path().join("assets/font.woff2"), [0, 255, 17, 8]).unwrap();
    root
}

#[test]
fn serves_exact_bytes_pretty_routes_directories_queries_and_unicode() {
    let root = fixture();
    let server = Running::start(root.path(), "/");
    for (path, body) in [
        ("/", "<h1>Home</h1>"),
        ("/index.html?test=one", "<h1>Home</h1>"),
        ("/guide/", "<h1>Guide</h1>"),
        ("/guide/next", "<h1>Next</h1>"),
        ("/guide/v1.2", "Version"),
        ("/%E6%97%A5%E6%9C%AC/with%20spaces.html", "Unicode"),
        ("/.well-known/example", "well known"),
    ] {
        let (status, _, bytes) = server.request("GET", path, None);
        assert_eq!(status, 200, "{path}");
        assert_eq!(bytes, body.as_bytes(), "{path}");
    }
    let (status, headers, _) = server.request("GET", "/guide?x=1", None);
    assert_eq!(status, 308);
    assert!(headers.contains("location: /guide/?x=1"));
    let (status, _, bytes) = server.request("GET", "/missing", None);
    assert_eq!(status, 404);
    assert_eq!(bytes, b"Custom not found");
    // Serving rereads changed files but never rebuilds or rewrites them.
    fs::write(root.path().join("index.html"), "Replacement").unwrap();
    assert_eq!(server.request("GET", "/", None).2, b"Replacement");
}

#[test]
fn content_types_head_and_method_contracts_match_the_artifact() {
    let root = fixture();
    let server = Running::start(root.path(), "/");
    for (path, mime) in [
        ("/index.html", "text/html; charset=utf-8"),
        ("/assets/style.css", "text/css; charset=utf-8"),
        ("/assets/search.js", "javascript"),
        ("/assets/font.woff2", "font/woff2"),
    ] {
        let (status, headers, bytes) = server.request("GET", path, None);
        assert_eq!(status, 200);
        assert!(headers.contains(mime), "{headers}");
        assert!(headers.contains("cache-control: no-store"));
        assert!(headers.contains("x-content-type-options: nosniff"));
        let (status, headers, body) = server.request("HEAD", path, None);
        assert_eq!(status, 200);
        assert!(body.is_empty());
        assert!(headers.contains(&format!("content-length: {}", bytes.len())));
    }
    assert_eq!(
        server.request("GET", "/assets/font.woff2", None).2,
        [0, 255, 17, 8]
    );
    let (status, headers, _) = server.request("POST", "/", None);
    assert_eq!(status, 405);
    assert!(headers.contains("allow: get, head"));
}

#[test]
fn malformed_traversal_metadata_and_foreign_hosts_are_not_served() {
    let root = fixture();
    let server = Running::start(root.path(), "/");
    for path in [
        "/../secret",
        "/%2e%2e/secret",
        "/guide/%2E%2E/index.html",
        "/%ZZ",
        "/%FF",
        "/%00",
        "/C:/private",
        "/guide%5C..%5Csecret",
        "//index.html",
        "/guide//",
        "/.secret",
        "/.git/config",
    ] {
        let (status, _, bytes) = server.request("GET", path, None);
        assert!([400, 403].contains(&status), "{path}: {status}");
        assert!(!bytes.windows(7).any(|s| s == b"private"));
    }
    assert_eq!(server.request("GET", "/", Some("untrusted.example")).0, 403);
}

#[test]
fn prefix_mounts_do_not_expose_the_site_at_other_routes() {
    let root = fixture();
    let server = Running::start(root.path(), "/products/docs/");
    assert!(server.url.ends_with("/products/docs/"));
    assert_eq!(
        server.request("GET", "/products/docs/guide/next", None).0,
        200
    );
    assert_eq!(
        server.request("GET", "/products/docs?from=home", None).0,
        308
    );
    assert!(
        server
            .request("GET", "/products/docs", None)
            .1
            .contains("location: /products/docs/")
    );
    for path in ["/", "/products/docs-other/", "/guide/next"] {
        assert_eq!(server.request("GET", path, None).0, 404);
    }
}

#[test]
fn invalid_roots_prefixes_and_occupied_ports_fail_and_shutdown_releases_the_port() {
    let root = fixture();
    let path = Utf8Path::from_path(root.path()).unwrap();
    for prefix in ["relative", "/../bad", "/query?x", "/a//b", "/%ZZ"] {
        assert!(StaticServer::bind(path, 0, prefix).is_err());
    }
    let empty = tempfile::tempdir().unwrap();
    assert!(StaticServer::bind(Utf8Path::from_path(empty.path()).unwrap(), 0, "/").is_err());
    let server = Running::start(root.path(), "/");
    let address = server.address().to_owned();
    let port = address.rsplit(':').next().unwrap().parse().unwrap();
    let error = StaticServer::bind(path, port, "/").err().unwrap();
    assert!(error.to_string().contains("already in use"));
    drop(server);
    for _ in 0..30 {
        if TcpStream::connect(&address).is_err() {
            return;
        }
        thread::sleep(Duration::from_millis(20));
    }
    panic!("server listener remained open after shutdown");
}

#[cfg(unix)]
#[test]
fn symlinked_files_and_directories_cannot_expose_outside_content() {
    let root = fixture();
    let outside = tempfile::tempdir().unwrap();
    fs::write(outside.path().join("secret"), "private").unwrap();
    std::os::unix::fs::symlink(outside.path().join("secret"), root.path().join("leak.txt"))
        .unwrap();
    std::os::unix::fs::symlink(outside.path(), root.path().join("leak-dir")).unwrap();
    let server = Running::start(root.path(), "/");
    assert_eq!(server.request("GET", "/leak.txt", None).0, 403);
    assert_eq!(server.request("GET", "/leak-dir/secret", None).0, 403);
}
