//! Invocation-only hosted build settings. Encoded data never becomes shell code.
use serde::Serialize;
use std::{fmt, str::FromStr};

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct BasePath(String);

impl FromStr for BasePath {
    type Err = String;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let text = value.trim();
        if value.len() > 2048
            || value.chars().any(char::is_control)
            || text.starts_with("//")
            || value
                .chars()
                .any(|c| c.is_whitespace() || "?#\\%".contains(c))
        {
            return Err(
                "base path must be a URL path without whitespace, escapes, query or fragment"
                    .into(),
            );
        }
        let text = text.trim_end_matches('/');
        if text.is_empty() {
            return Ok(Self(String::new()));
        }
        let text = if text.starts_with('/') {
            text.to_owned()
        } else {
            format!("/{text}")
        };
        if text[1..]
            .split('/')
            .any(|part| part.is_empty() || part == "." || part == "..")
        {
            return Err("base path cannot contain empty, . or .. segments".into());
        }
        Ok(Self(text))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct SiteUrl(String);

impl FromStr for SiteUrl {
    type Err = String;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let invalid = || {
            "site URL must be an absolute http(s) origin without credentials, path, query or fragment; use --base-path for a deployment prefix".to_owned()
        };
        if value.len() > 2048
            || value
                .chars()
                .any(|c| c.is_whitespace() || c.is_control() || c == '\\')
        {
            return Err(invalid());
        }
        let url = url::Url::parse(value).map_err(|_| invalid())?;
        let authority = value.split_once("://").ok_or_else(invalid)?.1;
        let end = authority.find(['/', '?', '#']).unwrap_or(authority.len());
        if authority[..end].is_empty()
            || authority[..end].contains('@')
            || !matches!(&authority[end..], "" | "/")
        {
            return Err(invalid());
        }
        if !matches!(url.scheme(), "http" | "https")
            || url.host_str().is_none()
            || !url.username().is_empty()
            || url.password().is_some()
            || url.path() != "/"
            || url.query().is_some()
            || url.fragment().is_some()
        {
            return Err(invalid());
        }
        Ok(Self(url.origin().ascii_serialization()))
    }
}

impl fmt::Display for BasePath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}
impl fmt::Display for SiteUrl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildOverrides {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_path: Option<BasePath>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub site_url: Option<SiteUrl>,
    pub no_check: bool,
}
impl BuildOverrides {
    pub fn is_empty(&self) -> bool {
        self.base_path.is_none() && self.site_url.is_none() && !self.no_check
    }

    /// Hex JSON is safe in both Dockerfile ENV and runtime argv. User strings
    /// cannot introduce shell substitutions or Dockerfile instructions.
    pub fn encoded(&self) -> Option<String> {
        if self.is_empty() {
            return None;
        }
        let mut value = serde_json::to_value(self).expect("build settings serialize");
        value["version"] = 1.into();
        let bytes = serde_json::to_vec(&value).expect("build settings serialize");
        Some(bytes.iter().map(|byte| format!("{byte:02x}")).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn deployment_values_have_explicit_root_and_origin_semantics() {
        for (input, expected) in [
            ("/", ""),
            ("", ""),
            ("docs/", "/docs"),
            ("/日本語/", "/日本語"),
        ] {
            assert_eq!(input.parse::<BasePath>().unwrap().to_string(), expected);
        }
        for value in [
            "//other", "/a//b", "/a/../b", "/a/./b", "/a%20b", "/a?b", "/a#b", "/a\\b", "/a b",
            "\n/a", " /a", "/a ",
        ] {
            assert!(value.parse::<BasePath>().is_err(), "{value}");
        }
        assert_eq!(
            "https://EXAMPLE.com:443/"
                .parse::<SiteUrl>()
                .unwrap()
                .to_string(),
            "https://example.com"
        );
        assert_eq!(
            "http://[::1]:3000".parse::<SiteUrl>().unwrap().to_string(),
            "http://[::1]:3000"
        );
        for value in [
            "file:///site",
            "example.com",
            "https://user:pass@example.com",
            "https://example.com/docs",
            "https://example.com/?x",
            "https://example.com/#x",
            "https://example.com\\other",
            "https:example.com",
            "https:///example.com",
            "https://example.com/a/../",
        ] {
            assert!(value.parse::<SiteUrl>().is_err(), "{value}");
        }
    }
}
