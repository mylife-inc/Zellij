//! Surgical CST edits preserve comments and all unrelated YAML spelling.
use rowan::ast::AstNode;
use serde_yaml_ng::Value;
use yaml_parser::{SyntaxKind as K, SyntaxNode, ast};
use zellij_core::{Error, Result};
fn invalid(message: impl std::fmt::Display) -> Error {
    Error::InvalidArgument(format!(
        "cannot update portable features: {message}; no files were changed"
    ))
}
fn map(node: &SyntaxNode) -> Option<SyntaxNode> {
    node.descendants()
        .find(|n| matches!(n.kind(), K::BLOCK_MAP | K::FLOW_MAP))
}
fn value(node: &SyntaxNode, name: &str) -> Option<SyntaxNode> {
    let entries: Vec<_> = if let Some(block) = ast::BlockMap::cast(node.clone()) {
        block
            .entries()
            .filter_map(|e| Some((e.key()?.syntax().clone(), e.value()?.syntax().clone())))
            .collect()
    } else {
        ast::FlowMap::cast(node.clone())?
            .entries()?
            .entries()
            .filter_map(|e| Some((e.key()?.syntax().clone(), e.value()?.syntax().clone())))
            .collect()
    };
    entries.into_iter().find_map(|(k, v)| {
        (serde_yaml_ng::from_str::<String>(&k.text().to_string())
            .ok()
            .as_deref()
            == Some(name))
        .then_some(v)
    })
}
fn insert(text: &mut String, map: &SyntaxNode, entry: &str) -> Result<()> {
    if let Some(flow) = ast::FlowMap::cast(map.clone()) {
        let pos: usize = flow
            .l_brace()
            .ok_or_else(|| invalid("missing mapping brace"))?
            .text_range()
            .end()
            .into();
        let comma = if flow.entries().is_some_and(|e| e.entries().next().is_some()) {
            ", "
        } else {
            ""
        };
        text.insert_str(pos, &format!("{entry}{comma}"));
    } else {
        let pos: usize = map.text_range().start().into();
        let indent = text[..pos].rsplit('\n').next().unwrap_or_default().len();
        if !text[..pos]
            .rsplit('\n')
            .next()
            .unwrap_or_default()
            .chars()
            .all(|c| c == ' ')
        {
            return Err(invalid("use a regular block or flow mapping"));
        }
        let newline = if text.contains("\r\n") { "\r\n" } else { "\n" };
        text.insert_str(pos, &format!("{entry}{newline}{}", " ".repeat(indent)));
    }
    Ok(())
}
pub fn select(original: &str, maths: bool, diagrams: bool) -> Result<String> {
    let mut expected: Value = serde_yaml_ng::from_str(original).map_err(invalid)?;
    if !expected.is_mapping() {
        return Err(invalid("zel.yaml must be a mapping"));
    }
    if expected.get("markdown").is_none() {
        expected["markdown"] = Value::Mapping(Default::default());
    }
    if !expected["markdown"].is_mapping() {
        return Err(invalid("markdown must be a mapping"));
    }
    expected["markdown"]["math"] = Value::Bool(maths);
    expected["markdown"]["mermaid"] = Value::Bool(diagrams);
    let mut text = original.to_owned();
    for (key, enabled) in [("math", maths), ("mermaid", diagrams)] {
        let tree = yaml_parser::parse(&text).map_err(invalid)?;
        let root = map(&tree).ok_or_else(|| invalid("missing root mapping"))?;
        let Some(markdown) = value(&root, "markdown") else {
            insert(
                &mut text,
                &root,
                &format!("markdown: {{math: {maths}, mermaid: {diagrams}}}"),
            )?;
            break;
        };
        if markdown
            .descendants_with_tokens()
            .any(|n| matches!(n.kind(), K::PROPERTIES | K::ALIAS))
        {
            return Err(invalid(
                "expand anchors/aliases in markdown before editing its features",
            ));
        }
        let mapping =
            map(&markdown).ok_or_else(|| invalid("markdown must be an explicit mapping"))?;
        if let Some(value) = value(&mapping, key) {
            let scalar = value
                .descendants_with_tokens()
                .find(|n| n.kind() == K::PLAIN_SCALAR)
                .ok_or_else(|| invalid("feature values must be plain booleans"))?;
            text.replace_range(
                std::ops::Range::<usize>::from(scalar.text_range()),
                &enabled.to_string(),
            );
        } else {
            insert(&mut text, &mapping, &format!("{key}: {enabled}"))?;
        }
    }
    let actual: Value = serde_yaml_ng::from_str(&text).map_err(invalid)?;
    if actual != expected {
        return Err(invalid("edit would change other settings"));
    }
    Ok(text)
}
