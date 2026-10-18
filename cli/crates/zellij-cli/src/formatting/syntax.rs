use pretty_yaml::{config::FormatOptions, format_text};
use rowan::ast::AstNode;
use serde_yaml_ng::Value;
use std::sync::LazyLock;
use yaml_parser::{SyntaxKind as K, SyntaxNode, ast};

type Failure = String;
fn parse(text: &str) -> Result<Value, Failure> {
    serde_yaml_ng::from_str(text).map_err(|e| format!("invalid single-document YAML: {e}"))
}
fn tree(text: &str) -> Result<SyntaxNode, Failure> {
    yaml_parser::parse(text).map_err(|e| format!("unsupported or invalid YAML syntax: {e}"))
}
fn protected(node: &SyntaxNode) -> bool {
    node.descendants_with_tokens().any(|n| {
        matches!(
            n.kind(),
            K::COMMENT | K::PROPERTIES | K::ALIAS | K::BLOCK_SCALAR
        )
    })
}
fn scalar(value: &Value) -> Option<String> {
    match value {
        Value::Null | Value::Bool(_) | Value::Number(_) => serde_yaml_ng::to_string(value)
            .ok()
            .map(|s| s.trim().to_owned()),
        Value::String(s) if !s.contains(['\n', '\r']) => {
            // Parse the candidate in flow context too: commas/colon/brackets
            // can change meaning even when the plain scalar parses by itself.
            let flow = format!("[{s}]");
            let expected = Value::Sequence(vec![value.clone()]);
            if !s.is_empty()
                && s.trim() == s
                && parse(&flow).ok().as_ref() == Some(&expected)
                && parse(s).ok().as_ref() == Some(value)
                && !s.contains('#')
            {
                Some(s.clone())
            } else {
                serde_json::to_string(s).ok()
            }
        }
        _ => None,
    }
}
fn compact(value: &Value) -> Option<String> {
    match value {
        Value::Sequence(items) => Some(format!(
            "[{}]",
            items
                .iter()
                .map(scalar)
                .collect::<Option<Vec<_>>>()?
                .join(", ")
        )),
        Value::Mapping(items) => Some(format!(
            "{{ {} }}",
            items
                .iter()
                .map(|(k, v)| Some(format!("{}: {}", scalar(k)?, scalar(v)?)))
                .collect::<Option<Vec<_>>>()?
                .join(", ")
        )),
        _ => None,
    }
}
fn replace(text: &mut String, edits: &mut [(std::ops::Range<usize>, String)]) {
    edits.sort_by_key(|(range, _)| std::cmp::Reverse(range.start));
    for (range, body) in edits {
        text.replace_range(range.clone(), body);
    }
}
fn normalize_quotes(text: &str) -> Result<String, Failure> {
    let mut edits = Vec::new();
    for token in tree(text)?
        .descendants_with_tokens()
        .filter_map(|n| n.into_token())
    {
        if matches!(
            token.kind(),
            K::SINGLE_QUOTED_SCALAR | K::DOUBLE_QUOTED_SCALAR
        ) && !token.text().contains(['\n', '\r'])
            && let Some(body) = scalar(&parse(token.text())?)
        {
            edits.push((token.text_range().into(), body));
        }
    }
    let mut text = text.to_owned();
    replace(&mut text, &mut edits);
    Ok(text)
}
fn compact_collections(text: &str) -> Result<String, Failure> {
    let root = tree(text)?;
    let mut edits = Vec::new();
    for node in root.descendants() {
        if !matches!(node.kind(), K::BLOCK_MAP | K::BLOCK_SEQ) || protected(&node) {
            continue;
        }
        // Keep document mappings legible; compact only nested mappings and
        // scalar lists. Scalar-only collections cannot overlap one another.
        if node.kind() == K::BLOCK_MAP
            && !node
                .ancestors()
                .skip(1)
                .any(|n| matches!(n.kind(), K::BLOCK_MAP_VALUE | K::BLOCK_SEQ_ENTRY))
        {
            continue;
        }
        let raw = node.text().to_string();
        // Non-first lines carry their original indent. Parse in a standalone
        // context after removing only that shared indentation.
        let start: usize = node.text_range().start().into();
        let column = text[..start]
            .rsplit('\n')
            .next()
            .unwrap_or_default()
            .chars()
            .count();
        let normalized = raw
            .lines()
            .enumerate()
            .map(|(i, line)| {
                if i == 0 {
                    line
                } else {
                    line.strip_prefix(&" ".repeat(column)).unwrap_or(line)
                }
            })
            .collect::<Vec<_>>()
            .join("\n");
        let Some(body) = parse(&normalized).ok().and_then(|v| compact(&v)) else {
            continue;
        };
        if body.chars().count() + column <= 80 {
            edits.push((node.text_range().into(), body));
        }
    }
    let mut text = text.to_owned();
    replace(&mut text, &mut edits);
    Ok(text)
}

fn order_section(text: &str, value: &Value) -> Result<String, Failure> {
    let Some(kind) = value.get("type").and_then(Value::as_str) else {
        return Ok(text.into());
    };
    // YAML's ordered mapping retains the engine's JSON property order; the
    // usual serde_json map intentionally sorts keys for other CLI consumers.
    static SCHEMA: LazyLock<Value> =
        LazyLock::new(|| serde_yaml_ng::from_str(zellij_core::schema::SCHEMA_JSON).unwrap());
    let section = &SCHEMA["sections"][kind];
    let Some(properties) = section["properties"].as_mapping() else {
        return Ok(text.into());
    };
    let mut names = properties
        .keys()
        .filter_map(Value::as_str)
        .collect::<Vec<_>>();
    let required = section["required"].as_sequence();
    names.sort_by_key(|name| {
        if *name == "type" {
            0
        } else if required.is_some_and(|r| r.contains(&Value::String((*name).into()))) {
            1
        } else {
            2
        }
    });
    let root = tree(text)?;
    // Preserve comment attachment and anchor-before-alias ordering. Syntax
    // formatting still runs, but reordering never moves these constructs.
    if protected(&root) {
        return Ok(text.into());
    }
    let Some(map_node) = root
        .descendants()
        .find(|n| matches!(n.kind(), K::BLOCK_MAP | K::FLOW_MAP))
    else {
        return Ok(text.into());
    };
    if let Some(flow) = ast::FlowMap::cast(map_node.clone()) {
        let Some(entries) = flow.entries() else {
            return Ok(text.into());
        };
        let mut body = Vec::new();
        for entry in entries.entries() {
            // Implicit nulls and complex keys keep their authored flow syntax.
            if entry.colon().is_none() || entry.key().is_none() {
                return Ok(text.into());
            }
            body.push(entry.syntax().text().to_string());
        }
        let mut converted = text.to_owned();
        converted.replace_range(
            std::ops::Range::<usize>::from(map_node.text_range()),
            &body.join("\n"),
        );
        return order_section(&converted, value);
    }
    let map = ast::BlockMap::cast(map_node).unwrap();
    let mut entries = Vec::new();
    for entry in map.entries() {
        let Some(key) = entry.key() else {
            return Ok(text.into());
        };
        let key = parse(&key.syntax().text().to_string())?;
        let Some(key) = key.as_str() else {
            return Ok(text.into());
        };
        entries.push((
            names
                .iter()
                .position(|name| *name == key)
                .unwrap_or(usize::MAX),
            entry.syntax().text().to_string(),
        ));
    }
    entries.sort_by_key(|(rank, _)| *rank);
    let mut result = text.to_owned();
    result.replace_range(
        std::ops::Range::<usize>::from(map.syntax().text_range()),
        &entries
            .into_iter()
            .map(|(_, body)| body.trim_end().to_owned())
            .collect::<Vec<_>>()
            .join("\n"),
    );
    Ok(result)
}

fn restore_block_scalars(original: &str, formatted: &str) -> Result<String, Failure> {
    let blocks = |text: &str| -> Result<Vec<_>, Failure> {
        Ok(tree(text)?
            .descendants()
            .filter(|node| node.kind() == K::BLOCK_SCALAR)
            .map(|node| {
                let mut range: std::ops::Range<usize> = node.text_range().into();
                // CST trivia owns trailing empty lines, but YAML's |+ value owns
                // their meaning. Include complete trailing lines in the restore.
                let whitespace: String = text[range.end..]
                    .chars()
                    .take_while(|c| c.is_ascii_whitespace())
                    .collect();
                if let Some(last) = whitespace.rfind('\n') {
                    range.end += last + 1;
                }
                let indent = text[..range.start]
                    .rsplit('\n')
                    .next()
                    .unwrap_or_default()
                    .bytes()
                    .take_while(|c| *c == b' ')
                    .count();
                let body = text[range.clone()].replace("\r\n", "\n");
                (range, indent, body)
            })
            .collect())
    };
    let before = blocks(original)?;
    let after = blocks(formatted)?;
    if before.len() != after.len() {
        return Err("formatting changed multiline scalar structure".into());
    }
    let mut edits = Vec::new();
    for ((_, old_indent, body), (range, new_indent, _)) in before.into_iter().zip(after) {
        let mut restored = String::new();
        for (i, line) in body.split_inclusive('\n').enumerate() {
            if i == 0 || line == "\n" {
                restored.push_str(line);
                continue;
            }
            if new_indent >= old_indent {
                restored.push_str(&" ".repeat(new_indent - old_indent));
                restored.push_str(line);
            } else {
                let remove =
                    (old_indent - new_indent).min(line.bytes().take_while(|c| *c == b' ').count());
                restored.push_str(
                    line.strip_prefix(&" ".repeat(remove))
                        .ok_or("cannot preserve multiline scalar indentation")?,
                );
            }
        }
        edits.push((range, restored));
    }
    let mut result = formatted.to_owned();
    replace(&mut result, &mut edits);
    Ok(result)
}

pub(super) fn format(text: &str) -> Result<String, Failure> {
    let before = parse(text)?;
    let mut options = FormatOptions::default();
    options.language.trailing_comma = false;
    options.language.prefer_single_line = true;
    // Trailing spaces inside literal/folded scalars are authored content.
    options.language.trim_trailing_whitespaces = false;
    let first = format_text(text, &options).map_err(|e| e.to_string())?;
    let quoted = normalize_quotes(&first)?;
    let compacted = compact_collections(&quoted)?;
    let ordered = order_section(&compacted, &before)?;
    let result = format_text(&ordered, &options).map_err(|e| e.to_string())?;
    // The upstream printer normalizes blank lines inside |+ scalars. Restore
    // their authored spelling and content, shifting only structural indent.
    let result = restore_block_scalars(text, &result)?;
    if parse(&result)? != before {
        return Err("formatting would change YAML values; refusing to write".into());
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn canonical_layout_and_schema_order_are_idempotent() {
        let input = "padding: 'large'\nheadline: Hello\ntype: hero\ncta:\n    - label: Read\n      href: /guide\n";
        let output = format(input).unwrap();
        assert!(
            output.starts_with("type: hero\nheadline: Hello\n"),
            "{output}"
        );
        assert!(output.contains("{ label: Read, href: /guide }"), "{output}");
        assert!(output.contains("padding: large"), "{output}");
        assert_eq!(format(&output).unwrap(), output);
        let flow = format("{ padding: large, headline: Hello, type: hero }\n").unwrap();
        assert!(flow.starts_with("type: hero\nheadline: Hello\n"), "{flow}");
    }
    #[test]
    fn comments_anchors_literals_and_scalar_types_survive() {
        for input in [
            "# Heading\nbase: &base { a: 'true', b: 1.00 } # inline\ncopy: *base\n",
            "text: |+\n  trailing  \n\n\n",
            "text: >-\n  folded text\n  next line\n",
            "name: \"null\"\nitems: ['yes', '42', true, null, 'a,b']\n",
            "# Order stays attached\nheadline: Hello # prose\ntype: hero\n",
            "text: |-\n  a\r\n  b\r\n",
            "outer:\n    text: |+\n        a  \n          \n\n    other: value\n",
            "items:\n    - |2+\n        text\n        \n\n    - next\n",
            "a: .nan\nb: .inf\n",
            "title: \"日本語 hello\"\ntruth: 'true'\nempty: ''\ncolon: 'a: b'\ncomma: 'a,b'\n",
        ] {
            let output = format(input).unwrap_or_else(|e| panic!("{e}: {input:?}"));
            assert_eq!(parse(&output).unwrap(), parse(input).unwrap());
            assert_eq!(format(&output).unwrap(), output);
            for line in input.lines().filter(|line| line.starts_with('#')) {
                assert!(output.contains(line));
            }
        }
    }
}
