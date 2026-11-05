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
        let address = format!("http://{}/client/v4/", listener.local_addr().unwrap());
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
const ACCOUNT: &str = "0123456789abcdef0123456789abcdef";
impl Server {
    fn client(&self) -> Client {
        Client::new(
            &self.address,
            "fixture-token",
            ACCOUNT,
            Duration::from_secs(5),
            Duration::from_millis(1),
        )
        .unwrap()
    }
}
fn ok(value: Value) -> Reply {
    json!({"success":true,"result":value}).into()
}
fn ready(production: bool) -> Value {
    json!({"id":"deployment-123","environment":if production {"production"} else {"preview"},"latest_stage":{"name":"deploy","status":"success"},"url":"https://abc.manual.pages.dev"})
}
fn target(production: bool) -> Target {
    Target {
        project: "manual".into(),
        branch: if production { "main" } else { "zel-preview" }.into(),
        production,
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
fn field(body: &[u8], name: &str) -> String {
    let text = String::from_utf8_lossy(body);
    text.split(&format!("name=\"{name}\"\r\n\r\n"))
        .nth(1)
        .unwrap()
        .split("\r\n")
        .next()
        .unwrap()
        .to_owned()
}
fn deployment_server(production: bool, cache: bool) -> Server {
    Server::new(move |request, _| {
        assert!(!request.headers.contains_key("cookie"));
        let auth = request.headers.get("authorization").unwrap();
        if request.path.contains("/pages/assets/") {
            assert_eq!(auth, "Bearer upload-jwt");
        } else {
            assert_eq!(auth, "Bearer fixture-token");
        }
        if request.path.ends_with("/projects/manual") {
            return ok(json!({"name":"manual","production_branch":"main"}));
        }
        if request.path.ends_with("/upload-token") {
            return ok(json!({"jwt":"upload-jwt"}));
        }
        if request.path.ends_with("/check-missing") {
            let body: Value = serde_json::from_slice(&request.body).unwrap();
            return ok(if cache {
                json!([])
            } else {
                body["hashes"].clone()
            });
        }
        if request.path.ends_with("/upload") {
            let body: Vec<Value> = serde_json::from_slice(&request.body).unwrap();
            for file in &body {
                assert_eq!(file["base64"], true);
                STANDARD.decode(file["value"].as_str().unwrap()).unwrap();
            }
            return ok(Value::Null);
        }
        if request.path.ends_with("/upsert-hashes") {
            return ok(Value::Null);
        }
        if request.path.ends_with("/deployments") {
            assert_eq!(request.method, "POST");
            assert!(request.headers["content-type"].starts_with("multipart/form-data; boundary="));
            assert_eq!(
                field(&request.body, "branch"),
                if production { "main" } else { "zel-preview" }
            );
            let manifest: BTreeMap<String, String> =
                serde_json::from_str(&field(&request.body, "manifest")).unwrap();
            assert!(!manifest.is_empty());
            assert!(
                manifest
                    .keys()
                    .all(|p| p.starts_with('/') && !p.contains(".zellij-output"))
            );
            let mut pending = ready(production);
            pending["latest_stage"] = json!({"name":"upload","status":"success"});
            return ok(pending);
        }
        assert!(request.path.ends_with("/deployments/deployment-123"));
        ok(ready(production))
    })
}
#[test]
fn upload_protocol_preserves_paths_hashes_types_and_waits_for_deploy_success() {
    let temp = tempfile::tempdir().unwrap();
    let root = root(&temp);
    write(root, "index.html", "<h1>Docs</h1>");
    write(root, "copy.html", "<h1>Docs</h1>");
    write(root, "assets/日本 100%.svg", "<svg/>");
    write(root, ".zellij-output", "private marker");
    for production in [false, true] {
        let server = deployment_server(production, false);
        let client = server.client();
        let destination = client.check_project("manual", production).unwrap();
        let deployed = client.deploy(&destination, root).unwrap();
        assert_eq!(
            deployed.url("/products/.docs").as_str(),
            "https://abc.manual.pages.dev/products/.docs/"
        );
        let requests = server.requests.lock().unwrap();
        let upload = requests
            .iter()
            .find(|r| r.path.ends_with("/upload"))
            .unwrap();
        let assets: Vec<Value> = serde_json::from_slice(&upload.body).unwrap();
        assert_eq!(assets.len(), 2);
        let manifest: BTreeMap<String, String> = serde_json::from_str(&field(
            &requests
                .iter()
                .find(|r| r.path.ends_with("/deployments"))
                .unwrap()
                .body,
            "manifest",
        ))
        .unwrap();
        assert_eq!(manifest.len(), 3);
        assert_eq!(manifest["/index.html"], manifest["/copy.html"]);
        for (name, hash) in manifest {
            let file = assets.iter().find(|f| f["key"] == hash).unwrap();
            let data = STANDARD.decode(file["value"].as_str().unwrap()).unwrap();
            assert_eq!(
                data,
                std::fs::read(root.join(name.trim_start_matches('/'))).unwrap()
            );
            assert_eq!(hash, hash_file(Utf8Path::new(&name), &data));
            assert_eq!(
                file["metadata"]["contentType"],
                if name.ends_with(".html") {
                    "text/html"
                } else {
                    "image/svg+xml"
                }
            );
        }
        assert_eq!(
            requests
                .iter()
                .filter(|r| r.path.ends_with("/deployments"))
                .count(),
            1
        );
        assert!(requests.last().unwrap().path.ends_with("/deployment-123"));
    }
}
#[test]
fn cached_files_skip_uploads_but_still_create_and_poll() {
    let temp = tempfile::tempdir().unwrap();
    write(root(&temp), "index.html", "cached");
    let server = deployment_server(false, true);
    server.client().deploy(&target(false), root(&temp)).unwrap();
    assert!(
        !server
            .requests
            .lock()
            .unwrap()
            .iter()
            .any(|r| r.path.ends_with("/upload"))
    );
}
#[test]
fn preview_cannot_select_the_projects_production_branch() {
    let server = Server::new(|_, _| ok(json!({"name":"manual","production_branch":"zel-preview"})));
    assert!(
        server
            .client()
            .check_project("manual", false)
            .err()
            .unwrap()
            .to_string()
            .contains("production branch")
    );
    assert_eq!(
        server
            .client()
            .check_project("manual", true)
            .unwrap()
            .branch,
        "zel-preview"
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
#[test]
fn unknown_missing_hashes_and_changed_assets_stop_before_upload_or_creation() {
    for mutate in [false, true] {
        let temp = tempfile::tempdir().unwrap();
        write(root(&temp), "index.html", "before");
        let path = root(&temp).join("index.html");
        let server = Server::new(move |request, _| {
            if request.path.ends_with("upload-token") {
                return ok(json!({"jwt":"upload-jwt"}));
            }
            assert!(request.path.ends_with("check-missing"));
            if mutate {
                std::fs::write(&path, "after").unwrap();
                ok(serde_json::from_slice::<Value>(&request.body).unwrap()["hashes"].clone())
            } else {
                ok(json!(["foreign"]))
            }
        });
        let error = server
            .client()
            .deploy(&target(false), root(&temp))
            .err()
            .unwrap()
            .to_string();
        assert!(
            error.contains(if mutate { "changed" } else { "outside" }),
            "{error}"
        );
        assert_eq!(server.requests.lock().unwrap().len(), 2);
    }
}
#[test]
fn provider_http_errors_and_redirects_never_echo_body_or_token() {
    for status in [302, 401, 403, 404, 429, 500] {
        let server = Server::new(move |_, _| Reply {
            status,
            body: json!({"secret":"fixture-token\u{1b}[31m"}),
            redirect: Some("https://elsewhere.invalid/".into()),
        });
        let error = server
            .client()
            .check_project("manual", false)
            .err()
            .unwrap()
            .to_string();
        assert!(error.contains(&status.to_string()), "{error}");
        assert!(!error.contains("fixture-token") && !error.contains('\u{1b}'));
        assert_eq!(server.requests.lock().unwrap().len(), 1);
    }
}
#[test]
fn malformed_and_unsuccessful_api_responses_fail_closed() {
    for response in [
        json!({"success":false,"result":{"name":"manual","production_branch":"main"}}),
        json!({"success":true}),
        json!({"success":true,"result":{"name":"other","production_branch":"main"}}),
        json!({"success":true,"result":{"name":"manual","production_branch":""}}),
    ] {
        let server = Server::new(move |_, _| response.clone().into());
        assert!(server.client().check_project("manual", false).is_err());
    }
    assert!(
        Client::new(
            API,
            "secret with whitespace",
            ACCOUNT,
            Duration::ZERO,
            Duration::ZERO
        )
        .is_err()
    );
    assert!(Client::new(API, "secret", "../account", Duration::ZERO, Duration::ZERO).is_err());
}
#[test]
fn creation_failure_is_not_retried() {
    let temp = tempfile::tempdir().unwrap();
    write(root(&temp), "index.html", "docs");
    let server = Server::new(|r, _| {
        if r.path.ends_with("upload-token") {
            ok(json!({"jwt":"jwt"}))
        } else if r.path.ends_with("check-missing") {
            ok(json!([]))
        } else if r.path.ends_with("upsert-hashes") {
            ok(Value::Null)
        } else {
            Reply {
                status: 503,
                body: json!({}),
                redirect: None,
            }
        }
    });
    assert!(server.client().deploy(&target(true), root(&temp)).is_err());
    assert_eq!(
        server
            .requests
            .lock()
            .unwrap()
            .iter()
            .filter(|r| r.path.ends_with("deployments"))
            .count(),
        1
    );
}
#[test]
fn failure_environment_identity_url_and_deadline_are_checked() {
    let temp = tempfile::tempdir().unwrap();
    write(root(&temp), "index.html", "docs");
    for scenario in [
        "failure",
        "environment",
        "url",
        "identity",
        "timeout",
        "skipped",
    ] {
        let server = Server::new(move |r, _| {
            if r.path.ends_with("upload-token") {
                return ok(json!({"jwt":"jwt"}));
            }
            if r.path.ends_with("check-missing") {
                return ok(json!([]));
            }
            if r.path.ends_with("upsert-hashes") {
                return ok(Value::Null);
            }
            let mut response = ready(false);
            match scenario {
                "failure" => response["latest_stage"]["status"] = json!("failure"),
                "skipped" => response["is_skipped"] = json!(true),
                "environment" => response["environment"] = json!("production"),
                "url" => response["url"] = json!("http://secret@evil.invalid/"),
                "identity" if r.method == "GET" => response["id"] = json!("other"),
                _ => response["latest_stage"]["status"] = json!("active"),
            }
            ok(response)
        });
        let mut client = server.client();
        if scenario == "timeout" {
            client.timeout = Duration::from_millis(150);
        }
        assert!(
            client.deploy(&target(false), root(&temp)).is_err(),
            "{scenario}"
        );
    }
}
#[test]
fn export_preflight_rejects_special_files_limits_and_missing_index_before_requests() {
    for name in [
        "_worker.js",
        "_redirects",
        "_headers",
        "_routes.json",
        "functions/handler.js",
        "bad?.svg",
    ] {
        if cfg!(windows) && name.contains('?') {
            continue;
        }
        let temp = tempfile::tempdir().unwrap();
        write(root(&temp), "index.html", "docs");
        write(root(&temp), name, "unsupported");
        assert!(Manifest::read(root(&temp)).is_err(), "{name}");
    }
    let temp = tempfile::tempdir().unwrap();
    let r = root(&temp);
    assert!(Manifest::read(r).is_err());
    write(r, "index.html", "docs");
    std::fs::File::create(r.join("large.bin"))
        .unwrap()
        .set_len(MAX_FILE + 1)
        .unwrap();
    let server = Server::new(|_, _| panic!("invalid export must not call API"));
    assert!(server.client().deploy(&target(false), r).is_err());
}
#[cfg(unix)]
#[test]
fn export_rejects_symbolic_links() {
    let temp = tempfile::tempdir().unwrap();
    write(root(&temp), "index.html", "docs");
    std::os::unix::fs::symlink("index.html", root(&temp).join("linked.html")).unwrap();
    assert!(Manifest::read(root(&temp)).is_err());
}
#[test]
fn portable_build_delivery_preserves_previous_output_and_source() {
    let temp = tempfile::tempdir().unwrap();
    let r = root(&temp);
    write(r, "zel.yaml", "version: 1\nrenderer: portable\n");
    write(r, "content/index.md", "# Cloudflare\n\n[Next](next.md)\n");
    write(r, "content/next.md", "# Next\n");
    write(r, "site/previous.txt", "retain");
    let site = zellij_core::site::Site::load(r).unwrap();
    let server = deployment_server(false, false);
    crate::delivery::deploy_cloudflare(
        &site,
        &site.content,
        &server.client(),
        "manual",
        false,
        &Default::default(),
    )
    .unwrap();
    let requests = server.requests.lock().unwrap();
    let create = requests
        .iter()
        .find(|r| r.path.ends_with("/deployments"))
        .unwrap();
    let manifest: Value = serde_json::from_str(&field(&create.body, "manifest")).unwrap();
    assert!(manifest["/index.html"].is_string() && manifest["/next.html"].is_string());
    assert!(
        manifest
            .as_object()
            .unwrap()
            .keys()
            .all(|p| !p.ends_with(".md"))
    );
    assert_eq!(
        std::fs::read_to_string(r.join("site/previous.txt")).unwrap(),
        "retain"
    );
    assert!(
        std::fs::read_to_string(r.join("content/index.md"))
            .unwrap()
            .contains("[Next](next.md)")
    );
}

#[test]
fn large_asset_batches_are_bounded_and_files_are_not_split() {
    let temp = tempfile::tempdir().unwrap();
    let r = root(&temp);
    write(r, "index.html", "docs");
    write(r, "one.bin", vec![1; 5 * 1024 * 1024]);
    write(r, "two.bin", vec![2; 5 * 1024 * 1024]);
    let server = deployment_server(false, false);
    server.client().deploy(&target(false), r).unwrap();
    let requests = server.requests.lock().unwrap();
    let uploads: Vec<_> = requests
        .iter()
        .filter(|r| r.path.ends_with("/upload"))
        .collect();
    assert_eq!(uploads.len(), 2);
    assert!(uploads.iter().all(|r| r.body.len() < BATCH_BYTES + 1024));
}

#[test]
fn invalid_build_options_stop_before_cloudflare_preflight() {
    let temp = tempfile::tempdir().unwrap();
    let r = root(&temp);
    write(r, "zel.yaml", "version: 1\nrenderer: portable\n");
    write(r, "content/index.md", "# Docs");
    let site = zellij_core::site::Site::load(r).unwrap();
    let server = Server::new(|_, _| panic!("invalid options must not call API"));
    let options = zellij_engine::BuildOptions {
        overrides: zellij_core::build_overrides::BuildOverrides {
            base_path: Some("/prefix".parse().unwrap()),
            ..Default::default()
        },
        ..Default::default()
    };
    assert!(
        crate::delivery::deploy_cloudflare(
            &site,
            &site.content,
            &server.client(),
            "manual",
            false,
            &options
        )
        .is_err()
    );
}

#[test]
#[ignore = "requires an already running container runtime and the checkout's engine image; CI runs it explicitly"]
fn hosted_export_upload_preserves_prefix_assets_and_previous_output() {
    let temp = tempfile::tempdir().unwrap();
    let r = root(&temp);
    write(r, "zel.yaml", "version: 1\nrenderer: hosted\n");
    write(
        r,
        "content/site.yaml",
        "name: Cloudflare acceptance\nhome: guide\nbasePath: /authored\nnav: {search: true}\n",
    );
    write(
        r,
        "content/guide/_sidebar.yaml",
        "title: Manual\nroute: /manual\nnav: [index, next]\n",
    );
    write(
        r,
        "content/guide/index.mdx",
        "# Cloudflare acceptance\n\n[Next](/manual/next)\n\n![Logo](/_zellij/assets/logo.svg)\n",
    );
    write(r, "content/guide/next.mdx", "# Next\n\nMore docs\n");
    write(
        r,
        "content/assets/logo.svg",
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"20\" height=\"20\"/>",
    );
    write(r, "site/previous.txt", "retain");
    let site = zellij_core::site::Site::load(r).unwrap();
    let server = deployment_server(false, false);
    crate::delivery::deploy_cloudflare(
        &site,
        &site.content,
        &server.client(),
        "manual",
        false,
        &zellij_engine::BuildOptions {
            overrides: zellij_core::build_overrides::BuildOverrides {
                base_path: Some("/products/.docs".parse().unwrap()),
                site_url: Some("https://docs.example".parse().unwrap()),
                ..Default::default()
            },
            ..Default::default()
        },
    )
    .unwrap();
    let requests = server.requests.lock().unwrap();
    let create = requests
        .iter()
        .find(|r| r.path.ends_with("/deployments"))
        .unwrap();
    let manifest: BTreeMap<String, String> =
        serde_json::from_str(&field(&create.body, "manifest")).unwrap();
    let mut assets = BTreeMap::new();
    for request in requests.iter().filter(|r| r.path.ends_with("/upload")) {
        for file in serde_json::from_slice::<Vec<Value>>(&request.body).unwrap() {
            assets.insert(
                file["key"].as_str().unwrap().to_owned(),
                STANDARD.decode(file["value"].as_str().unwrap()).unwrap(),
            );
        }
    }
    assert!(manifest.keys().all(|p| p.starts_with("/products/.docs/")));
    assert!(manifest.keys().all(|p| !p.ends_with(".mdx")
        && !p.ends_with("site.yaml")
        && !p.contains(".zellij-output")));
    assert!(
        manifest.contains_key("/products/.docs/index.html")
            && manifest.contains_key("/products/.docs/manual/next.html")
    );
    let manual = String::from_utf8_lossy(&assets[&manifest["/products/.docs/manual.html"]]);
    assert!(
        manual.contains("Cloudflare acceptance")
            && manual.contains("/products/.docs/manual/next")
            && manual.contains("/products/.docs/_zellij/assets/logo.svg")
    );
    assert_eq!(
        assets[&manifest["/products/.docs/_zellij/assets/logo.svg"]],
        std::fs::read(r.join("content/assets/logo.svg")).unwrap()
    );
    assert!(
        manifest
            .keys()
            .any(|p| p.contains("/_next/") && p.ends_with(".js"))
    );
    assert_eq!(
        std::fs::read_to_string(r.join("site/previous.txt")).unwrap(),
        "retain"
    );
    assert_eq!(std::fs::read_dir(r).unwrap().count(), 3);
    let count = requests.len();
    drop(requests);
    write(r, "content/site.yaml", "name: []\n");
    assert!(
        crate::delivery::deploy_cloudflare(
            &site,
            &site.content,
            &server.client(),
            "manual",
            false,
            &Default::default()
        )
        .is_err()
    );
    assert_eq!(
        server.requests.lock().unwrap().len(),
        count + 1,
        "failed build must perform only project preflight"
    );
}
