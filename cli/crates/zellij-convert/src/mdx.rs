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
fn opens_a_tag(rest: &str) -> bool {
    let rest = rest.strip_prefix('/').unwrap_or(rest);

    if rest.starts_with('!') {
        return true;
    }

    let mut chars = rest.chars();
    if !chars.next().is_some_and(|c| c.is_ascii_alphabetic()) {
        return false;
    }

    let after_name = rest.trim_start_matches(|c: char| {
        c.is_ascii_alphanumeric() || c == '.' || c == '-' || c == '_'
    });
    matches!(
        after_name.chars().next(),
        Some('>' | '/' | ' ' | '\t') | None
    )
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
fn escape_line(line: &str) -> String {
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
                } else if opens_a_tag(rest) {
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
            out.push_str(&escape_line(line));
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
