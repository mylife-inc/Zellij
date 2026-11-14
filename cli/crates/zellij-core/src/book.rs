//! Publication metadata and reading order, independent of web/print rendering.
use crate::{Error, Result, files};
use camino::Utf8Path;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct Manifest {
    pub version: u32,
    #[serde(deserialize_with = "text")]
    pub title: String,
    pub subtitle: Option<Text>,
    pub authors: Vec<Text>,
    #[serde(default = "english", deserialize_with = "text")]
    pub language: String,
    #[serde(default)]
    pub direction: Direction,
    pub cover: Option<Cover>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bibliography: Option<Bibliography>,
    #[serde(default)]
    pub front_matter: Vec<Chapter>,
    pub chapters: Vec<Entry>,
    #[serde(default)]
    pub appendices: Vec<Chapter>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct Bibliography {
    #[serde(deserialize_with = "text")]
    pub source: String,
    pub title: Option<Text>,
    #[serde(default)]
    pub include_uncited: bool,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(transparent)]
pub struct Text(#[serde(deserialize_with = "text")] pub String);
#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Direction {
    #[default]
    Ltr,
    Rtl,
}
impl Direction {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ltr => "ltr",
            Self::Rtl => "rtl",
        }
    }
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Cover {
    #[serde(deserialize_with = "text")]
    pub image: String,
    #[serde(deserialize_with = "text")]
    pub alt: String,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Chapter {
    #[serde(deserialize_with = "text")]
    pub id: String,
    #[serde(deserialize_with = "text")]
    pub source: String,
    pub title: Option<Text>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Part {
    #[serde(deserialize_with = "text")]
    pub id: String,
    #[serde(deserialize_with = "text")]
    pub title: String,
    pub chapters: Vec<Chapter>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Entry {
    Chapter(Chapter),
    Part(Part),
}
fn english() -> String {
    "en".into()
}
fn text<'de, D: serde::Deserializer<'de>>(de: D) -> std::result::Result<String, D::Error> {
    match serde_yaml_ng::Value::deserialize(de)? {
        serde_yaml_ng::Value::String(v)
            if !v.trim().is_empty() && !v.chars().any(char::is_control) =>
        {
            Ok(v)
        }
        _ => Err(serde::de::Error::custom(
            "expected nonempty text without control characters",
        )),
    }
}
fn invalid(message: impl std::fmt::Display) -> Error {
    Error::InvalidArgument(format!("book: {message}"))
}
/// A conventional language tag: a 2–8 letter language plus 1–8 alphanumeric subtags.
/// This checks syntax, not membership in the IANA registry.
pub fn language_tag(value: &str) -> bool {
    let mut parts = value.split('-');
    let first = parts.next().unwrap_or_default();
    (2..=8).contains(&first.len())
        && first.bytes().all(|b| b.is_ascii_alphabetic())
        && parts.all(|p| (1..=8).contains(&p.len()) && p.bytes().all(|b| b.is_ascii_alphanumeric()))
        && value.len() <= 63
}
impl Manifest {
    pub fn validate(&self) -> Result<()> {
        if self.version != 1 {
            return Err(invalid("version must be the integer 1"));
        }
        if !(1..=16).contains(&self.authors.len()) {
            return Err(invalid("authors must contain 1–16 names"));
        }
        if !language_tag(&self.language) {
            return Err(invalid(
                "language must be a conventional language tag, such as en or fr-CA",
            ));
        }
        if self.chapters.is_empty() {
            return Err(invalid(
                "chapters must contain at least one chapter or part",
            ));
        }
        let mut ids = BTreeSet::from(["title-page".to_string()]);
        if let Some(bib) = &self.bibliography {
            files::relative_path(Utf8Path::new(&bib.source))?;
            if !bib.source.ends_with(".json") {
                return Err(invalid(
                    "bibliography.source must name a local CSL-JSON .json file within content/",
                ));
            }
            ids.insert("bibliography".into());
        }
        let mut sources = BTreeSet::new();
        fn id(ids: &mut BTreeSet<String>, value: &str) -> Result<()> {
            if value.len() > 64
                || !value.starts_with(|c: char| c.is_ascii_lowercase())
                || !value
                    .bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-' || b == b'_')
            {
                return Err(invalid(
                    "IDs must start with a lowercase ASCII letter and use at most 64 lowercase letters, digits, hyphens or underscores",
                ));
            }
            if !ids.insert(value.into()) {
                return Err(invalid(format!("duplicate or reserved ID {value:?}")));
            }
            Ok(())
        }
        fn chapter(
            ids: &mut BTreeSet<String>,
            sources: &mut BTreeSet<String>,
            c: &Chapter,
        ) -> Result<()> {
            id(ids, &c.id)?;
            let path = Utf8Path::new(&c.source);
            files::relative_path(path)?;
            if !path
                .extension()
                .is_some_and(|s| s.eq_ignore_ascii_case("md") || s.eq_ignore_ascii_case("markdown"))
            {
                return Err(invalid(format!(
                    "{}: chapter source must be Markdown",
                    c.id
                )));
            }
            if !sources.insert(c.source.to_lowercase()) {
                return Err(invalid(format!("duplicate source {:?}", c.source)));
            }
            if sources.len() > 1024 {
                return Err(invalid("at most 1024 reading entries are supported"));
            }
            Ok(())
        }
        for c in &self.front_matter {
            chapter(&mut ids, &mut sources, c)?;
        }
        for entry in &self.chapters {
            match entry {
                Entry::Chapter(c) => chapter(&mut ids, &mut sources, c)?,
                Entry::Part(p) => {
                    id(&mut ids, &p.id)?;
                    if p.chapters.is_empty() {
                        return Err(invalid(format!("part {:?} has no chapters", p.id)));
                    }
                    for c in &p.chapters {
                        chapter(&mut ids, &mut sources, c)?;
                    }
                }
            }
        }
        for c in &self.appendices {
            chapter(&mut ids, &mut sources, c)?;
        }
        if let Some(cover) = &self.cover {
            files::relative_path(Utf8Path::new(&cover.image))?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const VALID: &str =
        "version: 1\ntitle: A book\nauthors: [Author]\nchapters: [{id: start, source: start.md}]\n";
    fn valid(yaml: &str) -> bool {
        serde_yaml_ng::from_str::<Manifest>(yaml).is_ok_and(|m| m.validate().is_ok())
    }
    #[test]
    fn publication_fields_are_typed_and_versioned() {
        assert!(valid(VALID));
        for (a, b) in [
            ("version: 1", "version: 2"),
            ("title: A book", "title: 123"),
            ("authors: [Author]", "authors: [true]"),
            ("authors: [Author]", "authors: []"),
            ("id: start", "id: title-page"),
            ("id: start", "id: Start"),
            ("source: start.md", "source: ../start.md"),
            ("source: start.md", "source: start.mdx"),
        ] {
            assert!(!valid(&VALID.replace(a, b)), "{b}");
        }
        assert!(!valid(&format!("{VALID}pdf: true\n")));
        assert!(!valid(&format!("{VALID}language: en--US\n")));
        assert!(valid(&format!("{VALID}language: ar\ndirection: rtl\n")));
    }
    #[test]
    fn identifiers_and_sources_are_unique_across_parts_and_matter() {
        assert!(!valid(&format!(
            "{VALID}appendices: [{{id: start, source: other.md}}]\n"
        )));
        assert!(!valid(&format!(
            "{VALID}front-matter: [{{id: intro, source: START.md}}]\n"
        )));
        assert!(!valid(&VALID.replace(
            "[{id: start, source: start.md}]",
            "[{id: part, title: Part, chapters: []}]"
        )));
        assert!(!valid(&VALID.replace(
            "[{id: start, source: start.md}]",
            "[{id: part, title: Part, chapters: [{id: part, source: start.md}]}]"
        )));
    }
}
