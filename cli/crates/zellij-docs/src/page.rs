use crate::{
    Page,
    config::{Logo, Navigation},
    links::{encode, href, prefix},
    markdown::escape,
};
use pulldown_cmark::HeadingLevel;

pub struct Presentation<'a> {
    pub name: &'a str,
    pub theme: &'a str,
    pub logo: Option<&'a Logo>,
}

pub fn contents(current: &Page, pages: &[Page], navigation: &Navigation) -> String {
    if navigation.custom {
        return format!(
            "<p>Explore the documentation.</p><ul class=\"document-list\">{}</ul>",
            navigation.render(current, pages)
        );
    }
    let links: String = pages
        .iter()
        .filter(|p| !p.source.as_str().is_empty())
        .map(|p| {
            format!(
                "<li><a href=\"{}\">{}</a></li>",
                href(&current.output, &p.output),
                escape(&p.markdown.title)
            )
        })
        .collect();
    format!("<p>Explore the documentation.</p><ul class=\"document-list\">{links}</ul>")
}

pub fn render(
    presentation: &Presentation<'_>,
    current: &Page,
    index: usize,
    pages: &[Page],
    navigation: &Navigation,
    body: &str,
) -> String {
    let prefix = prefix(&current.output);
    let title = escape(&current.markdown.title);
    let name = escape(presentation.name);
    let theme = presentation.theme;
    let nav = navigation.render(current, pages);
    let logo = presentation
        .logo
        .map(|logo| {
            format!(
                "<a class=\"brand-logo\" href=\"{}\"><img src=\"{}\" alt=\"{}\"></a>",
                escape(
                    &logo
                        .href
                        .clone()
                        .unwrap_or_else(|| format!("{prefix}index.html"))
                ),
                href(&current.output, camino::Utf8Path::new(&logo.src)),
                escape(&logo.alt)
            )
        })
        .unwrap_or_default();
    let mut crumbs = format!("<a href=\"{prefix}index.html\">Home</a>");
    let parent = current.output.parent().unwrap();
    let mut path = camino::Utf8PathBuf::new();
    for part in parent.as_str().split('/').filter(|p| !p.is_empty()) {
        path.push(part);
        let folder = crate::url_path(&path.join("index.html"));
        if folder == current.output {
            continue;
        }
        if let Some(p) = pages.iter().find(|p| p.output == folder) {
            crumbs.push_str(&format!(
                "<span>/</span><a href=\"{}\">{}</a>",
                href(&current.output, &folder),
                escape(&p.markdown.title)
            ));
        } else {
            crumbs.push_str(&format!("<span>/</span><span>{}</span>", escape(part)));
        }
    }
    if current.output != "index.html" {
        crumbs.push_str(&format!(
            "<span>/</span><span aria-current=\"page\">{title}</span>"
        ));
    }
    let toc: String = current
        .markdown
        .headings
        .iter()
        .filter(|h| h.level != HeadingLevel::H1)
        .map(|h| {
            format!(
                "<li><a href=\"#{}\">{}</a></li>",
                encode(&h.id),
                escape(&h.title)
            )
        })
        .collect();
    let toc = if toc.is_empty() {
        String::new()
    } else {
        format!(
            "<details class=\"toc\"><summary>On this page</summary><nav aria-label=\"On this page\"><ul>{toc}</ul></nav></details>"
        )
    };
    let heading = if current.markdown.has_h1 {
        String::new()
    } else {
        format!("<h1 id=\"_zel-title\">{title}</h1>")
    };
    let position = navigation.sequence.iter().position(|(i, _)| *i == index);
    let paging: String = [
        ("Previous", position.and_then(|i| i.checked_sub(1))),
        ("Next", position.map(|i| i + 1)),
    ]
    .into_iter()
    .filter_map(|(label, i)| {
        i.and_then(|i| navigation.sequence.get(i))
            .map(|(i, label_text)| {
                let p = &pages[*i];
                format!(
                    "<a rel=\"{}\" href=\"{}\"><small>{label}</small><span>{}</span></a>",
                    if label == "Next" { "next" } else { "prev" },
                    href(&current.output, &p.output),
                    escape(label_text)
                )
            })
    })
    .collect();
    let themes: String = zellij_core::schema::themes()
        .iter()
        .map(|t| {
            format!(
                "<option value=\"{}\"{}>{}</option>",
                escape(&t.name),
                if t.name == theme { " selected" } else { "" },
                escape(&t.label)
            )
        })
        .collect();
    format!(
        r##"<!doctype html>
<html lang="en" class="zellij-fonts" data-zellij-theme="{theme}" data-zellij-mode="system">
<head><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1">
<title>{title} · {name}</title><meta name="description" content="{description}">
<link rel="stylesheet" href="{prefix}_zellij/fonts/fonts.css">
<link rel="stylesheet" href="{prefix}_zellij/themes.css"><link rel="stylesheet" href="{prefix}_zellij/style.css">
<script defer src="{prefix}_zellij/search.js"></script><script defer src="{prefix}_zellij/site.js"></script></head>
<body data-root="{prefix}"><a class="skip" href="#_zel-content">Skip to content</a>
<header class="header"><div class="brand-group">{logo}<a class="brand" href="{prefix}index.html">{name}</a></div>
<div class="controls" hidden><button id="_zel-menu" aria-expanded="false" aria-controls="_zel-sidebar">Contents</button>
<button id="_zel-search-open">Search <kbd>⌘/Ctrl K</kbd></button><label class="sr-only" for="_zel-theme">Theme</label><select id="_zel-theme">{themes}</select>
<label class="sr-only" for="_zel-mode">Appearance</label><select id="_zel-mode"><option value="system">System</option><option value="light">Light</option><option value="dark">Dark</option></select></div></header>
<div class="layout"><aside class="sidebar" id="_zel-sidebar"><nav aria-label="Documentation"><p class="eyebrow">Contents</p><ul>{nav}</ul></nav></aside>
<main id="_zel-content" tabindex="-1"><nav class="breadcrumbs" aria-label="Breadcrumb">{crumbs}</nav>{toc}<article>{heading}{body}</article><nav class="paging" aria-label="Previous and next">{paging}</nav>
<footer>Built with Zellij · <a href="{prefix}_zellij/fonts/licenses.txt">Font licenses</a></footer></main></div>
<dialog id="_zel-search" aria-labelledby="_zel-search-title"><form method="dialog"><button class="close" aria-label="Close search">Close</button></form><label id="_zel-search-title" for="_zel-query">Search documentation</label><input id="_zel-query" type="search" autocomplete="off" placeholder="Search pages and headings…"><p id="_zel-status" role="status" aria-live="polite">Type to search.</p><ul id="_zel-results"></ul></dialog>
</body></html>
"##,
        description = escape(&current.markdown.description)
    )
}
