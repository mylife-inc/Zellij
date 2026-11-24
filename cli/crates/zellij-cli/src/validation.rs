use zellij_core::site::SiteInput;
use zellij_core::{Error, Result};

pub fn check(
    dir: Option<&SiteInput>,
    target: Option<&str>,
    options: zellij_engine::CheckOptions,
    as_json: bool,
    offline: bool,
) -> Result<()> {
    let selected = crate::select_site(dir)?;
    let dir = &selected;
    // Invalid configuration is reported through the native check's structured
    // diagnostics, preserving --json's one-report/no-stderr contract.
    let automatic = crate::default_target(dir).unwrap_or(if offline { "file" } else { "portable" });
    let selected = target.unwrap_or(automatic);
    if offline && !matches!(selected, "file" | "static") {
        return Err(Error::InvalidArgument(
            "--offline applies to cached rich validation (--target file or static)".into(),
        ));
    }
    match selected {
        "file" => return hosted_report(dir, options, as_json, "file", offline),
        "static" if as_json => return hosted_report(dir, options, true, "static", offline),
        "static" => {
            let content = crate::resolve_content(dir)?;
            let report = crate::rich_build::check(&content, options, offline, None)?
                .ok_or_else(|| Error::InvalidArgument("rich validation cancelled".into()))?;
            emit_hosted(&report);
            // Composed import commands must retain control to explain that
            // converted sources survive a failed optional validation.
            return if report.ok {
                Ok(())
            } else {
                Err(Error::ContentInvalid)
            };
        }
        "portable" => {}
        _ => unreachable!("clap validates the check target"),
    }
    // Portable compilation always validates local references, so --deep is
    // accepted for shared CI commands without changing the validation coverage.
    let exit_code = portable(dir, options.strict, as_json)?;
    // One-shot CLI behavior; watch consumes the same report without exiting.
    if exit_code != 0 {
        std::process::exit(exit_code);
    }
    Ok(())
}

fn hosted_report(
    dir: &SiteInput,
    options: zellij_engine::CheckOptions,
    as_json: bool,
    target: &str,
    offline: bool,
) -> Result<()> {
    use zellij_engine::{HostedCheckReport, HostedDiagnostic, Severity};
    let mut root = dir.root().to_string();
    let result = (|| {
        let content = crate::resolve_content(dir)?;
        root = content.to_string();
        crate::rich_build::check(&content, options, offline, None)?
            .ok_or_else(|| Error::InvalidArgument("rich validation cancelled".into()))
    })();
    let (report, exit_code) = match result {
        Ok(mut report) => {
            report.target = target.into();
            let code = if report.ok { 0 } else { 3 };
            (report, code)
        }
        Err(error) => {
            let code = match &error {
                Error::NoEngine { .. } | Error::MountEmpty { .. } => "runtime",
                Error::InvalidArgument(_) | Error::UnsafePath { .. } => "configuration",
                _ => "engine",
            };
            (
                HostedCheckReport {
                    schema_version: 1,
                    target: target.into(),
                    root,
                    deep: options.deep,
                    strict: options.strict,
                    ok: false,
                    pages: None,
                    diagnostics: vec![HostedDiagnostic {
                        severity: Severity::Error,
                        code: code.into(),
                        file: None,
                        message: error.to_string(),
                    }],
                },
                error.exit_code(),
            )
        }
    };
    if as_json {
        crate::print_json(&serde_json::to_value(report).map_err(std::io::Error::other)?)?;
    } else {
        emit_hosted(&report);
    }
    if exit_code != 0 {
        std::process::exit(exit_code);
    }
    Ok(())
}

pub(crate) fn emit_hosted(report: &zellij_engine::HostedCheckReport) {
    zellij_core::statusln!("{} {}", if report.ok { "✓" } else { "✗" }, report.root);
    for d in &report.diagnostics {
        zellij_core::diagnosticln!(
            "{}: {}",
            if d.severity == zellij_engine::Severity::Error {
                "error"
            } else {
                "warning"
            },
            d.message
        );
    }
    if let Some(pages) = report.pages {
        zellij_core::statusln!("  {pages} routes checked; no site files written");
    }
    if report.strict
        && !report.ok
        && report
            .diagnostics
            .iter()
            .all(|d| d.severity == zellij_engine::Severity::Warning)
    {
        zellij_core::diagnosticln!("error: Strict check failed; warnings are rejected by --strict");
    }
}

pub fn portable(dir: &SiteInput, strict: bool, as_json: bool) -> Result<i32> {
    let report = zellij_docs::check(dir, strict);
    if as_json {
        let value = serde_json::to_value(&report).map_err(std::io::Error::other)?;
        crate::print_json(&value)?;
    } else {
        zellij_core::statusln!("{} {}", if report.ok { "✓" } else { "✗" }, report.root);
        for diagnostic in &report.diagnostics {
            zellij_core::diagnosticln!("{diagnostic}");
        }
        if let Some(pages) = report.pages {
            zellij_core::statusln!("  {pages} HTML pages checked; no files written");
        }
        if strict
            && !report.ok
            && report
                .diagnostics
                .iter()
                .all(|d| d.severity == zellij_docs::Severity::Warning)
        {
            zellij_core::diagnosticln!("error: warnings are rejected by --strict");
        }
    }
    // The full report already contains the failure. In JSON mode stderr stays
    // empty; don't append an unrelated human-only error through main's handler.
    Ok(report.exit_code)
}
