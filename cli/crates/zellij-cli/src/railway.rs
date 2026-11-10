//! Existing Railway image services only. Environment-scoped source updates use
//! the public GraphQL API; no resource creation, variable writes, or redeploy retry.
use camino::Utf8Path;
use reqwest::{blocking::Client as HttpClient, header};
use serde_json::{Value, json};
use std::{
    io::Read,
    time::{Duration, Instant},
};
use zellij_core::{
    Error, Result,
    delivery::{ImageKind, ImagePlatform, Provider, railway_image_repository, validate_railway_id},
    site::Site,
};
use zellij_engine::{BuildOptions, ContainerEngine};

fn failed(message: impl Into<String>) -> Error {
    Error::DeploymentFailed(message.into())
}
fn required(name: &str) -> Result<String> {
    std::env::var(name)
        .ok()
        .filter(|s| !s.is_empty())
        .ok_or_else(|| Error::InvalidArgument(format!("set {name} before Railway deployment")))
}

// Staged values are never decrypted or logged; only an empty patch is accepted.
const TARGET: &str = r#"query ZelRailwayTarget($service: String!, $environment: String!) {
  projectToken { projectId environmentId }
  service(id: $service) { id projectId deletedAt }
  environment(id: $environment) { id projectId deletedAt
    volumeInstances(first: 100) { pageInfo { hasNextPage } edges { node { serviceId deletedAt } } }
  }
  environmentStagedChanges(environmentId: $environment) {
    environmentId status patch(decryptVariables: false)
  }
  serviceInstance(serviceId: $service, environmentId: $environment) {
    id serviceId environmentId updatedAt source { image repo }
    startCommand preDeployCommand cronSchedule railwayConfigFile
    healthcheckPath healthcheckTimeout sleepApplication
    numReplicas region restartPolicyType restartPolicyMaxRetries
    rootDirectory buildCommand dockerfilePath overlapSeconds drainingSeconds
    latestDeployment { id status }
  }
}"#;
const VARIABLES: &str = r#"query ZelRailwayVariables($project: String!, $environment: String!, $service: String!) {
 variablesForServiceDeployment(projectId: $project, environmentId: $environment, serviceId: $service)
}"#;
const UPDATE: &str = r#"mutation ZelRailwayImage($service: String!, $environment: String!, $input: ServiceInstanceUpdateInput!) {
 serviceInstanceUpdate(serviceId: $service, environmentId: $environment, input: $input)
}"#;
const DEPLOY: &str = r#"mutation ZelRailwayDeploy($service: String!, $environment: String!) {
 serviceInstanceDeployV2(serviceId: $service, environmentId: $environment)
}"#;
const STATUS: &str = r#"query ZelRailwayStatus($id: String!) {
 deployment(id: $id) { id projectId environmentId serviceId status deploymentStopped }
}"#;

struct Client {
    http: HttpClient,
    endpoint: String,
    project: String,
    environment: String,
    budget: Duration,
    interval: Duration,
}
impl Client {
    fn new(token: &str, project: String, environment: String, endpoint: String) -> Result<Self> {
        validate_railway_id(&project)?;
        validate_railway_id(&environment)?;
        if token.is_empty() || token.len() > 16_384 || !token.bytes().all(|b| b.is_ascii_graphic())
        {
            return Err(Error::InvalidArgument(
                "RAILWAY_TOKEN must be a single project token without whitespace".into(),
            ));
        }
        let mut headers = header::HeaderMap::new();
        let mut token =
            header::HeaderValue::from_str(token).map_err(|_| failed("invalid Railway token"))?;
        token.set_sensitive(true);
        headers.insert("project-access-token", token);
        let http = HttpClient::builder()
            .default_headers(headers)
            .redirect(reqwest::redirect::Policy::none())
            .connect_timeout(Duration::from_secs(10))
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|_| failed("cannot initialize Railway HTTPS client"))?;
        Ok(Self {
            http,
            endpoint,
            project,
            environment,
            budget: Duration::from_secs(300),
            interval: Duration::from_secs(3),
        })
    }
    fn variables(&self, service: &str) -> Value {
        json!({"project":self.project, "environment":self.environment, "service":service})
    }
    fn request(&self, query: &str, variables: Value, deadline: Instant) -> Result<Value> {
        let remaining = deadline
            .checked_duration_since(Instant::now())
            .ok_or_else(|| failed("Railway operation timed out"))?;
        let mut response = self.http.post(&self.endpoint).timeout(remaining.min(Duration::from_secs(30)))
            .json(&json!({"query":query,"variables":variables})).send()
            .map_err(|_| failed("Railway request failed or timed out; remote state may have changed; details omitted"))?;
        if !response.status().is_success() {
            return Err(failed(format!(
                "Railway API returned HTTP {}; response omitted; inspect the service before retrying",
                response.status().as_u16()
            )));
        }
        if response.content_length().is_some_and(|n| n > 1024 * 1024) {
            return Err(failed("Railway response exceeded 1 MiB"));
        }
        let mut bytes = Vec::new();
        response
            .by_ref()
            .take(1024 * 1024 + 1)
            .read_to_end(&mut bytes)
            .map_err(|_| failed("Railway response read failed; details omitted"))?;
        if bytes.len() > 1024 * 1024 {
            return Err(failed("Railway response exceeded 1 MiB"));
        }
        let value: Value = serde_json::from_slice(&bytes)
            .map_err(|_| failed("Railway returned invalid JSON; response omitted"))?;
        // GraphQL can return HTTP 200 and partial data alongside an error.
        if !value["errors"].is_null() && value["errors"] != json!([]) {
            return Err(failed(
                "Railway GraphQL returned errors; partial data ignored and details omitted. Inspect the service before retrying",
            ));
        }
        if !value["data"].is_object() {
            return Err(failed("Railway returned no GraphQL data"));
        }
        Ok(value["data"].clone())
    }
    fn target(&self, service: &str, deadline: Instant) -> Result<Value> {
        let data = self.request(TARGET, self.variables(service), deadline)?;
        if data["projectToken"]["projectId"] != self.project
            || data["projectToken"]["environmentId"] != self.environment
            || data["service"]["id"] != service
            || data["service"]["projectId"] != self.project
            || !data["service"]["deletedAt"].is_null()
            || data["environment"]["id"] != self.environment
            || data["environment"]["projectId"] != self.project
            || !data["environment"]["deletedAt"].is_null()
        {
            return Err(failed(
                "Railway project token, service and environment must match RAILWAY_PROJECT_ID / RAILWAY_ENVIRONMENT_ID / the selected service",
            ));
        }
        // The nullable unmergedChangesCount alone cannot confirm that a
        // staged patch is empty. Read the pending patch explicitly and fail closed
        // unless Railway confirms an empty patch for this exact environment.
        let staged = &data["environmentStagedChanges"];
        if staged["environmentId"] != self.environment
            || staged["status"] != "STAGED"
            || staged["patch"] != json!({})
        {
            return Err(failed(
                "Railway environment has staged changes or their state is unavailable; resolve them before deployment",
            ));
        }
        let volumes = &data["environment"]["volumeInstances"];
        if volumes["pageInfo"]["hasNextPage"] != false
            || !volumes["edges"].as_array().is_some_and(|v| {
                v.iter()
                    .all(|v| v["node"]["serviceId"] != service || !v["node"]["deletedAt"].is_null())
            })
        {
            return Err(failed(
                "Railway requires a service without attached volumes and a complete volume inventory (at most 100 per environment)",
            ));
        }
        let instance = &data["serviceInstance"];
        if instance["serviceId"] != service
            || instance["environmentId"] != self.environment
            || instance["id"].as_str().is_none_or(str::is_empty)
            || instance["updatedAt"].as_str().is_none_or(str::is_empty)
            || instance["source"]["image"]
                .as_str()
                .is_none_or(str::is_empty)
            || !empty(&instance["source"]["repo"])
        {
            return Err(failed(
                "Railway requires an existing image-based service instance in the selected environment; Git source services are unsupported",
            ));
        }
        if [
            "startCommand",
            "preDeployCommand",
            "cronSchedule",
            "railwayConfigFile",
        ]
        .iter()
        .any(|k| !empty(&instance[k]))
            || instance["sleepApplication"] == true
        {
            return Err(failed(
                "Railway requires an always-on web service without custom start/pre-deploy/cron commands or a repository config file",
            ));
        }
        if !instance["healthcheckPath"].as_str().is_some_and(|s| {
            s.starts_with('/') && !s.starts_with("//") && s.bytes().all(|b| b.is_ascii_graphic())
        }) {
            return Err(failed(
                "configure a Railway HTTP healthcheck path before deploying",
            ));
        }
        // Railway returns all rendered variables as a scalar. Retain only the
        // two runtime controls; never serialize/log unrelated values.
        let variables = self.request(VARIABLES, self.variables(service), deadline)?;
        let variables = &variables["variablesForServiceDeployment"];
        if variables["PORT"] != "3000"
            || variables["HOSTNAME"]
                .as_str()
                .is_some_and(|s| s != "0.0.0.0")
        {
            return Err(failed(
                "Railway requires the service variable PORT=3000; HOSTNAME must be absent or 0.0.0.0",
            ));
        }
        Ok(instance.clone())
    }
    fn check(&self, service: &str) -> Result<Value> {
        Provider::Railway.validate_site(service)?;
        let target = self.target(service, Instant::now() + self.budget)?;
        id(&target["latestDeployment"]["id"])?;
        if target["latestDeployment"]["status"] != "SUCCESS" {
            return Err(failed(
                "Railway requires a successful existing deployment, without another deployment in progress",
            ));
        }
        Ok(target)
    }
    fn rollout(
        &self,
        service: &str,
        before: &Value,
        repository: &str,
        image: &str,
    ) -> Result<String> {
        crate::published_image::validate_digest(repository, image)?;
        let deadline = Instant::now() + self.budget;
        if self.target(service, deadline)? != *before {
            return Err(failed(
                "Railway service changed during the image build; no source update started",
            ));
        }
        // Only the environment-scoped image source is patched. No full-object
        // replacement, variable write, or global serviceConnect operation.
        let result = (|| {
            let mut variables = self.variables(service);
            variables["input"] = json!({"source":{"image":image}});
            let updated = self.request(UPDATE, variables, deadline)?;
            if updated["serviceInstanceUpdate"] != true {
                return Err(failed(
                    "Railway did not acknowledge the image source update",
                ));
            }
            let expected = config(before, image);
            let updated = self.target(service, deadline)?;
            if config(&updated, image) != expected
                || updated["source"]["image"] != image
                || updated["latestDeployment"] != before["latestDeployment"]
            {
                return Err(failed(
                    "Railway service configuration or deployment changed after the source update",
                ));
            }
            let response = self.request(DEPLOY, self.variables(service), deadline)?;
            let deployment = id(&response["serviceInstanceDeployV2"])?;
            if before["latestDeployment"]["id"] == deployment {
                return Err(failed("Railway returned the previous deployment ID"));
            }
            loop {
                let response = self.request(STATUS, json!({"id":deployment}), deadline)?;
                let receipt = &response["deployment"];
                if receipt["id"] != deployment
                    || receipt["projectId"] != self.project
                    || receipt["environmentId"] != self.environment
                    || receipt["serviceId"] != service
                    || receipt["deploymentStopped"].as_bool().is_none()
                {
                    return Err(failed(
                        "Railway deployment identity or instance state is invalid",
                    ));
                }
                match receipt["status"].as_str() {
                    Some("SUCCESS") => {
                        // Instances can all be stopped while a new deployment
                        // is queued/building. Require running instances only
                        // when Railway reports a successful deployment.
                        if receipt["deploymentStopped"] != false {
                            return Err(failed(
                                "Railway deployment succeeded but its instances stopped",
                            ));
                        }
                        let current = self.target(service, deadline)?;
                        if current["source"]["image"] != image
                            || config(&current, image) != expected
                            || current["latestDeployment"]["id"] != deployment
                            || current["latestDeployment"]["status"] != "SUCCESS"
                        {
                            return Err(failed(
                                "Railway deployment was superseded or configuration changed before success could be confirmed",
                            ));
                        }
                        return Ok(deployment.into());
                    }
                    Some("INITIALIZING" | "QUEUED" | "BUILDING" | "DEPLOYING") => {}
                    _ => {
                        return Err(failed(
                            "Railway deployment failed, is paused, needs approval, or returned an unsupported status; inspect the service dashboard",
                        ));
                    }
                }
                if Instant::now() + self.interval >= deadline {
                    return Err(failed("Railway deployment timed out before SUCCESS"));
                }
                std::thread::sleep(self.interval);
            }
        })();
        result.map_err(|e| failed(format!("{e}. The image source may already be updated and a deployment may still finish. No automatic retry or rollback was attempted; the pushed image is retained. Inspect Railway before retrying")))
    }
}
fn empty(v: &Value) -> bool {
    v.is_null() || v == "" || v.as_array().is_some_and(Vec::is_empty)
}
fn id(v: &Value) -> Result<&str> {
    v.as_str()
        .filter(|s| validate_railway_id(s).is_ok())
        .ok_or_else(|| failed("Railway returned an invalid deployment ID"))
}
fn config(value: &Value, image: &str) -> Value {
    let mut value = value.clone();
    let object = value.as_object_mut().expect("validated service instance");
    object.remove("updatedAt");
    object.remove("latestDeployment");
    value["source"]["image"] = json!(image);
    value
}
fn build_and_rollout(
    client: &Client,
    service: &str,
    repository: &str,
    build: impl FnOnce() -> Result<String>,
) -> Result<(String, String)> {
    let target = client.check(service)?;
    let image = build()?;
    let deployment = client.rollout(service, &target, repository, &image)?;
    Ok((deployment, image))
}
pub(crate) fn deploy(
    site: &Site,
    content: &Utf8Path,
    service: &str,
    kind: ImageKind,
    tag: &str,
    options: &BuildOptions,
) -> Result<()> {
    let repository = railway_image_repository(tag)?;
    let client = Client::new(
        &required("RAILWAY_TOKEN")?,
        required("RAILWAY_PROJECT_ID")?,
        required("RAILWAY_ENVIRONMENT_ID")?,
        "https://backboard.railway.com/graphql/v2".into(),
    )?;
    let (deployment, image) = build_and_rollout(&client, service, repository, || {
        let engine = ContainerEngine::detect(None)?;
        crate::images::build_image(
            site,
            content,
            &engine,
            kind,
            tag,
            Some(ImagePlatform::Amd64),
            options,
        )?;
        engine.image_push(tag, None).map_err(|e| failed(format!("{e}. Registry push failed; authenticate the selected container runtime to the image registry. Railway was not updated; the local image remains")))?;
        crate::published_image::inspect(&engine, tag, repository)
    })?;
    zellij_core::statusln!(
        "✓ Railway deployment {deployment} reached SUCCESS\n  {image}\n  Service image source is pinned to this digest; public DNS/TLS and ongoing health were not verified"
    );
    Ok(())
}

#[cfg(test)]
#[path = "railway_tests.rs"]
mod tests;
