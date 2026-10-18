use super::*;
use serde_json::{Value, json};
use std::{
    io::{BufRead, BufReader, Write},
    net::TcpListener,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread,
};

#[derive(Clone, Debug)]
struct Request {
    method: String,
    path: String,
    headers: BTreeMap<String, String>,
    body: Vec<u8>,
}
struct Reply {
    status: u16,
    body: Value,
    redirect: Option<String>,
}
impl From<Value> for Reply {
    fn from(body: Value) -> Self {
        Self {
            status: 200,
            body,
            redirect: None,
        }
    }
}
struct Server {
    address: String,
    requests: Arc<Mutex<Vec<Request>>>,
    stop: Arc<AtomicBool>,
    worker: Option<thread::JoinHandle<()>>,
}
impl Server {
    fn new(handler: impl Fn(&Request, usize) -> Reply + Send + 'static) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let address = format!("http://{}/api/v1/", listener.local_addr().unwrap());
        let requests = Arc::new(Mutex::new(Vec::new()));
        let stop = Arc::new(AtomicBool::new(false));
        let (record, stopped) = (requests.clone(), stop.clone());
        let worker = thread::spawn(move || {
            while !stopped.load(Ordering::SeqCst) {
                let (mut socket, _) = match listener.accept() {
                    Ok(pair) => pair,
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(2));
                        continue;
                    }
                    Err(e) => panic!("{e}"),
                };
                socket.set_nonblocking(false).unwrap();
                socket
                    .set_read_timeout(Some(Duration::from_secs(5)))
                    .unwrap();
                let mut reader = BufReader::new(socket.try_clone().unwrap());
                let mut line = String::new();
                reader.read_line(&mut line).unwrap();
                let parts: Vec<_> = line.split_whitespace().collect();
                let (method, path) = (parts[0].to_owned(), parts[1].to_owned());
                let mut headers = BTreeMap::new();
                loop {
                    let mut line = String::new();
                    reader.read_line(&mut line).unwrap();
                    if line == "\r\n" {
                        break;
                    }
                    let (key, value) = line.split_once(':').unwrap();
                    headers.insert(key.to_ascii_lowercase(), value.trim().to_owned());
                }
                let size = headers
                    .get("content-length")
                    .map(|v| v.parse::<usize>().unwrap())
                    .unwrap_or(0);
                let mut body = vec![0; size];
                reader.read_exact(&mut body).unwrap();
                let request = Request {
                    method,
                    path,
                    headers,
                    body,
                };
                let index = record.lock().unwrap().len();
                let reply = handler(&request, index);
                record.lock().unwrap().push(request);
                let body = reply.body.to_string();
                let location = reply
                    .redirect
                    .map(|v| format!("Location: {v}\r\n"))
                    .unwrap_or_default();
                // A timeout test may close the connection before this reply.
                let _ = write!(
                    socket,
                    "HTTP/1.1 {} Fixture\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n{}\r\n{}",
                    reply.status,
                    body.len(),
                    location,
                    body
                );
            }
        });
        Self {
            address,
            requests,
            stop,
            worker: Some(worker),
        }
    }
    fn client(&self) -> Client {
        Client::new(
            &self.address,
            "fixture-token",
            Duration::from_secs(10),
            Duration::from_millis(1),
        )
        .unwrap()
    }
}
impl Drop for Server {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
        let result = self.worker.take().unwrap().join();
        if !thread::panicking() {
            result.unwrap();
        }
    }
}
fn write(root: &Utf8Path, path: &str, body: impl AsRef<[u8]>) {
    let path = root.join(path);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, body).unwrap();
}
fn ready() -> Value {
    json!({"id":"deploy123", "state":"ready", "deploy_ssl_url":"https://deploy123--manual.netlify.app"})
}
fn export(root: &Utf8Path) {
    write(root, "index.html", "<h1>Generated</h1>");
}

#[test]
fn digest_upload_deduplicates_preserves_hidden_prefix_and_escapes_paths() {
    for production in [false, true] {
        let tmp = tempfile::tempdir().unwrap();
        let root = Utf8Path::from_path(tmp.path()).unwrap();
        write(root, "products/.docs/index.html", b"same bytes");
        write(root, "products/.docs/copy.html", b"same bytes");
        write(root, "products/.docs/assets/日本 space%.png", [0, 1, 255]);
        write(root, ".zellij-output", "private marker");
        let manifest = Manifest::read(root).unwrap();
        let hashes = manifest.hashes.clone();
        let expected = manifest.by_hash.clone();
        let server = Server::new(move |request, index| {
            assert_eq!(request.headers["authorization"], "Bearer fixture-token");
            assert!(request.headers["user-agent"].starts_with("zel/"));
            match index {
                0 => { assert_eq!(request.method, "POST"); assert_eq!(request.path, "/api/v1/sites/manual.netlify.app/deploys");
                    let body: Value = serde_json::from_slice(&request.body).unwrap();
                    assert_eq!(body, json!({"files":hashes, "async":true, "draft":!production}));
                    json!({"id":"deploy123","state":"preparing"}).into()
                }
                1 => json!({"id":"deploy123","state":"prepared","required":expected.keys().collect::<Vec<_>>(),"required_functions":[]}).into(),
                2 | 3 => { assert_eq!(request.method, "PUT"); assert_eq!(request.headers["content-type"], "application/octet-stream");
                    let hash = format!("{:x}", Sha1::digest(&request.body));
                    let (name, path) = &expected[&hash];
                    let mut url = Url::parse("https://fixture/api/v1/deploys/deploy123/files/").unwrap();
                    url.path_segments_mut().unwrap().pop_if_empty().extend(name.trim_start_matches('/').split('/'));
                    assert_eq!(request.path, url.path());
                    assert_eq!(request.body, std::fs::read(path).unwrap());
                    json!({}).into()
                }
                4 => json!({"id":"deploy123","state":"processing"}).into(),
                5 => ready().into(),
                _ => panic!("unexpected request"),
            }
        });
        let deployed = server
            .client()
            .deploy("manual.netlify.app", root, production)
            .unwrap();
        assert_eq!(
            deployed.url("/products/.docs").as_str(),
            "https://deploy123--manual.netlify.app/products/.docs/"
        );
        assert_eq!(server.requests.lock().unwrap().len(), 6);
    }
}

#[test]
fn portable_delivery_builds_generated_files_once_and_preserves_previous_output() {
    let tmp = tempfile::tempdir().unwrap();
    let root = Utf8Path::from_path(tmp.path()).unwrap();
    write(root, "zel.yaml", "version: 1\nrenderer: portable\n");
    write(root, "content/index.md", "# Manual\n\n[Next](more.md)\n");
    write(root, "content/more.md", "# More\n");
    write(root, "content/unreferenced.txt", "DO NOT UPLOAD");
    write(root, "site/previous.txt", "previous output");
    let site = zellij_core::site::Site::load(root).unwrap();
    let server = Server::new(|request, index| match index {
        0 => {
            assert_eq!(request.method, "GET");
            json!({"id":"site123"}).into()
        }
        1 => {
            assert_eq!(request.method, "POST");
            let body: Value = serde_json::from_slice(&request.body).unwrap();
            let files = body["files"].as_object().unwrap();
            assert!(files.contains_key("/index.html") && files.contains_key("/more.html"));
            assert!(!files.keys().any(|k| k.ends_with(".md")
                || k.contains("zel.yaml")
                || k.contains("previous")
                || k.contains("unreferenced")
                || k.contains(".zellij-output")));
            ready().into()
        }
        _ => panic!("duplicate build/delivery request"),
    });
    crate::delivery::deploy(&site, &site.content, &server.client(), "site123", false).unwrap();
    assert_eq!(
        std::fs::read_to_string(root.join("site/previous.txt")).unwrap(),
        "previous output"
    );
    assert_eq!(std::fs::read_dir(root.join("site")).unwrap().count(), 1);
    assert_eq!(server.requests.lock().unwrap().len(), 2);
    // A failed native build cannot create a remote deployment.
    write(root, "content/index.md", "[Broken](missing.md)");
    let server = Server::new(|_, index| {
        assert_eq!(index, 0);
        json!({"id":"site123"}).into()
    });
    assert!(
        crate::delivery::deploy(&site, &site.content, &server.client(), "site123", false).is_err()
    );
    assert_eq!(server.requests.lock().unwrap().len(), 1);
}

#[test]
fn api_failures_are_bounded_and_redact_untrusted_bodies_and_credentials() {
    let tmp = tempfile::tempdir().unwrap();
    let root = Utf8Path::from_path(tmp.path()).unwrap();
    export(root);
    for status in [401, 403, 404, 429, 500] {
        let server = Server::new(move |_, _| Reply {
            status,
            body: json!({"message":"fixture-token secret \u{1b}[31m"}),
            redirect: None,
        });
        let error = server.client().check_site("manual").unwrap_err();
        assert!(error.to_string().contains(&status.to_string()));
        assert!(
            !error.to_string().contains("fixture-token") && !error.to_string().contains("secret")
        );
        assert_eq!(error.exit_code(), 3);
        assert_eq!(server.requests.lock().unwrap().len(), 1);
    }
    // Redirects must not receive bearer tokens, even on successful endpoints.
    let target = Server::new(|_, _| panic!("credential forwarded"));
    let redirect = target.address.clone();
    let server = Server::new(move |_, _| Reply {
        status: 307,
        body: json!({}),
        redirect: Some(redirect.clone()),
    });
    assert!(
        server
            .client()
            .check_site("manual")
            .unwrap_err()
            .to_string()
            .contains("redirect refused")
    );
    assert!(target.requests.lock().unwrap().is_empty());
    // No retry after an uncertain POST; finite polling, even if never ready.
    let server = Server::new(|_, _| json!({"id":"deploy123","state":"preparing"}).into());
    let client = Client::new(
        &server.address,
        "fixture-token",
        Duration::from_millis(300),
        Duration::from_millis(30),
    )
    .unwrap();
    assert!(client.deploy("manual", root, false).is_err());
    assert_eq!(
        server
            .requests
            .lock()
            .unwrap()
            .iter()
            .filter(|r| r.method == "POST")
            .count(),
        1
    );
}

#[test]
fn invalid_or_failed_deployments_never_upload_unknown_files_or_report_success() {
    let tmp = tempfile::tempdir().unwrap();
    let root = Utf8Path::from_path(tmp.path()).unwrap();
    export(root);
    for body in [
        json!({"id":"../../other", "state":"ready"}),
        json!({"id":"deploy123", "state":"error", "error_message":"secret"}),
        json!({"id":"deploy123", "state":"prepared", "required":["unknown-hash"]}),
        json!({"id":"deploy123", "state":"prepared", "required":[], "required_functions":["unexpected"]}),
        json!({"id":"deploy123", "state":"prepared"}),
        json!({"id":"deploy123", "state":"ready", "deploy_ssl_url":"http://insecure.test"}),
        json!({"id":"deploy123", "state":"ready", "deploy_ssl_url":"https://user:secret@site.test"}),
        json!({"id":"deploy123", "state":"unknown"}),
        json!({"invalid":"secret"}),
    ] {
        let server = Server::new(move |_, _| body.clone().into());
        let error = server.client().deploy("manual", root, false).err().unwrap();
        assert!(!error.to_string().contains("secret"));
        assert_eq!(server.requests.lock().unwrap().len(), 1);
    }
    let hash = format!(
        "{:x}",
        Sha1::digest(std::fs::read(root.join("index.html")).unwrap())
    );
    let server = Server::new(move |_, index| {
        if index == 0 {
            json!({"id":"deploy123","state":"prepared","required":[hash]}).into()
        } else {
            Reply {
                status: 500,
                body: json!({}),
                redirect: None,
            }
        }
    });
    assert!(server.client().deploy("manual", root, false).is_err());
    assert_eq!(server.requests.lock().unwrap().len(), 2);
}

#[test]
fn invalid_exports_fail_before_creating_a_deployment() {
    let tmp = tempfile::tempdir().unwrap();
    let root = Utf8Path::from_path(tmp.path()).unwrap();
    let server = Server::new(|_, _| panic!("invalid export deployed"));
    assert!(server.client().deploy("manual", root, false).is_err());
    export(root);
    write(root, "bad#file.png", "bad");
    assert!(server.client().deploy("manual", root, false).is_err());
    std::fs::remove_file(root.join("bad#file.png")).unwrap();
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(root.join("index.html"), root.join("link.html")).unwrap();
        assert!(server.client().deploy("manual", root, false).is_err());
    }
    assert!(server.requests.lock().unwrap().is_empty());
}

#[test]
#[ignore = "requires an already running container runtime and the checkout's engine image; CI runs it explicitly"]
fn hosted_container_export_upload_preserves_prefix_assets_and_previous_output() {
    let tmp = tempfile::tempdir().unwrap();
    let root = Utf8Path::from_path(tmp.path()).unwrap();
    write(root, "zel.yaml", "version: 1\nrenderer: hosted\n");
    write(
        root,
        "content/site.yaml",
        "name: Netlify acceptance\nhome: 01-guide\nbasePath: /products/.docs\nnav: {search: true, themeToggle: true}\n",
    );
    write(
        root,
        "content/01-guide/_sidebar.yaml",
        "title: Manual\nroute: /manual\nnav: [index, next]\n",
    );
    write(
        root,
        "content/01-guide/index.mdx",
        "# Netlify acceptance\n\n[Next](/manual/next)\n\n![Logo](/_zellij/assets/logo.svg)\n",
    );
    write(
        root,
        "content/01-guide/next.mdx",
        "# Next page\n\nMore documentation.\n",
    );
    write(
        root,
        "content/assets/logo.svg",
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"20\" height=\"20\"><circle cx=\"10\" cy=\"10\" r=\"9\"/></svg>",
    );
    write(root, "site/previous.txt", "previous output");
    let manifest = Arc::new(Mutex::new(BTreeMap::<String, String>::new()));
    let uploaded = Arc::new(Mutex::new(BTreeMap::<String, Vec<u8>>::new()));
    let (files, bytes) = (manifest.clone(), uploaded.clone());
    let observed_root = root.to_owned();
    let server = Server::new(move |request, _| match request.method.as_str() {
        "POST" => {
            let stages = std::fs::read_dir(&observed_root)
                .unwrap()
                .flatten()
                .filter(|entry| {
                    entry
                        .file_name()
                        .to_string_lossy()
                        .starts_with(".zel-deploy-")
                })
                .count();
            assert_eq!(
                stages, 1,
                "hosted export must use the project's shared filesystem"
            );
            let body: Value = serde_json::from_slice(&request.body).unwrap();
            assert_eq!(body["draft"], true);
            let hashes: BTreeMap<String, String> =
                serde_json::from_value(body["files"].clone()).unwrap();
            assert!(
                files.lock().unwrap().is_empty(),
                "more than one deployment created"
            );
            *files.lock().unwrap() = hashes.clone();
            json!({"id":"deploy123","state":"prepared","required":hashes.values().collect::<Vec<_>>()}).into()
        }
        "PUT" => {
            let hash = format!("{:x}", Sha1::digest(&request.body));
            let files = files.lock().unwrap();
            let matches_path =
                files
                    .iter()
                    .filter(|(_, digest)| **digest == hash)
                    .any(|(path, _)| {
                        let mut url =
                            Url::parse("https://fixture/api/v1/deploys/deploy123/files/").unwrap();
                        url.path_segments_mut()
                            .unwrap()
                            .pop_if_empty()
                            .extend(path.trim_start_matches('/').split('/'));
                        url.path() == request.path
                    });
            assert!(matches_path, "unrecognized uploaded file {}", request.path);
            assert!(
                bytes
                    .lock()
                    .unwrap()
                    .insert(hash, request.body.clone())
                    .is_none(),
                "duplicate upload"
            );
            json!({}).into()
        }
        "GET" if request.path.ends_with("/sites/site123") => json!({"id":"site123"}).into(),
        "GET" => {
            assert!(
                files
                    .lock()
                    .unwrap()
                    .values()
                    .all(|hash| bytes.lock().unwrap().contains_key(hash))
            );
            ready().into()
        }
        _ => panic!("unexpected API operation"),
    });
    let site = zellij_core::site::Site::load(root).unwrap();
    crate::delivery::deploy(&site, &site.content, &server.client(), "site123", false).unwrap();
    let files = manifest.lock().unwrap();
    let bytes = uploaded.lock().unwrap();
    assert!(
        files
            .keys()
            .all(|path| path.starts_with("/products/.docs/"))
    );
    assert!(!files.keys().any(|path| path.ends_with(".mdx")
        || path.contains(".zellij-output")
        || path.contains(".zellij-deployment")
        || path.ends_with("site.yaml")));
    let home = String::from_utf8_lossy(&bytes[&files["/products/.docs/index.html"]]);
    assert!(home.contains("Netlify acceptance"));
    assert!(home.contains("/products/.docs/manual/next"));
    assert!(home.contains("/products/.docs/_zellij/assets/logo.svg"));
    assert!(files.contains_key("/products/.docs/manual/next.html"));
    assert!(files.contains_key("/products/.docs/_zellij/assets/logo.svg"));
    assert!(
        files
            .keys()
            .any(|path| path.contains("/_next/") && path.ends_with(".js"))
    );
    assert!(
        files
            .keys()
            .any(|path| path.contains("search") && path.ends_with(".json"))
    );
    assert_eq!(
        std::fs::read_dir(root).unwrap().count(),
        3,
        "temporary hosted export was not cleaned up"
    );
    assert_eq!(
        std::fs::read_to_string(root.join("site/previous.txt")).unwrap(),
        "previous output"
    );
}
