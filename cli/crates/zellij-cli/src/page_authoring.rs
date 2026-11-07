//! Native page planning. Registration remains an explicit pages.yaml edit.
use crate::{args::NewPageArgs, authoring::Prompt};
use camino::Utf8Path;
use std::{collections::BTreeSet, io};
use zellij_core::{Error, Result, files, schema};

fn invalid(message: impl Into<String>) -> Error {
    Error::InvalidArgument(message.into())
}

fn validate_sections(sections: &[String]) -> Result<()> {
    let mut seen = BTreeSet::new();
    for kind in sections {
        if !schema::is_section_type(kind) {
            return Err(invalid(format!(
                "no section type called {kind:?}; run zel list sections"
            )));
        }
        if !seen.insert(kind) {
            return Err(invalid(format!(
                "section type {kind:?} is repeated; each type creates one file per page"
            )));
        }
    }
    Ok(())
}

fn directory(root: &Utf8Path, relative: &str) -> Result<()> {
    let path = files::contained_path(root, Utf8Path::new(relative))?;
    if path.exists() && !path.is_dir() {
        return Err(files::unsafe_path(&path, "expected a section directory"));
    }
    Ok(())
}

fn name_destination(root: &Utf8Path, name: &str) -> Result<()> {
    files::relative_path(Utf8Path::new(name))?;
    directory(root, &format!("sections/{name}"))
}

fn section_answers(answer: &str) -> Result<Vec<String>> {
    let types = schema::section_types();
    let sections = answer
        .split(',')
        .map(|value| {
            let value = value.trim();
            value
                .parse::<usize>()
                .ok()
                .and_then(|n| n.checked_sub(1))
                .and_then(|i| types.get(i))
                .map_or_else(|| value.to_owned(), Clone::clone)
        })
        .collect::<Vec<_>>();
    validate_sections(&sections)?;
    Ok(sections)
}

pub(crate) fn run(options: &NewPageArgs) -> Result<()> {
    // Invalid supplied selections fail before any question or write.
    validate_sections(&options.sections)?;
    let root = crate::authoring_content(options.dir.as_ref())?;
    directory(&root, "sections")?;
    if let Some(name) = &options.name {
        name_destination(&root, name)?;
    }
    let mut name = options.name.clone();
    let mut sections = options.sections.clone();
    if options.interactive {
        let stdin = io::stdin();
        let stderr = io::stderr();
        let mut prompt = Prompt::new(
            stdin.lock(),
            stderr.lock(),
            "page creation cancelled; no section files were written",
        );
        prompt.message(
            "Create hosted page sections. :q cancels; supplied values skip their questions.",
        )?;
        if name.is_none() {
            loop {
                let answer = prompt.ask("Page name", None)?.unwrap_or_default();
                let result = if answer.trim().is_empty() || answer.chars().any(char::is_control) {
                    Err(invalid(
                        "page name must be nonempty text without control characters",
                    ))
                } else {
                    name_destination(&root, &answer)
                };
                match result {
                    Ok(()) => {
                        name = Some(answer);
                        break;
                    }
                    Err(error) => prompt.problem(&error)?,
                }
            }
        }
        if sections.is_empty() {
            for (index, kind) in schema::section_types().iter().enumerate() {
                prompt.message(&format!("  {}. {kind}", index + 1))?;
            }
            loop {
                let answer = prompt
                    .ask(
                        "Sections (comma-separated names or numbers, in page order)",
                        Some("hero"),
                    )?
                    .unwrap_or_else(|| "hero".into());
                match section_answers(&answer) {
                    Ok(value) => {
                        sections = value;
                        break;
                    }
                    Err(error) => prompt.problem(&error)?,
                }
            }
        }
    }
    let name =
        name.ok_or_else(|| invalid("page name is required; pass a name or use --interactive"))?;
    let (scaffold, entry) = zellij_scaffold::page(&name, &sections);
    // The writer checks the entire plan before creating any file, including
    // collisions later in the list. It never overwrites authored sections.
    for path in scaffold.write_new(&root)? {
        zellij_core::statusln!("✓ {path}");
    }
    // Keep the existing report and preserve authored pages.yaml comments/order.
    zellij_core::reportln!("\n  Add this to pages.yaml:\n");
    for line in entry.lines() {
        zellij_core::reportln!("    {line}");
    }
    Ok(())
}
