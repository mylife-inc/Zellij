//! Semantic book statements resolved once, independently of the output adapter.
use crate::{Page, Publication, equations, links};
use pulldown_cmark::{Event, Tag, TagEnd};
use serde::Serialize;
use zellij_core::{Error, Result};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum StatementKind {
    Theorem,
    Lemma,
    Proposition,
    Corollary,
    Definition,
    Example,
    Remark,
    Proof,
}
impl StatementKind {
    pub fn name(self) -> &'static str {
        match self {
            Self::Theorem => "Theorem",
            Self::Lemma => "Lemma",
            Self::Proposition => "Proposition",
            Self::Corollary => "Corollary",
            Self::Definition => "Definition",
            Self::Example => "Example",
            Self::Remark => "Remark",
            Self::Proof => "Proof",
        }
    }
    pub(crate) fn proof(self) -> bool {
        self == Self::Proof
    }
}
#[derive(Clone, Debug, Serialize)]
pub struct Statement {
    pub id: String,
    pub kind: StatementKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub number: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    pub entry: String,
    pub source: String,
    pub anchor: String,
    /// Encoded portable path and fragment, relative to the publication root.
    pub href: String,
    #[serde(skip)]
    pub(crate) page: usize,
}
impl Statement {
    pub(crate) fn label(&self) -> String {
        self.number.as_ref().map_or_else(
            || self.kind.name().into(),
            |n| format!("{} {n}", self.kind.name()),
        )
    }
    pub(crate) fn heading(&self) -> String {
        self.title
            .as_ref()
            .map_or_else(|| self.label(), |t| format!("{} ({t})", self.label()))
    }
    pub(crate) fn class(&self) -> String {
        format!("zellij-statement-{}", self.kind.name().to_ascii_lowercase())
    }
}
pub(crate) struct Header {
    pub id: String,
    kind: StatementKind,
    title: Option<String>,
    pub consumed: usize,
}
fn invalid(message: impl std::fmt::Display) -> Error {
    Error::InvalidArgument(format!("book statements: {message}"))
}

/// Inspect events immediately AFTER an ordinary blockquote start. All adapters
/// consume the same header; offsets are not persisted across reference lowering.
pub(crate) fn header(events: &[Event<'_>]) -> Result<Option<Header>> {
    if !matches!(events.first(), Some(Event::Start(Tag::Paragraph))) {
        return Ok(None);
    }
    let mut text = String::new();
    let mut plain = true;
    let mut consumed = 1;
    for event in &events[1..] {
        consumed += 1;
        match event {
            Event::End(TagEnd::Paragraph) => break,
            Event::Text(s) => text.push_str(s),
            _ => plain = false,
        }
    }
    let Some(marker) = text.strip_prefix("[!") else {
        return Ok(None);
    };
    let Some((kind, rest)) = marker.split_once(']') else {
        return Ok(None);
    };
    let kind = match kind {
        "THEOREM" => StatementKind::Theorem,
        "LEMMA" => StatementKind::Lemma,
        "PROPOSITION" => StatementKind::Proposition,
        "COROLLARY" => StatementKind::Corollary,
        "DEFINITION" => StatementKind::Definition,
        "EXAMPLE" => StatementKind::Example,
        "REMARK" => StatementKind::Remark,
        "PROOF" => StatementKind::Proof,
        _ if rest.trim_start().starts_with("{#") => {
            return Err(invalid(
                "unknown statement kind; use THEOREM, LEMMA, PROPOSITION, COROLLARY, DEFINITION, EXAMPLE, REMARK or PROOF",
            ));
        }
        _ => return Ok(None),
    };
    if !plain || (!rest.is_empty() && !rest.starts_with(char::is_whitespace)) {
        return Err(invalid(
            "put [!KIND] {#id} and an optional plain-text title on one line, followed by a blank quoted line before the body",
        ));
    }
    let label = rest.trim_start().strip_prefix("{#").ok_or_else(|| {
        invalid("every statement, including a proof, needs a stable {#id} label after [!KIND]")
    })?;
    let (id, title) = label
        .split_once('}')
        .ok_or_else(|| invalid("unclosed {#id} label"))?;
    equations::identifier(id).map_err(|_| invalid("labels must start with an ASCII letter and contain at most 128 ASCII letters, digits, colons, dots, underscores or hyphens"))?;
    if !title.is_empty() && !title.starts_with(char::is_whitespace) {
        return Err(invalid(
            "separate the optional title from {#id} with a space",
        ));
    }
    let title = title.trim();
    if title.chars().any(char::is_control) {
        return Err(invalid("statement titles must be plain text on one line"));
    }
    Ok(Some(Header {
        id: id.into(),
        kind,
        title: (!title.is_empty()).then(|| title.into()),
        consumed,
    }))
}

pub(crate) fn resolve(book: &mut Publication, pages: &[Page]) -> Result<()> {
    let mut ids: std::collections::BTreeMap<String, String> = book
        .equations
        .iter()
        .map(|e| (e.id.clone(), e.source.clone()))
        .collect();
    for (entry, prefix) in equations::prefixes(book) {
        let page = &pages[entry.index];
        let result = (|| {
            let mut ordinal = 0;
            let mut depth = 0usize;
            let mut active: Option<(usize, bool)> = None;
            let mut events = page.markdown.events.iter();
            while let Some(event) = events.next() {
                match event {
                    Event::Start(Tag::BlockQuote(None)) => {
                        if let Some(h) = header(events.as_slice())? {
                            if depth != 0 {
                                return Err(invalid(
                                    "statement blocks must be top-level, outside lists, other quotes and footnotes",
                                ));
                            }
                            if let Some(first) = ids.insert(h.id.clone(), page.source.to_string()) {
                                return Err(invalid(format!(
                                    "duplicate label {:?}; first defined in {first}",
                                    h.id
                                )));
                            }
                            for _ in 0..h.consumed {
                                events.next();
                            }
                            let number = if h.kind.proof() {
                                None
                            } else {
                                ordinal += 1;
                                Some(format!("{prefix}.{ordinal}"))
                            };
                            let anchor = format!("_zel-stmt-{}", h.id);
                            book.statements.push(Statement {
                                href: format!(
                                    "{}#{}",
                                    links::encode(page.output.as_str()),
                                    links::encode(&anchor)
                                ),
                                id: h.id,
                                kind: h.kind,
                                number,
                                title: h.title,
                                entry: entry.id.clone(),
                                source: page.source.to_string(),
                                anchor,
                                page: entry.index,
                            });
                            active = Some((depth, false));
                        }
                        depth += 1;
                    }
                    Event::Start(Tag::Heading { .. } | Tag::FootnoteDefinition(_))
                        if active.is_some() =>
                    {
                        return Err(invalid(
                            "statement bodies cannot contain headings or footnote definitions; put definitions outside and reference them from the body",
                        ));
                    }
                    Event::Start(_) => depth += 1,
                    Event::End(_) => {
                        depth -= 1;
                        if let Some((start, content)) = active
                            && depth == start
                        {
                            if !content {
                                return Err(invalid(
                                    "a statement must contain a body after its blank quoted line",
                                ));
                            }
                            active = None;
                        }
                    }
                    Event::Text(s)
                    | Event::Code(s)
                    | Event::InlineMath(s)
                    | Event::DisplayMath(s)
                        if !s.trim().is_empty() =>
                    {
                        if let Some((_, content)) = &mut active {
                            *content = true;
                        }
                    }
                    Event::FootnoteReference(_) => {
                        if let Some((_, content)) = &mut active {
                            *content = true;
                        }
                    }
                    _ => {}
                }
            }
            Ok(())
        })();
        result.map_err(|error| zellij_core::files::unsafe_path(&page.source, error.to_string()))?;
    }
    Ok(())
}
impl Publication {
    pub(crate) fn statement(&self, id: &str) -> &Statement {
        self.statements
            .iter()
            .find(|s| s.id == id)
            .expect("resolved statement")
    }
}

/// Search the same resolved labels/reference text readers see, not authoring markers.
pub(crate) fn text(events: &[Event<'_>], book: &Publication) -> Result<String> {
    let mut visible = vec![];
    let mut events = events.iter();
    while let Some(event) = events.next() {
        if matches!(event, Event::Start(Tag::BlockQuote(None)))
            && let Some(h) = header(events.as_slice())?
        {
            for _ in 0..h.consumed {
                events.next();
            }
            visible.push(Event::Text(book.statement(&h.id).heading().into()));
            visible.push(Event::SoftBreak);
        } else if let Event::DisplayMath(tex) = event
            && let equations::Expression::Numbered { tex, .. } = equations::expression(tex, true)?
        {
            visible.push(Event::DisplayMath(tex.into()));
        } else {
            visible.push(event.clone());
        }
    }
    Ok(crate::markdown::plain(&visible))
}
