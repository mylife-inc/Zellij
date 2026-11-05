use camino::{Utf8Path, Utf8PathBuf};
use serde_yaml_ng::{Mapping, Value};
use zellij_core::{Error, Result, files};

pub(super) fn invalid(message: impl std::fmt::Display) -> Error {
    Error::InvalidArgument(format!("Jekyll: {message}"))
}
pub(super) fn string<'a>(value: &'a Value, key: &str) -> Result<Option<&'a str>> {
    match value.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(v)) => Ok(Some(v)),
        _ => Err(invalid(format!("{key} must be a string"))),
    }
}
pub(super) fn boolean(value: &Value, key: &str) -> Result<Option<bool>> {
    match value.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::Bool(v)) => Ok(Some(*v)),
        _ => Err(invalid(format!("{key} must be a boolean"))),
    }
}
pub(super) fn yaml(raw: &str) -> Result<Value> {
    let mut value: Value = serde_yaml_ng::from_str(raw.trim_start_matches('\u{feff}'))
        .map_err(|e| invalid(format!("invalid YAML: {e}")))?;
    fn inert(value: &Value) -> bool {
        match value {
            Value::Tagged(_) => false,
            Value::Mapping(m) => m.iter().all(|(k, v)| k.is_string() && inert(v)),
            Value::Sequence(s) => s.iter().all(inert),
            _ => true,
        }
    }
    if !inert(&value) {
        return Err(invalid(
            "YAML tags and non-string mapping keys are not supported",
        ));
    }
    value
        .apply_merge()
        .map_err(|e| invalid(format!("invalid YAML merge: {e}")))?;
    Ok(value)
}
pub(super) fn path(value: &str) -> Result<Utf8PathBuf> {
    let path = value
        .strip_prefix("./")
        .unwrap_or(value)
        .trim_end_matches('/');
    if path.contains(['*', '?', '[', ']', '{', '}']) {
        return Err(invalid("glob/template paths require manual migration"));
    }
    let path = Utf8PathBuf::from(path);
    files::relative_path(&path)?;
    Ok(path)
}
pub(super) fn directory(root: &Utf8Path, value: &str) -> Result<Utf8PathBuf> {
    if matches!(value, "." | "./") {
        files::absolute_path(root)
    } else {
        files::contained_path(root, &path(value)?)
    }
}
pub(super) fn paths(value: &Value, key: &str) -> Result<Vec<Utf8PathBuf>> {
    match value.get(key) {
        None | Some(Value::Null) => Ok(Vec::new()),
        Some(Value::Sequence(values)) => values
            .iter()
            .map(|v| {
                path(
                    v.as_str()
                        .ok_or_else(|| invalid(format!("{key} entries must be paths")))?,
                )
            })
            .collect(),
        _ => Err(invalid(format!("{key} must be an array"))),
    }
}

pub(super) struct Defaults {
    sets: Vec<(String, Option<String>, Mapping)>,
}
impl Defaults {
    pub fn read(config: &Value) -> Result<Self> {
        let mut sets = Vec::new();
        if let Some(value) = config.get("defaults") {
            for entry in value
                .as_sequence()
                .ok_or_else(|| invalid("defaults must be an array"))?
            {
                let scope = entry
                    .get("scope")
                    .cloned()
                    .unwrap_or(Value::Mapping(Mapping::new()));
                let mapping = scope
                    .as_mapping()
                    .ok_or_else(|| invalid("defaults.scope must be a mapping"))?;
                if mapping
                    .keys()
                    .any(|k| !matches!(k.as_str(), Some("path" | "type")))
                {
                    return Err(invalid("unsupported defaults scope key"));
                }
                let prefix = string(&scope, "path")?
                    .unwrap_or("")
                    .trim_start_matches('/');
                if !prefix.is_empty() {
                    path(prefix)?;
                }
                let kind = string(&scope, "type")?.map(|kind| {
                    match kind {
                        "page" => "pages",
                        "post" => "posts",
                        "draft" => "drafts",
                        other => other,
                    }
                    .to_owned()
                });
                let values = entry
                    .get("values")
                    .and_then(Value::as_mapping)
                    .ok_or_else(|| invalid("defaults.values must be a mapping"))?
                    .clone();
                sets.push((prefix.to_owned(), kind, values));
            }
        }
        Ok(Self { sets })
    }
    pub fn apply(&self, source: &Utf8Path, kind: &str, raw: &str) -> Result<String> {
        let parsed = crate::frontmatter::parse(raw, source)?;
        let own = yaml(parsed.raw.unwrap_or(""))?;
        let mut effective = Mapping::new();
        let mut matching: Vec<_> = self
            .sets
            .iter()
            .filter(|(prefix, ty, _)| {
                source.as_str().starts_with(prefix) && ty.as_deref().is_none_or(|t| t == kind)
            })
            .collect();
        // Literal scopes: longer paths win, then type-specific scopes, then
        // later entries. Source front matter overrides inherited values.
        matching.sort_by_key(|(prefix, ty, _)| (prefix.chars().count(), ty.is_some()));
        for (_, _, values) in matching {
            merge(&mut effective, values, false);
        }
        if let Some(own) = own.as_mapping() {
            merge(&mut effective, own, kind == "pages");
        }
        let serialized =
            serde_yaml_ng::to_string(&effective).map_err(|e| invalid(e.to_string()))?;
        let mut result = format!("---\n{serialized}");
        if let Some(original) = parsed.raw {
            result.push_str("# Original Jekyll front matter before defaults:\n");
            for line in original.split(['\n', '\r', '\u{85}', '\u{2028}', '\u{2029}']) {
                result.push_str(&format!("# {line}\n"));
            }
        }
        result.push_str("---\n");
        result.push_str(parsed.body);
        Ok(result)
    }
}
fn merge(target: &mut Mapping, values: &Mapping, own: bool) {
    for (key, value) in values {
        if !own && value.is_null() && target.contains_key(key) {
            continue;
        }
        if let Some(Value::Mapping(old)) = target.get_mut(key)
            && let Value::Mapping(new) = value
        {
            merge(old, new, own);
        } else {
            target.insert(key.clone(), value.clone());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn yaml_merges_are_inert_and_bad_merge_values_are_rejected() {
        let value=yaml("base: &defaults {published: false, title: Inherited}\nvalues: {<<: *defaults, title: Own}\n").unwrap();
        assert_eq!(value["values"]["published"], Value::Bool(false));
        assert_eq!(value["values"]["title"].as_str(), Some("Own"));
        for raw in [
            "values: {<<: invalid}",
            "title: !custom value",
            "true: value",
        ] {
            assert!(yaml(raw).is_err(), "{raw}");
        }
    }
    #[test]
    fn scoped_defaults_preserve_specificity_and_authored_values() {
        let config=yaml("defaults:\n - scope: {path: guide, type: pages}\n   values: {title: Typed, description: Inherited, published: false}\n - scope: {path: guide}\n   values: {title: Untyped}\n - scope: {path: ''}\n   values: {title: Broad, description: null}\n - scope: {path: guide, type: page}\n   values: {title: Last typed}\n").unwrap();
        let defaults = Defaults::read(&config).unwrap();
        let path = Utf8Path::new("guide/topic.md");
        let raw = defaults
            .apply(path, "pages", "---\n---\n# Topic\n")
            .unwrap();
        let parsed = crate::frontmatter::parse(&raw, path).unwrap();
        assert_eq!(parsed.string("title").unwrap(), Some("Last typed"));
        assert_eq!(parsed.string("description").unwrap(), Some("Inherited"));
        assert_eq!(parsed.boolean("published").unwrap(), Some(false));
        let raw = defaults
            .apply(
                path,
                "pages",
                "---\ntitle: Own\npublished: true\n---\n# Topic\n",
            )
            .unwrap();
        let parsed = crate::frontmatter::parse(&raw, path).unwrap();
        assert_eq!(parsed.string("title").unwrap(), Some("Own"));
        assert_eq!(parsed.boolean("published").unwrap(), Some(true));
        assert_eq!(parsed.body, "# Topic\n");
    }
}
