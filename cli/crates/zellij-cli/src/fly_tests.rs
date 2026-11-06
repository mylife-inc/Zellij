use super::*;
use serde_json::{Value, json};
use std::collections::BTreeMap;
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

const BEFORE: &str = "2026-11-05T12:00:00Z";
const AFTER: &str = "2026-11-05T12:01:00Z";
fn image() -> String {
    format!("registry.fly.io/manual@sha256:{}", "a".repeat(64))
}
fn machine(id: &str) -> Value {
    json!({"id":id,"instance_id":format!("v1-{id}"),"name":format!("web-{id}"),"region":"iad","state":"started","host_status":"ok","cordoned":false,"updated_at":BEFORE,
        "config":{"image":"registry.fly.io/manual:old","env":{"KEEP":"unchanged"},"guest":{"cpu_kind":"shared","cpus":1,"memory_mb":256},"metadata":{"fly_process_group":"app","custom":"keep"},"restart":{"policy":"always"},"services":[{"protocol":"tcp","internal_port":3000,"ports":[{"port":443,"handlers":["tls","http"]}],"checks":[{"type":"http","port":3000,"path":"/"}]}]},
        "checks":[{"name":"servicecheck-00-http-3000","status":"passing","updated_at":BEFORE}],"image_ref":{"digest":format!("sha256:{}","b".repeat(64))}})
}
#[derive(Clone, Copy, Default)]
enum Fault {
    #[default]
    None,
    LeaseSecond,
    UpdateSecond,
    WrongReceipt,
    StaleCheck,
    MissingCheck,
    WrongDigest,
    Release,
    Drift,
    WrongOrg,
    Redirect,
}
fn fixture(fault: Fault) -> (Server, Arc<Mutex<Vec<Value>>>) {
    let machines = Arc::new(Mutex::new(vec![machine("a1"), machine("b2")]));
    let state = machines.clone();
    let server = Server::new(move |request, _| {
        assert_eq!(request.headers["authorization"], "FlyV1 fm2_fixture");
        let error = || Reply {
            status: 409,
            body: json!({"error":"secret-provider-output fm2_fixture"}),
            redirect: None,
        };
        if matches!(fault, Fault::Redirect) {
            return Reply {
                status: 307,
                body: json!({}),
                redirect: Some("http://127.0.0.1:1/leak".into()),
            };
        }
        if request.path == "/apps/manual" {
            return json!({"id":"app123","name":"manual","organization":{"slug":if matches!(fault,Fault::WrongOrg){"other"}else{"team"}}}).into();
        }
        let mut machines = state.lock().unwrap();
        if request.path == "/apps/manual/machines" {
            return json!(machines.clone()).into();
        }
        let id = request
            .path
            .trim_start_matches("/apps/manual/machines/")
            .split('/')
            .next()
            .unwrap();
        let m = machines.iter_mut().find(|m| m["id"] == id).unwrap();
        if request.path.ends_with("/lease") {
            if request.method == "DELETE" {
                assert_eq!(
                    request.headers["fly-machine-lease-nonce"],
                    format!("lease-{id}")
                );
                return if matches!(fault, Fault::Release) {
                    error()
                } else {
                    json!({"status":"success","data":{"ok":true}}).into()
                };
            }
            assert_eq!(request.method, "POST");
            let body: Value = serde_json::from_slice(&request.body).unwrap();
            assert_eq!(body["ttl"], 600);
            if matches!(fault, Fault::LeaseSecond) && id == "b2" {
                return error();
            }
            if matches!(fault, Fault::Drift) {
                m["instance_id"] = json!("competing-version");
            }
            return json!({"status":"success","data":{"nonce":format!("lease-{id}"),"expires_at":SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()+600}}).into();
        }
        if request.method == "POST" {
            assert_eq!(
                request.headers["fly-machine-lease-nonce"],
                format!("lease-{id}")
            );
            let body: Value = serde_json::from_slice(&request.body).unwrap();
            assert_eq!(body["current_version"], m["instance_id"]);
            assert_eq!(body["skip_launch"], false);
            let mut expected = m["config"].clone();
            expected["image"] = json!(image());
            assert_eq!(body["config"], expected, "preserve every non-image setting");
            if matches!(fault, Fault::UpdateSecond) && id == "b2" {
                return error();
            }
            m["config"] = expected;
            m["instance_id"] = json!(format!("v2-{id}"));
            m["updated_at"] = json!(AFTER);
            if !matches!(fault, Fault::StaleCheck) {
                m["checks"][0]["updated_at"] = json!(AFTER);
            }
            if matches!(fault, Fault::MissingCheck) {
                m["checks"] = json!([]);
            }
            if !matches!(fault, Fault::WrongDigest) {
                m["image_ref"]["digest"] = json!(image().split_once('@').unwrap().1);
            }
            let mut receipt = m.clone();
            if matches!(fault, Fault::WrongReceipt) {
                receipt["instance_id"] = json!("v1-a1");
            }
            return receipt.into();
        }
        assert_eq!(request.method, "GET");
        m.clone().into()
    });
    (server, machines)
}
impl Server {
    fn client(&self) -> Client {
        let mut client =
            Client::new("FlyV1 fm2_fixture", "team".into(), self.address.clone()).unwrap();
        client.budget = Duration::from_secs(5);
        client.interval = Duration::from_millis(1);
        client
    }
    fn mutations(&self, method: &str, lease: bool) -> usize {
        self.requests
            .lock()
            .unwrap()
            .iter()
            .filter(|r| r.method == method && r.path.ends_with("/lease") == lease)
            .count()
    }
}
#[test]
fn rollout_preserves_configuration_and_releases_all_leases() {
    let (server, machines) = fixture(Fault::None);
    let client = server.client();
    let target = client.check("manual").unwrap();
    client.rollout(&target, &image()).unwrap();
    assert!(
        machines
            .lock()
            .unwrap()
            .iter()
            .all(|m| m["config"]["image"] == image())
    );
    assert_eq!(server.mutations("POST", false), 2);
    assert_eq!(server.mutations("POST", true), 2);
    assert_eq!(server.mutations("DELETE", true), 2);
}
#[test]
fn lease_conflicts_and_version_drift_do_not_update_any_machine() {
    for fault in [Fault::LeaseSecond, Fault::Drift] {
        let (server, _) = fixture(fault);
        let client = server.client();
        let target = client.check("manual").unwrap();
        assert!(client.rollout(&target, &image()).is_err());
        assert_eq!(server.mutations("POST", false), 0);
        assert_eq!(server.mutations("DELETE", true), 1);
    }
}
#[test]
fn failed_or_unhealthy_rollouts_stop_and_clean_up_without_retry() {
    for fault in [
        Fault::UpdateSecond,
        Fault::WrongReceipt,
        Fault::StaleCheck,
        Fault::MissingCheck,
        Fault::WrongDigest,
        Fault::Release,
    ] {
        let (server, _) = fixture(fault);
        let mut client = server.client();
        let target = client.check("manual").unwrap();
        client.budget = Duration::from_secs(3);
        let error = client.rollout(&target, &image()).unwrap_err().to_string();
        assert!(
            !error.contains("secret-provider-output") && !error.contains("fm2_fixture"),
            "{error}"
        );
        assert_eq!(server.mutations("DELETE", true), 2, "{error}");
        assert!(server.mutations("POST", false) <= 2);
        if !matches!(fault, Fault::Release) {
            assert!(error.contains("non-atomic"), "{error}");
        }
    }
}
#[test]
fn preflight_rejects_other_accounts_redirects_and_incompatible_apps() {
    for fault in [Fault::WrongOrg, Fault::Redirect] {
        let (server, _) = fixture(fault);
        assert!(server.client().check("manual").is_err());
        assert_eq!(server.requests.lock().unwrap().len(), 1);
    }
    let mut cases = Vec::new();
    for (key, value) in [
        ("state", json!("stopped")),
        ("host_status", json!("unknown")),
        ("cordoned", json!(true)),
        ("checks", json!([])),
    ] {
        let mut m = machine("a1");
        m[key] = value;
        cases.push(m);
    }
    for (key, value) in [
        ("mounts", json!([{"volume":"vol"}])),
        ("init", json!({"cmd":["other"]})),
        ("containers", json!([{}])),
        ("services", json!([])),
    ] {
        let mut m = machine("a1");
        m["config"][key] = value;
        cases.push(m);
    }
    let mut m = machine("a1");
    m["config"]["services"][0]["internal_port"] = json!(8080);
    cases.push(m);
    let mut m = machine("a1");
    m["config"]["services"][0]["autostop"] = json!("stop");
    cases.push(m);
    let mut m = machine("a1");
    m["config"]["env"]["PORT"] = json!("8080");
    cases.push(m);
    for m in cases {
        assert!(validate_machine(&m).is_err(), "{m}");
    }
}
#[test]
fn digest_and_token_validation_never_echo_credentials() {
    for value in ["fm2_test", "FlyV1 fm2_test", "Bearer fm2_test"] {
        assert_eq!(token(value).unwrap(), ("fm2_test", "FlyV1 fm2_test".into()));
    }
    assert_eq!(token("legacy").unwrap().1, "Bearer legacy");
    for value in ["", "FlyV1 ", "fm2_a,fm2_b", "bad\ntoken", "a b"] {
        assert!(token(value).is_err());
    }
    validate_digest("manual", &image()).unwrap();
    for value in [
        image().replace("manual", "other"),
        image().replace("sha256:", "sha512:"),
        format!("{}x", image()),
    ] {
        assert!(validate_digest("manual", &value).is_err());
    }
}

#[test]
fn preflight_precedes_build_and_build_failure_cannot_mutate_machines() {
    let (server, _) = fixture(Fault::WrongOrg);
    assert!(
        build_and_rollout(&server.client(), "manual", || panic!(
            "must not build for a wrong account"
        ))
        .is_err()
    );
    let (server, _) = fixture(Fault::None);
    assert!(
        build_and_rollout(&server.client(), "manual", || Err(failed(
            "fixture build failed"
        )))
        .is_err()
    );
    assert_eq!(server.mutations("POST", true), 0);
    let mut builds = 0;
    let (count, _) = build_and_rollout(&server.client(), "manual", || {
        builds += 1;
        Ok(image())
    })
    .unwrap();
    assert_eq!((builds, count), (1, 2));
}

#[test]
fn changes_during_build_are_detected_before_leasing_or_updating() {
    let (server, state) = fixture(Fault::None);
    let result = build_and_rollout(&server.client(), "manual", || {
        state.lock().unwrap()[0]["config"]["env"]["KEEP"] = json!("changed by another deployer");
        Ok(image())
    });
    assert!(result.is_err());
    assert_eq!(server.mutations("POST", true), 0);
    assert_eq!(server.mutations("POST", false), 0);
}

#[cfg(unix)]
#[test]
fn inspect_uses_selected_runtime_and_accepts_only_one_matching_digest() {
    use std::os::unix::fs::PermissionsExt;
    let temp = tempfile::tempdir().unwrap();
    let program = temp.path().join("fake-container");
    let engine = ContainerEngine::new(
        zellij_engine::Runtime {
            program: program.to_str().unwrap().into(),
            host: Some("unix:///fixture-only.sock".into()),
            version: "fixture".into(),
        },
        "fixture-base",
    );
    for (output, success) in [
        (json!([image()]).to_string(), true),
        (json!([image(), image()]).to_string(), true),
        (
            json!([image(), image().replace("aaaa", "bbbb")]).to_string(),
            false,
        ),
        (
            json!([image().replace("manual", "other")]).to_string(),
            false,
        ),
        ("null".into(), false),
        ("[]".into(), false),
        ("not JSON".into(), false),
    ] {
        std::fs::write(&program,format!("#!/bin/sh\n[ \"$DOCKER_HOST\" = 'unix:///fixture-only.sock' ] || exit 2\n[ \"$1 $2 $3\" = 'image inspect --format' ] || exit 3\n[ \"$4\" = '{{{{json .RepoDigests}}}}' ] || exit 4\n[ \"$5\" = 'registry.fly.io/manual:fixture' ] || exit 5\nprintf '%s' '{output}'\n")).unwrap();
        std::fs::set_permissions(&program, std::fs::Permissions::from_mode(0o755)).unwrap();
        let result = inspect(&engine, "registry.fly.io/manual:fixture", "manual");
        assert_eq!(result.is_ok(), success, "{result:?}");
    }
    std::fs::write(
        &program,
        "#!/bin/sh\nprintf 'private credential' >&2\nexit 1\n",
    )
    .unwrap();
    let error = inspect(&engine, "registry.fly.io/manual:fixture", "manual")
        .unwrap_err()
        .to_string();
    assert!(!error.contains("private credential"));
}

#[test]
fn malformed_and_oversized_responses_are_bounded_and_redacted() {
    let server = Server::new(|_, _| json!({"provider_secret":"x".repeat(1024*1024)}).into());
    let error = server.client().check("manual").err().unwrap().to_string();
    assert!(error.contains("1 MiB") && !error.contains("provider_secret"));
}
