//! Versioned data-only starters. Creation never launches an engine or install hook.
use crate::args::{NewSiteArgs, TemplateAction, TemplateSourceArgs, TemplatesArgs};
use camino::{Utf8Path, Utf8PathBuf};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::{Read, Write},
};
use zellij_core::{Error, Result, files};
mod install;
mod manifest;
mod source;
use manifest::{Manifest, Renderer};
include!(concat!(env!("OUT_DIR"), "/templates.rs"));
const FILE_LIMIT: usize = 4 * 1024 * 1024;
const TOTAL_LIMIT: usize = 32 * 1024 * 1024;
const COUNT_LIMIT: usize = 512;
fn invalid(message: impl Into<String>) -> Error {
    Error::InvalidArgument(format!("template: {}", message.into()))
}
fn ensure(condition: bool, message: &str) -> Result<()> {
    if condition {
        Ok(())
    } else {
        Err(invalid(message))
    }
}
fn sha(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn limited(mut reader: impl Read, limit: usize) -> Result<Vec<u8>> {
    let mut bytes = Vec::new();
    reader
        .by_ref()
        .take(limit as u64 + 1)
        .read_to_end(&mut bytes)?;
    ensure(
        bytes.len() <= limit,
        "file or download exceeds its size limit",
    )?;
    Ok(bytes)
}
fn parse<T: serde::de::DeserializeOwned>(bytes: &[u8]) -> Result<T> {
    serde_yaml_ng::from_slice(bytes).map_err(|e| invalid(format!("invalid manifest/catalog: {e}")))
}
#[derive(Clone)]
struct Bundle {
    manifest: Manifest,
    entries: BTreeMap<String, Vec<u8>>,
    digest: String,
    origin: String,
}
impl Bundle {
    fn from_entries(entries: BTreeMap<String, Vec<u8>>, origin: String) -> Result<Self> {
        ensure(
            entries.len() <= COUNT_LIMIT
                && entries.values().map(Vec::len).sum::<usize>() <= TOTAL_LIMIT,
            "template exceeds file count or total size limit",
        )?;
        let manifest: Manifest = parse(
            entries
                .get("template.yaml")
                .ok_or_else(|| invalid("template.yaml is required"))?,
        )?;
        manifest.validate()?;
        let mut hash = Sha256::new();
        let mut names = BTreeSet::new();
        for (name, bytes) in &entries {
            manifest::path(name)?;
            ensure(
                names.insert(name.to_lowercase()),
                "case-colliding template paths",
            )?;
            ensure(
                name == "template.yaml" || name.starts_with("files/"),
                "archive entries must be template.yaml or inside files/",
            )?;
            ensure(
                bytes.len() <= FILE_LIMIT,
                "template file exceeds size limit",
            )?;
            ensure(
                !name
                    .strip_prefix("files/")
                    .and_then(|p| p.split('/').next())
                    .is_some_and(|p| {
                        p.eq_ignore_ascii_case(".zel-template.json")
                            || p.eq_ignore_ascii_case("site")
                    }),
                ".zel-template.json and site/ are reserved for generated output",
            )?;
            hash.update((name.len() as u64).to_le_bytes());
            hash.update(name.as_bytes());
            hash.update((bytes.len() as u64).to_le_bytes());
            hash.update(bytes);
        }
        ensure(
            entries.contains_key("files/zel.yaml")
                && entries.keys().any(|p| p.starts_with("files/content/")),
            "template needs zel.yaml and content/ files",
        )?;
        for name in manifest
            .substitute
            .iter()
            .chain(manifest.assets.iter().map(|a| &a.path))
        {
            ensure(
                entries.contains_key(&format!("files/{name}")),
                "manifest references a missing file/asset",
            )?;
        }
        // Detect file/directory and case-only collisions before touching a destination.
        for name in entries.keys() {
            for parent in Utf8Path::new(name).ancestors().skip(1) {
                ensure(
                    !names.contains(&parent.as_str().to_lowercase()),
                    "file/directory collision in template",
                )?;
            }
        }
        Ok(Self {
            manifest,
            entries,
            digest: format!("{:x}", hash.finalize()),
            origin,
        })
    }
}
fn selector(value: &str) -> Result<(&str, Option<&str>)> {
    let (id, version) = value
        .split_once('@')
        .map_or((value, None), |(id, v)| (id, Some(v)));
    ensure(
        manifest::slug(id),
        "invalid template ID; run `zel templates`, or use ./path for a local template",
    )?;
    if let Some(v) = version {
        manifest::version(v)?;
    }
    Ok((id, version))
}
fn local_selector(value: &str) -> bool {
    value.starts_with('.') || value.contains(['/', '\\']) || Utf8Path::new(value).is_absolute()
}
fn bundled() -> Result<Vec<Bundle>> {
    #[derive(serde::Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Index {
        schema: u32,
        templates: Vec<String>,
    }
    let index: Index = parse(
        BUNDLED
            .iter()
            .find(|(p, _)| *p == "catalog.yaml")
            .ok_or_else(|| invalid("bundled catalog missing"))?
            .1,
    )?;
    ensure(
        index.schema == 1 && !index.templates.is_empty(),
        "unsupported bundled catalog",
    )?;
    let mut seen = BTreeSet::new();
    index
        .templates
        .into_iter()
        .map(|id| {
            ensure(
                manifest::slug(&id) && seen.insert(id.clone()),
                "invalid/duplicate bundled template",
            )?;
            let prefix = format!("{id}/");
            let entries = BUNDLED
                .iter()
                .filter_map(|(path, bytes)| {
                    path.strip_prefix(&prefix)
                        .map(|p| (p.into(), bytes.to_vec()))
                })
                .collect();
            let bundle = Bundle::from_entries(entries, "bundled".into())?;
            ensure(bundle.manifest.id == id, "catalog ID and manifest disagree")?;
            Ok(bundle)
        })
        .collect()
}
fn resolve(value: &str, options: &TemplateSourceArgs) -> Result<Bundle> {
    if local_selector(value) {
        ensure(
            options.catalog.is_none(),
            "a local template cannot also select --catalog",
        )?;
        return source::local(Utf8Path::new(value));
    }
    let (id, selected) = selector(value)?;
    if let Some(url) = &options.catalog {
        return source::remote(id, selected, url, options);
    }
    bundled()?.into_iter().find(|b| b.manifest.id == id && selected.is_none_or(|v| v == b.manifest.version))
        .ok_or_else(|| invalid(format!("no bundled template {value:?}; run `zel templates` or select a release catalog with --catalog")))
}
pub(crate) fn inspect(args: &TemplatesArgs, json: bool) -> Result<()> {
    let manifests = if let Some(TemplateAction::Show { template }) = &args.action {
        vec![resolve(template, &args.source)?.manifest]
    } else if let Some(url) = &args.source.catalog {
        source::catalog(url, &args.source)?
            .templates
            .into_iter()
            .map(|t| t.manifest)
            .collect()
    } else {
        bundled()?.into_iter().map(|b| b.manifest).collect()
    };
    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(
                &serde_json::json!({"schema_version":1,"templates":manifests})
            )
            .expect("serializable catalog")
        );
    } else {
        for m in manifests {
            zellij_core::statusln!("{}@{} — {}", m.id, m.version, m.title);
            zellij_core::statusln!("  {}", m.description);
            zellij_core::statusln!(
                "  Renderer: {:?}; outputs: {:?}; minimum CLI: {}",
                m.renderer,
                m.outputs,
                m.minimum_cli
            );
            if args.action.is_some() {
                for (key, default) in m.variables {
                    zellij_core::statusln!("  --set {key}={default}");
                }
                if m.print.is_some() {
                    zellij_core::statusln!("  Print/PDF: unavailable in this template revision");
                }
                for a in m.assets {
                    zellij_core::statusln!("  Asset: {} ({})", a.path, a.license);
                }
            }
        }
    }
    Ok(())
}
pub(crate) fn create(args: &NewSiteArgs) -> Result<()> {
    let target = Utf8Path::new(&args.dir);
    install::destination(target)?;
    if local_selector(&args.template) {
        files::disjoint(Utf8Path::new(&args.template), target)?;
    }
    let bundle = resolve(&args.template, &args.source)?;
    install::create(bundle, target, args.name.as_deref(), &args.set)?;
    zellij_core::statusln!("✓ {}", args.dir);
    zellij_core::statusln!(
        "  Edit content/ and zel.yaml. From inside this folder, run `zel check` or `zel view`."
    );
    Ok(())
}
#[cfg(test)]
mod tests;
