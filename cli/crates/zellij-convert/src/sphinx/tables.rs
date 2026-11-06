//! Rectangular RST tables. Geometry is measured in display columns, never UTF-8
//! bytes. Cell bodies are parsed by rst.rs after their common margin is removed.
use super::invalid;
use std::ops::Range;
use unicode_width::UnicodeWidthChar;
use zellij_core::Result;

#[derive(Debug)]
pub struct Table {
    pub headers: usize,
    pub rows: Vec<Vec<String>>,
}

fn error() -> zellij_core::Error {
    invalid(
        "malformed or spanning table cells; use a rectangular grid/simple table without row or column spans",
    )
}

fn simple_border(line: &str, marker: char) -> Option<Vec<Range<usize>>> {
    if !line.starts_with(marker) || !line.chars().all(|c| c == marker || c == ' ') {
        return None;
    }
    let mut ranges = Vec::new();
    let mut start = None;
    for (i, ch) in line.char_indices().chain([(line.len(), ' ')]) {
        if ch == marker {
            start.get_or_insert(i);
        } else if let Some(start) = start.take() {
            ranges.push(start..i);
        }
    }
    (ranges.len() >= 2).then_some(ranges)
}

pub fn starts(line: &str) -> bool {
    line.starts_with("+-") || simple_border(line, '=').is_some()
}

struct Columns<'a> {
    text: &'a str,
    offsets: Vec<Option<usize>>,
}
impl<'a> Columns<'a> {
    fn new(text: &'a str) -> Result<Self> {
        let mut offsets = vec![Some(0)];
        for (byte, ch) in text.char_indices() {
            let width = ch.width().ok_or_else(error)?;
            let end = byte + ch.len_utf8();
            if width == 0 {
                if byte == 0 {
                    return Err(error());
                }
                *offsets.last_mut().unwrap() = Some(end);
            } else {
                offsets.extend(std::iter::repeat_n(None, width - 1));
                offsets.push(Some(end));
            }
        }
        Ok(Self { text, offsets })
    }
    fn width(&self) -> usize {
        self.offsets.len() - 1
    }
    fn slice(&self, range: Range<usize>) -> Result<&'a str> {
        let at = |i: usize| self.offsets[i.min(self.width())].ok_or_else(error);
        Ok(&self.text[at(range.start)?..at(range.end)?])
    }
}

fn finish_row(table: &mut Table, cells: Vec<Vec<String>>) -> Result<()> {
    if table.rows.len() >= 10000 || (table.rows.len() + 1) * cells.len() > 100000 {
        return Err(invalid("table exceeds 10000 rows or 100000 cells"));
    }
    table.rows.push(
        cells
            .into_iter()
            .map(|lines| {
                let margin = lines
                    .iter()
                    .filter(|s| !s.trim().is_empty())
                    .map(|s| s.len() - s.trim_start_matches(' ').len())
                    .min()
                    .unwrap_or(0);
                lines
                    .iter()
                    .map(|s| {
                        if s.trim().is_empty() {
                            ""
                        } else {
                            &s[margin..]
                        }
                    })
                    .collect::<Vec<_>>()
                    .join("\n")
                    .trim_matches('\n')
                    .to_owned()
            })
            .collect(),
    );
    Ok(())
}

pub fn parse(lines: &[&str]) -> Result<(Table, usize)> {
    let top = lines.first().ok_or_else(error)?.trim_end();
    let (columns, grid) = if top.starts_with("+-") {
        if !top.ends_with('+') || !top.bytes().all(|c| c == b'+' || c == b'-') {
            return Err(error());
        }
        let boundaries: Vec<_> = top.match_indices('+').map(|(i, _)| i).collect();
        let columns: Vec<_> = boundaries.windows(2).map(|p| p[0] + 1..p[1]).collect();
        if columns.is_empty() || columns.iter().any(Range::is_empty) {
            return Err(error());
        }
        (columns, true)
    } else {
        (simple_border(top, '=').ok_or_else(error)?, false)
    };
    if columns.len() > 128 || top.len() > 16384 {
        return Err(invalid(
            "table exceeds 128 columns or a 16384-column border",
        ));
    }
    let mut table = Table {
        headers: 0,
        rows: Vec::new(),
    };
    let mut cells: Option<Vec<Vec<String>>> = None;
    let mut header_seen = false;
    for (i, line) in lines.iter().enumerate().skip(1) {
        let line = line.trim_end_matches('\r');
        let trimmed = line.trim_end();
        let border = if grid && trimmed.starts_with('+') {
            let marker = trimmed.as_bytes().get(1).copied().ok_or_else(error)?;
            if !matches!(marker, b'-' | b'=')
                || trimmed.len() != top.len()
                || !top
                    .bytes()
                    .zip(trimmed.bytes())
                    .all(|(a, b)| if a == b'+' { b == a } else { b == marker })
            {
                return Err(error());
            }
            Some(marker as char)
        } else if !grid
            && trimmed.chars().all(|c| c == '=' || c == '-' || c == ' ')
            && !trimmed.is_empty()
        {
            let marker = trimmed.chars().next().unwrap();
            if simple_border(trimmed, marker).as_ref() != Some(&columns) {
                return Err(error());
            }
            Some(marker)
        } else {
            None
        };
        if let Some(marker) = border {
            if let Some(cells) = cells.take() {
                finish_row(&mut table, cells)?;
            } else if grid || marker != '=' || table.rows.is_empty() {
                return Err(error());
            }
            let ends =
                (grid || marker == '=') && lines.get(i + 1).is_none_or(|l| l.trim().is_empty());
            if ends {
                if (grid && marker != '-')
                    || (!grid && marker != '=')
                    || table.rows.len() <= table.headers
                {
                    return Err(error());
                }
                return Ok((table, i + 1));
            }
            if marker == '=' {
                if header_seen {
                    return Err(error());
                }
                table.headers = table.rows.len();
                header_seen = true;
            }
            continue;
        }
        if trimmed.is_empty() {
            if grid {
                return Err(error());
            }
            if let Some(cells) = &mut cells {
                for cell in cells {
                    cell.push(String::new());
                }
            }
            continue;
        }
        let visual = Columns::new(if grid { trimmed } else { line })?;
        if grid {
            if visual.width() != top.len() {
                return Err(error());
            }
            for at in std::iter::once(0).chain(columns.iter().map(|c| c.end)) {
                if visual.slice(at..at + 1)? != "|" {
                    return Err(error());
                }
            }
        } else {
            for pair in columns.windows(2) {
                if !visual
                    .slice(pair[0].end..pair[1].start)?
                    .chars()
                    .all(|c| c == ' ')
                {
                    return Err(error());
                }
            }
            let first = visual.slice(columns[0].clone())?;
            if !first.trim().is_empty() {
                if let Some(previous) = cells.take() {
                    finish_row(&mut table, previous)?;
                }
            } else if cells.is_none() {
                return Err(invalid(
                    "simple-table continuation has no preceding row; use an empty comment in the first column for an empty cell",
                ));
            }
        }
        let cells = cells.get_or_insert_with(|| vec![Vec::new(); columns.len()]);
        for (index, column) in columns.iter().enumerate() {
            let end = if !grid && index + 1 == columns.len() {
                visual.width()
            } else {
                column.end
            };
            cells[index].push(
                visual
                    .slice(column.start..end.max(column.start))?
                    .to_owned(),
            );
        }
    }
    Err(invalid("table is missing its closing border"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grid_coordinates_use_display_width_and_preserve_combining_text() {
        let (table, used) =
            parse(&["+------+-----+", "| 漢字 | e\u{301}   |", "+------+-----+"]).unwrap();
        assert_eq!(used, 3);
        assert_eq!(table.headers, 0);
        assert_eq!(table.rows[0][0].trim(), "漢字");
        assert_eq!(table.rows[0][1].trim(), "e\u{301}");
        // A wide character crossing a cell boundary is not sliced or lost.
        assert!(parse(&["+--+---+", "| 漢| b |", "+--+---+"]).is_err());
        assert!(parse(&["+--+---+", "\u{301}|a | b |", "+--+---+"]).is_err());
    }

    #[test]
    fn simple_rows_keep_continuations_blank_lines_and_unbounded_last_column() {
        let (table, _) = parse(&[
            "===  ===",
            "Key  Value",
            "===  ===",
            "a    first",
            "     continuation",
            "",
            "     another paragraph",
            "---  ---",
            "",
            "b    unbounded text",
            "===  ===",
        ])
        .unwrap();
        assert_eq!(table.headers, 1);
        assert_eq!(table.rows.len(), 3);
        assert_eq!(table.rows[1][1], "first\ncontinuation\n\nanother paragraph");
        assert_eq!(table.rows[2][1], "unbounded text");
        let (table, _) = parse(&[
            "===  ===",
            "Key  Value",
            "---  ---",
            "===  ===",
            "a    body",
            "---  ---",
            "",
            "===  ===",
        ])
        .unwrap();
        assert_eq!(table.headers, 1);
        assert_eq!(table.rows.len(), 2);
    }

    #[test]
    fn rejects_spans_multiple_header_separators_and_excessive_geometry() {
        for lines in [
            vec!["+---+---+", "| a     |", "+---+---+"],
            vec![
                "===  ===", "a    b", "===  ===", "c    d", "===  ===", "e    f", "===  ===",
            ],
            vec!["===  ===", "a    b", "--------", "===  ==="],
        ] {
            assert!(parse(&lines).is_err(), "{lines:?}");
        }
        let many = vec!["="; 129].join(" ");
        assert!(
            parse(&[&many])
                .unwrap_err()
                .to_string()
                .contains("128 columns")
        );
    }
}
