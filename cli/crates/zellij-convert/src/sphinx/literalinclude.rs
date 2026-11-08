//! Local literal text only: no RST parsing, execution, network reads or asset copying.
use super::invalid;
use std::collections::BTreeMap;
use zellij_core::Result;

const TOTAL_BYTES: usize = 32 * 1024 * 1024;
const SELECTED_LINES: usize = 100_000;

#[derive(Debug)]
pub struct Spec {
    pub(super) target: String,
    pub(super) language: String,
    pub(super) encoding: Option<String>,
    lines: Option<Vec<(usize, Option<usize>)>>,
    start: Option<(String, bool)>,
    end: Option<(String, bool)>,
    dedent: Option<Option<usize>>,
    tab_width: Option<usize>,
}

fn number(value: &str) -> Result<usize> {
    if value.is_empty() || !value.bytes().all(|b| b.is_ascii_digit()) {
        return Err(invalid("literalinclude requires an unsigned integer"));
    }
    value
        .parse()
        .map_err(|_| invalid("literalinclude integer is too large"))
}

impl Spec {
    pub fn parse(
        target: &str,
        options: &BTreeMap<String, String>,
        body: &str,
        language: &str,
    ) -> Result<Self> {
        if target.is_empty() || !body.is_empty() {
            return Err(invalid("literalinclude requires a filename and no body"));
        }
        let mut spec = Self {
            target: target.into(),
            language: language.into(),
            encoding: None,
            lines: None,
            start: None,
            end: None,
            dedent: None,
            tab_width: None,
        };
        for (key, value) in options {
            match key.as_str() {
                "language" if !value.is_empty() => spec.language = value.clone(),
                "encoding" if matches!(value.as_str(), "utf-8" | "utf-8-sig") => {
                    spec.encoding = Some(value.clone())
                }
                "dedent" => {
                    spec.dedent = Some(if value.is_empty() {
                        None
                    } else {
                        Some(number(value)?)
                    })
                }
                "tab-width" => {
                    let width = number(value)?;
                    if !(1..=64).contains(&width) {
                        return Err(invalid(
                            "literalinclude :tab-width: must be between 1 and 64",
                        ));
                    }
                    spec.tab_width = Some(width);
                }
                "lines" => {
                    let mut ranges = Vec::new();
                    for part in value.split(',').map(str::trim) {
                        let range = if let Some((start, end)) = part.split_once('-') {
                            let start = if start.trim().is_empty() {
                                1
                            } else {
                                number(start.trim())?
                            };
                            let end = if end.trim().is_empty() {
                                None
                            } else {
                                Some(number(end.trim())?)
                            };
                            if part == "-" {
                                return Err(invalid("literalinclude empty line range"));
                            }
                            (start, end)
                        } else {
                            let line = number(part)?;
                            (line, Some(line))
                        };
                        if range.0 == 0 || range.1.is_some_and(|end| end < range.0) {
                            return Err(invalid(
                                "literalinclude line numbers are one-based and ranges must ascend",
                            ));
                        }
                        ranges.push(range);
                    }
                    spec.lines = Some(ranges);
                }
                "start-at" | "start-after" | "end-at" | "end-before" if !value.is_empty() => {
                    let slot = if key.starts_with("start-") {
                        &mut spec.start
                    } else {
                        &mut spec.end
                    };
                    if slot
                        .replace((value.clone(), key.ends_with("-at")))
                        .is_some()
                    {
                        return Err(invalid(
                            "literalinclude start/end marker options are mutually exclusive",
                        ));
                    }
                }
                _ => {
                    return Err(invalid(format!(
                        "unsupported literalinclude option :{key}:; requires manual migration"
                    )));
                }
            }
        }
        if !spec
            .language
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "_+-".contains(c))
        {
            return Err(invalid("unsupported literalinclude language identifier"));
        }
        Ok(spec)
    }

    pub(super) fn excerpt(&self, raw: &str, encoding: &str) -> Result<String> {
        let raw = if encoding == "utf-8-sig" {
            raw.strip_prefix('\u{feff}').unwrap_or(raw)
        } else {
            raw
        };
        let expanded;
        let raw = if let Some(width) = self.tab_width {
            let mut out = String::new();
            let mut column = 0;
            for c in raw.chars() {
                if c == '\t' {
                    let spaces = width - column % width;
                    out.extend(std::iter::repeat_n(' ', spaces));
                    column += spaces;
                } else {
                    out.push(c);
                    column = if c == '\n' { 0 } else { column + 1 };
                }
                if out.len() > TOTAL_BYTES {
                    return Err(invalid("literalinclude expanded text exceeds 32 MiB"));
                }
            }
            expanded = out;
            &expanded
        } else {
            raw
        };
        let mut lines: Vec<_> = raw.split_inclusive('\n').collect();
        if let Some((marker, inclusive)) = &self.start {
            let pos = lines
                .iter()
                .position(|line| line.contains(marker))
                .ok_or_else(|| invalid("literalinclude start marker not found"))?;
            lines = lines[pos + usize::from(!inclusive)..].to_vec();
        }
        if let Some((marker, inclusive)) = &self.end {
            // Sphinx permits one marker as both start-at and end-before: the
            // first remaining line cannot terminate an end-before excerpt.
            let pos = lines
                .iter()
                .enumerate()
                .position(|(i, line)| (*inclusive || i > 0) && line.contains(marker))
                .ok_or_else(|| invalid("literalinclude end marker not found"))?;
            lines.truncate(pos + usize::from(*inclusive));
        }
        if let Some(ranges) = &self.lines {
            let mut selected = Vec::new();
            let mut bytes = 0usize;
            for &(start, end) in ranges {
                let end = end.unwrap_or(lines.len());
                if start > end || end > lines.len() {
                    return Err(invalid(
                        "literalinclude line selection is outside the excerpt",
                    ));
                }
                let count = end - start + 1;
                if count > SELECTED_LINES - selected.len() {
                    return Err(invalid("literalinclude selects more than 100000 lines"));
                }
                for line in &lines[start - 1..end] {
                    bytes += line.len();
                    if bytes > TOTAL_BYTES {
                        return Err(invalid("literalinclude selected text exceeds 32 MiB"));
                    }
                    selected.push(*line);
                }
            }
            lines = selected;
        }
        let auto_prefix = if self.dedent == Some(None) {
            let mut prefix: Option<&str> = None;
            for line in &lines {
                if line.trim_matches([' ', '\t', '\n']).is_empty() {
                    continue;
                }
                let indent = &line[..line.len() - line.trim_start_matches([' ', '\t']).len()];
                prefix = Some(match prefix {
                    None => indent,
                    Some(previous) => {
                        &previous[..previous
                            .bytes()
                            .zip(indent.bytes())
                            .take_while(|(a, b)| a == b)
                            .count()]
                    }
                });
            }
            prefix.unwrap_or("")
        } else {
            ""
        };
        let mut out = String::new();
        for line in lines {
            let line = match self.dedent {
                Some(None) => {
                    if line.trim_matches([' ', '\t', '\n']).is_empty() {
                        if line.ends_with('\n') { "\n" } else { "" }
                    } else {
                        &line[auto_prefix.len()..]
                    }
                }
                Some(Some(count)) => {
                    let content = line.trim_end_matches('\n');
                    let strip = count.min(content.chars().count());
                    if !content.chars().take(strip).all(|c| c == ' ' || c == '\t') {
                        return Err(invalid(
                            "literalinclude :dedent: would remove non-whitespace text",
                        ));
                    }
                    &line[strip..] // stripped characters are single-byte spaces/tabs
                }
                None => line,
            };
            out.push_str(line);
        }
        Ok(out)
    }
}
