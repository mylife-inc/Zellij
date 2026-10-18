//! Mermaid needs browser layout. Native compilation validates resource limits
//! and emits escaped, readable source; the local runtime validates grammar.
use crate::markdown::escape;
use zellij_core::{Error, Result};

pub const ASSETS: &[(&str, &[u8])] = &[
    ("mermaid.js", include_bytes!("../assets/mermaid/mermaid.js")),
    (
        "licenses.txt",
        include_bytes!("../assets/mermaid/licenses.txt"),
    ),
    (
        "manifest.json",
        include_bytes!("../assets/mermaid/manifest.json"),
    ),
    ("mermaid.css", include_bytes!("../assets/mermaid.css")),
];

pub fn render(source: &str, ordinal: usize) -> Result<String> {
    let problem = if ordinal > 32 {
        Some("at most 32 Mermaid diagrams are allowed per page")
    } else if source.len() > 65_536 {
        Some("source exceeds 64 KiB")
    } else if source.trim().is_empty() {
        Some("source must not be empty")
    } else {
        None
    };
    if let Some(problem) = problem {
        return Err(Error::InvalidArgument(format!(
            "mermaid {ordinal}: {problem}"
        )));
    }
    Ok(format!(
        "<figure class=\"mermaid-diagram\" data-status=\"pending\"><div class=\"mermaid-canvas mindmap-scroll\" hidden tabindex=\"0\" role=\"region\" aria-label=\"Mermaid diagram {ordinal}\"></div><details open><summary>Diagram {ordinal} source (Mermaid)</summary><pre tabindex=\"0\"><code>{}</code></pre></details><p role=\"status\"></p></figure>",
        escape(source)
    ))
}
