use camino::Utf8Path;
use zellij_core::{Error, Result};

pub fn check(
    dir: Option<&str>,
    target: Option<&str>,
    options: zellij_engine::CheckOptions,
    as_json: bool,
) -> Result<()> {
    if as_json && target == Some("static") {
        return Err(Error::InvalidArgument("--json is supported only for portable checks; hosted validation does not yet provide structured reports; no command was run".into()));
    }
    let selected = crate::select_site(dir)?;
    let dir = selected.as_str();
    // Invalid configuration is reported through the native check's structured
    // diagnostics, preserving --json's one-report/no-stderr contract.
    let automatic = crate::default_target(dir).unwrap_or("portable");
    match target.unwrap_or(automatic) {
        "static" if as_json => return Err(Error::InvalidArgument(
            "--json is supported only for portable checks; hosted validation does not yet provide structured reports; no command was run".into(),
        )),
        "static" => return crate::check(dir, options),
        "portable" => {}
        _ => unreachable!("clap validates the check target"),
    }
    // Portable compilation always validates local references, so --deep is
    // accepted for shared CI commands without changing the validation coverage.
    let report = zellij_docs::check(Utf8Path::new(dir), options.strict);
    if as_json {
        let value = serde_json::to_value(&report).map_err(std::io::Error::other)?;
        crate::print_json(&value)?;
    } else {
        println!("{} {}", if report.ok { "✓" } else { "✗" }, report.root);
        for diagnostic in &report.diagnostics {
            eprintln!("{diagnostic}");
        }
        if let Some(pages) = report.pages {
            println!("  {pages} HTML pages checked; no files written");
        }
        if options.strict
            && !report.ok
            && report
                .diagnostics
                .iter()
                .all(|d| d.severity == zellij_docs::Severity::Warning)
        {
            eprintln!("error: warnings are rejected by --strict");
        }
    }
    // The full report already contains the failure. In JSON mode stderr stays
    // empty; don't append an unrelated human-only error through main's handler.
    if report.exit_code != 0 {
        std::process::exit(report.exit_code);
    }
    Ok(())
}
