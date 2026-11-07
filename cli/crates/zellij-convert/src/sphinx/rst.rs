//! Block-oriented reStructuredText subset. Unsupported constructs fail rather
//! than being mistaken for Markdown; literal blocks never enter the parser.
use super::{
    invalid,
    substitutions::{self, Definition, Expansion},
};
use std::collections::{BTreeMap, BTreeSet};
use unicode_normalization::UnicodeNormalization;
use zellij_core::Result;

#[derive(Debug)]
pub enum Block {
    Heading {
        level: usize,
        text: String,
        id: String,
    },
    Text(String),
    Code {
        language: String,
        body: String,
    },
    Image {
        target: String,
        alt: String,
    },
    Toc(Toc),
    List {
        numbering: Option<super::lists::Numbering>,
        items: Vec<Vec<Block>>,
    },
    Table {
        caption: Option<String>,
        headers: usize,
        stubs: usize,
        rows: Vec<Vec<Vec<Block>>>,
    },
    Note {
        title: String,
        body: Vec<Block>,
    },
}
#[derive(Debug, Default)]
pub struct Toc {
    pub caption: Option<String>,
    pub hidden: bool,
    pub entries: Vec<(Option<String>, String)>,
}
#[derive(Debug)]
pub struct Document {
    pub title: String,
    pub blocks: Vec<Block>,
    pub labels: BTreeMap<String, (String, String)>,
    pub links: BTreeMap<String, String>,
    pub notes: Vec<String>,
    pub substitutions: substitutions::Definitions,
}
pub fn name(text: &str) -> String {
    text.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}
fn id(text: &str) -> Result<String> {
    let mut out = String::new();
    // Docutils normalizes ASCII-compatible section IDs. Non-decomposing
    // Unicode and duplicate IDs need an explicit ASCII label, not a guess.
    for c in text.to_lowercase().nfkd() {
        if c.is_ascii_alphanumeric() {
            out.push(c);
        } else if c.is_ascii() && !out.ends_with('-') {
            out.push('-');
        } else if !c.is_ascii() && !unicode_normalization::char::is_combining_mark(c) {
            return Err(invalid(
                "heading needs an explicit ASCII label for this Unicode identifier",
            ));
        }
    }
    let out = out
        .trim_start_matches(|c: char| c == '-' || c.is_ascii_digit())
        .trim_end_matches('-');
    if out.is_empty() {
        return Err(invalid("heading needs an explicit alphabetic label"));
    }
    Ok(out.into())
}
fn plain_title(text: &str) -> Result<String> {
    if text.contains(['<', '>', '\\', '|']) || text.contains(":`") {
        return Err(invalid(
            "complex heading markup requires a plain heading before importing",
        ));
    }
    Ok(text.replace(['*', '`'], ""))
}
pub(super) fn adornment(line: &str) -> Option<char> {
    let line = line.trim_end();
    let ch = line.chars().next()?;
    (line.len() >= 2 && ch.is_ascii_punctuation() && line.chars().all(|c| c == ch)).then_some(ch)
}
fn indented(lines: &[&str], mut i: usize) -> (String, usize) {
    let start = i;
    while i < lines.len() && (lines[i].trim().is_empty() || lines[i].starts_with(' ')) {
        i += 1;
    }
    let chunk = &lines[start..i];
    let indent = chunk
        .iter()
        .filter(|l| !l.trim().is_empty())
        .map(|l| l.len() - l.trim_start_matches(' ').len())
        .min()
        .unwrap_or(0);
    let body = chunk
        .iter()
        .map(|l| {
            if l.trim().is_empty() {
                ""
            } else {
                &l[indent..]
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    (body.trim_matches('\n').into(), i)
}
fn options(body: &str, csv_header: bool) -> Result<(BTreeMap<String, String>, String)> {
    let mut options = BTreeMap::new();
    let lines: Vec<_> = body.lines().collect();
    let mut i = 0;
    while let Some(line) = lines.get(i) {
        let Some(rest) = line.strip_prefix(':') else {
            break;
        };
        let Some((key, value)) = rest.split_once(':') else {
            return Err(invalid("malformed directive option"));
        };
        // A role such as :download:`file` at the start of CSV data is not
        // an option field: its closing colon is not followed by whitespace.
        if !value.is_empty() && !value.starts_with(char::is_whitespace) {
            break;
        }
        let mut value = value.trim().to_owned();
        i += 1;
        if lines
            .get(i)
            .is_some_and(|line| line.starts_with(' ') && !line.trim().is_empty())
        {
            if !csv_header || key != "header" {
                return Err(invalid(
                    "multiline directive options require manual migration (except CSV :header:)",
                ));
            }
            // The first blank line ends the option block. Do not consume it
            // while collecting a multiline header, or CSV body data could
            // subsequently be mistaken for more options.
            let end = i + lines[i..]
                .iter()
                .take_while(|line| line.starts_with(' ') && !line.trim().is_empty())
                .count();
            let (continuation, _) = indented(&lines[i..end], 0);
            value.push('\n');
            value.push_str(&continuation);
            i = end;
        }
        if options.insert(key.into(), value).is_some() {
            return Err(invalid("duplicate directive option"));
        }
    }
    if !options.is_empty() && lines.get(i).is_some_and(|line| !line.trim().is_empty()) {
        return Err(invalid(
            "directive options need a blank line before content",
        ));
    }
    Ok((options, lines[i..].join("\n").trim_matches('\n').into()))
}
pub fn target(text: &str) -> Result<(Option<String>, String)> {
    if let Some((label, tail)) = text.rsplit_once(" <") {
        let value = tail
            .strip_suffix('>')
            .ok_or_else(|| invalid("unclosed explicit link target"))?;
        if label.trim().is_empty() || value.is_empty() {
            return Err(invalid("empty link label or target"));
        }
        Ok((Some(label.into()), value.into()))
    } else {
        Ok((None, text.into()))
    }
}
pub fn parse(raw: &str, language: &str) -> Result<Document> {
    if raw.contains('\t') {
        return Err(invalid(
            "tabs in reStructuredText are unsupported; expand them to spaces",
        ));
    }
    let raw = raw.trim_start_matches('\u{feff}');
    let mut doc = Document {
        title: String::new(),
        blocks: Vec::new(),
        labels: BTreeMap::new(),
        links: BTreeMap::new(),
        notes: Vec::new(),
        substitutions: BTreeMap::new(),
    };
    doc.blocks = blocks(raw, language, &mut doc, 0)?;
    if doc.title.is_empty() {
        return Err(invalid(
            "each imported .rst page must have a document title",
        ));
    }
    Ok(doc)
}
fn blocks(raw: &str, language: &str, doc: &mut Document, depth: usize) -> Result<Vec<Block>> {
    if depth > 16 {
        return Err(invalid("table/admonition/list nesting exceeds 16"));
    }
    let lines: Vec<_> = raw.lines().collect();
    let mut out = Vec::new();
    let mut i = 0;
    let mut styles = Vec::new();
    let mut used_ids = BTreeSet::new();
    let mut pending_label: Option<String> = None;
    while i < lines.len() {
        let line = lines[i].trim_end();
        if line.is_empty() {
            out.push(Block::Text(String::new()));
            i += 1;
            continue;
        }
        if super::tables::starts(line) {
            if pending_label.is_some() || i > 0 && !lines[i - 1].trim().is_empty() {
                return Err(invalid(
                    "tables need a preceding blank line and cannot use section labels",
                ));
            }
            let (table, count) = super::tables::parse(&lines[i..])?;
            out.push(table_block(table, None, language, doc, depth)?);
            i += count;
            continue;
        }
        let over = adornment(line).is_some()
            && i + 2 < lines.len()
            && adornment(lines[i + 2]) == adornment(line)
            && !lines[i + 1].trim().is_empty();
        let under =
            i + 1 < lines.len() && adornment(lines[i + 1]).is_some() && !line.starts_with(' ');
        if over || under {
            if depth > 0 {
                return Err(invalid(
                    "headings inside tables/admonitions/lists require manual migration",
                ));
            }
            let (text, marker, count) = if over {
                (lines[i + 1], adornment(line).unwrap(), 3)
            } else {
                (line, adornment(lines[i + 1]).unwrap(), 2)
            };
            let underline = lines[i + count - 1];
            if underline.chars().count() < text.chars().count() {
                return Err(invalid("heading underline is shorter than its title"));
            }
            let style = (marker, over);
            let level = if let Some(index) = styles.iter().position(|s| *s == style) {
                index + 1
            } else {
                styles.push(style);
                styles.len()
            };
            if level > 6 {
                return Err(invalid("heading nesting exceeds six levels"));
            }
            let plain = plain_title(text)?;
            let anchor = id(pending_label.as_deref().unwrap_or(&plain))?;
            if !used_ids.insert(anchor.clone()) {
                return Err(invalid(
                    "duplicate heading identifier; add distinct explicit labels",
                ));
            }
            if doc.title.is_empty() {
                doc.title = plain.clone();
            }
            if let Some(label) = pending_label.take() {
                if doc
                    .labels
                    .insert(name(&label), (anchor.clone(), plain.clone()))
                    .is_some()
                {
                    return Err(invalid("duplicate section label"));
                }
            }
            if doc
                .links
                .insert(name(&plain), format!("#{anchor}"))
                .is_some()
            {
                return Err(invalid(
                    "duplicate implicit heading reference; rename the section",
                ));
            }
            out.push(Block::Heading {
                level,
                text: text.into(),
                id: anchor,
            });
            i += count;
            continue;
        }
        if let Some(rest) = line.strip_prefix(".. _") {
            if depth > 0 {
                return Err(invalid(
                    "targets inside tables/admonitions require manual migration",
                ));
            }
            let (label, href) = rest
                .split_once(':')
                .ok_or_else(|| invalid("invalid explicit hyperlink target"))?;
            if label.is_empty() || label.contains('`') {
                return Err(invalid("complex explicit target requires manual migration"));
            }
            if href.trim().is_empty() {
                if pending_label.replace(label.into()).is_some() {
                    return Err(invalid(
                        "multiple labels on one section require manual migration",
                    ));
                }
            } else if doc.links.insert(name(label), href.trim().into()).is_some() {
                return Err(invalid("duplicate named hyperlink"));
            }
            i += 1;
            continue;
        }
        if pending_label.is_some() {
            return Err(invalid(
                "explicit labels are supported directly before section headings only",
            ));
        }
        if super::lists::starts(&lines[i..])? {
            if depth >= 16 {
                return Err(invalid("table/admonition/list nesting exceeds 16"));
            }
            if i > 0
                && !lines[i - 1].trim().is_empty()
                && !matches!(out.last(), Some(Block::List { .. }))
            {
                return Err(invalid("a list needs a preceding blank line"));
            }
            let (list, count) = super::lists::parse(&lines[i..])?;
            let items = list
                .items
                .iter()
                .map(|item| blocks(item, language, doc, depth + 1))
                .collect::<Result<Vec<_>>>()?;
            out.push(Block::List {
                numbering: list.numbering,
                items,
            });
            i += count;
            continue;
        }
        if let Some(rest) = line.strip_prefix(".. |") {
            let (key, rest) = rest
                .split_once('|')
                .ok_or_else(|| invalid("unclosed substitution definition"))?;
            let key = substitutions::name(key)?;
            if !rest.starts_with(char::is_whitespace) {
                return Err(invalid(
                    "substitution definition needs whitespace before its directive",
                ));
            }
            let (directive, arg) = rest
                .trim_start()
                .split_once("::")
                .ok_or_else(|| invalid("substitution definition needs an inline directive"))?;
            let has_options = lines.get(i + 1).is_some_and(|l| !l.trim().is_empty());
            let (body, next) = indented(&lines, i + 1);
            let value = match directive {
                "replace" => {
                    if !arg.trim().is_empty() && !has_options && !body.is_empty() {
                        return Err(invalid(
                            "replace accepts one paragraph; a blank line starts another paragraph",
                        ));
                    }
                    substitutions::replacement(arg.trim(), &body)?
                }
                "image" => {
                    let (opts, content) = if has_options {
                        options(&body, false)?
                    } else {
                        (BTreeMap::new(), body)
                    };
                    if arg.trim().is_empty()
                        || !content.is_empty()
                        || opts
                            .keys()
                            .any(|key| !matches!(key.as_str(), "alt" | "target"))
                        || opts.get("target").is_some_and(|value| value.is_empty())
                    {
                        return Err(invalid(
                            "image substitutions accept a source and optional alt/target only",
                        ));
                    }
                    Definition::Image {
                        source: arg.trim().into(),
                        alt: opts.get("alt").cloned().unwrap_or_else(|| key.clone()),
                        target: opts.get("target").cloned(),
                    }
                }
                _ => {
                    return Err(invalid(
                        "only replace/image substitution directives are supported; date, unicode, raw and extensions require manual migration",
                    ));
                }
            };
            substitutions::insert(&mut doc.substitutions, key, value)?;
            i = next;
            continue;
        }
        if let Some(rest) = line.strip_prefix(".. ") {
            let option_block = lines.get(i + 1).is_some_and(|line| !line.trim().is_empty());
            let (body, next) = indented(&lines, i + 1);
            i = next;
            if let Some((directive, arg)) = rest.split_once("::") {
                let arg = arg.trim();
                let (opts, body) = if option_block {
                    options(&body, directive == "csv-table")?
                } else {
                    (BTreeMap::new(), body)
                };
                match directive {
                    "table" | "list-table" | "csv-table" => {
                        if option_block && opts.is_empty() && !body.is_empty() {
                            return Err(invalid(
                                "table content needs a blank line after the directive; multiline captions require manual migration",
                            ));
                        }
                        let table = super::table_directives::parse(directive, &opts, &body)?;
                        out.push(table_block(
                            table,
                            (!arg.is_empty()).then(|| arg.into()),
                            language,
                            doc,
                            depth,
                        )?);
                    }
                    "toctree" => {
                        if depth > 0 || !arg.is_empty() {
                            return Err(invalid(
                                "toctree must be at document level without an argument",
                            ));
                        }
                        let mut toc = Toc::default();
                        for (key, value) in &opts {
                            match key.as_str() {
                                "caption" if !value.is_empty() => {
                                    if value.contains('|') { return Err(invalid("toctree captions require plain text; expand substitutions manually")); }
                                    toc.caption = Some(value.clone());
                                },
                                "hidden" if value.is_empty() => toc.hidden = true,
                                "titlesonly" | "includehidden" if value.is_empty() => {},
                                "maxdepth" if value.parse::<u32>().is_ok() => doc.notes.push("toctree maxdepth is presentation-only; the full document hierarchy is imported".into()),
                                _ => return Err(invalid(format!("unsupported toctree option :{key}: (globs, numbering and custom transforms require manual migration)"))),
                            }
                        }
                        for entry in body.lines().filter(|l| !l.is_empty()) {
                            if entry.starts_with(char::is_whitespace) {
                                return Err(invalid("toctree entries must have equal indentation"));
                            }
                            let entry = target(entry)?;
                            if entry.0.as_ref().is_some_and(|label| label.contains('|')) {
                                return Err(invalid(
                                    "toctree titles require plain text; expand substitutions manually",
                                ));
                            }
                            toc.entries.push(entry);
                        }
                        out.push(Block::Toc(toc));
                    }
                    "code-block" | "code" | "sourcecode" => {
                        if !opts.is_empty() {
                            return Err(invalid("code-block options require manual migration"));
                        }
                        out.push(Block::Code {
                            language: if arg.is_empty() { language } else { arg }.into(),
                            body,
                        });
                    }
                    "image" => {
                        if arg.is_empty() || !body.is_empty() || opts.keys().any(|k| k != "alt") {
                            return Err(invalid("image accepts a target and optional :alt: only"));
                        }
                        out.push(Block::Image {
                            target: arg.into(),
                            alt: opts.get("alt").cloned().unwrap_or_default(),
                        });
                    }
                    "note" | "warning" | "tip" | "important" | "caution" | "danger"
                    | "attention" | "error" | "hint" | "admonition" => {
                        if !opts.is_empty() {
                            return Err(invalid("admonition options require manual migration"));
                        }
                        let (title, body) = if directive == "admonition" {
                            (arg, body)
                        } else {
                            (directive, format!("{arg}\n\n{body}"))
                        };
                        let body = blocks(&body, language, doc, depth + 1)?;
                        out.push(Block::Note {
                            title: title.into(),
                            body,
                        });
                    }
                    _ => {
                        return Err(invalid(format!(
                            "unsupported reStructuredText directive {directive:?}; includes, raw HTML, API generation and extensions are not executed"
                        )));
                    }
                }
            } else {
                if rest.starts_with(['|', '[', '_']) {
                    return Err(invalid(
                        "substitutions, footnotes and anonymous targets require manual migration",
                    ));
                }
                doc.notes
                    .push("reStructuredText comments were omitted from rendered content".into());
            }
            continue;
        }
        if line == ".." {
            let (_, next) = indented(&lines, i + 1);
            i = next;
            continue;
        }
        if line.ends_with("::") {
            let (body, next) = indented(&lines, i + 1);
            if body.is_empty() || next == i + 1 {
                return Err(invalid(
                    "literal block marker must be followed by an indented block",
                ));
            }
            let prefix = line.strip_suffix("::").unwrap();
            if !prefix.is_empty() {
                out.push(Block::Text(if prefix.ends_with(char::is_whitespace) {
                    prefix.trim_end().into()
                } else {
                    format!("{prefix}:")
                }));
            }
            out.push(Block::Code {
                language: language.into(),
                body,
            });
            i = next;
            continue;
        }
        let trimmed = line.trim_start();
        let indent = line.len() - trimmed.len();
        if indent > 0 {
            return Err(invalid(
                "indented quote/definition blocks require manual migration",
            ));
        }
        if (trimmed.starts_with(':') && !trimmed.contains(":`"))
            || trimmed.starts_with(['+', '='])
            || trimmed.starts_with("| ")
            || trimmed == "|"
            || trimmed.starts_with(">>>")
            || adornment(trimmed).is_some()
        {
            return Err(invalid(
                "tables, fields, line blocks, transitions and doctests require manual migration",
            ));
        }
        out.push(Block::Text(line.into()));
        i += 1;
    }
    if pending_label.is_some() {
        return Err(invalid("label has no following heading"));
    }
    Ok(out)
}

fn table_block(
    table: super::tables::Table,
    caption: Option<String>,
    language: &str,
    doc: &mut Document,
    depth: usize,
) -> Result<Block> {
    if depth >= 16 {
        return Err(invalid("table/admonition/list nesting exceeds 16"));
    }
    let rows = table
        .rows
        .into_iter()
        .map(|row| {
            row.into_iter()
                .map(|cell| {
                    // RST's escaped-whitespace marker explicitly requests an empty cell.
                    if cell.trim() == "\\" && cell.ends_with(' ') {
                        Ok(Vec::new())
                    } else {
                        blocks(&cell, language, doc, depth + 1)
                    }
                })
                .collect::<Result<Vec<_>>>()
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(Block::Table {
        caption,
        headers: table.headers,
        stubs: table.stubs,
        rows,
    })
}

pub fn escaped(text: &str) -> String {
    let mut out = String::new();
    for c in text.chars() {
        if c == '<' {
            out.push_str("&lt;");
            continue;
        }
        if c == '>' {
            out.push_str("&gt;");
            continue;
        }
        if c == '&' {
            out.push_str("&amp;");
            continue;
        }
        if "\\[]`*_|".contains(c) {
            out.push('\\');
        }
        out.push(c);
    }
    out
}
pub fn link(label: &str, url: &str) -> Result<String> {
    Ok(format!("[{}](<{}>)", escaped(label), destination(url)?))
}
pub fn destination(url: &str) -> Result<String> {
    if url.contains(['<', '>', '\n', '\r', '\0']) {
        return Err(invalid("invalid link destination"));
    }
    Ok(url.replace(' ', "%20"))
}
fn literal(text: &str) -> String {
    let fence = "`".repeat(text.split(|c| c != '`').map(str::len).max().unwrap_or(0) + 1);
    format!("{fence} {text} {fence}")
}
/// Generated markup is never reparsed as reStructuredText.
pub fn inline(text: &str, context: &mut Expansion<'_>) -> Result<String> {
    let mut out = String::new();
    let mut rest = text;
    while !rest.is_empty() {
        if let Some(tail) = rest.strip_prefix('\\') {
            let ch = tail
                .chars()
                .next()
                .ok_or_else(|| invalid("trailing reStructuredText escape"))?;
            if !ch.is_whitespace() {
                out.push_str(&escaped(&ch.to_string()));
            }
            rest = &tail[ch.len_utf8()..];
            continue;
        }
        if let Some(tail) = rest.strip_prefix("``") {
            let (code, tail) = tail
                .split_once("``")
                .ok_or_else(|| invalid("unclosed inline literal"))?;
            out.push_str(&literal(code));
            rest = tail;
            continue;
        }
        if rest.starts_with(':')
            && let Some(end) = rest.find(":`")
        {
            let role = &rest[1..end];
            if !role.is_empty() && role.chars().all(|c| c.is_ascii_alphanumeric() || c == ':') {
                let (body, tail) = rest[end + 2..]
                    .split_once('`')
                    .ok_or_else(|| invalid("unclosed role"))?;
                match role {
                    "doc" | "ref" | "download" => out.push_str(&context.reference(role, body)?),
                    "code" | "literal" | "command" | "file" | "kbd" => out.push_str(&literal(body)),
                    "emphasis" => out.push_str(&format!("*{}*", escaped(body))),
                    "strong" => out.push_str(&format!("**{}**", escaped(body))),
                    _ => {
                        return Err(invalid(format!(
                            "unsupported Sphinx role {role:?}; migrate domain/extension references manually"
                        )));
                    }
                }
                rest = tail;
                continue;
            }
        }
        if let Some(tail) = rest.strip_prefix('`') {
            let (body, tail) = tail
                .split_once('`')
                .ok_or_else(|| invalid("unclosed interpreted text"))?;
            if let Some(tail) = tail.strip_prefix('_') {
                out.push_str(&context.reference("link", body)?);
                rest = tail.strip_prefix('_').unwrap_or(tail);
            } else {
                return Err(invalid(
                    "default or trailing-role interpreted text requires an explicit supported prefix role",
                ));
            }
            continue;
        }
        if let Some(tail) = rest.strip_prefix('|') {
            if let Some((key, tail)) = tail.split_once('|')
                && !key.is_empty()
                && key.trim() == key
            {
                let before = &text[..text.len() - rest.len()];
                if before
                    .chars()
                    .next_back()
                    .is_some_and(char::is_alphanumeric)
                    || tail.chars().next().is_some_and(char::is_alphanumeric)
                {
                    return Err(invalid(
                        "substitutions next to word characters need escaped whitespace",
                    ));
                }
                let expanded = context.expand(key)?;
                if tail.starts_with("__") {
                    return Err(invalid(
                        "anonymous substitution links require manual migration",
                    ));
                }
                if let Some(tail) = tail.strip_prefix('_') {
                    if tail
                        .chars()
                        .next()
                        .is_some_and(|ch| ch.is_alphanumeric() || ch == '_')
                    {
                        return Err(invalid(
                            "substitution links next to word characters need escaped whitespace",
                        ));
                    }
                    out.push_str(&substitutions::linked(
                        &expanded,
                        &context.reference("named-target", key)?,
                    )?);
                    rest = tail;
                } else {
                    out.push_str(&expanded);
                    rest = tail;
                }
                continue;
            }
            // An escaped opening pipe or an unpaired literal pipe is not a reference.
            out.push_str("\\|");
            rest = tail;
            continue;
        }
        let length = rest
            .find(|c: char| !c.is_alphanumeric() && !"-._".contains(c))
            .unwrap_or(rest.len());
        if length > 0 {
            let word = &rest[..length];
            let reference = word.trim_end_matches('.');
            if let Some(label) = reference.strip_suffix('_') {
                if label.ends_with('_') {
                    return Err(invalid(
                        "anonymous reference targets require inline explicit URLs",
                    ));
                }
                out.push_str(&context.reference("link", label)?);
                out.push_str(&word[reference.len()..]);
            } else {
                out.push_str(&word.replace('_', "\\_"));
            }
            rest = &rest[length..];
            continue;
        }
        let ch = rest.chars().next().unwrap();
        if matches!(ch, '[' | ']') {
            return Err(invalid(
                "citations and bracket markup require manual migration",
            ));
        }
        // Markdown-only syntax should remain literal in the destination.
        if matches!(ch, '#' | '~') {
            out.push('\\');
        }
        match ch {
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '&' => out.push_str("&amp;"),
            _ => out.push(ch),
        }
        rest = &rest[ch.len_utf8()..];
    }
    Ok(out)
}

pub fn render_document(
    doc: &Document,
    resolve: &impl Fn(&str, &str) -> Result<String>,
) -> Result<String> {
    let mut validation = Expansion::new(&doc.substitutions, resolve);
    for key in doc.substitutions.keys() {
        validation.expand(key)?;
    }
    render(
        &doc.blocks,
        &mut Expansion::new(&doc.substitutions, resolve),
    )
}
fn render(blocks: &[Block], context: &mut Expansion<'_>) -> Result<String> {
    let mut out = String::new();
    for block in blocks {
        match block {
            Block::Heading { level, text, id } => out.push_str(&format!(
                "\n{} {} {{#{id}}}\n\n",
                "#".repeat(*level),
                inline(text, context)?
            )),
            Block::Text(text) => {
                out.push_str(&inline(text, context)?);
                out.push('\n');
            }
            Block::Code { language, body } => {
                if !language
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || "_+-".contains(c))
                {
                    return Err(invalid("unsupported code language identifier"));
                }
                let fence = "`"
                    .repeat((body.split(|c| c != '`').map(str::len).max().unwrap_or(0) + 1).max(3));
                out.push_str(&format!("\n{fence}{language}\n{body}\n{fence}\n\n"));
            }
            Block::Image { target, alt } => out.push_str(&format!(
                "\n!{}\n\n",
                link(alt, &context.reference("asset", target)?)?
            )),
            Block::Note { title, body } => {
                out.push_str(&format!("\n> **{}**\n>\n", inline(title, context)?));
                for line in render(body, context)?.lines() {
                    out.push_str(&format!("> {line}\n"));
                }
                out.push('\n');
            }
            Block::List { numbering, items } => {
                let tag = if let Some(numbering) = numbering {
                    let kind = numbering
                        .style
                        .html_type()
                        .map(|c| format!(" type=\"{c}\""))
                        .unwrap_or_default();
                    out.push_str(&format!("\n<ol start=\"{}\"{kind}>\n", numbering.start));
                    "ol"
                } else {
                    out.push_str("\n<ul>\n");
                    "ul"
                };
                for item in items {
                    out.push_str(&format!("<li>\n\n{}\n</li>\n", render(item, context)?));
                }
                out.push_str(&format!("</{tag}>\n\n"));
            }
            Block::Toc(toc) if !toc.hidden => {
                if let Some(caption) = &toc.caption {
                    out.push_str(&format!("\n**{}**\n\n", escaped(caption)));
                }
                for (label, target) in &toc.entries {
                    let text = label
                        .as_ref()
                        .map(|label| format!("{label} <{target}>"))
                        .unwrap_or(target.clone());
                    out.push_str(&format!("- {}\n", context.reference("toc", &text)?));
                }
                out.push('\n');
            }
            Block::Toc(_) => {}
            Block::Table {
                caption,
                headers,
                stubs,
                rows,
            } => {
                out.push_str("\n<table>\n");
                if let Some(caption) = caption {
                    out.push_str(&format!(
                        "<caption>\n\n{}\n\n</caption>\n",
                        inline(caption, context)?
                    ));
                }
                if *headers > 0 {
                    out.push_str("<thead>\n");
                } else {
                    out.push_str("<tbody>\n");
                }
                for (index, row) in rows.iter().enumerate() {
                    if *headers > 0 && index == *headers {
                        out.push_str("</thead>\n<tbody>\n");
                    }
                    out.push_str("<tr>\n");
                    for (column, cell) in row.iter().enumerate() {
                        let (tag, scope) = if index < *headers {
                            ("th", "")
                        } else if column < *stubs {
                            ("th", " scope=\"row\"")
                        } else {
                            ("td", "")
                        };
                        out.push_str(&format!(
                            "<{tag}{scope}>\n\n{}\n</{tag}>\n",
                            render(cell, context)?
                        ));
                    }
                    out.push_str("</tr>\n");
                }
                out.push_str("</tbody>\n</table>\n\n");
            }
        }
    }
    Ok(out)
}
