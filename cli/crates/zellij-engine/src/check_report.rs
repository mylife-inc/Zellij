//! Versioned reports from the real engine. Do not infer findings from log text.
use camino::Utf8Path;
use serde::{Deserialize, Serialize};
use zellij_core::{Error, Result};

use crate::CheckOptions;

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Error,
    Warning,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct HostedDiagnostic {
    pub severity: Severity,
    pub code: String,
    pub file: Option<String>,
    pub message: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct HostedCheckReport {
    pub schema_version: u32,
    pub target: String,
    pub root: String,
    pub deep: bool,
    pub strict: bool,
    pub ok: bool,
    pub pages: Option<usize>,
    pub diagnostics: Vec<HostedDiagnostic>,
}

impl HostedCheckReport {
    pub(crate) fn decode(
        stdout: &[u8],
        status: Option<i32>,
        mounted: &str,
        content: &Utf8Path,
        options: CheckOptions,
    ) -> Result<Self> {
        let invalid = || {
            Error::CheckFailed(format!(
                "the engine did not return a compatible JSON report (exit {status:?}). Select a ZELLIJ_IMAGE with hosted JSON support and refresh it with `zel pull`; run a text check for the engine/runtime diagnostic"
            ))
        };
        let mut report: Self = serde_json::from_slice(stdout).map_err(|_| invalid())?;
        let expected_ok = !report
            .diagnostics
            .iter()
            .any(|d| d.severity == Severity::Error)
            && !(options.strict
                && report
                    .diagnostics
                    .iter()
                    .any(|d| d.severity == Severity::Warning));
        if report.schema_version != 1
            || report.target != "static"
            || report.root != mounted
            || report.deep != options.deep
            || report.strict != options.strict
            || report.ok != expected_ok
            || (report.ok && report.pages.is_none())
            || status != Some(if report.ok { 0 } else { 1 })
            || report.diagnostics.iter().any(|d| {
                d.code.is_empty()
                    || d.message.is_empty()
                    || d.file.as_ref().is_some_and(|file| {
                        file.is_empty()
                            || file.starts_with('/')
                            || file.contains('\\')
                            || file.split('/').any(|part| part == "..")
                    })
            })
        {
            return Err(invalid());
        }
        // Rewrite fields after parsing: host paths may contain JSON quotes,
        // backslashes and Unicode. Never replace inside encoded JSON bytes.
        report.root = content.to_string();
        for diagnostic in &mut report.diagnostics {
            diagnostic.message = diagnostic.message.replace(mounted, content.as_str());
        }
        Ok(report)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};

    fn fixture() -> Value {
        json!({"schema_version": 1, "target": "static", "root": "/content",
            "deep": true, "strict": true, "ok": true, "pages": 2, "diagnostics": []})
    }
    fn decode(value: &Value, status: Option<i32>) -> Result<HostedCheckReport> {
        HostedCheckReport::decode(
            &serde_json::to_vec(value).unwrap(),
            status,
            "/content",
            Utf8Path::new("C:\\docs \"日本\""),
            CheckOptions {
                deep: true,
                strict: true,
            },
        )
    }

    #[test]
    fn reports_keep_host_path_escaping_and_validation_failures() {
        assert_eq!(
            decode(&fixture(), Some(0)).unwrap().root,
            "C:\\docs \"日本\""
        );
        let mut invalid = fixture();
        invalid["ok"] = json!(false);
        invalid["diagnostics"] = json!([{"severity": "error", "code": "content",
            "file": "site.yaml", "message": "Invalid /content/site.yaml: expected a string"}]);
        let report = decode(&invalid, Some(1)).unwrap();
        assert_eq!(
            report.diagnostics[0].message,
            "Invalid C:\\docs \"日本\"/site.yaml: expected a string"
        );
        let encoded = serde_json::to_vec(&report).unwrap();
        assert_eq!(
            serde_json::from_slice::<Value>(&encoded).unwrap()["root"],
            "C:\\docs \"日本\""
        );
        invalid["diagnostics"][0]["severity"] = json!("warning");
        assert!(!decode(&invalid, Some(1)).unwrap().ok);
    }

    #[test]
    fn incompatible_or_inconsistent_engine_reports_cannot_claim_success() {
        for (field, value) in [
            ("schema_version", json!(2)),
            ("target", json!("portable")),
            ("root", json!("/other")),
            ("deep", json!(false)),
            ("strict", json!(false)),
            ("ok", json!(false)),
            ("pages", Value::Null),
            (
                "diagnostics",
                json!([{"severity":"error", "code":"deep", "file":null, "message":"bad"}]),
            ),
        ] {
            let mut wrong = fixture();
            wrong[field] = value;
            assert!(decode(&wrong, Some(0)).is_err(), "{field}");
        }
        for status in [Some(1), Some(125), None] {
            assert!(decode(&fixture(), status).is_err());
        }
        assert!(
            HostedCheckReport::decode(
                b"old engine: check passed",
                Some(0),
                "/content",
                Utf8Path::new("docs"),
                CheckOptions::default()
            )
            .is_err()
        );
    }
}
