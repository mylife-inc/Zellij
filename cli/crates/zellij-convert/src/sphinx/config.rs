//! A deliberately non-executing Python data subset, not a Python interpreter.
use super::invalid;
use serde_json::{Map, Value};
use zellij_core::Result;

struct Reader<'a> {
    rest: &'a str,
}
impl Reader<'_> {
    fn space(&mut self, multiline: bool) {
        loop {
            self.rest = self.rest.trim_start_matches(|c: char| {
                c == ' ' || c == '\t' || multiline && matches!(c, '\n' | '\r')
            });
            if self.rest.starts_with('#') {
                self.rest = self.rest.trim_start_matches(|c| c != '\n');
                if multiline {
                    continue;
                }
            }
            break;
        }
    }
    fn take(&mut self, s: &str) -> bool {
        if let Some(rest) = self.rest.strip_prefix(s) {
            self.rest = rest;
            true
        } else {
            false
        }
    }
    fn string(&mut self) -> Result<String> {
        let quote = self
            .rest
            .chars()
            .next()
            .ok_or_else(|| invalid("expected a quoted string"))?;
        if !matches!(quote, '\'' | '"') {
            return Err(invalid("expected a quoted string"));
        }
        self.rest = &self.rest[1..];
        let mut out = String::new();
        loop {
            let ch = self
                .rest
                .chars()
                .next()
                .ok_or_else(|| invalid("unterminated config string"))?;
            self.rest = &self.rest[ch.len_utf8()..];
            if ch == quote {
                return Ok(out);
            }
            if matches!(ch, '\n' | '\r') {
                return Err(invalid(
                    "multiline/triple-quoted strings are unsupported; use a single quoted literal",
                ));
            }
            if ch == '\\' {
                let escape = self
                    .rest
                    .chars()
                    .next()
                    .ok_or_else(|| invalid("incomplete string escape"))?;
                self.rest = &self.rest[escape.len_utf8()..];
                out.push(match escape {
                    '\\' | '\'' | '"' => escape,
                    'n' => '\n',
                    'r' => '\r',
                    't' => '\t',
                    _ => {
                        return Err(invalid(
                            "unsupported config string escape; write Unicode characters literally",
                        ));
                    }
                });
            } else {
                out.push(ch);
            }
        }
    }
    fn value(&mut self, depth: usize) -> Result<Value> {
        if depth > 32 {
            return Err(invalid("config nesting exceeds 32"));
        }
        self.space(true);
        if self.rest.starts_with(['\'', '"']) {
            return self.string().map(Value::String);
        }
        let sequence = if self.take("[") {
            Some("]")
        } else if self.take("(") {
            Some(")")
        } else {
            None
        };
        if let Some(end) = sequence {
            let mut items = Vec::new();
            self.space(true);
            while !self.take(end) {
                items.push(self.value(depth + 1)?);
                self.space(true);
                if self.take(end) {
                    if end == ")" && items.len() == 1 {
                        return Err(invalid("a one-item tuple requires a trailing comma"));
                    }
                    break;
                }
                if !self.take(",") {
                    return Err(invalid("expected a comma in config sequence"));
                }
                self.space(true);
            }
            return Ok(Value::Array(items));
        }
        if self.take("{") {
            let mut items = Map::new();
            self.space(true);
            while !self.take("}") {
                let key = self.string()?;
                self.space(true);
                if !self.take(":") {
                    return Err(invalid("expected ':' in config dictionary"));
                }
                if items.insert(key, self.value(depth + 1)?).is_some() {
                    return Err(invalid("duplicate config dictionary key"));
                }
                self.space(true);
                if self.take("}") {
                    break;
                }
                if !self.take(",") {
                    return Err(invalid("expected a comma in config dictionary"));
                }
                self.space(true);
            }
            return Ok(Value::Object(items));
        }
        for (token, value) in [
            ("True", Value::Bool(true)),
            ("False", Value::Bool(false)),
            ("None", Value::Null),
        ] {
            if self.take(token) {
                return Ok(value);
            }
        }
        let end = self
            .rest
            .find(|c: char| !c.is_ascii_digit() && c != '-')
            .unwrap_or(self.rest.len());
        if end > 0 {
            let n = self.rest[..end]
                .parse::<i64>()
                .map_err(|_| invalid("expected an integer literal"))?;
            self.rest = &self.rest[end..];
            return Ok(Value::from(n));
        }
        Err(invalid(
            "conf.py must contain literal assignments only; imports, functions, expressions and executable Python require a separate simplified configuration",
        ))
    }
}

pub fn parse(text: &str) -> Result<Value> {
    let mut reader = Reader {
        rest: text.trim_start_matches('\u{feff}'),
    };
    let mut values = Map::new();
    loop {
        reader.space(true);
        if reader.rest.is_empty() {
            return Ok(Value::Object(values));
        }
        let end = reader
            .rest
            .find(|c: char| !c.is_ascii_alphanumeric() && c != '_')
            .unwrap_or(reader.rest.len());
        let key = &reader.rest[..end];
        if key.is_empty() || key.starts_with(|c: char| c.is_ascii_digit()) {
            return Err(invalid("expected a config assignment"));
        }
        reader.rest = &reader.rest[end..];
        reader.space(false);
        if !reader.take("=") {
            return Err(invalid(
                "conf.py must contain literal assignments only; Python is never executed",
            ));
        }
        let value = reader.value(0)?;
        if values.insert(key.into(), value).is_some() {
            return Err(invalid(format!("duplicate config assignment {key}")));
        }
        reader.space(false);
        if !reader.rest.is_empty() && !reader.take("\r\n") && !reader.take("\n") {
            return Err(invalid(
                "unsupported expression or statement after config literal",
            ));
        }
    }
}

pub fn string<'a>(v: &'a Value, key: &str) -> Result<Option<&'a str>> {
    v.get(key)
        .map(|v| {
            v.as_str()
                .ok_or_else(|| invalid(format!("{key} must be a string")))
        })
        .transpose()
}
pub fn strings(v: &Value, key: &str) -> Result<Vec<String>> {
    let Some(v) = v.get(key) else {
        return Ok(Vec::new());
    };
    v.as_array()
        .ok_or_else(|| invalid(format!("{key} must be a list/tuple of strings")))?
        .iter()
        .map(|v| {
            v.as_str()
                .map(str::to_owned)
                .ok_or_else(|| invalid(format!("{key} must contain strings")))
        })
        .collect()
}
pub fn validate(v: &Value) -> Result<()> {
    for key in v.as_object().unwrap().keys() {
        if !matches!(
            key.as_str(),
            "project"
                | "author"
                | "copyright"
                | "version"
                | "release"
                | "root_doc"
                | "master_doc"
                | "source_suffix"
                | "source_encoding"
                | "exclude_patterns"
                | "include_patterns"
                | "extensions"
                | "language"
                | "html_title"
                | "html_short_title"
                | "html_theme"
                | "html_theme_options"
                | "html_theme_path"
                | "templates_path"
                | "html_static_path"
                | "html_extra_path"
                | "html_baseurl"
                | "html_logo"
                | "html_favicon"
                | "html_css_files"
                | "html_js_files"
                | "html_show_sourcelink"
                | "html_show_sphinx"
                | "html_show_copyright"
                | "htmlhelp_basename"
                | "pygments_style"
                | "highlight_language"
                | "smartquotes"
                | "nitpicky"
        ) {
            return Err(invalid(format!(
                "unsupported config setting {key}; global content transforms, custom routing and extensions require manual migration"
            )));
        }
    }
    if !strings(v, "extensions")?.is_empty() {
        return Err(invalid(
            "Sphinx extensions (including MyST/autodoc) require manual migration",
        ));
    }
    if v.get("language")
        .is_some_and(|language| !language.is_null() && language.as_str() != Some(""))
    {
        return Err(invalid(
            "language-dependent processing (including localized image selection) requires manual migration; use a source copy with language = None and explicitly chosen resources",
        ));
    }
    if v.get("include_patterns").is_some() && strings(v, "include_patterns")? != ["**"] {
        return Err(invalid(
            "only the default include_patterns = ['**'] is supported",
        ));
    }
    if string(v, "source_encoding")?.is_some_and(|s| !matches!(s, "utf-8" | "utf-8-sig")) {
        return Err(invalid("only UTF-8 Sphinx source is supported"));
    }
    if let Some(suffix) = v.get("source_suffix") {
        let valid = suffix.as_str() == Some(".rst")
            || suffix == &serde_json::json!([".rst"])
            || suffix == &serde_json::json!({".rst":"restructuredtext"});
        if !valid {
            return Err(invalid(
                "only .rst reStructuredText source_suffix is supported",
            ));
        }
    }
    if v.get("root_doc").is_some()
        && v.get("master_doc").is_some()
        && v["root_doc"] != v["master_doc"]
    {
        return Err(invalid("root_doc and master_doc disagree"));
    }
    super::patterns::validate_exclusions(&strings(v, "exclude_patterns")?)?;
    for key in [
        "templates_path",
        "html_static_path",
        "html_extra_path",
        "html_theme_path",
    ] {
        for path in strings(v, key)? {
            if path.contains(['*', '?', '[', ']']) {
                return Err(invalid(format!(
                    "{key}: glob patterns are unsupported; use explicit contained file/directory paths"
                )));
            }
            zellij_core::files::relative_path(camino::Utf8Path::new(&path))?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn reads_literal_python_without_evaluation() {
        let v = parse("\u{feff}# config\nproject = 'Café: Docs' # title\nextensions = []\nexclude_patterns = (\n '_build', # output\n 'Thumbs.db',\n)\nsource_suffix = {'.rst': 'restructuredtext'}\nhtml_show_sphinx = False\n").unwrap();
        validate(&v).unwrap();
        assert_eq!(v["project"], "Café: Docs");
        assert_eq!(
            strings(&v, "exclude_patterns").unwrap(),
            ["_build", "Thumbs.db"]
        );
        for text in [
            "import os",
            "project = str(42)",
            "project='x'; open('marker', 'w')",
            "project=f'{run()}'",
            "project='x'\nproject='y'",
            "x={'a':1,'a':2}",
            "project='x' + 'y'",
            "extensions = [x for x in y]",
            "if True:\n project='x'",
            "project = '''multiline'''",
            "project = Trueish",
            "exclude_patterns = ('private')",
        ] {
            assert!(parse(text).is_err(), "{text}");
        }
    }
}
