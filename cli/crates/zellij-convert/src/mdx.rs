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

    // Doctypes pass through. Comments do not reach here — `escape_line`
    // rewrites them before this is asked, because MDX has no HTML comment and
    // letting one through fails the build rather than rendering nothing.
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

/// Elements HTML lets you leave open and JSX does not.
const VOID_ELEMENTS: [&str; 13] = [
    "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "source", "track",
    "wbr",
];

/// A void element written HTML-style at `rest`, as `(name, inside, end)`.
///
/// `inside` is everything between the name and the closing `>`; `end` is the
/// byte offset of that `>` within `rest`. Quotes are tracked, because an
/// attribute value may contain `>` — an alt text describing a flow, for one.
fn void_element(rest: &str) -> Option<(&str, &str, usize)> {
    let name = tag_name(rest)?;
    if !VOID_ELEMENTS.contains(&name.to_ascii_lowercase().as_str()) {
        return None;
    }

    let mut quote: Option<char> = None;
    for (at, ch) in rest.char_indices() {
        match (quote, ch) {
            (Some(q), c) if c == q => quote = None,
            (Some(_), _) => {}
            (None, '"' | '\'') => quote = Some(ch),
            (None, '>') => {
                let inside = &rest[name.len()..at];
                // Already self-closing; `opens_a_tag` handles that case.
                return (!inside.trim_end().ends_with('/')).then_some((name, inside, at));
            }
            _ => {}
        }
    }
    None
}

/// The body of an HTML comment opening at `rest`, if one does.
///
/// `rest` begins just after the `<`, so a comment starts `!--` and ends `-->`.
/// Returns what sits between them, which becomes the body of the MDX comment
/// that replaces it.
fn html_comment(rest: &str) -> Option<&str> {
    let body = rest.strip_prefix("!--")?;
    let end = body.find("-->")?;
    Some(&body[..end])
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

                // `](<path>)` is Markdown's angle-bracket link destination, the
                // form used whenever a filename contains spaces or parentheses.
                // Escaping the bracket turns a working link into literal text
                // reading `](&lt;…>)`, which is how a converted page kept every
                // one of its links to files whose names came from SharePoint and
                // rendered not one of them.
                if out.ends_with("](")
                    && let Some(close) = rest.find('>')
                {
                    out.push(ch);
                    out.push_str(&rest[..=close]);
                    skip_until = index + 1 + close + 1;
                } else if let Some((name, inside, end)) = void_element(rest) {
                    // `<img src="…">` is valid HTML and invalid JSX, so MDX
                    // rejects it. Escaping made the build pass and printed the
                    // tag into the page as text — an image replaced by its own
                    // source code, on a page whose whole point was the diagram.
                    // Self-closing it is what the author meant and what MDX
                    // accepts.
                    out.push('<');
                    out.push_str(name);
                    out.push_str(inside.trim_end());
                    out.push_str(" />");
                    skip_until = index + 1 + end + 1;
                } else if let Some(comment) = html_comment(rest) {
                    // MDX is JSX: `<!-- … -->` is not a comment, it is a syntax
                    // error on the `!`. `{/* … */}` is the equivalent and keeps
                    // the author's note out of the rendered page.
                    out.push_str("{/*");
                    out.push_str(&comment.replace("*/", "* /"));
                    out.push_str("*/}");
                    skip_until = index + "<!--".len() + comment.len() + "-->".len();
                } else if let Some(url) = autolink(rest) {
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

    /// MDX has no HTML comment: `<!--` is a syntax error on the `!`.
    ///
    /// MkDocs pages carry them routinely — "generated by, do not edit" notes at
    /// the top of a file. Passing one through failed the build on a page nobody
    /// had touched, with an error about an unexpected `!` that says nothing
    /// about where it came from.
    #[test]
    fn turns_an_html_comment_into_an_mdx_comment() {
        let out = make_mdx_safe("<!-- generated; do not edit -->\n\ntext\n");
        assert!(out.contains("{/* generated; do not edit */}"), "{out}");
        assert!(!out.contains("<!--"), "{out}");
        assert!(out.contains("text"), "{out}");
    }

    /// The comment's own text must not be able to close the MDX comment early.
    #[test]
    fn a_comment_containing_a_close_marker_stays_one_comment() {
        let out = make_mdx_safe("<!-- see */ here -->\n");
        assert_eq!(out.matches("*/}").count(), 1, "{out}");
    }

    /// `](<path>)` is Markdown's destination form for names with spaces.
    ///
    /// Escaping the bracket leaves `](&lt;…>)` in the page: not a link, and not
    /// obviously broken either — it renders as text that looks almost right.
    /// Every link to a file whose name came from SharePoint took this form.
    #[test]
    fn leaves_an_angle_bracketed_link_destination_alone() {
        let out = make_mdx_safe("see [the report](<../docs/Q3 Report (final).pdf>) for detail\n");
        assert!(out.contains("](<../docs/Q3 Report (final).pdf>)"), "{out}");
        assert!(!out.contains("&lt;"), "{out}");
    }

    /// The escape still applies to a `<` that is prose rather than a link.
    #[test]
    fn still_escapes_a_less_than_next_to_a_link() {
        let out = make_mdx_safe("[a](b) and x < y\n");
        assert!(out.contains("x &lt; y"), "{out}");
    }

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

    // `<br>` is valid HTML and invalid MDX. `<br/>` is complete on its own, so
    // the conversion closes it rather than escaping it.
    #[test]
    fn closes_a_void_element_written_html_style() {
        let out = make_mdx_safe("one<br>two");
        assert!(out.contains("<br />"), "{out}");
        assert!(!out.contains("&lt;br"), "{out}");
    }

    /// An `<img>` is the case that matters: escaping printed the tag into the
    /// page as text, so a page built around a diagram showed its own source
    /// instead. The build was green throughout.
    #[test]
    fn closes_an_img_and_keeps_every_attribute() {
        let out = make_mdx_safe(
            "<img src=\"../assets/atlas/01-overview.png\" alt=\"users, hub and spoke\" loading=\"lazy\">\n",
        );
        assert!(
            out.contains("src=\"../assets/atlas/01-overview.png\""),
            "{out}"
        );
        assert!(out.contains("loading=\"lazy\" />"), "{out}");
        assert!(!out.contains("&lt;img"), "{out}");
    }

    /// A `>` inside an attribute value must not be mistaken for the tag's end.
    #[test]
    fn closes_an_img_whose_alt_text_contains_an_angle_bracket() {
        let out = make_mdx_safe("<img src=\"a.png\" alt=\"client -> server\">\n");
        assert!(out.contains("alt=\"client -> server\" />"), "{out}");
        assert!(!out.contains("&lt;img"), "{out}");
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
