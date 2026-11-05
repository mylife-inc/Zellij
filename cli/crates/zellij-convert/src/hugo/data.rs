use camino::{Utf8Path, Utf8PathBuf};
use serde_yaml_ng::{Mapping, Value};
use zellij_core::{Error, Result, files};

pub(super) fn invalid(message: impl std::fmt::Display) -> Error {
    Error::InvalidArgument(format!("Hugo: {message}"))
}
pub(super) fn string<'a>(v: &'a Value, key: &str) -> Result<Option<&'a str>> {
    match v.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(s)) => Ok(Some(s)),
        _ => Err(invalid(format!("{key} must be a string"))),
    }
}
pub(super) fn boolean(v: &Value, key: &str) -> Result<bool> {
    match v.get(key) {
        None | Some(Value::Null) => Ok(false),
        Some(Value::Bool(b)) => Ok(*b),
        _ => Err(invalid(format!("{key} must be a boolean"))),
    }
}
pub(super) fn active(v: &Value, key: &str) -> bool {
    v.get(key).is_some_and(|v| match v {
        Value::Null => false,
        Value::Sequence(s) => !s.is_empty(),
        Value::Mapping(m) => !m.is_empty(),
        _ => true,
    })
}
fn normalize(value: Value, depth: usize) -> Result<Value> {
    if depth > 64 {
        return Err(invalid("metadata nesting exceeds 64 levels"));
    }
    Ok(match value {
        Value::Mapping(map) => {
            let mut result = Mapping::new();
            for (key, value) in map {
                let key = key
                    .as_str()
                    .ok_or_else(|| invalid("metadata keys must be strings"))?
                    .to_lowercase();
                if result
                    .insert(Value::String(key.clone()), normalize(value, depth + 1)?)
                    .is_some()
                {
                    return Err(invalid(format!("duplicate case-insensitive key {key}")));
                }
            }
            Value::Mapping(result)
        }
        Value::Sequence(seq) => Value::Sequence(
            seq.into_iter()
                .map(|v| normalize(v, depth + 1))
                .collect::<Result<_>>()?,
        ),
        Value::Tagged(_) => return Err(invalid("YAML tags are not supported")),
        v => v,
    })
}
fn toml_value(value: toml::Value) -> Value {
    match value {
        toml::Value::String(s) => Value::String(s),
        toml::Value::Datetime(d) => Value::String(d.to_string()),
        toml::Value::Integer(n) => Value::Number(n.into()),
        toml::Value::Float(n) => Value::Number(n.into()),
        toml::Value::Boolean(b) => Value::Bool(b),
        toml::Value::Array(s) => Value::Sequence(s.into_iter().map(toml_value).collect()),
        toml::Value::Table(m) => Value::Mapping(
            m.into_iter()
                .map(|(k, v)| (Value::String(k), toml_value(v)))
                .collect(),
        ),
    }
}
pub(super) fn parse(raw: &str, format: &str) -> Result<Value> {
    let raw = raw.trim_start_matches('\u{feff}');
    let mut value = if format == "toml" {
        toml_value(toml::Value::Table(
            raw.parse::<toml::Table>()
                .map_err(|e| invalid(format!("invalid TOML: {e}")))?,
        ))
    } else {
        if format == "json" {
            serde_json::from_str::<serde_json::Value>(raw)
                .map_err(|e| invalid(format!("invalid JSON: {e}")))?;
        }
        // YAML also checks duplicate keys in JSON objects.
        let mut v: Value =
            serde_yaml_ng::from_str(raw).map_err(|e| invalid(format!("invalid metadata: {e}")))?;
        v = normalize(v, 0)?;
        v.apply_merge()
            .map_err(|e| invalid(format!("invalid YAML merge: {e}")))?;
        v
    };
    if value.is_null() {
        value = Value::Mapping(Mapping::new());
    }
    if !value.is_mapping() {
        return Err(invalid("metadata must be a mapping"));
    }
    normalize(value, 0)
}

pub(super) fn frontmatter(raw: &str) -> Result<(Value, &str, &str)> {
    let raw = raw.strip_prefix('\u{feff}').unwrap_or(raw);
    // Hugo skips leading ASCII whitespace before inspecting front matter and
    // permits metadata immediately after an opening +++/--- delimiter.
    let candidate = raw.trim_start_matches(|c: char| c.is_ascii_whitespace());
    let delimiter = if candidate.starts_with("---") {
        "---"
    } else if candidate.starts_with("+++") {
        "+++"
    } else {
        ""
    };
    if !delimiter.is_empty() {
        let rest = &candidate[3..];
        let rest = rest
            .strip_prefix("\r\n")
            .or_else(|| rest.strip_prefix('\n'))
            .unwrap_or(rest);
        let mut offset = 0;
        for line in rest.split_inclusive('\n') {
            let end = line.trim_end_matches(['\r', '\n']);
            if end == delimiter {
                let metadata = &rest[..offset];
                return Ok((
                    parse(metadata, if delimiter == "+++" { "toml" } else { "yaml" })?,
                    &rest[offset + line.len()..],
                    metadata,
                ));
            }
            offset += line.len();
        }
        return Err(invalid("front matter is missing its closing delimiter"));
    }
    if candidate.starts_with("#+") || candidate.starts_with(['+', '-']) {
        return Err(invalid(
            "unsupported or malformed Hugo front matter delimiter",
        ));
    }
    if candidate.starts_with('{') {
        let mut stream =
            serde_json::Deserializer::from_str(candidate).into_iter::<serde_json::Value>();
        stream
            .next()
            .ok_or_else(|| invalid("empty JSON front matter"))?
            .map_err(|e| invalid(format!("invalid JSON front matter: {e}")))?;
        let end = stream.byte_offset();
        return Ok((
            parse(&candidate[..end], "json")?,
            &candidate[end..],
            &candidate[..end],
        ));
    }
    Ok((Value::Mapping(Mapping::new()), raw, ""))
}
pub(super) fn document(value: &Value, body: &str, original: &str) -> Result<String> {
    let yaml = serde_yaml_ng::to_string(value).map_err(|e| invalid(e.to_string()))?;
    let mut raw = format!("---\n{yaml}# Original Hugo front matter:\n");
    for line in original.split(['\n', '\r', '\u{85}', '\u{2028}', '\u{2029}']) {
        raw.push_str(&format!("# {line}\n"));
    }
    raw.push_str("---\n");
    raw.push_str(body);
    Ok(raw)
}
pub(super) fn directory(root: &Utf8Path, name: &str) -> Result<Utf8PathBuf> {
    if name.contains(['*', '?', '[', ']', '{', '}']) {
        return Err(invalid("directory patterns are unsupported"));
    }
    files::contained_path(root, Utf8Path::new(name.trim_end_matches('/')))
}

pub(super) fn date(v: &Value, keys: &[&str]) -> Result<Option<i64>> {
    for key in keys {
        let Some(s) = string(v, key)?.filter(|s| !s.is_empty()) else {
            continue;
        };
        let timestamp = chrono::DateTime::parse_from_rfc3339(s)
            .map(|v| v.timestamp())
            .ok()
            .or_else(|| {
                chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S%.f")
                    .ok()
                    .map(|v| v.and_utc().timestamp())
            })
            .or_else(|| {
                chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S%.f")
                    .ok()
                    .map(|v| v.and_utc().timestamp())
            })
            .or_else(|| {
                chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d")
                    .ok()
                    .map(|v| v.and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp())
            })
            .ok_or_else(|| invalid(format!("{key} requires an ISO date or RFC3339 timestamp")))?;
        return Ok(Some(timestamp));
    }
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn data_formats_retain_dates_and_refuse_duplicate_or_executable_metadata() {
        let toml = parse("title='One'\ndate=2020-01-01T00:00:00Z\n", "toml").unwrap();
        let yaml = parse("Title: One\nDate: 2020-01-01\n", "yaml").unwrap();
        assert_eq!(
            date(&toml, &["date"]).unwrap(),
            date(&yaml, &["date"]).unwrap()
        );
        let merged = parse("base: &base {title: Old}\n<<: *base\ntitle: Own\n", "yaml").unwrap();
        assert_eq!(string(&merged, "title").unwrap(), Some("Own"));
        for (raw, format) in [
            ("Title: a\ntitle: b", "yaml"),
            ("x: !ruby foo", "yaml"),
            ("{\"title\":\"a\",\"title\":\"b\"}", "json"),
            ("title='a'\ntitle='b'", "toml"),
        ] {
            assert!(parse(raw, format).is_err(), "{raw}");
        }
        let offset = parse("date: '2020-01-01T01:00:00+01:00'", "yaml").unwrap();
        assert_eq!(
            date(&offset, &["date"]).unwrap(),
            date(&yaml, &["date"]).unwrap()
        );
    }
    #[test]
    fn all_frontmatter_forms_preserve_body_and_comment_original_unicode_lines() {
        for source in [
            "\u{feff}+++\r\ntitle='One'\r\n+++\r\n# Body",
            "---\ntitle: One\n---\n# Body",
            "{\"title\":\"One\"}\n# Body",
        ] {
            let (meta, body, original) = frontmatter(source).unwrap();
            assert_eq!(body.trim(), "# Body");
            let normalized = document(&meta, body, original).unwrap();
            let parsed = crate::frontmatter::parse(&normalized, Utf8Path::new("page.md")).unwrap();
            assert_eq!(parsed.string("title").unwrap(), Some("One"));
            assert_eq!(parsed.body, body);
        }
        let raw = document(
            &parse("title: Safe", "yaml").unwrap(),
            "# Body",
            "foo\u{2028}title: Injected",
        )
        .unwrap();
        assert!(raw.contains("# title: Injected"));
    }
}
