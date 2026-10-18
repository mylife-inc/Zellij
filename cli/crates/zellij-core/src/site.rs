//! Versioned site roots and explicit legacy content folders. Discovery never
//! scans a repository or walks its parents.
use camino::{Utf8Path, Utf8PathBuf};
use serde::Deserialize;
use serde_yaml_ng::{Mapping, Value};
use std::{fs, io};

use crate::{Error, Project, Result, files};

pub const CONFIG: &str = "zel.yaml";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Renderer {
    Portable,
    Hosted,
}

#[derive(Debug)]
pub struct Site {
    pub root: Utf8PathBuf,
    pub content: Utf8PathBuf,
    pub configured: bool,
    pub renderer: Renderer,
    pub output: Utf8PathBuf,
    pub port: u16,
    pub image: Option<crate::delivery::ImageSettings>,
    pub publish: Option<crate::delivery::PublishSettings>,
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
    Error::InvalidArgument(format!("{}: {message}", root.join(CONFIG)))
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
    pub fn load(root: &Utf8Path) -> Result<Self> {
        let root = root.canonicalize_utf8()?;
        if !root.is_dir() {
            return Err(files::unsafe_path(
                &root,
                "expected a site/content directory",
            ));
        }
        let mut settings = match fs::symlink_metadata(root.join(CONFIG)) {
            Err(e) if e.kind() == io::ErrorKind::NotFound => None,
            Err(e) => return Err(e.into()),
            Ok(meta) => {
                if is_link(&meta) || !meta.is_file() {
                    return Err(invalid(
                        &root,
                        "expected a regular file, not a symlink or junction",
                    ));
                }
                Some(
                    serde_yaml_ng::from_str::<Mapping>(&fs::read_to_string(root.join(CONFIG))?)
                        .map_err(|e| invalid(&root, e))?,
                )
            }
        };
        let configured = settings.as_ref().is_some_and(|s| s.contains_key("version"));
        let mut port = 3000;
        let mut image = None;
        let mut publish = None;
        let (content, renderer, output) = if configured {
            let settings = settings.as_mut().unwrap();
            if settings.remove("version").and_then(|v| v.as_u64()) != Some(1) {
                return Err(invalid(&root, "version must be the integer 1"));
            }
            let renderer: Renderer = parse(
                &root,
                settings.remove("renderer").ok_or_else(|| {
                    invalid(&root, "version: 1 requires renderer: portable or hosted")
                })?,
            )?;
            let content = root.join("content");
            let meta = fs::symlink_metadata(&content).map_err(|e| {
                invalid(
                    &root,
                    format!("content/ must be an existing directory: {e}"),
                )
            })?;
            if is_link(&meta) || !meta.is_dir() {
                return Err(invalid(
                    &root,
                    "content/ must be a directory, not a symlink or junction",
                ));
            }
            let output = match settings.remove("build") {
                Some(value) => {
                    let build: Build = parse(&root, value)?;
                    let path = build
                        .out
                        .as_str()
                        .filter(|s| !s.trim().is_empty())
                        .ok_or_else(|| {
                            invalid(&root, "build.out must be a nonempty path string")
                        })?;
                    root.join(path)
                }
                None => root.join("site"),
            };
            if let Some(value) = settings.remove("preview") {
                port = parse::<Preview>(&root, value)?.port;
            }
            image = settings
                .remove("image")
                .map(|value| parse(&root, value))
                .transpose()?;
            publish = settings
                .remove("publish")
                .map(|value| parse(&root, value))
                .transpose()?;
            if renderer == Renderer::Hosted && !settings.is_empty() {
                return Err(invalid(
                    &root,
                    "hosted zel.yaml supports version, renderer, build, preview, image and publish only; put presentation settings in content/site.yaml",
                ));
            }
            (content, renderer, output)
        } else {
            if settings.as_ref().is_some_and(|s| {
                ["renderer", "build", "preview", "image", "publish"]
                    .iter()
                    .any(|k| s.contains_key(*k))
            }) {
                return Err(invalid(
                    &root,
                    "renderer/build/preview/image/publish require version: 1 and the content/ layout",
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
            root,
            content,
            configured,
            renderer,
            output,
            port,
            image,
            publish,
            portable: Value::Mapping(settings.unwrap_or_default()),
        })
    }

    pub fn require(&self, renderer: Renderer) -> Result<()> {
        if self.configured && self.renderer != renderer {
            return Err(invalid(
                &self.root,
                format!(
                    "renderer is {:?}; this operation requires {:?}",
                    self.renderer, renderer
                ),
            ));
        }
        Ok(())
    }
}
