//! Source website addresses are lookup keys, never arbitrary filesystem roots.
use std::collections::BTreeMap;

use camino::{Utf8Path, Utf8PathBuf};
use unicode_normalization::UnicodeNormalization;
use url::Url;
use zellij_core::{Error, Result};

use crate::references::{decode, encode_path};

fn invalid(message: &str) -> Error {
    Error::InvalidArgument(format!("source website URL: {message}"))
}

/// Decode once, preserve case, and normalize only URL separators/dot segments.
fn path_key(path: &str) -> Result<String> {
    if !path.starts_with('/') || path.starts_with("//") || path.contains(['?', '#', '\\']) {
        return Err(invalid(
            "expected an absolute local path without query or fragment",
        ));
    }
    let mut parts = Vec::new();
    for part in path.split('/').skip(1) {
        let decoded = decode(part).ok_or_else(|| invalid("malformed percent encoding or UTF-8"))?;
        if decoded.contains(['/', '\\', ':', '?', '#']) || decoded.chars().any(char::is_control) {
            return Err(invalid("encoded separators or reserved characters in path"));
        }
        match decoded.as_str() {
            "" | "." => {}
            ".." => {
                parts
                    .pop()
                    .ok_or_else(|| invalid("path escapes the website root"))?;
            }
            _ => parts.push(decoded.nfc().collect::<String>()),
        }
    }
    Ok(format!("/{}", parts.join("/")))
}

fn http_url(value: &str) -> Result<Url> {
    let authority = value
        .split_once("://")
        .ok_or_else(|| invalid("expected an HTTP(S) URL"))?
        .1;
    let authority = authority.split(['/', '?', '#']).next().unwrap();
    if authority.is_empty() || authority.contains('@') {
        return Err(invalid("expected an HTTP(S) authority without credentials"));
    }
    let parsed = Url::parse(value).map_err(|_| invalid("expected an HTTP(S) URL"))?;
    if !matches!(parsed.scheme(), "http" | "https")
        || parsed.host_str().is_none()
        || !parsed.username().is_empty()
        || parsed.password().is_some()
        || value.contains('\\')
        || value.chars().any(char::is_control)
        || value.trim() != value
    {
        return Err(invalid(
            "expected an HTTP(S) URL without credentials or backslashes",
        ));
    }
    Ok(parsed)
}

pub(crate) struct SourceUrls {
    base: String,
    origin: Option<url::Origin>,
    // None reserves a known omitted/ambiguous document URL, preventing it from
    // being copied as an asset or silently redirected to another document.
    documents: BTreeMap<String, Option<String>>,
    // URL directories of each source page, not its filesystem parent. More
    // than one means that hosting/trailing-slash behavior is unspecified.
    directories: BTreeMap<Utf8PathBuf, Vec<String>>,
}

pub(crate) enum DocumentMatch<'a> {
    Found(&'a str),
    Unmapped,
}

impl SourceUrls {
    pub fn new(origin: Option<&str>, base: &str) -> Result<Self> {
        let origin = origin
            .map(|value| {
                let parsed = http_url(value)?;
                if parsed.path() != "/" || parsed.query().is_some() || parsed.fragment().is_some() {
                    return Err(invalid(
                        "site origin must not include a path, query or fragment",
                    ));
                }
                Ok(parsed.origin())
            })
            .transpose()?;
        Ok(Self {
            base: path_key(base)?.trim_end_matches('/').to_owned(),
            origin,
            documents: BTreeMap::new(),
            directories: BTreeMap::new(),
        })
    }

    pub fn from_site_url(value: Option<&str>) -> Result<Self> {
        match value.filter(|value| !value.is_empty()) {
            None => Self::new(None, "/"),
            Some(value) => {
                let parsed = http_url(value)?;
                if parsed.query().is_some() || parsed.fragment().is_some() {
                    return Err(invalid("site_url must not include a query or fragment"));
                }
                Self::new(Some(&parsed.origin().ascii_serialization()), parsed.path())
            }
        }
    }

    /// Relative is a decoded source permalink, not a source file to be read.
    pub fn add(&mut self, relative: &str, target: Option<String>) -> Result<()> {
        let path = format!("{}/{}", self.base, relative.trim_start_matches('/'));
        let key = path_key(&encode_path(&path))?;
        self.documents
            .entry(key)
            .and_modify(|old| {
                if *old != target {
                    *old = None;
                }
            })
            .or_insert(target);
        Ok(())
    }

    /// Register a decoded source page URL, retaining the significant trailing
    /// slash. Call more than once when the source's slash policy is unknown.
    pub fn add_page(&mut self, source: &Utf8Path, relative: &str) -> Result<()> {
        let path = format!("{}/{}", self.base, relative.trim_start_matches('/'));
        let key = path_key(&encode_path(&path))?;
        let directory = if path.ends_with('/') {
            key.trim_end_matches('/').to_owned()
        } else {
            key.rsplit_once('/').unwrap().0.to_owned()
        };
        let directories = self.directories.entry(source.to_owned()).or_default();
        if !directories.contains(&directory) {
            directories.push(directory);
        }
        Ok(())
    }

    /// Resolve only known document URLs. Never interpret the result as a file
    /// path: ordinary relative asset references keep their source semantics.
    pub fn relative_document(
        &self,
        source: &Utf8Path,
        value: &str,
    ) -> Result<Option<DocumentMatch<'_>>> {
        let Some(directories) = self.directories.get(source) else {
            return Ok(None);
        };
        let targets = directories
            .iter()
            .map(|directory| {
                let path = path_key(&format!("{}/{value}", encode_path(directory)))?;
                Ok(self.documents.get(&path))
            })
            .collect::<Result<Vec<_>>>()?;
        if targets.iter().all(Option::is_none) {
            return Ok(None);
        }
        // A match for only one possible slash policy is not evidence that
        // the other policy was never used. Leave such links for review.
        if let Some(Some(Some(target))) = targets.first()
            && targets.iter().all(|candidate| *candidate == targets[0])
        {
            return Ok(Some(DocumentMatch::Found(target)));
        }
        Ok(Some(DocumentMatch::Unmapped))
    }

    /// Full URLs are recognized only for the configured origin. Nothing is fetched.
    pub fn path(&self, value: &str) -> Result<Option<String>> {
        if value.starts_with('/') && !value.starts_with("//") {
            return path_key(value).map(Some);
        }
        if value.starts_with("//") {
            return Ok(None);
        }
        let Ok(parsed) = http_url(value) else {
            return Ok(None);
        };
        if self.origin.as_ref() != Some(&parsed.origin()) {
            return Ok(None);
        }
        // Inspect the authored path before Url can erase dot segments. The
        // parser above establishes the authority; query/fragment were split by
        // the reference resolver and are preserved verbatim there.
        let authority = value.split_once("://").unwrap().1;
        let path = authority.find('/').map_or("/", |index| &authority[index..]);
        path_key(path).map(Some)
    }

    pub fn document(&self, path: &str) -> Option<&Option<String>> {
        self.documents.get(path)
    }

    /// Decoded asset path relative to the already-contained source asset root.
    pub fn asset(&self, path: &str) -> Option<String> {
        if path == self.base {
            Some(String::new())
        } else {
            path.strip_prefix(&format!("{}/", self.base))
                .map(str::to_owned)
        }
    }
}
