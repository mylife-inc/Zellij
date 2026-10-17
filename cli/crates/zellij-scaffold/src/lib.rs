//! What `zel new` writes.
//!
//! A scaffold is built in memory as a list of (relative path, contents) and
//! written in one pass. Building it first means the not-empty check, the
//! dry run and the summary all read the same thing the writer will write —
//! a scaffold that reported files it did not create would be worse than none.
//!
//! Nothing here needs a container. Content is YAML and MDX; generating it is
//! string work, and a writer should not need Docker to start a folder.

use camino::{Utf8Path, Utf8PathBuf};
use zellij_core::{Error, Result};

mod content;
pub mod pipeline;
pub mod project;
mod section;

pub use section::section_file;

/// A set of files to create, relative to some root.
#[derive(Debug, Default)]
pub struct Scaffold {
    files: Vec<(Utf8PathBuf, String)>,
}

impl Scaffold {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, path: impl Into<Utf8PathBuf>, body: impl Into<String>) -> &mut Self {
        self.files.push((path.into(), body.into()));
        self
    }

    pub fn files(&self) -> impl Iterator<Item = (&Utf8Path, &str)> {
        self.files
            .iter()
            .map(|(path, body)| (path.as_path(), body.as_str()))
    }

    pub fn is_empty(&self) -> bool {
        self.files.is_empty()
    }

    /// Write everything under `root`.
    ///
    /// Refuses a non-empty directory unless `force`, because merging into a
    /// folder somebody already wrote in is how a scaffold quietly overwrites
    /// their work. A directory holding only dotfiles — a fresh `git init` — is
    /// treated as empty, since refusing that helps nobody.
    pub fn write(&self, root: &Utf8Path, force: bool) -> Result<Vec<Utf8PathBuf>> {
        if !force && has_visible_entries(root)? {
            return Err(Error::NotEmpty {
                path: root.to_owned(),
            });
        }

        self.write_with_policy(root, force)
    }

    /// Add files to an existing content folder without replacing any file.
    pub fn write_new(&self, root: &Utf8Path) -> Result<Vec<Utf8PathBuf>> {
        self.write_with_policy(root, false)
    }

    fn write_with_policy(&self, root: &Utf8Path, overwrite: bool) -> Result<Vec<Utf8PathBuf>> {
        zellij_core::files::write_files(root, &self.files().collect::<Vec<_>>(), overwrite)?;
        Ok(self.files.iter().map(|(path, _)| path.clone()).collect())
    }
}

fn has_visible_entries(dir: &Utf8Path) -> Result<bool> {
    let entries = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(err) => return Err(err.into()),
    };

    for entry in entries {
        let name = entry?.file_name();
        if !name.to_string_lossy().starts_with('.') {
            return Ok(true);
        }
    }

    Ok(false)
}

/// Options for `zel new docs`.
#[derive(Debug, Clone)]
pub struct DocsOptions {
    pub name: String,
    pub theme: String,
    /// Also write a guide folder. Off produces a landing page and nothing else.
    pub guide: bool,
    /// `site.yaml` and one page, for someone who wants no example at all.
    pub minimal: bool,
}

impl Default for DocsOptions {
    fn default() -> Self {
        Self {
            name: "Docs".into(),
            theme: "paper".into(),
            guide: false,
            minimal: false,
        }
    }
}

/// A content folder that passes `zel check` the moment it is written.
///
/// Not a skeleton with `TODO` in it: every field written is a field the schema
/// accepts, so each file is a working example of its own format. Somebody's
/// first minute with Zellij should not be a validation error they did not
/// cause.
pub fn docs(options: &DocsOptions) -> Scaffold {
    let mut scaffold = Scaffold::new();
    content::docs_into(&mut scaffold, options);
    scaffold
}

/// A guide folder: a sidebar and an index page.
///
/// `order` decides the numeric prefix, and so where the guide sits among its
/// siblings. Callers pass the next unused number rather than a name, because
/// two guides sharing a prefix is a build error and the caller is the one that
/// can see the directory.
pub fn guide(name: &str, title: &str, order: usize) -> Scaffold {
    let mut scaffold = Scaffold::new();
    content::guide_into(&mut scaffold, name, title, order);
    scaffold
}

/// A page, and one file per section it names.
///
/// Returns the YAML to append to `pages.yaml` alongside the section files,
/// rather than editing that file here — rewriting somebody's `pages.yaml`
/// would lose their comments and their ordering.
pub fn page(name: &str, section_types: &[String]) -> (Scaffold, String) {
    content::page_into(name, section_types)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn yaml(scaffold: &Scaffold, filename: &str) -> serde_yaml_ng::Value {
        let (_, body) = scaffold.files().find(|(p, _)| p == filename).unwrap();
        serde_yaml_ng::from_str(body).unwrap()
    }

    #[test]
    fn display_names_and_titles_round_trip_without_changing_yaml_structure() {
        for name in [
            "Acme: Docs",
            "A # comment",
            "\"quoted\" \\ path",
            "TRUE",
            "Null",
            "42",
            "العربية Café",
            "line\nnext\r\ntab\tend",
            "a\u{85}b\u{2028}c\u{2029}d",
        ] {
            let scaffold = docs(&DocsOptions {
                name: name.into(),
                ..Default::default()
            });
            let site = yaml(&scaffold, "site.yaml");
            assert_eq!(site["name"].as_str(), Some(name));
            assert_eq!(
                site["footer"]["legal"].as_str(),
                Some(format!("© {name}").as_str())
            );
            assert_eq!(
                yaml(&scaffold, "pages.yaml")[0]["title"].as_str(),
                Some(name)
            );
            assert_eq!(
                yaml(&scaffold, "sections/home/hero.yaml")["headline"].as_str(),
                Some(name)
            );

            let guide = guide("manual", name, 1);
            assert_eq!(
                yaml(&guide, "01-manual/_sidebar.yaml")["title"].as_str(),
                Some(name)
            );
            let (_, mdx) = guide
                .files()
                .find(|(p, _)| p == "01-manual/index.mdx")
                .unwrap();
            let frontmatter: serde_yaml_ng::Value =
                serde_yaml_ng::from_str(mdx.split("---\n").nth(1).unwrap()).unwrap();
            assert_eq!(frontmatter["title"].as_str(), Some(name));
        }
    }

    #[test]
    fn guide_cta_only_exists_when_its_target_is_generated() {
        for minimal in [false, true] {
            for guide in [false, true] {
                let scaffold = docs(&DocsOptions {
                    minimal,
                    guide,
                    ..Default::default()
                });
                let hero = yaml(&scaffold, "sections/home/hero.yaml");
                assert_eq!(hero.get("cta").is_some(), guide);
                if guide {
                    assert_eq!(hero["cta"][0]["href"].as_str(), Some("/guide"));
                }
            }
        }
    }

    #[test]
    fn page_identifiers_that_look_like_yaml_values_remain_strings() {
        for name in ["true", "null", "42"] {
            let (_, entry) = page(name, &[]);
            let pages: serde_yaml_ng::Value = serde_yaml_ng::from_str(&entry).unwrap();
            assert_eq!(pages[0]["name"].as_str(), Some(name));
            assert_eq!(
                pages[0]["sections"][0].as_str(),
                Some(format!("{name}/hero").as_str())
            );
        }
    }

    fn temp() -> (tempfile::TempDir, Utf8PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        let path = Utf8PathBuf::from_path_buf(dir.path().to_path_buf()).unwrap();
        (dir, path)
    }

    #[test]
    fn docs_writes_a_complete_folder() {
        let (_guard, root) = temp();
        let target = root.join("my-docs");

        let options = DocsOptions {
            name: "Acme".into(),
            guide: true,
            ..Default::default()
        };
        let written = docs(&options).write(&target, false).unwrap();

        for expected in [
            "site.yaml",
            "pages.yaml",
            "menu.yaml",
            "sections/home/hero.yaml",
            "01-guide/_sidebar.yaml",
            "01-guide/index.mdx",
        ] {
            assert!(
                written.iter().any(|p| p == expected),
                "{expected} was not written; got {written:?}"
            );
            assert!(target.join(expected).exists(), "{expected} is not on disk");
        }
    }

    // The whole promise of `zel new docs` is a folder that validates. YAML that
    // does not parse cannot, and this is the cheapest half of that check —
    // the other half is a CI step that runs the real validator over it.
    #[test]
    fn every_generated_yaml_parses() {
        let (_guard, root) = temp();
        let target = root.join("docs");

        let options = DocsOptions {
            name: "Acme".into(),
            guide: true,
            ..Default::default()
        };
        let scaffold = docs(&options);
        scaffold.write(&target, false).unwrap();

        for (path, body) in scaffold.files() {
            if path.extension() != Some("yaml") {
                continue;
            }

            serde_yaml_ng::from_str::<serde_yaml_ng::Value>(body)
                .unwrap_or_else(|err| panic!("{path} is not valid YAML: {err}"));
        }
    }

    #[test]
    fn the_site_name_reaches_the_files() {
        let options = DocsOptions {
            name: "Contoso".into(),
            theme: "cupertino".into(),
            ..Default::default()
        };
        let scaffold = docs(&options);

        let site = scaffold
            .files()
            .find(|(path, _)| path == "site.yaml")
            .map(|(_, body)| body)
            .unwrap();

        assert!(site.contains(r#"name: "Contoso""#), "{site}");
        assert!(site.contains(r#"theme: "cupertino""#), "{site}");
    }

    #[test]
    fn a_non_empty_directory_is_refused() {
        let (_guard, root) = temp();
        let target = root.join("taken");
        std::fs::create_dir_all(&target).unwrap();
        std::fs::write(target.join("notes.md"), "mine").unwrap();

        let err = docs(&DocsOptions::default())
            .write(&target, false)
            .unwrap_err();
        assert!(matches!(err, Error::NotEmpty { .. }));

        // And the refusal left it alone.
        assert_eq!(
            std::fs::read_to_string(target.join("notes.md")).unwrap(),
            "mine"
        );

        docs(&DocsOptions::default()).write(&target, true).unwrap();
        assert!(target.join("site.yaml").exists());
    }

    // `git init` before `zel new docs` is a normal order to work in, and a
    // directory holding only .git is empty as far as anyone is concerned.
    #[test]
    fn a_directory_of_only_dotfiles_counts_as_empty() {
        let (_guard, root) = temp();
        let target = root.join("fresh");
        std::fs::create_dir_all(target.join(".git")).unwrap();

        docs(&DocsOptions::default()).write(&target, false).unwrap();
        assert!(target.join("site.yaml").exists());
    }

    #[test]
    fn guides_are_numbered_so_they_do_not_collide() {
        let first = guide("guide", "Guide", 1);
        let second = guide("api", "API", 2);

        assert!(first.files().any(|(p, _)| p == "01-guide/_sidebar.yaml"));
        assert!(second.files().any(|(p, _)| p == "02-api/_sidebar.yaml"));
    }

    #[test]
    fn a_page_names_the_sections_it_creates() {
        let (scaffold, entry) = page("pricing", &["hero".into(), "faq".into()]);

        assert!(
            scaffold
                .files()
                .any(|(p, _)| p == "sections/pricing/hero.yaml")
        );
        assert!(
            scaffold
                .files()
                .any(|(p, _)| p == "sections/pricing/faq.yaml")
        );
        assert!(entry.contains(r#"- name: "pricing""#), "{entry}");
        assert!(entry.contains("pricing/hero"), "{entry}");
        assert!(entry.contains("pricing/faq"), "{entry}");
    }
}
