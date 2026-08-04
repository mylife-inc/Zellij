//! A section file, generated from the schema the engine validates against.
//!
//! Required fields are written with a placeholder; optional ones are written
//! **commented out, with their defaults**. Commented rather than omitted,
//! because a field you can see and uncomment is discoverable and a field you
//! have to look up is not.

use zellij_core::schema::{self, Field};

/// One section file for `section_type`, ready to edit.
///
/// An unknown type still produces a file — with only the shared fields — since
/// the caller has already been told the type is unknown and writing nothing
/// after saying so is a second failure.
pub fn section_file(section_type: &str) -> String {
    let mut out = format!("type: {section_type}\n");

    let Some(fields) = schema::fields_of(section_type) else {
        return out;
    };

    let (required, optional): (Vec<Field>, Vec<Field>) =
        fields.into_iter().partition(|field| field.required);

    for field in &required {
        out.push_str(&render(field, section_type, false));
    }

    if !optional.is_empty() {
        out.push_str("\n# Uncomment what you need.\n");
        for field in &optional {
            out.push_str(&render(field, section_type, true));
        }
    }

    out
}

fn render(field: &Field, section_type: &str, commented: bool) -> String {
    let prefix = if commented { "# " } else { "" };
    let name = &field.name;

    // A nested field is a list of objects far more often than a bare one, and
    // showing its shape is the whole reason `zel explain` exists — a scaffold
    // that wrote `cta:` and stopped would send the reader back to the docs.
    if field.nested {
        let mut out = format!("{prefix}{name}:\n");

        if let Some(inner) = schema::nested_fields(section_type, name) {
            let mut entry = String::new();
            for sub in inner.iter().filter(|sub| sub.required) {
                entry.push_str(&format!("{}: {}, ", sub.name, placeholder(sub)));
            }
            let entry = entry.trim_end_matches(", ").to_string();

            if !entry.is_empty() {
                out.push_str(&format!("{prefix}  - {{ {entry} }}\n"));
                return out;
            }
        }

        out.push_str(&format!("{prefix}  - {{}}\n"));
        return out;
    }

    let value = field.default.clone().unwrap_or_else(|| placeholder(field));

    let annotation = if !field.choices.is_empty() {
        format!("   # {}", field.choices.join(" | "))
    } else {
        String::new()
    };

    format!("{prefix}{name}: {value}{annotation}\n")
}

fn placeholder(field: &Field) -> String {
    if let Some(first) = field.choices.first() {
        return first.clone();
    }

    match field.type_name.as_str() {
        "boolean" => "false".into(),
        "integer" | "number" => "0".into(),
        t if t.starts_with("list") => "[]".into(),
        _ => "TODO".into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_hero_has_its_required_field_uncommented() {
        let file = section_file("hero");

        assert!(file.starts_with("type: hero\n"), "{file}");
        assert!(
            file.lines().any(|l| l.starts_with("headline:")),
            "headline should be live, not commented:\n{file}"
        );
    }

    #[test]
    fn optional_fields_are_commented_out() {
        let file = section_file("hero");

        // `badge` is optional, so it must not be a live key.
        assert!(
            !file.lines().any(|l| l.starts_with("badge:")),
            "badge should be commented:\n{file}"
        );
        assert!(file.contains("# badge:"), "{file}");
    }

    #[test]
    fn enum_fields_list_their_choices() {
        let file = section_file("hero");
        assert!(
            file.contains("floating | framed | full-bleed"),
            "visualStyle should show its choices:\n{file}"
        );
    }

    #[test]
    fn nested_fields_show_their_shape() {
        let file = section_file("hero");
        // `cta` is a list of objects whose entries require href and label.
        assert!(file.contains("# cta:"), "{file}");
        assert!(file.contains("href:"), "{file}");
        assert!(file.contains("label:"), "{file}");
    }

    // The uncommented part of the file is what the validator will see. If that
    // is not parseable YAML the scaffold has produced something that cannot
    // build, which is the one thing it must never do.
    #[test]
    fn the_live_part_of_every_section_type_parses() {
        for section_type in schema::section_types() {
            let file = section_file(&section_type);
            let live: String = file
                .lines()
                .filter(|line| !line.trim_start().starts_with('#'))
                .collect::<Vec<_>>()
                .join("\n");

            serde_yaml_ng::from_str::<serde_yaml_ng::Value>(&live).unwrap_or_else(|err| {
                panic!("{section_type} generated invalid YAML: {err}\n{live}")
            });
        }
    }

    #[test]
    fn an_unknown_type_still_writes_something() {
        assert_eq!(section_file("not-a-type"), "type: not-a-type\n");
    }
}
