//! Making ordinary Markdown safe to compile as MDX.
//!
//! Zellij renders guide pages through MDX, which is Markdown plus JSX. That
//! addition is not free: `{` opens a JavaScript expression and `<` opens a tag.
//! Prose written for MkDocs or Sphinx has never had to care, so a page saying
//! `set {timeout} to <n> seconds` is perfectly good Markdown and a syntax error
//! as MDX.
//!
//! This escapes the two characters where they would be read as code. It is the
//! difference between a converter whose output builds and one whose output
//! looks right and fails — and the failure lands on someone who did not write
//! the file and has no reason to suspect the conversion.

/// True where a line opens or closes a fenced code block.
fn is_fence(line: &str) -> bool {
    let trimmed = line.trim_start();
    trimmed.starts_with("```") || trimmed.starts_with("~~~")
}

/// True when `<` at this position begins something MDX will accept as a tag:
/// `<div`, `</div`, `<Foo`, or a comment.
///
/// The name has to end the way a tag's name ends. `<https://example.com>` opens
/// with a letter and is not a tag — MDX reads `https` as the name and fails on
/// the colon, which is how a converted page that looked right failed to build.
/// Is this `<` the start of markup, or a character in a sentence?
///
/// The test is whether the tag is **closed**, not whether the name looks like
/// an element. MDX is JSX: every tag needs a closing tag or a slash, including
/// the ones HTML lets you leave open — `<br>` is an error, `<br/>` is not. So a
/// name alone proves nothing.
///
/// Two earlier rules were both wrong, and both were found by converting a
/// MkDocs tree and compiling the result:
///
/// - "looks like a tag name" accepted `<n>`, which is what a technical writer
///   types for a placeholder. *Expected a closing tag for `<n>`.*
/// - "is a real HTML element" accepted `<span>` in the sentence "use `<div>`
///   and `<span>` for layout", where the author is naming tags rather than
///   opening them. *Expected a closing tag for `<span>`.*
///
/// A tag that is genuinely markup is balanced, because whoever wrote it wanted
/// it to render. A tag that is prose is not, because they were writing about it.
fn opens_a_tag(rest: &str, rest_of_document: &str) -> bool {
    // A closing tag is markup if its opener was; the opener already decided.
    if let Some(rest) = rest.strip_prefix('/') {
        return tag_name(rest).is_some();
    }

    // Comments and doctypes.
    if rest.starts_with('!') {
        return true;
    }

    let Some(name) = tag_name(rest) else {
        return false;
    };

    // Self-closing: `<br/>`, `<img src="…" />`. Complete on its own.
    if let Some(open) = rest.find('>')
        && rest[..open].trim_end().ends_with('/')
    {
        return true;
    }

    // Otherwise it is markup only if something closes it later.
    let closing = format!("</{name}");
    rest_of_document.find(&closing).is_some_and(|at| {
        matches!(
            rest_of_document[at + closing.len()..].chars().next(),
            Some('>' | ' ' | '\t') | None
        )
    })
}

/// The tag name at the start of `rest`, if it is shaped like one.
fn tag_name(rest: &str) -> Option<&str> {
    if !rest.starts_with(|c: char| c.is_ascii_alphabetic()) {
        return None;
    }

    let len = rest
        .find(|c: char| !(c.is_ascii_alphanumeric() || c == '.' || c == '-' || c == '_'))
        .unwrap_or(rest.len());
    let (name, after) = rest.split_at(len);

    matches!(after.chars().next(), Some('>' | '/' | ' ' | '\t') | None).then_some(name)
}

/// The URL of an autolink starting at this position, if there is one.
///
/// MDX has no autolinks: `<https://example.com>` is a tag as far as it is
/// concerned. Rewriting to `[url](url)` keeps a working link where escaping
/// would leave literal text with angle brackets around it.
fn autolink(rest: &str) -> Option<&str> {
    let end = rest.find('>')?;
    let url = &rest[..end];

    let looks_like_a_url = url.starts_with("http://")
        || url.starts_with("https://")
        || (url.contains('@') && !url.contains(' '));

    (looks_like_a_url && !url.contains(' ')).then_some(url)
}

/// Escapes one line's worth of prose.
///
/// `document` is the whole source, because deciding whether `<div>` is markup
/// means knowing whether a `</div>` follows — and it rarely follows on the
/// same line.
fn escape_line(line: &str, document: &str) -> String {
    let mut out = String::with_capacity(line.len());
    // Inline code spans are literal in MDX too, so their contents are left be.
    let mut in_code_span = false;
    // How many bytes of the line the autolink rewrite has already consumed.
    let mut skip_until = 0usize;

    for (index, ch) in line.char_indices() {
        if index < skip_until {
            continue;
        }

        match ch {
            '`' => {
                in_code_span = !in_code_span;
                out.push(ch);
            }
            '{' | '}' if !in_code_span => {
                out.push('\\');
                out.push(ch);
            }
            '<' if !in_code_span => {
                let rest = &line[index + 1..];

                if let Some(url) = autolink(rest) {
                    out.push_str(&format!("[{url}]({url})"));
                    // Past the closing '>' as well as the URL itself.
                    skip_until = index + 1 + url.len() + 1;
                } else if opens_a_tag(rest, document) {
                    out.push(ch);
                } else {
                    out.push_str("&lt;");
                }
            }
            _ => out.push(ch),
        }
    }

    out
}

/// Rewrites Markdown so MDX will compile it, leaving code blocks alone.
pub fn make_mdx_safe(source: &str) -> String {
    let mut out = String::with_capacity(source.len());
    let mut in_fence = false;

    for line in source.lines() {
        if is_fence(line) {
            in_fence = !in_fence;
            out.push_str(line);
        } else if in_fence {
            // Code is quoted verbatim by definition. Escaping inside a fence
            // would put backslashes into commands people copy and run.
            out.push_str(line);
        } else {
            out.push_str(&escape_line(line, source));
        }
        out.push('\n');
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escapes_braces_that_mdx_would_read_as_an_expression() {
        assert_eq!(
            make_mdx_safe("set {timeout} now\n"),
            "set \\{timeout\\} now\n"
        );
    }

    #[test]
    fn escapes_a_less_than_that_opens_no_tag() {
        assert_eq!(make_mdx_safe("wait < 5 seconds\n"), "wait &lt; 5 seconds\n");
    }

    // `<n>` is what a technical writer types for a placeholder, and MDX reads
    // it as an element it must find a closing tag for. Escaping it is the whole
    // difference between a converted folder that builds and one that does not.
    #[test]
    fn escapes_a_placeholder_that_is_not_an_element() {
        let out = make_mdx_safe("Set the timeout to <n> seconds.");
        assert!(out.contains("&lt;n"), "{out}");
        assert!(!out.contains("<n>"), "{out}");
    }

    #[test]
    fn escapes_an_angle_bracketed_placeholder_with_a_hyphen() {
        let out = make_mdx_safe("Send <your-token> in the header.");
        assert!(out.contains("&lt;your-token"), "{out}");
        assert!(!out.contains("<your-token>"), "{out}");
    }

    #[test]
    fn leaves_a_component_alone() {
        let out = make_mdx_safe("<Callout type=\"tip\">hello</Callout>");
        assert!(!out.contains("&lt;"), "{out}");
    }

    // MDX is JSX: an unclosed tag is an error whatever its name, so prose that
    // names a tag has to be escaped even when the name is a real element.
    #[test]
    fn escapes_an_element_named_in_prose() {
        let out = make_mdx_safe("Use <div> and <span> for layout.");
        assert!(out.contains("&lt;div"), "{out}");
        assert!(out.contains("&lt;span"), "{out}");
    }

    #[test]
    fn keeps_a_balanced_element() {
        let out = make_mdx_safe("<div class=\"note\">careful</div>");
        assert!(!out.contains("&lt;"), "{out}");
    }

    // The closing tag can be paragraphs away; the decision is per document.
    #[test]
    fn keeps_an_element_closed_further_down() {
        let out = make_mdx_safe("<div>\n\nsome prose\n\n</div>");
        assert!(!out.contains("&lt;"), "{out}");
    }

    // `<br>` is valid HTML and invalid MDX. `<br/>` is complete on its own.
    #[test]
    fn escapes_a_void_element_written_html_style() {
        let out = make_mdx_safe("one<br>two");
        assert!(out.contains("&lt;br"), "{out}");
    }

    #[test]
    fn keeps_a_self_closing_element() {
        let out = make_mdx_safe("one<br/>two and <img src=\"a.png\" />");
        assert!(!out.contains("&lt;"), "{out}");
    }

    #[test]
    fn leaves_real_tags_alone() {
        assert_eq!(make_mdx_safe("<div>hi</div>\n"), "<div>hi</div>\n");
    }

    #[test]
    fn rewrites_an_autolink_as_a_markdown_link() {
        // MDX has no autolinks — it reads `<https` as a tag name and fails on
        // the colon. Escaping would keep the build green and lose the link, so
        // the link is kept instead.
        assert_eq!(
            make_mdx_safe("see <https://example.com>\n"),
            "see [https://example.com](https://example.com)\n"
        );
    }

    #[test]
    fn rewrites_an_email_autolink_too() {
        assert_eq!(
            make_mdx_safe("mail <a@b.com> now\n"),
            "mail [a@b.com](a@b.com) now\n"
        );
    }

    #[test]
    fn escapes_a_tag_name_that_is_really_a_scheme() {
        assert_eq!(make_mdx_safe("<ftp:host>\n"), "&lt;ftp:host>\n");
    }

    #[test]
    fn leaves_fenced_code_untouched() {
        let source = "text {a}\n```sh\nrun --flag {x} < input\n```\nmore {b}\n";
        let expected = "text \\{a\\}\n```sh\nrun --flag {x} < input\n```\nmore \\{b\\}\n";
        assert_eq!(make_mdx_safe(source), expected);
    }

    #[test]
    fn leaves_inline_code_untouched() {
        assert_eq!(
            make_mdx_safe("use `{placeholder}` here\n"),
            "use `{placeholder}` here\n"
        );
    }
}
