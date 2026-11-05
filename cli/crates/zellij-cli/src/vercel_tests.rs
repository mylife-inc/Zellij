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
impl Server {
    fn client(&self) -> Client {
        Client::new(
            &self.address,
            "fixture-token",
            Some("team_123"),
            Duration::from_secs(5),
            Duration::from_millis(1),
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
fn project() -> Value {
    json!({"id":"prj_123", "name":"manual", "accountId":"team_123"})
}
fn target(production: bool) -> Target {
    Target {
        project: serde_json::from_value(project()).unwrap(),
        production,
    }
}
fn ready(production: bool) -> Value {
    json!({"id":"dpl_123", "readyState":"READY", "target":if production {Some("production")} else {None}, "projectId":"prj_123", "url":"manual-123.vercel.app", "aliasAssigned":true})
}
fn deployment_server(production: bool) -> Server {
    Server::new(move |r, _| {
        assert_eq!(
            r.headers.get("authorization").unwrap(),
            "Bearer fixture-token"
        );
        assert!(!r.headers.contains_key("cookie"));
        assert!(r.path.contains("teamId=team_123"));
        match (r.method.as_str(), r.path.split('?').next().unwrap()) {
            ("GET", "/v9/projects/manual") | ("GET", "/v9/projects/prj_123") => project().into(),
            ("POST", "/v2/files") => {
                assert_eq!(r.headers["x-vercel-digest"], hash(&r.body));
                assert_eq!(r.headers["content-length"], r.body.len().to_string());
                assert_eq!(r.headers["content-type"], "application/octet-stream");
                json!({"urls":[]}).into()
            }
            ("POST", "/v13/deployments") => {
                assert!(r.path.contains("prebuilt=1"));
                let body: Value = serde_json::from_slice(&r.body).unwrap();
                assert_eq!(body["project"], "prj_123");
                assert_eq!(body["name"], "manual");
                assert_eq!(body["version"], 2);
                assert_eq!(
                    body.get("target").cloned(),
                    if production {
                        Some(json!("production"))
                    } else {
                        None
                    }
                );
                assert!(body.get("gitSource").is_none() && body.get("projectSettings").is_none());
                json!({"id":"dpl_123","readyState":"QUEUED","target":if production {Some("production")} else {None}}).into()
            }
            ("GET", "/v13/deployments/dpl_123") => ready(production).into(),
            _ => panic!("unexpected request {r:?}"),
        }
    })
}
fn uploaded(server: &Server) -> BTreeMap<String, Vec<u8>> {
    let requests = server.requests.lock().unwrap();
    let assets: BTreeMap<_, _> = requests
        .iter()
        .filter(|r| r.path.starts_with("/v2/files?"))
        .map(|r| (r.headers["x-vercel-digest"].clone(), r.body.clone()))
        .collect();
    let create = requests
        .iter()
        .find(|r| r.method == "POST" && r.path.starts_with("/v13/deployments?"))
        .unwrap();
    let body: Value = serde_json::from_slice(&create.body).unwrap();
    body["files"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| {
            let bytes = match v.get("sha") {
                Some(sha) => assets[sha.as_str().unwrap()].clone(),
                None => v["data"].as_str().unwrap().as_bytes().to_vec(),
            };
            (v["file"].as_str().unwrap().to_owned(), bytes)
        })
        .collect()
}
#[test]
fn native_upload_uses_prebuilt_layout_hashes_preview_and_explicit_production() {
    for production in [false, true] {
        let tmp = tempfile::tempdir().unwrap();
        let r = root(&tmp);
        write(r, "index.html", "<h1>Docs</h1>");
        write(r, "guide.html", "<h1>Docs</h1>"); // Deduplicated upload.
        write(r, "日本/100%.svg", [0u8, 1, 255]);
        write(r, ".zellij-output", "ownership");
        let server = deployment_server(production);
        let client = server.client();
        let target = client.check_project("manual", production).unwrap();
        let result = client.deploy(&target, r).unwrap();
        assert_eq!(
            result.url("/docs").as_str(),
            "https://manual-123.vercel.app/docs/"
        );
        let files = uploaded(&server);
        assert_eq!(files[".vercel/output/static/日本/100%.svg"], [0, 1, 255]);
        assert_eq!(files.len(), 4);
        assert!(!files.keys().any(|k| k.contains(".zellij-output")));
        let cfg: Value = serde_json::from_slice(&files[".vercel/output/config.json"]).unwrap();
        assert_eq!(cfg["version"], 3);
        assert_eq!(cfg["routes"][0]["handle"], "filesystem");
        assert!(
            cfg["routes"]
                .as_array()
                .unwrap()
                .iter()
                .skip(1)
                .all(|r| r["check"] == true)
        );
        let requests = server.requests.lock().unwrap();
        assert_eq!(
            requests
                .iter()
                .filter(|r| r.path.starts_with("/v2/files?"))
                .count(),
            2
        );
        assert_eq!(
            requests
                .iter()
                .filter(|r| r.method == "POST" && r.path.starts_with("/v13/deployments?"))
                .count(),
            1
        );
    }
}
#[test]
fn project_identity_and_team_are_checked_before_build() {
    for response in [
        json!({"id":"prj_wrong","name":"other","accountId":"team_123"}),
        json!({"id":"prj_123","name":"manual","accountId":"team_other"}),
        json!({"id":"bad","name":"manual","accountId":"team_123"}),
    ] {
        let server = Server::new(move |_, _| response.clone().into());
        assert!(server.client().check_project("manual", false).is_err());
        assert_eq!(server.requests.lock().unwrap().len(), 1);
    }
}
#[test]
fn errors_are_redacted_redirects_refused_and_creation_not_retried() {
    for status in [301, 401, 403, 404, 429, 500] {
        let server = Server::new(move |_, _| Reply {
            status,
            body: json!({"error":"fixture-token PRIVATE"}),
            redirect: Some("http://127.0.0.1:1/leak".into()),
        });
        let error = server
            .client()
            .check_project("manual", false)
            .err()
            .unwrap()
            .to_string();
        assert!(error.contains(&status.to_string()), "{error}");
        assert!(!error.contains("fixture-token") && !error.contains("PRIVATE"));
        assert_eq!(server.requests.lock().unwrap().len(), 1);
    }
    let tmp = tempfile::tempdir().unwrap();
    write(root(&tmp), "index.html", "Docs");
    let server = Server::new(|r, _| {
        if r.path.starts_with("/v2/files?") {
            json!({}).into()
        } else {
            Reply {
                status: 500,
                body: json!({"error":"PRIVATE"}),
                redirect: None,
            }
        }
    });
    assert!(server.client().deploy(&target(false), root(&tmp)).is_err());
    assert_eq!(server.requests.lock().unwrap().len(), 2);
}
#[test]
fn status_identity_environment_failure_and_url_are_not_silently_accepted() {
    let tmp = tempfile::tempdir().unwrap();
    write(root(&tmp), "index.html", "Docs");
    for (field, value) in [
        ("id", json!("dpl_other")),
        ("projectId", json!("prj_other")),
        ("target", json!("production")),
        ("readyState", json!("ERROR")),
        ("readyState", json!("CANCELED")),
        ("readyState", json!("UNKNOWN")),
        ("errorCode", json!("failed")),
        ("aliasError", json!({"message":"PRIVATE"})),
        ("url", json!("attacker.example")),
        ("url", json!("foo.vercel.app@attacker.example")),
        ("url", json!("foo.vercel.app/?token=PRIVATE")),
    ] {
        let server = Server::new(move |r, _| {
            if r.path.starts_with("/v2/files?") {
                return json!({}).into();
            }
            let mut result = ready(false);
            if r.method == "GET" {
                result[field] = value.clone();
            }
            result.into()
        });
        let e = server
            .client()
            .deploy(&target(false), root(&tmp))
            .err()
            .unwrap()
            .to_string();
        assert!(!e.contains("PRIVATE"), "{e}");
    }
}
#[test]
fn production_waits_for_alias_assignment_and_preview_does_not() {
    for production in [false, true] {
        let tmp = tempfile::tempdir().unwrap();
        write(root(&tmp), "index.html", "Docs");
        let server = Server::new(move |r, n| {
            if r.path.starts_with("/v2/files?") {
                return json!({}).into();
            }
            let mut response = ready(production);
            response["aliasAssigned"] = if n < 3 { json!(false) } else { json!(1234567) };
            response.into()
        });
        server
            .client()
            .deploy(&target(production), root(&tmp))
            .unwrap();
        assert_eq!(
            server.requests.lock().unwrap().len(),
            if production { 4 } else { 3 }
        );
    }
}
#[test]
fn invalid_configuration_and_export_fail_before_any_upload() {
    for token in ["", "bad\ntoken", "bad token"] {
        assert!(Client::new(API, token, None, Duration::ZERO, Duration::ZERO).is_err());
    }
    for team in ["", "team_", "https://example.com", "bad"] {
        assert!(Client::new(API, "token", Some(team), Duration::ZERO, Duration::ZERO).is_err());
    }
    // '#' is a valid filename on every supported OS, but invalid for this
    // upload protocol. Windows rejects '?' before zel can inspect the file.
    for file in ["vercel.json", ".vercel/output/config.json", "bad#.html"] {
        let tmp = tempfile::tempdir().unwrap();
        write(root(&tmp), "index.html", "Docs");
        write(root(&tmp), file, "unsupported");
        let server = Server::new(|_, _| panic!("must not upload"));
        assert!(server.client().deploy(&target(false), root(&tmp)).is_err());
    }
    let tmp = tempfile::tempdir().unwrap();
    let r = root(&tmp);
    write(r, "other.txt", "no home");
    assert!(Manifest::read(r).is_err());
    write(r, "index.html", "Docs");
    std::fs::File::create(r.join("oversized"))
        .unwrap()
        .set_len(MAX_FILE + 1)
        .unwrap();
    assert!(
        Manifest::read(r)
            .err()
            .unwrap()
            .to_string()
            .contains("100 MiB")
    );
}
#[cfg(unix)]
#[test]
fn symlinks_and_special_files_are_rejected() {
    let tmp = tempfile::tempdir().unwrap();
    let r = root(&tmp);
    write(r, "index.html", "Docs");
    std::os::unix::fs::symlink(r.join("index.html"), r.join("link")).unwrap();
    assert!(Manifest::read(r).is_err());
    std::fs::remove_file(r.join("link")).unwrap();
    let _socket = std::os::unix::net::UnixListener::bind(r.join("socket")).unwrap();
    assert!(Manifest::read(r).is_err());
}
#[test]
fn deadline_stops_polling_without_recreating_deployment() {
    let tmp = tempfile::tempdir().unwrap();
    write(root(&tmp), "index.html", "Docs");
    let server = Server::new(|r, _| {
        if r.path.starts_with("/v2/files?") {
            return json!({}).into();
        }
        let mut response = ready(false);
        response["readyState"] = json!("BUILDING");
        response.into()
    });
    let mut client = server.client();
    client.timeout = Duration::from_millis(150);
    client.poll_interval = Duration::from_millis(40);
    assert!(
        client
            .deploy(&target(false), root(&tmp))
            .err()
            .unwrap()
            .to_string()
            .contains("timed out")
    );
    assert_eq!(
        server
            .requests
            .lock()
            .unwrap()
            .iter()
            .filter(|r| r.method == "POST" && r.path.starts_with("/v13/deployments?"))
            .count(),
        1
    );
}
#[test]
fn portable_delivery_preserves_authored_files_and_previous_output() {
    let tmp = tempfile::tempdir().unwrap();
    let r = root(&tmp);
    write(r, "zel.yaml", "version: 1\nrenderer: portable\n");
    write(r, "content/index.md", "# Docs\n\n[Guide](guide.md)\n");
    write(r, "content/guide.md", "# Guide\n");
    write(r, "site/previous.txt", "keep");
    let site = zellij_core::site::Site::load(r).unwrap();
    let server = deployment_server(false);
    crate::delivery::deploy_vercel(
        &site,
        &site.content,
        &server.client(),
        "manual",
        false,
        &Default::default(),
    )
    .unwrap();
    let files = uploaded(&server);
    assert!(
        files.contains_key(".vercel/output/static/index.html")
            && files.contains_key(".vercel/output/static/guide.html")
    );
    assert!(
        String::from_utf8_lossy(&files[".vercel/output/static/index.html"]).contains("guide.html")
    );
    assert_eq!(
        std::fs::read_to_string(r.join("site/previous.txt")).unwrap(),
        "keep"
    );
    assert_eq!(std::fs::read_dir(r).unwrap().count(), 3);
    let count = server.requests.lock().unwrap().len();
    write(r, "content/index.md", "[Broken](missing.md)");
    assert!(
        crate::delivery::deploy_vercel(
            &site,
            &site.content,
            &server.client(),
            "manual",
            false,
            &Default::default()
        )
        .is_err()
    );
    assert_eq!(server.requests.lock().unwrap().len(), count + 1);
    let deny = Server::new(|_, _| Reply {
        status: 404,
        body: json!({}),
        redirect: None,
    });
    let e = crate::delivery::deploy_vercel(
        &site,
        &site.content,
        &deny.client(),
        "manual",
        false,
        &Default::default(),
    )
    .unwrap_err()
    .to_string();
    assert!(e.contains("project preflight") && e.contains("404"));
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
        "name: Vercel acceptance\nhome: guide\nbasePath: /authored\nnav: {search: true}\n",
    );
    write(
        r,
        "content/guide/_sidebar.yaml",
        "title: Manual\nroute: /manual\nnav: [index, next]\n",
    );
    write(
        r,
        "content/guide/index.mdx",
        "# Vercel acceptance\n\n[Next](/manual/next)\n\n![Logo](/_zellij/assets/logo.svg)\n",
    );
    write(r, "content/guide/next.mdx", "# Next\n\nMore docs\n");
    write(
        r,
        "content/assets/logo.svg",
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"20\" height=\"20\"/>",
    );
    write(r, "site/previous.txt", "retain");
    let site = zellij_core::site::Site::load(r).unwrap();
    let server = deployment_server(false);
    crate::delivery::deploy_vercel(
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
    let files = uploaded(&server);
    let assets: BTreeMap<_, _> = files
        .iter()
        .filter_map(|(p, bytes)| {
            p.strip_prefix(".vercel/output/static")
                .map(|p| (p.to_owned(), bytes.clone()))
        })
        .collect();
    let manifest: BTreeMap<_, _> = assets.keys().map(|p| (p.clone(), p.clone())).collect();
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
        manual.contains("Vercel acceptance")
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
    let count = server.requests.lock().unwrap().len();
    write(r, "content/site.yaml", "name: []\n");
    assert!(
        crate::delivery::deploy_vercel(
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

#[test]
fn personal_scope_omits_team_query_and_invalid_options_stop_before_api() {
    let server = Server::new(|r, _| {
        assert_eq!(r.path, "/v9/projects/manual");
        let mut response = project();
        response["accountId"] = json!("personal123");
        response.into()
    });
    let client = Client::new(
        &server.address,
        "fixture-token",
        None,
        Duration::from_secs(5),
        Duration::ZERO,
    )
    .unwrap();
    client.check_project("manual", false).unwrap();
    let tmp = tempfile::tempdir().unwrap();
    let r = root(&tmp);
    write(r, "zel.yaml", "version: 1\nrenderer: portable\n");
    write(r, "content/index.md", "# Docs");
    let site = zellij_core::site::Site::load(r).unwrap();
    let options = zellij_engine::BuildOptions {
        overrides: zellij_core::build_overrides::BuildOverrides {
            base_path: Some("/prefix".parse().unwrap()),
            ..Default::default()
        },
        ..Default::default()
    };
    assert!(
        crate::delivery::deploy_vercel(&site, &site.content, &client, "manual", false, &options)
            .is_err()
    );
    assert_eq!(server.requests.lock().unwrap().len(), 1);
}

#[test]
fn upload_api_error_stops_before_creation_and_large_responses_are_bounded() {
    let tmp = tempfile::tempdir().unwrap();
    write(root(&tmp), "index.html", "Docs");
    let server = Server::new(|r, _| {
        assert!(r.path.starts_with("/v2/files?"));
        json!({"error":{"message":"PRIVATE fixture-token"}}).into()
    });
    let e = server
        .client()
        .deploy(&target(false), root(&tmp))
        .err()
        .unwrap()
        .to_string();
    assert!(e.contains("asset upload") && !e.contains("PRIVATE") && !e.contains("fixture-token"));
    assert_eq!(server.requests.lock().unwrap().len(), 1);
    let oversized = Server::new(|_, _| json!({"large":"x".repeat(RESPONSE_LIMIT as usize)}).into());
    assert!(
        oversized
            .client()
            .check_project("manual", false)
            .err()
            .unwrap()
            .to_string()
            .contains("exceeded 8 MiB")
    );
}
