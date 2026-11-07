//! Collect wrapper settings before invoking the existing native scaffold writer.
use crate::{app, args::NewAppArgs, authoring::Prompt};
use camino::Utf8Path;
use std::io;
use zellij_core::{Error, Result, files, site::SiteInput};

fn valid_name(name: &str) -> Result<()> {
    if name.trim().is_empty() || name.chars().any(char::is_control) {
        return Err(Error::InvalidArgument(
            "app name must be nonempty text without control characters".into(),
        ));
    }
    Ok(())
}

pub(crate) fn plan(options: &NewAppArgs) -> Result<NewAppArgs> {
    let out = files::absolute_path(Utf8Path::new(&options.dir))?;
    zellij_scaffold::Scaffold::new()
        .add("package.json", "")
        .preflight(&out, false)
        .map_err(app::destination_error)?;
    // Explicit values fail before questions, rather than being replaced by defaults.
    if let Some(name) = &options.name {
        valid_name(name)?;
    }
    app::selected_features(&options.features)?;
    if let Some(engine) = &options.engine {
        app::resolve_engine(engine, &out)?;
    }
    let mut default_port = if options.site_root.is_some() {
        app::resolve_source(options.site_root.as_ref(), &out)?.1
    } else {
        3000
    };
    let mut planned = options.clone();
    planned.interactive = false;
    let stdin = io::stdin();
    let stderr = io::stderr();
    let mut prompt = Prompt::new(
        stdin.lock(),
        stderr.lock(),
        "app creation cancelled; no wrapper files were written",
    );
    prompt.message("Create a Next.js wrapper. Enter keeps a displayed default; :q cancels. Supplied values skip their questions.")?;
    if options.engine.is_none() {
        loop {
            let answer = prompt
                .ask("Built engine package", Some("engine/packages/zellij"))?
                .unwrap_or_else(|| "engine/packages/zellij".into());
            match app::resolve_engine(&answer, &out) {
                Ok(engine) => {
                    planned.engine = Some(engine.into_string());
                    break;
                }
                Err(error) => prompt.problem(&error)?,
            }
        }
    }
    if !options.with_docs && options.site_root.is_none() {
        planned.with_docs = prompt.choose(
            "Content",
            &[
                ("existing", "Reference an existing hosted site"),
                ("starter", "Create hosted docs inside the wrapper"),
            ],
            "existing",
        )? == "starter";
        if !planned.with_docs {
            let default = crate::select_site(None)
                .map(|root| root.root().to_string())
                .unwrap_or_else(|_| "docs/zellij".into());
            loop {
                let answer = prompt
                    .ask("Hosted site folder", Some(&default))?
                    .unwrap_or_else(|| default.clone());
                let input = SiteInput::from(answer.as_str());
                match app::resolve_source(Some(&input), &out) {
                    Ok((_, port)) => {
                        planned.site_root = Some(input);
                        default_port = port;
                        break;
                    }
                    Err(error) => prompt.problem(&error)?,
                }
            }
        }
    }
    if options.name.is_none() {
        let default = out.file_name().unwrap_or("Zellij app");
        loop {
            let answer = prompt
                .ask("App name", Some(default))?
                .unwrap_or_else(|| default.into());
            match valid_name(&answer) {
                Ok(()) => {
                    planned.name = Some(answer);
                    break;
                }
                Err(error) => prompt.problem(&error)?,
            }
        }
    }
    if options.port.is_none() {
        loop {
            let answer = prompt
                .ask("Dev/start port", Some(&default_port.to_string()))?
                .unwrap_or_else(|| default_port.to_string());
            match answer.trim().parse::<u16>() {
                Ok(port) if port != 0 => {
                    planned.port = Some(port);
                    break;
                }
                _ => prompt.problem(&Error::InvalidArgument(
                    "app port must be between 1 and 65535".into(),
                ))?,
            }
        }
    }
    if options.features.is_empty() {
        planned.features = vec![prompt.choose(
            "Optional features",
            &[
                ("none", "No optional packages"),
                ("maths", "Math rendering"),
                ("diagrams", "Mermaid diagrams"),
                ("all", "Maths and diagrams"),
            ],
            "none",
        )?];
    }
    Ok(planned)
}
