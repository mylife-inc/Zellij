//! Material for MkDocs syntax, rewritten into plain Markdown.
//!
//! MkDocs Material adds constructs that are not Markdown and that MDX has no
//! idea about: admonitions (`!!! note`), content tabs (`=== "Title"`) and
//! attribute lists (`[text](url){ .md-button }`). A converter that leaves them
//! alone produces pages that build and are wrong — the marker prints as literal
//! text in the middle of a sentence, and a reader sees `!!! important "How to
//! read the levels"` where the source had a callout.
//!
//! Nothing here is a perfect translation; these are Material features and Zellij
//! is not Material. They become the closest plain-Markdown equivalent, which
//! renders sensibly everywhere and loses only the styling.

/// Indentation width of a line, in spaces (tabs count as four).
fn indent_of(line: &str) -> usize {
    line.chars()
        .take_while(|c| c.is_whitespace())
        .map(|c| if c == '\t' { 4 } else { 1 })
        .sum()
}

/// A `!!! type "Optional title"` or `=== "Title"` opener, as `(label, indent)`.
fn opener(line: &str) -> Option<(String, usize)> {
    let indent = indent_of(line);
    let rest = line.trim_start();

    let (kind, remainder) = if let Some(r) = rest.strip_prefix("!!!") {
        ("admonition", r)
    } else {
        ("tab", rest.strip_prefix("===")?)
    };

    let remainder = remainder.trim();
    // `===` is also a setext heading underline, which is three or more `=` and
    // nothing else. A tab always carries a quoted title.
    if kind == "tab" && !remainder.starts_with('"') {
        return None;
    }

    let title = remainder
        .split_once('"')
        .and_then(|(_, after)| after.split_once('"'))
        .map(|(title, _)| title.to_string());
    let word = remainder
        .split_whitespace()
        .next()
        .filter(|w| !w.starts_with('"'))
        .map(|w| w.trim_end_matches('"').to_string());

    let label = match (word, title) {
        (Some(word), Some(title)) if !title.is_empty() => {
            format!("{} — {title}", capitalise(&word))
        }
        (Some(word), _) => capitalise(&word),
        (None, Some(title)) => title,
        (None, None) => return None,
    };
    Some((label, indent))
}

fn capitalise(word: &str) -> String {
    let mut chars = word.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

/// True where a line opens or closes a fenced code block.
fn is_fence(line: &str) -> bool {
    let t = line.trim_start();
    t.starts_with("```") || t.starts_with("~~~")
}

/// Strips a trailing attribute list: `[text](url){ .md-button }`.
///
/// Material uses these to style links as buttons. They are not Markdown, so the
/// braces survived into the page — and `make_mdx_safe` then escaped them, which
/// printed `\{ .md-button \}` after every call to action.
fn strip_attr_lists(line: &str) -> String {
    let mut out = String::with_capacity(line.len());
    let mut rest = line;
    while let Some(at) = rest.find('{') {
        let Some(close) = rest[at..].find('}') else {
            break;
        };
        let inside = &rest[at + 1..at + close];
        // Only an attribute list: classes, ids and key=value, nothing else.
        let attrs = !inside.is_empty()
            && inside.split_whitespace().all(|token| {
                token.starts_with('.') || token.starts_with('#') || token.contains('=')
            });
        if attrs && (rest[..at].ends_with(')') || rest[..at].ends_with("} ")) {
            out.push_str(rest[..at].trim_end());
            rest = &rest[at + close + 1..];
        } else {
            out.push_str(&rest[..=at]);
            rest = &rest[at + 1..];
        }
    }
    out.push_str(rest);
    out
}

/// Rewrites Material's own syntax into Markdown every renderer understands.
pub fn demote_material(source: &str) -> String {
    let mut out = String::with_capacity(source.len());
    let mut in_fence = false;
    let mut block: Option<usize> = None;

    for line in source.lines() {
        if is_fence(line) {
            in_fence = !in_fence;
            out.push_str(line);
            out.push('\n');
            continue;
        }
        if in_fence {
            out.push_str(line);
            out.push('\n');
            continue;
        }

        // Inside an admonition body: quote it until the indentation drops back.
        if let Some(open_indent) = block {
            if line.trim().is_empty() {
                out.push_str(">\n");
                continue;
            }
            if indent_of(line) > open_indent {
                let body = line.trim_start();
                out.push_str("> ");
                out.push_str(&strip_attr_lists(body));
                out.push('\n');
                continue;
            }
            block = None;
            out.push('\n');
        }

        if let Some((label, indent)) = opener(line) {
            out.push_str("> **");
            out.push_str(&label);
            out.push_str("**\n>\n");
            block = Some(indent);
            continue;
        }

        out.push_str(&strip_attr_lists(line));
        out.push('\n');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `!!! note` printed as literal text mid-paragraph. The page built, and
    /// read as though someone had pasted the markup by accident.
    #[test]
    fn an_admonition_becomes_a_blockquote() {
        let out = demote_material("!!! note\n    Stays true during migration.\n\nAfter.\n");
        assert!(out.contains("> **Note**"), "{out}");
        assert!(out.contains("> Stays true during migration."), "{out}");
        assert!(out.contains("After."), "{out}");
        assert!(!out.contains("!!!"), "{out}");
    }

    #[test]
    fn an_admonition_keeps_its_title() {
        let out =
            demote_material("!!! important \"How to read the levels\"\n    L0 is the estate.\n");
        assert!(
            out.contains("> **Important — How to read the levels**"),
            "{out}"
        );
    }

    #[test]
    fn a_content_tab_becomes_a_labelled_block() {
        let out = demote_material("=== \"Azure\"\n    Use Container Apps.\n");
        assert!(out.contains("> **Azure**"), "{out}");
        assert!(out.contains("> Use Container Apps."), "{out}");
    }

    /// `===` under a line is a setext heading, not a tab.
    #[test]
    fn a_setext_heading_underline_is_left_alone() {
        let out = demote_material("Title\n=====\n\ntext\n");
        assert!(out.contains("====="), "{out}");
    }

    #[test]
    fn an_attribute_list_after_a_link_is_dropped() {
        let out = demote_material("[Open the deck](deck.pptx){ .md-button .md-button--primary }\n");
        assert_eq!(out.trim(), "[Open the deck](deck.pptx)");
    }

    /// A brace that is prose, or code, must survive.
    #[test]
    fn a_brace_that_is_not_an_attribute_list_survives() {
        let out = demote_material("set {timeout} to 30\n");
        assert!(out.contains("{timeout}"), "{out}");
    }

    #[test]
    fn fenced_code_is_untouched() {
        let out = demote_material("```\n!!! note\n=== \"x\"\n```\n");
        assert!(out.contains("!!! note"), "{out}");
        assert!(out.contains("=== \"x\""), "{out}");
    }
}
