//! Edit a fresh engine-owned section template without a runtime. This accepts
//! scalar/JSON field edits, optional-field removal and bounded array sizes.
use serde_json::{Map, Value, json};
use zellij_core::{Error, Result, schema};

fn invalid(message: impl Into<String>) -> Error {
    Error::InvalidArgument(message.into())
}
fn failure(path: &str, message: &str) -> Error {
    invalid(format!("{path}: {message}"))
}
fn parts(path: &str) -> Result<Vec<&str>> {
    let parts: Vec<_> = path.split('.').collect();
    if parts.len() > 32
        || parts
            .iter()
            .any(|p| p.is_empty() || !p.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_'))
    {
        return Err(failure(
            path,
            "use dot-separated field names and zero-based array indexes",
        ));
    }
    if parts[0] == "type" {
        return Err(failure(
            path,
            "choose the section type as the command argument",
        ));
    }
    Ok(parts)
}
fn shape<'a>(schema: &'a Value, part: &str) -> Option<&'a Value> {
    if schema
        .get("properties")
        .is_some_and(|v| v.get(part).is_some())
        || (schema.get("items").is_some() && part.parse::<usize>().is_ok())
    {
        return Some(schema);
    }
    schema["anyOf"]
        .as_array()?
        .iter()
        .find_map(|option| shape(option, part))
}
fn field<'a>(schema: &'a Value, part: &str, path: &str) -> Result<&'a Value> {
    let schema = shape(schema, part)
        .ok_or_else(|| failure(path, "unknown field; use zel explain for available fields"))?;
    if let Some(properties) = schema.get("properties") {
        return Ok(&properties[part]);
    }
    index(part, path)?;
    Ok(&schema["items"])
}
fn array_schema(schema: &Value) -> Option<&Value> {
    if schema.get("items").is_some() {
        return Some(schema);
    }
    schema["anyOf"].as_array()?.iter().find_map(array_schema)
}
fn index(part: &str, path: &str) -> Result<usize> {
    if part.len() > 1 && part.starts_with('0') {
        return Err(failure(path, "array indexes cannot have leading zeros"));
    }
    part.parse()
        .map_err(|_| failure(path, "expected a zero-based array index"))
}
fn has_type(schema: &Value, kind: &str) -> bool {
    schema["type"] == kind
        || schema["type"]
            .as_array()
            .is_some_and(|v| v.iter().any(|v| v == kind))
        || schema["anyOf"]
            .as_array()
            .is_some_and(|v| v.iter().any(|v| has_type(v, kind)))
}
fn seed(schema: &Value, name: &str) -> Result<Value> {
    if let Some(v) = schema.get("const").or_else(|| schema.get("default")) {
        return Ok(v.clone());
    }
    if let Some(v) = schema["enum"].as_array().and_then(|v| v.first()) {
        return Ok(v.clone());
    }
    if let Some(options) = schema["anyOf"].as_array() {
        return seed(&options[0], name);
    }
    if has_type(schema, "object") {
        let mut object = Map::new();
        if let Some(required) = schema["required"].as_array() {
            for key in required {
                let key = key.as_str().unwrap();
                object.insert(key.into(), seed(&schema["properties"][key], key)?);
            }
        }
        return Ok(Value::Object(object));
    }
    if has_type(schema, "array") {
        let count = schema["minItems"].as_u64().unwrap_or(1).max(1) as usize;
        return Ok(Value::Array(vec![seed(&schema["items"], name)?; count]));
    }
    if has_type(schema, "string") {
        return Ok(json!(match name {
            "image" | "visual" | "logos" | "avatar" | "photo" | "poster" => "assets/replace-me.svg",
            "video" | "src" => "assets/replace-me.mp4",
            "href" => "#",
            _ => "TODO",
        }));
    }
    if has_type(schema, "boolean") {
        return Ok(json!(false));
    }
    if has_type(schema, "number") || has_type(schema, "integer") {
        return Ok(schema.get("minimum").cloned().unwrap_or(json!(0)));
    }
    Err(failure(name, "unsupported authoring field"))
}
/// Deliberately closed to the schema vocabulary emitted by this engine version.
/// If the engine adds a constraint, authoring fails until it can enforce it.
fn supported(schema: &Value) -> Result<()> {
    for key in schema
        .as_object()
        .ok_or_else(|| invalid("invalid embedded section schema"))?
        .keys()
    {
        if ![
            "$schema",
            "type",
            "properties",
            "required",
            "items",
            "anyOf",
            "enum",
            "const",
            "default",
            "minLength",
            "minItems",
            "maxItems",
            "minimum",
            "maximum",
            "pattern",
            "zellijReference",
        ]
        .contains(&key.as_str())
        {
            return Err(invalid(format!(
                "unsupported authoring schema constraint {key}; update the native authoring validator"
            )));
        }
    }
    for key in ["properties"] {
        if let Some(fields) = schema[key].as_object() {
            for child in fields.values() {
                supported(child)?;
            }
        }
    }
    if let Some(child) = schema.get("items") {
        supported(child)?;
    }
    if let Some(children) = schema["anyOf"].as_array() {
        for child in children {
            supported(child)?;
        }
    }
    Ok(())
}
fn validate(schema: &Value, value: &Value, path: &str) -> Result<()> {
    if let Some(options) = schema["anyOf"].as_array() {
        return if options
            .iter()
            .any(|option| validate(option, value, path).is_ok())
        {
            Ok(())
        } else {
            Err(failure(path, "value does not match any allowed type"))
        };
    }
    if schema.get("const").is_some_and(|v| v != value) {
        return Err(failure(path, "constant field cannot be changed"));
    }
    if schema["enum"]
        .as_array()
        .is_some_and(|v| !v.contains(value))
    {
        return Err(failure(path, &format!("choose one of {}", schema["enum"])));
    }
    let kind = match value {
        Value::String(_) => "string",
        Value::Bool(_) => "boolean",
        Value::Number(_) => "number",
        Value::Null => "null",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    };
    if !has_type(schema, kind)
        && !(kind == "number"
            && has_type(schema, "integer")
            && value.as_f64().is_some_and(|v| v.fract() == 0.0))
    {
        return Err(failure(path, &format!("expected {}", schema["type"])));
    }
    match value {
        Value::String(text) => {
            // Match Zod's UTF-16 length checks for the current generated rules.
            if schema["minLength"]
                .as_u64()
                .is_some_and(|min| text.encode_utf16().count() < min as usize)
            {
                return Err(failure(path, "text is too short"));
            }
            if let Some(pattern) = schema["pattern"].as_str() {
                let pattern = regex::Regex::new(pattern)
                    .map_err(|_| failure(path, "unsupported embedded pattern"))?;
                if !pattern.is_match(text) {
                    return Err(failure(path, "text does not match the required pattern"));
                }
            }
        }
        Value::Number(number) => {
            let n = number
                .as_f64()
                .ok_or_else(|| failure(path, "expected a finite number"))?;
            if schema["minimum"].as_f64().is_some_and(|v| n < v)
                || schema["maximum"].as_f64().is_some_and(|v| n > v)
            {
                return Err(failure(path, "number is outside the allowed range"));
            }
        }
        Value::Array(items) => {
            bounds(schema, items.len(), path)?;
            for (i, item) in items.iter().enumerate() {
                validate(&schema["items"], item, &format!("{path}.{i}"))?;
            }
        }
        Value::Object(object) => {
            if let Some(required) = schema["required"].as_array() {
                for key in required {
                    let key = key.as_str().unwrap();
                    if !object.contains_key(key) {
                        return Err(failure(
                            &format!("{path}.{key}"),
                            "required field is missing",
                        ));
                    }
                }
            }
            for (key, value) in object {
                validate(field(schema, key, path)?, value, &format!("{path}.{key}"))?;
            }
        }
        _ => {}
    }
    Ok(())
}
fn bounds(schema: &Value, count: usize, path: &str) -> Result<()> {
    let min = schema["minItems"].as_u64().unwrap_or(0) as usize;
    let max = schema["maxItems"].as_u64().unwrap_or(256).min(256) as usize;
    if count < min || count > max {
        return Err(failure(
            path,
            &format!("item count must be between {min} and {max}"),
        ));
    }
    Ok(())
}
fn target<'a>(
    value: &'a mut Value,
    schema: &Value,
    parts: &[&str],
    path: &str,
) -> Result<&'a mut Value> {
    let Some((head, tail)) = parts.split_first() else {
        return Ok(value);
    };
    let next = field(schema, head, path)?;
    let branch = shape(schema, head).unwrap();
    if (branch.get("properties").is_some() && !value.is_object())
        || (branch.get("items").is_some() && !value.is_array())
    {
        *value = seed(branch, head)?;
    }
    let child = match value {
        Value::Object(object) => {
            if !object.contains_key(*head) {
                object.insert((*head).into(), seed(next, head)?);
            }
            object.get_mut(*head).unwrap()
        }
        Value::Array(items) => items.get_mut(index(head, path)?).ok_or_else(|| {
            failure(
                path,
                "array index is out of range; set its size with --count first",
            )
        })?,
        _ => return Err(failure(path, "this field has no nested fields")),
    };
    target(child, next, tail, path)
}
#[derive(Clone)]
pub struct Section {
    section_type: String,
    value: Value,
}
impl Section {
    pub fn new(section_type: &str) -> Result<Self> {
        let schema = schema::section_schema(section_type).ok_or_else(|| {
            invalid(format!(
                "no section type called {section_type:?}; run zel list sections"
            ))
        })?;
        supported(schema)?;
        let value = serde_yaml_ng::from_str(schema::section_template(section_type).unwrap())
            .map_err(|e| invalid(format!("invalid embedded template: {e}")))?;
        let section = Self {
            section_type: section_type.into(),
            value,
        };
        section.validate()?;
        Ok(section)
    }
    pub fn kind(&self) -> &str {
        &self.section_type
    }
    pub fn value(&self) -> &Value {
        &self.value
    }
    pub fn field_schema(&self, path: &str) -> Result<&'static Value> {
        let mut current = schema::section_schema(&self.section_type).unwrap();
        for part in parts(path)? {
            current = field(current, part, path)?;
        }
        Ok(current)
    }
    pub fn set(&mut self, path: &str, text: &str) -> Result<()> {
        let field = self.field_schema(path)?;
        let value = if has_type(field, "string") {
            json!(text)
        } else {
            serde_json::from_str(text)
                .map_err(|_| failure(path, "expected a JSON boolean or number"))?
        };
        if value.is_array() || value.is_object() || value.is_null() {
            return Err(failure(
                path,
                "--set accepts scalar text, booleans or numbers; use --count for lists",
            ));
        }
        self.assign(path, value)
    }
    pub fn set_json(&mut self, path: &str, text: &str) -> Result<()> {
        // The normal JSON parser rejects trailing input and excessive nesting.
        let value = serde_json::from_str(text)
            .map_err(|_| failure(path, "expected a complete JSON value"))?;
        self.assign(path, value)
    }
    fn assign(&mut self, path: &str, value: Value) -> Result<()> {
        validate(self.field_schema(path)?, &value, path)?;
        let mut updated = self.value.clone();
        *target(
            &mut updated,
            schema::section_schema(&self.section_type).unwrap(),
            &parts(path)?,
            path,
        )? = value;
        // Setting a media alternative switches the engine's exclusive group.
        if let Some(group) =
            schema::authoring_rules(&self.section_type).and_then(|r| r["exactlyOne"].as_array())
        {
            let root = parts(path)?[0];
            if group.iter().any(|v| v == root) {
                for other in group {
                    let other = other.as_str().unwrap();
                    if other != root {
                        updated.as_object_mut().unwrap().remove(other);
                    }
                }
            }
        }
        self.value = updated;
        Ok(())
    }
    pub fn unset(&mut self, path: &str) -> Result<()> {
        self.field_schema(path)?;
        let parts = parts(path)?;
        let (key, parents) = parts.split_last().unwrap();
        let mut schema = schema::section_schema(&self.section_type).unwrap();
        let mut value = &mut self.value;
        for part in parents {
            schema = field(schema, part, path)?;
            value = match value {
                Value::Object(object) => object.get_mut(*part),
                Value::Array(items) => items.get_mut(index(part, path)?),
                _ => None,
            }
            .ok_or_else(|| failure(path, "parent field is missing or has a different type"))?;
        }
        let object = value.as_object_mut().ok_or_else(|| {
            failure(
                path,
                "--unset removes object fields; replace or resize a list instead",
            )
        })?;
        let branch = shape(schema, key).unwrap();
        if branch["required"]
            .as_array()
            .is_some_and(|required| required.iter().any(|name| name.as_str() == Some(*key)))
        {
            return Err(failure(path, "required field cannot be removed"));
        }
        object.remove(*key);
        Ok(())
    }
    pub fn count_bounds(&self, path: &str) -> Result<(usize, usize)> {
        let field = array_schema(self.field_schema(path)?)
            .ok_or_else(|| failure(path, "expected a list field"))?;
        Ok((
            field["minItems"].as_u64().unwrap_or(0) as usize,
            field["maxItems"].as_u64().unwrap_or(256).min(256) as usize,
        ))
    }
    pub fn count(&mut self, path: &str, count: usize) -> Result<()> {
        let field = array_schema(self.field_schema(path)?)
            .ok_or_else(|| failure(path, "--count requires a list field"))?;
        if self.aligned_cells(path) {
            return Err(failure(
                path,
                "cell count follows columns; use --count columns=N",
            ));
        }
        if !has_type(field, "array") {
            return Err(failure(path, "--count requires a list field"));
        }
        bounds(field, count, path)?;
        let mut updated = self.value.clone();
        let value = target(
            &mut updated,
            schema::section_schema(&self.section_type).unwrap(),
            &parts(path)?,
            path,
        )?;
        if !value.is_array() {
            *value = seed(field, parts(path)?.last().unwrap())?;
        }
        let items = value.as_array_mut().unwrap();
        let example = seed(&field["items"], parts(path)?.last().unwrap())?;
        items.resize(count, example);
        self.value = updated;
        self.align_rows()?;
        Ok(())
    }
    fn align_rows(&mut self) -> Result<()> {
        if let Some(rule) =
            schema::authoring_rules(&self.section_type).and_then(|r| r.get("alignedRows"))
        {
            let columns = rule["columns"].as_str().unwrap();
            let rows = rule["rows"].as_str().unwrap();
            let cells = rule["cells"].as_str().unwrap();
            let size = self.value[columns].as_array().unwrap().len();
            for row in self.value[rows].as_array_mut().unwrap() {
                row[cells]
                    .as_array_mut()
                    .unwrap()
                    .resize(size, json!("TODO"));
            }
        }
        Ok(())
    }
    pub fn aligned_cells(&self, path: &str) -> bool {
        let Some(rule) =
            schema::authoring_rules(&self.section_type).and_then(|r| r.get("alignedRows"))
        else {
            return false;
        };
        let p: Vec<_> = path.split('.').collect();
        p.len() == 3
            && p[0] == rule["rows"].as_str().unwrap()
            && p[2] == rule["cells"].as_str().unwrap()
    }
    pub fn highlight(&mut self, selected: &str) -> Result<()> {
        if self.section_type != "pricing-table" {
            return Err(invalid("--highlight requires pricing-table"));
        }
        let tiers = self.value["tiers"].as_array_mut().unwrap();
        let selected = if selected == "none" {
            None
        } else if let Ok(index) = selected.parse::<usize>() {
            if index == 0 || index > tiers.len() {
                return Err(invalid("highlight tier number is out of range"));
            }
            Some(index - 1)
        } else {
            let matches: Vec<_> = tiers
                .iter()
                .enumerate()
                .filter(|(_, v)| v["name"] == selected)
                .map(|(i, _)| i)
                .collect();
            if matches.len() != 1 {
                return Err(invalid(
                    "highlight must name exactly one tier, use a 1-based tier number, or be none",
                ));
            }
            Some(matches[0])
        };
        for (i, tier) in tiers.iter_mut().enumerate() {
            tier["highlighted"] = json!(selected == Some(i));
        }
        Ok(())
    }
    pub fn validate(&self) -> Result<()> {
        validate(
            schema::section_schema(&self.section_type).unwrap(),
            &self.value,
            &self.section_type,
        )?;
        if let Some(rules) = schema::authoring_rules(&self.section_type) {
            if let Some(group) = rules["exactlyOne"].as_array() {
                if group
                    .iter()
                    .filter(|key| !self.value[key.as_str().unwrap()].is_null())
                    .count()
                    != 1
                {
                    return Err(invalid(
                        "section must contain exactly one media alternative",
                    ));
                }
            }
            if let Some(rule) = rules.get("alignedRows") {
                let count = self.value[rule["columns"].as_str().unwrap()]
                    .as_array()
                    .unwrap()
                    .len();
                if !self.value[rule["rows"].as_str().unwrap()]
                    .as_array()
                    .unwrap()
                    .iter()
                    .all(|row| {
                        row[rule["cells"].as_str().unwrap()]
                            .as_array()
                            .unwrap()
                            .len()
                            == count
                    })
                {
                    return Err(invalid("every matrix row must have one cell per column"));
                }
            }
        }
        Ok(())
    }
    pub fn render(&self) -> Result<String> {
        self.validate()?;
        // Leave the original artifact byte-for-byte intact if it is unchanged.
        let template = schema::section_template(&self.section_type).unwrap();
        let original: Value = serde_yaml_ng::from_str(template).unwrap();
        if original == self.value {
            return Ok(template.into());
        }
        let mut body = format!("type: {}\n", self.section_type);
        for (key, value) in self.value.as_object().unwrap() {
            if key != "type" {
                yaml_field(&mut body, key, value, 0);
            }
        }
        body.push('\n');
        let mut skip = false;
        for line in template.lines().filter(|line| line.starts_with('#')) {
            if let Some(text) = line.strip_prefix("# ") {
                if let Some((field, _)) = text.split_once(':') {
                    if !field.starts_with(' ')
                        && original.get(field).is_none()
                        && schema::section_schema(&self.section_type).unwrap()["properties"]
                            .get(field)
                            .is_some()
                    {
                        skip = self.value.get(field).is_some();
                    }
                } else if !text.starts_with(' ') {
                    skip = false;
                }
            }
            if !skip {
                body.push_str(line);
                body.push('\n');
            }
        }
        for (key, value) in original.as_object().unwrap() {
            if self.value.get(key).is_none() {
                let mut example = String::new();
                yaml_field(&mut example, key, value, 0);
                for line in example.lines() {
                    body.push_str("# ");
                    body.push_str(line);
                    body.push('\n');
                }
            }
        }
        Ok(body)
    }
}
fn yaml_field(out: &mut String, key: &str, value: &Value, indent: usize) {
    out.push_str(&format!("{}{key}:", " ".repeat(indent)));
    yaml_value(out, value, indent);
}
fn yaml_value(out: &mut String, value: &Value, indent: usize) {
    match value {
        Value::Object(object) if !object.is_empty() => {
            out.push('\n');
            for (key, value) in object {
                yaml_field(out, key, value, indent + 2);
            }
        }
        Value::Array(items) if !items.is_empty() => {
            out.push('\n');
            for item in items {
                out.push_str(&format!("{}-", " ".repeat(indent + 2)));
                yaml_value(out, item, indent + 2);
            }
        }
        Value::String(text) => {
            out.push(' ');
            out.push_str(&zellij_core::yaml::string(text));
            out.push('\n');
        }
        _ => {
            out.push(' ');
            out.push_str(&value.to_string());
            out.push('\n');
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn complete_json_values_validate_before_replacing_a_field() {
        let mut section = Section::new("hero").unwrap();
        for (path, value) in [
            ("cta", "[{\"label\":\"Read\"}]"),
            (
                "cta",
                "[{\"label\":\"Read\",\"href\":\"/guide\",\"unknown\":true}]",
            ),
            ("cta", "[null]"),
            ("headline", "42"),
            ("headline", "null"),
            ("headline", "\"First\" \"Second\""),
            ("background", "{\"gradient\":[\"accent\"]}"),
            ("type", "\"hero\""),
        ] {
            let before = section.value.clone();
            assert!(section.set_json(path, value).is_err(), "{path}={value}");
            assert_eq!(section.value, before);
        }
        section
            .set_json("background", r#"{"gradient":["accent","surface"]}"#)
            .unwrap();
        section
            .set_json(
                "cta",
                r#"[{"label":"Read 日本","href":"/guide","style":"ghost"}]"#,
            )
            .unwrap();
        let value: Value = serde_yaml_ng::from_str(&section.render().unwrap()).unwrap();
        assert_eq!(value, section.value);
        assert_eq!(value["cta"][0]["label"], "Read 日本");
    }
    #[test]
    fn optional_fields_can_be_removed_without_creating_missing_parents() {
        let mut section = Section::new("hero").unwrap();
        section
            .set_json(
                "cta",
                r#"[{"label":"Read","href":"/guide","style":"ghost"}]"#,
            )
            .unwrap();
        section.unset("cta.0.style").unwrap();
        section.unset("cta.0.style").unwrap();
        assert!(section.value["cta"][0].get("style").is_none());
        for path in [
            "headline",
            "type",
            "cta.0.label",
            "cta.0",
            "cta.1.style",
            "unknown",
            "background.gradient",
            "cta.00.style",
        ] {
            let before = section.value.clone();
            assert!(section.unset(path).is_err(), "{path}");
            assert_eq!(section.value, before);
        }
        section.unset("cta").unwrap();
        assert!(section.value.get("cta").is_none());
        section.render().unwrap();
    }
    #[test]
    fn structured_fields_retain_media_and_matrix_cross_field_rules() {
        let mut media = Section::new("hero-split").unwrap();
        media.set_json("visual", r#""assets/hero.svg""#).unwrap();
        assert!(media.value.get("terminal").is_none());
        media
            .set_json("terminal", r#"{"body":"echo hello","title":"Demo"}"#)
            .unwrap();
        assert!(media.value.get("visual").is_none());
        media.unset("terminal.title").unwrap();
        media.render().unwrap();
        media.unset("terminal").unwrap();
        assert!(media.render().is_err());

        let mut matrix = Section::new("matrix").unwrap();
        matrix
            .set_json("columns", r#"[{"label":"A"},{"label":"B"},{"label":"C"}]"#)
            .unwrap();
        assert!(matrix.render().is_err());
        matrix
            .set_json(
                "rows",
                r#"[{"label":"One","cells":["1","2","3"]},{"label":"Two","cells":["4","5","6"]}]"#,
            )
            .unwrap();
        matrix.render().unwrap();
    }
    #[test]
    fn every_embedded_template_is_accepted_and_unchanged() {
        for kind in schema::section_types() {
            let section = Section::new(&kind).unwrap();
            assert_eq!(
                section.render().unwrap(),
                schema::section_template(&kind).unwrap()
            );
        }
    }
    #[test]
    fn scalar_values_nested_arrays_and_optional_fields_round_trip() {
        let mut section = Section::new("hero").unwrap();
        let headline = "TRUE: 'quotes' \"double\" \\ line\n日本語\u{85}\u{2028}\u{ffff}";
        section.set("headline", headline).unwrap();
        section.set("badge", "New").unwrap();
        section.count("cta", 2).unwrap();
        section.set("cta.0.label", "Start").unwrap();
        section.set("cta.0.href", "/guide").unwrap();
        section.set("background.gradient.0", "accent").unwrap();
        section.set("background.gradient.1", "surface").unwrap();
        let body = section.render().unwrap();
        let parsed: Value = serde_yaml_ng::from_str(&body).unwrap();
        assert_eq!(parsed, section.value);
        assert_eq!(parsed["headline"], headline);
        assert!(!body.contains("# badge:") && body.contains("# subhead:"));
        assert!(section.set("id", "Bad Anchor").is_err());
        assert!(section.set("visualStyle", "unknown").is_err());
        assert!(section.set("unknown", "value").is_err());
        assert!(section.set("headline", "{unsafe: false}").is_ok());
        assert!(section.set("type", "hero-split").is_err());
        assert!(section.set("cta.02.label", "Bad").is_err());
        assert!(section.count("cta", 3).is_err());
    }
    #[test]
    fn matrix_counts_remain_aligned_and_new_items_do_not_copy_authored_values() {
        let mut section = Section::new("matrix").unwrap();
        section.set("columns.0.label", "Preserve").unwrap();
        section.count("columns", 4).unwrap();
        section.count("rows", 3).unwrap();
        assert_eq!(section.value["columns"][2]["label"], "TODO");
        assert!(
            section.value["rows"]
                .as_array()
                .unwrap()
                .iter()
                .all(|r| r["cells"].as_array().unwrap().len() == 4)
        );
        section.count("columns", 2).unwrap();
        section.validate().unwrap();
        assert_eq!(section.value["columns"][0]["label"], "Preserve");
        assert!(section.count("rows.0.cells", 3).is_err());
        assert!(section.count("columns", 6).is_err());
    }
    #[test]
    fn engine_rules_switch_media_and_pricing_shortcuts_are_unambiguous() {
        for (kind, field) in [
            ("hero-split", "visual"),
            ("steps-panel", "diagram"),
            ("media-panel", "video"),
        ] {
            let mut section = Section::new(kind).unwrap();
            section.set(field, "authored-content").unwrap();
            section.validate().unwrap();
            assert!(section.value.get("terminal").is_none());
            if kind != "media-panel" {
                assert!(section.render().unwrap().contains("# terminal:"));
            }
            if kind == "media-panel" {
                assert!(section.value.get("image").is_none());
            }
        }
        let mut section = Section::new("pricing-table").unwrap();
        section.count("tiers", 3).unwrap();
        section.set("tiers.1.name", "Team").unwrap();
        section.highlight("Team").unwrap();
        assert_eq!(section.value["tiers"][1]["highlighted"], true);
        assert!(section.highlight("TODO").is_err());
        section.highlight("3").unwrap();
        assert_eq!(section.value["tiers"][2]["highlighted"], true);
        section.highlight("none").unwrap();
        assert!(
            section.value["tiers"]
                .as_array()
                .unwrap()
                .iter()
                .all(|v| v["highlighted"] == false)
        );
        assert!(section.count("tiers", 0).is_err());
        assert!(section.count("tiers", 5).is_err());
        let mut cta = Section::new("cta-banner").unwrap();
        cta.count("cta", 2).unwrap();
        cta.set("cta.1.label", "Read").unwrap();
        cta.validate().unwrap();
        let mut media = Section::new("media-panel").unwrap();
        assert!(media.set("scrim", "1.1").is_err());
        assert!(media.set("scrim", "false").is_err());
        media.set("scrim", "0.5").unwrap();
    }
}
