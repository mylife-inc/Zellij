//! Inert YAML metadata shared by importers. Source text is retained for review.

use camino::Utf8Path;
use serde_yaml_ng::Value;
use zellij_core::{Error, Result};

pub(crate) struct Frontmatter<'a> {
    pub raw: Option<&'a str>,
    pub body: &'a str,
    data: Value,
    path: &'a Utf8Path,
}

fn invalid(path: &Utf8Path, reason: impl std::fmt::Display) -> Error {
    Error::InvalidArgument(format!("{path}: invalid front matter: {reason}"))
}

pub(crate) fn parse<'a>(source: &'a str, path: &'a Utf8Path) -> Result<Frontmatter<'a>> {
    let source = source.strip_prefix('\u{feff}').unwrap_or(source);
    let empty = || Frontmatter {
        raw: None,
        body: source,
        data: Value::Null,
        path,
    };
    let Some(first) = source.split_inclusive('\n').next() else {
        return Ok(empty());
    };
    if first.trim_end_matches(['\r', '\n']) != "---" {
        return Ok(empty());
    }
    let rest = &source[first.len()..];
    let mut offset = 0;
    for line in rest.split_inclusive('\n') {
        if matches!(line.trim_end_matches(['\r', '\n']), "---" | "...") {
            let raw = &rest[..offset];
            let data: Value = serde_yaml_ng::from_str(raw).map_err(|e| invalid(path, e))?;
            if !data.is_null() && !data.is_mapping() {
                return Err(invalid(path, "expected a YAML mapping"));
            }
            return Ok(Frontmatter {
                raw: Some(raw),
                body: &rest[offset + line.len()..],
                data,
                path,
            });
        }
        offset += line.len();
    }
    Err(invalid(path, "missing closing --- or ... delimiter"))
}

impl Frontmatter<'_> {
    pub fn contains(&self, field: &str) -> bool {
        self.data.get(field).is_some()
    }

    pub fn number(&self, field: &str) -> Result<Option<f64>> {
        match self.data.get(field) {
            None | Some(Value::Null) => Ok(None),
            Some(Value::Number(value)) => value
                .as_f64()
                .filter(|n| n.is_finite())
                .map(Some)
                .ok_or_else(|| invalid(self.path, format!("{field} must be a finite number"))),
            _ => Err(invalid(
                self.path,
                format!("{field} must be a finite number"),
            )),
        }
    }

    pub fn string(&self, field: &str) -> Result<Option<&str>> {
        match self.data.get(field) {
            None | Some(Value::Null) => Ok(None),
            Some(Value::String(value)) if field == "description" || !value.trim().is_empty() => {
                Ok(Some(value))
            }
            _ => Err(invalid(
                self.path,
                format!(
                    "{field} must be a {}string",
                    if field == "description" {
                        ""
                    } else {
                        "nonempty "
                    }
                ),
            )),
        }
    }

    pub fn boolean(&self, field: &str) -> Result<Option<bool>> {
        match self.data.get(field) {
            None | Some(Value::Null) => Ok(None),
            Some(Value::Bool(value)) => Ok(Some(*value)),
            _ => Err(invalid(self.path, format!("{field} must be a boolean"))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_bom_crlf_empty_metadata_and_eof_delimiters() {
        let path = Utf8Path::new("docs/page.md");
        let parsed = parse("\u{feff}---\r\ntitle: 'A: title'\r\ndescription: |\r\n  One\r\n  Two\r\n...\r\n# Body\r\n", path).unwrap();
        assert_eq!(parsed.string("title").unwrap(), Some("A: title"));
        assert_eq!(parsed.string("description").unwrap(), Some("One\nTwo\n"));
        assert_eq!(parsed.body, "# Body\r\n");
        assert!(parsed.raw.unwrap().contains("description:"));
        assert_eq!(parse("---\n---", path).unwrap().body, "");
        assert_eq!(parse("\u{feff}# Plain", path).unwrap().body, "# Plain");
    }

    #[test]
    fn rejects_malformed_documents_and_incompatible_used_fields() {
        let path = Utf8Path::new("docs/page.md");
        for source in [
            "---\ntitle: x",
            "---\n- one\n---\n",
            "---\nx: [\n---\n",
            "---\ntitle: x\ntitle: y\n---\n",
        ] {
            assert!(parse(source, path).is_err(), "{source}");
        }
        for value in ["123", "[]", "''", "!custom name"] {
            let raw = format!("---\ntitle: {value}\n---\n");
            let parsed = parse(&raw, path).unwrap();
            assert!(parsed.string("title").is_err(), "{value}");
        }
        let parsed = parse("---\ndraft: 'false'\n---\n", path).unwrap();
        assert!(parsed.boolean("draft").is_err());
    }
}
