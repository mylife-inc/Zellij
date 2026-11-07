//! Document-scoped, bounded inline substitutions; no source evaluation.
use super::{invalid, rst};
use pulldown_cmark::{Event, Parser, Tag};
use std::collections::BTreeMap;
use zellij_core::Result;

#[derive(Debug)]
pub enum Definition {
    Replace(String),
    Literal(String),
    Image {
        source: String,
        alt: String,
        target: Option<String>,
    },
}
pub type Definitions = BTreeMap<String, Definition>;

pub fn name(text: &str) -> Result<String> {
    if text.is_empty()
        || text.trim() != text
        || text.len() > 256
        || text.contains(['|', '\\'])
        || text.chars().any(char::is_control)
    {
        return Err(invalid(
            "substitution names must be nonempty, unescaped and at most 256 bytes",
        ));
    }
    Ok(text.split_whitespace().collect::<Vec<_>>().join(" "))
}

pub fn insert(defs: &mut Definitions, key: String, value: Definition) -> Result<()> {
    if defs.len() >= 1000 {
        return Err(invalid("document exceeds 1000 substitution definitions"));
    }
    if defs.insert(key, value).is_some() {
        return Err(invalid("duplicate substitution definition"));
    }
    Ok(())
}

pub fn replacement(arg: &str, body: &str) -> Result<Definition> {
    let text = [arg, body]
        .into_iter()
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("\n");
    if text.is_empty() || text.len() > 65536 {
        return Err(invalid(
            "replacement text must contain one paragraph of at most 65536 bytes",
        ));
    }
    let lines: Vec<_> = text.lines().collect();
    for (i, line) in lines.iter().enumerate() {
        if line.is_empty()
            || line.starts_with(char::is_whitespace)
            || line.starts_with("..")
            || line.starts_with("| ")
            || line.ends_with("::")
            || super::tables::starts(line)
            || rst::adornment(line).is_some()
            || super::lists::starts(&lines[i..])?
            || line.starts_with(':') && !line.contains(":`")
        {
            return Err(invalid(
                "replace accepts a single inline paragraph, without options or block constructs",
            ));
        }
    }
    Ok(Definition::Replace(lines.join(" ")))
}

pub fn literal(text: &str) -> Result<Definition> {
    if text.len() > 65536 || text.contains(['\n', '\r', '\0']) {
        return Err(invalid(
            "version/release substitutions must be single-line strings of at most 65536 bytes",
        ));
    }
    Ok(Definition::Literal(text.into()))
}

/// Wrap already-rendered inline markup, retaining emphasis and inline images.
pub fn linked(markup: &str, url: &str) -> Result<String> {
    if Parser::new(markup).any(|e| matches!(e, Event::Start(Tag::Link { .. }))) {
        return Err(invalid("a linked substitution cannot contain another link"));
    }
    Ok(format!("[{markup}](<{}>)", rst::destination(url)?))
}

pub struct Expansion<'a> {
    definitions: &'a Definitions,
    resolve: &'a dyn Fn(&str, &str) -> Result<String>,
    active: Vec<String>,
    calls: usize,
    bytes: usize,
}
impl<'a> Expansion<'a> {
    pub fn new(
        definitions: &'a Definitions,
        resolve: &'a dyn Fn(&str, &str) -> Result<String>,
    ) -> Self {
        Self {
            definitions,
            resolve,
            active: Vec::new(),
            calls: 0,
            bytes: 0,
        }
    }

    pub fn reference(&self, kind: &str, text: &str) -> Result<String> {
        (self.resolve)(kind, text)
    }

    pub fn expand(&mut self, text: &str) -> Result<String> {
        let requested = name(text)?;
        let definitions = self.definitions;
        let (key, value) = if let Some(pair) = definitions.get_key_value(&requested) {
            pair
        } else {
            let folded = requested.to_lowercase();
            let mut matching = definitions
                .iter()
                .filter(|(key, _)| key.to_lowercase() == folded);
            let pair = matching
                .next()
                .ok_or_else(|| invalid(format!("undefined substitution {requested:?}")))?;
            if matching.next().is_some() {
                return Err(invalid(format!(
                    "ambiguous substitution case for {requested:?}; use an exact name"
                )));
            }
            pair
        };
        if self.active.contains(key) {
            return Err(invalid(format!("circular substitution reference {key:?}")));
        }
        if self.active.len() >= 16 {
            return Err(invalid("substitution nesting exceeds 16"));
        }
        self.calls += 1;
        if self.calls > 10000 {
            return Err(invalid("substitution expansion exceeds 10000 references"));
        }
        self.active.push(key.clone());
        let output = match value {
            Definition::Replace(text) => rst::inline(text, self)?,
            Definition::Literal(text) => {
                // Configuration values are text, not reStructuredText/Markdown.
                let mut escaped = rst::escaped(text);
                for ch in ['#', '~', '+', '-', '.', '=', '!'] {
                    escaped = escaped.replace(ch, &format!("\\{ch}"));
                }
                escaped
            }
            Definition::Image {
                source,
                alt,
                target,
            } => {
                let image = format!("!{}", rst::link(alt, &self.reference("asset", source)?)?);
                if let Some(target) = target {
                    let href = if let Some(name) = target
                        .strip_suffix('_')
                        .filter(|_| !target.contains(['/', ':', '#', '?']))
                    {
                        self.reference("named-target", name)?
                    } else {
                        target.clone()
                    };
                    linked(&image, &href)?
                } else {
                    image
                }
            }
        };
        self.active.pop();
        // Only a paragraph of inline elements may be inserted. In particular,
        // replacing a token at the start of a line must not invent a list.
        if Parser::new(&output).any(|event| matches!(event, Event::Start(tag)
            if !matches!(tag, Tag::Paragraph | Tag::Emphasis | Tag::Strong | Tag::Link { .. } | Tag::Image { .. })))
        {
            return Err(invalid("substitution expansion must remain inline content"));
        }
        self.bytes += output.len();
        if self.bytes > 4 * 1024 * 1024 {
            return Err(invalid(
                "substitution expansion exceeds 4 MiB of processed text",
            ));
        }
        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn resolve(kind: &str, text: &str) -> Result<String> {
        Ok(if kind == "asset" {
            format!("/assets/{text}")
        } else {
            text.into()
        })
    }

    #[test]
    fn exact_names_whitespace_and_unique_case_fallback_are_distinct() {
        let defs = BTreeMap::from([
            ("Product".into(), Definition::Replace("**First**".into())),
            ("product".into(), Definition::Replace("Second".into())),
            ("Two words".into(), Definition::Replace("two words".into())),
        ]);
        let mut context = Expansion::new(&defs, &resolve);
        assert_eq!(context.expand("Product").unwrap(), "**First**");
        assert_eq!(context.expand("product").unwrap(), "Second");
        assert!(context.expand("PRODUCT").is_err());
        assert_eq!(context.expand("two   words").unwrap(), "two words");
        assert!(context.expand("missing").is_err());
        assert!(name(&"n".repeat(257)).is_err());
    }

    #[test]
    fn literal_values_and_code_are_not_evaluated_as_substitutions_or_markup() {
        let value = "1. |undefined| **plain** <tag> {code()}";
        let defs = BTreeMap::from([("version".into(), literal(value).unwrap())]);
        let mut context = Expansion::new(&defs, &resolve);
        let expanded = context.expand("version").unwrap();
        let text: String = Parser::new(&expanded)
            .filter_map(|e| match e {
                Event::Text(t) => Some(t.to_string()),
                _ => None,
            })
            .collect();
        assert_eq!(text, value);
        assert_eq!(
            rst::inline("``|missing|``", &mut context).unwrap(),
            "` |missing| `"
        );
        assert_eq!(
            rst::inline("\\|missing|", &mut context).unwrap(),
            "\\|missing\\|"
        );
        assert_eq!(
            rst::inline("word\\ |version|\\ suffix", &mut context).unwrap(),
            format!("word{expanded}suffix")
        );
    }

    #[test]
    fn substitution_depth_cycles_and_expansion_budgets_are_bounded() {
        for length in [16, 17] {
            let defs: Definitions = (0..length)
                .map(|i| {
                    (
                        format!("n{i}"),
                        Definition::Replace(if i + 1 == length {
                            "leaf".into()
                        } else {
                            format!("|n{}|", i + 1)
                        }),
                    )
                })
                .collect();
            let result = Expansion::new(&defs, &resolve).expand("n0");
            if length == 16 {
                assert_eq!(result.unwrap(), "leaf");
            } else {
                assert!(
                    result
                        .unwrap_err()
                        .to_string()
                        .contains("nesting exceeds 16")
                );
            }
        }
        let defs = BTreeMap::from([
            ("a".into(), Definition::Replace("|b|".into())),
            ("b".into(), Definition::Replace("|a|".into())),
        ]);
        assert!(
            Expansion::new(&defs, &resolve)
                .expand("a")
                .unwrap_err()
                .to_string()
                .contains("circular")
        );
        let defs = BTreeMap::from([("x".into(), Definition::Replace("x".into()))]);
        let mut context = Expansion::new(&defs, &resolve);
        for _ in 0..10000 {
            context.expand("x").unwrap();
        }
        assert!(
            context
                .expand("x")
                .unwrap_err()
                .to_string()
                .contains("10000 references")
        );
        let defs = BTreeMap::from([("x".into(), Definition::Replace("x".repeat(65536)))]);
        let mut context = Expansion::new(&defs, &resolve);
        for _ in 0..64 {
            context.expand("x").unwrap();
        }
        assert!(
            context
                .expand("x")
                .unwrap_err()
                .to_string()
                .contains("4 MiB")
        );
    }

    #[test]
    fn image_urls_ending_in_underscores_are_not_named_references() {
        for target in [
            "https://example.com/name_",
            "/guide/name_",
            "details.html#section_",
            "details.html?q=value_",
        ] {
            let defs = BTreeMap::from([(
                "icon".into(),
                Definition::Image {
                    source: "icon.svg".into(),
                    alt: "Icon".into(),
                    target: Some(target.into()),
                },
            )]);
            assert_eq!(
                Expansion::new(&defs, &resolve).expand("icon").unwrap(),
                format!("[![Icon](</assets/icon.svg>)](<{target}>)")
            );
        }
    }

    #[test]
    fn replacement_blocks_and_definition_count_are_rejected() {
        for (arg, body) in [
            ("", ""),
            ("one", "two\n\nthree"),
            ("Title", "-----"),
            ("- list", ""),
            ("text", ":trim:"),
            ("text", ".. include:: private"),
        ] {
            assert!(replacement(arg, body).is_err(), "{arg:?} {body:?}");
        }
        assert!(replacement(&"x".repeat(65537), "").is_err());
        assert!(literal("two\nlines").is_err());
        assert!(linked("[inner](<https://example.com>)", "https://example.net").is_err());
        assert_eq!(
            linked("*formatted*", "guide.html").unwrap(),
            "[*formatted*](<guide.html>)"
        );
        let mut defs = Definitions::new();
        for i in 0..1000 {
            insert(&mut defs, i.to_string(), Definition::Replace("x".into())).unwrap();
        }
        assert!(insert(&mut defs, "more".into(), Definition::Replace("x".into())).is_err());
    }
}
