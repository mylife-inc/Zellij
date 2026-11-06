//! GitLab and CircleCI use the same validated commands, with service-native
//! event gates and credential boundaries. Generation never contacts either CI.
use super::*;
use zellij_core::{ci::System, site::Renderer};

fn quote(value: &str) -> Result<String> {
    literal(value)?;
    if value.contains("<<") || value.contains("$[[") {
        return Err(invalid(
            "values cannot contain CI interpolation expressions",
        ));
    }
    Ok(format!("'{}'", value.replace('\'', "'\\''")))
}
fn export(script: &mut String, key: &str, value: &str) -> Result<()> {
    script.push_str(&format!("export {key}={}\n", quote(value)?));
    Ok(())
}
fn key(options: &CiOptions) -> String {
    format!("{:x}", Sha256::digest(options.settings.workflow.as_bytes()))[..16].into()
}
fn image(publish: &Publish) -> Option<ImageKind> {
    match publish {
        Publish::Image { kind, .. } | Publish::Fly { kind, .. } | Publish::Railway { kind, .. } => {
            Some(*kind)
        }
        Publish::LegacyImage => Some(ImageKind::Next),
        _ => None,
    }
}
fn docker(options: &CiOptions) -> bool {
    options.renderer == Renderer::Hosted || image(&options.publish).is_some()
}
fn architectures(options: &CiOptions) -> Vec<&'static str> {
    match options.platform {
        Some(ImagePlatform::All) => vec!["amd64", "arm64"],
        Some(ImagePlatform::Arm64) => vec!["arm64"],
        _ => vec!["amd64"],
    }
}
fn registry(tag: &str) -> Result<&str> {
    zellij_core::delivery::validate_image_reference(tag)?;
    let (host, _) = tag
        .split_once('/')
        .ok_or_else(|| invalid("image.name requires an explicit remote registry hostname"))?;
    if !host.contains('.') || host == "localhost" || host.starts_with("127.") {
        return Err(invalid(
            "image.name requires an explicit remote registry hostname",
        ));
    }
    Ok(host)
}
fn image_repository(tag: &str) -> &str {
    let end = tag.rfind('/').unwrap() + 1;
    match tag[end..].find(':') {
        Some(index) => &tag[..end + index],
        None => tag,
    }
}
fn validate(options: &CiOptions) -> Result<Prepared> {
    let prepared = prepare(options)?;
    if matches!(options.publish, Publish::LegacyImage) {
        return Err(invalid(
            "GitLab/CircleCI require an explicit image.name; implicit GHCR publication is GitHub-only",
        ));
    }
    if options.platform == Some(ImagePlatform::All)
        && !matches!(options.publish, Publish::Image { .. })
    {
        return Err(invalid("combined platforms require image publication"));
    }
    if let Publish::Image { tag, .. } | Publish::Railway { tag, .. } = &options.publish {
        registry(tag)?;
        quote(tag)?;
    }
    if let Publish::Railway { tag, .. } = &options.publish {
        zellij_core::delivery::railway_image_repository(tag)?;
    }
    Ok(prepared)
}
/// Positive globs have identical semantics on both services: * and ? do not
/// cross '/', ** does. Only the limited Settings alphabet reaches this function.
fn tag_pattern(glob: &str) -> String {
    let mut out = String::from("^");
    let mut chars = glob.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '*' if chars.peek() == Some(&'*') => {
                chars.next();
                out.push_str(".*");
            }
            '*' => out.push_str("[^/]*"),
            '?' => out.push_str("[^/]"),
            '.' => out.push_str("\\."),
            _ => out.push(c),
        }
    }
    out.push('$');
    out
}
fn gitlab_conditions(settings: &Settings) -> (String, String) {
    let branch = format!("$CI_COMMIT_BRANCH == \"{}\"", settings.branch);
    let mut publish = vec![];
    if settings.triggers.contains(&Trigger::Push) {
        publish.push(format!("($CI_PIPELINE_SOURCE == \"push\" && {branch})"));
    }
    if settings.triggers.contains(&Trigger::Manual) {
        publish.push(format!("(($CI_PIPELINE_SOURCE == \"web\" || $CI_PIPELINE_SOURCE == \"api\" || $CI_PIPELINE_SOURCE == \"trigger\") && {branch})"));
    }
    if !settings.schedules.is_empty() {
        publish.push(format!("($CI_PIPELINE_SOURCE == \"schedule\" && {branch})"));
    }
    for tag in &settings.tags {
        publish.push(format!(
            "($CI_PIPELINE_SOURCE == \"push\" && $CI_COMMIT_TAG =~ /{}/)",
            tag_pattern(tag).replace('/', "\\/")
        ));
    }
    let refs = if publish.is_empty() {
        "$CI_PIPELINE_SOURCE == \"zel_disabled\"".into()
    } else {
        publish.join(" || ")
    };
    let eligible = if settings.triggers.contains(&Trigger::PullRequest) {
        format!("$CI_PIPELINE_SOURCE == \"merge_request_event\" || ({refs})")
    } else {
        refs.clone()
    };
    (
        eligible,
        format!("$CI_COMMIT_REF_PROTECTED == \"true\" && ({refs})"),
    )
}
fn circle_conditions(settings: &Settings) -> (Value, Value) {
    let eq = |key: &str, value: &str| json!({"equal":[value, format!("<< {key} >>")]});
    let branch =
        json!({"and":[eq("pipeline.git.branch", &settings.branch),eq("pipeline.git.tag", "")]});
    let event = |name: &str| eq("pipeline.event.name", name);
    let mut publish = vec![];
    for (enabled, name) in [
        (settings.triggers.contains(&Trigger::Push), "push"),
        (settings.triggers.contains(&Trigger::Manual), "api"),
        (!settings.schedules.is_empty(), "schedule"),
    ] {
        if enabled {
            publish.push(json!({"and":[event(name),branch]}));
        }
    }
    for tag in &settings.tags {
        publish.push(json!({"and":[event("push"),{"matches":{"pattern":tag_pattern(tag),"value":"<< pipeline.git.tag >>"}}]}));
    }
    let publication = if publish.is_empty() {
        json!(false)
    } else {
        json!({"or":publish})
    };
    let eligible = if settings.triggers.contains(&Trigger::PullRequest) {
        json!({"or":[publication,{"and":[event("pull_request"),{"not":eq("pipeline.event.action", "closed")}]}]})
    } else {
        publication.clone()
    };
    (eligible, publication)
}

#[derive(Clone, Copy)]
enum Task {
    Verify,
    Publish,
    Stage,
    Manifest,
}
fn install_script(
    options: &CiOptions,
    prepared: &Prepared,
    system: System,
    manifest: bool,
) -> Result<String> {
    let mut script = String::from("set -eu\numask 077\n");
    let temp = if system == System::Gitlab {
        "${CI_BUILDS_DIR:?GitLab shared builds directory is required}"
    } else {
        "${TMPDIR:-/tmp}"
    };
    script.push_str(&format!("ZEL_WORK=$(mktemp -d \"{temp}/zel-ci.XXXXXX\")\nexport TMPDIR=\"$ZEL_WORK\"\nexport DOCKER_CONFIG=\"$ZEL_WORK/docker\"\nmkdir -p \"$DOCKER_CONFIG\"\nZEL_LOGIN_REGISTRY=''\ncleanup() {{\n  if [ -n \"$ZEL_LOGIN_REGISTRY\" ]; then docker logout \"$ZEL_LOGIN_REGISTRY\" >/dev/null 2>&1 || true; fi\n  rm -rf -- \"$ZEL_WORK\"\n}}\ntrap cleanup EXIT\n"));
    if manifest {
        return Ok(script);
    }
    export(&mut script, "ZEL_VERSION", &prepared.version)?;
    script.push_str("export ZEL_INSTALL_DIR=\"$ZEL_WORK/bin\"\nbase=\"https://github.com/mylife-inc/releases/releases/download/zellij-$ZEL_VERSION\"\ncurl --fail --silent --show-error --location \"$base/install.sh\" --output \"$ZEL_WORK/install.sh\" || { echo 'The pinned zel release must be published before this pipeline can run.' >&2; exit 1; }\nsh \"$ZEL_WORK/install.sh\"\nexport PATH=\"$ZEL_INSTALL_DIR:$PATH\"\n");
    export(
        &mut script,
        "SITE_ROOT",
        &if options.site_root == "." {
            ".".into()
        } else {
            format!("./{}", options.site_root)
        },
    )?;
    if let Some(config) = &options.config {
        export(&mut script, "SITE_CONFIG", &format!("./{config}"))?;
    }
    export(
        &mut script,
        "ZELLIJ_IMAGE",
        &format!(
            "ghcr.io/mylife-inc/zellij-base:{}",
            prepared.version.trim_start_matches('v')
        ),
    )?;
    for (name, value) in &prepared.override_env {
        export(&mut script, name, value.as_str().unwrap())?;
    }
    if !options.overrides.no_check {
        script.push_str(&format!(
            "zel check {} --strict --deep\n",
            prepared.selector
        ));
    }
    Ok(script)
}
fn login(script: &mut String, host: &str, system: System) -> Result<()> {
    export(script, "ZEL_LOGIN_REGISTRY", host)?;
    if system == System::Gitlab {
        script.push_str("if [ -z \"${REGISTRY_USERNAME:-}\" ] && [ -z \"${REGISTRY_PASSWORD:-}\" ] && [ \"$ZEL_LOGIN_REGISTRY\" = \"${CI_REGISTRY:-}\" ]; then\n  REGISTRY_USERNAME=${CI_REGISTRY_USER:-}\n  REGISTRY_PASSWORD=${CI_REGISTRY_PASSWORD:-}\nfi\n");
    }
    script.push_str(": \"${REGISTRY_USERNAME:?Set REGISTRY_USERNAME in the publication credential scope}\"\n: \"${REGISTRY_PASSWORD:?Set REGISTRY_PASSWORD in the publication credential scope}\"\nprintf '%s' \"$REGISTRY_PASSWORD\" | docker login \"$ZEL_LOGIN_REGISTRY\" --username \"$REGISTRY_USERNAME\" --password-stdin\n");
    Ok(())
}
fn saved_destination(
    script: &mut String,
    provider: Provider,
    saved: &Option<String>,
) -> Result<()> {
    if let Some(value) = saved {
        provider.validate_site(value)?;
        export(script, provider.site_environment(), value)?;
    }
    Ok(())
}
/// Check the entire digest and repository, including Docker Hub's canonical
/// aliases. Never accept an arbitrary artifact as an imagetools argument.
fn digest_validator(repository: &str) -> Result<String> {
    let canonical = repository
        .strip_prefix("docker.io/")
        .or_else(|| repository.strip_prefix("index.docker.io/"));
    let mut aliases = vec![repository.to_owned()];
    if let Some(repo) = canonical {
        let repo = if repo.contains('/') {
            repo.into()
        } else {
            format!("library/{repo}")
        };
        aliases.extend([
            repo.clone(),
            format!("docker.io/{repo}"),
            format!("index.docker.io/{repo}"),
        ]);
    }
    let patterns = aliases
        .iter()
        .map(|s| quote(&format!("{s}@sha256:")))
        .collect::<Result<Vec<_>>>()?
        .join("* | ");
    Ok(format!(
        "validate_digest() {{\n  case \"$1\" in {patterns}*) ;; *) echo 'Unexpected image repository in digest receipt' >&2; return 1 ;; esac\n  digest=${{1##*@sha256:}}\n  [ \"${{#digest}}\" -eq 64 ] || {{ echo 'Invalid digest length' >&2; return 1; }}\n  case \"$digest\" in *[!0-9a-f]*) echo 'Invalid digest characters' >&2; return 1 ;; esac\n}}\n"
    ))
}
fn script(
    options: &CiOptions,
    prepared: &Prepared,
    system: System,
    task: Task,
    arch: &str,
) -> Result<String> {
    let mut s = install_script(options, prepared, system, matches!(task, Task::Manifest))?;
    let selector = prepared.selector;
    let flags = format!("{}{}", prepared.features_flag, prepared.override_flags);
    let platform = if image(&options.publish).is_some() {
        format!(" --platform linux/{arch}")
    } else {
        String::new()
    };
    let receipt = format!(".zel-ci-{}", key(options));
    if matches!(task, Task::Verify) {
        if let Some(kind) = image(&options.publish) {
            s.push_str(&format!(
                "zel image {} {selector} --tag zel-ci-check:local{platform}{flags}\n",
                kind.as_str()
            ));
        } else {
            s.push_str(&format!(
                "zel build {selector} --out \"$ZEL_WORK/site\"{flags}\n"
            ));
        }
        return Ok(s);
    }
    let prod = if options.settings.production {
        " --prod"
    } else {
        ""
    };
    match &options.publish {
        Publish::Image { kind, tag } => {
            let repository = image_repository(tag);
            export(&mut s, "IMAGE_TAG", tag)?;
            login(&mut s, registry(tag)?, system)?;
            match task {
                Task::Stage => {
                    let id = if system == System::Gitlab { "${CI_PIPELINE_ID:?}-${CI_JOB_ID:?}" } else { "${CIRCLE_WORKFLOW_ID:?}-${CIRCLE_BUILD_NUM:?}" };
                    export(&mut s, "IMAGE_REPOSITORY", repository)?;
                    s.push_str(&format!("BUILD_ID=\"{id}\"\ncase \"$BUILD_ID\" in ''|*[!a-zA-Z0-9-]*) echo 'Invalid CI build identity' >&2; exit 1;; esac\nBUILD_TAG=\"$IMAGE_REPOSITORY:zel-$BUILD_ID-{arch}\"\n[ ! -e '{receipt}' ] && [ ! -L '{receipt}' ] || {{ echo 'Reserved CI artifact directory already exists' >&2; exit 1; }}\nzel image {} {selector} --tag \"$BUILD_TAG\"{platform}{flags}\nzel image push \"$BUILD_TAG\"\n", kind.as_str()));
                    s.push_str(&digest_validator(repository)?);
                    s.push_str("docker image inspect --format '{{range .RepoDigests}}{{println .}}{{end}}' \"$BUILD_TAG\" > \"$ZEL_WORK/digests\"\nIMAGE_DIGEST=''\nwhile IFS= read -r candidate; do\n  [ -n \"$candidate\" ] || continue\n  if validate_digest \"$candidate\" 2>/dev/null; then\n    if [ -n \"$IMAGE_DIGEST\" ] && [ \"${IMAGE_DIGEST##*@}\" != \"${candidate##*@}\" ]; then echo 'Ambiguous registry digests' >&2; exit 1; fi\n    IMAGE_DIGEST=$candidate\n  fi\ndone < \"$ZEL_WORK/digests\"\n[ -n \"$IMAGE_DIGEST\" ] || { echo 'Registry digest missing after push' >&2; exit 1; }\n");
                    s.push_str(&format!("mkdir '{receipt}'\nprintf '%s\\n' \"$IMAGE_DIGEST\" > '{receipt}/{arch}.digest'\n"));
                }
                Task::Manifest => {
                    s.push_str(&digest_validator(repository)?);
                    for (arch, variable) in [("amd64", "AMD64_IMAGE"), ("arm64", "ARM64_IMAGE")] {
                        s.push_str(&format!("[ -f '{receipt}/{arch}.digest' ] && [ ! -L '{receipt}/{arch}.digest' ] || {{ echo 'Missing image receipt' >&2; exit 1; }}\n{variable}=$(cat '{receipt}/{arch}.digest')\nvalidate_digest \"${variable}\"\n"));
                    }
                    s.push_str("docker buildx imagetools create --tag \"$IMAGE_TAG\" \"$AMD64_IMAGE\" \"$ARM64_IMAGE\"\ndocker buildx imagetools inspect \"$IMAGE_TAG\"\n");
                }
                Task::Publish => s.push_str(&format!("zel image {} {selector} --tag \"$IMAGE_TAG\"{platform}{flags}\nzel image push \"$IMAGE_TAG\"\n", kind.as_str())),
                Task::Verify => unreachable!(),
            }
        }
        Publish::Fly { app, kind } => {
            saved_destination(&mut s, Provider::Fly, app)?;
            s.push_str("TOKEN=${FLY_API_TOKEN:?Set FLY_API_TOKEN in the publication credential scope}\ncase \"$TOKEN\" in 'FlyV1 '*) TOKEN=${TOKEN#'FlyV1 '} ;; 'Bearer '*) TOKEN=${TOKEN#'Bearer '} ;; esac\nZEL_LOGIN_REGISTRY=registry.fly.io\nprintf '%s' \"$TOKEN\" | docker login registry.fly.io --username x --password-stdin\nunset TOKEN\n");
            s.push_str(&format!(
                "zel deploy fly {selector} --kind {} --prod{flags}\n",
                kind.as_str()
            ));
        }
        Publish::Railway { service, kind, tag } => {
            saved_destination(&mut s, Provider::Railway, service)?;
            login(&mut s, registry(tag)?, system)?;
            export(&mut s, "IMAGE_TAG", tag)?;
            s.push_str(&format!(
                "zel deploy railway {selector} --kind {} --tag \"$IMAGE_TAG\" --prod{flags}\n",
                kind.as_str()
            ));
        }
        Publish::Netlify { site_id } => {
            saved_destination(&mut s, Provider::Netlify, site_id)?;
            s.push_str(&format!("zel deploy netlify {selector}{prod}{flags}\n"));
        }
        Publish::Cloudflare { project } => {
            saved_destination(&mut s, Provider::Cloudflare, project)?;
            s.push_str(&format!("zel deploy cloudflare {selector}{prod}{flags}\n"));
        }
        Publish::Vercel { project } => {
            saved_destination(&mut s, Provider::Vercel, project)?;
            s.push_str(&format!("zel deploy vercel {selector}{prod}{flags}\n"));
        }
        Publish::GithubPages { repository } => {
            saved_destination(&mut s, Provider::GithubPages, repository)?;
            s.push_str(&format!(
                "zel deploy github-pages {selector} --prod{flags}\n"
            ));
        }
        Publish::S3 { destination } => {
            saved_destination(&mut s, Provider::S3, destination)?;
            s.push_str(&format!("zel deploy s3 {selector} --prod{flags}\n"));
        }
        Publish::Nothing | Publish::LegacyImage => unreachable!(),
    }
    Ok(s)
}
fn header(options: &CiOptions) -> String {
    let mut text = String::from(
        "# Generated by zel ci. Standalone configuration; review before enabling.\n# Requires the pinned published zel release and engine image for hosted/image builds.\n# ci run and --provision are GitHub-only. No remote settings were changed.\n",
    );
    if !options.settings.schedules.is_empty() {
        text.push_str(&format!(
            "# Create schedule triggers separately on branch {}. Requested UTC cron:\n",
            options.settings.branch
        ));
        for cron in &options.settings.schedules {
            text.push_str(&format!("#   {cron}\n"));
        }
    }
    text
}
fn serialize(options: &CiOptions, doc: Value, notes: &str) -> Result<String> {
    let yaml = serde_yaml_ng::to_string(&doc)
        .map_err(|e| invalid(&format!("cannot serialize workflow: {e}")))?;
    Ok(format!("{}{notes}{yaml}", header(options)))
}

pub fn gitlab(options: &CiOptions) -> Result<String> {
    let prepared = validate(options)?;
    let (eligible, publication) = gitlab_conditions(&options.settings);
    let delivering = !matches!(options.publish, Publish::Nothing);
    let combined = options.platform == Some(ImagePlatform::All);
    let mut doc = json!({"stages":["build","publish"], "workflow":{"rules":[{"if":eligible},{"when":"never"}]}});
    let receipt = format!(".zel-ci-{}", key(options));
    let env_name = format!("zellij/{}", key(options));
    let make = |task: Task, arch: &str| -> Result<Value> {
        let publish = !matches!(task, Task::Verify);
        let manifest = matches!(task, Task::Manifest);
        let use_docker = docker(options) || manifest;
        let mut job = json!({
            "stage":if manifest { "publish" } else { "build" }, "image":"ubuntu:24.04",
            "tags":[format!("saas-linux-small-{arch}")], "timeout":"30 minutes",
            "before_script":[format!("apt-get update -qq && DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends ca-certificates curl git{}", if use_docker { " docker.io docker-buildx" } else { "" })],
            "script":[script(options, &prepared, System::Gitlab, task, arch)?],
            "rules":if publish { json!([{"if":publication}]) } else if delivering { json!([{"if":publication,"when":"never"},{"if":eligible}]) } else { json!([{"if":eligible}]) }
        });
        if use_docker {
            job["services"] = json!([{"name":"docker:29.9.3-dind","alias":"docker","command":["--mtu=1400"],"variables":{"HEALTHCHECK_TCP_PORT":"2376"}}]);
            job["variables"] = json!({"DOCKER_HOST":"tcp://docker:2376","DOCKER_TLS_CERTDIR":"/certs","DOCKER_TLS_VERIFY":"1","DOCKER_CERT_PATH":"/certs/client"});
        }
        if publish {
            job["environment"] = json!({"name":env_name,"action":if matches!(task, Task::Stage) { "prepare" } else { "start" }});
            if !matches!(task, Task::Stage) {
                job["resource_group"] = json!(env_name);
            }
        }
        if matches!(task, Task::Stage) {
            job["artifacts"] =
                json!({"paths":[format!("{receipt}/{arch}.digest")],"expire_in":"1 day"});
        }
        Ok(job)
    };
    for arch in architectures(options) {
        let suffix = if combined {
            format!("_{arch}")
        } else {
            String::new()
        };
        doc[format!("zel_verify{suffix}")] = make(Task::Verify, arch)?;
        if delivering {
            doc[format!("zel_publish{suffix}")] =
                make(if combined { Task::Stage } else { Task::Publish }, arch)?;
        }
    }
    if combined {
        let mut job = make(Task::Manifest, "amd64")?;
        job["needs"] = json!([{"job":"zel_publish_amd64","artifacts":true},{"job":"zel_publish_arm64","artifacts":true}]);
        doc["zel_manifest"] = job;
    }
    serialize(
        options,
        doc,
        &format!(
            "# GitLab.com Linux runners; self-managed Docker executors need privileged DinD,\n# shared /builds and /certs/client volumes. Publication requires protected refs.\n# Scope protected publication variables to environment {env_name}.\n# Disable protected-variable access for merge request pipelines.\n"
        ),
    )
}

pub fn circle(options: &CiOptions) -> Result<String> {
    let prepared = validate(options)?;
    let (eligible, publication) = circle_conditions(&options.settings);
    let delivering = !matches!(options.publish, Publish::Nothing);
    let combined = options.platform == Some(ImagePlatform::All);
    let context = options
        .settings
        .publish_context
        .as_deref()
        .unwrap_or("zellij-publish");
    let receipt = format!(".zel-ci-{}", key(options));
    let mut jobs = json!({});
    let mut verify_jobs = vec![];
    let mut publish_jobs = vec![];
    let make = |task: Task, arch: &str| -> Result<Value> {
        let mut steps = vec![json!("checkout")];
        if matches!(task, Task::Manifest) {
            steps.push(json!({"attach_workspace":{"at":"."}}));
        }
        steps.push(json!({"run":{"name":"Zellij documentation", "shell":"/bin/bash -eo pipefail", "command":script(options, &prepared, System::Circle, task, arch)?, "no_output_timeout":"30m"}}));
        if matches!(task, Task::Stage) {
            steps.push(json!({"persist_to_workspace":{"root":".","paths":[format!("{receipt}/{arch}.digest")]}}));
        }
        Ok(
            json!({"machine":{"image":"ubuntu-2404:current"},"resource_class":if arch == "arm64" { "arm.medium" } else { "medium" },"steps":steps}),
        )
    };
    // Explicit tag participation also applies to prerequisites of the manifest.
    // The workflow-level logic supplies the actual event/ref allowlist.
    let filters = "pipeline.git.tag != \"\" or pipeline.git.branch != \"\"";
    for arch in architectures(options) {
        let suffix = if combined {
            format!("_{arch}")
        } else {
            String::new()
        };
        let name = format!("zel_verify{suffix}");
        jobs[&name] = make(Task::Verify, arch)?;
        verify_jobs.push(json!({name:{"filters":filters}}));
        if delivering {
            let name = format!("zel_publish{suffix}");
            jobs[&name] = make(if combined { Task::Stage } else { Task::Publish }, arch)?;
            let mut invocation = json!({"context":[context],"filters":filters});
            if !combined {
                invocation["serial-group"] = json!(format!(
                    "<< pipeline.project.slug >>/zellij-{}",
                    key(options)
                ));
            }
            publish_jobs.push(json!({name:invocation}));
        }
    }
    if combined {
        jobs["zel_manifest"] = make(Task::Manifest, "amd64")?;
        publish_jobs.push(json!({"zel_manifest":{"context":[context],"requires":["zel_publish_amd64","zel_publish_arm64"],"filters":filters,"serial-group":format!("<< pipeline.project.slug >>/zellij-{}", key(options))}}));
    }
    let mut workflows = json!({"verify":{"when":if delivering { json!({"and":[eligible,{"not":publication}]}) } else { eligible },"jobs":verify_jobs}});
    if delivering {
        workflows["publish"] = json!({"when":publication,"jobs":publish_jobs});
    }
    serialize(
        options,
        json!({"version":2.1,"jobs":jobs,"workflows":workflows}),
        &format!(
            "# CircleCI Cloud with GitHub App/OAuth integration and Linux machine executors.\n# Configure the restricted {context} context for publication jobs only.\n# Do not put publication credentials in project-wide environment variables.\n# Configure context project/ref restrictions separately; YAML is not an access policy.\n"
        ),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    fn options(publish: Publish) -> CiOptions {
        CiOptions {
            renderer: Renderer::Portable,
            site_root: "docs/Team's $manual 日本".into(),
            config: None,
            settings: Settings::default(),
            version: "0.2.0-rc.2".into(),
            publish,
            platform: None,
            features: None,
            overrides: Default::default(),
            build_args: vec![],
        }
    }
    #[test]
    fn providers_share_commands_but_not_credential_scopes() {
        let providers = [
            (
                Publish::Netlify {
                    site_id: Some("manual.netlify.app".into()),
                },
                "netlify",
                false,
            ),
            (
                Publish::Cloudflare {
                    project: Some("manual".into()),
                },
                "cloudflare",
                false,
            ),
            (
                Publish::Vercel {
                    project: Some("manual".into()),
                },
                "vercel",
                false,
            ),
            (
                Publish::GithubPages {
                    repository: Some("acme/manual".into()),
                },
                "github-pages",
                true,
            ),
            (
                Publish::S3 {
                    destination: Some("manual-bucket/docs".into()),
                },
                "s3",
                true,
            ),
            (
                Publish::Fly {
                    app: Some("manual".into()),
                    kind: ImageKind::Static,
                },
                "fly",
                true,
            ),
            (
                Publish::Railway {
                    service: None,
                    kind: ImageKind::Next,
                    tag: "ghcr.io/acme/manual:v1".into(),
                },
                "railway",
                true,
            ),
        ];
        for (publish, provider, production) in providers {
            let mut opts = options(publish);
            opts.settings.production = production;
            let gl: Value = serde_yaml_ng::from_str(&gitlab(&opts).unwrap()).unwrap();
            let cc: Value = serde_yaml_ng::from_str(&circle(&opts).unwrap()).unwrap();
            assert!(gl["zel_verify"]["environment"].is_null());
            assert!(
                gl["zel_publish"]["rules"][0]["if"]
                    .as_str()
                    .unwrap()
                    .contains("CI_COMMIT_REF_PROTECTED")
            );
            assert!(
                gl["zel_publish"]["script"][0]
                    .as_str()
                    .unwrap()
                    .contains(&format!("zel deploy {provider} "))
            );
            assert!(
                cc["jobs"]["zel_publish"]["steps"][1]["run"]["command"]
                    .as_str()
                    .unwrap()
                    .contains(&format!("zel deploy {provider} "))
            );
            assert!(cc["workflows"]["verify"]["jobs"][0]["zel_verify"]["context"].is_null());
            assert_eq!(
                cc["workflows"]["publish"]["jobs"][0]["zel_publish"]["context"][0],
                "zellij-publish"
            );
            let verify = gl["zel_verify"]["script"][0].as_str().unwrap();
            assert!(!verify.contains("docker login") && !verify.contains("zel deploy"));
            assert_eq!(
                gl["zel_verify"]["services"].is_array(),
                image(&opts.publish).is_some()
            );
        }
    }
    #[test]
    fn native_images_join_digests_after_both_successful_builds() {
        for kind in [ImageKind::Static, ImageKind::Next] {
            let mut opts = options(Publish::Image {
                kind,
                tag: "docker.io/acme/manual:final".into(),
            });
            opts.platform = Some(ImagePlatform::All);
            let gl: Value = serde_yaml_ng::from_str(&gitlab(&opts).unwrap()).unwrap();
            let cc: Value = serde_yaml_ng::from_str(&circle(&opts).unwrap()).unwrap();
            for arch in ["amd64", "arm64"] {
                assert_eq!(
                    gl[format!("zel_publish_{arch}")]["tags"][0],
                    format!("saas-linux-small-{arch}")
                );
                assert!(
                    gl[format!("zel_publish_{arch}")]["script"][0]
                        .as_str()
                        .unwrap()
                        .contains(&format!("--platform linux/{arch}"))
                );
                assert_eq!(
                    gl[format!("zel_publish_{arch}")]["environment"]["action"],
                    "prepare"
                );
                assert!(gl[format!("zel_verify_{arch}")]["environment"].is_null());
            }
            assert_eq!(gl["zel_manifest"]["needs"].as_array().unwrap().len(), 2);
            assert_eq!(
                cc["jobs"]["zel_publish_arm64"]["resource_class"],
                "arm.medium"
            );
            let manifest = gl["zel_manifest"]["script"][0].as_str().unwrap();
            assert!(manifest.contains("validate_digest \"$AMD64_IMAGE\""));
            assert!(manifest.contains("validate_digest \"$ARM64_IMAGE\""));
            assert!(!manifest.contains("zel image") && !manifest.contains("install.sh"));
        }
    }
    #[test]
    fn event_gates_preserve_branch_tags_manual_and_schedule_intent() {
        let mut opts = options(Publish::Nothing);
        opts.settings.triggers = vec![Trigger::PullRequest];
        let (eligible, publication) = gitlab_conditions(&opts.settings);
        assert!(eligible.contains("merge_request_event"));
        assert!(!publication.contains("merge_request_event"));
        let (_, publication) = circle_conditions(&opts.settings);
        assert_eq!(publication, false);
        opts.settings.triggers = vec![Trigger::Manual];
        opts.settings.tags = vec!["release/v?.*".into()];
        opts.settings.schedules = vec!["17 3 * * 1".into()];
        for yaml in [gitlab(&opts).unwrap(), circle(&opts).unwrap()] {
            assert!(yaml.contains("schedule") && yaml.contains("17 3 * * 1"));
            assert!(!yaml.contains("merge_request_event") && !yaml.contains("pull_request"));
        }
        assert_eq!(tag_pattern("release/v?.*"), "^release/v[^/]\\.[^/]*$");
        assert_eq!(tag_pattern("v**"), "^v.*$");
    }
    #[test]
    fn quoted_values_cannot_inject_shell_or_ci_expressions() {
        assert_eq!(
            quote("Team's $HOME; $(touch no)").unwrap(),
            "'Team'\\''s $HOME; $(touch no)'"
        );
        for bad in [
            "${{ secrets.X }}",
            "$[[ inputs.x ]]",
            "<< pipeline.id >>",
            "bad\npath",
        ] {
            let mut opts = options(Publish::Nothing);
            opts.site_root = bad.into();
            assert!(gitlab(&opts).is_err());
            assert!(circle(&opts).is_err());
        }
        let mut opts = options(Publish::Nothing);
        opts.renderer = Renderer::Hosted;
        let gl: Value = serde_yaml_ng::from_str(&gitlab(&opts).unwrap()).unwrap();
        assert_eq!(gl["zel_verify"]["variables"]["DOCKER_TLS_VERIFY"], "1");
        assert!(
            gl["zel_verify"]["script"][0]
                .as_str()
                .unwrap()
                .contains("${CI_BUILDS_DIR:?")
        );
        opts.platform = Some(ImagePlatform::All);
        assert!(gitlab(&opts).is_err());
    }
}
