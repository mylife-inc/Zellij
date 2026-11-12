use super::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub(super) struct Manifest {
    pub schema: u32,
    pub id: String,
    pub version: String,
    pub title: String,
    pub description: String,
    pub minimum_cli: String,
    pub minimum_engine: String,
    pub renderer: Renderer,
    pub kind: Kind,
    pub outputs: Vec<Output>,
    /// Books explicitly disclose that print is unavailable in this revision.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub print: Option<Print>,
    pub variables: BTreeMap<String, String>,
    pub substitute: Vec<String>,
    #[serde(default)]
    pub assets: Vec<Asset>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preview: Option<String>,
}
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub(super) enum Renderer {
    Portable,
    Hosted,
}
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub(super) enum Kind {
    Site,
    Book,
}
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub(super) enum Output {
    Web,
    File,
}
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub(super) enum Print {
    Unavailable,
}
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(super) struct Asset {
    pub path: String,
    pub license: String,
}

pub(super) fn slug(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 64
        && !value.starts_with('-')
        && !value.ends_with('-')
        && value
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}
pub(super) fn text(value: &str) -> Result<()> {
    ensure(
        !value.trim().is_empty() && value.len() <= 4096 && !value.chars().any(char::is_control),
        "template text must be nonempty, at most 4096 bytes and contain no control characters",
    )
}
pub(super) fn version(value: &str) -> Result<semver::Version> {
    semver::Version::parse(value).map_err(|_| invalid("expected an exact semantic version"))
}
pub(super) fn path(value: &str) -> Result<()> {
    files::relative_path(Utf8Path::new(value))?;
    for part in value.split('/') {
        let stem = part
            .split('.')
            .next()
            .unwrap_or_default()
            .to_ascii_uppercase();
        ensure(
            !part.ends_with(['.', ' '])
                && !part
                    .chars()
                    .any(|c| c.is_control() || "<>\"|?*".contains(c))
                && !matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL")
                && !(stem.len() == 4
                    && (stem.starts_with("COM") || stem.starts_with("LPT"))
                    && matches!(stem.as_bytes()[3], b'1'..=b'9')),
            "template paths must be portable across Windows, macOS and Linux",
        )?;
    }
    Ok(())
}
impl Manifest {
    pub fn validate(&self) -> Result<()> {
        ensure(
            self.schema == 1 && slug(&self.id),
            "unsupported template schema or invalid template ID",
        )?;
        version(&self.version)?;
        text(&self.title)?;
        text(&self.description)?;
        let current = version(env!("CARGO_PKG_VERSION"))?;
        ensure(
            version(&self.minimum_cli)? <= current && version(&self.minimum_engine)? <= current,
            "template requires a newer CLI/engine; upgrade zel first",
        )?;
        ensure(
            self.outputs.contains(&Output::Web)
                && (self.renderer == Renderer::Portable || !self.outputs.contains(&Output::File))
                && self.outputs.len()
                    == self
                        .outputs
                        .iter()
                        .map(|o| format!("{o:?}"))
                        .collect::<BTreeSet<_>>()
                        .len(),
            "invalid template output capabilities",
        )?;
        ensure(
            self.kind != Kind::Book || self.print.is_some(),
            "book templates must declare print capability",
        )?;
        ensure(
            self.variables.len() <= 32 && self.variables.contains_key("title"),
            "templates need a title variable and at most 32 variables",
        )?;
        for (key, value) in &self.variables {
            ensure(slug(key), "invalid template variable name")?;
            text(value)?;
        }
        let mut paths = BTreeSet::new();
        for name in &self.substitute {
            path(name)?;
            ensure(paths.insert(name), "duplicate substitution file")?;
        }
        let mut assets = BTreeSet::new();
        for asset in &self.assets {
            path(&asset.path)?;
            text(&asset.license)?;
            ensure(assets.insert(&asset.path), "duplicate declared asset")?;
        }
        if let Some(preview) = &self.preview {
            path(preview)?;
            ensure(
                assets.contains(preview),
                "preview must be a licensed declared asset",
            )?;
        }
        Ok(())
    }
}
