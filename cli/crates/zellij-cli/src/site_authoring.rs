//! Native, opt-in site creation. Answers are collected before any files are written.
use crate::{
    args::{NewDocsArgs, NewPortableArgs},
    authoring::Prompt,
};
use camino::Utf8Path;
use std::io::{self, BufRead, Write};
use zellij_core::{Error, Result, files, schema};
use zellij_scaffold::{DocsOptions, PortableOptions, Scaffold};

struct Options<'a> {
    portable: bool,
    dir: &'a str,
    name: Option<&'a str>,
    theme: Option<&'a str>,
    interactive: bool,
    minimal: bool,
    full: bool,
    guide: bool,
    force: bool,
}
pub(crate) fn portable(args: &NewPortableArgs) -> Result<()> {
    run(Options {
        portable: true,
        dir: &args.dir,
        name: args.name.as_deref(),
        theme: args.theme.as_deref(),
        interactive: args.interactive,
        minimal: args.minimal,
        full: false,
        guide: false,
        force: false,
    })
}
pub(crate) fn docs(args: &NewDocsArgs) -> Result<()> {
    run(Options {
        portable: false,
        dir: &args.dir,
        name: args.name.as_deref(),
        theme: args.theme.as_deref(),
        interactive: args.interactive,
        minimal: args.minimal,
        full: args.full,
        guide: args.guide,
        force: args.force,
    })
}
fn valid_name(name: &str) -> Result<()> {
    if name.trim().is_empty() || name.chars().any(char::is_control) {
        return Err(Error::InvalidArgument(
            "site name must be nonempty text without control characters".into(),
        ));
    }
    Ok(())
}
fn valid_theme(theme: &str) -> Result<()> {
    if !schema::themes().iter().any(|t| t.name == theme) {
        return Err(Error::InvalidArgument(format!(
            "unknown theme {theme:?}; run zel list themes"
        )));
    }
    Ok(())
}
fn choose<R: BufRead, W: Write>(
    prompt: &mut Prompt<R, W>,
    label: &str,
    choices: &[(&str, &str)],
    default: &str,
) -> Result<String> {
    for (i, (name, description)) in choices.iter().enumerate() {
        prompt.message(&format!("  {}. {name} — {description}", i + 1))?;
    }
    loop {
        let Some(answer) = prompt.ask(label, Some(default))? else {
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
        prompt.problem(&Error::InvalidArgument(
            "choose a name or number from the list".into(),
        ))?;
    }
}
fn plan<R: BufRead, W: Write>(
    options: &Options<'_>,
    default_name: &str,
    prompt: &mut Prompt<R, W>,
) -> Result<Scaffold> {
    let mut name = options.name.unwrap_or(default_name).to_owned();
    let mut theme = options
        .theme
        .unwrap_or(if options.portable {
            "cupertino"
        } else {
            "paper"
        })
        .to_owned();
    // Supplied invalid values fail before reading input, never silently replaced.
    valid_name(&name)?;
    valid_theme(&theme)?;
    let mut minimal = options.minimal;
    let mut full = options.full;
    let mut guide = options.guide;
    if options.interactive {
        prompt.message("Press Enter to keep a displayed default; :q cancels. Values supplied by flags are retained.")?;
        if options.name.is_none() {
            while let Some(answer) = prompt.ask("Site name", Some(&name))? {
                match valid_name(&answer) {
                    Ok(()) => {
                        name = answer;
                        break;
                    }
                    Err(error) => prompt.problem(&error)?,
                }
            }
        }
        if options.theme.is_none() {
            let themes: Vec<_> = schema::themes()
                .iter()
                .map(|t| (t.name.as_str(), t.description.as_str()))
                .collect();
            theme = choose(prompt, "Theme (name or number)", &themes, &theme)?;
        }
        if !minimal && !full {
            let choices: &[(&str, &str)] = if options.portable {
                &[
                    (
                        "standard",
                        "Home page, getting-started guide and local image",
                    ),
                    ("minimal", "One Markdown page"),
                ]
            } else {
                &[
                    ("standard", "Landing page with hero and features"),
                    ("minimal", "Landing page with a hero"),
                    ("full", "Catalogue with every section type and local media"),
                ]
            };
            let starter = choose(prompt, "Starter (name or number)", choices, "standard")?;
            minimal = starter == "minimal";
            full = starter == "full";
        }
        if !options.portable && !guide {
            guide = choose(
                prompt,
                "Include a guide?",
                &[
                    ("no", "Landing content only"),
                    ("yes", "Also create a guide folder and navigation"),
                ],
                "no",
            )? == "yes";
        }
    }
    if options.portable {
        zellij_scaffold::portable(&PortableOptions {
            name,
            theme,
            minimal,
        })
    } else {
        let settings = DocsOptions {
            name,
            theme,
            guide,
            minimal,
        };
        if full {
            zellij_scaffold::configured_full_docs(&settings)
        } else {
            Ok(zellij_scaffold::configured_docs(&settings))
        }
    }
}
fn run(options: Options<'_>) -> Result<()> {
    let target = files::absolute_path(Utf8Path::new(options.dir))?;
    let destination_error = |error| match error {
        // Portable creation has no --force option; retain its existing diagnostic.
        Error::NotEmpty { path } if options.portable => Error::FileExists { path },
        other => other,
    };
    // Check the root and common configuration path before prompting. The final
    // plan still checks every file and repeats the checks at write time.
    Scaffold::new()
        .add("zel.yaml", "")
        .preflight(&target, options.force)
        .map_err(destination_error)?;
    let stdin = io::stdin();
    let stderr = io::stderr();
    let mut prompt = Prompt::new(
        stdin.lock(),
        stderr.lock(),
        "site creation cancelled; no site files were written",
    );
    if options.interactive {
        prompt.message(&format!(
            "Create {} content in {}",
            if options.portable {
                "portable Markdown"
            } else {
                "hosted YAML/MDX"
            },
            zellij_core::reporting::plain(target.as_str())
        ))?;
    }
    let scaffold = plan(
        &options,
        target.file_name().unwrap_or(if options.portable {
            "Documentation"
        } else {
            "Docs"
        }),
        &mut prompt,
    )?;
    let written = scaffold
        .write(&target, options.force)
        .map_err(destination_error)?;
    drop(prompt);
    zellij_core::statusln!("✓ {target}");
    for path in written {
        zellij_core::statusln!("  {path}");
    }
    // Suggestions use fixed relative paths even when the destination contains shell syntax.
    zellij_core::statusln!("\n  From inside the created folder:");
    if options.portable {
        zellij_core::statusln!("    zel check . --strict");
        zellij_core::statusln!("    zel build .");
        zellij_core::statusln!("    zel view . --watch");
    } else {
        zellij_core::statusln!("    zel dev .");
    }
    Ok(())
}
