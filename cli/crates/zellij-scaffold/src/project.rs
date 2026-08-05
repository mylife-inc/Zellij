//! A repository with documentation and a product as siblings.
//!
//! The arrangement is the valuable part, and it is the part nothing else
//! provides: a folder each, a pipeline scoped to the documentation, and a
//! README saying why they sit together. Documentation in the same repository
//! as the code it describes stays in the same branch, the same review and the
//! same pull request as the change it documents, which is the only arrangement
//! under which it stays accurate.
//!
//! The product folder is left empty on purpose. `cargo new`, `go mod init` and
//! `npm init` exist, are better than anything here would be, and are what
//! somebody would run anyway — so this prints the one for their language
//! rather than producing a worse version of it.

use crate::{DocsOptions, Scaffold, docs, pipeline};

/// The command that starts a project in a given language.
///
/// A hint, not a scaffold. Nothing is run and nothing is written; the point is
/// to save looking it up while keeping Zellij out of the business of
/// generating other people's projects.
pub fn product_hint(lang: &str, name: &str) -> Option<String> {
    Some(match lang {
        "rust" => "cargo init".to_owned(),
        // The module path carries the project's name, so it has to be the
        // project's name — a hint that has to be edited before it is run is
        // barely a hint.
        "go" => format!("go mod init example.com/{name}"),
        "node" => "npm init -y".to_owned(),
        "python" => format!("printf '[project]\\nname = \"{name}\"\\n' > pyproject.toml"),
        "java" => "gradle init --type java-application".to_owned(),
        "kotlin" => "gradle init --type kotlin-application".to_owned(),
        _ => return None,
    })
}

pub fn known_languages() -> &'static [&'static str] {
    &["rust", "go", "node", "python", "java", "kotlin", "none"]
}

#[derive(Debug, Clone)]
pub struct ProjectOptions {
    pub name: String,
    pub docs_dir: String,
    pub product_dir: String,
    pub product_lang: String,
}

impl Default for ProjectOptions {
    fn default() -> Self {
        Self {
            name: "project".into(),
            docs_dir: "docs".into(),
            product_dir: "product".into(),
            product_lang: "none".into(),
        }
    }
}

pub fn project(options: &ProjectOptions) -> Scaffold {
    let ProjectOptions {
        name,
        docs_dir,
        product_dir,
        ..
    } = options;

    // The documentation half is the content folder, unchanged — one scaffold,
    // not two that can disagree.
    let mut scaffold = Scaffold::new();
    for (path, body) in docs(&DocsOptions {
        name: name.clone(),
        guide: true,
        ..Default::default()
    })
    .files()
    {
        scaffold.add(format!("{docs_dir}/{path}"), body);
    }

    // Empty, and said to be empty. A `.gitkeep` alone looks like an oversight.
    scaffold.add(
        format!("{product_dir}/README.md"),
        format!(
            "# {name}\n\
             \n\
             Your code goes here. Zellij did not put anything in this folder on\n\
             purpose — your own tooling makes a better starting point than a\n\
             documentation tool guessing at one.\n\
             \n\
             Delete this file once there is something to replace it.\n"
        ),
    );

    scaffold.add(
        ".github/workflows/docs.yml",
        pipeline::github(&pipeline::CiOptions {
            content: docs_dir.clone(),
            publish: pipeline::Publish::Image,
            registry: "ghcr.io".into(),
        }),
    );

    scaffold.add(
        "README.md",
        format!(
            "# {name}\n\
             \n\
             ```\n\
             {docs_dir}/       the public documentation, as Zellij content\n\
             {product_dir}/    the thing it documents\n\
             ```\n\
             \n\
             The two sit together so a change and the documentation of that change\n\
             arrive in the same pull request. That is the only arrangement under\n\
             which documentation stays accurate.\n\
             \n\
             ## Working on the documentation\n\
             \n\
             ```\n\
             zel dev {docs_dir}      serve it, and edit\n\
             zel check {docs_dir}    validate it\n\
             ```\n\
             \n\
             `.github/workflows/docs.yml` validates it on every pull request and\n\
             builds a container image on `main`. A push that only touches\n\
             `{product_dir}/` does neither.\n"
        ),
    );

    scaffold.add(
        ".gitignore",
        "node_modules/\n\
         dist/\n\
         .next/\n\
         out/\n\
         .DS_Store\n",
    );

    scaffold
}

#[cfg(test)]
mod tests {
    use super::*;

    fn paths(scaffold: &Scaffold) -> Vec<String> {
        scaffold.files().map(|(p, _)| p.to_string()).collect()
    }

    #[test]
    fn the_two_folders_are_siblings() {
        let out = project(&ProjectOptions {
            name: "acme".into(),
            ..Default::default()
        });
        let paths = paths(&out);

        assert!(paths.iter().any(|p| p == "docs/site.yaml"), "{paths:?}");
        assert!(paths.iter().any(|p| p == "product/README.md"), "{paths:?}");
        assert!(
            paths.iter().any(|p| p == ".github/workflows/docs.yml"),
            "{paths:?}"
        );
    }

    #[test]
    fn the_folder_names_are_honoured_everywhere() {
        let out = project(&ProjectOptions {
            name: "acme".into(),
            docs_dir: "documentation".into(),
            product_dir: "app".into(),
            ..Default::default()
        });

        let paths = paths(&out);
        assert!(
            paths.iter().any(|p| p == "documentation/site.yaml"),
            "{paths:?}"
        );
        assert!(paths.iter().any(|p| p == "app/README.md"), "{paths:?}");

        // And the pipeline has to agree, or it watches a folder nobody uses.
        let workflow = out
            .files()
            .find(|(p, _)| p == ".github/workflows/docs.yml")
            .map(|(_, b)| b.to_string())
            .unwrap();
        assert!(workflow.contains("'documentation/**'"), "{workflow}");
        assert!(workflow.contains("CONTENT: documentation"), "{workflow}");
    }

    // The content half must be the same scaffold `zel new docs` produces, or
    // the two drift and only one of them stays valid.
    #[test]
    fn the_documentation_half_is_the_docs_scaffold() {
        let standalone = docs(&DocsOptions {
            name: "acme".into(),
            guide: true,
            ..Default::default()
        });
        let inside = project(&ProjectOptions {
            name: "acme".into(),
            ..Default::default()
        });

        for (path, body) in standalone.files() {
            let found = inside
                .files()
                .find(|(p, _)| p.as_str() == format!("docs/{path}"))
                .map(|(_, b)| b);
            assert_eq!(found, Some(body), "docs/{path} differs from `zel new docs`");
        }
    }

    #[test]
    fn a_hint_is_offered_for_known_languages_and_not_invented_for_others() {
        assert_eq!(product_hint("rust", "acme").as_deref(), Some("cargo init"));
        assert_eq!(product_hint("none", "acme"), None);
        assert_eq!(product_hint("brainfuck", "acme"), None);
    }

    // A hint that has to be edited before it runs is barely a hint.
    #[test]
    fn a_hint_carrying_a_name_carries_the_right_one() {
        assert_eq!(
            product_hint("go", "contoso").as_deref(),
            Some("go mod init example.com/contoso")
        );
        assert!(
            product_hint("python", "contoso")
                .unwrap()
                .contains("contoso")
        );
    }
}
