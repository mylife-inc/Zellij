//! Deploy immutable images to existing Fly web Machines. Never create, scale,
//! destroy, change networking, or reconstruct configuration from a subset.
use camino::Utf8Path;
use reqwest::{Method, blocking::Client as HttpClient, header};
use serde_json::{Value, json};
use std::{
    collections::BTreeSet,
    io::Read,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use zellij_core::{
    Error, Result,
    delivery::{ImageKind, ImagePlatform, Provider},
    site::Site,
};
use zellij_engine::{BuildOptions, ContainerEngine};

fn failed(message: impl Into<String>) -> Error {
    Error::DeploymentFailed(message.into())
}
fn invalid(message: impl Into<String>) -> Error {
    Error::InvalidArgument(message.into())
}
fn required(name: &str) -> Result<String> {
    std::env::var(name)
        .ok()
        .filter(|s| !s.is_empty())
        .ok_or_else(|| {
            invalid(format!(
                "set {name} in the environment before Fly deployment"
            ))
        })
}

/// Single tokens only. Fly's registry needs the token without the auth scheme.
pub(crate) fn token(value: &str) -> Result<(&str, String)> {
    let raw = value
        .strip_prefix("FlyV1 ")
        .or_else(|| value.strip_prefix("Bearer "))
        .unwrap_or(value);
    if raw.is_empty()
        || raw.len() > 16_384
        || !raw.bytes().all(|b| b.is_ascii_graphic() && b != b',')
    {
        return Err(invalid(
            "FLY_API_TOKEN must contain one token, optionally prefixed by 'FlyV1 ' or 'Bearer '; token lists and whitespace are unsupported",
        ));
    }
    let scheme = if ["fm1r_", "fm1a_", "fm2_"]
        .iter()
        .any(|p| raw.starts_with(p))
    {
        "FlyV1"
    } else {
        "Bearer"
    };
    Ok((raw, format!("{scheme} {raw}")))
}

struct Client {
    http: HttpClient,
    base: String,
    org: String,
    budget: Duration,
    interval: Duration,
}
#[derive(Clone)]
struct Target {
    app: String,
    identity: Value,
    machines: Vec<Value>,
}
impl Client {
    fn new(token_value: &str, org: String, base: String) -> Result<Self> {
        Provider::Fly
            .validate_site(&org)
            .map_err(|_| invalid("FLY_ORG must be the expected Fly organization slug"))?;
        let (_, authorization) = token(token_value)?;
        let mut headers = header::HeaderMap::new();
        let mut authorization = header::HeaderValue::from_str(&authorization)
            .map_err(|_| invalid("invalid Fly token"))?;
        authorization.set_sensitive(true);
        headers.insert(header::AUTHORIZATION, authorization);
        let http = HttpClient::builder()
            .default_headers(headers)
            .redirect(reqwest::redirect::Policy::none())
            .connect_timeout(Duration::from_secs(10))
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|_| failed("cannot initialize Fly HTTPS client"))?;
        Ok(Self {
            http,
            base,
            org,
            budget: Duration::from_secs(300),
            interval: Duration::from_secs(2),
        })
    }
    fn request(
        &self,
        method: Method,
        path: &str,
        body: Option<&Value>,
        nonce: Option<&str>,
        deadline: Instant,
    ) -> Result<Value> {
        let remaining = deadline
            .checked_duration_since(Instant::now())
            .ok_or_else(|| failed("Fly operation timed out"))?;
        let mut request = self
            .http
            .request(method, format!("{}{path}", self.base))
            .timeout(remaining.min(Duration::from_secs(30)));
        if let Some(body) = body {
            request = request.json(body);
        }
        if let Some(nonce) = nonce {
            request = request.header("fly-machine-lease-nonce", nonce);
        }
        let mut response = request.send().map_err(|e| {
            failed(if e.is_timeout() || Instant::now() >= deadline {
                "Fly operation timed out"
            } else {
                "Fly request failed; remote state may have changed; response details omitted"
            })
        })?;
        if !response.status().is_success() {
            return Err(failed(format!(
                "Fly API returned HTTP {}; response body omitted. Inspect the app before retrying",
                response.status().as_u16()
            )));
        }
        if response.content_length().is_some_and(|n| n > 1024 * 1024) {
            return Err(failed("Fly response exceeded 1 MiB"));
        }
        let mut bytes = Vec::new();
        response
            .by_ref()
            .take(1024 * 1024 + 1)
            .read_to_end(&mut bytes)
            .map_err(|_| failed("Fly response read failed or timed out; details omitted"))?;
        if bytes.len() > 1024 * 1024 {
            return Err(failed("Fly response exceeded 1 MiB"));
        }
        serde_json::from_slice(&bytes)
            .map_err(|_| failed("Fly returned invalid JSON; response omitted"))
    }
    fn app(&self, app: &str, deadline: Instant) -> Result<Value> {
        let value = self.request(Method::GET, &format!("apps/{app}"), None, None, deadline)?;
        if value["name"] != app
            || value["organization"]["slug"] != self.org
            || value["id"].as_str().is_none_or(str::is_empty)
        {
            return Err(failed(
                "Fly app identity or organization does not match FLY_APP/FLY_ORG",
            ));
        }
        Ok(
            json!({"id":value["id"], "name":value["name"], "organization":value["organization"]["slug"]}),
        )
    }
    fn machines(&self, app: &str, deadline: Instant) -> Result<Vec<Value>> {
        let value = self.request(
            Method::GET,
            &format!("apps/{app}/machines"),
            None,
            None,
            deadline,
        )?;
        let machines = value
            .as_array()
            .ok_or_else(|| failed("Fly returned an invalid Machines list"))?;
        if machines.is_empty() || machines.len() > 10 {
            return Err(failed("Fly deployment requires 1–10 existing web Machines"));
        }
        let mut ids = BTreeSet::new();
        let mut result = Vec::new();
        for listed in machines {
            let id = identifier(listed, "id")?;
            if !ids.insert(id.to_owned()) {
                return Err(failed("Fly returned duplicate Machine IDs"));
            }
            let machine = self.request(
                Method::GET,
                &format!("apps/{app}/machines/{id}"),
                None,
                None,
                deadline,
            )?;
            if machine["id"] != id {
                return Err(failed("Fly returned a different Machine"));
            }
            validate_machine(&machine)?;
            result.push(machine);
        }
        result.sort_by(|a, b| a["id"].as_str().cmp(&b["id"].as_str()));
        Ok(result)
    }
    fn check(&self, app: &str) -> Result<Target> {
        Provider::Fly.validate_site(app)?;
        let deadline = Instant::now() + self.budget;
        Ok(Target {
            app: app.into(),
            identity: self.app(app, deadline)?,
            machines: self.machines(app, deadline)?,
        })
    }
    fn unchanged(&self, target: &Target, deadline: Instant) -> Result<()> {
        if self.app(&target.app, deadline)? != target.identity {
            return Err(failed("Fly app identity changed"));
        }
        let machines = self.machines(&target.app, deadline)?;
        if machines.len() != target.machines.len()
            || machines
                .iter()
                .zip(&target.machines)
                .any(|(a, b)| !same_version(a, b))
        {
            return Err(failed(
                "Fly Machine membership or configuration changed; no update was started",
            ));
        }
        Ok(())
    }
    fn rollout(&self, target: &Target, image: &str) -> Result<()> {
        validate_digest(&target.app, image)?;
        let deadline = Instant::now() + self.budget;
        self.unchanged(target, deadline)?;
        let mut leases: Vec<(String, String)> = Vec::new();
        let result = (|| {
            // Acquire every lease before updating any Machine. Optimistic version
            // checks also catch changes between preflight and lease acquisition.
            for machine in &target.machines {
                let id = identifier(machine, "id")?;
                let path = format!("apps/{}/machines/{id}", target.app);
                let lease = self.request(
                    Method::POST,
                    &format!("{path}/lease"),
                    Some(&json!({"description":"zel image deployment", "ttl":600})),
                    None,
                    deadline,
                )?;
                let nonce = lease["data"]["nonce"].as_str().filter(|n| !n.is_empty() && n.len() <= 4096 && n.bytes().all(|b| b.is_ascii_graphic())).ok_or_else(|| failed("Fly lease receipt is invalid; a lease may remain until its TTL expires"))?;
                leases.push((id.into(), nonce.into()));
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map_err(|_| failed("system clock is invalid"))?
                    .as_secs();
                if lease["status"] != "success"
                    || lease["data"]["expires_at"]
                        .as_u64()
                        .is_none_or(|e| e < now + 330)
                {
                    return Err(failed("Fly lease does not cover the rollout budget"));
                }
                let current = self.request(Method::GET, &path, None, Some(nonce), deadline)?;
                validate_machine(&current)?;
                if !same_version(&current, machine) {
                    return Err(failed("Fly Machine changed before its lease was acquired"));
                }
            }
            let mut updated = Vec::new();
            for (machine, (id, nonce)) in target.machines.iter().zip(&leases) {
                let path = format!("apps/{}/machines/{id}", target.app);
                let mut config = machine["config"].clone();
                config["image"] = json!(image);
                let response = self.request(Method::POST, &path, Some(&json!({"config":config,"current_version":identifier(machine,"instance_id")?,"skip_launch":false})), Some(nonce), deadline)?;
                let version = identifier(&response, "instance_id")?;
                // Some update receipts omit updated_at. Use the subsequent GET
                // for this exact instance in that case, never the local clock.
                let since = if response["updated_at"].is_null() {
                    None
                } else {
                    Some(timestamp(&response["updated_at"])?)
                };
                if response["id"] != *id
                    || response["instance_id"] == machine["instance_id"]
                    || response["config"] != config
                {
                    return Err(failed(
                        "Fly update receipt does not identify the requested new Machine version",
                    ));
                }
                loop {
                    let current = self.request(Method::GET, &path, None, Some(nonce), deadline)?;
                    if current["id"] != *id
                        || current["instance_id"] != version
                        || current["config"] != config
                        || current["cordoned"] == true
                        || current["host_status"] != "ok"
                    {
                        return Err(failed(
                            "Fly Machine identity, configuration or host changed during rollout",
                        ));
                    }
                    let since = match since {
                        Some(since) => since,
                        None => timestamp(&current["updated_at"])?,
                    };
                    if current["state"] == "started"
                        && checks_ready(&current, machine, since)
                        && current["image_ref"]["digest"] == image.split_once('@').unwrap().1
                    {
                        updated.push(current);
                        break;
                    }
                    if matches!(
                        current["state"].as_str(),
                        Some("destroyed" | "destroying" | "failed")
                    ) {
                        return Err(failed("Fly Machine failed during rollout"));
                    }
                    if Instant::now() + self.interval >= deadline {
                        return Err(failed(
                            "Fly rollout timed out waiting for the new image and fresh passing health checks",
                        ));
                    }
                    std::thread::sleep(self.interval);
                }
            }
            // Membership, version and health are checked again before success.
            self.unchanged(
                &Target {
                    machines: updated,
                    ..target.clone()
                },
                deadline,
            )
        })();
        let mut release_failed = false;
        for (id, nonce) in leases.iter().rev() {
            let release = self.request(
                Method::DELETE,
                &format!("apps/{}/machines/{id}/lease", target.app),
                None,
                Some(nonce),
                Instant::now() + Duration::from_secs(5),
            );
            if !release.is_ok_and(|v| v["status"] == "success" && v["data"]["ok"] == true) {
                release_failed = true;
            }
        }
        if release_failed {
            zellij_core::diagnosticln!(
                "Fly lease cleanup was not confirmed; leases expire after 600 seconds. Inspect the app before another deployment."
            );
        }
        if let Err(error) = result {
            return Err(failed(format!(
                "{error}. Fly updates are non-atomic: some Machines may already run the new image. No automatic retry or rollback was attempted; the published image is retained"
            )));
        }
        if release_failed {
            return Err(failed(
                "Fly rollout passed health checks but lease cleanup was not confirmed",
            ));
        }
        Ok(())
    }
}
fn identifier<'a>(value: &'a Value, key: &str) -> Result<&'a str> {
    value[key]
        .as_str()
        .filter(|s| {
            !s.is_empty()
                && s.len() <= 128
                && s.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')
        })
        .ok_or_else(|| failed("Fly returned an invalid Machine identity"))
}
fn timestamp(value: &Value) -> Result<chrono::DateTime<chrono::FixedOffset>> {
    value
        .as_str()
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
        .ok_or_else(|| failed("Fly returned a missing or invalid health/version timestamp"))
}
fn empty(value: &Value) -> bool {
    value.is_null()
        || value.as_array().is_some_and(Vec::is_empty)
        || value.as_object().is_some_and(|o| o.is_empty())
}
fn validate_machine(machine: &Value) -> Result<()> {
    identifier(machine, "id")?;
    identifier(machine, "instance_id")?;
    if machine["state"] != "started"
        || machine["host_status"] != "ok"
        || machine["cordoned"] == true
        || !empty(&machine["incomplete_config"])
    {
        return Err(failed(
            "Fly requires started, reachable, uncordoned Machines with complete configuration",
        ));
    }
    let config = &machine["config"];
    if !config.is_object()
        || config["image"].as_str().is_none_or(str::is_empty)
        || ["mounts", "files", "containers", "processes"]
            .iter()
            .any(|key| !empty(&config[key]))
        || config["init"].as_object().is_some_and(|o| {
            o.iter()
                .any(|(key, value)| !empty(value) && !(key == "tty" && value == false))
        })
        || (!config["init"].is_null() && !config["init"].is_object())
        || config["env"]["PORT"].as_str().is_some_and(|s| s != "3000")
        || config["env"]["HOSTNAME"]
            .as_str()
            .is_some_and(|s| s != "0.0.0.0")
        || config["metadata"]["fly_process_group"]
            .as_str()
            .is_some_and(|s| s != "app")
    {
        return Err(failed(
            "Fly supports single-container web Machines without mounts, injected files, process/entrypoint overrides or a different PORT/HOSTNAME; other process groups are unsupported",
        ));
    }
    let services = config["services"]
        .as_array()
        .filter(|s| !s.is_empty())
        .ok_or_else(|| failed("Fly requires existing HTTP services on internal port 3000"))?;
    for service in services {
        let ports = service["ports"].as_array();
        if service["internal_port"] != 3000
            || service["protocol"] != "tcp"
            || !matches!(service["autostop"].as_str(), None | Some("off"))
            || service["autostop"] == true
            || !ports.is_some_and(|ports| {
                !ports.is_empty()
                    && ports.iter().all(|p| {
                        p["handlers"]
                            .as_array()
                            .is_some_and(|h| h.iter().any(|v| v == "http"))
                    })
            })
        {
            return Err(failed(
                "Fly web services must use TCP/HTTP on internal port 3000 with autostop disabled",
            ));
        }
    }
    let checks = machine["checks"]
        .as_array()
        .filter(|c| !c.is_empty())
        .ok_or_else(|| {
            failed("Fly requires configured, passing health checks before deployment")
        })?;
    let mut names = BTreeSet::new();
    for check in checks {
        let name = check["name"]
            .as_str()
            .filter(|n| !n.is_empty())
            .ok_or_else(|| failed("Fly health check has no name"))?;
        if !names.insert(name) || check["status"] != "passing" {
            return Err(failed(
                "Fly requires all existing health checks to pass before deployment",
            ));
        }
        timestamp(&check["updated_at"])?;
    }
    Ok(())
}
fn same_version(a: &Value, b: &Value) -> bool {
    ["id", "instance_id", "config", "name", "region", "image_ref"]
        .iter()
        .all(|key| a[key] == b[key])
}
fn checks_ready(
    current: &Value,
    before: &Value,
    since: chrono::DateTime<chrono::FixedOffset>,
) -> bool {
    let Some(checks) = current["checks"].as_array() else {
        return false;
    };
    let expected = before["checks"].as_array().expect("preflight checks");
    checks.len() == expected.len()
        && expected.iter().all(|old| {
            checks.iter().filter(|c| c["name"] == old["name"]).count() == 1
                && checks.iter().any(|c| {
                    c["name"] == old["name"]
                        && c["status"] == "passing"
                        && timestamp(&c["updated_at"]).is_ok_and(|t| {
                            t >= since && timestamp(&old["updated_at"]).is_ok_and(|old| t > old)
                        })
                })
        })
}
fn validate_digest(app: &str, image: &str) -> Result<()> {
    let prefix = format!("registry.fly.io/{app}@sha256:");
    if !image.strip_prefix(&prefix).is_some_and(|s| {
        s.len() == 64
            && s.bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    }) {
        return Err(failed(
            "Fly requires the pushed image's SHA-256 repository digest",
        ));
    }
    Ok(())
}

fn inspect(engine: &ContainerEngine, tag: &str, app: &str) -> Result<String> {
    crate::published_image::inspect(engine, tag, &format!("registry.fly.io/{app}"))
}
pub(crate) fn deploy(
    site: &Site,
    content: &Utf8Path,
    app: &str,
    kind: ImageKind,
    options: &BuildOptions,
) -> Result<()> {
    let client = Client::new(
        &required("FLY_API_TOKEN")?,
        required("FLY_ORG")?,
        "https://api.machines.dev/v1/".into(),
    )?;
    let (count, image) = build_and_rollout(&client, app, || {
        let engine = ContainerEngine::detect(None)?;
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| failed("system clock is invalid"))?
            .as_nanos();
        let tag = format!("registry.fly.io/{app}:zel-{nonce}-{}", std::process::id());
        crate::images::build_image(
            site,
            content,
            &engine,
            kind,
            &tag,
            Some(ImagePlatform::Amd64),
            options,
        )?;
        engine.image_push(&tag, None).map_err(|e| failed(format!("{e}. Fly registry push failed; local image {tag} remains. Authenticate the selected container runtime to registry.fly.io, then retry; no Machines updated")))?;
        inspect(&engine, &tag, app)
    })?;
    zellij_core::statusln!(
        "✓ Fly app {app}: {count} Machines updated with fresh passing health checks\n  {image}\n  Public DNS/TLS and external reachability were not verified"
    );
    Ok(())
}

fn build_and_rollout(
    client: &Client,
    app: &str,
    build: impl FnOnce() -> Result<String>,
) -> Result<(usize, String)> {
    let target = client.check(app)?;
    let image = build()?;
    client.rollout(&target, &image)?;
    Ok((target.machines.len(), image))
}

#[cfg(test)]
#[path = "fly_tests.rs"]
mod tests;
