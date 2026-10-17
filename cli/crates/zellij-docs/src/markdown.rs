use pulldown_cmark::{CodeBlockKind, Event, HeadingLevel, Options, Parser, Tag, TagEnd};
use serde_yaml_ng::Value;
use std::{collections::BTreeSet, sync::LazyLock};
use syntect::{highlighting::ThemeSet, parsing::SyntaxSet};
use zellij_core::{Error, Result};

#[derive(Debug)]
pub struct Heading {
    pub id: String,
    pub title: String,
    pub level: HeadingLevel,
}

pub struct Markdown {
    pub events: Vec<Event<'static>>,
    pub headings: Vec<Heading>,
    pub title: String,
    pub description: String,
    pub text: String,
    pub order: i64,
    pub has_h1: bool,
    pub literal_html: bool,
}

pub fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn invalid(message: impl Into<String>) -> Error {
    Error::InvalidArgument(message.into())
}

fn frontmatter(source: &str) -> Result<(Value, String)> {
    let source = source.trim_start_matches('\u{feff}').replace("\r\n", "\n");
    let Some(rest) = source.strip_prefix("---\n") else {
        return Ok((Value::Null, source));
    };
    let mut offset = 0;
    for line in rest.split_inclusive('\n') {
        if matches!(line.trim_end_matches('\n'), "---" | "...") {
            let meta: Value = serde_yaml_ng::from_str(&rest[..offset])
                .map_err(|e| invalid(format!("invalid Markdown front matter: {e}")))?;
            if !meta.is_mapping() && !meta.is_null() {
                return Err(invalid("Markdown front matter must be a mapping"));
            }
            return Ok((meta, rest[offset + line.len()..].to_owned()));
        }
        offset += line.len();
    }
    Err(invalid(
        "Markdown front matter has no closing --- delimiter",
    ))
}

fn metadata_string(meta: &Value, key: &str) -> Result<Option<String>> {
    match meta.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(s)) if !s.trim().is_empty() => Ok(Some(s.clone())),
        _ => Err(invalid(format!(
            "front matter {key} must be a nonempty string"
        ))),
    }
}

pub fn slug(text: &str) -> String {
    let mut out = String::new();
    for c in text.to_lowercase().chars() {
        if c.is_alphanumeric() || c == '_' || c == '-' {
            out.push(c);
        } else if c.is_whitespace() {
            out.push('-');
        }
    }
    if out.is_empty() {
        "section".into()
    } else {
        out
    }
}

fn plain(events: &[Event<'_>]) -> String {
    let mut text = String::new();
    for event in events {
        match event {
            Event::Text(s) | Event::Code(s) => text.push_str(s),
            Event::SoftBreak | Event::HardBreak | Event::End(_) => text.push(' '),
            _ => {}
        }
    }
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

pub fn parse(source: &str, fallback: &str) -> Result<Markdown> {
    let (meta, body) = frontmatter(source)?;
    let options = Options::ENABLE_TABLES
        | Options::ENABLE_FOOTNOTES
        | Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_TASKLISTS
        | Options::ENABLE_HEADING_ATTRIBUTES
        | Options::ENABLE_GFM;
    let mut events: Vec<_> = Parser::new_ext(&body, options)
        .map(Event::into_static)
        .collect();
    let mut used = BTreeSet::new();
    for event in &events {
        if let Event::Start(Tag::FootnoteDefinition(id)) = event {
            if id.starts_with("_zel-") || !used.insert(id.to_string()) {
                return Err(invalid(format!("duplicate or reserved footnote ID {id:?}")));
            }
        }
    }
    // Reserve explicit IDs before deriving any slugs, including IDs appearing later.
    for event in &events {
        if let Event::Start(Tag::Heading { id: Some(id), .. }) = event {
            if id.is_empty()
                || id.starts_with("_zel-")
                || id.chars().any(|c| c.is_whitespace() || c.is_control())
            {
                return Err(invalid(format!("invalid or reserved heading ID {id:?}")));
            }
            if !used.insert(id.to_string()) {
                return Err(invalid(format!("duplicate heading ID {id:?}")));
            }
        }
    }
    let mut headings = Vec::new();
    let mut first_h1 = None;
    let mut literal_html = false;
    for i in 0..events.len() {
        if let Event::Start(Tag::Heading { level, id, .. }) = &events[i] {
            let level = *level;
            let end = (i + 1..events.len())
                .find(|j| matches!(events[*j], Event::End(TagEnd::Heading(_))))
                .unwrap();
            let title = plain(&events[i + 1..end]);
            let id = if let Some(id) = id {
                id.to_string()
            } else {
                let base = slug(&title);
                let mut candidate = base.clone();
                let mut n = 0;
                while candidate.starts_with("_zel-") || !used.insert(candidate.clone()) {
                    n += 1;
                    candidate = format!("section-{n}");
                    if !base.starts_with("_zel-") {
                        candidate = format!("{base}-{n}");
                    }
                }
                candidate
            };
            if level == HeadingLevel::H1 && first_h1.is_none() {
                first_h1 = Some(title.clone());
            }
            events[i] = Event::Start(Tag::Heading {
                level,
                id: Some(id.clone().into()),
                classes: vec![],
                attrs: vec![],
            });
            headings.push(Heading { id, title, level });
        } else if let Event::Html(raw) | Event::InlineHtml(raw) = &events[i] {
            // Portable Markdown is data, not a second MDX/browser program. Raw
            // HTML is shown literally, and reported by the build command.
            literal_html = true;
            events[i] = Event::Text(raw.clone());
        }
    }
    let has_h1 = first_h1.is_some();
    let title = metadata_string(&meta, "title")?
        .or(first_h1)
        .unwrap_or_else(|| fallback.into());
    let description = metadata_string(&meta, "description")?.unwrap_or_default();
    let order = match meta.get("order") {
        None => 0,
        Some(v) => v
            .as_i64()
            .ok_or_else(|| invalid("front matter order must be an integer"))?,
    };
    Ok(Markdown {
        text: plain(&events),
        events,
        headings,
        title,
        description,
        order,
        has_h1,
        literal_html,
    })
}

pub fn highlight(events: Vec<Event<'static>>) -> Result<Vec<Event<'static>>> {
    static SYNTAXES: LazyLock<SyntaxSet> = LazyLock::new(SyntaxSet::load_defaults_newlines);
    static THEMES: LazyLock<ThemeSet> = LazyLock::new(ThemeSet::load_defaults);
    let mut output = Vec::new();
    let mut iter = events.into_iter();
    while let Some(event) = iter.next() {
        if let Event::Start(Tag::CodeBlock(kind)) = event {
            let language = match kind {
                CodeBlockKind::Fenced(s) => s.split_whitespace().next().unwrap_or("").to_owned(),
                _ => String::new(),
            };
            let mut code = String::new();
            for event in iter.by_ref() {
                match event {
                    Event::End(TagEnd::CodeBlock) => break,
                    Event::Text(s) => code.push_str(&s),
                    _ => {}
                }
            }
            let syntax = SYNTAXES
                .find_syntax_by_token(&language)
                .unwrap_or_else(|| SYNTAXES.find_syntax_plain_text());
            let html = syntect::html::highlighted_html_for_string(
                &code,
                &SYNTAXES,
                syntax,
                &THEMES.themes["base16-ocean.dark"],
            )
            .map_err(|e| invalid(format!("syntax highlighting failed: {e}")))?;
            output.push(Event::Html(
                format!(
                    "<div class=\"code-block\" data-language=\"{}\">{html}</div>",
                    escape(&language)
                )
                .into(),
            ));
        } else {
            output.push(event);
        }
    }
    Ok(output)
}
