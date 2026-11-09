//! Explicit import asset policies. Public links never fetch or copy asset bytes.
use camino::Utf8PathBuf;
use zellij_core::{Error, Result};

#[derive(Debug, Clone, Default)]
pub enum AssetPolicy {
    #[default]
    Copy,
    Skip,
    Link(AssetBaseUrl),
}

/// Validated absolute HTTP(S) directory prefix, with a trailing slash.
#[derive(Debug, Clone)]
pub struct AssetBaseUrl(String);

impl AssetBaseUrl {
    pub fn parse(value: &str) -> Result<Self> {
        let invalid = || {
            Error::InvalidArgument(
            "--asset-base-url must be an absolute HTTP(S) directory URL without credentials, query, fragment, whitespace, backslashes or dot segments".into()
        )
        };
        if value.chars().any(|c| c.is_control() || c.is_whitespace())
            || value.contains(['\\', '?', '#'])
        {
            return Err(invalid());
        }
        let (_, remainder) = value.split_once("://").ok_or_else(invalid)?;
        let (authority, path) = remainder.split_once('/').unwrap_or((remainder, ""));
        if authority.is_empty() || authority.contains('@') {
            return Err(invalid());
        }
        for segment in path.split('/') {
            let decoded = crate::references::decode(segment).ok_or_else(invalid)?;
            if matches!(decoded.as_str(), "." | "..")
                || decoded.chars().any(char::is_control)
                || decoded.contains('\\')
            {
                return Err(invalid());
            }
        }
        let url = url::Url::parse(value).map_err(|_| invalid())?;
        if !matches!(url.scheme(), "http" | "https")
            || url.host_str().is_none()
            || !url.username().is_empty()
            || url.password().is_some()
            || url.query().is_some()
            || url.fragment().is_some()
        {
            return Err(invalid());
        }
        let mut base = url.to_string();
        if !base.ends_with('/') {
            base.push('/');
        }
        Ok(Self(base))
    }

    pub(crate) fn href(&self, relative: &str) -> String {
        format!("{}{}", self.0, crate::references::encode_path(relative))
    }
}

#[derive(Debug, Clone)]
pub struct LinkedAsset {
    pub source: Utf8PathBuf,
    /// Public URL without per-reference query/fragment suffixes.
    pub url: String,
}
