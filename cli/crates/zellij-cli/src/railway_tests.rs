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

const PROJECT: &str = "11111111-1111-4111-8111-111111111111";
const ENVIRONMENT: &str = "22222222-2222-4222-8222-222222222222";
const SERVICE: &str = "33333333-3333-4333-8333-333333333333";
const OLD: &str = "44444444-4444-4444-8444-444444444444";
const NEW: &str = "55555555-5555-4555-8555-555555555555";
const REPOSITORY: &str = "ghcr.io/acme/docs";
fn image() -> String {
    format!("{REPOSITORY}@sha256:{}", "a".repeat(64))
}
fn target() -> Value {
    json!({"projectToken":{"projectId":PROJECT,"environmentId":ENVIRONMENT},
        "service":{"id":SERVICE,"projectId":PROJECT,"deletedAt":null},
        "environment":{"id":ENVIRONMENT,"projectId":PROJECT,"deletedAt":null,"unmergedChangesCount":0,"volumeInstances":{"pageInfo":{"hasNextPage":false},"edges":[]}},
        "serviceInstance":{"id":"instance-id","serviceId":SERVICE,"environmentId":ENVIRONMENT,"updatedAt":"2026-11-05T12:00:00Z","source":{"image":"ghcr.io/acme/docs:old","repo":null},"startCommand":null,"preDeployCommand":[],"cronSchedule":null,"railwayConfigFile":null,"healthcheckPath":"/","healthcheckTimeout":300,"sleepApplication":false,"numReplicas":2,"region":"us-west2","restartPolicyType":"ON_FAILURE","restartPolicyMaxRetries":10,"rootDirectory":null,"buildCommand":null,"dockerfilePath":null,"overlapSeconds":30,"drainingSeconds":20,"latestDeployment":{"id":OLD,"status":"SUCCESS"}}})
}
fn fixture(mode: &'static str) -> Server {
    let state = Mutex::new(target());
    let polls = Mutex::new(0);
    let reads = Mutex::new(0);
    Server::new(move |req, _| {
        assert_eq!(req.method, "POST");
        assert_eq!(req.path, "/");
        assert_eq!(
            req.headers.get("project-access-token").unwrap(),
            "fixture-secret"
        );
        assert!(!req.headers.contains_key("authorization"));
        let body: Value = serde_json::from_slice(&req.body).unwrap();
        let query = body["query"].as_str().unwrap();
        let vars = &body["variables"];
        let mut state = state.lock().unwrap();
        if query == TARGET {
            assert_eq!(vars["service"], SERVICE);
            assert_eq!(vars["environment"], ENVIRONMENT);
            let mut reads = reads.lock().unwrap();
            *reads += 1;
            let mut reply = state.clone();
            match mode {
                "scope" => reply["projectToken"]["environmentId"] = json!(PROJECT),
                "staged" => reply["environment"]["unmergedChangesCount"] = json!(1),
                "volumes" => {
                    reply["environment"]["volumeInstances"]["edges"] =
                        json!([{"node":{"serviceId":SERVICE,"deletedAt":null}}])
                }
                "pagination" => {
                    reply["environment"]["volumeInstances"]["pageInfo"]["hasNextPage"] = json!(true)
                }
                "git" => reply["serviceInstance"]["source"]["repo"] = json!("acme/repo"),
                "start" => reply["serviceInstance"]["startCommand"] = json!("custom"),
                "cron" => reply["serviceInstance"]["cronSchedule"] = json!("* * * * *"),
                "config" => reply["serviceInstance"]["railwayConfigFile"] = json!("railway.toml"),
                "sleep" => reply["serviceInstance"]["sleepApplication"] = json!(true),
                "health" => reply["serviceInstance"]["healthcheckPath"] = Value::Null,
                "active" => {
                    reply["serviceInstance"]["latestDeployment"]["status"] = json!("DEPLOYING")
                }
                "changed" if *reads > 1 => reply["serviceInstance"]["numReplicas"] = json!(3),
                "superseded" if *reads > 3 => {
                    reply["serviceInstance"]["latestDeployment"]["id"] = json!(OLD)
                }
                _ => {}
            }
            return json!({"data":reply}).into();
        }
        if query == VARIABLES {
            assert_eq!(vars["project"], PROJECT);
            return json!({"data":{"variablesForServiceDeployment":{"PORT":if mode == "port" { "8080" } else { "3000" },"UNRELATED_SECRET":"must-never-appear"}}}).into();
        }
        if query == UPDATE {
            assert_eq!(vars["input"], json!({"source":{"image":image()}}));
            assert_eq!(vars["environment"], ENVIRONMENT);
            assert_eq!(vars["service"], SERVICE);
            if mode == "partial" {
                return json!({"data":{"serviceInstanceUpdate":true},"errors":[{"message":"fixture-secret must-never-appear"}]}).into();
            }
            state["serviceInstance"]["source"]["image"] = json!(image());
            state["serviceInstance"]["updatedAt"] = json!("2026-11-05T12:01:00Z");
            if mode == "settings" {
                state["serviceInstance"]["region"] = json!("unexpected");
            }
            return json!({"data":{"serviceInstanceUpdate":mode != "unacknowledged"}}).into();
        }
        if query == DEPLOY {
            assert_eq!(vars["environment"], ENVIRONMENT);
            assert_eq!(vars["service"], SERVICE);
            state["serviceInstance"]["latestDeployment"] = json!({"id":NEW,"status":"SUCCESS"});
            return json!({"data":{"serviceInstanceDeployV2":if mode == "old-receipt" { OLD } else { NEW }}}).into();
        }
        assert_eq!(query, STATUS);
        assert_eq!(vars["id"], NEW);
        let mut polls = polls.lock().unwrap();
        *polls += 1;
        let status = match mode {
            "failed" => "FAILED",
            "approval" => "NEEDS_APPROVAL",
            "sleeping" => "SLEEPING",
            "timeout" => "DEPLOYING",
            "unknown" => "NEW_STATUS",
            _ if *polls == 1 => "DEPLOYING",
            _ => "SUCCESS",
        };
        json!({"data":{"deployment":{"id":NEW,"projectId":PROJECT,"environmentId":if mode == "wrong-receipt" { PROJECT } else { ENVIRONMENT },"serviceId":SERVICE,"status":status,"deploymentStopped":false}}}).into()
    })
}
fn client(server: &Server) -> Client {
    let mut client = Client::new(
        "fixture-secret",
        PROJECT.into(),
        ENVIRONMENT.into(),
        server.address.clone(),
    )
    .unwrap();
    client.budget = Duration::from_secs(5);
    client.interval = Duration::from_millis(1);
    client
}
fn query_count(server: &Server, query: &str) -> usize {
    server
        .requests
        .lock()
        .unwrap()
        .iter()
        .filter(|r| serde_json::from_slice::<Value>(&r.body).unwrap()["query"] == query)
        .count()
}
#[test]
fn railway_builds_once_patches_only_image_and_waits_for_its_deployment() {
    let server = fixture("success");
    let client = client(&server);
    let mut builds = 0;
    let result = build_and_rollout(&client, SERVICE, REPOSITORY, || {
        builds += 1;
        Ok(image())
    })
    .unwrap();
    assert_eq!(builds, 1);
    assert_eq!(result, (NEW.into(), image()));
    assert_eq!(query_count(&server, UPDATE), 1);
    assert_eq!(query_count(&server, DEPLOY), 1);
    assert_eq!(query_count(&server, STATUS), 2);
}
#[test]
fn railway_preflight_rejects_unsupported_targets_before_building() {
    for mode in [
        "scope",
        "staged",
        "volumes",
        "pagination",
        "git",
        "start",
        "cron",
        "config",
        "sleep",
        "health",
        "active",
        "port",
    ] {
        let server = fixture(mode);
        let client = client(&server);
        let error = build_and_rollout(&client, SERVICE, REPOSITORY, || panic!("built for {mode}"))
            .unwrap_err()
            .to_string();
        assert!(!error.contains("must-never-appear"));
        assert_eq!(query_count(&server, UPDATE), 0, "{mode}");
    }
}
#[test]
fn railway_build_failure_or_target_drift_never_updates_source() {
    for mode in ["success", "changed"] {
        let server = fixture(mode);
        let client = client(&server);
        let result = build_and_rollout(&client, SERVICE, REPOSITORY, || {
            if mode == "success" {
                Err(failed("build failed"))
            } else {
                Ok(image())
            }
        });
        assert!(result.is_err());
        assert_eq!(query_count(&server, UPDATE), 0);
    }
}
#[test]
fn railway_validates_digest_before_any_update() {
    let server = fixture("success");
    let client = client(&server);
    for digest in ["ghcr.io/other/docs@sha256:abc", "ghcr.io/acme/docs:latest"] {
        assert!(build_and_rollout(&client, SERVICE, REPOSITORY, || Ok(digest.into())).is_err());
    }
    assert_eq!(query_count(&server, UPDATE), 0);
}
#[test]
fn railway_mutation_errors_are_redacted_and_never_retried() {
    for mode in ["partial", "unacknowledged", "settings"] {
        let server = fixture(mode);
        let client = client(&server);
        let error = build_and_rollout(&client, SERVICE, REPOSITORY, || Ok(image()))
            .unwrap_err()
            .to_string();
        assert!(error.contains("may already be updated"), "{error}");
        assert!(!error.contains("fixture-secret"));
        assert!(!error.contains("must-never-appear"));
        assert_eq!(query_count(&server, UPDATE), 1);
        assert_eq!(query_count(&server, DEPLOY), 0);
    }
}
#[test]
fn railway_rejects_failed_paused_foreign_or_superseded_receipts() {
    for mode in [
        "failed",
        "approval",
        "sleeping",
        "unknown",
        "wrong-receipt",
        "old-receipt",
        "superseded",
    ] {
        let server = fixture(mode);
        let client = client(&server);
        assert!(
            build_and_rollout(&client, SERVICE, REPOSITORY, || Ok(image())).is_err(),
            "{mode}"
        );
        assert_eq!(query_count(&server, UPDATE), 1);
        assert_eq!(query_count(&server, DEPLOY), 1);
    }
}
#[test]
fn railway_timeout_does_not_repeat_deployment() {
    let server = fixture("timeout");
    let mut client = client(&server);
    let before = client.check(SERVICE).unwrap();
    client.budget = Duration::from_millis(100);
    let error = client
        .rollout(SERVICE, &before, REPOSITORY, &image())
        .unwrap_err()
        .to_string();
    assert!(error.contains("timed out"), "{error}");
    assert_eq!(query_count(&server, DEPLOY), 1);
}
#[test]
fn railway_http_errors_redirects_large_bodies_and_graphql_errors_are_bounded() {
    for (status, body, redirect) in [
        (401, json!({"secret":"fixture-secret"}), None),
        (302, json!({}), Some("https://example.invalid/".into())),
        (
            200,
            json!({"data":{},"errors":[{"message":"fixture-secret"}]}),
            None,
        ),
        (200, json!({"data":{"huge":"x".repeat(1024*1024)}}), None),
    ] {
        let server = Server::new(move |_, _| Reply {
            status,
            body: body.clone(),
            redirect: redirect.clone(),
        });
        let error = client(&server).check(SERVICE).unwrap_err().to_string();
        assert!(!error.contains("fixture-secret"));
        assert_eq!(server.requests.lock().unwrap().len(), 1);
    }
}
#[test]
fn railway_credentials_require_explicit_ids_and_a_project_token() {
    for token in ["", "Bearer secret", "token\n"] {
        assert!(
            Client::new(
                token,
                PROJECT.into(),
                ENVIRONMENT.into(),
                "https://example.invalid".into()
            )
            .is_err()
        );
    }
    assert!(
        Client::new(
            "token",
            "project-name".into(),
            ENVIRONMENT.into(),
            "https://example.invalid".into()
        )
        .is_err()
    );
}
