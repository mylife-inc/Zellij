//! Parsed local RST fragments; selection follows Docutils, without parser plugins.
use super::invalid;
use std::collections::BTreeMap;
use zellij_core::Result;

#[derive(Debug)]
pub struct Spec {
    pub target: String,
    pub encoding: Option<String>,
    start_line: Option<i64>,
    end_line: Option<i64>,
    start_after: Option<String>,
    end_before: Option<String>,
    tab_width: usize,
}
impl Spec {
    pub fn parse(target: &str, options: &BTreeMap<String, String>, body: &str) -> Result<Self> {
        if target.is_empty() || !body.is_empty() {
            return Err(invalid("include requires a filename and no body"));
        }
        let mut spec = Self {
            target: target.into(),
            encoding: None,
            start_line: None,
            end_line: None,
            start_after: None,
            end_before: None,
            tab_width: 8,
        };
        for (key, value) in options {
            match key.as_str() {
                "encoding" if matches!(value.as_str(), "utf-8" | "utf-8-sig") => {
                    spec.encoding = Some(value.clone())
                }
                "start-line" | "end-line" => {
                    let n = value
                        .parse()
                        .map_err(|_| invalid("include line selection requires a signed integer"))?;
                    if key == "start-line" {
                        spec.start_line = Some(n);
                    } else {
                        spec.end_line = Some(n);
                    }
                }
                "start-after" if !value.is_empty() => spec.start_after = Some(value.clone()),
                "end-before" if !value.is_empty() => spec.end_before = Some(value.clone()),
                "tab-width" => {
                    spec.tab_width = value
                        .parse()
                        .map_err(|_| invalid("include tab-width requires an integer"))?;
                    if !(1..=64).contains(&spec.tab_width) {
                        return Err(invalid("include tab-width must be between 1 and 64"));
                    }
                }
                _ => {
                    return Err(invalid(format!(
                        "unsupported include option :{key}:; use literalinclude for code, or migrate parser/presentation options manually"
                    )));
                }
            }
        }
        Ok(spec)
    }
    pub fn excerpt(&self, raw: &str, encoding: &str) -> Result<String> {
        let raw = if encoding == "utf-8-sig" {
            raw.strip_prefix('\u{feff}').unwrap_or(raw)
        } else {
            raw
        };
        let selected;
        let mut text = if self.start_line.is_some() || self.end_line.is_some() {
            let lines: Vec<_> = raw.lines().collect();
            let bound = |value: i64| -> usize {
                if value < 0 {
                    (lines.len() as i64).saturating_add(value).max(0) as usize
                } else {
                    value.min(lines.len() as i64) as usize
                }
            };
            let start = self.start_line.map(bound).unwrap_or(0);
            let end = self.end_line.map(bound).unwrap_or(lines.len()).max(start);
            selected = lines[start..end].join("\n");
            selected.as_str()
        } else {
            raw
        };
        if let Some(marker) = &self.start_after {
            let start = text
                .find(marker)
                .ok_or_else(|| invalid("include start-after marker not found"))?;
            text = &text[start + marker.len()..];
        }
        if let Some(marker) = &self.end_before {
            let end = text
                .find(marker)
                .ok_or_else(|| invalid("include end-before marker not found"))?;
            text = &text[..end];
        }
        let mut out = String::new();
        let mut column = 0;
        for c in text.chars() {
            if c == '\t' {
                let spaces = self.tab_width - column % self.tab_width;
                out.extend(std::iter::repeat_n(' ', spaces));
                column += spaces;
            } else {
                out.push(c);
                column = if c == '\n' { 0 } else { column + 1 };
            }
            if out.len() > 32 * 1024 * 1024 {
                return Err(invalid("include expanded text exceeds 32 MiB"));
            }
        }
        Ok(out)
    }
}
