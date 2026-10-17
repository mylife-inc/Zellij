//! String scalars for the human-editable YAML templates shared by CLI tools.

use std::fmt::Write;

/// Use JSON's quoting, then escape characters YAML forbids literally and its
/// extra line breaks. This also preserves values in YAML 1.1 parsers.
pub fn string(value: &str) -> String {
    let json = serde_json::to_string(value).expect("serializing a string cannot fail");
    let mut out = String::with_capacity(json.len());
    for ch in json.chars() {
        if matches!(
            ch,
            '\u{7f}'..='\u{9f}' | '\u{2028}' | '\u{2029}' | '\u{fffe}' | '\u{ffff}'
        ) {
            write!(out, "\\u{:04x}", u32::from(ch)).expect("writing to a string cannot fail");
        } else {
            out.push(ch);
        }
    }
    out
}
