//! Zellij's content model, as the CLI knows it.
//!
//! Compiled in from `generated/schema.json`, which the engine emits from the
//! same zod schemas that validate content at build time. The CLI cannot run a
//! zod schema, and a hand-written copy of nineteen section definitions would
//! disagree with the real ones within a release — so it embeds the real ones
//! instead, and `zel explain` describes exactly what `zel check` enforces.
//!
//! Regenerate with:
//!
//! ```text
//! node engine/packages/zellij/bin/zellij.mjs context cli/crates/zellij-core/generated/schema.json --json
//! ```

use serde_json::Value;
use std::sync::LazyLock;

const SCHEMA_JSON: &str = include_str!("../generated/schema.json");

/// The agent-facing reference, embedded so `zel context` needs nothing at all.
pub const CONTEXT_MARKDOWN: &str = include_str!("../generated/context.md");

static SCHEMA: LazyLock<Value> = LazyLock::new(|| {
    // Parsed once, and a panic here is correct: the file is compiled in, so
    // invalid JSON is a broken build rather than anything a user did.
    serde_json::from_str(SCHEMA_JSON).expect("embedded schema.json is not valid JSON")
});

/// Fields shared by every section, described once rather than nineteen times.
const SHARED_FIELDS: &[&str] = &["type", "id", "motion", "background", "padding", "titleGradient"];

/// One field of a section, in the terms someone writing YAML thinks in.
#[derive(Debug, Clone)]
pub struct Field {
    pub name: String,
    /// A readable type: `string`, `list of …`, or the alternatives of an enum.
    pub type_name: String,
    pub required: bool,
    /// The default as it would be written in YAML, when there is one.
    pub default: Option<String>,
    /// Present when the field is an enum, for prompting.
    pub choices: Vec<String>,
    /// True when the field is an object or a list of objects, and so has
    /// fields of its own worth showing separately.
    pub nested: bool,
}

/// Every section type the engine knows, in registration order.
pub fn section_types() -> Vec<String> {
    SCHEMA
        .get("sectionTypes")
        .and_then(Value::as_array)
        .map(|types| {
            types
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default()
}

pub fn is_section_type(name: &str) -> bool {
    section_types().iter().any(|known| known == name)
}

/// The fields of a section type, shared ones excluded.
pub fn fields_of(section_type: &str) -> Option<Vec<Field>> {
    let schema = SCHEMA.get("sections")?.get(section_type)?;
    Some(fields_from(schema, true))
}

/// The fields of any object schema.
pub fn fields_from(schema: &Value, skip_shared: bool) -> Vec<Field> {
    let properties = match schema.get("properties").and_then(Value::as_object) {
        Some(properties) => properties,
        None => return Vec::new(),
    };

    let required: Vec<&str> = schema
        .get("required")
        .and_then(Value::as_array)
        .map(|names| names.iter().filter_map(Value::as_str).collect())
        .unwrap_or_default();

    properties
        .iter()
        .filter(|(name, _)| !(skip_shared && SHARED_FIELDS.contains(&name.as_str())))
        .map(|(name, field)| Field {
            name: name.clone(),
            type_name: describe(field),
            required: required.contains(&name.as_str()),
            default: field.get("default").map(render_yaml),
            choices: choices_of(field),
            nested: object_behind(field).is_some(),
        })
        .collect()
}

/// The fields of an object nested inside a section — the tiles of a grid, the
/// tiers of a pricing table, the people in a team.
pub fn nested_fields(section_type: &str, field: &str) -> Option<Vec<Field>> {
    let schema = SCHEMA.get("sections")?.get(section_type)?;
    let property = schema.get("properties")?.get(field)?;
    let inner = object_behind(property)?;
    Some(fields_from(inner, false))
}

/// The object schema behind a field, looking through lists and unions.
pub fn object_behind(field: &Value) -> Option<&Value> {
    if field.get("properties").is_some() {
        return Some(field);
    }

    if let Some(items) = field.get("items") {
        return object_behind(items);
    }

    for key in ["anyOf", "oneOf"] {
        if let Some(options) = field.get(key).and_then(Value::as_array) {
            for option in options {
                if let Some(found) = object_behind(option) {
                    return Some(found);
                }
            }
        }
    }

    None
}

fn choices_of(field: &Value) -> Vec<String> {
    field
        .get("enum")
        .and_then(Value::as_array)
        .map(|values| values.iter().map(render_yaml).collect())
        .unwrap_or_default()
}

/// A JSON value as it would be written in a YAML file.
fn render_yaml(value: &Value) -> String {
    match value {
        Value::String(text) => text.clone(),
        Value::Array(items) => format!(
            "[{}]",
            items.iter().map(render_yaml).collect::<Vec<_>>().join(", ")
        ),
        other => other.to_string(),
    }
}

fn describe(field: &Value) -> String {
    if let Some(values) = field.get("enum").and_then(Value::as_array) {
        return values
            .iter()
            .map(render_yaml)
            .collect::<Vec<_>>()
            .join(" | ");
    }

    if let Some(constant) = field.get("const") {
        return render_yaml(constant);
    }

    for key in ["anyOf", "oneOf"] {
        if let Some(options) = field.get(key).and_then(Value::as_array) {
            let mut described: Vec<String> = options
                .iter()
                .filter(|option| option.get("type").and_then(Value::as_str) != Some("null"))
                .map(describe)
                .collect();
            described.dedup();
            return described.join(" | ");
        }
    }

    match field.get("type").and_then(Value::as_str) {
        Some("array") => {
            let inner = field.get("items").map(describe).unwrap_or_else(|| "any".into());
            format!("list of {inner}")
        }
        Some("object") => "object".to_owned(),
        Some(other) => other.to_owned(),
        None => "any".to_owned(),
    }
}
