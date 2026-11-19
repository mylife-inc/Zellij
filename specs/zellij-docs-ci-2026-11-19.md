# Automatic documentation deployment — 2026-11-19

The user authorized automatic production documentation publication. The workflow
is enabled, merged, and its first automatic deployment completed successfully.

## Source and CI

- [PR #210](https://github.com/mylife-inc/Zellij/pull/210) merged at
  `2026-11-19T20:50:55Z` as `c4c2bb4bb22f9a8a4148ef9742823bf159cbd6cd`.
- The merge tree exactly matches reviewed head
  `a59b8304d96a803b3aa6e8247a9dc9cd35269b8d`.
- [General CI 44605947631](https://github.com/mylife-inc/Zellij/actions/runs/44605947631):
  all 33 jobs passed, including CLI, Windows/macOS/Linux portable tests,
  Homebrew, images, templates and browser acceptance.
- [PR documentation validation 44605947651](https://github.com/mylife-inc/Zellij/actions/runs/44605947651):
  build passed; production publication intentionally skipped for the PR.
- Local publication/release-policy tests: 39 passed. Native documentation tests:
  91 passed, 0 ignored. Workflow lint, shell syntax and diff checks passed.

Two earlier CI findings were fixed before merge, with no checks bypassed:

1. `44602927256` found stale generated portable theme CSS. Commit
   `be1190838991a3972844a3551aba3b5186303878` adds the 135 missing preview
   declarations; removing only those additions reproduces the prior CSS bytes.
2. `44603977358` exhausted the default Node worker heap during macOS declaration
   emission. Commit `a59b8304d96a803b3aa6e8247a9dc9cd35269b8d` sets a 4096 MiB
   old-space limit for that build step. The later hosted macOS job passed.

## Automatic production run

[Deploy documentation 44612823122](https://github.com/mylife-inc/Zellij/actions/runs/44612823122)
was triggered by the merge's `push` event on `main`. Both jobs passed. Its
production token was provided through `NETLIFY_DOCS_PRODUCTION_TOKEN`; existing
staging credentials were not changed. The publisher verified the project/domain,
source ancestry and draft responses before promoting the checked artifact.

- Existing site: `zellij-docs`, `91774174-d4c2-41b5-aa4b-ac61eeadbf51`.
- Public domain: <https://zellij.shebka.net>.
- New deployment: `6aff622de7f414ee8b4d4a38`.
- Publication time: `2026-11-19T20:55:19.419Z`.
- Receipt state: `verified`.
- Previous deployment: `6aff4d3001a5a3ee682b3ff2`.
- Original pre-update deployment remains `6a7cb3dc45caa1c8e95f9c03`;
  see [the initial publication record](zellij-netlify-production-2026-11-19.md).

No new CLI version was released. The existing main-image workflow
[44612823252](https://github.com/mylife-inc/Zellij/actions/runs/44612823252)
also passed all 13 jobs and updated the development image variants. The separate
main-push CI run `44612823176` was still running at this evidence checkpoint;
the complete passing CI result above is for the identical PR tree.

## Verification and retained evidence

The runner's preservation gate passed for 46 original routes, 13 added routes,
282 original heading IDs, 82 compatibility mappings and 29 byte-identical
authored images, with current repository links and no captured browser errors
after hydration.

The workflow compared six hosted articles and the logo/favicon bytes on both
the draft and production domain. Independent verification then compared all
587 artifact-file digests with Netlify's published inventory; every digest
matched. One additional provider-generated `netlify.toml` matched the previously
reviewed provider configuration. Netlify's case-normalized inventory was checked
only after confirming the source contains no case collisions.

The new build retains all 411 non-framework file paths from the direct
deployment. Framework chunk splitting/names differ between builds, explaining
the total of 587 rather than the prior artifact's 588 files.

Normal visible Chrome checks passed on production for home hydration, client
navigation through CLI/build/templates/books, search navigation to compose,
the compose example, theme switching and both favicon conventions. No console
or page errors were captured. This is not an exhaustive remote browser crawl;
complete preservation checks ran against the local CI export.

The Actions site artifact is retained for 14 days and its publication receipt for
90 days. A durable ignored local copy is stored under
`backups/zellij-netlify-2026-11-19/automatic-ci-44612823122/`, including the
receipt, file manifest, verification output, browser log and CRC-checked ZIP.

- ZIP: `documentation-site.zip`, 587 files, 28,958,393 bytes.
- SHA-256: `a1a09b7d969c97075fa17254a91e7ddddbc560c0bcf0f578510c56bef4b65973`.

For routine use and recovery, see [the publishing guide](zellij-docs-deployment.md).
See also [machine-readable evidence](evidence/zellij-docs-ci-2026-11-19.json).
