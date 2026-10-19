//! Native, offline Markdown publication. No container, Node, fetch-based router
//! or build-time CDN is involved. Hosted YAML/MDX sites remain the engine's job.
use camino::{Utf8Path, Utf8PathBuf};
use pulldown_cmark::{Event, Tag};
use serde_json::json;
use std::{collections::BTreeMap, fs};
use zellij_core::{Error, Result, files, output::StaticOutput, site::SiteInput};

mod config;
mod diagnostics;
use diagnostics::Diagnostics;
pub use diagnostics::{CheckReport, Code, Diagnostic, Severity};
mod hosted;
mod links;
mod markdown;
mod math;
mod mermaid;
mod mindmap;
mod page;
pub mod server;
pub mod watch;
include!(concat!(env!("OUT_DIR"), "/fonts.rs"));

#[derive(Default)]
pub struct BuildOptions<'a> {
    pub name: Option<&'a str>,
    pub theme: Option<&'a str>,
}

/// Validate portable presentation settings without scanning or building content.
pub fn validate_settings(value: serde_yaml_ng::Value) -> Result<()> {
    config::Config::load(value).map(|_| ())
}

pub fn has_config(root: &Utf8Path) -> bool {
    config::present(root)
}

fn validate_theme(theme: &str) -> Result<()> {
    if !zellij_core::schema::themes()
        .iter()
        .any(|t| t.name == theme)
    {
        return Err(Error::InvalidArgument(format!(
            "unknown theme {theme:?}; run zel list themes"
        )));
    }
    Ok(())
}
pub struct BuildReport {
    pub pages: usize,
    pub warnings: Vec<String>,
}

/// A complete export that has not replaced the previous site yet.
/// Preview can hold its publication lock for only the final directory swap.
pub struct PreparedBuild {
    stage: StaticOutput,
    report: BuildReport,
}
impl PreparedBuild {
    pub fn publish(self) -> Result<BuildReport> {
        self.stage.publish()?;
        Ok(self.report)
    }
}
struct Page {
    source: Utf8PathBuf,
    output: Utf8PathBuf,
    markdown: markdown::Markdown,
}

// Filesystem paths become URL paths at this boundary. On Unix a backslash is
// an authored filename character and remains invalid, not a separator.
fn url_path(path: &Utf8Path) -> Utf8PathBuf {
    #[cfg(windows)]
    return path.as_str().replace('\\', "/").into();
    #[cfg(not(windows))]
    path.to_owned()
}

fn portable_path(path: &Utf8Path) -> Result<()> {
    files::relative_path(path)?;
    for part in path.as_str().split('/') {
        let stem = part.split('.').next().unwrap().to_ascii_uppercase();
        let reserved = [
            "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7",
            "COM8", "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
            "COM¹", "COM²", "COM³", "LPT¹", "LPT²", "LPT³",
        ];
        if part.ends_with(['.', ' '])
            || part
                .chars()
                .any(|c| c.is_control() || "<>\"|?*".contains(c))
            || reserved.contains(&stem.as_str())
        {
            return Err(files::unsafe_path(
                path,
                "name cannot be represented portably on Windows",
            ));
        }
    }
    Ok(())
}

fn ignored_name(name: &str) -> bool {
    name.starts_with('.') || name == "node_modules"
}

fn discover(root: &Utf8Path) -> Result<Vec<Utf8PathBuf>> {
    let mut found = Vec::new();
    for entry in walkdir::WalkDir::new(root)
        .follow_links(false)
        .sort_by_file_name()
        .into_iter()
        .filter_entry(|e| e.depth() == 0 || !ignored_name(&e.file_name().to_string_lossy()))
    {
        let entry = entry.map_err(|e| files::unsafe_path(root, e.to_string()))?;
        let path = Utf8Path::from_path(entry.path())
            .ok_or_else(|| files::unsafe_path(root, "non-UTF-8 filename"))?;
        if entry.file_type().is_symlink() {
            return Err(files::unsafe_path(
                path,
                "symbolic links are not allowed inside documentation",
            ));
        }
        if !entry.file_type().is_file() {
            continue;
        }
        if path
            .extension()
            .is_some_and(|e| e.eq_ignore_ascii_case("mdx"))
        {
            return Err(files::unsafe_path(
                path,
                "portable builds accept Markdown, not MDX; use the hosted static target for MDX",
            ));
        }
        if path
            .extension()
            .is_some_and(|e| e.eq_ignore_ascii_case("md") || e.eq_ignore_ascii_case("markdown"))
        {
            let canonical = path.canonicalize_utf8()?;
            let relative = url_path(
                canonical
                    .strip_prefix(root)
                    .map_err(|_| files::unsafe_path(path, "document escapes source"))?,
            );
            portable_path(&relative)?;
            if relative
                .components()
                .next()
                .is_some_and(|p| p.as_str().eq_ignore_ascii_case("_zellij"))
            {
                return Err(files::unsafe_path(
                    path,
                    "_zellij is reserved for generated assets",
                ));
            }
            found.push(relative);
        }
    }
    if found.is_empty() {
        return Err(files::unsafe_path(
            root,
            "no .md or .markdown documents found",
        ));
    }
    Ok(found)
}

pub fn default_output(source: impl Into<SiteInput>) -> Result<Utf8PathBuf> {
    let site = zellij_core::site::Site::load(source)?;
    site.require(zellij_core::site::Renderer::Portable)?;
    if site.configured {
        Ok(site.output)
    } else {
        let name = site
            .root
            .file_name()
            .ok_or_else(|| files::unsafe_path(&site.root, "specify --out for a filesystem root"))?;
        Ok(site.root.with_file_name(format!("{name}-site")))
    }
}

pub fn build(
    source: impl Into<SiteInput>,
    output: &Utf8Path,
    options: BuildOptions<'_>,
) -> Result<BuildReport> {
    prepare(source, output, options)?.publish()
}

/// Validate and render in memory. No output directory or temporary files are created.
/// This checks content, not destination permissions, ownership, or available disk space.
pub fn check(source: impl Into<SiteInput>, strict: bool) -> CheckReport {
    let source = source.into();
    let mut diagnostics = Diagnostics::new(source.root());
    let compiled = compile(
        &source,
        BuildOptions::default(),
        &mut diagnostics,
        None,
        false,
    );
    let pages = compiled.as_ref().ok().map(|site| site.pages);
    let exit_code = match compiled {
        Ok(_) if strict && !diagnostics.items.is_empty() => 3,
        Ok(_) => 0,
        Err(Error::Io(_)) => 1,
        Err(_) => 3,
    };
    CheckReport {
        schema_version: 1,
        target: "portable",
        root: diagnostics.root.to_string(),
        strict,
        ok: exit_code == 0,
        pages,
        diagnostics: diagnostics.items,
        exit_code,
    }
}

struct CompiledSite {
    root: Utf8PathBuf,
    files: BTreeMap<Utf8PathBuf, Vec<u8>>,
    pages: usize,
    features: Vec<&'static str>,
}

pub fn prepare(
    source: impl Into<SiteInput>,
    output: &Utf8Path,
    options: BuildOptions<'_>,
) -> Result<PreparedBuild> {
    prepare_layout(source, output, options, None)
}

pub(crate) fn prepare_layout(
    source: impl Into<SiteInput>,
    output: &Utf8Path,
    options: BuildOptions<'_>,
    configured: Option<bool>,
) -> Result<PreparedBuild> {
    let source = source.into();
    let mut diagnostics = Diagnostics::new(source.root());
    let compiled = compile(&source, options, &mut diagnostics, configured, false)?;
    let stage = StaticOutput::prepare(&compiled.root, output)?;
    let writes: Vec<_> = compiled
        .files
        .iter()
        .map(|(path, body)| (path.as_path(), body.as_slice()))
        .collect();
    files::write_files(&stage.directory, &writes, false)?;
    Ok(PreparedBuild {
        stage,
        report: BuildReport {
            pages: compiled.pages,
            warnings: diagnostics.warnings(),
        },
    })
}

fn compile(
    source: &SiteInput,
    options: BuildOptions<'_>,
    diagnostics: &mut Diagnostics,
    configured: Option<bool>,
    hosted: bool,
) -> Result<CompiledSite> {
    let selected = diagnostics.capture(
        source.root().canonicalize_utf8().map_err(Error::from),
        Code::Source,
        None,
    )?;
    diagnostics.root = selected.clone();
    let selected_config = source.config_path();
    let config_name = Utf8Path::new(selected_config.file_name().unwrap_or(config::FILE));
    let site = diagnostics.capture(
        zellij_core::site::Site::load(source),
        Code::Configuration,
        Some(config_name),
    )?;
    if configured.is_some_and(|expected| expected != site.configured) {
        return diagnostics.capture(
            Err(Error::InvalidArgument(format!(
                "site layout changed; restore {} or restart preview",
                site.config_path
            ))),
            Code::Configuration,
            Some(config_name),
        );
    }
    diagnostics.capture(
        site.require(zellij_core::site::Renderer::Portable),
        Code::Configuration,
        Some(config_name),
    )?;
    // Flat sources retain their original meaning even if hosted markers exist;
    // configured sources always compile content/, never their generated output.
    let root = if site.configured {
        site.content
    } else {
        selected.clone()
    };
    diagnostics.root = root.clone();
    let relative_config = if site.configured {
        Utf8Path::new("..").join(config_name)
    } else {
        config_name.to_owned()
    };
    let config_file = relative_config.as_path();
    let config = diagnostics.capture(
        config::Config::load(site.portable),
        Code::Configuration,
        Some(config_file),
    )?;
    let name = options
        .name
        .or(config.name.as_deref())
        .unwrap_or_else(|| selected.file_name().unwrap_or("Documentation"));
    let theme = options
        .theme
        .or(config.theme.as_deref())
        .unwrap_or("cupertino");
    diagnostics.capture(
        validate_theme(theme),
        Code::Configuration,
        Some(config_file),
    )?;
    if name.trim().is_empty() {
        return diagnostics.capture(
            Err(Error::InvalidArgument("site name must not be empty".into())),
            Code::Configuration,
            Some(config_file),
        );
    }
    let sources = diagnostics.capture(discover(&root), Code::Source, None)?;
    let mut pages = Vec::new();
    let mut first_error = None;
    for source in &sources {
        let stem = source.file_stem().unwrap();
        let directory = source.parent().unwrap();
        let index = stem.eq_ignore_ascii_case("index")
            || (stem.eq_ignore_ascii_case("readme")
                && !sources.iter().any(|p| {
                    p.parent() == Some(directory)
                        && p.file_stem()
                            .is_some_and(|s| s.eq_ignore_ascii_case("index"))
                }));
        let output = if index {
            url_path(&directory.join("index.html"))
        } else {
            source.with_extension("html")
        };
        let fallback = if stem.eq_ignore_ascii_case("index") || stem.eq_ignore_ascii_case("readme")
        {
            directory.file_name().unwrap_or(name)
        } else {
            stem
        };
        let parsed = diagnostics
            .capture(
                fs::read_to_string(root.join(source)).map_err(Error::from),
                Code::Source,
                Some(source),
            )
            .and_then(|text| {
                diagnostics.capture(
                    markdown::parse(
                        &text,
                        fallback,
                        config.markdown.math,
                        config.markdown.mermaid,
                    )
                    .map_err(|e| files::unsafe_path(source, e.to_string())),
                    Code::Markdown,
                    Some(source),
                )
            });
        let markdown = match parsed {
            Ok(markdown) => markdown,
            Err(error) => {
                first_error.get_or_insert(error);
                continue;
            }
        };
        if markdown.literal_html {
            diagnostics.raw_html(source);
        }
        pages.push(Page {
            source: source.clone(),
            output,
            markdown,
        });
    }
    if let Some(error) = first_error.take() {
        return Err(error);
    }
    if !pages.iter().any(|p| p.output == "index.html") {
        pages.push(Page {
            source: "".into(),
            output: "index.html".into(),
            markdown: diagnostics.capture(
                markdown::parse("", name, false, false),
                Code::Rendering,
                None,
            )?,
        });
    }
    pages.sort_by(|a, b| {
        (a.output != "index.html", a.markdown.order, &a.output).cmp(&(
            b.output != "index.html",
            b.markdown.order,
            &b.output,
        ))
    });
    let navigation = diagnostics.capture(
        config::Navigation::resolve(&config, &pages),
        Code::Configuration,
        Some(config_file),
    )?;
    let presentation = page::Presentation {
        name,
        theme,
        logo: config.brand.as_ref().map(|b| &b.logo),
    };
    let mut resolver = links::Resolver {
        root: &root,
        pages: &pages,
        assets: BTreeMap::new(),
    };
    if let Some((path, bytes)) =
        diagnostics.capture(config.logo(&root), Code::Configuration, Some(config_file))?
    {
        resolver.assets.insert(path, bytes);
    }
    let mut rewritten = Vec::new();
    for page in &pages {
        let mut events = page.markdown.events.clone();
        for event in &mut events {
            let (url, image) = match event {
                Event::Start(Tag::Link { dest_url, .. }) => (dest_url, false),
                Event::Start(Tag::Image { dest_url, .. }) => (dest_url, true),
                _ => continue,
            };
            match diagnostics.capture(
                resolver.rewrite(page, url, image),
                Code::Reference,
                Some(&page.source),
            ) {
                Ok(value) => *url = value.into(),
                Err(error) => {
                    first_error.get_or_insert(error);
                }
            }
        }
        rewritten.push(events);
    }
    if let Some(error) = first_error {
        return Err(error);
    }
    let hosted_files = if hosted {
        Some(hosted::emit(
            &config,
            name,
            theme,
            &pages,
            &navigation,
            &rewritten,
            &resolver.assets,
        )?)
    } else {
        None
    };
    let mut files = BTreeMap::<Utf8PathBuf, Vec<u8>>::new();
    for ((index, page), events) in pages.iter().enumerate().zip(rewritten) {
        let mut body = String::new();
        let highlighted = diagnostics.capture(
            markdown::highlight(events, config.markdown.mermaid),
            Code::Rendering,
            Some(&page.source),
        )?;
        pulldown_cmark::html::push_html(&mut body, highlighted.into_iter());
        if page.source.as_str().is_empty() {
            body = page::contents(page, &pages, &navigation);
        }
        let html = page::render(&presentation, page, index, &pages, &navigation, &body);
        if files
            .insert(page.output.clone(), html.into_bytes())
            .is_some()
        {
            return diagnostics.capture(
                Err(files::unsafe_path(
                    &page.output,
                    "documents have the same HTML output path",
                )),
                Code::OutputPath,
                Some(&page.source),
            );
        }
    }
    for (path, bytes) in resolver.assets {
        if files.insert(path.clone(), bytes).is_some() {
            return diagnostics.capture(
                Err(files::unsafe_path(
                    &path,
                    "asset collides with a generated page",
                )),
                Code::OutputPath,
                Some(&path),
            );
        }
    }
    for (path, bytes) in FONTS {
        files.insert(
            Utf8PathBuf::from(format!("_zellij/fonts/{path}")),
            bytes.to_vec(),
        );
    }
    if pages
        .iter()
        .any(|p| p.markdown.has_math || p.markdown.has_mermaid)
    {
        for (name, bytes) in math::ASSETS {
            files.insert(format!("_zellij/math/{name}").into(), bytes.to_vec());
        }
    }
    if pages.iter().any(|p| p.markdown.has_mermaid) {
        for (name, bytes) in mermaid::ASSETS {
            files.insert(format!("_zellij/mermaid/{name}").into(), bytes.to_vec());
        }
    }
    if pages.iter().any(|p| p.markdown.has_mindmap) {
        files.insert(
            "_zellij/mindmap.css".into(),
            include_bytes!("../assets/mindmap.css").to_vec(),
        );
    }
    files.insert(
        "_zellij/themes.css".into(),
        include_bytes!("../assets/themes.css").to_vec(),
    );
    files.insert(
        "_zellij/style.css".into(),
        include_bytes!("../assets/style.css").to_vec(),
    );
    files.insert(
        "_zellij/site.js".into(),
        include_bytes!("../assets/site.js").to_vec(),
    );
    let search: Vec<_> = pages.iter().map(|p| json!({
        "path": p.output.as_str(), "title": p.markdown.title, "text": p.markdown.text,
        "headings": p.markdown.headings.iter().map(|h| json!({"id": h.id, "title": h.title})).collect::<Vec<_>>()
    })).collect();
    let data = diagnostics
        .capture(
            serde_json::to_string(&search).map_err(|e| Error::Io(std::io::Error::other(e))),
            Code::Rendering,
            None,
        )?
        .replace('<', "\\u003c")
        .replace('\u{2028}', "\\u2028")
        .replace('\u{2029}', "\\u2029");
    files.insert(
        "_zellij/search.js".into(),
        format!("window.ZELLIJ_SEARCH={data};\n").into_bytes(),
    );
    // Check case-only ancestor conflicts too: A/x and a/y collide on Windows
    // even though their complete paths differ. Every failure precedes publish.
    let mut names = BTreeMap::new();
    for path in files.keys() {
        diagnostics.capture(portable_path(path), Code::OutputPath, None)?;
        for ancestor in path.ancestors().skip(1) {
            if files.contains_key(ancestor) {
                return diagnostics.capture(
                    Err(files::unsafe_path(
                        path,
                        "file/directory output path collision",
                    )),
                    Code::OutputPath,
                    None,
                );
            }
        }
        for component in path.ancestors().filter(|p| !p.as_str().is_empty()) {
            if let Some(previous) = names.insert(component.as_str().to_lowercase(), component)
                && previous != component
            {
                return diagnostics.capture(
                    Err(files::unsafe_path(path, "case-only output path collision")),
                    Code::OutputPath,
                    None,
                );
            }
        }
    }
    Ok(CompiledSite {
        root,
        files: hosted_files.unwrap_or(files),
        pages: pages.len(),
        features: [
            ("maths", pages.iter().any(|p| p.markdown.has_math)),
            ("diagrams", pages.iter().any(|p| p.markdown.has_mermaid)),
        ]
        .into_iter()
        .filter_map(|(name, used)| used.then_some(name))
        .collect(),
    })
}

pub struct HostedReport {
    pub pages: usize,
    pub warnings: Vec<String>,
    /// Optional hosted features actually required by the parsed documents.
    pub features: Vec<&'static str>,
}

/// Adapt validated portable Markdown to the hosted engine's inert document
/// format. Callers own the new/empty destination; authored content is never edited.
pub fn adapt_hosted(source: impl Into<SiteInput>, output: &Utf8Path) -> Result<HostedReport> {
    let source = source.into();
    let mut diagnostics = Diagnostics::new(source.root());
    let compiled = compile(
        &source,
        BuildOptions::default(),
        &mut diagnostics,
        None,
        true,
    )?;
    files::disjoint(&compiled.root, output)?;
    files::empty_directory(output)?;
    let writes: Vec<_> = compiled
        .files
        .iter()
        .map(|(p, b)| (p.as_path(), b.as_slice()))
        .collect();
    files::write_files(output, &writes, false)?;
    Ok(HostedReport {
        pages: compiled.pages,
        warnings: diagnostics.warnings(),
        features: compiled.features,
    })
}
