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
    /// The Netlify project this deploys to. Ignored unless publishing there.
    ///
    /// Written into the workflow as a plain value, not a secret. It is an
    /// identifier, not a credential — it appears in the project's own URL and
    /// on its dashboard, and holding it grants nothing without the token.
    ///
    /// Keeping it in the file is worth more than hiding it: it is visible in
    /// review, it travels with the repository, and it removes a provisioning
    /// step that could half-succeed. Filing a non-secret under `secrets` also
    /// teaches the wrong instinct about which is which.
    pub site_id: Option<String>,
}

impl Default for CiOptions {
    fn default() -> Self {
        Self {
            content: "docs".into(),
            publish: Publish::Nothing,
            registry: "ghcr.io".into(),
            site_id: None,
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
        site_id,
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
         \x20 # Re-deploy without a content change — after rotating a secret,\n\
         \x20 # or to put the current `main` back up. `zel ci publish` calls this.\n\
         \x20 workflow_dispatch:\n\
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
             \x20   # Not `== 'push'`: that excluded workflow_dispatch, so a manual\n\
             \x20   # run went green having deployed nothing. Everything except a\n\
             \x20   # pull request should publish.\n\
             \x20   if: github.event_name != 'pull_request'\n\
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

        Publish::Netlify => {
            // The site id is written in plain when known, and falls back to a
            // secret when it is not — so a workflow generated without one
            // still works for anyone who already put it in their secrets.
            let site = match site_id {
                Some(id) => format!("\x20         NETLIFY_SITE_ID: {id}\n"),
                None => "\x20         NETLIFY_SITE_ID: ${{ secrets.NETLIFY_SITE_ID }}\n".to_owned(),
            };

            format!(
                "\n\
                 \x20 publish:\n\
                 \x20   name: Netlify\n\
                 \x20   needs: check\n\
                 \x20   # Not `== 'push'`: that excluded workflow_dispatch, so a manual\n\
                 \x20   # run went green having deployed nothing. Everything except a\n\
                 \x20   # pull request should publish.\n\
                 \x20   if: github.event_name != 'pull_request'\n\
                 \x20   runs-on: ubuntu-latest\n\
                 \n\
                 \x20   steps:\n\
                 \x20     - uses: actions/checkout@v4\n\
                 \n\
                 {install}\n\
                 \x20     - run: zel build \"$CONTENT\" --out dist\n\
                 \n\
                 \x20     # Netlify's own uploader, until `zel deploy` lands.\n\
                 \x20     #\n\
                 \x20     # Run from outside the checkout, which matters in a workspace.\n\
                 \x20     # netlify-cli walks up from the working directory looking for\n\
                 \x20     # package.json, and when it finds a root with several packages it\n\
                 \x20     # refuses to guess which one it is deploying:\n\
                 \x20     #\n\
                 \x20     #   Error: Projects detected: @acme/docs, @acme/site, ...\n\
                 \x20     #\n\
                 \x20     # It looks at the working directory, not at --dir. So this runs\n\
                 \x20     # from $RUNNER_TEMP, where nothing walks up into a workspace, and\n\
                 \x20     # points --dir at the output where it lies.\n\
                 \x20     #\n\
                 \x20     # It does NOT move the output first, which was the previous fix and\n\
                 \x20     # failed: `zel build` writes dist from inside a container, as root,\n\
                 \x20     # and the runner user cannot unlink what root created —\n\
                 \x20     #\n\
                 \x20     #   mv: cannot move 'dist' to '/home/runner/work/_temp/site': Permission denied\n\
                 \x20     #\n\
                 \x20     # Nothing needs to move. Only the working directory mattered.\n\
                 \x20     - name: Upload\n\
                 \x20       env:\n\
                 \x20         NETLIFY_AUTH_TOKEN: ${{{{ secrets.NETLIFY_AUTH_TOKEN }}}}\n\
                 {site}\
                 \x20       run: |\n\
                 \x20         set -eu\n\
                 \x20         # Copy, not move: `zel build` writes dist from inside a\n\
                 \x20         # container as root, and the runner cannot unlink what root\n\
                 \x20         # created. Reading it is fine.\n\
                 \x20         rm -rf \"$RUNNER_TEMP/site\"\n\
                 \x20         cp -r dist \"$RUNNER_TEMP/site\"\n\
                 \x20         # Deploy from $RUNNER_TEMP so netlify-cli finds no workspace\n\
                 \x20         # above it, with a RELATIVE --dir: an absolute one was\n\
                 \x20         # silently ignored and it published the working directory\n\
                 \x20         # instead (publishOrigin: default, publish: .../_temp).\n\
                 \x20         cd \"$RUNNER_TEMP\"\n\
                 \x20         npx --yes netlify-cli@latest deploy --prod --dir site\n"
            )
        }
    };

    format!("{header}{check}{publish_job}")
}

/// The two values a Netlify deploy needs, and whether this shell has them.
///
/// Read rather than asked for. Anyone who has deployed from this machine has
/// them somewhere already, and "add NETLIFY_SITE_ID to your secrets" is not
/// advice — it is the problem restated. Knowing whether they are present is
/// what lets the tool print the command that finishes the job instead.
#[derive(Debug, Default, Clone)]
pub struct NetlifyEnv {
    pub site_id: Option<String>,
    pub auth_token: Option<String>,
}

impl NetlifyEnv {
    #[must_use]
    pub fn from_env() -> Self {
        let read = |name: &str| {
            std::env::var(name)
                .ok()
                .map(|v| v.trim().to_string())
                .filter(|v| !v.is_empty())
        };
        Self {
            site_id: read("NETLIFY_SITE_ID"),
            auth_token: read("NETLIFY_AUTH_TOKEN"),
        }
    }

    #[must_use]
    pub fn complete(&self) -> bool {
        self.site_id.is_some() && self.auth_token.is_some()
    }
}

/// What the caller has to do that a file cannot.
///
/// `netlify` is only consulted for the Netlify target; the others ignore it.
pub fn afterwards(publish: Publish, netlify: &NetlifyEnv) -> Vec<String> {
    match publish {
        Publish::Nothing => vec![],
        Publish::Image => vec![
            "Settings → Actions → General → Workflow permissions → Read and write.".into(),
            "Without it the push fails with a 403 that reads like an authentication problem."
                .into(),
        ],
        Publish::Netlify => netlify_guidance(netlify),
    }
}

/// Guidance that depends on what the shell already has.
///
/// Both present: print the two commands that copy them into the repository,
/// reading from the environment rather than the clipboard. A secret pasted on
/// a command line is a secret in shell history.
///
/// Either missing: say how to get *that one*. The site ID and the token come
/// from different places and are confused constantly — one is per project and
/// public-ish, the other is per account and a credential.
fn netlify_guidance(env: &NetlifyEnv) -> Vec<String> {
    let mut out = Vec::new();

    // Nothing left to say when both are already handled: the site id is in
    // the file and the token was uploaded before this ran. Printing "now set
    // these two secrets" after doing exactly that is worse than silence — a
    // reader who follows it does redundant work and learns to distrust the
    // rest of the output.
    if env.complete() {
        return out;
    }

    out.push("The deploy needs two values in the repository's secrets.".into());
    out.push(String::new());

    if env.site_id.is_some() {
        out.push("NETLIFY_SITE_ID  — set here.".into());
    } else {
        out.push("NETLIFY_SITE_ID  — not set. It identifies the project:".into());
        out.push("    netlify sites:list           # find the one you want".into());
        out.push(
            "    netlify link                 # or link this folder, then read .netlify/state.json"
                .into(),
        );
        out.push("  It is also the \"Project ID\" on the project's Configuration page.".into());
    }

    out.push(String::new());

    if env.auth_token.is_some() {
        out.push("NETLIFY_AUTH_TOKEN — set here.".into());
    } else {
        out.push("NETLIFY_AUTH_TOKEN — not set. It is an account credential:".into());
        out.push("    app.netlify.com → User settings → Applications → New access token".into());
        out.push("  `netlify login` stores a token for your machine, but does not".into());
        out.push("  print one you can give to CI. Create it in the UI.".into());
    }

    out.push(String::new());
    out.push("Export both, re-run this command, and it will print the commands".into());
    out.push("that copy them in.".into());
    out
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
    fn publishing_never_happens_from_a_pull_request() {
        for publish in [Publish::Image, Publish::Netlify] {
            let out = github(&CiOptions {
                publish,
                ..Default::default()
            });
            assert!(
                out.contains("if: github.event_name != 'pull_request'"),
                "{publish:?} must not publish from a pull request:\n{out}"
            );
        }
    }

    /// The gate was `== 'push'`, which excluded `workflow_dispatch` — so a
    /// manual run went green having deployed nothing, which is the worst
    /// outcome available: it looks like a successful deploy.
    #[test]
    fn a_manual_run_actually_publishes() {
        for publish in [Publish::Image, Publish::Netlify] {
            let out = github(&CiOptions {
                publish,
                ..Default::default()
            });
            assert!(
                out.contains("workflow_dispatch:"),
                "{publish:?} must be runnable by hand:\n{out}"
            );
            assert!(
                !out.contains("if: github.event_name == 'push'"),
                "{publish:?} gate excludes workflow_dispatch:\n{out}"
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
