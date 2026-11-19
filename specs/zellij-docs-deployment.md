# Publishing Zellij's own documentation

The **Deploy documentation** workflow (`.github/workflows/docs.yml`) publishes
`examples/zellij-docs` to https://zellij.shebka.net independently of CLI releases.

## Automatic and manual runs

A merge/push to `main` starts the workflow when it changes the documentation,
engine, dependency manifests, deployment workflow/helper, preservation checks,
installer asset or preservation baselines. Pull requests run the same validation
and build without receiving production credentials or publishing anything.
Changes only to unrelated files do not redeploy the site.

Merge only after the PR's required CI checks pass. Each deployment additionally
runs its own strict/deep content validation, production static export and local
browser preservation checks before publication. These deployment checks do not
wait for the separate, duplicate main-push CI run to finish.

For an intentional rebuild without a content edit, select **Actions → Deploy
documentation → Run workflow → main**, or run:

```sh
gh workflow run docs.yml --repo mylife-inc/Zellij --ref main
```

After sourcing `scripts/app.sh`, `zdocs-publish run` does the same thing.
`zdocs-publish status` shows the workflow and whether its secret exists;
`on`/`off` enable or disable this workflow through GitHub. Disabling it affects
its PR, automatic and manual runs; ordinary CI still checks documentation.

## Destination and credentials

Production is the existing Netlify project `zellij-docs`, ID
`91774174-d4c2-41b5-aa4b-ac61eeadbf51`, with custom domain `zellij.shebka.net`.
The publisher verifies both values before uploading. It does not create a
project, change DNS, transfer an account or enable Netlify Git integration.

Set repository Actions secret **`NETLIFY_DOCS_PRODUCTION_TOKEN`** to a Netlify
token that can deploy to this project. Enter it directly in GitHub or use
`gh secret set NETLIFY_DOCS_PRODUCTION_TOKEN --repo mylife-inc/Zellij` and paste
at the prompt. Never put the token in source, an issue, logs or a command argument.
The project ID is public configuration in the publisher, not a secret.

Existing `NETLIFY_AUTH_TOKEN`/`NETLIFY_SITE_ID` remain dedicated to the acceptance
fixtures and their staging destination. They are not fallbacks for production.
`DEPLOY_DOCS` is obsolete: release tags build docs but cannot publish them.

The build job has no production token. The publish job downloads its checked
artifact and installs pinned Netlify CLI 27.0.2 without credentials. Only the
publication step receives the production token and a read-only GitHub token.

## Publication and recovery

The publisher confirms that its source commit is on `main`, checks the current
project/domain and uploads a **draft**. It compares six hosted articles plus the
logo/favicon bytes against the checked artifact. It then verifies that production
has not changed during review and promotes that exact ready draft without a
rebuild. The same HTTP checks run on the public domain after promotion.

Production jobs are serialized without cancelling an active upload/promotion.
If a newer automated deployment is already live, an older queued job skips its
publication. Divergent history, a locked site, a wrong destination or failed
draft validation stops publication. Mutating API requests are not retried
because a lost response may still represent a completed operation.

The workflow retains the built site for 14 days and a publication receipt for
90 days. The job summary and receipt identify the source commit, draft/new deploy
and previous deploy. The original 2026-11-19 backup is also retained locally;
see [the initial publication record](zellij-netlify-production-2026-11-19.md).

If a post-publication check fails, inspect the receipt and Netlify's current
published deploy first. A network challenge is not proof that the artifact is
broken. Restore the recorded previous deploy from Netlify's Deploys screen when
rollback is warranted, then fix/revert source before the next automatic publish.
A failure before promotion leaves the previous production deploy in place.

Complete route, legacy-anchor, image and interactive search checks run locally
on the CI runner. Remote checks compare selected HTTP responses and assets;
they are not a claim of exhaustive browser testing against Netlify. The provider
previously blocked a rapid headless remote crawl with HTTP 403.
