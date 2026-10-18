//! Command definitions, shortcuts and completion share one argument schema.
use clap::{CommandFactory, FromArgMatches, Parser, Subcommand};

// Each flattened shortcut declares its spelling, descriptive route, and mapping
// once. Its argument type is also used by the descriptive command. Normalizing
// before dispatch keeps global-option policy and all side effects identical.
macro_rules! shortcuts {
    (args { $( $variant:ident($args:ty) => ($short:literal, $full:literal, $map:expr); )* }
     unit { $( $unit:ident => ($unit_short:literal, $unit_full:literal, $command:expr); )* }) => {
        #[derive(Subcommand, Debug, PartialEq, Eq)]
        pub(crate) enum Shortcut {
            $( #[command(name = $short, about = concat!("Shortcut for `zel ", $full, "`."))]
            $variant($args), )*
            $( #[command(name = $unit_short, about = concat!("Shortcut for `zel ", $unit_full, "`."))]
            $unit, )*
        }
        impl Shortcut {
            fn into_command(self) -> Command {
                match self {
                    $( Self::$variant(args) => ($map)(args), )*
                    $( Self::$unit => $command, )*
                }
            }
        }
        const SHORTCUTS: &[(&str, &str)] = &[
            $( ($short, $full), )*
            $( ($unit_short, $unit_full), )*
        ];
    };
}

shortcuts! {
    args {
        Init(NewPortableArgs) => ("init", "new portable", |a| Command::New(NewTarget::Portable(a)));
        Docs(NewDocsArgs) => ("nd", "new docs", |a| Command::New(NewTarget::Docs(a)));
        Section(NewSectionArgs) => ("ns", "new section", |a| Command::New(NewTarget::Section(a)));
        Page(NewPageArgs) => ("np", "new page", |a| Command::New(NewTarget::Page(a)));
        Guide(NewGuideArgs) => ("ng", "new guide", |a| Command::New(NewTarget::Guide(a)));
        Project(NewProjectArgs) => ("npr", "new project", |a| Command::New(NewTarget::Project(a)));
        StaticImage(SiteImageArgs) => ("si", "image static", |a| Command::Image { action: ImageAction::Static(a) });
        NextImage(SiteImageArgs) => ("ni", "image next", |a| Command::Image { action: ImageAction::Next(a) });
        ImageRun(ImageRunArgs) => ("ir", "image run", |a| Command::Image { action: ImageAction::Run(a) });
        ImagePush(ImagePushArgs) => ("ip", "image push", |a| Command::Image { action: ImageAction::Push(a) });
        Dockerfile(ImageDockerfileArgs) => ("idf", "image dockerfile", |a| Command::Image { action: ImageAction::Dockerfile(a) });
    }
    unit {
        Sections => ("sections", "list sections", Command::List(ListTarget::Sections));
        Themes => ("themes", "list themes", Command::List(ListTarget::Themes));
        Formats => ("formats", "list formats", Command::List(ListTarget::Formats));
        Targets => ("targets", "list targets", Command::List(ListTarget::Targets));
    }
}

impl Cli {
    /// The same tree is used for parsing, help, and generated completions.
    fn command_tree() -> clap::Command {
        let mut command = Self::command();
        fn describe_aliases(command: &mut clap::Command, prefix: &str) {
            for child in command.get_subcommands_mut() {
                let aliases: Vec<_> = child
                    .get_visible_aliases()
                    .map(|alias| format!("zel {prefix}{alias}"))
                    .collect();
                if !aliases.is_empty() {
                    *child = child
                        .clone()
                        .after_help(format!("Also accepted: {}", aliases.join(", ")));
                }
                describe_aliases(child, &format!("{prefix}{} ", child.get_name()));
            }
        }
        describe_aliases(&mut command, "");
        for (short, full) in SHORTCUTS {
            let mut leaf = &mut command;
            for part in full.split_whitespace() {
                leaf = leaf
                    .find_subcommand_mut(part)
                    .expect("shortcut target exists");
            }
            *leaf = leaf.clone().after_help(format!("Shortcut: zel {short}"));
        }
        command
    }

    fn from_matches(matches: &clap::ArgMatches) -> Result<Self, clap::Error> {
        let mut cli = Self::from_arg_matches(matches)?;
        if let Command::Shortcut(shortcut) = cli.command {
            cli.command = shortcut.into_command();
        }
        if let Command::Netlify(options) = cli.command {
            cli.command = Command::Deploy {
                provider: Some("netlify".into()),
                options,
            };
        }
        Ok(cli)
    }

    pub(crate) fn parse_args() -> Self {
        Self::from_matches(&Self::command_tree().get_matches()).unwrap_or_else(|err| err.exit())
    }
}

pub(crate) fn completions(shell: clap_complete::Shell) -> zellij_core::Result<()> {
    use std::io::Write as _;
    // Generators expect an infallible writer; buffer first so a closed stdout
    // is an ordinary IO error rather than a panic inside the generator.
    let mut script = Vec::new();
    clap_complete::generate(shell, &mut Cli::command_tree(), "zel", &mut script);
    std::io::stdout().lock().write_all(&script)?;
    Ok(())
}

#[derive(Parser, Debug, PartialEq, Eq)]
#[command(
    name = "zel",
    version,
    about = "Turn a folder of content into a documentation site.",
    long_about = None,
)]
pub(crate) struct Cli {
    #[command(subcommand)]
    pub(crate) command: Command,

    /// JSON output for context, list, explain and detect, plus portable check.
    #[arg(long, global = true)]
    pub(crate) json: bool,
}

#[derive(Subcommand, Debug, PartialEq, Eq)]
pub(crate) enum Command {
    /// Create portable Markdown, hosted content, an app, a section, a page or a guide.
    #[command(subcommand)]
    New(NewTarget),

    /// Show a section type's fields, defaults and allowed values.
    #[command(visible_alias = "ex")]
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
    #[command(visible_alias = "ctx")]
    Context {
        /// Where to write it. Defaults to standard output.
        file: Option<String>,
    },

    /// Say what tool a documentation folder was written for.
    Detect { dir: String },

    /// Convert documentation written for another tool into a new folder.
    #[command(visible_alias = "cv")]
    Convert(ImportArgs),

    /// Validate portable Markdown natively, or hosted content using its engine.
    #[command(visible_alias = "c")]
    Check {
        /// Site root or explicit legacy content folder; defaults to ./zel.yaml or ./docs/zellij/zel.yaml.
        dir: Option<String>,
        /// Override automatic selection: portable Markdown or hosted static content.
        #[arg(long, value_parser = ["portable", "static"])]
        target: Option<String>,
        /// Hosted: parse MDX and check local references. Portable always checks references.
        #[arg(long)]
        deep: bool,
        /// Fail when validation reports warnings.
        #[arg(long)]
        strict: bool,
    },

    /// Development server. Edit content, reload the page.
    Dev {
        /// Site root or explicit legacy content folder; defaults to ./zel.yaml or ./docs/zellij/zel.yaml.
        dir: Option<String>,

        #[arg(long)]
        port: Option<u16>,
    },

    /// Build the site.
    #[command(visible_alias = "b")]
    Build {
        /// Site root or explicit legacy content folder; defaults to ./zel.yaml or ./docs/zellij/zel.yaml.
        dir: Option<String>,

        /// Output: use the configured renderer, or infer it for explicit legacy content; see `zel list targets`.
        #[arg(long)]
        target: Option<String>,

        #[arg(long)]
        out: Option<String>,

        /// Image tag. `--target image` only.
        #[arg(long)]
        tag: Option<String>,

        /// Portable site name: overrides zel.yaml, then defaults to the source folder name.
        #[arg(long)]
        name: Option<String>,

        /// Portable theme: overrides zel.yaml, then defaults to cupertino; see `zel list themes`.
        #[arg(long)]
        theme: Option<String>,
    },

    /// Build static HTML and deploy it; omitted provider uses saved publication.
    #[command(
        after_help = "Shortcut: zel netlify [site]. Bare zel deploy is equivalent to zel publish."
    )]
    Deploy {
        /// Currently netlify. With no provider, use zel.yaml's publish settings.
        provider: Option<String>,
        #[command(flatten)]
        options: DeliveryArgs,
    },

    /// Shortcut for `zel deploy netlify`.
    Netlify(DeliveryArgs),

    /// Build, run and push container images.
    Image {
        #[command(subcommand)]
        action: ImageAction,
    },

    /// Generate a GitHub workflow; `ci run` dispatches an existing workflow.
    Ci(CiArgs),

    /// Detect, convert and validate in one step.
    #[command(visible_alias = "mig")]
    Migrate(ImportArgs),

    /// Build once and deliver using saved publication settings.
    #[command(visible_alias = "pub")]
    Publish(DeliveryArgs),

    /// Convert, check, build and deploy in one step.
    Ship(ShipArgs),

    /// Serve what `build` produced.
    #[command(visible_alias = "s")]
    Serve {
        /// Configured site or explicit generated output folder.
        dir: Option<String>,

        #[arg(long)]
        port: Option<u16>,

        /// URL prefix for an export configured with a deployment basePath.
        #[arg(long, default_value = "/")]
        base_path: String,
    },

    /// Build a portable Markdown site, serve it locally, and open a browser.
    #[command(visible_alias = "v")]
    View {
        /// Site root or explicit legacy content folder; defaults to ./zel.yaml or ./docs/zellij/zel.yaml.
        dir: Option<String>,
        #[arg(long)]
        out: Option<String>,
        #[arg(long)]
        name: Option<String>,
        #[arg(long)]
        theme: Option<String>,
        /// Local port; use 0 to choose an available port.
        #[arg(long)]
        port: Option<u16>,
        /// Print the URL without launching a browser.
        #[arg(long, conflicts_with = "browser")]
        no_open: bool,
        /// Browser executable to launch with the URL as its sole argument.
        #[arg(long)]
        browser: Option<String>,
        /// Rebuild on source edits and reload the HTTP preview after success.
        #[arg(long)]
        watch: bool,
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

    /// Print a shell completion script to standard output.
    #[command(visible_alias = "comp")]
    Completions {
        #[arg(value_enum)]
        shell: clap_complete::Shell,
    },

    #[command(flatten)]
    Shortcut(Shortcut),
}

#[derive(Subcommand, Debug, PartialEq, Eq)]
pub(crate) enum ImageAction {
    /// Build HTML and package it with an HTTP server (portable or hosted).
    Static(SiteImageArgs),
    /// Build the Next.js server image, adapting portable Markdown when needed.
    Next(SiteImageArgs),
    /// The same as `build --target image`.
    Build {
        dir: Option<String>,

        #[arg(long)]
        tag: Option<String>,
    },
    /// Run it locally — the last check before it goes anywhere.
    Run(ImageRunArgs),
    /// Push it to a registry.
    Push(ImagePushArgs),
    /// Print a static or Next.js Dockerfile; omitted kind retains the Next recipe.
    Dockerfile(ImageDockerfileArgs),
}

#[derive(clap::Args, Debug, PartialEq, Eq)]
#[command(args_conflicts_with_subcommands = true, subcommand_negates_reqs = true)]
pub(crate) struct CiArgs {
    /// Site root; defaults to . or docs/zellij when zel.yaml is present.
    #[arg(conflicts_with = "site_root")]
    pub(crate) dir: Option<String>,
    #[command(flatten)]
    pub(crate) options: CiInitArgs,
    #[command(subcommand)]
    pub(crate) action: Option<CiAction>,
}

#[derive(clap::Args, Debug, Default, PartialEq, Eq)]
pub(crate) struct CiInitArgs {
    /// Configured site or explicit legacy content folder (--content is retained).
    #[arg(long, visible_alias = "content")]
    pub(crate) site_root: Option<String>,
    /// Exact compatible release version. Overrides ci.zel-version.
    #[arg(long)]
    pub(crate) zel_version: Option<String>,
    /// Override saved delivery with Netlify deployment.
    #[arg(long, conflicts_with = "target")]
    pub(crate) deploy: Option<String>,
    /// Legacy override: static validates/builds; image builds/pushes a Next image.
    #[arg(long, value_parser = ["static", "image"], conflicts_with = "deploy")]
    pub(crate) target: Option<String>,
    /// Netlify site ID; --site remains accepted.
    #[arg(long, visible_alias = "site")]
    pub(crate) site_id: Option<String>,
    /// Request production Netlify deployments; previews are the default.
    #[arg(long)]
    pub(crate) prod: bool,
    /// Overwrite the selected workflow, retaining all other files.
    #[arg(long)]
    pub(crate) force: bool,
    /// Explicitly upload NETLIFY_AUTH_TOKEN through gh after writing the workflow.
    #[arg(long, conflicts_with = "no_provision")]
    pub(crate) provision: bool,
    /// Retained spelling for the default: write local files only.
    #[arg(long, conflicts_with = "provision")]
    pub(crate) no_provision: bool,
}

#[derive(Subcommand, Debug, PartialEq, Eq)]
pub(crate) enum CiAction {
    /// Write a workflow into the Git repository containing the selected site.
    Init {
        /// Currently github.
        system: String,
        #[command(flatten)]
        options: CiInitArgs,
    },
    /// Dispatch an existing workflow on GitHub; does not generate or deploy locally.
    #[command(name = "run", visible_alias = "publish")]
    Run {
        #[arg(long, default_value = "docs.yml")]
        workflow: String,
        #[arg(long, default_value = "main")]
        r#ref: String,
    },
}

#[derive(Subcommand, Debug, PartialEq, Eq)]
pub(crate) enum NewTarget {
    /// A native Markdown starter with zel.yaml; no Docker or Node required.
    Portable(NewPortableArgs),
    /// A Next.js wrapper app with the engine installed.
    App {
        #[arg(default_value = ".")]
        dir: String,

        /// Also write a starter content folder, so there is something to open.
        #[arg(long)]
        with_docs: bool,
    },
    /// A hosted YAML/MDX content folder on its own.
    Docs(NewDocsArgs),
    /// A section file, with its required fields filled in.
    Section(NewSectionArgs),
    /// A page, and the section files it references.
    Page(NewPageArgs),
    /// A guide folder with a sidebar and an index page.
    Guide(NewGuideArgs),
    /// A repository with documentation and a product as siblings, and a
    /// pipeline that publishes the first on every push.
    Project(NewProjectArgs),
}

#[derive(Subcommand, Debug, PartialEq, Eq)]
pub(crate) enum ListTarget {
    /// Every section type.
    Sections,
    /// Every theme.
    Themes,
    /// Every documentation format `convert` recognises.
    Formats,
    /// Everything `build` can produce.
    Targets,
}

#[derive(clap::Args, Debug, PartialEq, Eq)]
pub(crate) struct NewPortableArgs {
    #[arg(default_value = "docs/zellij")]
    pub(crate) dir: String,
    /// Site name. Defaults to the directory's name.
    #[arg(long)]
    pub(crate) name: Option<String>,
    /// One of the themes reported by `zel list themes`.
    #[arg(long, default_value = "cupertino")]
    pub(crate) theme: String,
    /// Only zel.yaml and content/index.md, without the example guide or image.
    #[arg(long)]
    pub(crate) minimal: bool,
}

#[derive(clap::Args, Debug, PartialEq, Eq)]
pub(crate) struct NewDocsArgs {
    #[arg(default_value = "docs/zellij")]
    pub(crate) dir: String,

    /// The site name. Defaults to the directory's.
    #[arg(long)]
    pub(crate) name: Option<String>,

    /// One of the fourteen — `zel list themes`.
    #[arg(long, default_value = "paper")]
    pub(crate) theme: String,

    /// Also create a guide folder.
    #[arg(long)]
    pub(crate) guide: bool,

    /// site.yaml and one page, and nothing else.
    #[arg(long)]
    pub(crate) minimal: bool,

    /// Write into a directory that is not empty.
    #[arg(long)]
    pub(crate) force: bool,
}

#[derive(clap::Args, Debug, PartialEq, Eq)]
pub(crate) struct NewSectionArgs {
    pub(crate) section_type: String,

    #[arg(long)]
    pub(crate) out: Option<String>,
}

#[derive(clap::Args, Debug, PartialEq, Eq)]
pub(crate) struct NewPageArgs {
    pub(crate) name: String,

    /// Comma-separated section types. Defaults to a single hero.
    #[arg(long, value_delimiter = ',')]
    pub(crate) sections: Vec<String>,

    #[arg(long)]
    pub(crate) dir: Option<String>,
}

#[derive(clap::Args, Debug, PartialEq, Eq)]
pub(crate) struct NewGuideArgs {
    pub(crate) name: String,

    #[arg(long)]
    pub(crate) title: Option<String>,

    #[arg(long)]
    pub(crate) dir: Option<String>,
}

#[derive(clap::Args, Debug, PartialEq, Eq)]
pub(crate) struct NewProjectArgs {
    pub(crate) name: String,

    /// What to put in the product folder: rust | go | java | kotlin |
    /// python | node | none.
    #[arg(long, default_value = "none")]
    pub(crate) product_lang: String,

    #[arg(long, default_value = "docs/zellij")]
    pub(crate) docs_dir: String,

    #[arg(long, default_value = "product")]
    pub(crate) product_dir: String,
}

#[derive(clap::Args, Debug, PartialEq, Eq)]
pub(crate) struct ImageRunArgs {
    pub(crate) tag: Option<String>,

    #[arg(long, default_value_t = 3000)]
    pub(crate) port: u16,
}

#[derive(clap::Args, Debug, PartialEq, Eq)]
pub(crate) struct ImagePushArgs {
    pub(crate) tag: String,

    #[arg(long)]
    pub(crate) registry: Option<String>,
}

#[derive(clap::Args, Debug, PartialEq, Eq)]
pub(crate) struct ImageDockerfileArgs {
    pub(crate) dir: Option<String>,

    #[arg(long, value_enum, default_value = "next")]
    pub(crate) kind: ImageKind,
}

#[derive(clap::ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ImageKind {
    Static,
    Next,
}

#[derive(clap::Args, Debug, PartialEq, Eq)]
pub(crate) struct SiteImageArgs {
    /// Site root or explicit Markdown/hosted content folder.
    pub(crate) dir: Option<String>,

    /// Image name; defaults to <site-name>-static:latest.
    #[arg(long)]
    pub(crate) tag: Option<String>,
}

#[cfg(test)]
#[path = "args_tests.rs"]
mod tests;

#[derive(clap::Args, Debug, PartialEq, Eq)]
pub(crate) struct DeliveryArgs {
    /// Site root; defaults to ./zel.yaml or ./docs/zellij/zel.yaml.
    pub(crate) dir: Option<String>,
    /// Explicit provider override for saved static-site deployment.
    #[arg(long)]
    pub(crate) to: Option<String>,
    /// Exact image reference for saved registry publication only.
    #[arg(long)]
    pub(crate) tag: Option<String>,
    /// Existing Netlify site ID or domain; overrides NETLIFY_SITE_ID and publish.site.
    #[arg(long = "site-id", visible_alias = "site")]
    pub(crate) site_id: Option<String>,
    /// Update the live Netlify site. Omit to create a draft preview.
    #[arg(long)]
    pub(crate) prod: bool,
    /// Print the plan without tools, credentials, builds or remote requests.
    #[arg(long)]
    pub(crate) dry_run: bool,
}

/// Explicit layout keeps existing scripts compatible until a versioned default change.
#[derive(Clone, Copy, Debug, PartialEq, Eq, clap::ValueEnum)]
pub(crate) enum ImportLayout {
    Content,
    Site,
}

#[derive(clap::Args, Debug, PartialEq, Eq)]
pub(crate) struct ImportArgs {
    pub(crate) source: String,
    /// New or empty destination outside the source tree.
    #[arg(long)]
    pub(crate) out: String,
    /// Force a converter; required when detection is ambiguous.
    #[arg(long)]
    pub(crate) from: Option<String>,
    /// Site writes zel.yaml and content/; content preserves the legacy layout.
    #[arg(long, value_enum, default_value = "content")]
    pub(crate) layout: ImportLayout,
    /// Convert in memory and report files/warnings without writing or validating.
    #[arg(long)]
    pub(crate) dry_run: bool,
}

#[derive(clap::Args, Debug, PartialEq, Eq)]
pub(crate) struct ShipArgs {
    pub(crate) source: String,
    /// New configured site root, outside the source tree; contains zel.yaml and content/.
    #[arg(long)]
    pub(crate) out: String,
    #[arg(long)]
    pub(crate) from: Option<String>,
    /// Deploy static HTML to an existing Netlify site.
    #[arg(long, required_unless_present = "image", conflicts_with = "image")]
    pub(crate) provider: Option<String>,
    /// Build and push this image kind instead of deploying static HTML.
    #[arg(long, value_enum, requires = "tag", conflicts_with_all = ["site_id", "prod"])]
    pub(crate) image: Option<ImageKind>,
    /// Exact registry destination for --image.
    #[arg(long, requires = "image")]
    pub(crate) tag: Option<String>,
    /// Existing Netlify site ID or domain; defaults to NETLIFY_SITE_ID.
    #[arg(long = "site-id", visible_alias = "site")]
    pub(crate) site_id: Option<String>,
    /// Request production publication; the default is a draft preview.
    #[arg(long)]
    pub(crate) prod: bool,
    /// Convert in memory and show delivery intent; no writes, tools or network.
    #[arg(long)]
    pub(crate) dry_run: bool,
}
