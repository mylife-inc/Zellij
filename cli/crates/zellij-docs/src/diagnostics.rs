//! Machine-readable validation uses the same compiler as publication.
use camino::{Utf8Path, Utf8PathBuf};
use serde::Serialize;
use zellij_core::{Error, Result};

#[derive(Debug, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Error,
    Warning,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Code {
    Source,
    Configuration,
    Markdown,
    Reference,
    OutputPath,
    Rendering,
    RawHtml,
}

#[derive(Debug, Serialize)]
pub struct Diagnostic {
    pub severity: Severity,
    pub code: Code,
    /// Source-relative filename when known; never a guessed line or column.
    pub file: Option<String>,
    pub message: String,
}

impl std::fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let severity = match self.severity {
            Severity::Error => "error",
            Severity::Warning => "warning",
        };
        write!(f, "{severity}: ")?;
        if let Some(file) = &self.file {
            write!(f, "{file}: ")?;
        }
        f.write_str(&self.message)
    }
}

#[derive(Debug, Serialize)]
pub struct CheckReport {
    pub schema_version: u32,
    pub target: &'static str,
    pub root: String,
    pub strict: bool,
    pub ok: bool,
    /// Generated HTML page count, including a synthetic home, on compilation success.
    pub pages: Option<usize>,
    pub diagnostics: Vec<Diagnostic>,
    #[serde(skip)]
    pub exit_code: i32,
}

pub(crate) struct Diagnostics {
    pub root: Utf8PathBuf,
    pub items: Vec<Diagnostic>,
}
impl Diagnostics {
    pub fn new(root: &Utf8Path) -> Self {
        Self {
            root: root.to_owned(),
            items: vec![],
        }
    }
    pub fn capture<T>(
        &mut self,
        result: Result<T>,
        code: Code,
        file: Option<&Utf8Path>,
    ) -> Result<T> {
        result.inspect_err(|error| {
            let inferred = match error {
                Error::UnsafePath { path, .. } if code == Code::Source => Some(path.as_path()),
                _ => None,
            };
            let file = file.or(inferred).and_then(|path| {
                let relative = if path.is_absolute() {
                    path.strip_prefix(&self.root).ok()?
                } else {
                    path
                };
                (!relative.as_str().is_empty()).then(|| crate::url_path(relative).to_string())
            });
            let message = match error {
                Error::UnsafePath { reason, .. } if code != Code::OutputPath => reason.clone(),
                _ => error.to_string(),
            };
            self.items.push(Diagnostic {
                severity: Severity::Error,
                code,
                file,
                message,
            });
        })
    }
    pub fn raw_html(&mut self, file: &Utf8Path) {
        self.items.push(Diagnostic {
            severity: Severity::Warning,
            code: Code::RawHtml,
            file: Some(file.to_string()),
            message:
                "raw HTML is displayed as text; portable Markdown does not execute HTML or MDX"
                    .into(),
        });
    }
    pub fn warnings(&self) -> Vec<String> {
        self.items
            .iter()
            .filter(|d| d.severity == Severity::Warning)
            .map(|d| {
                d.file
                    .as_ref()
                    .map_or_else(|| d.message.clone(), |f| format!("{f}: {}", d.message))
            })
            .collect()
    }
}
