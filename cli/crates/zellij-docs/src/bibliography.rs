//! Resolve numeric book citations once, before any web/print adapter runs.
use crate::{Page, Publication, config, links, markdown};
use camino::Utf8Path;
use pulldown_cmark::{Event, LinkType, Tag, TagEnd};
use serde::Serialize;
use serde_json::{Value, json};
use std::{collections::BTreeSet, fs};
use zellij_core::{Error, Result, book::Manifest};

pub(crate) const SCHEME: &str = "zellij-cite:";
pub(crate) const OUTPUT: &str = "bibliography.html";
#[derive(Clone, Debug, Serialize)]
pub struct Bibliography {
    pub title: String,
    pub source: String,
    pub style: &'static str,
    pub href: &'static str,
    pub references: Vec<Reference>,
    #[serde(skip)]
    pub(crate) original: Vec<u8>,
}
#[derive(Clone, Debug, Serialize)]
pub struct Reference {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_id: Option<String>,
    pub number: usize,
    pub title: String,
    pub text: String,
    pub anchor: String,
    pub href: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub link_text: Option<String>,
}
fn invalid(message: impl std::fmt::Display) -> Error {
    Error::InvalidArgument(format!("book bibliography: {message}"))
}
fn string<'a>(value: &'a Value, key: &str) -> Result<Option<&'a str>> {
    let Some(value) = value.get(key) else {
        return Ok(None);
    };
    let s = value
        .as_str()
        .filter(|s| !s.trim().is_empty() && s.len() <= 16_384 && !s.chars().any(char::is_control))
        .ok_or_else(|| {
            invalid(format!(
                "{key} must be nonempty plain text without control characters (at most 16384 bytes)"
            ))
        })?;
    Ok(Some(s.trim()))
}
fn required<'a>(value: &'a Value, key: &str) -> Result<&'a str> {
    string(value, key)?.ok_or_else(|| invalid(format!("missing {key}")))
}
fn year(value: &Value) -> Result<String> {
    let Some(issued) = value.get("issued") else {
        return Ok("n.d.".into());
    };
    if !issued.is_object() {
        return Err(invalid("issued must be a CSL date object"));
    }
    if let Some(v) = string(issued, "literal")?.or(string(issued, "raw")?) {
        return Ok(v.into());
    }
    let parts = issued
        .get("date-parts")
        .and_then(Value::as_array)
        .filter(|p| (1..=2).contains(&p.len()))
        .ok_or_else(|| {
            invalid("issued needs literal, raw or date-parts containing one date or a date range")
        })?;
    let mut years = vec![];
    for date in parts {
        let date = date
            .as_array()
            .filter(|p| (1..=3).contains(&p.len()))
            .ok_or_else(|| invalid("each date-parts date needs a year and optional month/day"))?;
        let y = date[0]
            .as_i64()
            .or_else(|| date[0].as_str().and_then(|s| s.parse().ok()))
            .filter(|y| *y != 0 && (-9999..=9999).contains(y))
            .ok_or_else(|| {
                invalid("issued year must be a nonzero integer between -9999 and 9999")
            })?;
        // Numeric style displays years; still reject malformed month/day values.
        for (i, v) in date.iter().enumerate().skip(1) {
            let n = v
                .as_u64()
                .or_else(|| v.as_str().and_then(|s| s.parse().ok()));
            if !n.is_some_and(|n| (1..=if i == 1 { 12 } else { 31 }).contains(&n)) {
                return Err(invalid("invalid issued month/day"));
            }
        }
        let display = if y < 0 {
            format!("{} BCE", -y)
        } else {
            y.to_string()
        };
        if years.last() != Some(&display) {
            years.push(display);
        }
    }
    Ok(years.join("–"))
}
fn names(value: &Value) -> Result<Option<String>> {
    let Some(authors) = value.get("author") else {
        return Ok(None);
    };
    let authors = authors
        .as_array()
        .filter(|a| a.len() <= 64)
        .ok_or_else(|| invalid("author must be an array of at most 64 CSL name objects"))?;
    let mut names = vec![];
    for author in authors {
        if !author.is_object() {
            return Err(invalid("each author must be a CSL name object"));
        }
        if let Some(literal) = string(author, "literal")? {
            names.push(literal.to_owned());
            continue;
        }
        let family = required(author, "family")?;
        let mut pieces = vec![];
        for key in ["given", "dropping-particle", "non-dropping-particle"] {
            if let Some(v) = string(author, key)? {
                pieces.push(v);
            }
        }
        pieces.push(family);
        let mut name = pieces.join(" ");
        if let Some(suffix) = string(author, "suffix")? {
            name.push_str(&format!(", {suffix}"));
        }
        names.push(name);
    }
    Ok((!names.is_empty()).then(|| names.join(", ")))
}
fn reference(value: &Value) -> Result<Reference> {
    let source_id = match value.get("id") {
        Some(Value::Number(n)) if n.is_u64() => n.to_string(),
        _ => required(value, "id")?.to_owned(),
    };
    let id = string(value, "citation-key")?.unwrap_or(&source_id);
    identifier(id)?;
    required(value, "type")?; // Generic numeric presentation, not a CSL style engine.
    let title = required(value, "title")?.to_owned();
    let mut fields = vec![];
    if let Some(author) = names(value)? {
        fields.push(author);
    }
    fields.push(format!("({})", year(value)?));
    fields.push(title.clone());
    for key in [
        "container-title",
        "publisher",
        "publisher-place",
        "volume",
        "issue",
        "page",
    ] {
        let numeric = value
            .get(key)
            .filter(|v| v.is_number() && matches!(key, "volume" | "issue" | "page"))
            .map(Value::to_string);
        let field = if let Some(n) = &numeric {
            Some(n.as_str())
        } else {
            string(value, key)?
        };
        if let Some(v) = field {
            fields.push(match key {
                "volume" => format!("vol. {v}"),
                "issue" => format!("no. {v}"),
                "page" => format!("pp. {v}"),
                _ => v.into(),
            });
        }
    }
    let mut url = string(value, "URL")?
        .map(|s| {
            let parsed = url::Url::parse(s)
                .map_err(|_| invalid("URL must be an absolute HTTP(S) address"))?;
            if !matches!(parsed.scheme(), "https" | "http")
                || parsed.host_str().is_none()
                || !parsed.username().is_empty()
                || parsed.password().is_some()
                || s.contains('\\')
            {
                return Err(invalid(
                    "URL must be an absolute HTTP(S) address without credentials",
                ));
            }
            Ok(parsed.to_string())
        })
        .transpose()?;
    let mut link_text = url.as_ref().map(|_| "Online source".into());
    if let Some(doi) = string(value, "DOI")? {
        let (prefix, suffix) = doi.split_once('/').ok_or_else(|| {
            invalid("DOI must be an identifier such as 10.1234/example, without a URL prefix")
        })?;
        if !prefix
            .strip_prefix("10.")
            .is_some_and(|p| (4..=9).contains(&p.len()) && p.bytes().all(|b| b.is_ascii_digit()))
            || suffix.is_empty()
            || doi.chars().any(char::is_whitespace)
            || doi.contains('\\')
        {
            return Err(invalid(
                "DOI must be an identifier such as 10.1234/example, without a URL prefix",
            ));
        }
        let mut target = url::Url::parse("https://doi.org/").unwrap();
        target.set_path(doi);
        url = Some(target.to_string());
        link_text = Some(format!("DOI: {doi}"));
    }
    let anchor = format!("_zel-bib-{id}");
    Ok(Reference {
        id: id.into(),
        source_id: (source_id != id).then(|| source_id.clone()),
        number: 0,
        title,
        text: fields
            .into_iter()
            .map(|s| {
                if s.ends_with(['.', '!', '?']) {
                    s
                } else {
                    format!("{s}.")
                }
            })
            .collect::<Vec<_>>()
            .join(" "),
        href: format!("{OUTPUT}#{}", links::encode(&anchor)),
        anchor,
        url,
        link_text,
    })
}
pub(crate) fn prepare(
    manifest: Option<&Manifest>,
    root: &Utf8Path,
    pages: &mut Vec<Page>,
) -> Result<Option<Bibliography>> {
    let Some(settings) = manifest.and_then(|m| m.bibliography.as_ref()) else {
        return Ok(None);
    };
    if pages.iter().any(|p| {
        p.output.as_str().eq_ignore_ascii_case(OUTPUT)
            || p.source.as_str().eq_ignore_ascii_case("bibliography.md")
    }) {
        return Err(invalid(
            "bibliography.html is reserved for the generated bibliography; rename the authored document",
        ));
    }
    let relative = Utf8Path::new(&settings.source);
    crate::portable_path(relative)?;
    if relative.as_str().split('/').any(crate::ignored_name) || relative.starts_with("_zellij") {
        return Err(invalid("source must be outside hidden/generated paths"));
    }
    let path = config::regular(root, relative)?;
    if fs::metadata(&path)?.len() > 8 * 1024 * 1024 {
        return Err(invalid("catalog exceeds 8 MiB"));
    }
    let original = fs::read(path)?;
    let values: Value = serde_json::from_slice(&original)
        .map_err(|e| invalid(format!("{}: {e}", settings.source)))?;
    let values = values
        .as_array()
        .filter(|a| !a.is_empty() && a.len() <= 10000)
        .ok_or_else(|| invalid("CSL-JSON catalog must be an array of 1–10000 references"))?;
    let mut references = vec![];
    let mut ids = BTreeSet::new();
    let mut source_ids = BTreeSet::new();
    for (i, value) in values.iter().enumerate() {
        let r = reference(value)
            .map_err(|e| invalid(format!("{}: item {}: {e}", settings.source, i + 1)))?;
        if !ids.insert(r.id.clone()) {
            return Err(invalid(format!("duplicate reference ID {:?}", r.id)));
        }
        let source_id = r.source_id.as_ref().unwrap_or(&r.id);
        if !source_ids.insert(source_id.clone()) {
            return Err(invalid(format!("duplicate CSL source ID {source_id:?}")));
        }
        references.push(r);
    }
    let title = settings
        .title
        .as_ref()
        .map_or("Bibliography", |t| t.0.as_str())
        .to_owned();
    pages.push(Page {
        source: OUTPUT.into(),
        output: OUTPUT.into(),
        original: String::new(),
        markdown: markdown::parse("", &title, false, false)?,
    });
    Ok(Some(Bibliography {
        title,
        source: settings.source.clone(),
        style: "numeric",
        href: OUTPUT,
        references,
        original,
    }))
}
fn identifier(id: &str) -> Result<()> {
    if id.is_empty()
        || id.len() > 128
        || !id.starts_with(|c: char| c.is_ascii_alphanumeric() || c == '_')
        || !id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b":._-".contains(&b))
    {
        return Err(invalid(format!(
            "invalid citation ID {id:?}; use 1–128 ASCII letters, digits, colons, dots, underscores or hyphens, starting with a letter, digit or underscore; supply citation-key for other CSL IDs"
        )));
    }
    Ok(())
}
#[derive(Debug)]
struct Cite {
    id: String,
    locator: Option<String>,
}
fn cluster(raw: &str) -> Result<Vec<Cite>> {
    if raw.chars().any(char::is_control) {
        return Err(invalid("citation clusters must stay on one line"));
    }
    let mut result = vec![];
    for item in raw.split(';') {
        let item = item
            .trim()
            .strip_prefix('@')
            .ok_or_else(|| invalid("use [@id] or [@id, p. 42; @other]"))?;
        let (id, locator) = item
            .split_once(',')
            .map_or((item, None), |(a, b)| (a, Some(b.trim())));
        let id = id.trim();
        identifier(id)?;
        if locator.is_some_and(|s| s.is_empty() || s.len() > 256 || s.chars().any(char::is_control))
        {
            return Err(invalid(
                "citation locators must be 1–256 bytes of plain text on one line",
            ));
        }
        result.push(Cite {
            id: id.into(),
            locator: locator.map(str::to_owned),
        });
        if result.len() > 32 {
            return Err(invalid("at most 32 references per citation cluster"));
        }
    }
    Ok(result)
}
/// Keep code, escaped brackets and ordinary links as authored. Citation-looking
/// reference definitions are rejected so they cannot silently override citations.
pub(crate) fn validate_markup(events: &[Event<'_>]) -> Result<()> {
    let mut heading = false;
    for e in events {
        match e {
            Event::Start(Tag::Heading { .. }) => heading = true,
            Event::End(TagEnd::Heading(_)) => heading = false,
            Event::Start(Tag::Image { dest_url, id, .. })
                if dest_url.starts_with(SCHEME) || id.starts_with('@') =>
            {
                return Err(invalid("citations cannot be image references"));
            }
            Event::Start(Tag::Link {
                dest_url,
                link_type,
                id,
                ..
            }) => {
                if let Some(raw) = dest_url.strip_prefix(SCHEME) {
                    if *link_type != LinkType::ShortcutUnknown || heading {
                        return Err(invalid(
                            "citations use standalone [@id] markers outside headings and link labels",
                        ));
                    }
                    cluster(raw)?;
                } else if id.starts_with('@') {
                    return Err(invalid(
                        "@ citation IDs cannot be Markdown reference-link definitions; use a different link label",
                    ));
                }
            }
            _ => {}
        }
    }
    Ok(())
}
// A definition may appear before or after its call site. Scan it at first use,
// guard cycles/repeated calls, then retain uncited definitions for HTML parity.
fn reading_citations<'a>(events: &'a [Event<'_>]) -> Vec<&'a str> {
    let mut notes = std::collections::BTreeMap::new();
    let mut i = 0;
    while i < events.len() {
        if let Event::Start(Tag::FootnoteDefinition(id)) = &events[i] {
            let end = i
                + 1
                + events[i + 1..]
                    .iter()
                    .position(|e| matches!(e, Event::End(TagEnd::FootnoteDefinition)))
                    .expect("balanced footnote events");
            notes.insert(id.as_ref(), &events[i + 1..end]);
            i = end;
        }
        i += 1;
    }
    fn walk<'a>(
        events: &'a [Event<'_>],
        notes: &std::collections::BTreeMap<&str, &'a [Event<'_>]>,
        visited: &mut BTreeSet<String>,
        out: &mut Vec<&'a str>,
    ) {
        let mut pending = vec![events];
        while let Some(events) = pending.pop() {
            let mut i = 0;
            while i < events.len() {
                match &events[i] {
                    Event::Start(Tag::FootnoteDefinition(_)) => {
                        while i < events.len()
                            && !matches!(events[i], Event::End(TagEnd::FootnoteDefinition))
                        {
                            i += 1;
                        }
                    }
                    Event::FootnoteReference(id) if visited.insert(id.to_string()) => {
                        if let Some(body) = notes.get(id.as_ref()) {
                            pending.push(&events[i + 1..]);
                            pending.push(body);
                            break;
                        }
                    }
                    Event::Start(Tag::Link { dest_url, .. }) => {
                        if let Some(raw) = dest_url.strip_prefix(SCHEME) {
                            out.push(raw);
                        }
                    }
                    _ => {}
                }
                i += 1;
            }
        }
    }
    let mut out = vec![];
    let mut visited = BTreeSet::new();
    walk(events, &notes, &mut visited, &mut out);
    // Unreferenced footnotes are already rejected by the PDF adapter. Still
    // validate/lower them for HTML rather than leave unresolved private links.
    for e in events {
        if let Event::Start(Tag::FootnoteDefinition(id)) = e
            && visited.insert(id.to_string())
        {
            walk(notes[id.as_ref()], &notes, &mut visited, &mut out);
        }
    }
    out
}
pub(crate) fn resolve(book: &mut Publication, pages: &[Page]) -> Result<()> {
    let mut order = vec![];
    let mut seen = BTreeSet::new();
    for entry in &book.entries {
        for raw in reading_citations(&pages[entry.index].markdown.events) {
            let bib = book.bibliography.as_ref().ok_or_else(|| {
                invalid(format!(
                    "{}: citations require book.bibliography.source",
                    pages[entry.index].source
                ))
            })?;
            for cite in cluster(raw)? {
                if !bib.references.iter().any(|r| r.id == cite.id) {
                    return Err(invalid(format!(
                        "{}: undefined citation {:?} in {}",
                        pages[entry.index].source, cite.id, bib.source
                    )));
                }
                if seen.insert(cite.id.clone()) {
                    order.push(cite.id);
                }
            }
        }
    }
    if let Some(bib) = &mut book.bibliography {
        if book.manifest.bibliography.as_ref().unwrap().include_uncited {
            for r in &bib.references {
                if seen.insert(r.id.clone()) {
                    order.push(r.id.clone());
                }
            }
        }
        let all = std::mem::take(&mut bib.references);
        bib.references = order
            .into_iter()
            .enumerate()
            .map(|(i, id)| {
                let mut r = all.iter().find(|r| r.id == id).unwrap().clone();
                r.number = i + 1;
                r
            })
            .collect();
    }
    Ok(())
}
pub(crate) fn references(
    events: Vec<Event<'static>>,
    book: &Publication,
    page: &Page,
) -> Result<Vec<Event<'static>>> {
    let mut out = vec![];
    let mut events = events.into_iter();
    while let Some(event) = events.next() {
        if let Event::Start(Tag::Link { dest_url, .. }) = &event
            && let Some(raw) = dest_url.strip_prefix(SCHEME)
        {
            let bib = book.bibliography.as_ref().expect("validated citations");
            out.push(Event::Text("[".into()));
            for (i, cite) in cluster(raw)?.iter().enumerate() {
                if i > 0 {
                    out.push(Event::Text("; ".into()));
                }
                let r = bib
                    .references
                    .iter()
                    .find(|r| r.id == cite.id)
                    .expect("resolved citation");
                out.push(Event::Start(Tag::Link {
                    link_type: LinkType::Inline,
                    dest_url: format!("{}{}", links::prefix(&page.output), r.href).into(),
                    title: format!("Reference {}: {}", r.number, r.title).into(),
                    id: "".into(),
                }));
                out.push(Event::Text(r.number.to_string().into()));
                out.push(Event::End(TagEnd::Link));
                if let Some(locator) = &cite.locator {
                    out.push(Event::Text(format!(", {locator}").into()));
                }
            }
            out.push(Event::Text("]".into()));
            for event in events.by_ref() {
                if matches!(event, Event::End(TagEnd::Link)) {
                    break;
                }
            }
        } else {
            out.push(event);
        }
    }
    Ok(out)
}
pub(crate) fn search(book: &Publication) -> String {
    let bib = book.bibliography.as_ref().unwrap();
    format!(
        "{} {}",
        bib.title,
        bib.references
            .iter()
            .map(|r| format!(
                "[{}] {} {}",
                r.number,
                r.text,
                r.link_text.as_deref().unwrap_or("")
            ))
            .collect::<Vec<_>>()
            .join(" ")
    )
}
pub(crate) fn html(book: &Publication) -> String {
    let bib = book.bibliography.as_ref().unwrap();
    let escape = markdown::escape;
    if bib.references.is_empty() {
        return "<p>No references cited.</p>".into();
    }
    format!(
        "<ol class=\"zellij-bibliography\">{}</ol>",
        bib.references
            .iter()
            .map(|r| format!(
                "<li id=\"{}\"><p><strong>[{}]</strong> {}{}</p></li>",
                r.anchor,
                r.number,
                escape(&r.text),
                r.url.as_ref().map_or(String::new(), |u| format!(
                    " <a href=\"{}\">{}</a>",
                    escape(u),
                    escape(r.link_text.as_deref().unwrap())
                ))
            ))
            .collect::<String>()
    )
}
pub(crate) fn tree(book: &Publication) -> Value {
    fn text(s: impl Into<String>) -> Value {
        json!({"type":"text","value":s.into()})
    }
    fn node(tag: &str, props: Value, children: Vec<Value>) -> Value {
        json!({"type":"element","tagName":tag,"properties":props,"children":children})
    }
    let bib = book.bibliography.as_ref().unwrap();
    if bib.references.is_empty() {
        return node("p", json!({}), vec![text("No references cited.")]);
    }
    node(
        "ol",
        json!({"className":["zellij-bibliography"]}),
        bib.references
            .iter()
            .map(|r| {
                let mut children = vec![
                    node("strong", json!({}), vec![text(format!("[{}]", r.number))]),
                    text(format!(" {}", r.text)),
                ];
                if let Some(url) = &r.url {
                    children.push(text(" "));
                    children.push(node(
                        "a",
                        json!({"href":url}),
                        vec![text(r.link_text.as_ref().unwrap())],
                    ));
                }
                node(
                    "li",
                    json!({"id":r.anchor}),
                    vec![node("p", json!({}), children)],
                )
            })
            .collect(),
    )
}
