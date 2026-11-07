//! Optional line-oriented wizard. Prompts go to stderr; YAML is written only
//! after all answers validate. Explicit --interactive also accepts piped input.
use crate::args::NewSectionArgs;
use camino::{Utf8Path, Utf8PathBuf};
use serde_json::Value;
use std::{
    collections::BTreeSet,
    io::{self, BufRead, IsTerminal, Write},
};
use zellij_core::{Error, Result, files, schema};
use zellij_scaffold::authoring::Section;

fn invalid(message: impl Into<String>) -> Error {
    Error::InvalidArgument(message.into())
}
fn assignment(input: &str) -> Result<(&str, &str)> {
    input
        .split_once('=')
        .filter(|(key, _)| !key.is_empty())
        .ok_or_else(|| invalid("expected FIELD=VALUE"))
}
fn output(path: &str) -> Result<(Utf8PathBuf, String)> {
    let path = Utf8Path::new(path);
    let name = path
        .file_name()
        .filter(|s| !s.is_empty())
        .ok_or_else(|| files::unsafe_path(path, "expected a filename"))?;
    let parent = path
        .parent()
        .filter(|s| !s.as_str().is_empty())
        .unwrap_or(Utf8Path::new("."));
    files::preflight(parent, &[Utf8Path::new(name)], false)?;
    Ok((parent.to_owned(), name.into()))
}
pub(crate) struct Prompt<R, W> {
    input: R,
    output: W,
    cancellation: &'static str,
}
impl<R: BufRead, W: Write> Prompt<R, W> {
    pub(crate) fn new(input: R, output: W, cancellation: &'static str) -> Self {
        Self {
            input,
            output,
            cancellation,
        }
    }
    pub(crate) fn message(&mut self, text: &str) -> Result<()> {
        writeln!(self.output, "{text}")?;
        Ok(())
    }
    pub(crate) fn ask(&mut self, label: &str, default: Option<&str>) -> Result<Option<String>> {
        write!(self.output, "{label}")?;
        if let Some(default) = default {
            write!(
                self.output,
                " [{}]",
                zellij_core::reporting::plain(default).replace(['\n', '\r'], " ")
            )?;
        }
        write!(self.output, ": ")?;
        self.output.flush()?;
        let mut answer = String::new();
        if self.input.read_line(&mut answer)? == 0 {
            return Err(invalid(format!("{} (end of input)", self.cancellation)));
        }
        let answer = answer.trim_end_matches(['\r', '\n']);
        if answer == ":q" {
            return Err(invalid(self.cancellation));
        }
        Ok((!answer.is_empty()).then(|| answer.to_owned()))
    }
    pub(crate) fn problem(&mut self, error: &Error) -> Result<()> {
        writeln!(self.output, "{error}")?;
        Ok(())
    }
    pub(crate) fn choose(
        &mut self,
        label: &str,
        choices: &[(&str, &str)],
        default: &str,
    ) -> Result<String> {
        for (i, (name, description)) in choices.iter().enumerate() {
            self.message(&format!("  {}. {name} — {description}", i + 1))?;
        }
        loop {
            let Some(answer) = self.ask(label, Some(default))? else {
                return Ok(default.into());
            };
            let answer = answer.trim();
            if let Some((name, _)) = choices.iter().find(|(name, _)| *name == answer) {
                return Ok((*name).into());
            }
            if let Some((name, _)) = answer
                .parse::<usize>()
                .ok()
                .and_then(|n| n.checked_sub(1))
                .and_then(|i| choices.get(i))
            {
                return Ok((*name).into());
            }
            self.problem(&invalid("choose a name or number from the list"))?;
        }
    }
    fn kind(&mut self) -> Result<String> {
        let types = schema::section_types();
        for (i, kind) in types.iter().enumerate() {
            writeln!(self.output, "  {}. {kind}", i + 1)?;
        }
        loop {
            let answer = self
                .ask("Section type (name or number)", None)?
                .unwrap_or_default();
            let kind = answer
                .parse::<usize>()
                .ok()
                .and_then(|n| n.checked_sub(1))
                .and_then(|i| types.get(i))
                .unwrap_or(&answer);
            if schema::is_section_type(kind) {
                return Ok(kind.clone());
            }
            self.problem(&invalid("choose a section type from the list"))?;
        }
    }
    fn fields(
        &mut self,
        section: &mut Section,
        path: &str,
        fields: &BTreeSet<String>,
        counts: &BTreeSet<String>,
    ) -> Result<()> {
        let value = at(section.value(), path)
            .cloned()
            .ok_or_else(|| invalid("missing authoring field"))?;
        match value {
            Value::Object(object) => {
                for key in object
                    .keys()
                    .filter(|k| k.as_str() != "type" && k.as_str() != "highlighted")
                {
                    self.fields(
                        section,
                        &if path.is_empty() {
                            key.clone()
                        } else {
                            format!("{path}.{key}")
                        },
                        fields,
                        counts,
                    )?;
                }
            }
            Value::Array(items) => {
                if !counts.contains(path) && !section.aligned_cells(path) {
                    loop {
                        let (min, max) = section.count_bounds(path)?;
                        let count = self.ask(
                            &format!("Number of {path} ({min}–{max})"),
                            Some(&items.len().to_string()),
                        )?;
                        let Some(count) = count else {
                            break;
                        };
                        match count
                            .parse::<usize>()
                            .map_err(|_| invalid("enter a whole item count"))
                            .and_then(|n| {
                                let mut candidate = section.clone();
                                candidate.count(path, n)?;
                                if fields.iter().any(|key| {
                                    at(candidate.value(), key) != at(section.value(), key)
                                }) || counts.iter().any(|key| {
                                    at(candidate.value(), key)
                                        .and_then(Value::as_array)
                                        .map(Vec::len)
                                        != at(section.value(), key)
                                            .and_then(Value::as_array)
                                            .map(Vec::len)
                                }) {
                                    return Err(invalid(
                                        "that count would discard a value supplied by a flag",
                                    ));
                                }
                                *section = candidate;
                                Ok(())
                            }) {
                            Ok(()) => break,
                            Err(error) => self.problem(&error)?,
                        }
                    }
                }
                let size = at(section.value(), path).unwrap().as_array().unwrap().len();
                for i in 0..size {
                    self.fields(section, &format!("{path}.{i}"), fields, counts)?;
                }
            }
            _ if fields.contains(path) => {}
            value => {
                let default = value
                    .as_str()
                    .map(str::to_owned)
                    .unwrap_or_else(|| value.to_string());
                while let Some(answer) = self.ask(path, Some(&default))? {
                    match section.set(path, &answer) {
                        Ok(()) => break,
                        Err(error) => self.problem(&error)?,
                    }
                }
            }
        }
        Ok(())
    }
}
fn at<'a>(mut value: &'a Value, path: &str) -> Option<&'a Value> {
    if path.is_empty() {
        return Some(value);
    }
    for part in path.split('.') {
        value = if value.is_array() {
            value.get(part.parse::<usize>().ok()?)?
        } else {
            value.get(part)?
        };
    }
    Some(value)
}
fn prepare<R: BufRead, W: Write>(
    args: &NewSectionArgs,
    interactive: bool,
    prompt: &mut Prompt<R, W>,
) -> Result<(String, Option<String>, String)> {
    let mut destination = args.out.clone().or_else(|| args.name.clone());
    if let Some(path) = &destination {
        output(path)?;
    }
    let kind = match &args.section_type {
        Some(kind) => kind.clone(),
        None if interactive => prompt.kind()?,
        None => {
            return Err(invalid(
                "provide a section type or use --interactive; run zel list sections",
            ));
        }
    };
    let mut section = Section::new(&kind)?;
    if kind != "pricing-table" && args.highlight.is_some() {
        return Err(invalid("--highlight requires pricing-table"));
    }
    let mut counts = BTreeSet::new();
    for input in &args.counts {
        let (path, count) = assignment(input)?;
        if !counts.insert(path.to_owned()) {
            return Err(invalid(format!("repeated item count for {path}")));
        }
        section.count(
            path,
            count
                .parse()
                .map_err(|_| invalid("item count must be a whole number"))?,
        )?;
    }
    if let Some(count) = args.tiers {
        if kind != "pricing-table" {
            return Err(invalid("--tiers requires pricing-table"));
        }
        if !counts.insert("tiers".into()) {
            return Err(invalid("--tiers conflicts with --count tiers=N"));
        }
        section.count("tiers", count)?;
    }
    let mut fields = BTreeSet::new();
    for input in &args.fields {
        let (path, text) = assignment(input)?;
        if !fields.insert(path.to_owned()) {
            return Err(invalid(format!("repeated field assignment for {path}")));
        }
        section.set(path, text)?;
    }
    for input in &args.fields {
        let (path, text) = assignment(input)?;
        let matches = match at(section.value(), path) {
            Some(Value::String(value)) => value == text,
            Some(value) => serde_json::from_str::<Value>(text).ok().as_ref() == Some(value),
            None => false,
        };
        if !matches {
            return Err(invalid(format!(
                "conflicting field assignments remove or change {path}"
            )));
        }
    }
    for input in &args.counts {
        let (path, count) = assignment(input)?;
        if at(section.value(), path)
            .and_then(Value::as_array)
            .map(Vec::len)
            != count.parse::<usize>().ok()
        {
            return Err(invalid(format!("conflicting item counts for {path}")));
        }
    }
    if args.highlight.is_some()
        && fields
            .iter()
            .any(|p| p.starts_with("tiers.") && p.ends_with(".highlighted"))
    {
        return Err(invalid(
            "--highlight conflicts with explicit tier highlighted fields",
        ));
    }
    section.validate()?;
    if interactive {
        writeln!(
            prompt.output,
            "Press Enter to keep a starter value; :q cancels. Values supplied by flags are retained."
        )?;
        prompt.fields(&mut section, "", &fields, &counts)?;
    }
    if let Some(highlight) = &args.highlight {
        section.highlight(highlight)?;
    } else if interactive
        && kind == "pricing-table"
        && !fields
            .iter()
            .any(|s| s.starts_with("tiers.") && s.ends_with(".highlighted"))
    {
        loop {
            let answer = prompt
                .ask(
                    "Highlight tier (name, 1-based number, or none)",
                    Some("none"),
                )?
                .unwrap_or_else(|| "none".into());
            match section.highlight(&answer) {
                Ok(()) => break,
                Err(error) => prompt.problem(&error)?,
            }
        }
    }
    if interactive && destination.is_none() {
        loop {
            let answer = prompt.ask("Output file (Enter prints YAML)", None)?;
            match answer {
                None => break,
                Some(path) => match output(&path) {
                    Ok(_) => {
                        destination = Some(path);
                        break;
                    }
                    Err(error) => prompt.problem(&error)?,
                },
            }
        }
    }
    Ok((section.render()?, destination, kind))
}
pub(crate) fn run(args: &NewSectionArgs) -> Result<()> {
    let stdin = io::stdin();
    let stderr = io::stderr();
    let interactive = args.interactive
        || (args.section_type.is_none() && stdin.is_terminal() && stderr.is_terminal());
    let (body, destination, kind) = prepare(
        args,
        interactive,
        &mut Prompt::new(
            stdin.lock(),
            stderr.lock(),
            "section authoring cancelled; no section was written",
        ),
    )?;
    if let Some(path) = destination {
        let (parent, name) = output(&path)?;
        files::write_files(&parent, &[(Utf8Path::new(&name), &body)], false)?;
        zellij_core::statusln!("✓ {path}");
        zellij_core::statusln!("  zel explain {kind} — every field it takes");
    } else {
        io::stdout().lock().write_all(body.as_bytes())?;
    }
    Ok(())
}
