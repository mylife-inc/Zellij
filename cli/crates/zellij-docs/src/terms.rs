//! Shared glossary/index catalogs, occurrence identity and web/print backmatter.
use crate::{Page, Publication, config, links, markdown};
use camino::Utf8Path;
use pulldown_cmark::{Event, Tag, TagEnd};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
};
use zellij_core::{Error, Result, book::Manifest};

pub(crate) const USE_PREFIX: &str = "_zel-use-";
#[derive(Clone, Debug, Serialize)]
pub struct TermCollection {
    pub kind: &'static str,
    pub title: String,
    pub source: String,
    pub href: &'static str,
    pub terms: Vec<Term>,
    #[serde(skip)]
    pub(crate) original: Vec<u8>,
}
#[derive(Clone, Debug, Serialize)]
pub struct Term {
    pub id: String,
    pub term: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub definition: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent: Option<String>,
    pub sort: String,
    pub see_also: Vec<String>,
    pub anchor: String,
    pub href: String,
    pub occurrences: Vec<TermOccurrence>,
}
#[derive(Clone, Debug, Serialize)]
pub struct TermOccurrence {
    pub entry: String,
    pub source: String,
    pub label: String,
    pub anchor: String,
    pub href: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
struct Input {
    id: String,
    term: String,
    definition: Option<String>,
    sort: Option<String>,
    parent: Option<String>,
    #[serde(default)]
    see_also: Vec<String>,
}
fn invalid(s: impl std::fmt::Display) -> Error {
    Error::InvalidArgument(format!("book terms: {s}"))
}
fn plain(s: &str, limit: usize) -> Result<()> {
    if s.trim().is_empty() || s.len() > limit || s.chars().any(char::is_control) {
        return Err(invalid(format!(
            "expected nonempty plain text without control characters (at most {limit} bytes)"
        )));
    }
    Ok(())
}
fn identifier(s: &str) -> Result<()> {
    if s.is_empty()
        || s.len() > 128
        || !s.starts_with(|c: char| c.is_ascii_alphanumeric() || c == '_')
        || !s
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b":._-".contains(&b))
    {
        return Err(invalid(format!(
            "invalid term ID {s:?}; use 1–128 ASCII letters, digits, colon, dot, underscore or hyphen, starting with a letter, digit or underscore"
        )));
    }
    Ok(())
}
pub(crate) fn target(url: &str) -> Option<(&'static str, &str)> {
    url.strip_prefix("glossary:")
        .map(|s| ("glossary", s))
        .or_else(|| url.strip_prefix("index:").map(|s| ("index", s)))
}
pub(crate) fn prepare(
    manifest: Option<&Manifest>,
    root: &Utf8Path,
    pages: &mut Vec<Page>,
) -> Result<Vec<TermCollection>> {
    let Some(m) = manifest else { return Ok(vec![]) };
    let mut collections = vec![];
    for (kind, settings, href, title) in [
        ("glossary", &m.glossary, "glossary.html", "Glossary"),
        ("index", &m.index, "book-index.html", "Index"),
    ] {
        let Some(settings) = settings else { continue };
        if pages
            .iter()
            .any(|p| p.output.as_str().eq_ignore_ascii_case(href))
        {
            return Err(invalid(format!(
                "{href} is reserved for generated {kind}; rename the authored document"
            )));
        }
        let relative = Utf8Path::new(&settings.source);
        crate::portable_path(relative)?;
        if relative.as_str().split('/').any(crate::ignored_name) || relative.starts_with("_zellij")
        {
            return Err(invalid(
                "catalog source must be outside hidden/generated paths",
            ));
        }
        let path = config::regular(root, relative)?;
        if fs::metadata(&path)?.len() > 8 * 1024 * 1024 {
            return Err(invalid("catalog exceeds 8 MiB"));
        }
        let original = fs::read(path)?;
        let inputs: Vec<Input> = serde_json::from_slice(&original)
            .map_err(|e| invalid(format!("{}: {e}", settings.source)))?;
        if inputs.is_empty() || inputs.len() > 10_000 {
            return Err(invalid("catalog must contain 1–10000 terms"));
        }
        let mut terms = vec![];
        let mut ids = BTreeSet::new();
        for input in inputs {
            identifier(&input.id)?;
            plain(&input.term, 512)?;
            if !ids.insert(input.id.clone()) {
                return Err(invalid(format!("duplicate {kind} ID {:?}", input.id)));
            }
            if kind == "glossary" {
                plain(
                    input
                        .definition
                        .as_deref()
                        .ok_or_else(|| invalid("glossary terms require definition"))?,
                    16_384,
                )?;
                if input.parent.is_some() {
                    return Err(invalid("parent is only supported in index catalogs"));
                }
            } else if input.definition.is_some() {
                return Err(invalid(
                    "index terms do not take definition; use a glossary catalog",
                ));
            }
            let sort = input.sort.unwrap_or_else(|| input.term.clone());
            plain(&sort, 512)?;
            if input.see_also.len() > 32 {
                return Err(invalid("at most 32 see-also terms per entry"));
            }
            let mut see = BTreeSet::new();
            for id in &input.see_also {
                identifier(id)?;
                if id == &input.id || !see.insert(id) {
                    return Err(invalid("see-also must be distinct other term IDs"));
                }
            }
            let anchor = format!("_zel-{kind}-{}", input.id);
            terms.push(Term {
                id: input.id,
                term: input.term,
                definition: input.definition,
                parent: input.parent,
                sort,
                see_also: input.see_also,
                href: format!("{href}#{}", links::encode(&anchor)),
                anchor,
                occurrences: vec![],
            });
        }
        for term in &terms {
            for id in &term.see_also {
                if !ids.contains(id) {
                    return Err(invalid(format!(
                        "{}: undefined see-also term {id:?}",
                        term.id
                    )));
                }
            }
            if let Some(parent) = &term.parent {
                let p = terms
                    .iter()
                    .find(|t| &t.id == parent)
                    .ok_or_else(|| invalid(format!("{}: undefined parent {parent:?}", term.id)))?;
                if parent == &term.id || p.parent.is_some() {
                    return Err(invalid(
                        "index supports one subentry level; parent must be a different top-level term",
                    ));
                }
            }
        }
        let keys: BTreeMap<_, _> = terms
            .iter()
            .map(|t| (t.id.clone(), (t.sort.to_lowercase(), t.id.clone())))
            .collect();
        terms.sort_by_key(|t| {
            (
                keys[t.parent.as_ref().unwrap_or(&t.id)].clone(),
                t.parent.is_some(),
                keys[&t.id].clone(),
            )
        });
        let title = settings.title.as_ref().map_or(title, |t| &t.0).to_owned();
        plain(&title, 512)?;
        pages.push(Page {
            source: href.into(),
            output: href.into(),
            original: String::new(),
            markdown: markdown::parse("", &title, false, false)?,
        });
        collections.push(TermCollection {
            kind,
            title,
            source: settings.source.clone(),
            href,
            terms,
            original,
        });
    }
    Ok(collections)
}
// Visit footnote bodies at their first call site, regardless of definition
// placement. Occurrence anchors are still allocated in source order.
fn reading_links(events: &[Event<'_>]) -> Vec<usize> {
    let mut notes = BTreeMap::new();
    for (i, e) in events.iter().enumerate() {
        if let Event::Start(Tag::FootnoteDefinition(id)) = e {
            let end = i
                + 1
                + events[i + 1..]
                    .iter()
                    .position(|e| matches!(e, Event::End(TagEnd::FootnoteDefinition)))
                    .expect("balanced notes");
            notes.insert(id.as_ref(), (i + 1, end));
        }
    }
    fn walk(
        events: &[Event<'_>],
        range: (usize, usize),
        notes: &BTreeMap<&str, (usize, usize)>,
        visited: &mut BTreeSet<String>,
        out: &mut Vec<usize>,
    ) {
        let mut pending = vec![range];
        while let Some((mut i, end)) = pending.pop() {
            while i < end {
                match &events[i] {
                    Event::Start(Tag::FootnoteDefinition(id)) => i = notes[id.as_ref()].1,
                    Event::FootnoteReference(id) if visited.insert(id.to_string()) => {
                        if let Some(body) = notes.get(id.as_ref()) {
                            pending.push((i + 1, end));
                            pending.push(*body);
                            break;
                        }
                    }
                    Event::Start(Tag::Link { .. }) => out.push(i),
                    _ => {}
                }
                i += 1;
            }
        }
    }
    let mut out = vec![];
    let mut visited = BTreeSet::new();
    walk(events, (0, events.len()), &notes, &mut visited, &mut out);
    for e in events {
        if let Event::Start(Tag::FootnoteDefinition(id)) = e
            && visited.insert(id.to_string())
        {
            walk(events, notes[id.as_ref()], &notes, &mut visited, &mut out);
        }
    }
    out
}
pub(crate) fn resolve(book: &mut Publication, pages: &[Page]) -> Result<()> {
    let lookup: BTreeMap<_, _> = book
        .terms
        .iter()
        .enumerate()
        .flat_map(|(c, b)| {
            b.terms
                .iter()
                .enumerate()
                .map(move |(t, term)| ((b.kind, term.id.clone()), (c, t)))
        })
        .collect();
    let mut total = 0;
    for entry in &book.entries {
        let page = &pages[entry.index];
        let mut heading = false;
        let mut images = 0;
        let mut counts = BTreeMap::new();
        let mut pending = BTreeMap::new();
        for (i, event) in page.markdown.events.iter().enumerate() {
            match event {
                Event::Start(Tag::Heading { .. }) => heading = true,
                Event::End(TagEnd::Heading(_)) => heading = false,
                Event::Start(Tag::Image { dest_url, .. }) => {
                    images += 1;
                    if target(dest_url).is_some() {
                        return Err(invalid("term references cannot be images"));
                    }
                }
                Event::End(TagEnd::Image) => images -= 1,
                Event::Start(Tag::Link { dest_url, id, .. }) => {
                    if id.starts_with(USE_PREFIX) {
                        return Err(invalid(
                            "Markdown reference-link IDs beginning _zel-use- are reserved",
                        ));
                    }
                    let Some((kind, id)) = target(dest_url) else {
                        continue;
                    };
                    identifier(id)?;
                    if heading || images > 0 {
                        return Err(invalid(
                            "term references must be outside headings and image labels",
                        ));
                    }
                    let mut label = String::new();
                    for e in &page.markdown.events[i + 1..] {
                        match e {
                            Event::End(TagEnd::Link) => break,
                            Event::Text(s) | Event::Code(s) => label.push_str(s),
                            _ => {
                                return Err(invalid(
                                    "term link labels must be plain text or inline code on one line",
                                ));
                            }
                        }
                    }
                    plain(&label, 512)?;
                    let &(c, t) = lookup.get(&(kind, id.into())).ok_or_else(|| {
                        invalid(format!(
                            "{}: unknown {kind} term {id:?}; define it in book.{kind}.source",
                            page.source
                        ))
                    })?;
                    total += 1;
                    if total > 100_000 {
                        return Err(invalid("at most 100000 term occurrences per book"));
                    }
                    let n = counts.entry((kind, id)).or_insert(0);
                    *n += 1;
                    let anchor = format!("{USE_PREFIX}{kind}-{id}-{n}");
                    let label = entry
                        .number
                        .as_ref()
                        .map_or_else(|| entry.title.clone(), |n| format!("{n}. {}", entry.title));
                    pending.insert(
                        i,
                        (
                            c,
                            t,
                            TermOccurrence {
                                entry: entry.id.clone(),
                                source: page.source.to_string(),
                                label,
                                href: format!(
                                    "{}#{}",
                                    links::encode(page.output.as_str()),
                                    links::encode(&anchor)
                                ),
                                anchor,
                            },
                        ),
                    );
                }
                _ => {}
            }
        }
        for i in reading_links(&page.markdown.events) {
            if let Some((c, t, occurrence)) = pending.remove(&i) {
                book.terms[c].terms[t].occurrences.push(occurrence);
            }
        }
        debug_assert!(pending.is_empty());
    }
    Ok(())
}
pub(crate) fn references(
    mut events: Vec<Event<'static>>,
    book: &Publication,
    page: &Page,
) -> Vec<Event<'static>> {
    let lookup: BTreeMap<_, _> = book
        .terms
        .iter()
        .flat_map(|c| {
            c.terms
                .iter()
                .map(move |t| ((c.kind, t.id.as_str()), (c, t)))
        })
        .collect();
    let mut counts = BTreeMap::new();
    for event in &mut events {
        if let Event::Start(Tag::Link {
            dest_url,
            id,
            title,
            ..
        }) = event
            && let Some((kind, key)) = target(dest_url)
        {
            let (collection, term) = lookup.get(&(kind, key)).expect("validated term");
            let n = counts.entry((kind, key.to_owned())).or_insert(0);
            *n += 1;
            *id = format!("{USE_PREFIX}{kind}-{key}-{n}").into();
            *title = format!("{}: {}", collection.title, term.term).into();
            *dest_url = format!("{}{}", links::prefix(&page.output), term.href).into();
        }
    }
    events
}
pub(crate) fn collection_for_page<'a>(
    book: &'a Publication,
    page: &Page,
) -> Option<&'a TermCollection> {
    book.terms.iter().find(|c| page.output == c.href)
}
pub(crate) fn has_anchor(book: &Publication, page: &Page, id: &str) -> bool {
    book.terms.iter().any(|c| {
        c.terms.iter().any(|t| {
            (page.output == c.href && t.anchor == id)
                || t.occurrences
                    .iter()
                    .any(|o| o.source == page.source.as_str() && o.anchor == id)
        })
    })
}
pub(crate) fn search(c: &TermCollection) -> String {
    format!(
        "{} {}",
        c.title,
        c.terms
            .iter()
            .map(|t| format!("{} {}", t.term, t.definition.as_deref().unwrap_or("")))
            .collect::<Vec<_>>()
            .join(" ")
    )
}
fn node(tag: &str, props: Value, children: Vec<Value>) -> Value {
    json!({"type":"element","tagName":tag,"properties":props,"children":children})
}
fn text(s: impl Into<String>) -> Value {
    json!({"type":"text","value":s.into()})
}
fn web_href(href: &str, hosted: bool) -> String {
    if hosted {
        href.strip_prefix("index.html#")
            .map_or_else(|| format!("/{href}"), |fragment| format!("/#{fragment}"))
    } else {
        href.into()
    }
}
pub(crate) fn tree(c: &TermCollection, hosted: bool) -> Value {
    let lookup: BTreeMap<_, _> = c.terms.iter().map(|t| (t.id.as_str(), t)).collect();
    let entries = c.terms.iter().map(|t| {
        let mut children = vec![node("p", json!({"className":["zellij-term-name"]}), vec![node("strong", json!({}), vec![text(&t.term)])])];
        if let Some(definition) = &t.definition { children.push(node("p", json!({}), vec![text(definition)])); }
        if !t.see_also.is_empty() {
            let mut links = vec![text("See also: ")];
            for (i,id) in t.see_also.iter().enumerate() {
                if i>0 {links.push(text(", "));}
                let other=lookup[id.as_str()];
                links.push(node("a",json!({"href":format!("#{}",links::encode(&other.anchor))}),vec![text(&other.term)]));
            }
            children.push(node("p",json!({}),links));
        }
        let mut occurrences=vec![text(if t.occurrences.is_empty(){"No occurrences."}else{"Occurrences: "})];
        for (i,o) in t.occurrences.iter().enumerate() {
            if i>0 {occurrences.push(text("; "));}
            occurrences.push(node("a",json!({"href":web_href(&o.href,hosted)}),vec![text(format!("{} ({})",o.label,i+1))]));
        }
        children.push(node("p",json!({"className":["zellij-term-occurrences"]}),occurrences));
        node("li",json!({"id":t.anchor,"className":[if t.parent.is_some(){"zellij-term-child"}else{"zellij-term-entry"}]}),children)
    }).collect();
    node("ul", json!({"className":["zellij-terms"]}), entries)
}
pub(crate) fn html(c: &TermCollection) -> String {
    fn render(v: &Value) -> String {
        if v["type"] == "text" {
            return markdown::escape(v["value"].as_str().unwrap());
        }
        let tag = v["tagName"].as_str().unwrap();
        let mut attrs = String::new();
        for (key, value) in v["properties"].as_object().unwrap() {
            let (key, value) = if key == "className" {
                (
                    "class",
                    value
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|v| v.as_str().unwrap())
                        .collect::<Vec<_>>()
                        .join(" "),
                )
            } else {
                (key.as_str(), value.as_str().unwrap().to_owned())
            };
            attrs.push_str(&format!(" {key}=\"{}\"", markdown::escape(&value)));
        }
        format!(
            "<{tag}{attrs}>{}</{tag}>",
            v["children"]
                .as_array()
                .unwrap()
                .iter()
                .map(render)
                .collect::<String>()
        )
    }
    render(&tree(c, false))
}
pub(crate) fn latex(
    c: &TermCollection,
    page: usize,
    book: &Publication,
    label: fn(usize, &str) -> String,
    tex: fn(&str) -> String,
) -> String {
    let mut out = format!(
        "\\backmatter\n\\chapter{{{}}}\n\\hypertarget{{{}}}{{}}\n\\hypertarget{{{}}}{{}}\n",
        tex(&c.title),
        label(page, ""),
        label(page, "_zel-title")
    );
    // refcount is already required by hyperref. Deduplicate resolved folios per
    // entry, while retaining one link to the first occurrence on each page.
    out.push_str(r"\providecommand{\zeltermpage}[1]{\edef\zelfolio{\getpagerefnumber{#1}}\ifcsname zeltermseen\zelfolio\endcsname\else\expandafter\def\csname zeltermseen\zelfolio\endcsname{}\zeltermsep\hyperlink{#1}{\pageref*{#1}}\def\zeltermsep{, }\fi}");
    out.push('\n');
    for t in &c.terms {
        out.push_str(&format!(
            "\\zelheadingroom\\par\\begingroup{}\\hypertarget{{{}}}{{}}\\textbf{{{}}}\\par\n",
            if t.parent.is_some() {
                "\\leftskip=1.5em "
            } else {
                ""
            },
            label(page, &t.anchor),
            tex(&t.term)
        ));
        if let Some(def) = &t.definition {
            out.push_str(&format!("{}\\par\n", tex(def)));
        }
        if !t.see_also.is_empty() {
            out.push_str("See also: ");
            for (i, id) in t.see_also.iter().enumerate() {
                if i > 0 {
                    out.push_str(", ");
                }
                let other = c.terms.iter().find(|t| &t.id == id).unwrap();
                out.push_str(&format!(
                    "\\hyperlink{{{}}}{{{}}}",
                    label(page, &other.anchor),
                    tex(&other.term)
                ));
            }
            out.push_str("\\par\n");
        }
        if t.occurrences.is_empty() {
            out.push_str("No occurrences.\\par\n");
        } else {
            out.push_str("Pages: \\def\\zeltermsep{} ");
            for o in &t.occurrences {
                let entry = book.entries.iter().find(|e| e.id == o.entry).unwrap();
                out.push_str(&format!(
                    "\\zeltermpage{{{}}}",
                    label(entry.index, &o.anchor)
                ));
            }
            out.push_str("\\par\n");
        }
        out.push_str("\\endgroup\\addvspace{0.8\\baselineskip}\n");
    }
    out
}
