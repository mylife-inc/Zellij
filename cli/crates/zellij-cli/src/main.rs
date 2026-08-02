//! The Zellij command line.
//!
//! Named `zel` rather than `zellij`: the latter is already a widely used Rust
//! terminal multiplexer, and two binaries of the same name on one PATH is a
//! problem nobody should have to debug.
//!
//! Commands are declared here and implemented elsewhere — this file is about
//! what the tool offers and how it reports, never about how any of it works.
//! See `specs/zellij-cli-spec.md` for the surface in full.

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

    /// Check the environment: container runtime, image, versions.
    Doctor,
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
        #[arg(default_value = ".")]
        dir: String,

        /// One example of every section type.
        #[arg(long)]
        full: bool,
    },
    /// A section file, with its required fields filled in.
    Section {
        section_type: Option<String>,

        #[arg(long)]
        out: Option<String>,

        /// Ask for the fields that have no sensible default.
        #[arg(long, short)]
        interactive: bool,
    },
    /// A page, and the section files it references.
    Page { name: String },
    /// A guide folder with a sidebar and an index page.
    Guide { name: String },
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

fn list_targets() -> zellij_core::Result<()> {
    for (name, description) in [
        ("static", "HTML, CSS and JS. Hosts free on any CDN. The default."),
        ("node", "A traced Node server. Keeps image optimisation."),
        ("image:static", "Container, nginx serving the static folder. ~30 MB."),
        ("image:node", "Container, Node serving the app. ~150 MB."),
    ] {
        println!("{name:<14} {description}");
    }

    Ok(())
}
