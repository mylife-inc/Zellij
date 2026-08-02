//! The Zellij command line.
//!
//! Commands are declared here and implemented elsewhere: this file is about
//! what the tool offers and how it reports, never about how any of it works.
//! See `specs/zellij-cli-spec.md` for the full surface.

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "zellij",
    version,
    about = "Turn a folder of content into a documentation site.",
    long_about = None,
)]
struct Cli {
    #[command(subcommand)]
    command: Command,

    /// Machine-readable output. Every command supports it, because a tool only
    /// a human can read cannot be used by a pipeline.
    #[arg(long, global = true)]
    json: bool,

    #[arg(long, short, global = true)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Command {
    /// Write a content folder that already validates.
    New {
        /// Where to write it.
        #[arg(default_value = ".")]
        dir: String,

        /// One example of every section type, rather than the minimum.
        #[arg(long)]
        full: bool,
    },

    /// Write the reference an AI agent authors a content folder from.
    ///
    /// Needs neither Docker nor Node: the file is generated from the engine's
    /// schemas at release time and compiled into this binary.
    Context {
        /// Where to write it. Defaults to standard output.
        file: Option<String>,
    },

    /// Convert documentation written for another tool.
    Convert {
        /// The existing documentation folder.
        source: String,

        /// Where to write the Zellij content folder. Never the source: a
        /// conversion that overwrote the original would be unrecoverable.
        #[arg(long)]
        out: String,

        /// Force a format instead of detecting one.
        #[arg(long)]
        from: Option<String>,

        /// Report what would be written, and write nothing.
        #[arg(long)]
        dry_run: bool,
    },

    /// Validate a content folder.
    Check {
        #[arg(default_value = ".")]
        dir: String,
    },

    /// List what this build knows about.
    List {
        #[command(subcommand)]
        what: ListTarget,
    },

    /// Check the environment: container runtime, image, versions.
    Doctor,
}

#[derive(Subcommand)]
enum ListTarget {
    /// Every section type, and what it is for.
    Sections,
    /// Every documentation format `convert` recognises.
    Formats,
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Command::List { what: ListTarget::Formats } => list_formats(),
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
