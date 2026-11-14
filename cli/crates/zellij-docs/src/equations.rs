//! Book-wide equation identity and numbering, resolved before any renderer runs.
use crate::{Page, Publication, links};
use pulldown_cmark::{Event, LinkType, Tag, TagEnd};
use serde::Serialize;
use zellij_core::{Error, Result};

#[derive(Clone, Debug, Serialize)]
pub struct Equation {
    pub id: String,
    pub number: String,
    pub entry: String,
    pub source: String,
    pub anchor: String,
    /// Encoded portable path and fragment, relative to the publication root.
    pub href: String,
    #[serde(skip)]
    pub(crate) page: usize,
}

pub(crate) enum Expression {
    Plain,
    Numbered { id: String, tex: String },
    Reference { id: String, style: ReferenceStyle },
}
fn invalid(message: impl std::fmt::Display) -> Error {
    Error::InvalidArgument(format!("book equations: {message}"))
}
fn at(page: &Page, error: Error) -> Error {
    zellij_core::files::unsafe_path(&page.source, error.to_string())
}
pub(crate) fn identifier(id: &str) -> Result<()> {
    if id.len() > 128
        || !id.starts_with(|c: char| c.is_ascii_alphabetic())
        || !id
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || b":._-".contains(&c))
    {
        return Err(invalid(
            "labels must start with an ASCII letter and contain at most 128 ASCII letters, digits, colons, dots, underscores or hyphens",
        ));
    }
    Ok(())
}
fn argument(source: &str, after: usize) -> Result<(&str, usize)> {
    let rest = source[after..].trim_start();
    let Some(body) = rest.strip_prefix('{') else {
        return Err(invalid(
            "expected a literal braced label or environment name",
        ));
    };
    let end = body
        .find('}')
        .ok_or_else(|| invalid("unclosed braced argument"))?;
    Ok((&body[..end], source.len() - body.len() + end + 1))
}

/// Recognize semantic commands without interpreting comments, escaped slashes,
/// or commands with a longer name. The math renderer still validates the TeX.
pub(crate) fn expression(source: &str, block: bool) -> Result<Expression> {
    let mut i = 0;
    let mut depth = 0usize;
    let mut environments = 0usize;
    let mut label = None;
    while i < source.len() {
        let c = source[i..].chars().next().unwrap();
        if c == '%' {
            i += source[i..].find('\n').unwrap_or(source.len() - i);
            continue;
        }
        if c == '{' {
            depth += 1;
        }
        if c == '}' {
            depth = depth.saturating_sub(1);
        }
        if c != '\\' {
            i += c.len_utf8();
            continue;
        }
        let start = i;
        i += 1;
        let name_start = i;
        while i < source.len() && source.as_bytes()[i].is_ascii_alphabetic() {
            i += 1;
        }
        if i == name_start {
            // A TeX control symbol, including \\, \%, \{ and \}.
            if i < source.len() {
                i += source[i..].chars().next().unwrap().len_utf8();
            }
            continue;
        }
        match &source[name_start..i] {
            "label" => {
                if !block || depth != 0 || environments != 0 || label.is_some() {
                    return Err(invalid(
                        "use exactly one top-level \\label{id} in each numbered display expression",
                    ));
                }
                let (id, end) = argument(source, i)?;
                identifier(id)?;
                label = Some((id.to_owned(), start..end));
                i = end;
            }
            "ref" | "eqref" | "autoref" => {
                let style = match &source[name_start..i] {
                    "eqref" => ReferenceStyle::Equation,
                    "autoref" => ReferenceStyle::Auto,
                    _ => ReferenceStyle::Number,
                };
                let (id, end) = argument(source, i)?;
                identifier(id)?;
                if block || !source[..start].trim().is_empty() || !source[end..].trim().is_empty() {
                    return Err(invalid(
                        "put each reference in its own inline expression: $\\eqref{id}$, $\\ref{id}$ or $\\autoref{id}$",
                    ));
                }
                return Ok(Expression::Reference {
                    id: id.into(),
                    style,
                });
            }
            "tag" | "notag" | "nonumber" => {
                return Err(invalid(
                    "equation numbers come from the book; use \\label{id} instead of manual tags or numbering overrides",
                ));
            }
            "begin" => {
                let (environment, end) = argument(source, i)?;
                if [
                    "equation", "align", "alignat", "flalign", "gather", "multline", "eqnarray",
                ]
                .contains(&environment.trim_end_matches('*'))
                {
                    return Err(invalid(
                        "use display math with a top-level \\label{id}; use aligned/gathered for multiple lines, not a separate numbering environment",
                    ));
                }
                environments += 1;
                i = end;
            }
            "end" => {
                let (_, end) = argument(source, i)?;
                environments = environments.saturating_sub(1);
                i = end;
            }
            _ => {}
        }
    }
    if let Some((id, range)) = label {
        let mut tex = source.to_owned();
        // Preserve the boundary between a preceding control word and the next
        // letter: removing a label from \alpha\label{id}x must not form \alphax.
        tex.replace_range(range, " ");
        if !tex
            .lines()
            .any(|line| !line.trim().is_empty() && !line.trim_start().starts_with('%'))
        {
            return Err(invalid("a labelled equation must contain math"));
        }
        Ok(Expression::Numbered { id, tex })
    } else {
        Ok(Expression::Plain)
    }
}

pub(crate) fn resolve(book: &mut Publication, pages: &[Page]) -> Result<()> {
    let mut ids = std::collections::BTreeMap::<String, String>::new();
    for (entry, prefix) in prefixes(book) {
        let page = &pages[entry.index];
        let mut ordinal = 0;
        let mut restricted = 0;
        for event in &page.markdown.events {
            match event {
                Event::Start(Tag::Heading { .. } | Tag::Link { .. }) => restricted += 1,
                Event::End(TagEnd::Heading(_) | TagEnd::Link) => restricted -= 1,
                Event::InlineMath(tex) | Event::DisplayMath(tex) => {
                    let parsed = expression(tex, matches!(event, Event::DisplayMath(_)))
                        .map_err(|e| at(page, e))?;
                    if restricted > 0 && !matches!(parsed, Expression::Plain) {
                        return Err(at(
                            page,
                            invalid(
                                "labelled equations and book references belong in prose, not headings or links",
                            ),
                        ));
                    }
                    match parsed {
                        Expression::Numbered { id, .. } => {
                            if let Some(first) = ids.insert(id.clone(), page.source.to_string()) {
                                return Err(at(
                                    page,
                                    invalid(format!(
                                        "duplicate label {id:?}; first defined in {first}"
                                    )),
                                ));
                            }
                            ordinal += 1;
                            let anchor = format!("_zel-eq-{id}");
                            book.equations.push(Equation {
                                href: format!(
                                    "{}#{}",
                                    links::encode(page.output.as_str()),
                                    links::encode(&anchor)
                                ),
                                id,
                                number: format!("{prefix}.{ordinal}"),
                                entry: entry.id.clone(),
                                source: page.source.to_string(),
                                anchor,
                                page: entry.index,
                            });
                        }
                        Expression::Reference { .. } => {}
                        Expression::Plain => {}
                    }
                }
                _ => {}
            }
        }
    }
    Ok(())
}

impl Publication {
    pub(crate) fn equation(&self, id: &str) -> &Equation {
        self.equations
            .iter()
            .find(|e| e.id == id)
            .expect("resolved equation")
    }
}

/// References become ordinary validated links for all three output adapters.
/// Run after authored-link validation; the registry already validates targets.
pub(crate) fn references(
    events: Vec<Event<'static>>,
    book: &Publication,
    page: &Page,
    pages: &[Page],
) -> Result<Vec<Event<'static>>> {
    let mut result = vec![];
    for event in events {
        if let Event::InlineMath(tex) = &event
            && let Expression::Reference { id, style } = expression(tex, false)?
        {
            let reference = target(book, &id, style)?;
            let dest = format!(
                "{}#{}",
                links::href(&page.output, &pages[reference.page].output),
                links::encode(reference.anchor)
            );
            result.push(Event::Start(Tag::Link {
                link_type: LinkType::Inline,
                dest_url: dest.into(),
                title: reference.title.into(),
                id: "".into(),
            }));
            result.push(Event::Text(reference.text.into()));
            result.push(Event::End(TagEnd::Link));
        } else {
            result.push(event);
        }
    }
    Ok(result)
}

// Prefixes are shared by independent equation and statement counters.
pub(crate) fn prefixes(book: &Publication) -> Vec<(crate::ReadingEntry, String)> {
    let mut front = 0;
    book.entries
        .iter()
        .map(|entry| {
            let prefix = match &entry.number {
                Some(n) => n.clone(),
                None if entry.kind == "title-page" => "T".into(),
                None => {
                    front += 1;
                    format!("F{front}")
                }
            };
            (entry.clone(), prefix)
        })
        .collect()
}
#[derive(Clone, Copy)]
pub(crate) enum ReferenceStyle {
    Number,
    Equation,
    Auto,
}
struct Reference<'a> {
    page: usize,
    anchor: &'a str,
    title: String,
    text: String,
}
fn target<'a>(book: &'a Publication, id: &str, style: ReferenceStyle) -> Result<Reference<'a>> {
    if let Some(eq) = book.equations.iter().find(|eq| eq.id == id) {
        let title = format!("Equation {}", eq.number);
        let text = match style {
            ReferenceStyle::Number => eq.number.clone(),
            ReferenceStyle::Equation => format!("({})", eq.number),
            ReferenceStyle::Auto => title.clone(),
        };
        return Ok(Reference {
            page: eq.page,
            anchor: &eq.anchor,
            title,
            text,
        });
    }
    if let Some(s) = book.statements.iter().find(|s| s.id == id) {
        let text = match style {
            ReferenceStyle::Equation => {
                return Err(invalid(format!(
                    "{id:?} is a statement; use \\ref or \\autoref instead of \\eqref"
                )));
            }
            ReferenceStyle::Number => s.number.clone().ok_or_else(|| {
                invalid(format!(
                    "{id:?} is an unnumbered proof; use \\autoref instead of \\ref"
                ))
            })?,
            ReferenceStyle::Auto => s.label(),
        };
        return Ok(Reference {
            page: s.page,
            anchor: &s.anchor,
            title: s.heading(),
            text,
        });
    }
    Err(invalid(format!(
        "undefined {}label {id:?}",
        if matches!(style, ReferenceStyle::Equation) {
            "equation "
        } else {
            "book "
        }
    )))
}
pub(crate) fn validate_references(book: &Publication, pages: &[Page]) -> Result<()> {
    for page in pages {
        for event in &page.markdown.events {
            if let Event::InlineMath(tex) = event
                && let Expression::Reference { id, style } =
                    expression(tex, false).map_err(|e| at(page, e))?
            {
                target(book, &id, style).map_err(|e| at(page, e))?;
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn semantic_scanning_respects_comments_escapes_and_command_boundaries() {
        assert!(matches!(
            expression("x % \\label{ignored}\n", true).unwrap(),
            Expression::Plain
        ));
        assert!(matches!(
            expression(r"\\label{x}", true).unwrap(),
            Expression::Plain
        ));
        assert!(matches!(
            expression(r"\labelled{x}", true).unwrap(),
            Expression::Plain
        ));
        assert!(
            matches!(expression(r"x\%\label{real}",true).unwrap(),Expression::Numbered{id,..} if id=="real")
        );
        assert!(
            matches!(expression(r"\text{\{x\}}\label{real}",true).unwrap(),Expression::Numbered{id,..} if id=="real")
        );
        assert!(expression(r"x\label{unfinished", true).is_err());
        assert!(expression(r"x\label{inject\input{file}}", true).is_err());
    }
}
