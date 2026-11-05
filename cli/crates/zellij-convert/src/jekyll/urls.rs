use super::config::invalid;
use crate::{references::decode, source_urls::SourceUrls};
use camino::Utf8Path;
use zellij_core::Result;

pub(super) fn permalink(value: &str) -> Result<String> {
    if !value.starts_with('/')
        || value.starts_with("//")
        || value.contains(['?', '#', '\\', ':', '{', '}'])
    {
        return Err(invalid(
            "permalink must be an absolute local path without placeholders, query or fragment",
        ));
    }
    let mut decoded = Vec::new();
    let inner = value.strip_prefix('/').unwrap().trim_end_matches('/');
    if !inner.is_empty() {
        for part in inner.split('/') {
            let part = decode(part).ok_or_else(|| invalid("malformed permalink encoding"))?;
            if part.is_empty()
                || part.contains(['/', '\\', ':', '?', '#', '%'])
                || part.chars().any(char::is_control)
            {
                return Err(invalid("unsafe permalink segment"));
            }
            zellij_core::files::relative_path(Utf8Path::new(&part))?;
            decoded.push(part);
        }
    }
    let mut result = format!("/{}", decoded.join("/"));
    if value.ends_with('/') && !result.ends_with('/') {
        result.push('/');
    }
    Ok(result)
}
pub(super) fn canonical(
    source: &Utf8Path,
    collection: Option<&str>,
    explicit: Option<&str>,
    style: &str,
) -> Result<String> {
    if let Some(value) = explicit {
        return permalink(value);
    }
    let stem = source.with_extension("");
    let path = stem.as_str();
    let path = if let Some(collection) = collection {
        format!(
            "{collection}/{}",
            path.strip_prefix(&format!("_{collection}/")).unwrap()
        )
    } else {
        path.to_owned()
    };
    if collection.is_none() && (path == "index" || path.ends_with("/index")) {
        return Ok(format!("/{}", path.strip_suffix("index").unwrap()));
    }
    Ok(format!(
        "/{path}{}",
        if style == "pretty" { "/" } else { ".html" }
    ))
}
pub(super) fn register(
    urls: &mut SourceUrls,
    address: &str,
    target: Option<String>,
    authored: bool,
) -> Result<()> {
    urls.add(address, target.clone())?;
    if authored {
        urls.add_unprefixed(address, target.clone())?;
    }
    if address.ends_with('/') {
        urls.add(&format!("{address}index.html"), target.clone())?;
        if authored {
            urls.add_unprefixed(&format!("{address}index.html"), target)?;
        }
    }
    Ok(())
}
