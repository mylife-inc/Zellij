use super::data::{boolean, invalid, string};
use camino::Utf8Path;
use serde_yaml_ng::Value;
use unicode_normalization::UnicodeNormalization;
use zellij_core::{Result, files};

pub(super) fn segment(value: &str, keep_case: bool) -> Result<String> {
    // Refuse unusual path punctuation instead of approximating Hugo's full
    // Unicode/percent-escape sanitization. Keep ordinary Unicode names.
    static ALLOWED: std::sync::LazyLock<regex::Regex> =
        std::sync::LazyLock::new(|| regex::Regex::new(r"^[\p{L}\p{N}\p{M} ._+~@-]+$").unwrap());
    if !ALLOWED.is_match(value) {
        return Err(invalid(format!("unsupported source URL segment {value:?}")));
    }
    let mut out = String::new();
    let mut pending_space = false;
    for c in value.chars() {
        if c == ' ' {
            pending_space = !out.is_empty() && !out.ends_with('-');
            continue;
        }
        if pending_space && c != '-' {
            out.push('-');
        }
        pending_space = false;
        out.push(c);
    }
    if !keep_case {
        out = out.to_lowercase();
    }
    let out: String = out.nfc().collect();
    files::relative_path(Utf8Path::new(&out))?;
    Ok(out)
}
pub(super) fn literal(value: &str) -> Result<String> {
    let value = value.trim_start_matches('/');
    if value.is_empty() {
        return Ok("/".into());
    }
    let mut parts = Vec::new();
    for p in value.trim_end_matches('/').split('/') {
        let p = crate::references::decode(p).ok_or_else(|| invalid("invalid URL encoding"))?;
        if p.contains(['/', '\\', ':', '?', '#', '%', '{', '}']) {
            return Err(invalid("unsafe URL segment"));
        }
        files::relative_path(Utf8Path::new(&p))?;
        parts.push(p);
    }
    let mut out = format!("/{}", parts.join("/"));
    if value.ends_with('/') || Utf8Path::new(value).extension().is_none() {
        out.push('/');
    }
    Ok(out)
}
pub(super) fn canonical(path: &Utf8Path, metadata: &Value, config: &Value) -> Result<String> {
    let stem = path.file_stem().unwrap();
    let bundle = matches!(stem, "index" | "_index");
    if bundle && (string(metadata, "url")?.is_some() || string(metadata, "slug")?.is_some()) {
        return Err(invalid(format!(
            "{path}: bundle/section URL or slug overrides require manual migration"
        )));
    }
    if let Some(value) = string(metadata, "url")?.filter(|v| !v.is_empty()) {
        if value.starts_with("//") || value.contains(['?', '#', '\\', ':', '{', '}']) {
            return Err(invalid(
                "url must be a local path without query, fragment or template",
            ));
        }
        return literal(value);
    }
    let keep_case = boolean(config, "disablepathtolower")?;
    let mut parts = Vec::new();
    for c in path.parent().unwrap_or(Utf8Path::new("")).components() {
        parts.push(segment(c.as_str(), keep_case)?);
    }
    if !bundle {
        parts.push(segment(
            string(metadata, "slug")?
                .filter(|s| !s.is_empty())
                .unwrap_or(stem),
            keep_case,
        )?);
    }
    let path = parts.join("/");
    Ok(if path.is_empty() {
        "/".into()
    } else {
        format!("/{path}/")
    })
}
