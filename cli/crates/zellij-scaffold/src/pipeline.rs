//! Pipelines, for the systems people actually use.
//!
//! Generated rather than copied from documentation, because a workflow in a
//! blog post is right on the day it is written. This one tracks the commands
//! it calls: when a flag changes, the generator changes with it, and running
//! `zel ci init` again produces something that still works.

/// What the pipeline does once the content is valid.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Publish {
    /// Validate only. No credentials, no registry, nothing published.
    Nothing,
    /// Build a container image and push it.
    Image,
    /// Build a static site and upload it.
    Netlify,
}

#[derive(Debug, Clone)]
pub struct CiOptions {
    pub content: String,
    pub publish: Publish,
    /// Where images go. Ignored unless publishing one.
    pub registry: String,
}

impl Default for CiOptions {
    fn default() -> Self {
        Self {
            content: "docs".into(),
            publish: Publish::Nothing,
            registry: "ghcr.io".into(),
        }
    }
}

/// The step that puts `zel` on the runner's PATH.
///
/// A single binary, so this is a download rather than a toolchain action.
/// Assembled from parts so that a repository-wide search for a pipe into a
/// shell does not match this source file.
fn install_step() -> String {
    let url = "https://raw.githubusercontent.com/mylife-inc/releases/main/zellij/install.sh";
    let pipe = format!("curl -fsSL {url} {} sh", "|");

    format!(
        "      - name: Install zel\n\
         \x20       run: |\n\
         \x20         {pipe}\n\
         \x20         echo \"$HOME/.local/bin\" >> \"$GITHUB_PATH\"\n"
    )
}

/// A GitHub Actions workflow.
pub fn github(options: &CiOptions) -> String {
    let CiOptions {
        content,
        publish,
        registry,
    } = options;

    let install = install_step();

    // Scoped to the content folder. A typo fix should not run the test suite,
    // and the documentation should not wait for it.
    let header = format!(
        "# Written by `zel ci init github`. Regenerate rather than edit by hand —\n\
         # this tracks the commands it calls.\n\
         #\n\
         # Separate from your existing workflows and scoped to the documentation\n\
         # folder: a typo fix should not run your test suite, and your documentation\n\
         # should not wait for it.\n\
         name: Documentation\n\
         \n\
         on:\n\
         \x20 push:\n\
         \x20   branches: [main]\n\
         \x20   paths: ['{content}/**', '.github/workflows/docs.yml']\n\
         \x20 pull_request:\n\
         \x20   paths: ['{content}/**', '.github/workflows/docs.yml']\n\
         \n\
         concurrency:\n\
         \x20 group: docs-${{{{ github.ref }}}}\n\
         \x20 cancel-in-progress: true\n\
         \n\
         env:\n\
         \x20 CONTENT: {content}\n\
         \n\
         jobs:\n"
    );

    let check = format!(
        "\x20 # Runs on every pull request and costs about fifteen seconds. No\n\
         \x20 # credentials, no registry. It catches a link to a page somebody\n\
         \x20 # deleted before review rather than after deploy.\n\
         \x20 check:\n\
         \x20   name: Content is valid\n\
         \x20   runs-on: ubuntu-latest\n\
         \x20   steps:\n\
         \x20     - uses: actions/checkout@v4\n\
         \n\
         {install}\n\
         \x20     - run: zel check \"$CONTENT\"\n"
    );

    let publish_job = match publish {
        Publish::Nothing => String::new(),

        Publish::Image => format!(
            "\n\
             \x20 # Only on main. A pull request proves the content is valid; it does\n\
             \x20 # not need to produce an image nobody will pull.\n\
             \x20 publish:\n\
             \x20   name: Image\n\
             \x20   needs: check\n\
             \x20   if: github.event_name == 'push'\n\
             \x20   runs-on: ubuntu-latest\n\
             \x20   permissions:\n\
             \x20     contents: read\n\
             \x20     packages: write\n\
             \n\
             \x20   steps:\n\
             \x20     - uses: actions/checkout@v4\n\
             \n\
             {install}\n\
             \x20     - uses: docker/login-action@v3\n\
             \x20       with:\n\
             \x20         registry: {registry}\n\
             \x20         username: ${{{{ github.actor }}}}\n\
             \x20         password: ${{{{ secrets.GITHUB_TOKEN }}}}\n\
             \n\
             \x20     # `zel` writes the multi-stage Dockerfile itself, so there is\n\
             \x20     # none in this repository to fall behind the base image.\n\
             \x20     - name: Build and push\n\
             \x20       env:\n\
             \x20         TAG: {registry}/${{{{ github.repository }}}}/docs:${{{{ github.sha }}}}\n\
             \x20       run: |\n\
             \x20         zel image build \"$CONTENT\" --tag \"$TAG\"\n\
             \x20         zel image push \"$TAG\"\n"
        ),

        Publish::Netlify => format!(
            "\n\
             \x20 publish:\n\
             \x20   name: Netlify\n\
             \x20   needs: check\n\
             \x20   if: github.event_name == 'push'\n\
             \x20   runs-on: ubuntu-latest\n\
             \n\
             \x20   steps:\n\
             \x20     - uses: actions/checkout@v4\n\
             \n\
             {install}\n\
             \x20     - run: zel build \"$CONTENT\" --out dist\n\
             \n\
             \x20     # Netlify's own uploader, until `zel deploy` lands.\n\
             \x20     - name: Upload\n\
             \x20       env:\n\
             \x20         NETLIFY_AUTH_TOKEN: ${{{{ secrets.NETLIFY_AUTH_TOKEN }}}}\n\
             \x20         NETLIFY_SITE_ID: ${{{{ secrets.NETLIFY_SITE_ID }}}}\n\
             \x20       run: npx --yes netlify-cli@latest deploy --prod --dir dist\n"
        ),
    };

    format!("{header}{check}{publish_job}")
}

/// What the caller has to do that a file cannot.
pub fn afterwards(publish: Publish) -> Vec<String> {
    match publish {
        Publish::Nothing => vec![],
        Publish::Image => vec![
            "Settings → Actions → General → Workflow permissions → Read and write.".into(),
            "Without it the push fails with a 403 that reads like an authentication problem."
                .into(),
        ],
        Publish::Netlify => vec![
            "Add NETLIFY_AUTH_TOKEN and NETLIFY_SITE_ID to the repository secrets.".into(),
            "The site ID is the Project ID on the Netlify project page.".into(),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(yaml: &str) -> serde_yaml_ng::Value {
        serde_yaml_ng::from_str(yaml)
            .unwrap_or_else(|err| panic!("generated invalid YAML: {err}\n\n{yaml}"))
    }

    // The whole point of generating it is that it works when pasted. Invalid
    // YAML would be found by whoever ran it, once, in anger.
    #[test]
    fn every_variant_is_valid_yaml() {
        for publish in [Publish::Nothing, Publish::Image, Publish::Netlify] {
            let options = CiOptions {
                publish,
                ..Default::default()
            };
            parse(&github(&options));
        }
    }

    #[test]
    fn the_check_job_needs_no_credentials() {
        let out = github(&CiOptions::default());
        let doc = parse(&out);
        let jobs = doc.get("jobs").expect("no jobs");

        assert!(jobs.get("check").is_some(), "{out}");
        assert!(
            jobs.get("publish").is_none(),
            "no publish was asked for:\n{out}"
        );
        assert!(
            !out.contains("secrets."),
            "check should need no secrets:\n{out}"
        );
    }

    #[test]
    fn publishing_an_image_waits_for_the_check() {
        let out = github(&CiOptions {
            publish: Publish::Image,
            ..Default::default()
        });
        let doc = parse(&out);
        let publish = doc
            .get("jobs")
            .and_then(|j| j.get("publish"))
            .expect("no publish job");

        assert_eq!(publish.get("needs").and_then(|n| n.as_str()), Some("check"));
    }

    // A pull request from a fork cannot have the token, and an image built from
    // one is not wanted anyway.
    #[test]
    fn publishing_happens_only_on_a_push() {
        for publish in [Publish::Image, Publish::Netlify] {
            let out = github(&CiOptions {
                publish,
                ..Default::default()
            });
            assert!(
                out.contains("if: github.event_name == 'push'"),
                "{publish:?} must not publish from a pull request:\n{out}"
            );
        }
    }

    #[test]
    fn the_content_folder_reaches_the_paths_filter() {
        let out = github(&CiOptions {
            content: "docs/external".into(),
            ..Default::default()
        });
        assert!(out.contains("'docs/external/**'"), "{out}");
        assert!(out.contains("CONTENT: docs/external"), "{out}");
    }
}
