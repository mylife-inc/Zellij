//! Preserve source-authored heading IDs without making them MDX expressions.

use pulldown_cmark::{Event, HeadingLevel, Options, Parser, Tag, TagEnd};
use regex::Regex;
use std::{collections::BTreeSet, ops::Range, sync::LazyLock};
use zellij_core::{Error, Result};

use crate::mdx::make_mdx_safe_in_context;

struct Heading {
    span: Range<usize>,
    content: Range<usize>,
    level: HeadingLevel,
    id: Option<String>,
    text: String,
    top_level: bool,
}

fn parse(source: &str) -> Vec<Heading> {
    let mut headings = Vec::new();
    let mut current: Option<Heading> = None;
    let mut depth = 0;
    for (event, span) in Parser::new_ext(
        source,
        Options::ENABLE_HEADING_ATTRIBUTES
            | Options::ENABLE_STRIKETHROUGH
            | Options::ENABLE_TABLES
            | Options::ENABLE_FOOTNOTES,
    )
    .into_offset_iter()
    {
        match event {
            Event::Start(Tag::Heading { level, id, .. }) => {
                current = Some(Heading {
                    content: span.end..span.start,
                    span,
                    level,
                    id: id.map(|id| id.to_string()),
                    text: String::new(),
                    top_level: depth == 0,
                });
                depth += 1;
            }
            Event::End(TagEnd::Heading(_)) => {
                let mut heading = current.take().expect("balanced Markdown heading");
                // Docusaurus also accepts an end-of-heading MDX/HTML comment.
                // Inspect only parser-recognized heading content, never code
                // examples or ordinary comments elsewhere in the document.
                static COMMENT: LazyLock<Regex> = LazyLock::new(|| {
                    Regex::new(r"\s+(?:\{/\*\s*#([^\s*<>]+)\s*\*/\}|<!--\s*#([^\s<>]+)\s*-->)\s*$")
                        .unwrap()
                });
                if heading.id.is_none()
                    && let Some(content) = source.get(heading.content.clone())
                    && let Some(captures) = COMMENT.captures(content)
                {
                    heading.id = Some(
                        captures
                            .get(1)
                            .or_else(|| captures.get(2))
                            .unwrap()
                            .as_str()
                            .into(),
                    );
                    heading.content.end = heading.content.start + captures.get(0).unwrap().start();
                    heading.text.clear();
                    for event in Parser::new(&source[heading.content.clone()]) {
                        match event {
                            Event::Text(s) | Event::Code(s) => heading.text.push_str(&s),
                            Event::SoftBreak | Event::HardBreak => heading.text.push(' '),
                            _ => {}
                        }
                    }
                }
                headings.push(heading);
                depth -= 1;
            }
            event => {
                if let Some(heading) = &mut current {
                    heading.content.start = heading.content.start.min(span.start);
                    heading.content.end = heading.content.end.max(span.end);
                    match &event {
                        Event::Text(text) | Event::Code(text) => heading.text.push_str(text),
                        Event::SoftBreak | Event::HardBreak => heading.text.push(' '),
                        _ => {}
                    }
                }
                match event {
                    Event::Start(_) => depth += 1,
                    Event::End(_) => depth -= 1,
                    _ => {}
                }
            }
        }
    }
    headings
}

pub(crate) fn first_title(source: &str) -> Option<String> {
    parse(source)
        .into_iter()
        .find(|h| h.top_level && h.level == HeadingLevel::H1)
        .map(|h| h.text)
        .filter(|title| !title.trim().is_empty())
}

/// Convert top-level explicitly identified headings to ordinary literal MDX
/// headings. A leading H1 already supplies the page title; retain just its ID
/// in the body so the layout still renders one title.
pub(crate) fn render(source: &str) -> Result<String> {
    let mut out = String::new();
    let mut position = 0;
    let mut used = BTreeSet::new();
    for heading in parse(source) {
        let Some(id) = &heading.id else { continue };
        if !heading.top_level {
            return Err(Error::InvalidArgument(format!(
                "heading ID {id:?} is inside a list, blockquote or footnote; move the heading to the document level before importing"
            )));
        }
        if id.is_empty()
            || !id
                .chars()
                .all(|c| c.is_alphanumeric() || "_.:-".contains(c))
        {
            return Err(Error::InvalidArgument(format!(
                "invalid heading ID {id:?}; use letters, numbers, underscores, hyphens, dots or colons"
            )));
        }
        if !used.insert(id.clone()) {
            return Err(Error::InvalidArgument(format!(
                "duplicate heading ID {id:?}"
            )));
        }
        out.push_str(&make_mdx_safe_in_context(
            &source[position..heading.span.start],
            source,
        ));
        let leading_title =
            heading.level == HeadingLevel::H1 && source[..heading.span.start].trim().is_empty();
        if leading_title {
            out.push_str(&format!("<div id=\"{id}\" />\n"));
        } else {
            let content = source.get(heading.content.clone()).unwrap_or("");
            let content = make_mdx_safe_in_context(content, source)
                .trim_end_matches('\n')
                .replace('\n', " ");
            let level = heading.level as u8;
            out.push_str(&format!("<h{level} id=\"{id}\">{content}</h{level}>\n"));
        }
        position = heading.span.end;
    }
    out.push_str(&make_mdx_safe_in_context(&source[position..], source));
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preserves_explicit_heading_ids_and_one_title() {
        let source = "# Welcome {#intro}\n\n## Install **now** and `zel` {#setup}\n\nText.\n";
        assert_eq!(first_title(source).as_deref(), Some("Welcome"));
        assert_eq!(
            render(source).unwrap(),
            "<div id=\"intro\" />\n\n<h2 id=\"setup\">Install **now** and `zel`</h2>\n\nText.\n"
        );
    }

    #[test]
    fn handles_setext_closing_hashes_and_unicode() {
        let source = "# Title\n\n## Café ## {#café}\n\nSetext {#setext}\n------\n";
        let out = render(source).unwrap();
        assert!(out.contains("<h2 id=\"café\">Café</h2>"), "{out}");
        assert!(out.contains("<h2 id=\"setext\">Setext</h2>"), "{out}");
    }

    #[test]
    fn preserves_code_examples_and_escapes_expressions() {
        let source = "````md\n```\n# Example {#example}\n````\n\n## Use {timeout} {#timeout}\n";
        let out = render(source).unwrap();
        assert!(out.contains("# Example {#example}"), "{out}");
        assert!(
            out.contains("<h2 id=\"timeout\">Use \\{timeout\\}</h2>"),
            "{out}"
        );
    }

    #[test]
    fn rejects_duplicate_unsafe_and_nested_ids() {
        for source in [
            "## One {#same}\n\n## Two {#same}\n",
            "## Unsafe {#bad\"id}\n",
            "> ## Nested {#nested}\n",
        ] {
            assert!(render(source).is_err(), "{source}");
        }
    }

    #[test]
    fn title_ignores_examples_and_keeps_inline_markup_as_plain_text() {
        assert_eq!(
            first_title("```md\n# Example {#fake}\n```\n\n# Real **title** {#real}\n").as_deref(),
            Some("Real title")
        );
        assert_eq!(first_title("# {#empty}\n"), None);
    }

    #[test]
    fn preserves_docusaurus_comment_ids_without_evaluating_mdx() {
        let source = "# Welcome {/* #welcome */}\n\n## Install <!-- #install -->\n\n## Literal `{/* #code */}`\n\n## Expression {run()}\n";
        let out = render(source).unwrap();
        assert_eq!(first_title(source).as_deref(), Some("Welcome"));
        assert!(out.contains("<div id=\"welcome\" />"), "{out}");
        assert!(out.contains("<h2 id=\"install\">Install</h2>"), "{out}");
        assert!(out.contains("## Literal `{/* #code */}`"), "{out}");
        assert!(out.contains("## Expression \\{run()\\}"), "{out}");
    }

    #[test]
    fn balanced_markup_can_span_a_replaced_heading() {
        let source = "<Callout>\n\n## In the callout {#callout}\n\nText.\n\n</Callout>\n";
        let out = render(source).unwrap();
        assert!(out.starts_with("<Callout>\n"), "{out}");
        assert!(
            out.contains("<h2 id=\"callout\">In the callout</h2>"),
            "{out}"
        );
        assert!(out.ends_with("</Callout>\n"), "{out}");
    }
}
