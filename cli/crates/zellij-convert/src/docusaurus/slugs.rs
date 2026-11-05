//! Explicit Docusaurus slugs become routes within the converted guide.
//! These are URL paths only; they never select source or destination files.
use camino::Utf8Path;
use unicode_normalization::UnicodeNormalization;
use zellij_core::{Error, Result};

use super::strip_number_prefix;

fn invalid(source: &Utf8Path, message: &str) -> Error {
    Error::InvalidArgument(format!("{source}: invalid Docusaurus slug: {message}"))
}

fn segment(value: &str, source: &Utf8Path) -> Result<String> {
    if value.is_empty()
        || value
            .chars()
            .any(|c| c.is_control() || "\\/:*?\"<>|#%".contains(c))
        || value.ends_with(['.', ' '])
        || value.eq_ignore_ascii_case(".git")
    {
        return Err(invalid(
            source,
            "expected portable URL path segments without queries, fragments or reserved characters",
        ));
    }
    let stem = value.split('.').next().unwrap().to_ascii_uppercase();
    if matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL")
        || ["COM", "LPT"].iter().any(|prefix| {
            stem.strip_prefix(prefix).is_some_and(|tail| {
                matches!(
                    tail,
                    "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" | "¹" | "²" | "³"
                )
            })
        })
    {
        return Err(invalid(
            source,
            "URL segment is a reserved Windows filename",
        ));
    }
    Ok(value.nfc().collect())
}

fn resolve(slug: &str, base: Vec<String>, source: &Utf8Path) -> Result<String> {
    if slug.is_empty() || slug.starts_with("//") || slug.contains(['?', '#', '\\', ':']) {
        return Err(invalid(
            source,
            "expected a nonempty local URL path, without a scheme, host, query or fragment",
        ));
    }
    let absolute = slug.starts_with('/');
    let mut parts = if absolute { Vec::new() } else { base };
    let path = slug.strip_prefix('/').unwrap_or(slug).trim_end_matches('/');
    if !path.is_empty() {
        for raw in path.split('/') {
            let decoded = crate::references::decode(raw)
                .ok_or_else(|| invalid(source, "malformed UTF-8 or percent encoding"))?;
            match decoded.as_str() {
                "." | ".." if absolute || raw != decoded => {
                    return Err(invalid(
                        source,
                        "absolute or encoded dot segments are not supported",
                    ));
                }
                "." => {}
                ".." => {
                    parts.pop().ok_or_else(|| {
                        invalid(source, "relative slug escapes the documentation root")
                    })?;
                }
                _ => parts.push(segment(&decoded, source)?),
            }
        }
    }
    let mut route = String::from("/guide");
    if !parts.is_empty() {
        route.push('/');
        route.push_str(&parts.join("/"));
    }
    Ok(route)
}

pub(super) fn document(slug: &str, source: &Utf8Path, prefixes: bool) -> Result<String> {
    let mut base = Vec::new();
    if !slug.starts_with('/') {
        for part in source
            .parent()
            .unwrap_or(Utf8Path::new(""))
            .as_str()
            .split('/')
            .filter(|part| !part.is_empty())
        {
            base.push(segment(
                if prefixes {
                    strip_number_prefix(part)
                } else {
                    part
                },
                source,
            )?);
        }
    }
    resolve(slug, base, source)
}

pub(super) fn category(slug: &str, source: &Utf8Path) -> Result<String> {
    // Docusaurus joins category slugs with the docs base, not the category folder.
    resolve(slug, Vec::new(), source)
}

#[cfg(test)]
mod tests {
    use super::*;
    use camino::Utf8PathBuf;

    #[test]
    fn explicit_paths_preserve_case_unicode_and_relative_directory_semantics() {
        let source = Utf8PathBuf::from("01-Guides/02-Deep/topic.md");
        for (slug, prefixes, expected) in [
            ("/Custom/API/", true, "/guide/Custom/API"),
            ("new", true, "/guide/Guides/Deep/new"),
            ("../Start", true, "/guide/Guides/Start"),
            ("../../", true, "/guide"),
            (".", true, "/guide/Guides/Deep"),
            ("new", false, "/guide/01-Guides/02-Deep/new"),
            (
                "/Caf%C3%A9/Release%20Notes",
                true,
                "/guide/Café/Release Notes",
            ),
            ("/Cafe\u{301}", true, "/guide/Café"),
            ("/", true, "/guide"),
        ] {
            assert_eq!(document(slug, &source, prefixes).unwrap(), expected);
        }
        assert_eq!(category("Intro", &source).unwrap(), "/guide/Intro");
    }

    #[test]
    fn ambiguous_unsafe_and_unportable_paths_are_rejected() {
        let source = Utf8PathBuf::from("topic.md");
        for slug in [
            "",
            "../escape",
            "https://example.com",
            "//host/path",
            "/a?x",
            "/a#x",
            "/a\\b",
            "/a//b",
            "/a/../b",
            "/%2e",
            "/%2E%2E/x",
            "/a%2fb",
            "/a%5Cb",
            "/bad%",
            "/bad%FF",
            "/%252e",
            "/%00",
            "/a%3fb",
            "/.git/x",
            "/CON",
            "/nul.txt",
            "/LPT1",
            "/a.",
            "/a ",
            "/bad|name",
        ] {
            assert!(document(slug, &source, true).is_err(), "{slug:?}");
        }
    }
}
