//! Pure GitHub workflow generation. Values are serialized, never pasted into
//! executable shell text. Delivery reuses the native build-once commands.
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use zellij_core::{
    Error, Result,
    build_overrides::BuildOverrides,
    ci::{Settings, Trigger},
    delivery::{ImageKind, ImagePlatform, Provider},
    features::Features,
};

#[derive(Debug, Clone)]
pub enum Publish {
    Nothing,
    Image {
        kind: ImageKind,
        tag: String,
    },
    /// Compatibility for --target image, which historically chose GHCR.
    LegacyImage,
    Netlify {
        site_id: Option<String>,
    },
    GithubPages {
        repository: Option<String>,
    },
    Vercel {
        project: Option<String>,
    },
    Cloudflare {
        project: Option<String>,
    },
}
#[derive(Debug, Clone)]
pub struct CiOptions {
    pub site_root: String,
    pub config: Option<String>,
    pub settings: Settings,
    pub version: String,
    pub publish: Publish,
    pub platform: Option<ImagePlatform>,
    pub features: Option<Features>,
    pub overrides: BuildOverrides,
    pub build_args: Vec<zellij_core::build_args::BuildArg>,
}

fn invalid(message: &str) -> Error {
    Error::InvalidArgument(format!("ci: {message}"))
}
fn literal(value: &str) -> Result<()> {
    if value.chars().any(char::is_control) || value.contains("${{") {
        return Err(invalid(
            "paths and values cannot contain control characters or GitHub expressions",
        ));
    }
    Ok(())
}
/// GitHub path filters use glob syntax independently of YAML quoting.
fn path_pattern(path: &str) -> String {
    let mut out = String::new();
    for c in path.chars() {
        if "\\*?[]!+".contains(c) {
            out.push('\\');
        }
        out.push(c);
    }
    out
}
fn install(version: &str) -> Value {
    json!({
        "name": "Install pinned zel",
        "env": {"ZEL_VERSION": version, "ZEL_INSTALL_DIR": "${{ runner.temp }}/zel-bin"},
        "run": "set -eu\nbase=\"https://github.com/mylife-inc/releases/releases/download/zellij-$ZEL_VERSION\"\ncurl --fail --silent --show-error --location \"$base/install.sh\" --output \"$RUNNER_TEMP/zel-install.sh\" || { echo 'The pinned zel release must be published before this workflow can run.' >&2; exit 1; }\nsh \"$RUNNER_TEMP/zel-install.sh\"\nprintf '%s\\n' \"$ZEL_INSTALL_DIR\" >> \"$GITHUB_PATH\"\n"
    })
}
fn initial_steps(version: &str, selector: &str, no_check: bool) -> Vec<Value> {
    let mut steps = vec![
        json!({"uses":"actions/checkout@v7", "with":{"persist-credentials": false}}),
        install(version),
    ];
    if !no_check {
        steps.push(json!({"name":"Validate documentation", "run":format!("zel check {selector} --strict --deep")}));
    }
    steps
}
fn publication_condition(settings: &Settings) -> String {
    let branch = format!("github.ref == 'refs/heads/{}'", settings.branch);
    let mut clauses = vec![];
    if settings.triggers.contains(&Trigger::Push) {
        clauses.push(format!("(github.event_name == 'push' && {branch})"));
    }
    if settings.triggers.contains(&Trigger::Manual) {
        clauses.push(format!(
            "(github.event_name == 'workflow_dispatch' && {branch})"
        ));
    }
    if !settings.tags.is_empty() {
        // The push.tags filter is the allowlist for tag events.
        clauses.push("(github.event_name == 'push' && github.ref_type == 'tag')".into());
    }
    if !settings.schedules.is_empty() {
        clauses.push(format!("(github.event_name == 'schedule' && {branch})"));
    }
    if clauses.is_empty() {
        "false".into()
    } else {
        clauses.join(" || ")
    }
}

pub fn github(options: &CiOptions) -> Result<String> {
    let CiOptions {
        site_root,
        config,
        settings,
        version,
        publish,
        platform,
        features,
        overrides,
        build_args,
    } = options;
    if !build_args.is_empty() && !matches!(publish, Publish::Image { .. } | Publish::LegacyImage) {
        return Err(invalid("--build-arg requires image CI delivery"));
    }
    if platform.is_some() && !matches!(publish, Publish::Image { .. } | Publish::LegacyImage) {
        return Err(invalid("platform requires image delivery"));
    }
    // Use native runners: Node/Next RUN steps have previously crashed under
    // QEMU in our release builds. Local cross-builds still use the caller's builder.
    let runner = if *platform == Some(ImagePlatform::Arm64) {
        "ubuntu-24.04-arm"
    } else {
        "ubuntu-latest"
    };
    let platform_flag = platform
        .map(|p| format!(" --platform {p}"))
        .unwrap_or_default();
    let features_flag = features
        .map(|features| format!(" --features {}", features.as_str()))
        .unwrap_or_default();
    settings.validate()?;
    if matches!(publish, Publish::GithubPages { .. }) {
        Provider::GithubPages.require_production(settings.production)?;
        if settings.branch == "gh-pages" {
            return Err(invalid(
                "GitHub Pages CI must run from a source branch other than gh-pages",
            ));
        }
    }
    let version = zellij_core::ci::version_pin(version)?;
    if (!overrides.is_empty()
        || !build_args.is_empty()
        || *platform == Some(ImagePlatform::All)
        || matches!(
            publish,
            Publish::Cloudflare { .. } | Publish::Vercel { .. } | Publish::GithubPages { .. }
        ))
        && !semver::Version::parse(version.trim_start_matches('v'))
            .is_ok_and(|version| version >= semver::Version::parse("0.2.0-rc.2").unwrap())
    {
        return Err(invalid(
            "Cloudflare/Vercel/GitHub Pages deployment, build overrides, image build arguments and combined image publication require a CLI release of 0.2.0-rc.2 or later; generation does not verify that the release has been published",
        ));
    }
    let mut override_flags = String::new();
    let mut override_env = serde_json::Map::new();
    for (i, arg) in zellij_core::build_args::normalize(build_args)
        .iter()
        .enumerate()
    {
        literal(arg.assignment())?;
        let key = format!("ZEL_IMAGE_ARG_{i}");
        override_env.insert(key.clone(), json!(arg.assignment()));
        override_flags.push_str(&format!(" --build-arg \"${key}\""));
    }
    if let Some(prefix) = &overrides.base_path {
        let prefix = prefix.to_string();
        literal(&prefix)?;
        override_env.insert(
            "ZEL_BUILD_BASE_PATH".into(),
            json!(if prefix.is_empty() { "/" } else { &prefix }),
        );
        override_flags.push_str(" --base-path \"$ZEL_BUILD_BASE_PATH\"");
    }
    if let Some(origin) = &overrides.site_url {
        let origin = origin.to_string();
        literal(&origin)?;
        override_env.insert("ZEL_BUILD_SITE_URL".into(), json!(origin));
        override_flags.push_str(" --site-url \"$ZEL_BUILD_SITE_URL\"");
    }
    if overrides.no_check {
        override_flags.push_str(" --no-check");
    }
    literal(site_root)?;
    if let Some(config) = config {
        literal(config)?;
        zellij_core::files::relative_path(camino::Utf8Path::new(config))?;
        let parent = camino::Utf8Path::new(config)
            .parent()
            .unwrap_or(camino::Utf8Path::new(""));
        if parent != camino::Utf8Path::new(if site_root == "." { "" } else { site_root }) {
            return Err(invalid(
                "configuration must be directly inside the selected site root",
            ));
        }
    }
    let selector = if config.is_some() {
        "--config \"$SITE_CONFIG\""
    } else {
        "\"$SITE_ROOT\""
    };
    if site_root != "." {
        zellij_core::files::relative_path(camino::Utf8Path::new(site_root))?;
    }
    let path = if site_root == "." {
        "**".into()
    } else {
        format!("{}/**", path_pattern(site_root))
    };
    let paths = json!([path, format!(".github/workflows/{}", settings.workflow)]);
    let mut events = json!({});
    if settings.triggers.contains(&Trigger::PullRequest) {
        events["pull_request"] = json!({"paths": paths});
    }
    if settings.triggers.contains(&Trigger::Push) || !settings.tags.is_empty() {
        let branches = if settings.triggers.contains(&Trigger::Push) {
            vec![settings.branch.clone()]
        } else {
            vec![]
        };
        events["push"] = json!({"paths": paths});
        if !branches.is_empty() {
            events["push"]["branches"] = json!(branches);
        }
        if !settings.tags.is_empty() {
            events["push"]["tags"] = json!(settings.tags);
        }
    }
    if settings.triggers.contains(&Trigger::Manual) {
        events["workflow_dispatch"] = json!({});
    }
    if !settings.schedules.is_empty() {
        events["schedule"] = json!(
            settings
                .schedules
                .iter()
                .map(|cron| json!({"cron":cron}))
                .collect::<Vec<_>>()
        );
    }
    let condition = publication_condition(settings);
    let delivering = !matches!(publish, Publish::Nothing);
    let mut steps = initial_steps(&version, selector, overrides.no_check);
    let build = match publish {
        Publish::Image { kind, .. } => format!(
            "zel image {} {selector} --tag zel-ci-check:local",
            kind.as_str()
        ),
        Publish::LegacyImage => format!("zel image next {selector} --tag zel-ci-check:local"),
        _ => format!("zel build {selector} --out \"$RUNNER_TEMP/zellij-site\""),
    };
    steps.push(json!({"name":"Build without publishing", "run":format!("{build}{platform_flag}{features_flag}{override_flags}")}));
    let mut jobs = json!({"verify": {"runs-on":runner, "timeout-minutes":30, "steps": steps}});
    if delivering {
        jobs["verify"]["if"] = json!(format!("${{{{ !({condition}) }}}}"));
        let mut steps = initial_steps(&version, selector, overrides.no_check);
        let mut permissions = json!({"contents":"read"});
        match publish {
            Publish::Image { .. } | Publish::LegacyImage => {
                let tag = match publish {
                    Publish::Image { tag, .. } => Some(tag.as_str()),
                    _ => None,
                };
                let registry = match tag {
                    Some(value) => {
                        zellij_core::delivery::validate_image_reference(value)?;
                        let (registry, _) = value.split_once('/').ok_or_else(|| invalid("CI image publication requires an explicit registry hostname in image.name"))?;
                        if !registry.contains('.')
                            || registry == "localhost"
                            || registry.starts_with("127.")
                        {
                            return Err(invalid(
                                "CI image publication requires an explicit remote registry hostname in image.name",
                            ));
                        }
                        registry
                    }
                    None => "ghcr.io",
                };
                let ghcr = registry == "ghcr.io";
                if ghcr {
                    permissions["packages"] = json!("write");
                }
                steps.push(json!({"name":"Sign in to registry", "uses":"docker/login-action@v3", "with": {
                    "registry": registry,
                    "username": if ghcr { "${{ github.actor }}" } else { "${{ secrets.REGISTRY_USERNAME }}" },
                    "password": if ghcr { "${{ secrets.GITHUB_TOKEN }}" } else { "${{ secrets.REGISTRY_PASSWORD }}" }
                }}));
                if let Some(tag) = tag {
                    steps.push(json!({"name":"Build once and publish", "env":{"IMAGE_TAG": tag}, "run":format!("zel publish {selector} --tag \"$IMAGE_TAG\"{platform_flag}{features_flag}{override_flags}")}));
                } else {
                    steps.push(json!({"name":"Build once and publish", "env":{"REPOSITORY":"${{ github.repository }}", "REVISION":"${{ github.sha }}"}, "run":"TAG=\"ghcr.io/${REPOSITORY,,}/docs:$REVISION\"\nzel image next \"$SITE_ROOT\" --tag \"$TAG\"\nzel image push \"$TAG\"\n".replace("\"$SITE_ROOT\"", selector).replace("--tag \"$TAG\"", &format!("--tag \"$TAG\"{platform_flag}{features_flag}{override_flags}"))}));
                }
            }
            Publish::Netlify { site_id } => {
                if let Some(id) = site_id {
                    zellij_core::delivery::validate_site_identifier(id)?;
                }
                let prod = if settings.production { " --prod" } else { "" };
                steps.push(json!({"name":"Build once and deploy", "env":{
                    "NETLIFY_SITE_ID": site_id.as_deref().unwrap_or("${{ vars.NETLIFY_SITE_ID || secrets.NETLIFY_SITE_ID }}"),
                    "NETLIFY_AUTH_TOKEN":"${{ secrets.NETLIFY_AUTH_TOKEN }}"
                }, "run":format!("zel deploy netlify {selector}{prod}{features_flag}{override_flags}")}));
            }
            Publish::GithubPages { repository } => {
                if let Some(repository) = repository {
                    Provider::GithubPages.validate_site(repository)?;
                }
                steps.push(json!({"name":"Build once and publish GitHub Pages", "env":{
                    "GITHUB_PAGES_REPOSITORY":repository.as_deref().unwrap_or("${{ vars.GITHUB_PAGES_REPOSITORY || secrets.GITHUB_PAGES_REPOSITORY }}"),
                    "GITHUB_PAGES_TOKEN":"${{ secrets.GITHUB_PAGES_TOKEN }}"
                },"run":format!("zel deploy github-pages {selector} --prod{features_flag}{override_flags}")}));
            }
            Publish::Vercel { project } => {
                if let Some(project) = project {
                    Provider::Vercel.validate_site(project)?;
                }
                let prod = if settings.production { " --prod" } else { "" };
                steps.push(json!({"name":"Build once and deploy", "env":{
                    "VERCEL_PROJECT_ID": project.as_deref().unwrap_or("${{ vars.VERCEL_PROJECT_ID || secrets.VERCEL_PROJECT_ID }}"),
                    "VERCEL_TEAM_ID":"${{ vars.VERCEL_TEAM_ID || secrets.VERCEL_TEAM_ID }}",
                    "VERCEL_TOKEN":"${{ secrets.VERCEL_TOKEN }}"
                }, "run":format!("zel deploy vercel {selector}{prod}{features_flag}{override_flags}")}));
            }
            Publish::Cloudflare { project } => {
                if let Some(project) = project {
                    Provider::Cloudflare.validate_site(project)?;
                }
                let prod = if settings.production { " --prod" } else { "" };
                steps.push(json!({"name":"Build once and deploy", "env":{
                    "CLOUDFLARE_PAGES_PROJECT_NAME": project.as_deref().unwrap_or("${{ vars.CLOUDFLARE_PAGES_PROJECT_NAME || secrets.CLOUDFLARE_PAGES_PROJECT_NAME }}"),
                    "CLOUDFLARE_ACCOUNT_ID":"${{ vars.CLOUDFLARE_ACCOUNT_ID || secrets.CLOUDFLARE_ACCOUNT_ID }}",
                    "CLOUDFLARE_API_TOKEN":"${{ secrets.CLOUDFLARE_API_TOKEN }}"
                }, "run":format!("zel deploy cloudflare {selector}{prod}{features_flag}{override_flags}")}));
            }
            Publish::Nothing => unreachable!(),
        }
        jobs["publish"] = json!({"if":format!("${{{{ {condition} }}}}"), "runs-on":runner, "timeout-minutes":30, "permissions":permissions, "steps":steps});
    }
    if *platform == Some(ImagePlatform::All) {
        combined_jobs(
            &mut jobs,
            publish,
            selector,
            &format!("{features_flag}{override_flags}"),
        )?;
    }
    // GitHub group names are case-insensitive. Hash the case-sensitive filename
    // so even Manual.yml and manual.yml have distinct, bounded-length keys.
    let workflow_key = format!("{:x}", Sha256::digest(settings.workflow.as_bytes()));
    let mut doc = json!({
        "name":format!("Documentation ({})", settings.workflow), "on": events, "permissions":{"contents":"read"},
        "defaults":{"run":{"shell":"bash"}},
        // The file identifies the workflow even if its display name is edited.
        // A shared display name previously coupled independent sites' queues.
        "concurrency":{"group":format!("docs-{workflow_key}-${{{{ github.ref }}}}"), "cancel-in-progress":false},
        "env":{"SITE_ROOT": if site_root == "." { ".".into() } else { format!("./{site_root}") }, "ZELLIJ_IMAGE":format!("ghcr.io/mylife-inc/zellij-base:{}", version.trim_start_matches('v'))},
        "jobs":jobs
    });
    if let Some(config) = config {
        doc["env"]["SITE_CONFIG"] = json!(format!("./{config}"));
    }
    doc["env"].as_object_mut().unwrap().extend(override_env);
    let yaml = serde_yaml_ng::to_string(&doc)
        .map_err(|e| invalid(&format!("cannot serialize workflow: {e}")))?;
    Ok(format!(
        "# Generated by zel ci. Review before enabling; regenerate with --force.\n# Requires the pinned compatible zel release and, for hosted/image builds, its engine image.\n# Generation does not check release availability or provision credentials.\n{yaml}"
    ))
}

/// Native builds avoid emulating Node/Next. Publish the final tag only after
/// both jobs succeed, using immutable digests rather than mutable staging tags.
fn combined_jobs(jobs: &mut Value, publish: &Publish, selector: &str, flags: &str) -> Result<()> {
    let (kind, tag, repository) = match publish {
        Publish::Image { kind, tag } => {
            // The registry hostname was validated before reaching this point.
            let (prefix, name) = tag.rsplit_once('/').unwrap();
            let name = name.split(':').next().unwrap();
            (*kind, Some(tag.as_str()), Some(format!("{prefix}/{name}")))
        }
        Publish::LegacyImage => (ImageKind::Next, None, None),
        _ => return Err(invalid("combined platforms require image publication")),
    };
    let mut template = jobs["publish"].take();
    template["steps"].as_array_mut().unwrap().pop();
    let login = template["steps"]
        .as_array()
        .unwrap()
        .last()
        .unwrap()
        .clone();
    // PRs validate each target, with neither credentials nor publication steps.
    jobs["verify"]["runs-on"] = json!("${{ matrix.runner }}");
    jobs["verify"]["strategy"] = json!({"fail-fast":false,"matrix":{"include":[
        {"runner":"ubuntu-latest","platform":"linux/amd64"},
        {"runner":"ubuntu-24.04-arm","platform":"linux/arm64"}
    ]}});
    let verify = jobs["verify"]["steps"]
        .as_array_mut()
        .unwrap()
        .last_mut()
        .unwrap();
    verify["env"] = json!({"IMAGE_PLATFORM":"${{ matrix.platform }}"});
    verify["run"] = json!(
        verify["run"]
            .as_str()
            .unwrap()
            .replace(" --platform all", " --platform \"$IMAGE_PLATFORM\"")
    );
    for (arch, runner) in [("amd64", "ubuntu-latest"), ("arm64", "ubuntu-24.04-arm")] {
        let mut job = template.clone();
        job["runs-on"] = json!(runner);
        job["outputs"] = json!({"image":"${{ steps.image.outputs.image }}"});
        let normalize = if repository.is_none() {
            "IMAGE_REPOSITORY=\"ghcr.io/${REPOSITORY,,}/docs\"\n"
        } else {
            ""
        };
        let script = format!(
            "set -eu\n{normalize}BUILD_TAG=\"$IMAGE_REPOSITORY:zel-${{GITHUB_RUN_ID}}-${{GITHUB_RUN_ATTEMPT}}-{arch}\"\nzel image {} {selector} --tag \"$BUILD_TAG\" --platform linux/{arch}{flags}\nzel image push \"$BUILD_TAG\"\nIMAGE_DIGEST=$(docker image inspect --format '{{{{index .RepoDigests 0}}}}' \"$BUILD_TAG\")\ncase \"$IMAGE_DIGEST\" in \"$IMAGE_REPOSITORY\"@sha256:*) ;; *) echo 'Registry digest missing after push' >&2; exit 1;; esac\nprintf 'image=%s\\n' \"$IMAGE_DIGEST\" >> \"$GITHUB_OUTPUT\"\n",
            kind.as_str()
        );
        let env = match &repository {
            Some(repository) => json!({"IMAGE_REPOSITORY":repository}),
            None => json!({"REPOSITORY":"${{ github.repository }}"}),
        };
        job["steps"].as_array_mut().unwrap().push(
            json!({"name":format!("Build and publish {arch}"),"id":"image","env":env,"run":script}),
        );
        jobs[format!("publish_{arch}")] = job;
    }
    let mut env = json!({
        "AMD64_IMAGE":"${{ needs.publish_amd64.outputs.image }}",
        "ARM64_IMAGE":"${{ needs.publish_arm64.outputs.image }}"
    });
    let mut script = "set -eu\n".to_owned();
    if let Some(tag) = tag {
        env["IMAGE_TAG"] = json!(tag);
    } else {
        env["REPOSITORY"] = json!("${{ github.repository }}");
        env["REVISION"] = json!("${{ github.sha }}");
        script.push_str("IMAGE_TAG=\"ghcr.io/${REPOSITORY,,}/docs:$REVISION\"\n");
    }
    script.push_str("docker buildx imagetools create --tag \"$IMAGE_TAG\" \"$AMD64_IMAGE\" \"$ARM64_IMAGE\"\ndocker buildx imagetools inspect \"$IMAGE_TAG\"\n");
    jobs["publish"] = json!({
        "needs":["publish_amd64","publish_arm64"],
        "if":template["if"], "runs-on":"ubuntu-latest", "timeout-minutes":10,
        "permissions":template["permissions"],
        "steps":[login,{"name":"Publish combined image tag","env":env,"run":script}]
    });
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn options(publish: Publish) -> CiOptions {
        CiOptions {
            site_root: "docs/Team's [manual] 日本".into(),
            config: None,
            settings: Settings::default(),
            version: "0.2.0-rc.1".into(),
            publish,
            platform: None,
            features: None,
            overrides: BuildOverrides::default(),
            build_args: Vec::new(),
        }
    }

    #[test]
    fn workflow_queues_preserve_case_sensitive_file_identity() {
        let mut options = options(Publish::Nothing);
        options.settings.workflow = "Manual.yml".into();
        let upper: Value = serde_yaml_ng::from_str(&github(&options).unwrap()).unwrap();
        options.settings.workflow = "manual.yml".into();
        let lower: Value = serde_yaml_ng::from_str(&github(&options).unwrap()).unwrap();
        let group = |workflow: &Value| {
            workflow["concurrency"]["group"]
                .as_str()
                .unwrap()
                .to_lowercase()
        };
        assert_ne!(group(&upper), group(&lower));
    }
    fn parse(options: &CiOptions) -> Value {
        serde_yaml_ng::from_str(&github(options).unwrap()).unwrap()
    }
    #[test]
    fn default_and_delivery_workflows_build_once_with_scoped_credentials() {
        let plain = parse(&options(Publish::Nothing));
        assert!(plain["jobs"].get("publish").is_none());
        assert!(plain["jobs"]["verify"].get("if").is_none());
        assert!(!plain.to_string().contains("secrets."));
        for publication in [
            Publish::Image {
                kind: ImageKind::Static,
                tag: "ghcr.io/acme/docs:stable".into(),
            },
            Publish::Image {
                kind: ImageKind::Next,
                tag: "registry.example/acme/docs:stable".into(),
            },
            Publish::LegacyImage,
            Publish::Netlify {
                site_id: Some("site-id".into()),
            },
        ] {
            let options = options(publication);
            let doc = parse(&options);
            let verify = &doc["jobs"]["verify"];
            assert!(!verify.to_string().contains("secrets."));
            let publish = &doc["jobs"]["publish"];
            assert!(
                publish["if"]
                    .as_str()
                    .unwrap()
                    .contains("github.ref == 'refs/heads/main'")
            );
            assert!(
                !publish["if"]
                    .as_str()
                    .unwrap()
                    .contains("!= 'pull_request'")
            );
            let steps = publish["steps"].as_array().unwrap();
            let commands: Vec<_> = steps.iter().filter_map(|s| s["run"].as_str()).collect();
            assert_eq!(
                commands
                    .iter()
                    .filter(|s| s.starts_with("zel check"))
                    .count(),
                1
            );
            assert!(!commands.iter().any(|s| s.starts_with("zel build")));
            assert!(!commands.iter().any(|s| s.contains("npx")));
            assert!(!verify.to_string().contains("login-action"));
            assert_eq!(doc["permissions"], json!({"contents":"read"}));
        }
    }
    #[test]
    fn triggers_paths_and_version_are_pinned_and_preserved_as_data() {
        let mut options = options(Publish::Netlify { site_id: None });
        options.settings.branch = "release/docs".into();
        options.settings.triggers = vec![Trigger::PullRequest, Trigger::Manual];
        options.settings.tags = vec!["v*".into()];
        options.settings.schedules = vec!["17 3 * * 1".into()];
        options.settings.production = true;
        options.settings.workflow = "manual.yml".into();
        let doc = parse(&options);
        assert_eq!(doc["env"]["SITE_ROOT"], "./docs/Team's [manual] 日本");
        assert_eq!(
            doc["on"]["pull_request"]["paths"],
            json!([
                "docs/Team's \\[manual\\] 日本/**",
                ".github/workflows/manual.yml"
            ])
        );
        assert_eq!(doc["on"]["push"]["tags"], json!(["v*"]));
        assert!(doc["on"]["push"].get("branches").is_none());
        assert_eq!(doc["on"]["schedule"][0]["cron"], "17 3 * * 1");
        let steps = doc["jobs"]["publish"]["steps"].as_array().unwrap();
        assert_eq!(steps[1]["env"]["ZEL_VERSION"], "v0.2.0-rc.1");
        assert!(
            steps[1]["run"]
                .as_str()
                .unwrap()
                .contains("releases/download/zellij-$ZEL_VERSION")
        );
        assert_eq!(
            steps.last().unwrap()["run"],
            "zel deploy netlify \"$SITE_ROOT\" --prod"
        );
        assert!(
            doc["jobs"]["publish"]["if"]
                .as_str()
                .unwrap()
                .contains("github.ref_type == 'tag'")
        );
        assert!(
            doc["jobs"]["publish"]["if"]
                .as_str()
                .unwrap()
                .contains("refs/heads/release/docs")
        );
        options.site_root = "docs/${{ secrets.TOKEN }}".into();
        assert!(github(&options).is_err());
        options.site_root = "../docs".into();
        assert!(github(&options).is_err());
    }
}
