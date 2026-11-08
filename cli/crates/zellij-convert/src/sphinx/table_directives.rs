//! Table syntax; local file data is supplied by the bounded resource resolver.
use super::{invalid, tables::Table};
use std::collections::BTreeMap;
use zellij_core::Result;

type Options = BTreeMap<String, String>;

fn count(opts: &Options, key: &str) -> Result<usize> {
    opts.get(key).map_or(Ok(0), |value| {
        value
            .parse()
            .map_err(|_| invalid(format!(":{key}: requires a nonnegative integer")))
    })
}

fn validate_options(kind: &str, opts: &Options) -> Result<()> {
    for (key, value) in opts {
        let supported = match key.as_str() {
            "widths" => value == "auto",
            "header-rows" | "stub-columns" => kind != "table",
            "header" | "delim" | "quote" => kind == "csv-table",
            "keepspace" => kind == "csv-table" && value.is_empty(),
            _ => false,
        };
        if !supported {
            return Err(invalid(format!(
                "unsupported {kind} option :{key}:; remote CSV sources, custom presentation and escape rules require manual migration"
            )));
        }
    }
    Ok(())
}

pub fn parse(kind: &str, opts: &Options, body: &str) -> Result<Table> {
    validate_options(kind, opts)?;
    if kind == "table" {
        let lines: Vec<_> = body.lines().collect();
        let (table, used) = super::tables::parse(&lines)?;
        if lines[used..].iter().any(|line| !line.trim().is_empty()) {
            return Err(invalid("table directive must contain exactly one table"));
        }
        return Ok(table);
    }
    let headers = count(opts, "header-rows")?;
    let stubs = count(opts, "stub-columns")?;
    let mut rows = if kind == "list-table" {
        let rows = bullets(body, 10000)?;
        let mut cells = Vec::new();
        for row in rows {
            let row = bullets(&row, 128)?;
            if cells
                .first()
                .is_some_and(|first: &Vec<String>| first.len() != row.len())
            {
                return Err(invalid(
                    "list-table needs the same number of cells in every row",
                ));
            }
            if (cells.len() + 1) * row.len() > 100000 {
                return Err(invalid("table exceeds 100000 cells"));
            }
            cells.push(row);
        }
        cells
    } else {
        csv(body, &Dialect::new(opts)?)?
    };
    if rows.is_empty() || headers >= rows.len() {
        return Err(invalid(
            "table needs at least one body row after its header rows",
        ));
    }
    if stubs > 0 && rows.iter().any(|row| stubs >= row.len()) {
        return Err(invalid(
            "stub-columns must leave at least one data column in each source row",
        ));
    }
    let mut supplemental = if let Some(header) = opts.get("header") {
        let parsed = csv(header, &Dialect::new(opts)?)?;
        if parsed.is_empty() {
            return Err(invalid("CSV :header: must contain header data"));
        }
        parsed
    } else {
        Vec::new()
    };
    let headers = headers + supplemental.len();
    supplemental.append(&mut rows);
    rows = supplemental;
    let columns = rows.iter().map(Vec::len).max().unwrap_or(0);
    if columns == 0 || columns > 128 || rows.len() > 10000 || rows.len() * columns > 100000 {
        return Err(invalid(
            "table needs 1–128 columns, at most 10000 rows and 100000 cells",
        ));
    }
    // CSV permits short rows; list-table uniformity was checked above.
    for row in &mut rows {
        row.resize(columns, String::new());
    }
    Ok(Table {
        headers,
        stubs,
        rows,
    })
}

/// Split one uniform bullet list, retaining its item bodies and indentation.
/// Calling this twice reads the row/cell structure, not arbitrary nested RST.
fn bullets(body: &str, limit: usize) -> Result<Vec<String>> {
    let mut items: Vec<String> = Vec::new();
    let mut marker = None;
    let mut indent = 0;
    for line in body.lines() {
        if line.trim().is_empty() {
            if let Some(item) = items.last_mut() {
                item.push('\n');
            }
            continue;
        }
        let first = line.as_bytes()[0];
        if matches!(first, b'*' | b'-' | b'+') && (line.len() == 1 || line.as_bytes()[1] == b' ') {
            if marker.is_some_and(|m| m != first) {
                return Err(invalid(
                    "list-table needs one uniform bullet list at each level",
                ));
            }
            if items.len() >= limit {
                return Err(invalid(format!(
                    "list-table exceeds {limit} items at this level"
                )));
            }
            marker = Some(first);
            indent = 1 + line[1..].bytes().take_while(|b| *b == b' ').count();
            items.push(line[indent..].to_owned());
            indent = indent.max(2);
        } else {
            let item = items
                .last_mut()
                .ok_or_else(|| invalid("list-table requires a two-level bullet list"))?;
            if line.bytes().take_while(|b| *b == b' ').count() < indent {
                return Err(invalid(
                    "list-table content must stay inside its row and cell bullets",
                ));
            }
            item.push('\n');
            item.push_str(&line[indent..]);
        }
    }
    if items.is_empty() {
        return Err(invalid(
            "list-table requires a nonempty two-level bullet list",
        ));
    }
    Ok(items)
}

struct Dialect {
    delimiter: char,
    quote: char,
    keep_space: bool,
}
impl Dialect {
    fn new(opts: &Options) -> Result<Self> {
        fn character(opts: &Options, key: &str, default: char) -> Result<char> {
            let Some(text) = opts.get(key) else {
                return Ok(default);
            };
            if key == "delim" && matches!(text.as_str(), "space" | "tab") {
                return Ok(if text == "tab" { '\t' } else { ' ' });
            }
            let mut chars = text.chars();
            let c = chars
                .next()
                .ok_or_else(|| invalid(format!(":{key}: needs one literal character")))?;
            if chars.next().is_some() || c.is_control() || c.is_whitespace() {
                return Err(invalid(format!(
                    ":{key}: needs one non-whitespace literal character (or :delim: space/tab)"
                )));
            }
            Ok(c)
        }
        let delimiter = character(opts, "delim", ',')?;
        let quote = character(opts, "quote", '"')?;
        if delimiter == quote {
            return Err(invalid("CSV delimiter and quote must be distinct"));
        }
        Ok(Self {
            delimiter,
            quote,
            keep_space: opts.contains_key("keepspace"),
        })
    }
}

/// Strict quoted CSV, including doubled quotes and line breaks in cells.
/// After a closing quote only a delimiter or record end is accepted.
fn csv(body: &str, dialect: &Dialect) -> Result<Vec<Vec<String>>> {
    #[derive(Clone, Copy, PartialEq)]
    enum State {
        Start,
        Bare,
        Quoted,
        Closed,
    }
    let mut state = State::Start;
    let mut rows = Vec::new();
    let mut row = Vec::new();
    let mut field = String::new();
    let mut record = false;
    let mut chars = body.chars().peekable();
    while let Some(c) = chars.next() {
        if state == State::Quoted {
            if c == dialect.quote {
                if chars.peek() == Some(&dialect.quote) {
                    chars.next();
                    field.push(c);
                } else {
                    state = State::Closed;
                }
            } else {
                field.push(c);
            }
            continue;
        }
        if c == '\n' {
            if record {
                row.push(std::mem::take(&mut field));
            }
            rows.push(std::mem::take(&mut row));
            record = false;
            state = State::Start;
            if rows.len() > 10000 {
                return Err(invalid("CSV table exceeds 10000 rows"));
            }
            continue;
        }
        record = true;
        if state == State::Start && c == ' ' && !dialect.keep_space {
            continue;
        }
        if c == dialect.delimiter {
            row.push(std::mem::take(&mut field));
            if row.len() >= 128 {
                return Err(invalid("CSV table exceeds 128 columns"));
            }
            state = State::Start;
        } else if state == State::Start && c == dialect.quote {
            state = State::Quoted;
        } else if state == State::Closed {
            return Err(invalid(
                "CSV text after a closing quote needs a delimiter or newline",
            ));
        } else {
            field.push(c);
            state = State::Bare;
        }
    }
    if state == State::Quoted {
        return Err(invalid("CSV table has an unterminated quoted field"));
    }
    if record {
        row.push(field);
        rows.push(row);
    }
    Ok(rows)
}

#[derive(Debug)]
pub struct ExternalCsv {
    pub target: String,
    pub encoding: Option<String>,
    pub options: Options,
    pub caption: Option<String>,
    pub language: String,
    pub depth: usize,
}
impl ExternalCsv {
    pub fn new(
        mut options: Options,
        body: &str,
        caption: Option<String>,
        language: &str,
        depth: usize,
    ) -> Result<Self> {
        let target = options.remove("file").unwrap();
        if target.is_empty() || !body.trim().is_empty() {
            return Err(invalid(
                "csv-table :file: requires a filename and no inline body",
            ));
        }
        let encoding = options.remove("encoding");
        if encoding
            .as_deref()
            .is_some_and(|s| !matches!(s, "utf-8" | "utf-8-sig"))
        {
            return Err(invalid(
                "csv-table file encoding must be utf-8 or utf-8-sig",
            ));
        }
        validate_options("csv-table", &options)?;
        count(&options, "header-rows")?;
        count(&options, "stub-columns")?;
        Dialect::new(&options)?;
        Ok(Self {
            target,
            encoding,
            options,
            caption,
            language: language.into(),
            depth,
        })
    }
}

pub fn supplemental_rows(options: &Options) -> Result<usize> {
    options.get("header").map_or(Ok(0), |header| {
        csv(header, &Dialect::new(options)?).map(|rows| rows.len())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn opts(values: &[(&str, &str)]) -> Options {
        values
            .iter()
            .map(|(key, value)| ((*key).into(), (*value).into()))
            .collect()
    }

    #[test]
    fn list_rows_preserve_cell_blocks_empty_items_and_variable_markers() {
        let table = parse("list-table", &opts(&[("header-rows", "1"), ("stub-columns", "1")]),
            "* - Name\n  - Value\n\n* - Alpha\n  - First paragraph.\n\n    Second paragraph.\n\n*\n  + Beta\n  +\n").unwrap();
        assert_eq!((table.headers, table.stubs), (1, 1));
        assert_eq!(table.rows[1][1], "First paragraph.\n\nSecond paragraph.");
        assert_eq!(table.rows[2], ["Beta", ""]);
        let table = parse(
            "list-table",
            &Options::new(),
            "-   * - One\n      - Two\n    * .. note:: Nested\n\n         Body\n",
        )
        .unwrap();
        assert_eq!(table.rows[0][0], "- One\n- Two");
        assert!(table.rows[0][1].starts_with(".. note:: Nested\n\n   Body"));
    }

    #[test]
    fn csv_keeps_quotes_newlines_header_rows_and_short_records() {
        let table = parse("csv-table", &opts(&[("header", "Name,Value,Extra"), ("header-rows", "1")]),
            "Group,Heading\nalpha, \"A, B and \"\"C\"\"\n\nSecond paragraph.\",tail\nbeta,short\n\n").unwrap();
        assert_eq!(table.headers, 2);
        assert_eq!(table.rows[2][1], "A, B and \"C\"\n\nSecond paragraph.");
        assert_eq!(table.rows[3], ["beta", "short", ""]);
        assert_eq!(table.rows[4], ["", "", ""]);
    }

    #[test]
    fn csv_dialects_handle_unicode_quotes_spaces_and_literal_backslashes() {
        let table = parse(
            "csv-table",
            &opts(&[("delim", "│"), ("quote", "'")]),
            "Café│'漢字│é and ''quote'''\nslash│\\*literal\\*",
        )
        .unwrap();
        assert_eq!(table.rows[0][1], "漢字│é and 'quote'");
        assert_eq!(table.rows[1][1], "\\*literal\\*");
        let table = parse("csv-table", &opts(&[("delim", "space")]), "a  b\nc \"\"").unwrap();
        assert_eq!(table.rows, [vec!["a", "b"], vec!["c", ""]]);
        let table = parse("csv-table", &opts(&[("keepspace", "")]), "a,  b").unwrap();
        assert_eq!(table.rows[0][1], "  b");
    }

    #[test]
    fn malformed_directives_and_unsupported_options_are_rejected() {
        for body in [
            "",
            "* only one level",
            "* - a\n  - b\n* - c",
            "* - a\n+ - b",
            "* - a\ntext outside",
            "* - a\n  + b",
        ] {
            assert!(
                parse("list-table", &Options::new(), body).is_err(),
                "{body}"
            );
        }
        for body in ["", "a,\"unfinished", "\"closed\" tail,b", "\"a\" ,b"] {
            assert!(parse("csv-table", &Options::new(), body).is_err(), "{body}");
        }
        for values in [
            vec![("header-rows", "1")],
            vec![("header-rows", "-1")],
            vec![("header-rows", "999999999999999999999999")],
            vec![("stub-columns", "2")],
            vec![("widths", "10 20")],
            vec![("file", "secret.csv")],
            vec![("url", "https://example.invalid")],
            vec![("escape", "\\")],
            vec![("quote", ",")],
            vec![("keepspace", "true")],
        ] {
            assert!(
                parse("csv-table", &opts(&values), "a,b").is_err(),
                "{values:?}"
            );
        }
        assert!(parse("csv-table", &opts(&[("stub-columns", "1")]), "a,b\nshort").is_err());
        assert!(parse("csv-table", &Options::new(), &vec!["x"; 129].join(",")).is_err());
        assert!(parse("csv-table", &Options::new(), &"x\n".repeat(10001)).is_err());
        assert!(
            parse(
                "csv-table",
                &Options::new(),
                &format!("{}\n", vec!["x"; 128].join(",")).repeat(782)
            )
            .is_err()
        );
        assert!(parse("list-table", &Options::new(), &"* - x\n".repeat(10001)).is_err());
    }
}
