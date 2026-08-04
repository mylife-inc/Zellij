//! The Zellij command line.
//!
//! Named `zel` rather than `zellij`: the latter is already a widely used Rust
//! terminal multiplexer, and two binaries of the same name on one PATH is a
//! problem nobody should have to debug.
//!
//! Commands are declared here and implemented elsewhere — this file is about
//! what the tool offers and how it reports, never about how any of it works.
//! See `specs/zellij-cli-spec.md` for the surface in full.

use camino::{Utf8Path, Utf8PathBuf};
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "zel",
    version,
    about = "Turn a folder of content into a documentation site.",
    long_about = None,
)]
struct Cli {
    #[command(subcommand)]
    command: Command,

    /// Machine-readable output. Supported everywhere, because a tool only a
    /// human can read cannot be used by a pipeline.
    #[arg(long, global = true)]
    json: bool,

    #[arg(long, short, global = true)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Command {
    /// Create an app, a content folder, a section, a page or a guide.
    #[command(subcommand)]
    New(NewTarget),

    /// Show every field of a section type, with a worked example.
    Explain {
        /// A section type — `zel list sections` for all of them.
        section_type: String,
    },

    /// List what this build knows about.
    #[command(subcommand)]
    List(ListTarget),

    /// Write the reference an AI agent authors a content folder from.
    ///
    /// Needs neither Docker nor Node: generated from the engine's schemas at
    /// release time and compiled into this binary.
    Context {
        /// Where to write it. Defaults to standard output.
        file: Option<String>,
    },

    /// Say what tool a documentation folder was written for.
    Detect { dir: String },

    /// Convert documentation written for another tool into a new folder.
    Convert {
        source: String,

        /// Where to write it. Never the source — a conversion that overwrote
        /// the original would be unrecoverable.
        #[arg(long)]
        out: String,

        /// Force a format instead of detecting one.
        #[arg(long)]
        from: Option<String>,

        /// Report the plan, write nothing.
        #[arg(long)]
        dry_run: bool,
    },

    /// Validate a content folder.
    Check {
        #[arg(default_value = ".")]
        dir: String,
    },

    /// Development server with hot reload.
    Dev {
        #[arg(default_value = ".")]
        dir: String,

        #[arg(long, default_value_t = 3000)]
        port: u16,
    },

    /// Build the site.
    Build {
        #[arg(default_value = ".")]
        dir: String,

        /// What to produce — `zel list targets` for the differences.
        #[arg(long, default_value = "static")]
        target: String,

        #[arg(long)]
        out: Option<String>,

        /// Image tag. `--target image` only.
        #[arg(long)]
        tag: Option<String>,
    },

    /// Build the site and put it online.
    Deploy {
        /// netlify | cloudflare | vercel | ghpages | s3 | fly | railway.
        /// The provider decides the target: a CDN cannot run a container.
        provider: String,

        #[arg(default_value = ".")]
        dir: String,

        /// The site or project on the provider.
        #[arg(long)]
        site: Option<String>,
    },

    /// Build, run and push container images.
    Image {
        #[command(subcommand)]
        action: ImageAction,
    },

    /// Write a pipeline that checks, builds and deploys on push.
    Ci {
        #[command(subcommand)]
        action: CiAction,
    },

    /// Detect, convert and validate in one step.
    Migrate {
        source: String,

        #[arg(long)]
        out: String,
    },

    /// Check, build and deploy in one step.
    Publish {
        #[arg(default_value = ".")]
        dir: String,

        #[arg(long)]
        to: Option<String>,
    },

    /// Convert, check, build and deploy in one step.
    Ship {
        source: String,

        #[arg(long)]
        provider: Option<String>,
    },

    /// Serve what `build` produced.
    Serve {
        #[arg(default_value = "dist")]
        dir: String,

        #[arg(long, default_value_t = 3000)]
        port: u16,
    },

    /// Canonical formatting for the YAML.
    Fmt {
        #[arg(default_value = ".")]
        dir: String,

        /// Exit 1 if anything would change, and write nothing.
        #[arg(long)]
        check: bool,
    },

    /// Check the environment: container runtime, image, versions.
    Doctor,
}

#[derive(Subcommand)]
enum ImageAction {
    /// The same as `build --target image`.
    Build {
        #[arg(default_value = ".")]
        dir: String,

        #[arg(long)]
        tag: Option<String>,
    },
    /// Run it locally — the last check before it goes anywhere.
    Run {
        tag: Option<String>,

        #[arg(long, default_value_t = 3000)]
        port: u16,
    },
    /// Push it to a registry.
    Push {
        tag: String,

        #[arg(long)]
        registry: Option<String>,
    },
    /// Print the Dockerfile `image build` would have used.
    Dockerfile {
        #[arg(default_value = ".")]
        dir: String,
    },
}

#[derive(Subcommand)]
enum CiAction {
    /// Write a pipeline into the repository.
    Init {
        /// github | gitlab | circle.
        system: String,

        #[arg(long)]
        deploy: Option<String>,

        #[arg(long, default_value = "static")]
        target: String,
    },
}

#[derive(Subcommand)]
enum NewTarget {
    /// A Next.js wrapper app with the engine installed.
    App {
        #[arg(default_value = ".")]
        dir: String,

        /// Also write a starter content folder, so there is something to open.
        #[arg(long)]
        with_docs: bool,
    },
    /// A content folder on its own.
    Docs {
        #[arg(default_value = "content")]
        dir: String,

        /// The site name. Defaults to the directory's.
        #[arg(long)]
        name: Option<String>,

        /// One of the fourteen — `zel list themes`.
        #[arg(long, default_value = "paper")]
        theme: String,

        /// Also create a guide folder.
        #[arg(long)]
        guide: bool,

        /// site.yaml and one page, and nothing else.
        #[arg(long)]
        minimal: bool,

        /// Write into a directory that is not empty.
        #[arg(long)]
        force: bool,
    },
    /// A section file, with its required fields filled in.
    Section {
        section_type: String,

        #[arg(long)]
        out: Option<String>,
    },
    /// A page, and the section files it references.
    Page {
        name: String,

        /// Comma-separated section types. Defaults to a single hero.
        #[arg(long, value_delimiter = ',')]
        sections: Vec<String>,

        #[arg(long, default_value = ".")]
        dir: String,
    },
    /// A guide folder with a sidebar and an index page.
    Guide {
        name: String,

        #[arg(long)]
        title: Option<String>,

        #[arg(long, default_value = ".")]
        dir: String,
    },
    /// A repository with documentation and a product as siblings, and a
    /// pipeline that publishes the first on every push.
    Project {
        name: String,

        /// What to put in the product folder: rust | go | java | kotlin |
        /// python | node | none.
        #[arg(long, default_value = "none")]
        product_lang: String,

        #[arg(long, default_value = "docs")]
        docs_dir: String,

        #[arg(long, default_value = "product")]
        product_dir: String,
    },
}

#[derive(Subcommand)]
enum ListTarget {
    /// Every section type.
    Sections,
    /// Every theme.
    Themes,
    /// Every documentation format `convert` recognises.
    Formats,
    /// Everything `build` can produce.
    Targets,
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Command::List(ListTarget::Formats) => list_formats(),
        Command::List(ListTarget::Targets) => list_targets(),
        Command::List(ListTarget::Sections) => list_sections(),
        Command::Explain { ref section_type } => explain(section_type),
        Command::Context { ref file } => write_context(file.as_deref()),
        Command::Detect { ref dir } => detect(dir),
        Command::New(NewTarget::Docs {
            ref dir,
            ref name,
            ref theme,
            guide,
            minimal,
            force,
        }) => new_docs(dir, name.as_deref(), theme, guide, minimal, force),
        Command::New(NewTarget::Section {
            ref section_type,
            ref out,
        }) => new_section(section_type, out.as_deref()),
        Command::New(NewTarget::Page {
            ref name,
            ref sections,
            ref dir,
        }) => new_page(name, sections, dir),
        Command::New(NewTarget::Guide {
            ref name,
            ref title,
            ref dir,
        }) => new_guide(name, title.as_deref(), dir),
        Command::Convert {
            ref source,
            ref out,
            ref from,
            dry_run,
        } => convert(source, out, from.as_deref(), dry_run),
        _ => {
            eprintln!("not implemented yet — see specs/zellij-cli-spec.md");
            std::process::exit(1);
        }
    };

    if let Err(err) = result {
        eprintln!("error: {err}");
        std::process::exit(err.exit_code());
    }
}

// ----------------------------------------------------------------- creating

fn new_docs(
    dir: &str,
    name: Option<&str>,
    theme: &str,
    guide: bool,
    minimal: bool,
    force: bool,
) -> zellij_core::Result<()> {
    let target = Utf8PathBuf::from(dir);

    // A directory named `.` would take its name from wherever the shell
    // happens to be, which is right about as often as it is baffling.
    let name = name.map(str::to_owned).unwrap_or_else(|| {
        target
            .canonicalize_utf8()
            .ok()
            .as_deref()
            .and_then(Utf8Path::file_name)
            .or_else(|| target.file_name())
            .unwrap_or("Docs")
            .to_owned()
    });

    let options = zellij_scaffold::DocsOptions {
        name,
        theme: theme.to_owned(),
        guide,
        minimal,
    };

    let written = zellij_scaffold::docs(&options).write(&target, force)?;

    println!("✓ {target}");
    for path in &written {
        println!("  {path}");
    }
    println!("\n  Next: zel dev {target}");

    Ok(())
}

fn new_section(section_type: &str, out: Option<&str>) -> zellij_core::Result<()> {
    if !zellij_core::schema::is_section_type(section_type) {
        eprintln!("error: no section type called \"{section_type}\"");
        eprintln!("       zel list sections — there are nineteen");
        std::process::exit(2);
    }

    let body = zellij_scaffold::section_file(section_type);

    let Some(out) = out else {
        print!("{body}");
        return Ok(());
    };

    let path = Utf8PathBuf::from(out);

    // Writing over a section somebody has already filled in is unrecoverable,
    // and there is no flag for it: `--out` a different file and move it.
    if path.exists() {
        eprintln!("error: {path} already exists");
        std::process::exit(2);
    }

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&path, body)?;

    println!("✓ {path}");
    println!("  zel explain {section_type} — every field it takes");

    Ok(())
}

fn new_page(name: &str, sections: &[String], dir: &str) -> zellij_core::Result<()> {
    for section_type in sections {
        if !zellij_core::schema::is_section_type(section_type) {
            eprintln!("error: no section type called \"{section_type}\"");
            eprintln!("       zel list sections — there are nineteen");
            std::process::exit(2);
        }
    }

    let root = Utf8PathBuf::from(dir);
    let (scaffold, entry) = zellij_scaffold::page(name, sections);
    let written = scaffold.write(&root, true)?;

    for path in &written {
        println!("✓ {path}");
    }

    // pages.yaml is rewritten by hand rather than by us: parsing it and
    // writing it back would lose the comments and the ordering somebody chose,
    // and this is four lines to paste.
    println!("\n  Add this to pages.yaml:\n");
    for line in entry.lines() {
        println!("    {line}");
    }

    Ok(())
}

fn new_guide(name: &str, title: Option<&str>, dir: &str) -> zellij_core::Result<()> {
    let root = Utf8PathBuf::from(dir);
    let order = next_guide_order(&root);
    let title = title.map(str::to_owned).unwrap_or_else(|| {
        let mut chars = name.chars();
        match chars.next() {
            Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            None => name.to_owned(),
        }
    });

    let written = zellij_scaffold::guide(name, &title, order).write(&root, true)?;

    for path in &written {
        println!("✓ {path}");
    }

    Ok(())
}

/// The lowest prefix no guide folder is already using.
///
/// Two guides sharing a number is a build error, and the caller is the only
/// one that can see the directory — so the number is chosen here rather than
/// guessed by the scaffold.
fn next_guide_order(root: &Utf8Path) -> usize {
    let Ok(entries) = std::fs::read_dir(root) else {
        return 1;
    };

    let highest = entries
        .flatten()
        .filter_map(|entry| {
            let name = entry.file_name().to_string_lossy().into_owned();
            let (prefix, _) = name.split_once('-')?;
            prefix.parse::<usize>().ok()
        })
        .max()
        .unwrap_or(0);

    highest + 1
}

fn list_formats() -> zellij_core::Result<()> {
    let converters: Vec<_> = zellij_convert::all().collect();

    if converters.is_empty() {
        println!("No converters are registered in this build.");
        return Ok(());
    }

    for converter in converters {
        println!("{:<14} {}", converter.id(), converter.label());
    }

    Ok(())
}

// Two, not four. A traced Node server and an nginx-plus-static-folder image
// were both on this list once; neither is a product anyone chooses. The first
// is what a platform builds for you, the second is the static site in a box.
// Offering four made a decision between two look harder than it is.
fn list_targets() -> zellij_core::Result<()> {
    for (name, description) in [
        (
            "static",
            "A folder of HTML, CSS and JS. Any CDN, free. The default.",
        ),
        (
            "image",
            "A container running the site on port 3000. Any registry. ~150 MB.",
        ),
    ] {
        println!("{name:<8} {description}");
    }

    Ok(())
}

fn list_sections() -> zellij_core::Result<()> {
    for section_type in zellij_core::schema::section_types() {
        let fields = zellij_core::schema::fields_of(&section_type).unwrap_or_default();
        let required = fields.iter().filter(|field| field.required).count();
        println!(
            "{section_type:<16} {} fields, {required} required",
            fields.len()
        );
    }

    println!("\nRun `zel explain <type>` for any of them.");
    Ok(())
}

/// Everything a section takes, and an example to paste.
///
/// This is the command that exists so nobody has to remember nineteen sets of
/// fields. It prints what is required first, because that is what someone is
/// actually asking when they run it.
fn explain(section_type: &str) -> zellij_core::Result<()> {
    use zellij_core::schema;

    if !schema::is_section_type(section_type) {
        eprintln!("`{section_type}` is not a section type. Try `zel list sections`.");
        std::process::exit(2);
    }

    let fields = schema::fields_of(section_type).unwrap_or_default();
    let (required, optional): (Vec<_>, Vec<_>) =
        fields.into_iter().partition(|field| field.required);

    println!("{section_type}\n");

    if !required.is_empty() {
        println!("Required");
        for field in &required {
            println!("  {:<14} {}", field.name, field.type_name);
        }
        println!();
    }

    if !optional.is_empty() {
        println!("Optional");
        for field in &optional {
            let default = field
                .default
                .as_ref()
                .map(|value| format!("  (default {value})"))
                .unwrap_or_default();
            println!("  {:<14} {}{}", field.name, field.type_name, default);
        }
        println!();
    }

    // A field like `items` *is* the section, and "list of object" names its
    // existence without saying what may go in it — which is exactly enough to
    // write something plausible that the build rejects.
    for field in required.iter().chain(optional.iter()).filter(|f| f.nested) {
        if let Some(inner) = schema::nested_fields(section_type, &field.name) {
            println!("Each entry in `{}`", field.name);
            for nested in inner {
                let default = nested
                    .default
                    .as_ref()
                    .map(|value| format!("  (default {value})"))
                    .unwrap_or_default();
                let mark = if nested.required { "*" } else { " " };
                println!(
                    "  {mark}{:<13} {}{}",
                    nested.name, nested.type_name, default
                );
            }
            println!("  * required\n");
        }
    }

    println!("Every section also takes: id, background, padding, motion, titleGradient.");
    println!("\nWrite one with `zel new section {section_type}`.");
    Ok(())
}

fn write_context(file: Option<&str>) -> zellij_core::Result<()> {
    match file {
        Some(path) => {
            std::fs::write(path, zellij_core::schema::CONTEXT_MARKDOWN)?;
            println!("Wrote the Zellij reference to {path}");
        }
        None => print!("{}", zellij_core::schema::CONTEXT_MARKDOWN),
    }

    Ok(())
}

fn detect(dir: &str) -> zellij_core::Result<()> {
    let source = camino::Utf8Path::new(dir);
    let found = zellij_convert::detect(source);

    if found.is_empty() {
        println!("Nothing here looks like a documentation tool this build knows.");
        println!(
            "\nIf the folder is plain Markdown with no configuration, there is no\nstructure to convert from. Write the reference with `zel context` and\nask an AI agent to author the folder instead."
        );
        return Ok(());
    }

    for (converter, confidence) in found {
        let strength = if confidence >= zellij_convert::Confidence::CERTAIN {
            "certain"
        } else {
            "likely"
        };
        println!("{:<12} {}  ({strength})", converter.id(), converter.label());
    }

    Ok(())
}

fn convert(source: &str, out: &str, from: Option<&str>, dry_run: bool) -> zellij_core::Result<()> {
    use zellij_core::Error;

    let source = camino::Utf8Path::new(source);
    let out = camino::Utf8Path::new(out);

    let converter = match from {
        Some(id) => zellij_convert::all()
            .find(|converter| converter.id() == id)
            .ok_or_else(|| Error::UnknownFormat {
                path: source.to_owned(),
            })?,
        None => {
            let found = zellij_convert::detect(source);
            // Guessing between two plausible formats produces a conversion
            // that is wrong in ways nobody notices until much later, so an
            // ambiguous tree is a question rather than a coin toss.
            match found.first() {
                Some((converter, _)) => *converter,
                None => {
                    return Err(Error::UnknownFormat {
                        path: source.to_owned(),
                    });
                }
            }
        }
    };

    println!("Converting {source} as {}", converter.label());

    let conversion = converter.convert(source, &zellij_convert::ConvertOptions::default())?;

    if dry_run {
        println!("\nWould write {} files into {out}:", conversion.files.len());
        for file in &conversion.files {
            println!("  {}", file.path);
        }
    } else {
        // Nothing is written until the whole conversion has succeeded, so a
        // failure halfway through leaves no half-built folder behind for
        // someone to mistake for a finished one.
        for file in &conversion.files {
            let target = out.join(&file.path);
            if let Some(parent) = target.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(&target, &file.body)?;
        }
        println!("\nWrote {} files into {out}", conversion.files.len());
    }

    if !conversion.unmapped.is_empty() {
        println!(
            "\n{} thing(s) could not be carried over:",
            conversion.unmapped.len()
        );
        for entry in &conversion.unmapped {
            println!("  {}: {}", entry.source, entry.reason);
        }
    }

    for note in &conversion.notes {
        println!("\nNote: {note}");
    }

    if !dry_run {
        println!("\nValidate it with `zel check {out}`.");
    }

    Ok(())
}
