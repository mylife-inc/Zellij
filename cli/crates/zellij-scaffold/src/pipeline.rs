//! Pure GitHub workflow generation. Values are serialized, never pasted into
//! executable shell text. Delivery reuses the native build-once commands.
use serde_json::{Value, json};
use zellij_core::{
    Error, Result,
    ci::{Settings, Trigger},
    delivery::{ImageKind, ImagePlatform},
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
}
#[derive(Debug, Clone)]
pub struct CiOptions {
    pub site_root: String,
    pub config: Option<String>,
    pub settings: Settings,
    pub version: String,
    pub publish: Publish,
    pub platform: Option<ImagePlatform>,
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
fn initial_steps(version: &str, selector: &str) -> Vec<Value> {
    vec![
        json!({"uses":"actions/checkout@v7", "with":{"persist-credentials": false}}),
        install(version),
        json!({"name":"Validate documentation", "run":format!("zel check {selector} --strict --deep")}),
    ]
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
    } = options;
    if platform.is_some() && !matches!(publish, Publish::Image { .. } | Publish::LegacyImage) {
        return Err(invalid("platform requires image delivery"));
    }
    let platform_flag = platform
        .map(|p| format!(" --platform {p}"))
        .unwrap_or_default();
    settings.validate()?;
    let version = zellij_core::ci::version_pin(version)?;
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
    let mut steps = initial_steps(&version, selector);
    if *platform == Some(ImagePlatform::Arm64) {
        steps.push(json!({"name":"Enable ARM64 image builds", "uses":"docker/setup-qemu-action@v4", "with":{"platforms":"arm64"}}));
    }
    let build = match publish {
        Publish::Image { kind, .. } => format!(
            "zel image {} {selector} --tag zel-ci-check:local",
            kind.as_str()
        ),
        Publish::LegacyImage => format!("zel image next {selector} --tag zel-ci-check:local"),
        _ => format!("zel build {selector} --out \"$RUNNER_TEMP/zellij-site\""),
    };
    steps.push(json!({"name":"Build without publishing", "run":format!("{build}{platform_flag}")}));
    let mut jobs =
        json!({"verify": {"runs-on":"ubuntu-latest", "timeout-minutes":30, "steps": steps}});
    if delivering {
        jobs["verify"]["if"] = json!(format!("${{{{ !({condition}) }}}}"));
        let mut steps = initial_steps(&version, selector);
        if *platform == Some(ImagePlatform::Arm64) {
            steps.push(json!({"name":"Enable ARM64 image builds", "uses":"docker/setup-qemu-action@v4", "with":{"platforms":"arm64"}}));
        }
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
                    steps.push(json!({"name":"Build once and publish", "env":{"IMAGE_TAG": tag}, "run":format!("zel publish {selector} --tag \"$IMAGE_TAG\"{platform_flag}")}));
                } else {
                    steps.push(json!({"name":"Build once and publish", "env":{"REPOSITORY":"${{ github.repository }}", "REVISION":"${{ github.sha }}"}, "run":"TAG=\"ghcr.io/${REPOSITORY,,}/docs:$REVISION\"\nzel image next \"$SITE_ROOT\" --tag \"$TAG\"\nzel image push \"$TAG\"\n".replace("\"$SITE_ROOT\"", selector).replace("--tag \"$TAG\"", &format!("--tag \"$TAG\"{platform_flag}"))}));
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
                }, "run":format!("zel deploy netlify {selector}{prod}")}));
            }
            Publish::Nothing => unreachable!(),
        }
        jobs["publish"] = json!({"if":format!("${{{{ {condition} }}}}"), "runs-on":"ubuntu-latest", "timeout-minutes":30, "permissions":permissions, "steps":steps});
    }
    let mut doc = json!({
        "name":"Documentation", "on": events, "permissions":{"contents":"read"},
        "defaults":{"run":{"shell":"bash"}},
        "concurrency":{"group":"docs-${{ github.workflow }}-${{ github.ref }}", "cancel-in-progress":false},
        "env":{"SITE_ROOT": if site_root == "." { ".".into() } else { format!("./{site_root}") }, "ZELLIJ_IMAGE":format!("ghcr.io/mylife-inc/zellij-base:{}", version.trim_start_matches('v'))},
        "jobs":jobs
    });
    if let Some(config) = config {
        doc["env"]["SITE_CONFIG"] = json!(format!("./{config}"));
    }
    let yaml = serde_yaml_ng::to_string(&doc)
        .map_err(|e| invalid(&format!("cannot serialize workflow: {e}")))?;
    Ok(format!(
        "# Generated by zel ci. Review before enabling; regenerate with --force.\n# Requires the pinned compatible zel release and, for hosted/image builds, its engine image.\n# Generation does not check release availability or provision credentials.\n{yaml}"
    ))
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
        }
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
