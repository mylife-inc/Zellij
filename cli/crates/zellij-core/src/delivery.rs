//! Shared, validated delivery intent. Resolution does not probe tools, build,
//! read credentials or contact a registry; executors consume the resulting plan.
use crate::{Error, Result, site::Site};
use serde::Deserialize;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ImageKind {
    Static,
    Next,
}
impl ImageKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Static => "static",
            Self::Next => "next",
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ImageSettings {
    #[serde(deserialize_with = "image_reference")]
    pub name: String,
}

/// Only implemented actions are accepted. In particular a provider or token
/// cannot be silently ignored on a registry-only publication.
#[derive(Debug, Deserialize)]
#[serde(tag = "action", rename_all = "lowercase", deny_unknown_fields)]
pub enum PublishSettings {
    Push {
        image: ImageKind,
    },
    Deploy {
        provider: Provider,
        #[serde(default, deserialize_with = "site_identifier")]
        site: Option<String>,
    },
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Provider {
    Netlify,
}

fn site_identifier<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> std::result::Result<Option<String>, D::Error> {
    let serde_yaml_ng::Value::String(value) = serde_yaml_ng::Value::deserialize(d)? else {
        return Err(serde::de::Error::custom(
            "publish.site must be a site ID or domain string",
        ));
    };
    validate_site_identifier(&value).map_err(serde::de::Error::custom)?;
    Ok(Some(value))
}

pub fn validate_site_identifier(value: &str) -> Result<()> {
    if value.is_empty()
        || value.len() > 253
        || !value.split('.').all(|part| {
            !part.is_empty()
                && part.as_bytes()[0].is_ascii_alphanumeric()
                && part.as_bytes()[part.len() - 1].is_ascii_alphanumeric()
                && part.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'-')
        })
    {
        return Err(Error::InvalidArgument("Netlify requires an existing site ID or domain, without a URL scheme, path, whitespace or credentials".into()));
    }
    Ok(())
}

#[derive(Debug, PartialEq, Eq)]
pub enum DeliveryPlan {
    Push(ImagePublication),
    Netlify { site: String, production: bool },
}

impl DeliveryPlan {
    /// Explicit `deploy` supplies intent; `publish --to` can override only a
    /// deployment provider, never silently replace a selected container image.
    pub fn resolve(
        site: &Site,
        provider: Option<&str>,
        to: Option<&str>,
        tag: Option<&str>,
        site_id: Option<&str>,
        environment_site: Option<&str>,
        production: bool,
    ) -> Result<Self> {
        if provider.is_some() && to.is_some() && provider != to {
            return Err(Error::InvalidArgument(
                "the explicit provider conflicts with --to".into(),
            ));
        }
        let explicit = provider.or(to);
        if explicit.is_some_and(|p| p != "netlify") {
            return Err(Error::InvalidArgument(
                "unsupported deployment provider; currently only netlify is implemented".into(),
            ));
        }
        if provider.is_none() && matches!(site.publish, Some(PublishSettings::Push { .. })) {
            if to.is_some() || site_id.is_some() || production {
                return Err(Error::InvalidArgument("image publication does not accept --to, --site-id or --prod; use `zel deploy netlify` for an explicit static-site deployment".into()));
            }
            return ImagePublication::resolve(site, tag).map(Self::Push);
        }
        let saved_site = match &site.publish {
            Some(PublishSettings::Deploy { site, .. }) => site.as_deref(),
            _ => None,
        };
        if explicit.is_none() && !matches!(site.publish, Some(PublishSettings::Deploy { .. })) {
            return Err(Error::InvalidArgument("publish requires saved publish settings in version: 1 zel.yaml; or use `zel deploy netlify --site-id <id>`".into()));
        }
        if tag.is_some() {
            return Err(Error::InvalidArgument(
                "Netlify accepts generated static HTML, not an image tag; remove --tag".into(),
            ));
        }
        let destination = site_id.or(environment_site).or(saved_site).ok_or_else(|| Error::InvalidArgument("Netlify requires an existing site: pass --site-id (or --site), set NETLIFY_SITE_ID, or save publish.site in zel.yaml".into()))?;
        Self::netlify(destination, production)
    }

    /// Shared constructor for saved delivery and a newly imported site.
    pub fn netlify(destination: &str, production: bool) -> Result<Self> {
        validate_site_identifier(destination)?;
        Ok(Self::Netlify {
            site: destination.to_owned(),
            production,
        })
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ImagePublication {
    pub kind: ImageKind,
    pub tag: String,
}
impl ImagePublication {
    pub fn resolve(site: &Site, tag: Option<&str>) -> Result<Self> {
        let Some(PublishSettings::Push { image }) = site.publish else {
            return Err(Error::InvalidArgument(
                "image publication requires version: 1 with `publish: {action: push, image: static}` (or image: next) in zel.yaml".into(),
            ));
        };
        let tag = tag.or(site.image.as_ref().map(|s| s.name.as_str())).ok_or_else(|| {
            Error::InvalidArgument("publish requires an explicit destination: set image.name in zel.yaml or pass --tag; generated local image names are never publication defaults".into())
        })?;
        Self::new(image, tag)
    }

    pub fn new(kind: ImageKind, tag: &str) -> Result<Self> {
        validate_image_reference(tag)?;
        Ok(Self {
            kind,
            tag: tag.to_owned(),
        })
    }
}

fn image_reference<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> std::result::Result<String, D::Error> {
    // YAML otherwise coerces bools/numbers into String.
    let serde_yaml_ng::Value::String(value) = serde_yaml_ng::Value::deserialize(d)? else {
        return Err(serde::de::Error::custom(
            "image.name must be an image reference string",
        ));
    };
    validate_image_reference(&value).map_err(serde::de::Error::custom)?;
    Ok(value)
}

/// A taggable distribution reference: [registry[:port]/]path[:tag]. Digests
/// identify existing content and cannot name the output of a new image build.
/// Keep this independent of the selected container client. Grammar reference:
/// https://github.com/distribution/reference/blob/main/regexp.go
pub fn validate_image_reference(value: &str) -> Result<()> {
    fn valid(value: &str) -> bool {
        if value.is_empty() || !value.is_ascii() || value.contains('@') {
            return false;
        }
        let last = value.rsplit('/').next().unwrap_or_default();
        let (name, tag) = match last.rsplit_once(':') {
            Some((_, tag)) => (&value[..value.len() - tag.len() - 1], Some(tag)),
            None => (value, None),
        };
        if let Some(tag) = tag {
            if tag.is_empty()
                || tag.len() > 128
                || !tag.as_bytes()[0].is_ascii_alphanumeric() && !tag.starts_with('_')
                || !tag
                    .bytes()
                    .all(|c| c.is_ascii_alphanumeric() || b"_.-".contains(&c))
            {
                return false;
            }
        }
        if name.is_empty() || name.len() > 255 {
            return false;
        }
        let mut parts = name.split('/').peekable();
        let first = parts.next().unwrap_or_default();
        let registry = parts.peek().is_some()
            && (first.contains(['.', ':'])
                || first == "localhost"
                || first.bytes().any(|c| c.is_ascii_uppercase()));
        if registry {
            if !valid_registry(first) {
                return false;
            }
        } else if !valid_component(first) {
            return false;
        }
        parts.all(valid_component)
    }
    if valid(value) {
        Ok(())
    } else {
        // Do not repeat malformed values: they may contain an accidentally
        // pasted credential or terminal control sequence.
        Err(Error::InvalidArgument("invalid image reference: use [registry[:port]/]lowercase/repository[:tag], without a URL scheme, credentials, whitespace or digest".into()))
    }
}
fn valid_component(value: &str) -> bool {
    let bytes = value.as_bytes();
    let alnum = |b: u8| b.is_ascii_lowercase() || b.is_ascii_digit();
    let mut index = 0;
    loop {
        let start = index;
        while index < bytes.len() && alnum(bytes[index]) {
            index += 1;
        }
        if index == start {
            return false;
        }
        if index == bytes.len() {
            return true;
        }
        match bytes[index] {
            b'-' => {
                while index < bytes.len() && bytes[index] == b'-' {
                    index += 1;
                }
            }
            b'_' => {
                index += 1;
                if bytes.get(index) == Some(&b'_') {
                    index += 1;
                }
            }
            b'.' => index += 1,
            _ => return false,
        }
    }
}
fn valid_registry(value: &str) -> bool {
    let (host, port) = if value.starts_with('[') {
        let Some(end) = value.find(']') else {
            return false;
        };
        if value[1..end].contains('.') || value[1..end].parse::<std::net::Ipv6Addr>().is_err() {
            return false;
        }
        let suffix = &value[end + 1..];
        if suffix.is_empty() {
            return true;
        }
        let Some(port) = suffix.strip_prefix(':') else {
            return false;
        };
        ("localhost", Some(port))
    } else {
        match value.split_once(':') {
            Some((host, port)) => (host, Some(port)),
            None => (value, None),
        }
    };
    if port.is_some_and(|p| {
        p.is_empty()
            || !p.bytes().all(|c| c.is_ascii_digit())
            || !p.parse::<u16>().is_ok_and(|p| p > 0)
    }) {
        return false;
    }
    host.split('.').all(|label| {
        !label.is_empty()
            && label.as_bytes()[0].is_ascii_alphanumeric()
            && label.as_bytes()[label.len() - 1].is_ascii_alphanumeric()
            && label
                .bytes()
                .all(|c| c.is_ascii_alphanumeric() || c == b'-')
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn accepts_taggable_references_and_rejects_unbuildable_destinations() {
        for reference in [
            "manual",
            "acme/manual:v2",
            "ghcr.io/acme/manual",
            "localhost:5000/docs:test",
            "registry.example:443/team/doc--site__2:_V1",
            "[::1]:5000/acme/docs:tag",
            "REGISTRY/acme/docs",
        ] {
            assert!(validate_image_reference(reference).is_ok(), "{reference}");
        }
        for reference in [
            "",
            "-tag",
            "https://ghcr.io/acme/docs",
            "acme/Manual",
            "acme//docs",
            "acme/docs:",
            "acme/docs:-tag",
            "acme/docs@sha256:abc",
            "user:secret@registry/docs",
            "a/b\n",
            "a b",
            "../docs",
            "docs/",
            "a/ab___cd",
            "a/ab..cd",
            "[not-ip]/docs",
            "[::ffff:192.0.2.1]/docs",
            "registry:0/docs",
            "registry:70000/docs",
            "registry:/docs",
            "registry:abc/docs",
            "日本/docs",
        ] {
            assert!(validate_image_reference(reference).is_err(), "{reference}");
        }
    }
}
