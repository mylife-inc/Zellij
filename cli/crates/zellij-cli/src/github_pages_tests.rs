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
        let address = format!("http://{}/", listener.local_addr().unwrap());
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
                let request = match read_request(&socket) {
                    Ok(Some(request)) => request,
                    Ok(None) => continue, // Client cancelled before completing a request.
                    Err(error)
                        if matches!(
                            error.kind(),
                            std::io::ErrorKind::UnexpectedEof
                                | std::io::ErrorKind::ConnectionReset
                                | std::io::ErrorKind::ConnectionAborted
                                | std::io::ErrorKind::TimedOut
                                | std::io::ErrorKind::WouldBlock
                        ) =>
                    {
                        continue;
                    }
                    Err(error) => panic!("invalid fixture request: {error}"),
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
}
fn read_request(socket: &std::net::TcpStream) -> std::io::Result<Option<Request>> {
    let mut reader = BufReader::new(socket.try_clone()?);
    let mut line = String::new();
    if reader.read_line(&mut line)? == 0 || !line.ends_with("\r\n") {
        return Ok(None);
    }
    let parts: Vec<_> = line.split_whitespace().collect();
    if parts.len() != 3 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "invalid request line",
        ));
    }
    let (method, path) = (parts[0].to_owned(), parts[1].to_owned());
    let mut headers = BTreeMap::new();
    loop {
        let mut line = String::new();
        if reader.read_line(&mut line)? == 0 || !line.ends_with("\r\n") {
            return Ok(None);
        }
        if line == "\r\n" {
            break;
        }
        let (key, value) = line.split_once(':').ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, "invalid header")
        })?;
        assert!(
            headers
                .insert(key.to_ascii_lowercase(), value.trim().to_owned())
                .is_none(),
            "duplicate header: {key}"
        );
    }
    let size = headers
        .get("content-length")
        .map(|v| v.parse::<usize>())
        .transpose()
        .map_err(|_| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, "invalid content length")
        })?
        .unwrap_or(0);
    let mut body = vec![0; size];
    reader.read_exact(&mut body)?;
    Ok(Some(Request {
        method,
        path,
        headers,
        body,
    }))
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
const OLD: &str = "1111111111111111111111111111111111111111";
const NEW: &str = "2222222222222222222222222222222222222222";
const TREE: &str = "3333333333333333333333333333333333333333";
const OTHER: &str = "4444444444444444444444444444444444444444";
const REPO: &str = "acme/manual";
impl Server {
    fn client(&self) -> Client {
        Client::new(
            &self.address,
            "fixture-token",
            Duration::from_secs(5),
            Duration::from_millis(1),
            Duration::ZERO,
        )
        .unwrap()
    }
}
fn write(root: &Utf8Path, path: &str, body: impl AsRef<[u8]>) {
    let path = root.join(path);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, body).unwrap();
}
fn root(temp: &tempfile::TempDir) -> &Utf8Path {
    Utf8Path::from_path(temp.path()).unwrap()
}
fn repository() -> Value {
    json!({"id":42,"full_name":REPO,"default_branch":"main","archived":false})
}
fn pages() -> Value {
    json!({"build_type":"legacy","source":{"branch":"gh-pages","path":"/"},"html_url":"https://acme.github.io/manual/","cname":null})
}
fn reference(sha: &str) -> Value {
    json!({"ref":"refs/heads/gh-pages","object":{"type":"commit","sha":sha}})
}
fn target() -> Target {
    Target {
        repository: serde_json::from_value(repository()).unwrap(),
        pages: serde_json::from_value(pages()).unwrap(),
        head: OLD.into(),
        url: Url::parse("https://acme.github.io/manual/").unwrap(),
        mount: "/manual".into(),
    }
}
struct State {
    head: String,
    polls: usize,
}
impl State {
    fn respond(&mut self, r: &Request) -> Reply {
        assert_eq!(r.headers["authorization"], "Bearer fixture-token");
        assert_eq!(r.headers["x-github-api-version"], "2022-11-28");
        assert_eq!(r.headers["accept"], "application/vnd.github+json");
        assert!(!r.headers.contains_key("cookie"));
        let suffix = r
            .path
            .strip_prefix("/repos/acme/manual")
            .expect("repository-scoped API");
        match (r.method.as_str(),suffix) {
            ("GET", "")=>repository().into(),
            ("GET", "/pages")=>pages().into(),
            ("GET", "/git/ref/heads/gh-pages")=>reference(&self.head).into(),
            ("POST", "/git/blobs")=>{
                let body:Value=serde_json::from_slice(&r.body).unwrap();
                assert_eq!(body["encoding"],"base64");
                let bytes=STANDARD.decode(body["content"].as_str().unwrap()).unwrap();
                json!({"sha":blob_hash(&bytes)}).into()
            },
            ("POST", "/git/trees")=>{
                let body:Value=serde_json::from_slice(&r.body).unwrap();
                assert!(body.get("base_tree").is_none(),"snapshot must replace old files");
                for file in body["tree"].as_array().unwrap() {
                    assert_eq!(file["mode"],"100644");assert_eq!(file["type"],"blob");
                    assert_ne!(file.get("content").is_some(),file.get("sha").is_some());
                }
                json!({"sha":TREE}).into()
            },
            ("POST", "/git/commits")=>{
                let body:Value=serde_json::from_slice(&r.body).unwrap();
                assert_eq!(body["parents"],json!([OLD]));assert_eq!(body["tree"],TREE);
                json!({"sha":NEW,"tree":{"sha":TREE},"parents":[{"sha":OLD}]}).into()
            },
            ("PATCH", "/git/refs/heads/gh-pages")=>{
                let body:Value=serde_json::from_slice(&r.body).unwrap();
                assert_eq!(body,json!({"sha":NEW,"force":false}));
                self.head=NEW.into();reference(NEW).into()
            },
            ("POST", "/pages/builds")=>json!({"status":"queued","url":"https://api.github.com/repos/acme/manual/pages/builds/latest"}).into(),
            ("GET", "/pages/builds/latest")=>{
                self.polls+=1;
                json!({"commit":if self.polls==1 { OLD }else{ NEW },"status":if self.polls==2 {"building"}else{"built"},"error":{"message":null}}).into()
            },
            _=>panic!("unexpected request {r:?}"),
        }
    }
}
fn deployment_server(
    hook: impl Fn(&Request, &mut State) -> Option<Reply> + Send + 'static,
) -> Server {
    let state = Mutex::new(State {
        head: OLD.into(),
        polls: 0,
    });
    Server::new(move |r, _| {
        let mut state = state.lock().unwrap();
        hook(r, &mut state).unwrap_or_else(|| state.respond(r))
    })
}
fn uploaded(server: &Server) -> BTreeMap<String, Vec<u8>> {
    let requests = server.requests.lock().unwrap();
    let blobs: BTreeMap<_, _> = requests
        .iter()
        .filter(|r| r.path.ends_with("/git/blobs"))
        .map(|r| {
            let body: Value = serde_json::from_slice(&r.body).unwrap();
            let bytes = STANDARD.decode(body["content"].as_str().unwrap()).unwrap();
            (blob_hash(&bytes), bytes)
        })
        .collect();
    let tree = requests
        .iter()
        .find(|r| r.method == "POST" && r.path.ends_with("/git/trees"))
        .unwrap();
    let tree: Value = serde_json::from_slice(&tree.body).unwrap();
    tree["tree"]
        .as_array()
        .unwrap()
        .iter()
        .map(|f| {
            (
                f["path"].as_str().unwrap().into(),
                if let Some(text) = f["content"].as_str() {
                    text.as_bytes().to_vec()
                } else {
                    blobs[f["sha"].as_str().unwrap()].clone()
                },
            )
        })
        .collect()
}
fn deploy(server: &Server, root: &Utf8Path) -> Result<Deployed> {
    let client = server.client();
    let target = client.check_site(REPO)?;
    client.deploy(&target, root, Renderer::Portable, "")
}
#[test]
fn publishes_exact_snapshot_without_force_and_waits_for_matching_commit() {
    let tmp = tempfile::tempdir().unwrap();
    let r = root(&tmp);
    write(r, "index.html", "<h1>Docs é</h1>");
    write(r, "nested/binary.bin", [0, 255, 254, 10]);
    write(r, "same.bin", [0, 255, 254, 10]);
    write(r, ".zellij-output", "private marker");
    let server = deployment_server(|_, _| None);
    let done = deploy(&server, r).unwrap();
    assert_eq!(done.commit, NEW);
    assert_eq!(done.url.as_str(), "https://acme.github.io/manual/");
    let files = uploaded(&server);
    assert_eq!(files.len(), 4);
    assert_eq!(files["index.html"], "<h1>Docs é</h1>".as_bytes());
    assert_eq!(files["same.bin"], vec![0, 255, 254, 10]);
    assert_eq!(files["nested/binary.bin"], files["same.bin"]);
    assert!(files[".nojekyll"].is_empty());
    let requests = server.requests.lock().unwrap();
    assert_eq!(
        requests
            .iter()
            .filter(|r| r.path.ends_with("/git/blobs"))
            .count(),
        1
    );
    assert_eq!(requests.iter().filter(|r| r.method == "PATCH").count(), 1);
    assert_eq!(
        requests
            .iter()
            .filter(|r| r.path.ends_with("/pages/builds/latest"))
            .count(),
        3
    );
    assert_eq!(
        blob_hash(b"hello\n"),
        "ce013625030ba8dba906f756967f9e9ca394464a"
    );
}
#[test]
fn hosted_mount_is_stripped_once_and_custom_domains_are_preserved() {
    for (prefix, name) in [
        ("/manual", "index.html"),
        ("/manual/products", "products/index.html"),
    ] {
        let tmp = tempfile::tempdir().unwrap();
        write(
            root(&tmp),
            &format!("{}/index.html", prefix.trim_start_matches('/')),
            "Docs",
        );
        let server = deployment_server(|_, _| None);
        let done = server
            .client()
            .deploy(&target(), root(&tmp), Renderer::Hosted, prefix)
            .unwrap();
        assert_eq!(done.url.path(), format!("{prefix}/"));
        assert!(uploaded(&server).contains_key(name));
    }
    let server = deployment_server(|r, _| {
        if r.path.ends_with("/pages") {
            let mut p = pages();
            p["html_url"] = json!("https://docs.example/");
            p["cname"] = json!("docs.example");
            Some(p.into())
        } else {
            None
        }
    });
    let tmp = tempfile::tempdir().unwrap();
    write(root(&tmp), "index.html", "Docs");
    let done = deploy(&server, root(&tmp)).unwrap();
    assert_eq!(done.url.as_str(), "https://docs.example/");
    assert_eq!(uploaded(&server)["CNAME"], b"docs.example\n");
}
#[test]
fn preflight_rejects_unsafe_configuration_and_identity_before_mutation() {
    for (endpoint, field, value) in [
        ("", "/archived", json!(true)),
        ("", "/id", json!(0)),
        ("", "/full_name", json!("other/repo")),
        ("", "/default_branch", json!("gh-pages")),
        ("/pages", "/build_type", json!("workflow")),
        ("/pages", "/source/branch", json!("main")),
        ("/pages", "/source/path", json!("/docs")),
        (
            "/pages",
            "/html_url",
            json!("http://acme.github.io/manual/"),
        ),
        (
            "/pages",
            "/html_url",
            json!("https://token@acme.github.io/"),
        ),
        ("/pages", "/cname", json!("bad.example/extra")),
        ("/git/ref/heads/gh-pages", "/ref", json!("refs/heads/main")),
        ("/git/ref/heads/gh-pages", "/object/sha", json!("invalid")),
    ] {
        let server = deployment_server(move |r, _| {
            if r.path == format!("/repos/{REPO}{endpoint}") {
                let mut v = match endpoint {
                    "" => repository(),
                    "/pages" => pages(),
                    _ => reference(OLD),
                };
                *v.pointer_mut(field).unwrap() = value.clone();
                Some(v.into())
            } else {
                None
            }
        });
        assert!(
            server.client().check_site(REPO).is_err(),
            "{endpoint} {field}"
        );
        assert!(
            server
                .requests
                .lock()
                .unwrap()
                .iter()
                .all(|r| r.method == "GET")
        );
    }
    let server = deployment_server(|_, _| None);
    assert!(server.client().check_site("../repo").is_err());
    assert!(server.requests.lock().unwrap().is_empty());
    for token in ["", "bad token", "bad\ntoken"] {
        assert!(
            Client::new(
                "https://api.github.com/",
                token,
                Duration::from_secs(1),
                Duration::ZERO,
                Duration::ZERO
            )
            .is_err()
        );
    }
}
#[test]
fn concurrent_branch_or_settings_change_stops_before_branch_update() {
    for field in ["branch", "settings"] {
        let reads = Mutex::new(0);
        let server = deployment_server(move |r, _| {
            if r.path == format!("/repos/{REPO}") {
                *reads.lock().unwrap() += 1;
            }
            if *reads.lock().unwrap() > 1 {
                if field == "branch" && r.path.ends_with("/git/ref/heads/gh-pages") {
                    return Some(reference(OTHER).into());
                }
                if field == "settings" && r.path.ends_with("/pages") {
                    let mut p = pages();
                    p["html_url"] = json!("https://elsewhere.example/");
                    return Some(p.into());
                }
            }
            None
        });
        let tmp = tempfile::tempdir().unwrap();
        write(root(&tmp), "index.html", "Docs");
        assert!(
            deploy(&server, root(&tmp))
                .err()
                .unwrap()
                .to_string()
                .contains("changed during deployment")
        );
        assert!(
            !server
                .requests
                .lock()
                .unwrap()
                .iter()
                .any(|r| r.method == "PATCH")
        );
    }
}
#[test]
fn mutation_failures_are_redacted_and_not_retried() {
    for (method, path, status) in [
        ("POST", "/git/blobs", 403),
        ("POST", "/git/trees", 500),
        ("POST", "/git/commits", 422),
        ("PATCH", "/git/refs/heads/gh-pages", 409),
        ("POST", "/pages/builds", 403),
    ] {
        let server = deployment_server(move |r, _| {
            if r.method == method && r.path.ends_with(path) {
                Some(Reply {
                    status,
                    body: json!({"message":"PRIVATE fixture-token"}),
                    redirect: None,
                })
            } else {
                None
            }
        });
        let tmp = tempfile::tempdir().unwrap();
        write(root(&tmp), "index.html", "Docs");
        write(root(&tmp), "image.bin", [255, 0]);
        let e = deploy(&server, root(&tmp)).err().unwrap().to_string();
        assert!(
            e.contains(&format!("HTTP {status}"))
                && !e.contains("PRIVATE")
                && !e.contains("fixture-token"),
            "{e}"
        );
        let requests = server.requests.lock().unwrap();
        assert_eq!(
            requests
                .iter()
                .filter(|r| r.method == method && r.path.ends_with(path))
                .count(),
            1
        );
        assert!(requests.last().unwrap().path.ends_with(path));
    }
}
#[test]
fn wrong_object_identity_stops_publication() {
    for path in ["/git/blobs", "/git/trees", "/git/commits"] {
        let server = deployment_server(move |r, _| {
            if r.method == "POST" && r.path.ends_with(path) {
                Some(json!({"sha":"bad","tree":{"sha":TREE},"parents":[{"sha":OLD}]}).into())
            } else {
                None
            }
        });
        let tmp = tempfile::tempdir().unwrap();
        write(root(&tmp), "index.html", "Docs");
        write(root(&tmp), "image.bin", [255, 0]);
        assert!(deploy(&server, root(&tmp)).is_err());
        assert!(
            !server
                .requests
                .lock()
                .unwrap()
                .iter()
                .any(|r| r.method == "PATCH")
        );
    }
}
#[test]
fn failed_superseded_and_stale_builds_never_report_success() {
    for mode in ["failed", "superseded", "stale", "missing", "unknown"] {
        let server = deployment_server(move |r, state| {
            if mode == "superseded"
                && state.head == NEW
                && r.path.ends_with("/git/ref/heads/gh-pages")
            {
                return Some(reference(OTHER).into());
            }
            if r.path.ends_with("/pages/builds/latest") {
                return Some(if mode == "missing" {
                    Reply {
                        status: 404,
                        body: json!({}),
                        redirect: None,
                    }
                } else {
                    json!({"commit":if mode=="stale" {OLD}else{NEW},"status":match mode {"failed"=>"errored","unknown"=>"unexpected",_=>"built"},"error":{"message":if mode=="failed" {Some("PRIVATE fixture-token")}else{None}}}).into()
                });
            }
            None
        });
        let tmp = tempfile::tempdir().unwrap();
        write(root(&tmp), "index.html", "Docs");
        let mut client = server.client();
        client.timeout = Duration::from_secs(1);
        client.poll_interval = Duration::from_millis(25);
        let e = client
            .deploy(&target(), root(&tmp), Renderer::Portable, "")
            .err()
            .unwrap()
            .to_string();
        assert!(!e.contains("PRIVATE") && !e.contains("fixture-token"));
        if mode == "stale" || mode == "missing" {
            assert!(e.contains("timed out"), "{e}");
        }
        let requests = server.requests.lock().unwrap();
        assert_eq!(requests.iter().filter(|r| r.method == "PATCH").count(), 1);
        assert_eq!(
            requests
                .iter()
                .filter(|r| r.method == "POST" && r.path.ends_with("/pages/builds"))
                .count(),
            1
        );
    }
}
#[test]
fn redirects_are_not_followed_and_responses_are_bounded() {
    let sink = Server::new(|_, _| panic!("credential redirect followed"));
    let url = sink.address.clone();
    let server = Server::new(move |_, _| Reply {
        status: 302,
        body: json!({"message":"PRIVATE"}),
        redirect: Some(url.clone()),
    });
    assert!(
        server
            .client()
            .check_site(REPO)
            .err()
            .unwrap()
            .to_string()
            .contains("redirect refused")
    );
    assert!(sink.requests.lock().unwrap().is_empty());
    let server = Server::new(|_, _| json!({"body":"x".repeat(RESPONSE_LIMIT as usize)}).into());
    assert!(
        server
            .client()
            .check_site(REPO)
            .err()
            .unwrap()
            .to_string()
            .contains("exceeded 8 MiB")
    );
}
#[test]
fn manifest_rejects_unsafe_files_conflicting_domains_and_limits() {
    for path in [
        ".github/workflows/run.yml",
        ".git/config",
        ".gitattributes",
        "_config.yml",
        "bad#.html",
        "CNAME",
    ] {
        let tmp = tempfile::tempdir().unwrap();
        write(root(&tmp), "index.html", "Docs");
        write(root(&tmp), path, "bad");
        assert!(
            Manifest::read(root(&tmp), &target(), Renderer::Portable, "").is_err(),
            "{path}"
        );
    }
    let tmp = tempfile::tempdir().unwrap();
    write(root(&tmp), "index.html", "Docs");
    std::fs::File::create(root(&tmp).join("huge"))
        .unwrap()
        .set_len(MAX_FILE + 1)
        .unwrap();
    assert!(Manifest::read(root(&tmp), &target(), Renderer::Portable, "").is_err());
    let tmp = tempfile::tempdir().unwrap();
    write(root(&tmp), "manual/index.html", "Docs");
    write(root(&tmp), "outside.html", "Docs");
    assert!(Manifest::read(root(&tmp), &target(), Renderer::Hosted, "/manual").is_err());
    let tmp = tempfile::tempdir().unwrap();
    write(root(&tmp), "index.html", "Docs");
    for i in 0..=MAX_UPLOADS {
        write(
            root(&tmp),
            &format!("asset{i}"),
            [255, (i % 256) as u8, (i / 256) as u8],
        );
    }
    assert!(
        Manifest::read(root(&tmp), &target(), Renderer::Portable, "")
            .err()
            .unwrap()
            .to_string()
            .contains("200 distinct")
    );
}
#[cfg(unix)]
#[test]
fn export_links_and_special_files_are_rejected() {
    let tmp = tempfile::tempdir().unwrap();
    let r = root(&tmp);
    write(r, "index.html", "Docs");
    std::os::unix::fs::symlink(r.join("index.html"), r.join("link")).unwrap();
    assert!(Manifest::read(r, &target(), Renderer::Portable, "").is_err());
    std::fs::remove_file(r.join("link")).unwrap();
    let _socket = std::os::unix::net::UnixListener::bind(r.join("socket")).unwrap();
    assert!(Manifest::read(r, &target(), Renderer::Portable, "").is_err());
}
#[test]
fn hosted_defaults_follow_pages_mount_and_reject_outside_prefixes() {
    let t = target();
    let original = BuildOptions::default();
    let options = t.build_options(Renderer::Hosted, &original).unwrap();
    assert_eq!(options.overrides.base_path.unwrap().to_string(), "/manual");
    assert_eq!(
        options.overrides.site_url.unwrap().to_string(),
        "https://acme.github.io"
    );
    assert!(original.overrides.base_path.is_none());
    for prefix in ["", "/manual-other", "/other"] {
        let mut options = BuildOptions::default();
        options.overrides.base_path = Some(prefix.parse().unwrap());
        assert!(t.build_options(Renderer::Hosted, &options).is_err());
    }
    let mut options = BuildOptions::default();
    options.overrides.base_path = Some("/manual/guide".parse().unwrap());
    options.overrides.site_url = Some("https://explicit.example".parse().unwrap());
    let options = t.build_options(Renderer::Hosted, &options).unwrap();
    assert_eq!(
        options.overrides.site_url.unwrap().to_string(),
        "https://explicit.example"
    );
}
#[test]
fn portable_delivery_preserves_source_and_previous_output_and_preflights_before_build() {
    let tmp = tempfile::tempdir().unwrap();
    let r = root(&tmp);
    write(r, "zel.yaml", "version: 1\nrenderer: portable\n");
    write(r, "content/index.md", "# Docs\n\n[Guide](guide.md)\n");
    write(r, "content/guide.md", "# Guide\n");
    write(r, "site/previous.txt", "keep");
    let site = zellij_core::site::Site::load(r).unwrap();
    let server = deployment_server(|_, _| None);
    crate::delivery::deploy_github_pages(
        &site,
        &site.content,
        &server.client(),
        REPO,
        &Default::default(),
    )
    .unwrap();
    let files = uploaded(&server);
    assert!(String::from_utf8_lossy(&files["index.html"]).contains("guide.html"));
    assert!(files.contains_key("guide.html"));
    assert_eq!(
        std::fs::read_to_string(r.join("site/previous.txt")).unwrap(),
        "keep"
    );
    assert_eq!(std::fs::read_dir(r).unwrap().count(), 3);
    let count = server.requests.lock().unwrap().len();
    write(r, "content/index.md", "[Broken](missing.md)");
    assert!(
        crate::delivery::deploy_github_pages(
            &site,
            &site.content,
            &server.client(),
            REPO,
            &Default::default()
        )
        .is_err()
    );
    assert_eq!(server.requests.lock().unwrap().len(), count + 3);
    let mut options = BuildOptions::default();
    options.overrides.base_path = Some("/manual".parse().unwrap());
    assert!(
        crate::delivery::deploy_github_pages(
            &site,
            &site.content,
            &server.client(),
            REPO,
            &options
        )
        .is_err()
    );
    assert_eq!(server.requests.lock().unwrap().len(), count + 3);
    let deny = Server::new(|_, _| Reply {
        status: 404,
        body: json!({}),
        redirect: None,
    });
    let e = crate::delivery::deploy_github_pages(
        &site,
        &site.content,
        &deny.client(),
        REPO,
        &Default::default(),
    )
    .unwrap_err()
    .to_string();
    assert!(e.contains("repository preflight") && e.contains("404"));
}

#[test]
#[ignore = "requires an already running container runtime and the checkout's engine image; CI runs it explicitly"]
fn hosted_export_uses_pages_mount_and_preserves_assets_source_and_previous_output() {
    let temp = tempfile::tempdir().unwrap();
    let r = root(&temp);
    write(r, "zel.yaml", "version: 1\nrenderer: hosted\n");
    let authored =
        "name: GitHub Pages acceptance\nhome: guide\nbasePath: /authored\nnav: {search: true}\n";
    write(r, "content/site.yaml", authored);
    write(
        r,
        "content/guide/_sidebar.yaml",
        "title: Manual\nroute: /guide\nnav: [index, next]\n",
    );
    write(
        r,
        "content/guide/index.mdx",
        "# GitHub Pages acceptance\n\n[Next](/guide/next)\n\n![Logo](/_zellij/assets/logo.svg)\n",
    );
    write(r, "content/guide/next.mdx", "# Next\n\nMore docs\n");
    write(
        r,
        "content/assets/logo.svg",
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"20\" height=\"20\"/>",
    );
    write(r, "site/previous.txt", "retain");
    let site = zellij_core::site::Site::load(r).unwrap();
    let server = deployment_server(|_, _| None);
    crate::delivery::deploy_github_pages(
        &site,
        &site.content,
        &server.client(),
        REPO,
        &Default::default(),
    )
    .unwrap();
    let files = uploaded(&server);
    assert!(files.contains_key("index.html") && files.contains_key("guide/next.html"));
    assert!(files.keys().all(|p| !p.starts_with("manual/")
        && !p.ends_with(".mdx")
        && !p.ends_with("site.yaml")
        && !p.contains(".zellij-output")));
    let guide = String::from_utf8_lossy(&files["guide.html"]);
    assert!(
        guide.contains("GitHub Pages acceptance")
            && guide.contains("/manual/guide/next")
            && guide.contains("/manual/_zellij/assets/logo.svg")
    );
    assert_eq!(
        files["_zellij/assets/logo.svg"],
        std::fs::read(r.join("content/assets/logo.svg")).unwrap()
    );
    assert!(
        files
            .keys()
            .any(|p| p.starts_with("_next/") && p.ends_with(".js"))
    );
    assert_eq!(
        std::fs::read_to_string(r.join("site/previous.txt")).unwrap(),
        "retain"
    );
    assert_eq!(
        std::fs::read_to_string(r.join("content/site.yaml")).unwrap(),
        authored
    );
    assert_eq!(std::fs::read_dir(r).unwrap().count(), 3);
    let count = server.requests.lock().unwrap().len();
    write(r, "content/site.yaml", "name: []\n");
    assert!(
        crate::delivery::deploy_github_pages(
            &site,
            &site.content,
            &server.client(),
            REPO,
            &Default::default()
        )
        .is_err()
    );
    assert_eq!(
        server.requests.lock().unwrap().len(),
        count + 3,
        "failed build performs only repository, Pages and branch preflight"
    );
}
