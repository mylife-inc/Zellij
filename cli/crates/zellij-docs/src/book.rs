//! Resolve a publication once; navigation, inspection and export share its order.
use crate::{
    Page,
    config::{Item, Navigation},
    links,
    markdown::escape,
};
use camino::{Utf8Path, Utf8PathBuf};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
};
use zellij_core::{
    Error, Result,
    book::{Chapter, Entry, Manifest},
};

#[derive(Clone, Debug, Serialize)]
pub struct Publication {
    pub schema_version: u32,
    pub manifest: Manifest,
    /// Hash of canonical manifest, ordered original Markdown and referenced assets.
    /// Presentation overrides and output locations do not change content identity.
    pub revision: String,
    pub entries: Vec<ReadingEntry>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub equations: Vec<crate::equations::Equation>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub statements: Vec<crate::statements::Statement>,
}
#[derive(Clone, Debug, Serialize)]
pub struct ReadingEntry {
    pub id: String,
    pub kind: &'static str,
    pub number: Option<String>,
    pub part: Option<String>,
    pub title: String,
    pub source: Option<String>,
    /// Encoded path relative to the exported root; also works after relocation.
    pub href: String,
    #[serde(skip)]
    pub(crate) index: usize,
}
fn invalid(message: impl std::fmt::Display) -> Error {
    Error::InvalidArgument(format!("book: {message}"))
}
struct Builder<'a> {
    pages: &'a [Page],
    entries: Vec<ReadingEntry>,
    seen: BTreeSet<usize>,
}
impl Builder<'_> {
    fn chapter(
        &mut self,
        c: &Chapter,
        kind: &'static str,
        number: Option<String>,
        part: Option<&str>,
    ) -> Result<Item> {
        crate::portable_path(Utf8Path::new(&c.source))?;
        let index = self
            .pages
            .iter()
            .position(|p| p.source == c.source.as_str())
            .ok_or_else(|| {
                invalid(format!(
                    "{}: source {:?} is not a discovered Markdown document",
                    c.id, c.source
                ))
            })?;
        if !self.seen.insert(index) {
            return Err(invalid(format!(
                "{}: the title page is automatic and must not be listed as a chapter",
                c.id
            )));
        }
        let page = &self.pages[index];
        let title = c
            .title
            .as_ref()
            .map_or(&page.markdown.title, |s| &s.0)
            .clone();
        let label = number
            .as_ref()
            .map_or_else(|| title.clone(), |n| format!("{n}. {title}"));
        self.entries.push(ReadingEntry {
            id: c.id.clone(),
            kind,
            number,
            part: part.map(str::to_owned),
            title,
            source: Some(c.source.clone()),
            href: links::encode(page.output.as_str()),
            index,
        });
        Ok(Item::Page { index, label })
    }
}
fn appendix(mut n: usize) -> String {
    let mut value = String::new();
    while n > 0 {
        n -= 1;
        value.insert(0, (b'A' + (n % 26) as u8) as char);
        n /= 26;
    }
    value
}
pub(crate) fn resolve(manifest: &Manifest, pages: &[Page]) -> Result<(Navigation, Publication)> {
    let home = pages
        .iter()
        .position(|p| p.output == "index.html")
        .ok_or_else(|| invalid("missing title page"))?;
    let mut builder = Builder {
        pages,
        entries: vec![ReadingEntry {
            id: "title-page".into(),
            kind: "title-page",
            number: None,
            part: None,
            title: manifest.title.clone(),
            source: (!pages[home].source.as_str().is_empty())
                .then(|| pages[home].source.to_string()),
            href: "index.html".into(),
            index: home,
        }],
        seen: BTreeSet::from([home]),
    };
    let mut items = vec![Item::Page {
        index: home,
        label: manifest.title.clone(),
    }];
    for c in &manifest.front_matter {
        items.push(builder.chapter(c, "front-matter", None, None)?);
    }
    let mut chapter = 0;
    for entry in &manifest.chapters {
        match entry {
            Entry::Chapter(c) => {
                chapter += 1;
                items.push(builder.chapter(c, "chapter", Some(chapter.to_string()), None)?);
            }
            Entry::Part(p) => {
                let mut children = vec![];
                for c in &p.chapters {
                    chapter += 1;
                    children.push(builder.chapter(
                        c,
                        "chapter",
                        Some(chapter.to_string()),
                        Some(&p.id),
                    )?);
                }
                items.push(Item::Group {
                    label: p.title.clone(),
                    children,
                });
            }
        }
    }
    for (i, c) in manifest.appendices.iter().enumerate() {
        items.push(builder.chapter(c, "appendix", Some(appendix(i + 1)), None)?);
    }
    let omitted: Vec<_> = pages
        .iter()
        .enumerate()
        .filter(|(i, _)| !builder.seen.contains(i))
        .map(|(_, p)| p.source.as_str())
        .collect();
    if !omitted.is_empty() {
        return Err(invalid(format!(
            "unlisted Markdown documents: {}; list every document in the manifest or move drafts outside content/",
            omitted.join(", ")
        )));
    }
    let sequence = builder
        .entries
        .iter()
        .map(|e| {
            (
                e.index,
                e.number
                    .as_ref()
                    .map_or_else(|| e.title.clone(), |n| format!("{n}. {}", e.title)),
            )
        })
        .collect();
    Ok((
        Navigation {
            items,
            sequence,
            custom: true,
        },
        Publication {
            schema_version: 1,
            manifest: manifest.clone(),
            revision: String::new(),
            entries: builder.entries,
            equations: vec![],
            statements: vec![],
        },
    ))
}
pub(crate) fn cover(
    manifest: &Manifest,
    root: &Utf8Path,
) -> Result<Option<(Utf8PathBuf, Vec<u8>)>> {
    let Some(cover) = &manifest.cover else {
        return Ok(None);
    };
    let relative = Utf8Path::new(&cover.image);
    let path = crate::config::regular(root, relative)?;
    if relative.as_str().split('/').any(crate::ignored_name)
        || relative
            .components()
            .next()
            .is_some_and(|p| p.as_str().eq_ignore_ascii_case("_zellij"))
        || mime_guess::from_path(relative)
            .first_or_octet_stream()
            .type_()
            != "image"
    {
        return Err(invalid(
            "cover.image must name a local image outside hidden/generated paths",
        ));
    }
    Ok(Some((relative.to_owned(), fs::read(path)?)))
}
impl Publication {
    pub(crate) fn finish(
        &mut self,
        pages: &[Page],
        assets: &BTreeMap<Utf8PathBuf, Vec<u8>>,
    ) -> Result<()> {
        fn add(hash: &mut Sha256, bytes: &[u8]) {
            hash.update((bytes.len() as u64).to_be_bytes());
            hash.update(bytes);
        }
        let mut hash = Sha256::new();
        add(&mut hash, b"zellij-book-v1");
        add(
            &mut hash,
            &serde_json::to_vec(&self.manifest).map_err(|e| invalid(e.to_string()))?,
        );
        for entry in &self.entries {
            let page = &pages[entry.index];
            add(&mut hash, page.source.as_str().as_bytes());
            add(&mut hash, page.original.as_bytes());
        }
        for (path, bytes) in assets {
            add(&mut hash, path.as_str().as_bytes());
            add(&mut hash, bytes);
        }
        self.revision = format!("sha256:{:x}", hash.finalize());
        Ok(())
    }
    pub(crate) fn entry(&self, index: usize) -> &ReadingEntry {
        self.entries
            .iter()
            .find(|e| e.index == index)
            .expect("resolved publication contains every page")
    }
    pub(crate) fn introduction(&self) -> String {
        let m = &self.manifest;
        let subtitle = m
            .subtitle
            .as_ref()
            .map(|s| format!("<p class=\"book-subtitle\">{}</p>", escape(&s.0)))
            .unwrap_or_default();
        let authors = m
            .authors
            .iter()
            .map(|a| escape(&a.0))
            .collect::<Vec<_>>()
            .join(" · ");
        let cover = m
            .cover
            .as_ref()
            .map(|c| {
                format!(
                    "<img class=\"book-cover\" src=\"{}\" alt=\"{}\">",
                    links::encode(&c.image),
                    escape(&c.alt)
                )
            })
            .unwrap_or_default();
        format!("{subtitle}<p class=\"book-authors\">{authors}</p>{cover}")
    }
}
