//! YAML hierarchies become SVG and a semantic outline during compilation.
use crate::markdown::escape;
use serde::de::{self, DeserializeSeed, MapAccess, SeqAccess, Visitor};
use serde_yaml_ng::{Mapping, Value};
use std::fmt;
use zellij_core::{Error, Result};

const MAX_NODES: usize = 512;
const MAX_LEVELS: usize = 32;
const MAX_LABEL: usize = 256;
const ROW: f64 = 42.0;
const HEIGHT: f64 = 30.0;
const GAP: f64 = 44.0;
const MARGIN: f64 = 12.0;

// Bound expansion while deserializing, before YAML aliases can allocate a large
// Value tree. The graph's tighter node/depth limits are checked separately.
struct BoundedValue<'a> {
    remaining: &'a mut usize,
    depth: usize,
}
impl<'de> DeserializeSeed<'de> for BoundedValue<'_> {
    type Value = Value;
    fn deserialize<D: de::Deserializer<'de>>(
        self,
        deserializer: D,
    ) -> std::result::Result<Value, D::Error> {
        if *self.remaining == 0 {
            return Err(de::Error::custom(
                "mindmaps support at most 512 nodes (YAML expansion limit)",
            ));
        }
        if self.depth > MAX_LEVELS * 2 {
            return Err(de::Error::custom("mindmaps support at most 32 levels"));
        }
        *self.remaining -= 1;
        deserializer.deserialize_any(self)
    }
}
impl<'de> Visitor<'de> for BoundedValue<'_> {
    type Value = Value;
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("plain mindmap YAML, without custom tags")
    }
    fn visit_unit<E: de::Error>(self) -> std::result::Result<Value, E> {
        Ok(Value::Null)
    }
    fn visit_bool<E: de::Error>(self, v: bool) -> std::result::Result<Value, E> {
        Ok(Value::Bool(v))
    }
    fn visit_i64<E: de::Error>(self, v: i64) -> std::result::Result<Value, E> {
        Ok(Value::Number(v.into()))
    }
    fn visit_u64<E: de::Error>(self, v: u64) -> std::result::Result<Value, E> {
        Ok(Value::Number(v.into()))
    }
    fn visit_f64<E: de::Error>(self, v: f64) -> std::result::Result<Value, E> {
        if !v.is_finite() {
            return Err(E::custom("mindmap numbers must be finite"));
        }
        Ok(Value::Number(v.into()))
    }
    fn visit_str<E: de::Error>(self, v: &str) -> std::result::Result<Value, E> {
        if v.chars().count() > MAX_LABEL {
            return Err(E::custom("mindmap labels support at most 256 characters"));
        }
        Ok(Value::String(v.into()))
    }
    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> std::result::Result<Value, A::Error> {
        let mut values = Vec::new();
        while let Some(value) = seq.next_element_seed(BoundedValue {
            remaining: self.remaining,
            depth: self.depth + 1,
        })? {
            values.push(value);
        }
        Ok(Value::Sequence(values))
    }
    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> std::result::Result<Value, A::Error> {
        let mut values = Mapping::new();
        while let Some(key) = map.next_key_seed(BoundedValue {
            remaining: self.remaining,
            depth: self.depth + 1,
        })? {
            if !key.is_string() {
                return Err(de::Error::custom("mindmap keys must be strings"));
            }
            let value = map.next_value_seed(BoundedValue {
                remaining: self.remaining,
                depth: self.depth + 1,
            })?;
            if values.insert(key, value).is_some() {
                return Err(de::Error::custom("duplicate mindmap key"));
            }
        }
        Ok(Value::Mapping(values))
    }
}

struct Node {
    label: String,
    children: Vec<Node>,
    width: f64,
    y: f64,
}

fn invalid(message: impl Into<String>) -> Error {
    Error::InvalidArgument(message.into())
}

fn node(label: &str, value: &Value, depth: usize, count: &mut usize) -> Result<Node> {
    if depth >= MAX_LEVELS {
        return Err(invalid("mindmaps support at most 32 levels"));
    }
    *count += 1;
    if *count > MAX_NODES {
        return Err(invalid("mindmaps support at most 512 nodes"));
    }
    let label = label.split_whitespace().collect::<Vec<_>>().join(" ");
    if label.is_empty() || label.chars().any(char::is_control) {
        return Err(invalid(
            "mindmap labels must be nonempty text without control characters",
        ));
    }
    let mut children = Vec::new();
    let mut append = |value: &Value| -> Result<()> {
        match value {
            Value::Mapping(map) => {
                for (key, value) in map {
                    children.push(node(key.as_str().unwrap(), value, depth + 1, count)?);
                }
            }
            Value::String(s) => children.push(node(s, &Value::Null, depth + 1, count)?),
            Value::Bool(b) => children.push(node(&b.to_string(), &Value::Null, depth + 1, count)?),
            Value::Number(n) => {
                children.push(node(&n.to_string(), &Value::Null, depth + 1, count)?)
            }
            _ => {
                return Err(invalid(
                    "mindmap list items must be labels or mappings; nested lists and null items are unsupported",
                ));
            }
        }
        Ok(())
    };
    match value {
        Value::Null => {}
        Value::Sequence(values) => {
            for value in values {
                append(value)?;
            }
        }
        value => append(value)?,
    }
    // At 13px in the bundled monospace font, nine units per UTF-8 byte is
    // deliberately conservative, including wide Unicode and fallback glyphs.
    let width = label.len().max(3) as f64 * 9.0 + 24.0;
    Ok(Node {
        label,
        children,
        width,
        y: 0.0,
    })
}

fn position(node: &mut Node, depth: usize, widths: &mut Vec<f64>, row: &mut usize) {
    if widths.len() <= depth {
        widths.push(0.0);
    }
    widths[depth] = widths[depth].max(node.width);
    for child in &mut node.children {
        position(child, depth + 1, widths, row);
    }
    node.y = if node.children.is_empty() {
        let y = MARGIN + HEIGHT / 2.0 + *row as f64 * ROW;
        *row += 1;
        y
    } else {
        (node.children.first().unwrap().y + node.children.last().unwrap().y) / 2.0
    };
}

fn draw(node: &Node, depth: usize, columns: &[f64], edges: &mut String, boxes: &mut String) {
    let x = columns[depth];
    for child in &node.children {
        let start = x + node.width;
        let end = columns[depth + 1];
        let mid = (start + end) / 2.0;
        edges.push_str(&format!(
            "<path d=\"M {start} {} C {mid} {}, {mid} {}, {end} {}\"/>",
            node.y, node.y, child.y, child.y
        ));
        draw(child, depth + 1, columns, edges, boxes);
    }
    boxes.push_str(&format!(
        "<g class=\"mindmap-node\" data-depth=\"{depth}\"><rect x=\"{x}\" y=\"{}\" width=\"{}\" height=\"{HEIGHT}\" rx=\"7\"/><text x=\"{}\" y=\"{}\" dominant-baseline=\"central\" text-anchor=\"middle\">{}</text></g>",
        node.y - HEIGHT / 2.0, node.width, x + node.width / 2.0, node.y, escape(&node.label)
    ));
}

fn outline(node: &Node, html: &mut String) {
    html.push_str(&format!("<li>{}", escape(&node.label)));
    if !node.children.is_empty() {
        html.push_str("<ul>");
        for child in &node.children {
            outline(child, html);
        }
        html.push_str("</ul>");
    }
    html.push_str("</li>");
}

pub fn render(source: &str, ordinal: usize) -> Result<String> {
    render_inner(source, ordinal).map_err(|e| invalid(format!("mindmap {ordinal}: {e}")))
}

fn render_inner(source: &str, ordinal: usize) -> Result<String> {
    if source.len() > 65_536 {
        return Err(invalid("mindmap source exceeds 64 KiB"));
    }
    let mut remaining = MAX_NODES * 4;
    let value = BoundedValue {
        remaining: &mut remaining,
        depth: 0,
    }
    .deserialize(serde_yaml_ng::Deserializer::from_str(source))
    .map_err(|e| invalid(format!("invalid YAML: {e}")))?;
    let Some(map) = value.as_mapping().filter(|m| m.len() == 1) else {
        return Err(invalid(
            "a mindmap starts with exactly one root, written as one YAML key",
        ));
    };
    let (label, value) = map.iter().next().unwrap();
    let mut count = 0;
    let mut root = node(label.as_str().unwrap(), value, 0, &mut count)?;
    let mut widths = Vec::new();
    let mut rows = 0;
    position(&mut root, 0, &mut widths, &mut rows);
    let mut width = MARGIN;
    let columns: Vec<_> = widths
        .iter()
        .map(|w| {
            let x = width;
            width += w + GAP;
            x
        })
        .collect();
    width += MARGIN - GAP;
    let height = (rows - 1) as f64 * ROW + HEIGHT + MARGIN * 2.0;
    let mut edges = String::new();
    let mut boxes = String::new();
    draw(&root, 0, &columns, &mut edges, &mut boxes);
    let mut text = String::new();
    outline(&root, &mut text);
    let label = escape(&root.label);
    Ok(format!(
        "<figure class=\"mindmap\"><div class=\"mindmap-scroll\" tabindex=\"0\" role=\"group\" aria-label=\"Mindmap diagram: {label}\"><svg class=\"mindmap-svg\" xmlns=\"http://www.w3.org/2000/svg\" width=\"{width}\" height=\"{height}\" viewBox=\"0 0 {width} {height}\" role=\"img\" aria-labelledby=\"_zel-mindmap-{ordinal}-title\" aria-describedby=\"_zel-mindmap-{ordinal}-desc\"><title id=\"_zel-mindmap-{ordinal}-title\">Mindmap: {label}</title><desc id=\"_zel-mindmap-{ordinal}-desc\">Hierarchy of {count} nodes. A text outline follows the diagram.</desc><g class=\"mindmap-edges\">{edges}</g><g class=\"mindmap-nodes\">{boxes}</g></svg></div><details class=\"mindmap-outline\"><summary>Text outline: {label}</summary><ul>{text}</ul></details></figure>"
    ))
}
