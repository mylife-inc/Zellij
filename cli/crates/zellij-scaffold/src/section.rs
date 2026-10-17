//! Section YAML is generated and validated by the engine, then embedded in zel.
//! This preserves nested shapes, array bounds, and cross-field refinements.

use zellij_core::schema;

/// One section file for `section_type`, ready to edit.
pub fn section_file(section_type: &str) -> String {
    schema::section_template(section_type)
        .map(str::to_owned)
        .unwrap_or_else(|| format!("type: {section_type}\n"))
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
