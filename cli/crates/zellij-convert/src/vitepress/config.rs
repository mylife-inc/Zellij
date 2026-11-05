//! Read a narrow, inert module grammar. No imports or helpers are executed.
use std::sync::LazyLock;

use regex::Regex;
use serde_json::Value;
use zellij_core::{Error, Result};

pub(super) fn invalid(message: impl std::fmt::Display) -> Error {
    Error::InvalidArgument(format!("VitePress: {message}"))
}

fn trivia(mut text: &str) -> Option<&str> {
    loop {
        text = text.trim_start();
        if let Some(rest) = text.strip_prefix("//") {
            text = rest.split_once('\n').map_or("", |(_, rest)| rest);
        } else if let Some(rest) = text.strip_prefix("/*") {
            text = rest.split_once("*/")?.1;
        } else {
            return Some(text);
        }
    }
}

/// Find the closing object delimiter without treating braces in comments or
/// strings as syntax. JSON5 subsequently validates every value inside it.
fn object_end(text: &str) -> Option<usize> {
    if !text.starts_with('{') {
        return None;
    }
    let bytes = text.as_bytes();
    let mut index = 0;
    let mut depth = 0;
    while index < bytes.len() {
        match bytes[index] {
            b'\'' | b'"' => {
                let quote = bytes[index];
                index += 1;
                while index < bytes.len() && bytes[index] != quote {
                    if bytes[index] == b'\\' {
                        index += 1;
                    }
                    index += 1;
                }
            }
            b'/' if bytes.get(index + 1) == Some(&b'/') => {
                while index < bytes.len() && bytes[index] != b'\n' {
                    index += 1;
                }
            }
            b'/' if bytes.get(index + 1) == Some(&b'*') => {
                index += 2;
                while index + 1 < bytes.len() && &bytes[index..index + 2] != b"*/" {
                    index += 1;
                }
                if index + 1 >= bytes.len() {
                    return None;
                }
                index += 1;
            }
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(index + 1);
                }
            }
            _ => {}
        }
        index += 1;
    }
    None
}

fn literal(raw: &str) -> Option<Value> {
    static IMPORT: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r#"^import\s*\{\s*defineConfig\s*\}\s*from\s*(?:'vitepress'|"vitepress")\s*;?"#)
            .unwrap()
    });
    let mut text = trivia(raw.trim_start_matches('\u{feff}'))?;
    let imported = if let Some(found) = IMPORT.find(text) {
        text = trivia(&text[found.end()..])?;
        true
    } else {
        false
    };
    text = if let Some(rest) = text.strip_prefix("export default") {
        trivia(rest)?
    } else if !imported {
        trivia(trivia(text.strip_prefix("module.exports")?)?.strip_prefix('=')?)?
    } else {
        return None;
    };
    let wrapped = if imported && let Some(rest) = text.strip_prefix("defineConfig") {
        text = trivia(trivia(rest)?.strip_prefix('(')?)?;
        true
    } else {
        false
    };
    let end = object_end(text)?;
    let value = json5::from_str(&text[..end]).ok()?;
    let mut rest = trivia(&text[end..])?;
    if wrapped {
        rest = trivia(rest.strip_prefix(')')?)?;
    }
    if let Some(tail) = rest.strip_prefix(';') {
        rest = trivia(tail)?;
    }
    rest.is_empty().then_some(value)
}

pub(super) fn read(raw: &str) -> Result<Value> {
    let value = literal(raw).ok_or_else(|| invalid("configuration must export a literal object or use import { defineConfig } from 'vitepress' with export default defineConfig({...}); computed JavaScript, other imports and type expressions are not executed or interpreted"))?;
    for (key, empty) in [
        ("rewrites", serde_json::json!({})),
        ("locales", serde_json::json!({})),
        ("srcExclude", serde_json::json!([])),
        ("vite", serde_json::json!({})),
    ] {
        if value.get(key).is_some_and(|v| v != &empty) {
            return Err(invalid(format!(
                "{key} is not supported by this importer; simplify it in a source copy before converting"
            )));
        }
    }
    for key in [
        "extends",
        "transformPageData",
        "transformHead",
        "transformHtml",
        "buildEnd",
        "postRender",
        "outDir",
        "cacheDir",
    ] {
        if value.get(key).is_some() {
            return Err(invalid(format!(
                "{key} requires manual migration and is not interpreted"
            )));
        }
    }
    Ok(value)
}

pub(super) fn string<'a>(value: &'a Value, key: &str) -> Result<Option<&'a str>> {
    match value.get(key) {
        None => Ok(None),
        Some(Value::String(value)) => Ok(Some(value)),
        _ => Err(invalid(format!("{key} must be a string"))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_literal_modules_and_standard_helper_without_executing_them() {
        for raw in [
            "export default {title: 'A } title', themeConfig: {sidebar: []}}; // end",
            "\u{feff}/* lead */ module.exports = {title: 'A } title', /* } */ themeConfig: {sidebar: []}}",
            "import { defineConfig } from 'vitepress'\nexport default defineConfig({title: 'A } title', themeConfig: {sidebar: []}}); /* end */",
        ] {
            assert_eq!(read(raw).unwrap()["title"], "A } title");
        }
    }

    #[test]
    fn refuses_dynamic_modules_extra_statements_and_unsupported_routing() {
        for raw in [
            "export default buildConfig()",
            "export default {title: process.env.TITLE}",
            "import x from './config'; export default x",
            "const config = {}; export default config",
            "export default {}; run()",
            "export default {sidebar: [...items]}",
            "export default {title: `template`}",
            "export default {} satisfies UserConfig",
            "export default defineConfig({})",
            "export default {rewrites: {'a.md': 'b.md'}}",
            "export default {srcExclude: ['**/private.md']}",
            "export default {locales: {fr: {}}}",
            "export default {vite: {publicDir: 'elsewhere'}}",
            "export default {} /* unterminated",
        ] {
            assert!(read(raw).is_err(), "{raw}");
        }
    }
}
