//! Versioned site roots and explicit legacy content folders. Discovery never
//! scans a repository or walks its parents.
use camino::{Utf8Path, Utf8PathBuf};
use serde::Deserialize;
use serde_yaml_ng::{Mapping, Value};
use std::{fs, io};

use crate::{Error, Project, Result, files};

pub const CONFIG: &str = "zel.yaml";

/// A directory selection and an explicit file selection remain distinct even
/// when both point at zel.yaml. An explicit file may never fall back to discovery.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SiteInput {
    Directory(Utf8PathBuf),
    Config(Utf8PathBuf),
}
impl SiteInput {
    pub fn path(&self) -> &Utf8Path {
        match self {
            Self::Directory(path) | Self::Config(path) => path,
        }
    }
    pub fn root(&self) -> &Utf8Path {
        match self {
            Self::Directory(path) => path,
            Self::Config(path) => path
                .parent()
                .filter(|p| !p.as_str().is_empty())
                .unwrap_or(Utf8Path::new(".")),
        }
    }
    pub fn config_path(&self) -> Utf8PathBuf {
        match self {
            Self::Directory(path) => path.join(CONFIG),
            Self::Config(path) => path.clone(),
        }
    }
    pub fn explicit_config(&self) -> bool {
        matches!(self, Self::Config(_))
    }
    pub fn has_config(&self) -> bool {
        self.explicit_config() || present(self.root())
    }
}
impl std::fmt::Display for SiteInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.path().fmt(f)
    }
}
impl std::str::FromStr for SiteInput {
    type Err = std::convert::Infallible;
    fn from_str(value: &str) -> std::result::Result<Self, Self::Err> {
        Ok(Self::from(value))
    }
}
impl From<&str> for SiteInput {
    fn from(value: &str) -> Self {
        Self::Directory(value.into())
    }
}
impl From<Utf8PathBuf> for SiteInput {
    fn from(value: Utf8PathBuf) -> Self {
        Self::Directory(value)
    }
}
impl From<&Utf8PathBuf> for SiteInput {
    fn from(value: &Utf8PathBuf) -> Self {
        Self::Directory(value.clone())
    }
}
impl From<&Utf8Path> for SiteInput {
    fn from(value: &Utf8Path) -> Self {
        Self::Directory(value.to_owned())
    }
}
impl From<&SiteInput> for SiteInput {
    fn from(value: &SiteInput) -> Self {
        value.clone()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Renderer {
    Portable,
    Hosted,
}

#[derive(Debug)]
pub struct Site {
    pub input: SiteInput,
    pub config_path: Utf8PathBuf,
    pub root: Utf8PathBuf,
    pub content: Utf8PathBuf,
    pub configured: bool,
    pub renderer: Renderer,
    pub output: Utf8PathBuf,
    pub port: u16,
    pub image: Option<crate::delivery::ImageSettings>,
    pub publish: Option<crate::delivery::PublishSettings>,
    pub ci: crate::ci::Settings,
    /// Remaining portable presentation settings, validated by the renderer.
    pub portable: Value,
}

pub fn present(root: &Utf8Path) -> bool {
    // Invalid entries, including dangling links, must report their errors.
    fs::symlink_metadata(root.join(CONFIG)).is_ok()
}

pub fn select(cwd: &Utf8Path, explicit: Option<&str>) -> Result<Utf8PathBuf> {
    if let Some(path) = explicit {
        return Ok(cwd.join(path));
    }
    for root in [cwd.to_owned(), cwd.join("docs/zellij")] {
        if present(&root) {
            return Ok(root);
        }
    }
    Err(Error::InvalidArgument(format!(
        "no zel.yaml in the current directory or {}; create a site with `zel new portable`, or pass a site/content folder explicitly (for example `zel build docs/manual` or `zel build .`)",
        cwd.join("docs/zellij/zel.yaml")
    )))
}

pub fn is_link(metadata: &fs::Metadata) -> bool {
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        metadata.file_type().is_symlink() || metadata.file_attributes() & 0x400 != 0
    }
    #[cfg(not(windows))]
    metadata.file_type().is_symlink()
}

fn invalid(root: &Utf8Path, message: impl std::fmt::Display) -> Error {
    Error::InvalidArgument(format!("{}: {message}", root))
}

fn parse<T: serde::de::DeserializeOwned>(root: &Utf8Path, value: Value) -> Result<T> {
    serde_yaml_ng::from_value(value).map_err(|e| invalid(root, e))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Build {
    out: Value,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Preview {
    port: u16,
}

impl Site {
    /// Load one selected root. A versioned config always uses content/; a
    /// missing content directory is an error, never permission to scan root.
    pub fn load(input: impl Into<SiteInput>) -> Result<Self> {
        let input = input.into();
        let root = input.root().canonicalize_utf8()?;
        let config = if input.explicit_config() {
            let path = input.config_path();
            let name = path
                .file_name()
                .ok_or_else(|| files::unsafe_path(&path, "expected a configuration filename"))?;
            root.join(name)
        } else {
            root.join(CONFIG)
        };
        let input = if input.explicit_config() {
            SiteInput::Config(config.clone())
        } else {
            SiteInput::Directory(root.clone())
        };
        if !root.is_dir() {
            return Err(files::unsafe_path(
                &root,
                "expected a site/content directory",
            ));
        }
        let mut settings = match fs::symlink_metadata(&config) {
            Err(e) if e.kind() == io::ErrorKind::NotFound && !input.explicit_config() => None,
            Err(e) if input.explicit_config() => return Err(invalid(&config, e)),
            Err(e) => return Err(e.into()),
            Ok(meta) => {
                if is_link(&meta) || !meta.is_file() {
                    return Err(invalid(
                        &config,
                        "expected a regular file, not a symlink or junction",
                    ));
                }
                Some(
                    serde_yaml_ng::from_str::<Mapping>(&fs::read_to_string(&config)?)
                        .map_err(|e| invalid(&config, e))?,
                )
            }
        };
        let configured = settings.as_ref().is_some_and(|s| s.contains_key("version"));
        let mut port = 3000;
        let mut image = None;
        let mut publish = None;
        let mut ci = crate::ci::Settings::default();
        let (content, renderer, output) = if configured {
            let settings = settings.as_mut().unwrap();
            if settings.remove("version").and_then(|v| v.as_u64()) != Some(1) {
                return Err(invalid(&config, "version must be the integer 1"));
            }
            let renderer: Renderer = parse(
                &config,
                settings.remove("renderer").ok_or_else(|| {
                    invalid(&config, "version: 1 requires renderer: portable or hosted")
                })?,
            )?;
            let content = root.join("content");
            let meta = fs::symlink_metadata(&content).map_err(|e| {
                invalid(
                    &config,
                    format!("content/ must be an existing directory: {e}"),
                )
            })?;
            if is_link(&meta) || !meta.is_dir() {
                return Err(invalid(
                    &config,
                    "content/ must be a directory, not a symlink or junction",
                ));
            }
            let output = match settings.remove("build") {
                Some(value) => {
                    let build: Build = parse(&config, value)?;
                    let path = build
                        .out
                        .as_str()
                        .filter(|s| !s.trim().is_empty())
                        .ok_or_else(|| {
                            invalid(&config, "build.out must be a nonempty path string")
                        })?;
                    root.join(path)
                }
                None => root.join("site"),
            };
            if let Some(value) = settings.remove("preview") {
                port = parse::<Preview>(&config, value)?.port;
            }
            image = settings
                .remove("image")
                .map(|value| parse(&config, value))
                .transpose()?;
            publish = settings
                .remove("publish")
                .map(|value| parse(&config, value))
                .transpose()?;
            if let Some(value) = settings.remove("ci") {
                ci = parse(&config, value)?;
                ci.validate()?;
            }
            if renderer == Renderer::Hosted && !settings.is_empty() {
                return Err(invalid(
                    &config,
                    "hosted zel.yaml supports version, renderer, build, preview, image, publish and ci only; put presentation settings in content/site.yaml",
                ));
            }
            (content, renderer, output)
        } else {
            if settings.as_ref().is_some_and(|s| {
                ["renderer", "build", "preview", "image", "publish", "ci"]
                    .iter()
                    .any(|k| s.contains_key(*k))
            }) {
                return Err(invalid(
                    &config,
                    "renderer/build/preview/image/publish/ci require version: 1 and the content/ layout",
                ));
            }
            let renderer = if settings.is_some() {
                Renderer::Portable
            } else if Project::discover(&root).is_ok() {
                Renderer::Hosted
            } else {
                Renderer::Portable
            };
            let content = root.clone();
            let output = if renderer == Renderer::Hosted {
                Utf8PathBuf::from("dist")
            } else {
                root.with_file_name(format!(
                    "{}-site",
                    root.file_name().ok_or_else(|| files::unsafe_path(
                        &root,
                        "specify a source below the filesystem root"
                    ))?
                ))
            };
            (content, renderer, output)
        };
        Ok(Self {
            input,
            config_path: config,
            root,
            content,
            configured,
            renderer,
            output,
            port,
            image,
            publish,
            ci,
            portable: Value::Mapping(settings.unwrap_or_default()),
        })
    }

    pub fn require(&self, renderer: Renderer) -> Result<()> {
        if self.configured && self.renderer != renderer {
            return Err(invalid(
                &self.config_path,
                format!(
                    "renderer is {:?}; this operation requires {:?}",
                    self.renderer, renderer
                ),
            ));
        }
        Ok(())
    }
}
