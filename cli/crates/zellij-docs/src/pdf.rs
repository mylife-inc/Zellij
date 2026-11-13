//! A bounded Markdown-to-LaTeX adapter. Authored HTML/TeX is never a template.
//! The caller executes the optional toolchain; compilation itself stays native.
use crate::{BuildOptions, BuildReport, Diagnostics, Page, Publication, compile, links};
use camino::{Utf8Path, Utf8PathBuf};
use pulldown_cmark::{Event, HeadingLevel, Tag, TagEnd};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
};
use zellij_core::{Error, Result, files, output::StaticOutput, site::SiteInput};

pub struct Source {
    pub files: BTreeMap<Utf8PathBuf, Vec<u8>>,
    pub revision: String,
}
pub struct Prepared {
    stage: StaticOutput,
    pub source: Source,
    pub destination: Utf8PathBuf,
    report: BuildReport,
}
impl Prepared {
    /// Publish only after the caller has verified the PDF and written its receipt.
    pub fn publish(self) -> Result<BuildReport> {
        let path = self.destination.join("book.pdf");
        let metadata = fs::symlink_metadata(&path)?;
        if !metadata.is_file()
            || metadata.file_type().is_symlink()
            || !fs::read(&path)?.starts_with(b"%PDF-")
        {
            return Err(invalid("toolchain did not produce a regular PDF"));
        }
        self.stage.publish()?;
        Ok(self.report)
    }
}
/// Resolve the manuscript once, then stage the website and its exact print inputs.
/// Standalone export uses its own owned output directory and leaves the site alone.
pub fn prepare(
    source: &SiteInput,
    output: &Utf8Path,
    options: BuildOptions<'_>,
    with_site: bool,
) -> Result<Prepared> {
    let mut diagnostics = Diagnostics::new(source.root());
    let compiled = compile(source, options, &mut diagnostics, None, false, true)?;
    let stage = StaticOutput::prepare(&compiled.root, output)?;
    if with_site {
        files::write_files(
            &stage.directory,
            &compiled
                .files
                .iter()
                .map(|(p, b)| (p.as_path(), b.as_slice()))
                .collect::<Vec<_>>(),
            false,
        )?;
    }
    let destination = if with_site {
        stage.directory.join("_zellij/pdf")
    } else {
        stage.directory.clone()
    };
    fs::create_dir_all(&destination)?;
    Ok(Prepared {
        stage,
        source: compiled.pdf.expect("requested print source"),
        destination,
        report: BuildReport {
            pages: compiled.pages,
            warnings: diagnostics.warnings(),
        },
    })
}

/// Check print compatibility without a runtime or filesystem writes.
pub fn check(source: &SiteInput) -> Result<()> {
    let mut diagnostics = Diagnostics::new(source.root());
    compile(
        source,
        BuildOptions::default(),
        &mut diagnostics,
        None,
        false,
        true,
    )?;
    Ok(())
}
fn invalid(message: impl std::fmt::Display) -> Error {
    Error::InvalidArgument(format!("PDF: {message}"))
}
fn tex(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            '\\' => "\\textbackslash{}".into(),
            '{' => "\\{".into(),
            '}' => "\\}".into(),
            '$' => "\\$".into(),
            '&' => "\\&".into(),
            '#' => "\\#".into(),
            '%' => "\\%".into(),
            '_' => "\\_".into(),
            '~' => "\\textasciitilde{}".into(),
            '^' => "\\textasciicircum{}".into(),
            '\0' => String::new(),
            _ => c.to_string(),
        })
        .collect()
}
fn label(page: usize, id: &str) -> String {
    format!(
        "p{page}-{}",
        id.as_bytes()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>()
    )
}
struct Writer<'a> {
    pages: &'a [Page],
    assets: &'a BTreeMap<Utf8PathBuf, Vec<u8>>,
    files: BTreeMap<Utf8PathBuf, Vec<u8>>,
    images: BTreeMap<Utf8PathBuf, String>,
    notes: BTreeSet<(usize, String)>,
}
impl Writer<'_> {
    fn image(&mut self, relative: &Utf8Path) -> Result<String> {
        if let Some(name) = self.images.get(relative) {
            return Ok(name.clone());
        }
        let bytes = self
            .assets
            .get(relative)
            .ok_or_else(|| invalid(format!("missing image {relative}")))?;
        let ext = relative.extension().unwrap_or("").to_ascii_lowercase();
        // TeX cannot consume SVG/WebP/GIF directly. Refuse instead of losing artwork.
        if !matches!(ext.as_str(), "png" | "jpg" | "jpeg") {
            return Err(invalid(format!(
                "{relative}: print images must be local PNG or JPEG; replace/convert this image before requesting PDF"
            )));
        }
        let name = format!("assets/image-{}.{}", self.images.len(), ext);
        self.files.insert(name.clone().into(), bytes.clone());
        self.images.insert(relative.to_owned(), name.clone());
        Ok(name)
    }
    fn body(
        &mut self,
        page: usize,
        events: &[Event<'_>],
        footnotes: &BTreeMap<String, Vec<Event<'_>>>,
        in_note: bool,
    ) -> Result<String> {
        let mut out = String::new();
        let mut i = 0;
        let mut lists = vec![];
        let mut cell = 0;
        while i < events.len() {
            match &events[i] {
                Event::Start(Tag::FootnoteDefinition(_)) => {
                    while i < events.len()
                        && !matches!(events[i], Event::End(TagEnd::FootnoteDefinition))
                    {
                        i += 1;
                    }
                }
                Event::Start(Tag::Image { dest_url, .. }) => {
                    let parent = self.pages[page]
                        .output
                        .parent()
                        .unwrap_or(Utf8Path::new(""));
                    let relative = links::resolve(parent, dest_url).ok_or_else(|| invalid("print images must be local PNG/JPEG files without URL queries/fragments"))?;
                    let name = self.image(&relative)?;
                    out.push_str(&format!("\\includegraphics[width=\\linewidth,height=0.65\\textheight,keepaspectratio]{{{name}}}"));
                    // Alt text is retained as a print caption, not discarded.
                    let mut alt = String::new();
                    i += 1;
                    while i < events.len() && !matches!(events[i], Event::End(TagEnd::Image)) {
                        if let Event::Text(s) | Event::Code(s) = &events[i] {
                            alt.push_str(s);
                        }
                        i += 1;
                    }
                    if !alt.trim().is_empty() {
                        out.push_str(&format!("\\par{{\\small\\itshape {}}}\\par\n", tex(&alt)));
                    }
                }
                Event::Start(Tag::Link { dest_url, .. }) => {
                    if dest_url.starts_with("https:")
                        || dest_url.starts_with("http:")
                        || dest_url.starts_with("mailto:")
                        || dest_url.starts_with("tel:")
                    {
                        // Escape every TeX metacharacter; URLs are arguments, never commands.
                        out.push_str(&format!("\\href{{{}}}{{", tex(dest_url)));
                    } else {
                        if dest_url.contains('?') {
                            return Err(invalid("PDF links cannot retain URL query parameters"));
                        }
                        let (path, fragment) = dest_url.split_once('#').unwrap_or((dest_url, ""));
                        let target = if path.is_empty() {
                            page
                        } else {
                            let relative = links::resolve(
                                self.pages[page]
                                    .output
                                    .parent()
                                    .unwrap_or(Utf8Path::new("")),
                                path,
                            )
                            .ok_or_else(|| invalid("invalid print link"))?;
                            self.pages.iter().position(|p| p.output == relative).ok_or_else(|| invalid(format!("{dest_url}: PDF links must target a book page or an HTTP(S)/mailto/tel URL")))?
                        };
                        let fragment = links::decode(fragment)
                            .ok_or_else(|| invalid("invalid link fragment"))?;
                        out.push_str(&format!("\\hyperlink{{{}}}{{", label(target, &fragment)));
                    }
                }
                Event::End(TagEnd::Link) => out.push('}'),
                Event::Start(Tag::Heading { level, id, .. }) => {
                    let command = match level {
                        HeadingLevel::H1 | HeadingLevel::H2 => "section",
                        HeadingLevel::H3 => "subsection",
                        _ => "subsubsection",
                    };
                    out.push_str(&format!(
                        "\\hypertarget{{{}}}{{}}\\{command}*{{",
                        label(page, id.as_deref().unwrap_or(""))
                    ));
                }
                Event::End(TagEnd::Heading(_)) => out.push_str("}\n"),
                Event::Start(Tag::Paragraph) => {}
                Event::End(TagEnd::Paragraph) => out.push_str("\n\n"),
                Event::Start(Tag::Emphasis) => out.push_str("\\emph{"),
                Event::Start(Tag::Strong) => out.push_str("\\textbf{"),
                Event::Start(Tag::Strikethrough) => out.push_str("\\st{"),
                Event::End(TagEnd::Emphasis | TagEnd::Strong | TagEnd::Strikethrough) => {
                    out.push('}')
                }
                Event::Start(Tag::BlockQuote(kind)) => {
                    out.push_str("\\begin{quote}\n");
                    if let Some(kind) = kind {
                        out.push_str(&format!("\\textbf{{{kind:?}}}\\par\n"));
                    }
                }
                Event::End(TagEnd::BlockQuote(_)) => out.push_str("\\end{quote}\n"),
                Event::Start(Tag::List(start)) => {
                    if lists.len() >= 4 {
                        return Err(invalid("print lists support at most four nesting levels"));
                    }
                    let name = if start.is_some() {
                        "enumerate"
                    } else {
                        "itemize"
                    };
                    out.push_str(&format!("\\begin{{{name}}}\n"));
                    if let Some(n) = start
                        && *n > 1
                    {
                        let counter = ["enumi", "enumii", "enumiii", "enumiv"][lists.len()];
                        out.push_str(&format!("\\setcounter{{{counter}}}{{{}}}\n", n - 1));
                    }
                    lists.push(name);
                }
                Event::End(TagEnd::List(_)) => {
                    let name = lists.pop().ok_or_else(|| invalid("unbalanced list"))?;
                    out.push_str(&format!("\\end{{{name}}}\n"));
                }
                Event::Start(Tag::Item) => out.push_str("\\item "),
                Event::End(TagEnd::Item) => out.push('\n'),
                Event::Start(Tag::CodeBlock(_)) => {
                    out.push_str("\\begin{quote}\\small\\ttfamily\\raggedright\n");
                    i += 1;
                    while i < events.len() && !matches!(events[i], Event::End(TagEnd::CodeBlock)) {
                        if let Event::Text(s) = &events[i] {
                            for line in s.lines() {
                                out.push_str(
                                    &tex(line).replace(' ', "\\ ").replace('\t', "\\ \\ \\ \\ "),
                                );
                                out.push_str("\\strut\\par\n");
                            }
                        }
                        i += 1;
                    }
                    out.push_str("\\end{quote}\n");
                }
                Event::Code(s) => out.push_str(&format!("\\texttt{{{}}}", tex(s))),
                Event::Text(s) => out.push_str(&tex(s)),
                Event::InlineMath(s) => out.push_str(&format!("\\({s}\\)")),
                Event::DisplayMath(s) => out.push_str(&format!("\\[{s}\\]\n")),
                Event::SoftBreak => out.push(' '),
                Event::HardBreak => out.push_str("\\newline\n"),
                Event::Rule => out.push_str("\\par\\noindent\\rule{\\linewidth}{0.4pt}\\par\n"),
                Event::TaskListMarker(done) => out.push_str(if *done { "[x] " } else { "[ ] " }),
                Event::FootnoteReference(id) => {
                    if in_note {
                        return Err(invalid("nested print footnotes are unsupported"));
                    }
                    let note = footnotes
                        .get(id.as_ref())
                        .ok_or_else(|| invalid("undefined footnote"))?;
                    let anchor = format!("fn{}", label(page, id));
                    if self.notes.insert((page, id.to_string())) {
                        out.push_str(&format!(
                            "\\footnote{{\\label{{{anchor}}}{}}}",
                            self.body(page, note, footnotes, true)?
                        ));
                    } else {
                        out.push_str(&format!("\\textsuperscript{{\\ref{{{anchor}}}}}"));
                    }
                }
                Event::Start(Tag::Table(alignments)) => {
                    if alignments.is_empty() || alignments.len() > 6 {
                        return Err(invalid("print tables support one to six columns"));
                    }
                    let width = 0.86 / alignments.len() as f64;
                    out.push_str(&format!(
                        "\\begin{{longtable}}{{{}}}\\toprule\n",
                        format!("p{{{width:.4}\\linewidth}}").repeat(alignments.len())
                    ));
                }
                Event::Start(Tag::TableHead | Tag::TableRow) => cell = 0,
                Event::Start(Tag::TableCell) => {
                    if cell > 0 {
                        out.push_str(" & ");
                    }
                    cell += 1;
                }
                Event::End(TagEnd::TableCell) => {}
                Event::End(TagEnd::TableHead) => out.push_str(" \\\\\\midrule\\endhead\n"),
                Event::End(TagEnd::TableRow) => out.push_str(" \\\\\n"),
                Event::End(TagEnd::Table) => out.push_str("\\bottomrule\\end{longtable}\n"),
                other => {
                    return Err(invalid(format!(
                        "unsupported print Markdown construct: {other:?}"
                    )));
                }
            }
            i += 1;
        }
        Ok(out)
    }
}
pub(crate) fn compose(
    book: &Publication,
    pages: &[Page],
    rewritten: &[Vec<Event<'static>>],
    assets: &BTreeMap<Utf8PathBuf, Vec<u8>>,
) -> Result<Source> {
    let m = &book.manifest;
    if m.direction.as_str() != "ltr"
        || !m
            .language
            .split('-')
            .next()
            .unwrap_or("")
            .eq_ignore_ascii_case("en")
    {
        return Err(invalid(
            "this first print template supports English/LTR; multilingual/RTL print templates remain planned (web reading remains available)",
        ));
    }
    let mut w = Writer {
        pages,
        assets,
        files: BTreeMap::new(),
        images: BTreeMap::new(),
        notes: BTreeSet::new(),
    };
    let mut out = include_str!("../assets/book.tex").to_owned();
    out.push_str(&format!("\\hypersetup{{pdftitle={{{}}},pdfauthor={{{}}},pdfsubject={{{}}}}}\n\\begin{{document}}\n\\frontmatter\n\\begin{{titlingpage}}\\centering\n\\vspace*{{0.08\\textheight}}\n{{\\Huge\\bfseries\\color{{zelaccent}} {}\\par}}\n",tex(&m.title),tex(&m.authors.iter().map(|a|a.0.as_str()).collect::<Vec<_>>().join(", ")),book.revision,tex(&m.title)));
    if book.entries[0].source.is_none() {
        out.push_str(&format!(
            "\\hypertarget{{{}}}{{}}\n",
            label(book.entries[0].index, "")
        ));
    }
    if let Some(s) = &m.subtitle {
        out.push_str(&format!(
            "\\vspace{{1.2em}}{{\\Large {}\\par}}\n",
            tex(&s.0)
        ));
    }
    if let Some(c) = &m.cover {
        let name = w.image(Utf8Path::new(&c.image))?;
        out.push_str(&format!("\\vfill\\includegraphics[width=0.65\\linewidth,height=0.45\\textheight,keepaspectratio]{{{name}}}\\par\n"));
    }
    out.push_str(&format!(
        "\\vfill{{\\large {}\\par}}\\end{{titlingpage}}\n\\tableofcontents*\n",
        tex(&m
            .authors
            .iter()
            .map(|a| a.0.as_str())
            .collect::<Vec<_>>()
            .join(" · "))
    ));
    let mut main = false;
    let mut appendix = false;
    let mut part = None;
    for entry in &book.entries {
        let page = &pages[entry.index];
        if page.markdown.has_mermaid || page.markdown.has_mindmap {
            return Err(invalid(format!(
                "{}: diagram fences need a static PNG/JPEG print equivalent",
                page.source
            )));
        }
        if entry.kind == "chapter" && !main {
            out.push_str("\\mainmatter\n");
            main = true;
        }
        if entry.kind == "appendix" && !appendix {
            out.push_str("\\appendix\n");
            appendix = true;
        }
        if entry.part != part {
            if let Some(id) = &entry.part {
                let title = m
                    .chapters
                    .iter()
                    .find_map(|e| match e {
                        zellij_core::book::Entry::Part(p) if &p.id == id => Some(p.title.as_str()),
                        _ => None,
                    })
                    .expect("resolved part");
                out.push_str(&format!("\\part{{{}}}\n", tex(title)));
            }
            part = entry.part.clone();
        }
        if entry.source.is_none() {
            continue;
        }
        if entry.kind != "title-page" {
            out.push_str(&format!("\\chapter{{{}}}\n", tex(&page.markdown.title)));
        }
        out.push_str(&format!(
            "\\hypertarget{{{}}}{{}}\n",
            label(entry.index, "")
        ));
        let mut events = rewritten[entry.index].clone();
        // The first authored H1 supplies the chapter title. Preserve its link target.
        if entry.kind != "title-page"
            && let Some(start) = events.iter().position(|e| {
                matches!(
                    e,
                    Event::Start(Tag::Heading {
                        level: HeadingLevel::H1,
                        ..
                    })
                )
            })
        {
            if let Event::Start(Tag::Heading { id, .. }) = &events[start] {
                out.push_str(&format!(
                    "\\hypertarget{{{}}}{{}}\n",
                    label(entry.index, id.as_deref().unwrap_or(""))
                ));
            }
            let end = start
                + events[start..]
                    .iter()
                    .position(|e| matches!(e, Event::End(TagEnd::Heading(_))))
                    .unwrap();
            events.drain(start..=end);
        }
        let mut notes = BTreeMap::new();
        for (i, event) in events.iter().enumerate() {
            if let Event::Start(Tag::FootnoteDefinition(id)) = event {
                let end = i
                    + 1
                    + events[i + 1..]
                        .iter()
                        .position(|e| matches!(e, Event::End(TagEnd::FootnoteDefinition)))
                        .unwrap();
                notes.insert(id.to_string(), events[i + 1..end].to_vec());
            }
        }
        out.push_str(&w.body(entry.index, &events, &notes, false)?);
        out.push_str("\n\\clearpage\n");
    }
    out.push_str("\\end{document}\n");
    w.files.insert("book.tex".into(), out.into_bytes());
    w.files.insert(
        "publication.json".into(),
        serde_json::to_vec_pretty(book).map_err(|e| invalid(e.to_string()))?,
    );
    Ok(Source {
        files: w.files,
        revision: book.revision.clone(),
    })
}
