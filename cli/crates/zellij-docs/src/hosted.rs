//! A data-only bridge into the Next guide renderer. No Markdown is interpreted
//! as MDX or JavaScript, and every discovered page survives custom navigation.
use crate::{
    Page,
    config::{Config, Item, Navigation},
    links,
};
use camino::{Utf8Path, Utf8PathBuf};
use pulldown_cmark::{Alignment, CodeBlockKind, Event, Tag, TagEnd};
use serde_json::{Value, json};
use std::collections::BTreeMap;
use zellij_core::{Error, Result};

fn element(tag: &str, properties: Value, children: Vec<Value>) -> Value {
    json!({"type":"element", "tagName":tag, "properties":properties, "children":children})
}
fn text(value: &str) -> Value {
    json!({"type":"text", "value":value})
}
fn route(page: &Page) -> String {
    let path = page.output.as_str();
    if path == "index.html" {
        return "/".into();
    }
    // Keep the HTML suffix: foo.md and foo/index.md are different documents.
    format!("/{path}")
}
fn target(page: &Page, url: &str, pages: &[Page]) -> Result<String> {
    if url.starts_with('#') || url.contains(':') || url.starts_with("//") {
        return Ok(url.into());
    }
    let end = url.find(['?', '#']).unwrap_or(url.len());
    let (path, suffix) = url.split_at(end);
    if path.is_empty() {
        return Ok(url.into());
    }
    let resolved = links::resolve(page.output.parent().unwrap_or(Utf8Path::new("")), path)
        .ok_or_else(|| Error::InvalidArgument(format!("invalid adapted reference {url:?}")))?;
    let destination = if let Some(p) = pages.iter().find(|p| p.output == resolved) {
        links::encode(&route(p))
    } else {
        format!(
            "/_zellij/assets/portable/{}",
            links::encode(resolved.as_str())
        )
    };
    Ok(format!("{destination}{suffix}"))
}
fn nav(items: &[Item]) -> Vec<Value> {
    items
        .iter()
        .map(|item| match item {
            Item::Page { index, label } => json!({"page":format!("p{index}"), "label":label}),
            Item::Link { url, label } => json!({"href":url, "label":label}),
            Item::Group { label, children } => json!({"label":label, "items":nav(children)}),
        })
        .collect()
}
fn contents(items: &[Item], pages: &[Page]) -> Value {
    element(
        "ul",
        json!({}),
        items
            .iter()
            .filter(|item| !matches!(item, Item::Page { index, .. } if pages[*index].source.as_str().is_empty()))
            .map(|item| {
                let children = match item {
                    Item::Page { index, label } => vec![element(
                        "a",
                        json!({"href": links::encode(&route(&pages[*index]))}),
                        vec![text(label)],
                    )],
                    Item::Link { url, label } => {
                        vec![element("a", json!({"href":url}), vec![text(label)])]
                    }
                    Item::Group { label, children } => vec![text(label), contents(children, pages)],
                };
                element("li", json!({}), children)
            })
            .collect(),
    )
}

/// Pulldown events already contain normalized headings, escaped raw HTML and
/// expanded reference links. Build HAST directly instead of reparsing Markdown.
fn tree(page: &Page, pages: &[Page], events: &[Event<'_>], mermaid: bool) -> Result<Value> {
    let mut stack = vec![json!({"type":"root", "children":[]})];
    let mut footnotes = BTreeMap::<String, usize>::new();
    let mut aligns = vec![];
    let mut head = false;
    let mut cell = 0;
    let mut events = events.iter();
    fn push(stack: &mut [Value], node: Value) {
        stack.last_mut().unwrap()["children"]
            .as_array_mut()
            .unwrap()
            .push(node);
    }
    fn close(stack: &mut Vec<Value>) {
        let node = stack.pop().unwrap();
        push(stack, node);
    }
    while let Some(event) = events.next() {
        match event {
            Event::Start(Tag::Image {
                dest_url, title, ..
            }) => {
                let mut alt = String::new();
                let mut depth = 1;
                for event in events.by_ref() {
                    match event {
                        Event::Start(Tag::Image { .. }) => depth += 1,
                        Event::End(TagEnd::Image) => {
                            depth -= 1;
                            if depth == 0 {
                                break;
                            }
                        }
                        Event::Text(t) | Event::Code(t) => alt.push_str(t),
                        Event::SoftBreak | Event::HardBreak => alt.push(' '),
                        _ => {}
                    }
                }
                push(
                    &mut stack,
                    element(
                        "img",
                        json!({"src":target(page, dest_url, pages)?, "title":title.as_ref(), "alt":alt}),
                        vec![],
                    ),
                );
            }
            Event::Start(Tag::CodeBlock(kind)) => {
                let lang = match kind {
                    CodeBlockKind::Fenced(info) => info.split_whitespace().next().unwrap_or("text"),
                    _ => "text",
                };
                let mut code = String::new();
                for event in events.by_ref() {
                    match event {
                        Event::End(TagEnd::CodeBlock) => break,
                        Event::Text(t) => code.push_str(t),
                        _ => {}
                    }
                }
                let syntax = if (lang == "mermaid" && !mermaid) || lang == "math" {
                    "text"
                } else {
                    lang
                };
                push(
                    &mut stack,
                    element(
                        "pre",
                        json!({"dataLang":lang}),
                        vec![element(
                            "code",
                            json!({"className":[format!("language-{syntax}")]}),
                            vec![text(&code)],
                        )],
                    ),
                );
            }
            Event::Start(tag) => {
                let (name, props) = match tag {
                    Tag::HtmlBlock => ("div".into(), json!({})),
                    Tag::Paragraph => ("p".into(), json!({})),
                    Tag::Heading { level, id, .. } => {
                        (level.to_string(), json!({"id":id.as_deref()}))
                    }
                    Tag::BlockQuote(kind) => ("blockquote".into(), kind.map_or_else(|| json!({}), |kind| json!({"className":[format!("markdown-alert-{kind:?}").to_lowercase()]}))),
                    Tag::List(Some(start)) => ("ol".into(), json!({"start":start})),
                    Tag::List(None) => ("ul".into(), json!({})),
                    Tag::Item => ("li".into(), json!({})),
                    Tag::FootnoteDefinition(id) => {
                        let next = footnotes.len() + 1;
                        let n = *footnotes.entry(id.to_string()).or_insert(next);
                        let mut node = element(
                            "div",
                            json!({"id":id.as_ref(), "className":["footnote-definition"]}),
                            vec![],
                        );
                        node["children"].as_array_mut().unwrap().push(element(
                            "sup",
                            json!({}),
                            vec![text(&n.to_string())],
                        ));
                        stack.push(node);
                        continue;
                    }
                    Tag::Table(a) => {
                        aligns = a.clone();
                        ("table".into(), json!({}))
                    }
                    Tag::TableHead => {
                        head = true;
                        cell = 0;
                        stack.push(element("thead", json!({}), vec![]));
                        ("tr".into(), json!({}))
                    }
                    Tag::TableRow => {
                        cell = 0;
                        ("tr".into(), json!({}))
                    }
                    Tag::TableCell => {
                        let align = match aligns.get(cell) {
                            Some(Alignment::Left) => "left",
                            Some(Alignment::Center) => "center",
                            Some(Alignment::Right) => "right",
                            _ => "",
                        };
                        cell += 1;
                        (
                            if head { "th" } else { "td" }.into(),
                            if align.is_empty() {
                                json!({})
                            } else {
                                json!({"align":align})
                            },
                        )
                    }
                    Tag::Emphasis => ("em".into(), json!({})),
                    Tag::Strong => ("strong".into(), json!({})),
                    Tag::Strikethrough => ("del".into(), json!({})),
                    Tag::Link {
                        dest_url, title, ..
                    } => (
                        "a".into(),
                        json!({"href":target(page, dest_url, pages)?, "title":title.as_ref()}),
                    ),
                    _ => {
                        return Err(Error::InvalidArgument(format!(
                            "unsupported Markdown node in Next adapter: {tag:?}"
                        )));
                    }
                };
                stack.push(element(&name, props, vec![]));
            }
            Event::End(tag) => {
                if *tag == TagEnd::TableHead {
                    close(&mut stack);
                    close(&mut stack);
                    head = false;
                    stack.push(element("tbody", json!({}), vec![]));
                } else if *tag == TagEnd::Table {
                    close(&mut stack);
                    close(&mut stack);
                } else {
                    close(&mut stack);
                }
            }
            Event::Text(t) => push(&mut stack, text(t)),
            Event::Code(t) => push(&mut stack, element("code", json!({}), vec![text(t)])),
            Event::InlineMath(t) | Event::DisplayMath(t) => {
                let class = if matches!(event, Event::DisplayMath(_)) {
                    "math-display"
                } else {
                    "math-inline"
                };
                push(
                    &mut stack,
                    element(
                        "code",
                        json!({"className":["language-math", class]}),
                        vec![text(t)],
                    ),
                );
            }
            Event::FootnoteReference(id) => {
                let next = footnotes.len() + 1;
                let n = *footnotes.entry(id.to_string()).or_insert(next);
                push(
                    &mut stack,
                    element(
                        "sup",
                        json!({"className":["footnote-reference"]}),
                        vec![element(
                            "a",
                            json!({"href":format!("#{}", links::encode(id))}),
                            vec![text(&n.to_string())],
                        )],
                    ),
                );
            }
            Event::SoftBreak => push(&mut stack, text("\n")),
            Event::HardBreak => push(&mut stack, element("br", json!({}), vec![])),
            Event::Rule => push(&mut stack, element("hr", json!({}), vec![])),
            Event::TaskListMarker(checked) => push(
                &mut stack,
                element(
                    "input",
                    json!({"type":"checkbox", "disabled":true, "checked":checked}),
                    vec![],
                ),
            ),
            _ => {
                return Err(Error::InvalidArgument(
                    "unexpected raw Markdown event in Next adapter".into(),
                ));
            }
        }
    }
    if stack.len() != 1 {
        return Err(Error::InvalidArgument("unbalanced Markdown tree".into()));
    }
    Ok(stack.pop().unwrap())
}

pub(super) fn emit(
    config: &Config,
    name: &str,
    theme: &str,
    pages: &[Page],
    navigation: &Navigation,
    rewritten: &[Vec<Event<'_>>],
    assets: &BTreeMap<Utf8PathBuf, Vec<u8>>,
) -> Result<BTreeMap<Utf8PathBuf, Vec<u8>>> {
    for page in pages {
        if page
            .output
            .components()
            .next()
            .is_some_and(|p| p.as_str().eq_ignore_ascii_case("_next"))
        {
            return Err(Error::InvalidArgument(format!(
                "{}: /_next is reserved by Next.js; rename that folder or use a static image",
                page.source
            )));
        }
    }
    let mut files = BTreeMap::new();
    fn json_file(
        files: &mut BTreeMap<Utf8PathBuf, Vec<u8>>,
        path: String,
        value: Value,
    ) -> Result<()> {
        files.insert(
            path.into(),
            serde_json::to_vec_pretty(&value).map_err(|e| Error::Io(std::io::Error::other(e)))?,
        );
        Ok(())
    }
    let mut site = json!({"name":name, "theme":theme, "mode":"system", "home":"manual", "nav":{"search":true,"themeToggle":true}});
    if let Some(brand) = &config.brand {
        // Hosted branding expects a filesystem reference. A fixed safe name
        // avoids interpreting # or % in an authored image filename as URL syntax.
        let source = Utf8Path::new(&brand.logo.src);
        let logo_path = format!(
            "assets/portable-brand.{}",
            source.extension().unwrap_or("img").to_ascii_lowercase()
        );
        let bytes = assets
            .get(source)
            .ok_or_else(|| Error::InvalidArgument("missing validated branding asset".into()))?;
        files.insert(logo_path.clone().into(), bytes.clone());
        let mut logo = json!({"src":logo_path, "alt":brand.logo.alt});
        if let Some(href) = &brand.logo.href {
            logo["href"] = json!(href);
        }
        site["brand"] = json!({"title":name, "logo":logo});
    }
    json_file(&mut files, "site.yaml".into(), site)?;
    json_file(
        &mut files,
        "manual/_sidebar.yaml".into(),
        json!({"title":name, "route":"/", "nav":nav(&navigation.items)}),
    )?;
    for (index, (page, events)) in pages.iter().zip(rewritten).enumerate() {
        let mut body = tree(page, pages, events, config.markdown.mermaid)?;
        if !page.markdown.has_h1 {
            body["children"].as_array_mut().unwrap().insert(
                0,
                element(
                    "h1",
                    json!({"id":"_zel-title"}),
                    vec![text(&page.markdown.title)],
                ),
            );
        }
        if page.source.as_str().is_empty() {
            body["children"].as_array_mut().unwrap().push(element(
                "p",
                json!({}),
                vec![text("Explore the documentation.")],
            ));
            body["children"]
                .as_array_mut()
                .unwrap()
                .push(contents(&navigation.items, pages));
        }
        json_file(
            &mut files,
            format!("manual/p{index}.zeldoc.json"),
            json!({
                "version":1, "title":page.markdown.title, "description":page.markdown.description,
                "route":route(page), "text":page.markdown.text,
                "headings":page.markdown.headings.iter().map(|h| json!({"id":h.id, "text":h.title, "depth":h.level as u8})).collect::<Vec<_>>(),
                "body":body,
            }),
        )?;
    }
    for (path, bytes) in assets {
        files.insert(format!("assets/portable/{path}").into(), bytes.clone());
    }
    Ok(files)
}
