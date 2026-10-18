//! Equations become MathML before publication; no browser typesetter is needed.
use math_core::{LatexToMathML, MathCoreConfig, MathDisplay};
use zellij_core::{Error, Result};

pub const ASSETS: &[(&str, &[u8])] = &[
    ("math.css", include_bytes!("../assets/math/math.css")),
    (
        "mathmlfixes.css",
        include_bytes!("../assets/math/mathmlfixes.css"),
    ),
    (
        "STIXTwoMath-Regular.woff2",
        include_bytes!("../assets/math/STIXTwoMath-Regular.woff2"),
    ),
    ("OFL.txt", include_bytes!("../assets/math/OFL.txt")),
    (
        "math-core-LICENSE",
        include_bytes!("../assets/math/math-core-LICENSE"),
    ),
];

pub fn render(source: &str, block: bool, ordinal: usize) -> Result<String> {
    let converter = LatexToMathML::new(
        MathCoreConfig {
            annotation: true,
            // Reserve a separate namespace for each snippet, including user labels.
            id_prefix: format!("_zel-math-{ordinal}-"),
            ..Default::default()
        },
        vec![],
    )
    .map_err(|(e, _, _)| Error::InvalidArgument(format!("math configuration: {e}")))?;
    let result = converter
        .convert_with_local_state(
            source,
            if block {
                MathDisplay::Block
            } else {
                MathDisplay::Inline
            },
        )
        .map_err(|e| Error::InvalidArgument(format!("math expression {ordinal}: {e}")))?;
    if result.warnings.has_any() {
        return Err(Error::InvalidArgument(format!(
            "math expression {ordinal}: unresolved equation reference or unknown command; equation references must resolve within the same expression"
        )));
    }
    if block {
        // A span remains valid when a display expression occurs in a paragraph.
        // The scrollable wrapper keeps wide equations usable on small screens.
        Ok(format!(
            "<span class=\"math-display\" tabindex=\"0\" role=\"group\" aria-label=\"Equation\">{}</span>",
            result.mathml
        ))
    } else {
        Ok(result.mathml)
    }
}
