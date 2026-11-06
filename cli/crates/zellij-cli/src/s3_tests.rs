use super::*;
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
struct Response {
    status: u16,
    body: Vec<u8>,
    headers: BTreeMap<String, String>,
}
impl Response {
    fn new(status: u16, body: impl AsRef<[u8]>) -> Self {
        Self {
            status,
            body: body.as_ref().to_vec(),
            headers: BTreeMap::new(),
        }
    }
    fn header(mut self, name: &str, value: &str) -> Self {
        self.headers.insert(name.into(), value.into());
        self
    }
}
struct Server {
    address: String,
    requests: Arc<Mutex<Vec<Request>>>,
    stop: Arc<AtomicBool>,
    worker: Option<thread::JoinHandle<()>>,
}
impl Server {
    fn new(handler: impl Fn(&Request, usize) -> Response + Send + 'static) -> Self {
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
                let is_head = request.method == "HEAD";
                record.lock().unwrap().push(request);
                let headers = reply
                    .headers
                    .iter()
                    .map(|(k, v)| format!("{k}: {v}\r\n"))
                    .collect::<String>();
                let _ = write!(
                    socket,
                    "HTTP/1.1 {} Fixture\r\nContent-Length: {}\r\nConnection: close\r\n{}\r\n",
                    reply.status,
                    reply.body.len(),
                    headers
                );
                if !is_head {
                    let _ = socket.write_all(&reply.body);
                }
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
const WEBSITE: &str = "<WebsiteConfiguration xmlns=\"http://s3.amazonaws.com/doc/2006-03-01/\"><IndexDocument><Suffix>index.html</Suffix></IndexDocument></WebsiteConfiguration>";
const DEST: &str = "acme-docs/manual";
fn credentials() -> Credentials {
    // Public AWS documentation signing example, never live credentials.
    Credentials::new(
        "AKIAIOSFODNN7EXAMPLE",
        "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY",
        None,
    )
    .unwrap()
}
impl Server {
    fn client(&self) -> Client {
        Client::new(
            &self.address,
            "us-east-1",
            "123456789012",
            credentials(),
            Duration::from_secs(30),
        )
        .unwrap()
    }
}
fn deployment_server(
    change: impl Fn(&Request, usize) -> Option<Response> + Send + 'static,
) -> Server {
    Server::new(move |request, n| {
        assert_eq!(
            request.headers["x-amz-expected-bucket-owner"],
            "123456789012"
        );
        assert_eq!(request.headers["x-amz-content-sha256"], hash(&request.body));
        assert!(!request.headers.contains_key("x-amz-acl"));
        // Reconstruct the signed request independently of the request builder.
        let url = Url::parse(&format!(
            "http://{}{}",
            request.headers["host"], request.path
        ))
        .unwrap();
        let auth = &request.headers["authorization"];
        let signed = auth
            .split("SignedHeaders=")
            .nth(1)
            .unwrap()
            .split(',')
            .next()
            .unwrap();
        let headers = signed
            .split(';')
            .map(|k| (k.to_owned(), request.headers[k].clone()))
            .collect();
        assert_eq!(
            *auth,
            credentials().authorization(
                "us-east-1",
                &request.headers["x-amz-date"],
                &request.method.parse().unwrap(),
                &url,
                &headers
            )
        );
        if let Some(reply) = change(request, n) {
            return reply;
        }
        match (request.method.as_str(), request.path.as_str()) {
            ("HEAD", "/acme-docs") => {
                Response::new(200, "").header("x-amz-bucket-region", "us-east-1")
            }
            ("GET", "/acme-docs?website=") => Response::new(200, WEBSITE),
            ("HEAD", _) => Response::new(404, ""),
            ("PUT", _) => {
                let checksum = STANDARD.encode(Sha256::digest(&request.body));
                assert_eq!(request.headers["x-amz-checksum-sha256"], checksum);
                assert_eq!(request.headers["cache-control"], "no-cache");
                Response::new(200, "")
                    .header("etag", "\"new-etag\"")
                    .header("x-amz-checksum-sha256", &checksum)
            }
            _ => panic!("unexpected request {} {}", request.method, request.path),
        }
    })
}
fn root(temp: &tempfile::TempDir) -> &Utf8Path {
    Utf8Path::from_path(temp.path()).unwrap()
}
fn write(root: &Utf8Path, path: &str, data: impl AsRef<[u8]>) {
    let path = root.join(path);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, data).unwrap();
}
fn target() -> Target {
    Target {
        bucket: "acme-docs".into(),
        mount: "/manual".into(),
        website: website(WEBSITE.as_bytes()).unwrap(),
        url: "http://acme-docs.s3-website-us-east-1.amazonaws.com/manual/"
            .parse()
            .unwrap(),
    }
}
fn uploaded(server: &Server) -> BTreeMap<String, Vec<u8>> {
    server
        .requests
        .lock()
        .unwrap()
        .iter()
        .filter(|r| r.method == "PUT")
        .map(|r| {
            (
                r.path.strip_prefix("/acme-docs/").unwrap().to_owned(),
                r.body.clone(),
            )
        })
        .collect()
}
#[test]
fn signing_matches_three_published_aws_vectors() {
    // https://docs.aws.amazon.com/AmazonS3/latest/developerguide/sig-v4-header-based-auth.html
    let date = "20130524T000000Z";
    for (method, path, body, extra, signature) in [
        (
            Method::GET,
            "/test.txt",
            "",
            vec![("range", "bytes=0-9")],
            "f0e8bdb87c964420e857bd35b5d6ed310bd44f0170aba48dd91039c6036bdb41",
        ),
        (
            Method::PUT,
            "/test%24file.text",
            "Welcome to Amazon S3.",
            vec![
                ("date", "Fri, 24 May 2013 00:00:00 GMT"),
                ("x-amz-storage-class", "REDUCED_REDUNDANCY"),
            ],
            "98ad721746da40c64f1a55b78f14c238d841ea1380cd77a1b5971af0ece108bd",
        ),
        (
            Method::GET,
            "/?lifecycle=",
            "",
            vec![],
            "fea454ca298b7da1c68078a5d1bdbfbbe0d65c699e0f91ac7a200a0136783543",
        ),
    ] {
        let url = Url::parse(&format!("https://examplebucket.s3.amazonaws.com{path}")).unwrap();
        let mut headers = BTreeMap::from([
            ("host".into(), "examplebucket.s3.amazonaws.com".into()),
            ("x-amz-date".into(), date.into()),
            ("x-amz-content-sha256".into(), hash(body.as_bytes())),
        ]);
        headers.extend(extra.into_iter().map(|(k, v)| (k.into(), v.into())));
        assert!(
            credentials()
                .authorization("us-east-1", date, &method, &url, &headers)
                .ends_with(signature)
        );
    }
}
#[test]
fn temporary_credentials_require_a_session_and_sign_it() {
    assert!(Credentials::new("ASIAIOSFODNN7EXAMPLE", "secret", None).is_err());
    assert!(Credentials::new("bad key", "secret", None).is_err());
    assert!(Credentials::new("AKIAIOSFODNN7EXAMPLE", "secret\n", None).is_err());
    assert!(Credentials::new("ASIAIOSFODNN7EXAMPLE", "secret", Some("bad\ntoken")).is_err());
    let server = Server::new(|r, _| {
        assert_eq!(r.headers["x-amz-security-token"], "session-fixture");
        assert!(r.headers["authorization"].contains(";x-amz-security-token"));
        Response::new(403, "PRIVATE session-fixture")
    });
    let mut client = server.client();
    client.credentials =
        Credentials::new("ASIAIOSFODNN7EXAMPLE", "secret", Some("session-fixture")).unwrap();
    let e = client.check_site(DEST).err().unwrap().to_string();
    assert!(e.contains("403") && !e.contains("PRIVATE") && !e.contains("session-fixture"));
}
#[test]
fn destination_region_owner_and_xml_are_validated() {
    for good in ["abc", "acme.docs", "acme-docs/manual/v1.0_2"] {
        Provider::S3.validate_site(good).unwrap();
    }
    for bad in [
        "",
        "ab",
        "UPPER",
        "https://bucket",
        "s3://bucket",
        "127.0.0.1",
        "a..b",
        "-abc",
        "abc-",
        "xn--bucket",
        "bucket--x-s3",
        "bucket-s3alias",
        "bucket/",
        "bucket/../x",
        "bucket/a//b",
        "bucket/bad name",
        "bucket/é",
    ] {
        assert!(Provider::S3.validate_site(bad).is_err(), "{bad}");
    }
    assert_eq!(
        website_endpoint("us-east-1").unwrap(),
        "s3-website-us-east-1.amazonaws.com"
    );
    assert_eq!(
        website_endpoint("eu-central-1").unwrap(),
        "s3-website.eu-central-1.amazonaws.com"
    );
    for region in ["us-gov-west-1", "cn-north-1", "https://evil", "us-east-99"] {
        assert!(website_endpoint(region).is_err());
    }
    assert!(
        Client::new(
            "http://localhost",
            "us-east-1",
            "wrong-owner",
            credentials(),
            Duration::from_secs(1)
        )
        .is_err()
    );
    website(WEBSITE.as_bytes()).unwrap();
    for xml in [
        "<WebsiteConfiguration/>",
        "<WebsiteConfiguration><IndexDocument><Suffix>other.html</Suffix></IndexDocument></WebsiteConfiguration>",
        "<WebsiteConfiguration><RedirectAllRequestsTo><HostName>elsewhere</HostName></RedirectAllRequestsTo></WebsiteConfiguration>",
        "<!DOCTYPE x [<!ENTITY private SYSTEM 'file:///no'>]><WebsiteConfiguration>&private;</WebsiteConfiguration>",
    ] {
        assert!(website(xml.as_bytes()).is_err(), "{xml}");
    }
    for replacement in [
        "<RoutingRules><RoutingRule/></RoutingRules>",
        "<IndexDocument><Suffix>index.html</Suffix></IndexDocument>",
        "<Foreign xmlns=\"other\"/>",
    ] {
        assert!(
            website(
                WEBSITE
                    .replace(
                        "</WebsiteConfiguration>",
                        &format!("{replacement}</WebsiteConfiguration>")
                    )
                    .as_bytes()
            )
            .is_err()
        );
    }
}
#[test]
fn portable_upload_signs_encoded_binary_keys_and_conditions_without_deleting() {
    let tmp = tempfile::tempdir().unwrap();
    let r = root(&tmp);
    write(r, "index.html", "<a href=\"guide.html\">Docs</a>");
    write(r, "guide.html", "Guide");
    write(r, "assets/日本語 space%.bin", [0, 255, 1, 128]);
    write(r, ".zellij-output", "marker");
    let server = deployment_server(|request, _| {
        (request.method == "HEAD" && request.path == "/acme-docs/manual/index.html")
            .then(|| Response::new(200, "").header("etag", "\"old-etag\""))
    });
    let client = server.client();
    let t = client.check_site(DEST).unwrap();
    let result = client.deploy(&t, r, Renderer::Portable, "").unwrap();
    assert_eq!(result.objects, 3);
    assert_eq!(result.url.as_str(), t.url.as_str());
    let requests = server.requests.lock().unwrap();
    let puts = requests
        .iter()
        .filter(|q| q.method == "PUT")
        .collect::<Vec<_>>();
    assert_eq!(
        puts[0].path,
        "/acme-docs/manual/assets/%E6%97%A5%E6%9C%AC%E8%AA%9E%20space%25.bin"
    );
    assert_eq!(puts[0].body, [0, 255, 1, 128]);
    assert_eq!(puts[0].headers["if-none-match"], "*");
    assert_eq!(puts[0].headers["content-type"], "application/octet-stream");
    assert_eq!(puts.last().unwrap().path, "/acme-docs/manual/index.html");
    assert_eq!(puts.last().unwrap().headers["if-match"], "\"old-etag\"");
    assert_eq!(puts.last().unwrap().headers["content-type"], "text/html");
    assert!(
        requests
            .iter()
            .all(|q| matches!(q.method.as_str(), "GET" | "HEAD" | "PUT"))
    );
    assert!(requests.iter().all(|q| !q.path.contains(".zellij-output")));
    let first = requests.iter().position(|q| q.method == "PUT").unwrap();
    assert_eq!(
        requests[..first]
            .iter()
            .filter(|q| q.method == "HEAD" && q.path.starts_with("/acme-docs/manual/"))
            .count(),
        3
    );
}
#[test]
fn hosted_clean_routes_keep_prefix_once_and_defaults_do_not_edit_authored_options() {
    let t = target();
    let original = BuildOptions::default();
    let opts = t.build_options(Renderer::Hosted, &original).unwrap();
    assert_eq!(opts.overrides.base_path.unwrap().to_string(), "/manual");
    assert_eq!(
        opts.overrides.site_url.unwrap().to_string(),
        "http://acme-docs.s3-website-us-east-1.amazonaws.com"
    );
    assert!(original.overrides.base_path.is_none());
    for prefix in ["", "/other", "/manual-other"] {
        let mut o = BuildOptions::default();
        o.overrides.base_path = Some(prefix.parse().unwrap());
        assert!(t.build_options(Renderer::Hosted, &o).is_err());
    }
    let mut o = BuildOptions::default();
    o.overrides.base_path = Some("/manual/sub".parse().unwrap());
    o.overrides.site_url = Some("https://docs.example".parse().unwrap());
    assert_eq!(
        t.build_options(Renderer::Hosted, &o)
            .unwrap()
            .overrides
            .site_url
            .unwrap()
            .to_string(),
        "https://docs.example"
    );
    let tmp = tempfile::tempdir().unwrap();
    let r = root(&tmp);
    write(r, "manual/index.html", "Home");
    write(r, "manual/guide.html", "Guide");
    write(r, "manual/guide/next.html", "Next");
    write(r, "manual/_next/app.js", "JS");
    let server = deployment_server(|_, _| None);
    let result = server
        .client()
        .deploy(&t, r, Renderer::Hosted, "/manual")
        .unwrap();
    assert_eq!(result.objects, 6);
    let files = uploaded(&server);
    assert_eq!(files["manual/guide"], files["manual/guide.html"]);
    assert_eq!(files["manual/guide/next"], files["manual/guide/next.html"]);
    assert!(
        files
            .keys()
            .all(|k| !k.starts_with("manual/manual/") && k.starts_with("manual/"))
    );
    let mut t = target();
    t.mount.clear();
    t.url.set_path("/");
    let tmp = tempfile::tempdir().unwrap();
    write(root(&tmp), "index.html", "Root");
    assert_eq!(
        Manifest::read(root(&tmp), &t, Renderer::Hosted, "")
            .unwrap()
            .assets[0]
            .key,
        "index.html"
    );
}
#[test]
fn preflight_and_config_drift_refuse_all_writes() {
    for mode in 0..6 {
        let server = deployment_server(move |q, n| match mode {
            0 if n == 0 => Some(Response::new(403, "PRIVATE")),
            1 if n == 0 => Some(Response::new(200, "").header("x-amz-bucket-region", "eu-west-1")),
            2 if q.method == "GET" => Some(Response::new(200, "<WebsiteConfiguration/>")),
            3 if q.method == "HEAD" && q.path.ends_with("index.html") => {
                Some(Response::new(200, "").header("etag", "bad"))
            }
            4 if q.method == "GET" && n > 1 => Some(Response::new(
                200,
                WEBSITE.replace(
                    "</WebsiteConfiguration>",
                    "<ErrorDocument><Key>404.html</Key></ErrorDocument></WebsiteConfiguration>",
                ),
            )),
            5 if q.method == "HEAD" && q.path.ends_with("index.html") => {
                Some(Response::new(403, "PRIVATE"))
            }
            _ => None,
        });
        let tmp = tempfile::tempdir().unwrap();
        write(root(&tmp), "index.html", "Home");
        let client = server.client();
        let result = client
            .check_site(DEST)
            .and_then(|t| client.deploy(&t, root(&tmp), Renderer::Portable, ""));
        assert!(result.is_err(), "mode {mode}");
        assert!(!result.err().unwrap().to_string().contains("PRIVATE"));
        assert!(uploaded(&server).is_empty());
    }
}
#[test]
fn concurrent_writes_bad_receipts_and_partial_failures_are_not_retried() {
    for mode in 0..4 {
        let server = deployment_server(move |q, _| {
            if q.method != "PUT" {
                return None;
            }
            Some(match mode {
                0 => Response::new(412, "PRIVATE concurrent"),
                1 => Response::new(200, "")
                    .header("etag", "\"ok\"")
                    .header("x-amz-checksum-sha256", "wrong"),
                2 => Response::new(200, "").header(
                    "x-amz-checksum-sha256",
                    &STANDARD.encode(Sha256::digest(&q.body)),
                ),
                _ => Response::new(503, "PRIVATE retry-later"),
            })
        });
        let tmp = tempfile::tempdir().unwrap();
        write(root(&tmp), "index.html", "Home");
        write(root(&tmp), "app.js", "JS");
        let err = server
            .client()
            .deploy(&target(), root(&tmp), Renderer::Portable, "")
            .err()
            .unwrap()
            .to_string();
        assert!(!err.contains("PRIVATE"));
        assert_eq!(uploaded(&server).len(), 1);
        assert!(!uploaded(&server).contains_key("manual/index.html"));
        assert!(err.contains("partial") || err.contains("already have changed"));
    }
}
#[test]
fn redirects_response_limits_and_deadlines_are_bounded_and_redacted() {
    let sink = Server::new(|_, _| panic!("redirect followed"));
    let location = sink.address.clone();
    let server =
        Server::new(move |_, _| Response::new(307, "PRIVATE").header("location", &location));
    let e = server.client().check_site(DEST).err().unwrap().to_string();
    assert!(e.contains("redirect") && !e.contains("PRIVATE"));
    assert!(sink.requests.lock().unwrap().is_empty());
    let server = deployment_server(|q, _| {
        (q.method == "GET").then(|| Response::new(200, vec![b'x'; RESPONSE_LIMIT as usize + 1]))
    });
    assert!(
        server
            .client()
            .check_site(DEST)
            .err()
            .unwrap()
            .to_string()
            .contains("256 KiB")
    );
    let server = Server::new(|_, _| {
        thread::sleep(Duration::from_millis(200));
        Response::new(200, "PRIVATE")
    });
    let mut client = server.client();
    client.timeout = Duration::from_millis(50);
    let e = client.check_site(DEST).err().unwrap().to_string();
    assert!(e.contains("timed out") && !e.contains("PRIVATE"));
}
#[test]
fn manifest_rejects_collisions_missing_index_and_oversized_files_before_upload() {
    let tmp = tempfile::tempdir().unwrap();
    let r = root(&tmp);
    write(r, "manual/index.html", "Home");
    write(r, "manual/guide.html", "Guide");
    write(r, "manual/guide", "collision");
    assert!(
        Manifest::read(r, &target(), Renderer::Hosted, "/manual")
            .err()
            .unwrap()
            .to_string()
            .contains("collides")
    );
    std::fs::remove_file(r.join("manual/guide")).unwrap();
    write(r, "outside.txt", "Wrong mount");
    assert!(Manifest::read(r, &target(), Renderer::Hosted, "/manual").is_err());
    std::fs::remove_file(r.join("outside.txt")).unwrap();
    std::fs::remove_file(r.join("manual/index.html")).unwrap();
    assert!(Manifest::read(r, &target(), Renderer::Hosted, "/manual").is_err());
    write(r, "manual/index.html", "Home");
    let file = std::fs::File::create(r.join("manual/large.bin")).unwrap();
    file.set_len(MAX_FILE + 1).unwrap();
    assert!(
        Manifest::read(r, &target(), Renderer::Hosted, "/manual")
            .err()
            .unwrap()
            .to_string()
            .contains("25 MiB")
    );
}
#[cfg(unix)]
#[test]
fn manifest_rejects_links_and_special_files() {
    let tmp = tempfile::tempdir().unwrap();
    let r = root(&tmp);
    write(r, "index.html", "Home");
    std::os::unix::fs::symlink(r.join("index.html"), r.join("link")).unwrap();
    assert!(Manifest::read(r, &target(), Renderer::Portable, "").is_err());
    std::fs::remove_file(r.join("link")).unwrap();
    let _socket = std::os::unix::net::UnixListener::bind(r.join("socket")).unwrap();
    assert!(Manifest::read(r, &target(), Renderer::Portable, "").is_err());
}
#[test]
fn portable_delivery_preserves_source_and_output_and_preflights_before_build() {
    let tmp = tempfile::tempdir().unwrap();
    let r = root(&tmp);
    write(r, "zel.yaml", "version: 1\nrenderer: portable\n");
    write(r, "content/index.md", "# Docs\n\n[Guide](guide.md)\n");
    write(r, "content/guide.md", "# Guide\n");
    write(r, "site/previous.txt", "keep");
    let site = zellij_core::site::Site::load(r).unwrap();
    let server = deployment_server(|_, _| None);
    crate::delivery::deploy_s3(
        &site,
        &site.content,
        &server.client(),
        DEST,
        &Default::default(),
    )
    .unwrap();
    let files = uploaded(&server);
    assert!(String::from_utf8_lossy(&files["manual/index.html"]).contains("guide.html"));
    assert!(files.contains_key("manual/guide.html"));
    assert_eq!(
        std::fs::read_to_string(r.join("site/previous.txt")).unwrap(),
        "keep"
    );
    assert_eq!(std::fs::read_dir(r).unwrap().count(), 3);
    let count = server.requests.lock().unwrap().len();
    write(r, "content/index.md", "[Broken](missing.md)");
    assert!(
        crate::delivery::deploy_s3(
            &site,
            &site.content,
            &server.client(),
            DEST,
            &Default::default()
        )
        .is_err()
    );
    assert_eq!(server.requests.lock().unwrap().len(), count + 2);
    let mut options = BuildOptions::default();
    options.overrides.base_path = Some("/manual".parse().unwrap());
    assert!(
        crate::delivery::deploy_s3(&site, &site.content, &server.client(), DEST, &options).is_err()
    );
    assert_eq!(server.requests.lock().unwrap().len(), count + 2);
}

#[test]
#[ignore = "requires an already running container runtime and the checkout's engine image; CI runs it explicitly"]
fn hosted_export_uses_s3_mount_and_preserves_assets_source_and_previous_output() {
    let temp = tempfile::tempdir().unwrap();
    let r = root(&temp);
    write(r, "zel.yaml", "version: 1\nrenderer: hosted\n");
    let authored = "name: S3 acceptance\nhome: guide\nbasePath: /authored\nnav: {search: true}\n";
    write(r, "content/site.yaml", authored);
    write(
        r,
        "content/guide/_sidebar.yaml",
        "title: Manual\nroute: /guide\nnav: [index, next]\n",
    );
    write(
        r,
        "content/guide/index.mdx",
        "# S3 acceptance\n\n[Next](/guide/next)\n\n![Logo](/_zellij/assets/logo.svg)\n",
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
    crate::delivery::deploy_s3(
        &site,
        &site.content,
        &server.client(),
        DEST,
        &Default::default(),
    )
    .unwrap();
    let files = uploaded(&server);
    assert!(
        files.contains_key("manual/index.html") && files.contains_key("manual/guide/next.html")
    );
    assert!(files.keys().all(|p| p.starts_with("manual/")
        && !p.starts_with("manual/manual/")
        && !p.ends_with(".mdx")
        && !p.ends_with("site.yaml")
        && !p.contains(".zellij-output")));
    assert_eq!(files["manual/guide"], files["manual/guide.html"]);
    assert_eq!(files["manual/guide/next"], files["manual/guide/next.html"]);
    let guide = String::from_utf8_lossy(&files["manual/guide.html"]);
    assert!(
        guide.contains("S3 acceptance")
            && guide.contains("/manual/guide/next")
            && guide.contains("/manual/_zellij/assets/logo.svg")
    );
    assert_eq!(
        files["manual/_zellij/assets/logo.svg"],
        std::fs::read(r.join("content/assets/logo.svg")).unwrap()
    );
    assert!(
        files
            .keys()
            .any(|p| p.starts_with("manual/_next/") && p.ends_with(".js"))
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
        crate::delivery::deploy_s3(
            &site,
            &site.content,
            &server.client(),
            DEST,
            &Default::default()
        )
        .is_err()
    );
    assert_eq!(
        server.requests.lock().unwrap().len(),
        count + 2,
        "failed build performs only bucket and website preflight"
    );
}

#[test]
fn manifest_counts_clean_route_copies_toward_object_and_total_limits() {
    let tmp = tempfile::tempdir().unwrap();
    let r = root(&tmp);
    write(r, "manual/index.html", "Home");
    for n in 0..2_500 {
        write(r, &format!("manual/page{n}.html"), "");
    }
    let err = Manifest::read(r, &target(), Renderer::Hosted, "/manual")
        .err()
        .unwrap()
        .to_string();
    assert!(err.contains("5,000"), "{err}");
    let tmp = tempfile::tempdir().unwrap();
    let r = root(&tmp);
    write(r, "manual/index.html", "Home");
    for n in 0..2 {
        let path = r.join(format!("manual/large{n}.html"));
        let f = std::fs::File::create(path).unwrap();
        f.set_len(MAX_FILE).unwrap();
    }
    let err = Manifest::read(r, &target(), Renderer::Hosted, "/manual")
        .err()
        .unwrap()
        .to_string();
    assert!(err.contains("100 MiB"), "{err}");
}
#[test]
fn post_upload_configuration_drift_cannot_report_success() {
    let checks = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let count = checks.clone();
    let server = deployment_server(move |q, _| {
        if q.method == "GET" && count.fetch_add(1, Ordering::SeqCst) > 0 {
            Some(Response::new(
                200,
                WEBSITE.replace(
                    "</WebsiteConfiguration>",
                    "<ErrorDocument><Key>404.html</Key></ErrorDocument></WebsiteConfiguration>",
                ),
            ))
        } else {
            None
        }
    });
    let tmp = tempfile::tempdir().unwrap();
    write(root(&tmp), "index.html", "Home");
    let e = server
        .client()
        .deploy(&target(), root(&tmp), Renderer::Portable, "")
        .err()
        .unwrap()
        .to_string();
    assert!(e.contains("configuration changed while uploading"));
    assert_eq!(uploaded(&server).len(), 1);
}
#[test]
fn response_body_timeout_is_redacted() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = format!("http://{}/", listener.local_addr().unwrap());
    let headers_sent = Arc::new(AtomicBool::new(false));
    let sent = headers_sent.clone();
    let worker = thread::spawn(move || {
        let (mut socket, _) = listener.accept().unwrap();
        socket
            .set_read_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        assert_eq!(read_request(&socket).unwrap().unwrap().method, "GET");
        write!(
            socket,
            "HTTP/1.1 200 Fixture\r\nContent-Length: 7\r\nConnection: close\r\n\r\n"
        )
        .unwrap();
        socket.flush().unwrap();
        sent.store(true, Ordering::SeqCst);
        thread::sleep(Duration::from_millis(1250));
        let _ = socket.write_all(b"PRIVATE");
    });
    let client = Client::new(
        &address,
        "us-east-1",
        "123456789012",
        credentials(),
        Duration::from_secs(30),
    )
    .unwrap();
    let e = client
        .request(
            Method::GET,
            "/acme-docs",
            vec![],
            BTreeMap::new(),
            Instant::now() + Duration::from_secs(1),
        )
        .err()
        .unwrap()
        .to_string();
    assert!(e.contains("timed out") && !e.contains("PRIVATE"), "{e}");
    worker.join().unwrap();
    assert!(headers_sent.load(Ordering::SeqCst));
}
