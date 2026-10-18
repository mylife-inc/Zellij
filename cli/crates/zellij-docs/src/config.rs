//! Optional portable-project settings. Paths name source files, not URLs.
use crate::{Page, links::href, markdown::escape};
use camino::{Utf8Path, Utf8PathBuf};
use serde::Deserialize;
use std::{collections::BTreeSet, fs, io};
use zellij_core::{Result, files};

pub const FILE: &str = "zel.yaml";

#[derive(Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    #[serde(default, deserialize_with = "optional_text")]
    pub name: Option<String>,
    #[serde(default, deserialize_with = "optional_text")]
    pub theme: Option<String>,
    #[serde(default)]
    pub markdown: Markdown,
    pub brand: Option<Brand>,
    pub nav: Option<Vec<Entry>>,
}
#[derive(Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Markdown {
    #[serde(default)]
    pub math: bool,
    #[serde(default)]
    pub mermaid: bool,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Brand {
    pub logo: Logo,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Logo {
    #[serde(deserialize_with = "text")]
    pub src: String,
    #[serde(deserialize_with = "text")]
    pub alt: String,
    #[serde(default, deserialize_with = "optional_text")]
    pub href: Option<String>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Entry {
    #[serde(default, deserialize_with = "optional_text")]
    label: Option<String>,
    #[serde(default, deserialize_with = "optional_text")]
    page: Option<String>,
    #[serde(default, deserialize_with = "optional_text")]
    href: Option<String>,
    children: Option<Vec<Entry>>,
}

// serde_yaml permits numeric/bool-to-String coercion. Configuration text must
// actually be YAML strings, so accidental types cannot become public labels.
fn text<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> std::result::Result<String, D::Error> {
    match serde_yaml_ng::Value::deserialize(deserializer)? {
        serde_yaml_ng::Value::String(value) => Ok(value),
        _ => Err(serde::de::Error::custom("expected a string")),
    }
}
fn optional_text<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> std::result::Result<Option<String>, D::Error> {
    match Option::<serde_yaml_ng::Value>::deserialize(deserializer)? {
        None => Ok(None),
        Some(serde_yaml_ng::Value::String(value)) => Ok(Some(value)),
        _ => Err(serde::de::Error::custom("expected a string")),
    }
}

fn invalid(message: impl Into<String>) -> zellij_core::Error {
    files::unsafe_path(Utf8Path::new(FILE), message)
}
fn nonempty(value: &str, field: &str) -> Result<()> {
    if value.trim().is_empty() || value.chars().any(char::is_control) {
        return Err(invalid(format!(
            "{field} must be nonempty text without control characters"
        )));
    }
    Ok(())
}
fn external(url: &str, field: &str) -> Result<()> {
    let uri = url
        .split('#')
        .next()
        .unwrap_or_default()
        .parse::<hyper::Uri>();
    if url
        .chars()
        .any(|c| c.is_whitespace() || c.is_control() || c == '\\')
        || !uri.is_ok_and(|u| {
            matches!(u.scheme_str(), Some("https" | "http"))
                && u.authority().is_some_and(|a| !a.host().is_empty())
        })
    {
        return Err(invalid(format!(
            "{field} must be an absolute http:// or https:// URL"
        )));
    }
    Ok(())
}

pub fn present(root: &Utf8Path) -> bool {
    // A malformed entry (including a dangling link) must select the portable
    // path and report its error, rather than silently choosing another target.
    fs::symlink_metadata(root.join(FILE)).is_ok()
}
fn regular(root: &Utf8Path, relative: &Utf8Path) -> Result<Utf8PathBuf> {
    crate::portable_path(relative)?;
    let path = files::contained_path(root, relative)?;
    let mut component = root.to_owned();
    for part in relative.as_str().split('/') {
        component.push(part);
        let metadata =
            fs::symlink_metadata(&component).map_err(|e| invalid(format!("{relative}: {e}")))?;
        if crate::watch::is_link(&metadata) {
            return Err(invalid(format!(
                "{relative}: symbolic links and junctions are not allowed"
            )));
        }
    }
    if !path.is_file() {
        return Err(invalid(format!("{relative}: expected a regular file")));
    }
    Ok(path)
}
impl Config {
    pub fn load(root: &Utf8Path) -> Result<Self> {
        match fs::symlink_metadata(root.join(FILE)) {
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Self::default()),
            Err(error) => return Err(error.into()),
            Ok(_) => {}
        }
        let path = regular(root, Utf8Path::new(FILE))?;
        let config: Self = serde_yaml_ng::from_str(&fs::read_to_string(path)?)
            .map_err(|e| invalid(format!("invalid configuration: {e}")))?;
        if let Some(name) = &config.name {
            nonempty(name, "name")?;
        }
        if let Some(theme) = &config.theme {
            crate::validate_theme(theme).map_err(|e| invalid(e.to_string()))?;
        }
        if let Some(brand) = &config.brand {
            nonempty(&brand.logo.alt, "brand.logo.alt")?;
            if let Some(url) = &brand.logo.href {
                external(url, "brand.logo.href")?;
            }
        }
        Ok(config)
    }

    pub fn logo(&self, root: &Utf8Path) -> Result<Option<(Utf8PathBuf, Vec<u8>)>> {
        let Some(brand) = &self.brand else {
            return Ok(None);
        };
        let relative = Utf8Path::new(&brand.logo.src);
        let file = regular(root, relative)?;
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
                "brand.logo.src must name a local image outside hidden/generated paths",
            ));
        }
        Ok(Some((relative.to_owned(), fs::read(file)?)))
    }
}

pub enum Item {
    Page { index: usize, label: String },
    Link { url: String, label: String },
    Group { label: String, children: Vec<Item> },
}
pub struct Navigation {
    pub items: Vec<Item>,
    pub sequence: Vec<(usize, String)>,
    pub custom: bool,
}
impl Navigation {
    pub fn resolve(config: &Config, pages: &[Page]) -> Result<Self> {
        let mut navigation = Self {
            items: vec![],
            sequence: vec![],
            custom: config.nav.is_some(),
        };
        if let Some(entries) = &config.nav {
            let mut seen = BTreeSet::new();
            navigation.items =
                Self::entries(entries, pages, &mut seen, &mut navigation.sequence, 0)?;
        } else {
            for (index, page) in pages.iter().enumerate() {
                let label = page.markdown.title.clone();
                navigation.items.push(Item::Page {
                    index,
                    label: label.clone(),
                });
                navigation.sequence.push((index, label));
            }
        }
        Ok(navigation)
    }
    fn entries(
        entries: &[Entry],
        pages: &[Page],
        seen: &mut BTreeSet<usize>,
        sequence: &mut Vec<(usize, String)>,
        depth: usize,
    ) -> Result<Vec<Item>> {
        if depth > 16 {
            return Err(invalid("nav nesting exceeds 16 groups"));
        }
        entries
            .iter()
            .map(|entry| {
                if let Some(label) = &entry.label {
                    nonempty(label, "nav label")?;
                }
                if [
                    entry.page.is_some(),
                    entry.href.is_some(),
                    entry.children.is_some(),
                ]
                .into_iter()
                .filter(|v| *v)
                .count()
                    != 1
                {
                    return Err(invalid(
                        "each nav entry needs exactly one of page, href, or children",
                    ));
                }
                if let Some(path) = &entry.page {
                    crate::portable_path(Utf8Path::new(path))
                        .map_err(|e| invalid(format!("nav page {path:?}: {e}")))?;
                    let index = pages
                        .iter()
                        .position(|p| p.source == path.as_str())
                        .ok_or_else(|| {
                            invalid(format!(
                                "nav page {path:?} is not a discovered Markdown document"
                            ))
                        })?;
                    if !seen.insert(index) {
                        return Err(invalid(format!("duplicate nav page {path:?}")));
                    }
                    let label = entry
                        .label
                        .clone()
                        .unwrap_or_else(|| pages[index].markdown.title.clone());
                    sequence.push((index, label.clone()));
                    return Ok(Item::Page { index, label });
                }
                let label = entry
                    .label
                    .clone()
                    .ok_or_else(|| invalid("nav groups and external links need a label"))?;
                if let Some(url) = &entry.href {
                    external(url, "nav href")?;
                    Ok(Item::Link {
                        url: url.clone(),
                        label,
                    })
                } else {
                    let children = entry.children.as_deref().unwrap();
                    if children.is_empty() {
                        return Err(invalid("nav groups must not be empty"));
                    }
                    Ok(Item::Group {
                        label,
                        children: Self::entries(children, pages, seen, sequence, depth + 1)?,
                    })
                }
            })
            .collect()
    }
    pub fn render(&self, current: &Page, pages: &[Page]) -> String {
        self.render_items(&self.items, current, pages)
    }
    fn render_items(&self, items: &[Item], current: &Page, pages: &[Page]) -> String {
        items
            .iter()
            .map(|item| match item {
                Item::Page { index, label } => {
                    let page = &pages[*index];
                    let active = if page.output == current.output {
                        " aria-current=\"page\""
                    } else {
                        ""
                    };
                    let folder = if self.custom {
                        String::new()
                    } else {
                        page.output
                            .parent()
                            .filter(|p| !p.as_str().is_empty())
                            .map(|p| format!("<small>{}</small>", escape(p.as_str())))
                            .unwrap_or_default()
                    };
                    format!(
                        "<li><a href=\"{}\"{active}>{folder}{}</a></li>",
                        href(&current.output, &page.output),
                        escape(label)
                    )
                }
                Item::Link { url, label } => {
                    format!("<li><a href=\"{}\">{}</a></li>", escape(url), escape(label))
                }
                Item::Group { label, children } => format!(
                    "<li class=\"nav-group\"><span class=\"nav-label\">{}</span><ul>{}</ul></li>",
                    escape(label),
                    self.render_items(children, current, pages)
                ),
            })
            .collect()
    }
}
