//! String scalars for the human-editable YAML templates shared by CLI tools.

/// JSON strings are YAML 1.2 double-quoted scalars. Escape the extra YAML line
/// breaks too, so a display name is preserved rather than folded into spaces.
pub fn string(value: &str) -> String {
    serde_json::to_string(value)
        .expect("serializing a string cannot fail")
        .replace('\u{85}', "\\u0085")
        .replace('\u{2028}', "\\u2028")
        .replace('\u{2029}', "\\u2029")
}
