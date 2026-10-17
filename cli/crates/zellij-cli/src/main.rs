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
use serde_json::json;
use std::io::Write;
mod preview;

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

    /// JSON output for context, list, explain and detect. Other commands reject it.
    #[arg(long, global = true)]
    json: bool,
}

#[derive(Subcommand)]
enum Command {
    /// Create an app, a content folder, a section, a page or a guide.
    #[command(subcommand)]
    New(NewTarget),

    /// Show a section type's fields, defaults and allowed values.
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
        /// Parse MDX and check statically known local links, anchors and assets.
        #[arg(long)]
        deep: bool,
        /// Fail when the engine reports warnings.
        #[arg(long)]
        strict: bool,
    },

    /// Development server. Edit content, reload the page.
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

        /// Output: static for discovered Zellij content, portable for plain Markdown; see `zel list targets`.
        #[arg(long)]
        target: Option<String>,

        #[arg(long)]
        out: Option<String>,

        /// Image tag. `--target image` only.
        #[arg(long)]
        tag: Option<String>,

        /// Portable Markdown site name (defaults to the source folder name).
        #[arg(long)]
        name: Option<String>,

        /// Portable Markdown theme; see `zel list themes`.
        #[arg(long)]
        theme: Option<String>,
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

        /// URL prefix for an export configured with a deployment basePath.
        #[arg(long, default_value = "/")]
        base_path: String,
    },

    /// Build a portable Markdown site, serve it locally, and open a browser.
    View {
        #[arg(default_value = ".")]
        dir: String,
        #[arg(long)]
        out: Option<String>,
        #[arg(long)]
        name: Option<String>,
        #[arg(long)]
        theme: Option<String>,
        /// Local port; use 0 to choose an available port.
        #[arg(long, default_value_t = 3000)]
        port: u16,
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

        /// The content folder, as the pipeline will refer to it.
        #[arg(long)]
        content: Option<String>,

        /// The Netlify project id. Defaults to $NETLIFY_SITE_ID.
        #[arg(long)]
        site: Option<String>,

        /// Overwrite an existing workflow.
        #[arg(long)]
        force: bool,

        /// Write the workflow without uploading the Netlify token.
        #[arg(long)]
        no_provision: bool,
    },

    /// Run the deploy pipeline now, on GitHub.
    ///
    /// The pipeline deploys on a push that touches the content folder, which
    /// covers the ordinary case. This is for the times there is nothing to
    /// commit: a rotated Netlify token, a site restored after a bad deploy, a
    /// first run after adding the secrets.
    ///
    /// The build still happens on GitHub. Nothing is uploaded from here.
    Publish {
        /// The workflow file to run.
        #[arg(long, default_value = "docs.yml")]
        workflow: String,

        /// The branch to run it on.
        #[arg(long, default_value = "main")]
        r#ref: String,
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

    if cli.json
        && !matches!(
            cli.command,
            Command::Context { .. }
                | Command::List(_)
                | Command::Explain { .. }
                | Command::Detect { .. }
        )
    {
        eprintln!(
            "error: --json is supported only by context, list, explain and detect; no command was run"
        );
        std::process::exit(2);
    }

    let result = match cli.command {
        Command::List(ListTarget::Formats) => list_formats(cli.json),
        Command::List(ListTarget::Targets) => list_targets(cli.json),
        Command::List(ListTarget::Sections) => list_sections(cli.json),
        Command::List(ListTarget::Themes) => list_themes(cli.json),
        Command::Explain { ref section_type } => explain(section_type, cli.json),
        Command::Context { ref file } => write_context(file.as_deref(), cli.json),
        Command::Detect { ref dir } => detect(dir, cli.json),
        Command::Check {
            ref dir,
            deep,
            strict,
        } => check(dir, zellij_engine::CheckOptions { deep, strict }),
        Command::Dev { ref dir, port } => dev(dir, port),
        Command::Serve {
            ref dir,
            port,
            ref base_path,
        } => preview::serve(dir, port, base_path),
        Command::View {
            ref dir,
            ref out,
            ref name,
            ref theme,
            port,
            no_open,
            ref browser,
            watch,
        } => preview::view(
            dir,
            preview::ViewOptions {
                out: out.as_deref(),
                name: name.as_deref(),
                theme: theme.as_deref(),
                port,
                no_open,
                browser: browser.as_deref(),
                watch,
            },
        ),
        Command::Build {
            ref dir,
            ref target,
            ref out,
            ref tag,
            ref name,
            ref theme,
        } => build_dispatch(
            dir,
            target.as_deref(),
            out.as_deref(),
            tag.as_deref(),
            name.as_deref(),
            theme.as_deref(),
        ),
        Command::Image { ref action } => match action {
            ImageAction::Build { dir, tag } => image_build(dir, tag.as_deref(), None),
            ImageAction::Run { tag, port } => image_run(tag.as_deref(), *port),
            ImageAction::Push { tag, registry } => image_push(tag, registry.as_deref()),
            ImageAction::Dockerfile { dir } => image_dockerfile(dir),
        },
        Command::Doctor => doctor(),
        Command::Ci { ref action } => match action {
            CiAction::Init {
                system,
                deploy,
                target,
                content,
                site,
                force,
                no_provision,
            } => ci_init(
                system,
                deploy.as_deref(),
                target,
                content.as_deref(),
                site.as_deref(),
                *force,
                *no_provision,
            ),
            CiAction::Publish { workflow, r#ref } => ci_publish(workflow, r#ref),
        },
        Command::Migrate {
            ref source,
            ref out,
        } => migrate(source, out),
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
        Command::New(NewTarget::Project {
            ref name,
            ref product_lang,
            ref docs_dir,
            ref product_dir,
        }) => new_project(name, product_lang, docs_dir, product_dir),
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

// ----------------------------------------------------------------- checking

/// Where the content is, when nobody said.
///
/// `./content` first because that is what `zel new docs` writes, then the
/// current root, then `./docs` and `./documentation`. A folder that merely
/// holds Markdown is never assumed: guessing produces a build that looks
/// finished and is about something else.
fn resolve_content(dir: &str) -> zellij_core::Result<Utf8PathBuf> {
    let given = Utf8PathBuf::from(dir);

    if dir != "." {
        return if given.is_dir() {
            Ok(given)
        } else {
            Err(zellij_core::Error::NoContentFolder { path: given })
        };
    }

    Ok(zellij_core::Project::discover(&given)?.content)
}

fn check(dir: &str, options: zellij_engine::CheckOptions) -> zellij_core::Result<()> {
    let content = resolve_content(dir)?;
    let engine = zellij_engine::ContainerEngine::detect(None)?;

    // The trait has to be in scope for its methods to be callable.
    use zellij_engine::Engine as _;
    let report = match engine.check(&content, options) {
        Ok(report) => report,
        // Printed and exited here rather than returned: `main` prints the
        // error itself, and this one needs several paragraphs after it rather
        // than the same sentence twice.
        Err(err @ zellij_core::Error::MountEmpty { .. }) => {
            report_empty_mount(&err);
            std::process::exit(err.exit_code());
        }
        Err(err) => return Err(err),
    };

    print!("{}", report.output);

    if !report.ok {
        return Err(zellij_core::Error::ContentInvalid);
    }

    Ok(())
}

fn dev(dir: &str, port: u16) -> zellij_core::Result<()> {
    let content = resolve_content(dir)?;
    let engine = zellij_engine::ContainerEngine::detect(None)?;

    println!("  Engine   {}", engine.image());
    if !engine.image_is_local() {
        println!("           not pulled yet — this first run fetches ~237 MB");
    }
    println!("  Content  {content}");
    println!("  Serving  http://localhost:{port}");
    println!();
    println!("  Edit anything under {content} and reload. Ctrl-C to stop.");
    println!();

    match engine.dev(&content, port) {
        Err(err @ zellij_core::Error::MountEmpty { .. }) => {
            report_empty_mount(&err);
            std::process::exit(err.exit_code());
        }
        other => other,
    }
}

/// Shared by every command that mounts content, because every one of them can
/// meet a runtime that silently shares nothing.
fn report_empty_mount(err: &zellij_core::Error) {
    eprintln!("error: {err}");
    eprintln!();
    eprintln!("  The folder exists, and the container was given an empty one. A bind");
    eprintln!("  mount of a path the runtime cannot reach succeeds and mounts nothing,");
    eprintln!("  so this would otherwise be reported as a site with no pages.");
    eprintln!();
    eprintln!("  Colima shares $HOME and not /tmp. Either work somewhere under your");
    eprintln!("  home directory, or share the path:");
    eprintln!();
    eprintln!("    colima stop && colima start --mount '/tmp:w'");
    eprintln!();
    eprintln!("  Docker Desktop: Settings → Resources → File sharing.");
}

fn build_dispatch(
    dir: &str,
    target: Option<&str>,
    out: Option<&str>,
    tag: Option<&str>,
    name: Option<&str>,
    theme: Option<&str>,
) -> zellij_core::Result<()> {
    let target = target.unwrap_or_else(|| {
        if zellij_core::Project::discover(Utf8Path::new(dir)).is_ok() {
            "static"
        } else {
            "portable"
        }
    });
    if target != "portable" && (name.is_some() || theme.is_some()) {
        return Err(zellij_core::Error::InvalidArgument(
            "--name and --theme apply only to portable builds; configure hosted sites in site.yaml"
                .into(),
        ));
    }
    if target != "image" && tag.is_some() {
        return Err(zellij_core::Error::InvalidArgument(
            "--tag requires --target image".into(),
        ));
    }
    match target {
        "portable" => {
            let output = build_portable(dir, out, name, theme)?;
            println!(
                "  Open {} directly in your browser, or serve the folder over HTTP.",
                output.join("index.html")
            );
            Ok(())
        }
        "static" => build(dir, target, out),
        "image" => {
            if out.is_some() {
                return Err(zellij_core::Error::InvalidArgument(
                    "--out does not apply to image builds; use --tag".into(),
                ));
            }
            image_build(dir, tag, None)
        }
        _ => Err(zellij_core::Error::InvalidArgument(format!(
            "unknown build target {target:?}; run zel list targets"
        ))),
    }
}

fn build_portable(
    dir: &str,
    out: Option<&str>,
    name: Option<&str>,
    theme: Option<&str>,
) -> zellij_core::Result<Utf8PathBuf> {
    let source = Utf8Path::new(dir);
    let output = out
        .map(Utf8PathBuf::from)
        .map(Ok)
        .unwrap_or_else(|| zellij_docs::default_output(source))?;
    let report = zellij_docs::build(
        source,
        &output,
        zellij_docs::BuildOptions {
            name,
            theme: theme.unwrap_or("cupertino"),
        },
    )?;
    for warning in report.warnings {
        eprintln!("warning: {warning}");
    }
    println!("✓ {output}  ({} HTML pages)", report.pages);
    Ok(output)
}

fn build(dir: &str, target: &str, out: Option<&str>) -> zellij_core::Result<()> {
    use zellij_engine::{BuildMode, Engine as _};

    let mode = match target {
        "static" => BuildMode::Static,
        "image" => BuildMode::Image,
        other => {
            eprintln!("error: no target called \"{other}\"");
            eprintln!("       zel list targets");
            std::process::exit(2);
        }
    };

    let content = resolve_content(dir)?;
    let out = Utf8PathBuf::from(out.unwrap_or("dist"));
    let engine = zellij_engine::ContainerEngine::detect(None)?;

    println!("  Engine   {}", engine.image());
    println!("  Content  {content}");
    println!("  Target   {target} → {out}");
    println!();

    match engine.build(&content, &out, mode) {
        Err(err @ zellij_core::Error::MountEmpty { .. }) => {
            report_empty_mount(&err);
            std::process::exit(err.exit_code());
        }
        Err(err) => Err(err),
        Ok(()) => {
            let pages = std::fs::read_dir(&out)
                .map(|entries| entries.flatten().count())
                .unwrap_or(0);

            // A build that produced nothing did not succeed, whatever the
            // container said.
            //
            // `MountEmpty` above catches this from the input side — a content
            // folder the runtime cannot reach mounts as empty. The *output*
            // side fails the same way and had no such check: the container
            // writes its pages into its own copy of a path that was never
            // shared, exits cleanly, and the directory the caller is looking at
            // stays empty. The count was already being printed, so this
            // reported "✓ … (0 entries)" — a tick over nothing.
            //
            // Found by pointing `--out` at /tmp on a Colima host. It is the
            // same mistake this binary already explains for the input folder,
            // in the same message, and it deserved the same answer.
            if pages == 0 {
                eprintln!();
                eprintln!("error: the build wrote nothing to {out}");
                eprintln!();
                eprintln!("  The engine reported success and the output folder is empty, which");
                eprintln!("  happens when the container cannot reach the path you asked for: a");
                eprintln!("  bind mount of an unshared directory succeeds and shares nothing, so");
                eprintln!("  the pages were written inside the container and thrown away with it.");
                eprintln!();
                eprintln!("  Colima shares $HOME and not /tmp. Either write somewhere under your");
                eprintln!("  home directory, or share the path:");
                eprintln!();
                eprintln!("    colima stop && colima start --mount '/tmp:w'");
                eprintln!();
                eprintln!("  Docker Desktop: Settings → Resources → File sharing.");
                std::process::exit(1);
            }

            println!();
            println!("✓ {out}  ({pages} entries)");

            // A sitemap needs absolute URLs, so without `siteUrl` the engine
            // has nothing to write and produces an empty one. It is valid XML
            // and useless, and nothing else in the build says so — the site
            // ships, search engines get an empty index, and nobody finds out
            // for months.
            let sitemap = out.join("sitemap.xml");
            if let Ok(body) = std::fs::read_to_string(&sitemap)
                && !body.contains("<loc>")
            {
                println!();
                println!("! {sitemap} is empty.");
                println!("  A sitemap lists absolute URLs, so it needs the site's origin.");
                println!("  Set siteUrl in site.yaml:");
                println!();
                println!("    siteUrl: https://docs.example.com");
            }

            println!();
            println!("  Serve it from any static host.");
            Ok(())
        }
    }
}

/// A tag for a site nobody named: the content folder's own name.
fn default_tag(content: &Utf8Path) -> String {
    let name = content
        .canonicalize_utf8()
        .ok()
        .as_deref()
        .and_then(Utf8Path::file_name)
        .or_else(|| content.file_name())
        .unwrap_or("zellij-site")
        .to_ascii_lowercase();

    // Docker will not take a tag with characters a directory happily has.
    let safe: String = name
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '.' {
                c
            } else {
                '-'
            }
        })
        .collect();

    format!("{}:latest", safe.trim_matches('-'))
}

fn image_build(dir: &str, tag: Option<&str>, features: Option<&str>) -> zellij_core::Result<()> {
    let content = resolve_content(dir)?;
    let engine = zellij_engine::ContainerEngine::detect(None)?;
    let tag = tag
        .map(str::to_owned)
        .unwrap_or_else(|| default_tag(&content));

    println!("  Engine   {}", engine.image());
    println!("  Content  {content}");
    println!("  Tag      {tag}");
    println!();

    engine.image_build(&content, &tag, features, None)?;

    println!();
    println!("✓ {tag}");
    println!("  Run it:   zel image run {tag}");
    println!("  Push it:  zel image push {tag} --registry ghcr.io/you");
    Ok(())
}

fn image_run(tag: Option<&str>, port: u16) -> zellij_core::Result<()> {
    let engine = zellij_engine::ContainerEngine::detect(None)?;
    let tag = match tag {
        Some(tag) => tag.to_owned(),
        None => default_tag(&resolve_content(".")?),
    };

    println!("  Serving http://localhost:{port}  — Ctrl-C to stop");
    println!();
    engine.image_run(&tag, port)
}

fn image_push(tag: &str, registry: Option<&str>) -> zellij_core::Result<()> {
    let engine = zellij_engine::ContainerEngine::detect(None)?;
    engine.image_push(tag, registry)
}

fn image_dockerfile(dir: &str) -> zellij_core::Result<()> {
    let engine = zellij_engine::ContainerEngine::detect(None)?;
    let _ = resolve_content(dir)?;
    print!("{}", zellij_engine::image::dockerfile(engine.image(), None));
    Ok(())
}

// ------------------------------------------------------------------ pipelines

/// Ask GitHub to run the deploy pipeline.
///
/// Shells out to `gh` rather than speaking to the API directly. `zel` carries
/// no HTTP client, and pulling one in — with its TLS stack — to make a single
/// POST would add megabytes to a binary every user downloads, to save a
/// dependency most people deploying to GitHub already have.
///
/// When `gh` is absent the exact request is printed instead. A tool that
/// cannot do the thing should at least hand over the thing.
fn ci_publish(workflow: &str, git_ref: &str) -> zellij_core::Result<()> {
    let repo = github_repo();

    if !on_path("gh") {
        let slug = repo.unwrap_or_else(|| "<owner>/<repo>".into());
        println!("gh is not installed. This is the request it would have made:");
        println!();
        println!("  curl -X POST \\");
        println!("    -H \"Authorization: Bearer $GITHUB_TOKEN\" \\");
        println!("    -H \"Accept: application/vnd.github+json\" \\");
        println!(
            "    https://api.github.com/repos/{slug}/actions/workflows/{workflow}/dispatches \\"
        );
        println!("    -d '{{\"ref\":\"{git_ref}\"}}'");
        println!();
        println!("  The token needs Actions: write on that repository.");
        return Ok(());
    }

    let mut command = std::process::Command::new("gh");
    command.args(["workflow", "run", workflow, "--ref", git_ref]);
    if let Some(ref slug) = repo {
        command.args(["--repo", slug]);
    }

    let status = match command.status() {
        Ok(status) => status,
        Err(err) => {
            eprintln!("error: could not run gh: {err}");
            std::process::exit(1);
        }
    };

    if !status.success() {
        eprintln!("error: gh could not start {workflow}");
        eprintln!("       If it says the workflow was not found, it has to be on the");
        eprintln!("       repository's default branch before GitHub will accept a run.");
        std::process::exit(1);
    }

    println!("✓ {workflow} queued on {git_ref}");
    if let Some(slug) = repo {
        println!("  https://github.com/{slug}/actions");
    }
    Ok(())
}

/// `owner/repo` from the `origin` remote, when there is one to read.
fn github_repo() -> Option<String> {
    let out = std::process::Command::new("git")
        .args(["remote", "get-url", "origin"])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let url = String::from_utf8(out.stdout).ok()?;
    let url = url.trim().trim_end_matches(".git").to_owned();

    // Both forms: `git@host:owner/repo` and `https://host/owner/repo`. The
    // host may be an SSH alias — `github-mylife-inc` rather than github.com —
    // so this matches on shape rather than on the hostname.
    let tail = url
        .rsplit_once(':')
        .map(|(_, t)| t.to_owned())
        .unwrap_or(url);
    let parts: Vec<&str> = tail.rsplit('/').take(2).collect();
    (parts.len() == 2).then(|| format!("{}/{}", parts[1], parts[0]))
}

/// Is this program on the PATH?
fn on_path(program: &str) -> bool {
    std::process::Command::new(program)
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Refuse to write a Netlify pipeline that cannot deploy.
///
/// The workflow needs two things at run time: the project id, which the file
/// itself can carry, and an account token, which it cannot. Writing the file
/// without arranging the token produces a repository that looks configured and
/// silently publishes nothing — the failure mode worth spending a hard error to
/// avoid, because it is discovered days later by someone wondering why the site
/// is stale.
///
/// `--no-provision` opts out deliberately, for generating a file to review
/// before the credentials exist.
fn preflight_netlify(site_id: Option<&str>) {
    let token = std::env::var("NETLIFY_AUTH_TOKEN")
        .ok()
        .map(|v| v.trim().to_owned())
        .filter(|v| !v.is_empty());

    if site_id.is_none() {
        eprintln!("error: no Netlify project id");
        eprintln!("       pass --site, or export NETLIFY_SITE_ID.");
        eprintln!("       `netlify sites:list` prints it, and it is the \"Project ID\"");
        eprintln!("       on the project's Configuration page.");
        eprintln!();
        eprintln!("       It is written into the workflow, not stored as a secret —");
        eprintln!("       it identifies a project and grants nothing on its own.");
        std::process::exit(2);
    }

    let Some(token) = token else {
        eprintln!("error: no Netlify token to upload");
        eprintln!("       export NETLIFY_AUTH_TOKEN, then re-run.");
        eprintln!("       Create one at app.netlify.com → User settings → Applications.");
        eprintln!("       `netlify login` stores a token for this machine but will not");
        eprintln!("       print one you can give to CI.");
        eprintln!();
        eprintln!("       Or --no-provision to write the workflow anyway; it will not");
        eprintln!("       deploy until the secret exists.");
        std::process::exit(2);
    };

    if !on_path("gh") {
        eprintln!("error: gh is needed to upload the token, and is not installed");
        eprintln!("       Install it, or add the secret yourself:");
        eprintln!("         NETLIFY_AUTH_TOKEN, in Settings → Secrets and variables → Actions");
        eprintln!("       then re-run with --no-provision.");
        std::process::exit(2);
    }

    // Piped rather than passed as an argument: an argument is visible in the
    // process list to every other user on the machine.
    let outcome = (|| -> std::io::Result<std::process::ExitStatus> {
        use std::io::Write as _;
        let mut child = std::process::Command::new("gh")
            .args(["secret", "set", "NETLIFY_AUTH_TOKEN"])
            .stdin(std::process::Stdio::piped())
            .spawn()?;
        child
            .stdin
            .as_mut()
            .expect("stdin was piped")
            .write_all(token.as_bytes())?;
        child.wait()
    })();

    match outcome {
        Ok(status) if status.success() => {
            println!("✓ NETLIFY_AUTH_TOKEN uploaded to the repository's secrets");
        }
        Ok(_) => {
            eprintln!("error: gh could not set the secret");
            eprintln!("       The usual cause is a token without permission to write");
            eprintln!("       secrets. A fine-grained token needs Secrets: read and write");
            eprintln!("       on this repository; `gh auth refresh -s repo` covers the");
            eprintln!("       classic case.");
            eprintln!();
            eprintln!("       Nothing was written. Fix the permission and re-run.");
            std::process::exit(1);
        }
        Err(err) => {
            eprintln!("error: could not run gh: {err}");
            std::process::exit(1);
        }
    }
}

#[allow(clippy::fn_params_excessive_bools)]
fn ci_init(
    system: &str,
    deploy: Option<&str>,
    target: &str,
    content: Option<&str>,
    site: Option<&str>,
    force: bool,
    no_provision: bool,
) -> zellij_core::Result<()> {
    use zellij_scaffold::pipeline::{self, CiOptions, Publish};

    if system != "github" {
        eprintln!("error: no generator for \"{system}\" yet");
        eprintln!("       github is the only one implemented; gitlab and circle are planned");
        std::process::exit(2);
    }

    // Before anything else. If the file is already there, "that exists" is
    // what somebody needs to hear — not a question about which folder to use
    // for a file this is not going to write.
    let path = Utf8PathBuf::from(".github/workflows/docs.yml");
    if path.exists() && !force {
        eprintln!("error: {path} already exists");
        eprintln!("       --force overwrites it. The file says to regenerate rather");
        eprintln!("       than edit, so regenerating has to be possible.");
        eprintln!("       Or write elsewhere: zel ci init github > my-pipeline.yml");
        std::process::exit(2);
    }

    // What it publishes is one decision, expressed two ways. `--deploy netlify`
    // says where; `--target image` says what. Asking for both is a contradiction
    // rather than a combination.
    let publish = match (deploy, target) {
        (Some("netlify"), _) => Publish::Netlify,
        (Some(other), _) => {
            eprintln!("error: no pipeline for \"{other}\" yet");
            eprintln!("       netlify is the only deploy target implemented");
            std::process::exit(2);
        }
        (None, "image") => Publish::Image,
        (None, _) => Publish::Nothing,
    };

    // The path as the *pipeline* will use it, which is relative to the
    // repository root rather than to wherever this was run. Discovery only
    // finds ./content, so anything else has to be said — and guessing "docs"
    // because that is the common name would write a pipeline that points at
    // nothing for everyone who chose a different one.
    let content = match content {
        Some(given) => given.to_owned(),
        None => match resolve_content(".") {
            Ok(path) => path.to_string(),
            Err(_) => {
                eprintln!("error: no content folder found, and none given");
                eprintln!("       the pipeline needs the path as your repository sees it:");
                eprintln!("         zel ci init {system} --content docs");
                std::process::exit(2);
            }
        },
    };

    // The site id, in order of how deliberate it is: the flag, then the
    // environment. Only consulted for a Netlify deploy.
    let site_id = if matches!(publish, Publish::Netlify) {
        site.map(str::to_owned).or_else(|| {
            std::env::var("NETLIFY_SITE_ID")
                .ok()
                .map(|v| v.trim().to_owned())
                .filter(|v| !v.is_empty())
        })
    } else {
        None
    };

    // Fail before writing anything, rather than leaving a workflow that looks
    // finished and deploys nothing.
    //
    // Only when a deploy was asked for. `zel ci init github` with no `--deploy`
    // writes a check job that needs no credentials at all, and demanding a
    // token for that would be theatre.
    if matches!(publish, Publish::Netlify) && !no_provision {
        preflight_netlify(site_id.as_deref());
    }

    let options = CiOptions {
        content,
        publish,
        registry: "ghcr.io".to_owned(),
        site_id: site_id.clone(),
    };

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&path, pipeline::github(&options))?;

    println!("✓ {path}");
    println!("  Validates {} on every pull request.", options.content);

    // What was resolved, not merely what is exported — `--site` counts.
    let netlify = pipeline::NetlifyEnv::resolved(site_id.clone());
    let after = pipeline::afterwards(publish, &netlify);
    if !after.is_empty() {
        println!();
        println!("  Before it can publish:");
        for line in after {
            println!("    {line}");
        }
    }

    Ok(())
}

// ------------------------------------------------------------------ workflows

/// detect → convert → check, and stop at the first thing that fails.
///
/// It does nothing the three commands cannot. It exists because forgetting to
/// validate after a conversion is the mistake everybody makes once.
fn migrate(source: &str, out: &str) -> zellij_core::Result<()> {
    detect(source, false)?;
    println!();

    convert(source, out, None, false)?;
    println!();

    check(out, zellij_engine::CheckOptions::default())
}

fn doctor() -> zellij_core::Result<()> {
    println!("zel            {}  ({})", env!("CARGO_PKG_VERSION"), TARGET);
    println!();
    println!("Container runtime");

    match zellij_engine::ContainerEngine::detect(None) {
        Ok(engine) => {
            let runtime = engine.runtime();
            println!(
                "  ✓ {:<10} {} via {}",
                runtime.program,
                runtime.version,
                runtime.endpoint()
            );

            let state = if engine.image_is_local() {
                "local"
            } else {
                "not pulled — the first build will fetch it"
            };
            println!("  {:<12} {}  ({state})", "· engine", engine.image());
        }
        Err(err) => {
            println!("  ✗ none");
            for line in err.to_string().lines() {
                println!("    {line}");
            }
            println!();
            println!("  Commands that only read the content model still work:");
            println!("  new, explain, list, context, detect, convert.");
            return Err(err);
        }
    }

    println!();
    println!("Content");
    match resolve_content(".") {
        Ok(path) => println!("  ✓ {path}"),
        Err(_) => println!("  · none here — `zel new docs` writes one"),
    }

    Ok(())
}

/// The triple this binary was built for, for bug reports.
const TARGET: &str = env!("ZEL_TARGET");

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

fn new_project(
    name: &str,
    product_lang: &str,
    docs_dir: &str,
    product_dir: &str,
) -> zellij_core::Result<()> {
    use zellij_scaffold::project::{self, ProjectOptions};

    if !project::known_languages().contains(&product_lang) {
        eprintln!("error: no hint for \"{product_lang}\"");
        eprintln!("       known: {}", project::known_languages().join(", "));
        eprintln!("       any of them only prints a command; nothing is generated");
        std::process::exit(2);
    }

    let target = Utf8PathBuf::from(name);
    let options = ProjectOptions {
        name: name.to_owned(),
        docs_dir: docs_dir.to_owned(),
        product_dir: product_dir.to_owned(),
        product_lang: product_lang.to_owned(),
    };

    let written = project::project(&options).write(&target, false)?;

    println!("✓ {target}");
    println!(
        "  {:<24} the documentation, as Zellij content",
        format!("{docs_dir}/")
    );
    println!("  {:<24} yours", format!("{product_dir}/"));
    println!(
        "  {:<24} validates on every pull request, image on main",
        ".github/workflows/docs.yml"
    );
    println!("  {} files", written.len());

    println!();
    println!("  Write:  zel dev {target}/{docs_dir}");

    // A hint rather than a scaffold. `cargo init` is better than anything a
    // documentation tool would write in its place, and it is the command
    // somebody would have run anyway.
    match project::product_hint(product_lang, name) {
        Some(hint) => {
            println!("  Code:   cd {target}/{product_dir} && {hint}");
        }
        None => {
            println!("  Code:   {target}/{product_dir} is empty — start it with your own tooling");
        }
    }

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

    // The parent is an explicit user-selected root; the leaf must be new,
    // including when an existing dangling symlink occupies that name.
    let parent = path
        .parent()
        .filter(|p| !p.as_str().is_empty())
        .unwrap_or(camino::Utf8Path::new("."));
    let name = path
        .file_name()
        .ok_or_else(|| zellij_core::files::unsafe_path(&path, "expected a filename"))?;
    zellij_core::files::write_files(parent, &[(camino::Utf8Path::new(name), &body)], false)?;

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
    let written = scaffold.write_new(&root)?;

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

    let written = zellij_scaffold::guide(name, &title, order).write_new(&root)?;

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

fn print_json(value: &serde_json::Value) -> zellij_core::Result<()> {
    let mut stdout = std::io::stdout().lock();
    serde_json::to_writer_pretty(&mut stdout, value).map_err(std::io::Error::other)?;
    writeln!(stdout)?;
    Ok(())
}

fn list_formats(as_json: bool) -> zellij_core::Result<()> {
    let mut converters: Vec<_> = zellij_convert::all().collect();
    converters.sort_by_key(|converter| converter.id());

    if as_json {
        let formats: Vec<_> = converters
            .iter()
            .map(|converter| {
                json!({
                    "id": converter.id(), "label": converter.label(),
                })
            })
            .collect();
        return print_json(&json!({ "formats": formats }));
    }

    if converters.is_empty() {
        println!("No converters are registered in this build.");
        return Ok(());
    }

    for converter in converters {
        println!("{:<14} {}", converter.id(), converter.label());
    }

    Ok(())
}

// Portable Markdown is rendered natively; structured sites use the engine.
fn list_targets(as_json: bool) -> zellij_core::Result<()> {
    let targets = [
        (
            "static",
            "A hosted YAML/MDX website. Requires HTTP and the container engine.",
        ),
        (
            "image",
            "A container running the site on port 3000. Any registry. ~150 MB.",
        ),
        (
            "portable",
            "A Markdown folder rendered natively. Open index.html without a server.",
        ),
    ];
    if as_json {
        let targets: Vec<_> = targets
            .iter()
            .map(|(id, description)| {
                json!({
                    "id": id, "description": description,
                })
            })
            .collect();
        return print_json(&json!({ "targets": targets }));
    }
    for (name, description) in targets {
        println!("{name:<8} {description}");
    }

    Ok(())
}

fn list_sections(as_json: bool) -> zellij_core::Result<()> {
    let mut sections = Vec::new();
    for section_type in zellij_core::schema::section_types() {
        let fields = zellij_core::schema::fields_of(&section_type).unwrap_or_default();
        let required = fields.iter().filter(|field| field.required).count();
        if as_json {
            sections.push(json!({
                "type": section_type, "fields": fields.len(), "required": required,
            }));
        } else {
            println!(
                "{section_type:<16} {} fields, {required} required",
                fields.len()
            );
        }
    }

    if as_json {
        return print_json(&json!({ "sections": sections }));
    }
    println!("\nRun `zel explain <type>` for any of them.");
    Ok(())
}

fn list_themes(as_json: bool) -> zellij_core::Result<()> {
    let themes = zellij_core::schema::themes();
    if as_json {
        return print_json(&json!({ "themes": themes }));
    }
    for theme in themes {
        println!("{:<14} {} — {}", theme.name, theme.label, theme.description);
    }
    Ok(())
}

/// A section's fields, defaults and allowed values.
///
/// This is the command that exists so nobody has to remember every set of
/// fields. It prints what is required first, because that is what someone is
/// actually asking when they run it.
fn explain(section_type: &str, as_json: bool) -> zellij_core::Result<()> {
    use zellij_core::schema;

    let section = schema::section_schema(section_type).ok_or_else(|| {
        zellij_core::Error::InvalidArgument(format!(
            "`{section_type}` is not a section type. Try `zel list sections`."
        ))
    })?;
    if as_json {
        return print_json(section);
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

fn write_context(file: Option<&str>, as_json: bool) -> zellij_core::Result<()> {
    let body = if as_json {
        zellij_core::schema::SCHEMA_JSON
    } else {
        zellij_core::schema::CONTEXT_MARKDOWN
    };
    match file {
        Some(path) => {
            std::fs::write(path, body)?;
            if as_json {
                eprintln!("Wrote the Zellij schemas to {path}");
            } else {
                println!("Wrote the Zellij reference to {path}");
            }
        }
        None => std::io::stdout().lock().write_all(body.as_bytes())?,
    }

    Ok(())
}

fn detect(dir: &str, as_json: bool) -> zellij_core::Result<()> {
    let source = camino::Utf8Path::new(dir);
    if !std::fs::metadata(source)?.is_dir() {
        return Err(zellij_core::Error::InvalidArgument(format!(
            "{source} is not a directory"
        )));
    }
    let mut found = zellij_convert::detect(source);
    // Inventory registration order can differ across native builds. Keep ties
    // deterministic without changing the converters' confidence scores.
    found.sort_by(|a, b| {
        b.1.partial_cmp(&a.1)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.0.id().cmp(b.0.id()))
    });

    if as_json {
        let matches: Vec<_> = found
            .iter()
            .map(|(converter, confidence)| {
                json!({
                    "id": converter.id(),
                    "label": converter.label(),
                    "confidence": confidence.0,
                    "strength": if *confidence >= zellij_convert::Confidence::CERTAIN {
                        "certain"
                    } else {
                        "likely"
                    },
                })
            })
            .collect();
        return print_json(&json!({ "source": dir, "matches": matches }));
    }

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
    zellij_core::files::disjoint(source, out)?;
    zellij_core::files::empty_directory(out)?;

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
    let files: Vec<_> = conversion
        .files
        .iter()
        .map(|file| (file.path.as_path(), file.body.as_bytes()))
        .chain(
            conversion
                .assets
                .iter()
                .map(|asset| (asset.path.as_path(), asset.body.as_slice())),
        )
        .collect();
    zellij_core::files::preflight(
        out,
        &files.iter().map(|(path, _)| *path).collect::<Vec<_>>(),
        false,
    )?;

    if dry_run {
        println!("\nWould write {} files into {out}:", files.len());
        for (path, _) in &files {
            println!("  {path}");
        }
    } else {
        zellij_core::files::write_files(out, &files, false)?;
        println!(
            "\nWrote {} files into {out} ({} assets)",
            files.len(),
            conversion.assets.len()
        );
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
