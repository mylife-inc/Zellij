//! Native, offline Markdown publication. No container, Node, fetch-based router
//! or build-time CDN is involved. Hosted YAML/MDX sites remain the engine's job.
use camino::{Utf8Path, Utf8PathBuf};
use pulldown_cmark::{Event, Tag};
use serde_json::json;
use std::{collections::BTreeMap, fs};
use zellij_core::{Error, Result, files, output::StaticOutput};

mod config;
mod links;
mod markdown;
mod page;
pub mod server;
pub mod watch;
include!(concat!(env!("OUT_DIR"), "/fonts.rs"));

#[derive(Default)]
pub struct BuildOptions<'a> {
    pub name: Option<&'a str>,
    pub theme: Option<&'a str>,
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

fn discover(root: &Utf8Path) -> Result<Vec<Utf8PathBuf>> {
    let mut found = Vec::new();
    for entry in walkdir::WalkDir::new(root)
        .follow_links(false)
        .sort_by_file_name()
        .into_iter()
        .filter_entry(|e| {
            e.depth() == 0
                || (!e.file_name().to_string_lossy().starts_with('.')
                    && e.file_name() != "node_modules")
        })
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

pub fn default_output(source: &Utf8Path) -> Result<Utf8PathBuf> {
    let source = source.canonicalize_utf8()?;
    let name = source
        .file_name()
        .ok_or_else(|| files::unsafe_path(&source, "specify --out for a filesystem root"))?;
    Ok(source.with_file_name(format!("{name}-site")))
}

pub fn build(
    source: &Utf8Path,
    output: &Utf8Path,
    options: BuildOptions<'_>,
) -> Result<BuildReport> {
    prepare(source, output, options)?.publish()
}

pub fn prepare(
    source: &Utf8Path,
    output: &Utf8Path,
    options: BuildOptions<'_>,
) -> Result<PreparedBuild> {
    let root = source.canonicalize_utf8()?;
    if !root.is_dir() {
        return Err(files::unsafe_path(&root, "expected a Markdown directory"));
    }
    files::disjoint(&root, output)?;
    let config = config::Config::load(&root)?;
    let name = options
        .name
        .or(config.name.as_deref())
        .unwrap_or_else(|| root.file_name().unwrap_or("Documentation"));
    let theme = options
        .theme
        .or(config.theme.as_deref())
        .unwrap_or("cupertino");
    validate_theme(theme)?;
    if name.trim().is_empty() {
        return Err(Error::InvalidArgument("site name must not be empty".into()));
    }
    let sources = discover(&root)?;
    let mut pages = Vec::new();
    let mut warnings = Vec::new();
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
        let markdown = markdown::parse(&fs::read_to_string(root.join(source))?, fallback)
            .map_err(|e| files::unsafe_path(source, e.to_string()))?;
        if markdown.literal_html {
            warnings.push(format!("{source}: raw HTML is displayed as text; portable Markdown does not execute HTML or MDX"));
        }
        pages.push(Page {
            source: source.clone(),
            output,
            markdown,
        });
    }
    if !pages.iter().any(|p| p.output == "index.html") {
        pages.push(Page {
            source: "".into(),
            output: "index.html".into(),
            markdown: markdown::parse("", name)?,
        });
    }
    pages.sort_by(|a, b| {
        (a.output != "index.html", a.markdown.order, &a.output).cmp(&(
            b.output != "index.html",
            b.markdown.order,
            &b.output,
        ))
    });
    let navigation = config::Navigation::resolve(&config, &pages)?;
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
    if let Some((path, bytes)) = config.logo(&root)? {
        resolver.assets.insert(path, bytes);
    }
    let mut files = BTreeMap::<Utf8PathBuf, Vec<u8>>::new();
    for (index, page) in pages.iter().enumerate() {
        let mut events = page.markdown.events.clone();
        for event in &mut events {
            match event {
                Event::Start(Tag::Link { dest_url, .. }) => {
                    *dest_url = resolver.rewrite(page, dest_url, false)?.into()
                }
                Event::Start(Tag::Image { dest_url, .. }) => {
                    *dest_url = resolver.rewrite(page, dest_url, true)?.into()
                }
                _ => {}
            }
        }
        let mut body = String::new();
        pulldown_cmark::html::push_html(&mut body, markdown::highlight(events)?.into_iter());
        if page.source.as_str().is_empty() {
            body = page::contents(page, &pages, &navigation);
        }
        let html = page::render(&presentation, page, index, &pages, &navigation, &body);
        if files
            .insert(page.output.clone(), html.into_bytes())
            .is_some()
        {
            return Err(files::unsafe_path(
                &page.output,
                "documents have the same HTML output path",
            ));
        }
    }
    for (path, bytes) in resolver.assets {
        if files.insert(path.clone(), bytes).is_some() {
            return Err(files::unsafe_path(
                &path,
                "asset collides with a generated page",
            ));
        }
    }
    for (path, bytes) in FONTS {
        files.insert(
            Utf8PathBuf::from(format!("_zellij/fonts/{path}")),
            bytes.to_vec(),
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
    let data = serde_json::to_string(&search)
        .map_err(std::io::Error::other)?
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
        portable_path(path)?;
        for component in path.ancestors().filter(|p| !p.as_str().is_empty()) {
            if let Some(previous) = names.insert(component.as_str().to_lowercase(), component) {
                if previous != component {
                    return Err(files::unsafe_path(path, "case-only output path collision"));
                }
            }
        }
    }
    let stage = StaticOutput::prepare(&root, output)?;
    let writes: Vec<_> = files
        .iter()
        .map(|(path, body)| (path.as_path(), body.as_slice()))
        .collect();
    files::write_files(&stage.directory, &writes, false)?;
    Ok(PreparedBuild {
        stage,
        report: BuildReport {
            pages: pages.len(),
            warnings,
        },
    })
}
