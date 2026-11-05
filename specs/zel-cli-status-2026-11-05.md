# zel CLI implementation status — 2026-11-05

All registered command families have implementation handlers. The full CLI
design is **not complete**. Provider, format and advanced-option coverage must
be assessed separately from the existence of a command. This is the current
summary; the [original inventory](zel-cli-inventory-2026-10-17.md) retains the
dated audit and implementation history.

| Area | Implemented command forms | Remaining scope or limits |
| --- | --- | --- |
| Creation | `new portable` / `init`, `new docs`, `new app`, `new section`, `new page`, `new guide`, `new project`, with documented aliases | Interactive authoring and proposed per-field wizard flags remain unimplemented. |
| Inspection | `explain`, `list sections/themes/formats/targets`, `context`, `detect`, `doctor`, `version` | Doctor principally diagnoses container tooling; it is not a universal provider/site diagnostic. |
| Validation and local work | `check`, `fmt`, `watch`, `dev`, `build`, `serve`, `view` | Hosted operations need their documented runtime/engine prerequisites. JSON is supported per command, not universally. |
| Images | `image static`, `next`, legacy `build`, `dockerfile`, `run`, `push`; shortcuts including `si` and `ni` | Linux amd64/arm64, including combined publication through publish/ship/CI. These commands require a container runtime. |
| Delivery | `deploy`, `netlify`, `cloudflare` / `cf`, `vercel` / `vc`, `publish`, `ship` | Netlify, Cloudflare Pages and Vercel static adapters plus image registry publication are implemented. Cloudflare and Vercel live-account acceptance is pending. GitHub Pages, S3, Fly and Railway adapters remain unimplemented. |
| CI | `ci`, `ci init github`, `ci run` and legacy `ci publish` | GitHub Actions generation/dispatch work; GitLab/CircleCI adapters remain unimplemented. |
| Imports | `convert` / `cv`, `migrate` / `mig`; shared conversion in `ship` | MkDocs, Docusaurus, declarative VitePress, YAML Jekyll, declarative Hugo and a conservative Sphinx/reStructuredText subset. Full Sphinx/MyST/domain/extension fidelity remains incomplete. Plain Markdown is directly supported by the portable renderer. `--assets link` is not implemented. |
| Maintenance | `pull`, `upgrade`, `eject`, `features`, `completions` | Supported operations have documented runtime, version and filesystem requirements; they do not imply automatic dependency installation. |

Dispatch now handles every command explicitly. Only already-normalized aliases
are unreachable; adding a new command variant without a handler is a compile
error. The former generic “not implemented” catch-all was not reachable through
current parsed commands and has been removed. Existing alias tests exercise the
normalization contract.

Import website-link work adds root-relative and same-origin HTTP(S) Markdown
URL mapping for supported source prefixes/routes, preserving queries, fragments,
Unicode and asset bytes. Docusaurus routing is read only from direct literal
configuration. Relative document URLs now use source page URL directories and
report ambiguous slash policies or conflicting file interpretations. Versioned,
localized, multi-instance and custom plugin routing, HTML/JSX attributes and
relative website assets still require review. No source JavaScript is executed
and no URL is fetched. See [the conversion reference](../examples/zellij-docs/content/02-cli/convert.mdx).

VitePress imports accept literal `.vitepress/config.{js,ts,mjs,mts}` modules
and the standard `defineConfig` wrapper without evaluation. They read `srcDir`,
nested/multiple sidebars, page metadata, `base`/`cleanUrls` links and referenced
source/public assets. Dynamic config, rewrites, locales, exclusion patterns and
custom output/cache paths are rejected before writes. Vue, home layout and
custom Markdown presentation remain manual migration tasks. VitePress is new in
source; published rc.1 does not include it.

Jekyll imports accept YAML configuration, front-matter Markdown pages, enabled
documentation collections, literal scoped defaults and supported navigation
arrays. Links use source permalinks and deployment prefixes; eligible referenced
static assets respect exclusions, internal data/templates and generated output.
Liquid/Ruby/theme code is not executed. TOML, blog posts, dated collections,
globs, custom source routing and theme-specific navigation remain limited or
unsupported. See the conversion reference for exact accepted forms.

Hugo imports accept one TOML/YAML/JSON configuration, Markdown sections and
bundles, and all three front matter formats. Explicit output filename mapping
keeps `_index.md` section pages visible without losing source-file link lookup.
Visibility includes draft/date/headless/build rules; derived navigation uses
weight/date/title ordering. Source URLs and eligible content/static assets map
into editable hosted content. Menus, split configuration, modules, cascades,
localization, custom date rules and complex routing remain outside this subset.
Shortcodes and aliases are reported for manual work. See the conversion reference.

Sphinx imports read literal `conf.py` assignments, traverse explicit toctrees
and convert a conservative reStructuredText subset without Python or Docutils.
Supported headings, labels, flat lists, code, admonitions and images retain
content; document/section/download references and HTML-builder URLs map to
hosted guide routes and eligible assets. Unknown executable configuration,
extensions, MyST, API domains, includes, tables, substitutions, nested lists and
custom builders still require manual migration. Unlisted documents are omitted
and reported. The conversion reference specifies the exact accepted forms.

The newest published native candidate remains **`v0.2.0-rc.1`**. Current source
is newer and has not been published as rc.2. The old rc.2 publication proposal
predates subsequent fixes and should not be used unchanged. Production Netlify
migration also remains separate, pending the reported hosting-comment hydration
issue. Passing tests do not establish zero bugs or completion of the whole design.

PR #74 merged as `74abf77a6592633288692a1af21fee9905f3ace6` after all 16
checks passed on `5846c16c698aaa1212df6a42a89afe83c54aa877` in
[41991356062](https://github.com/mylife-inc/Zellij/actions/runs/41991356062).
Both post-merge workflows passed: main `41995023512`, base image `41995023365`.
This document's website-link milestone must still pass its own final-head CI
before merge; the previous PR's results do not validate new changes.

The first website-link CI run found a Windows MkDocs path-normalization bug.
Source `74bacea` fixes nested-page inventory and derived navigation paths;
final-head Windows and full CI acceptance are pending.

**PR #75 closure:** all 16 final-head checks passed in
[42001094217](https://github.com/mylife-inc/Zellij/actions/runs/42001094217) on
`bb2527a5f50b3b9ecd870c5bb5e3a9f9c20962b1`. It merged as
`2c5ac7caae98af6bf4ec552c467d2614904ef1fe`, with the identical tested source
tree `185ce605b71138a58d700b8e7a6c9e9f1850f77c`. Base-image post-merge run
`42005279024` passed; main `42005279080` was still running at this refresh.
The new relative-link milestone requires its own final-head verification.

The PR #75 post-merge Windows preview step subsequently failed an immediate
watch-announcement assertion. Source `a1003bd` waits for the separately relayed
stdout message, retaining bounded readiness, early-exit checks and the assertion.
This harness fix joins the relative-link PR and requires new Windows CI evidence.

**PR #76 closure:** relative website links and the Windows watch-readiness fix
merged as `e8f525d` after all 16 checks passed on `903c66d` in run `42007978630`.
Post-merge base-image run `42011273390` passed; main `42011273809` has 15 passing
jobs with CLI acceptance still running. New VitePress source `38b9d41` passes
140 local Rust tests, Clippy/formatting, deep/strict docs and all three Node
import-rendering fixtures. Its own hosted/native CI and merge remain pending.

**PR #77 closure:** VitePress merged as `0729235` after all 16 checks passed on
`155f09c` in `42014697065`. Both post-merge workflows passed (`42017999796`,
`42017999836`); PR #76's main run `42011273809` passed too. Current Jekyll
source `0073bca` passes 155 local Rust tests, Clippy/formatting, deep/strict docs
and all four Node rendering fixtures. Its own hosted/native CI and merge remain
pending; Hugo has not been implemented by this milestone.

### 2026-11-05 — Hugo documentation import implementation

Source `5631e6c` adds one-file TOML/YAML/JSON Hugo configuration, Markdown
sections/bundles, publication rules, source links and eligible assets through
convert/cv, migrate/mig and ship. Explicit output filename mapping preserves
visible `_index.md` pages and original source links. Menus, modules, cascades,
localization and complex routing remain limited. Local 171 Rust tests, Clippy,
strict docs and five Node rendering fixtures pass; final-head CI is pending.
See the current CLI status and the dated audit for precise scope and evidence.
PR #78 merged as `6190e51` after all 16 checks passed on `ffdeca7`; post-merge
base image passed, with main CI still running. Sphinx remains planned.


### 2026-11-05 — Sphinx reStructuredText implementation

Source `968b469` adds the Sphinx converter through detect/formats/convert/migrate
and ship. Literal conf.py, explicit toctrees, a conservative reStructuredText
subset, doc/ref/download roles, HTML-builder URLs and eligible referenced assets
are supported without Python execution or installation. Unsupported constructs
fail before writes; unlisted sources remain unpublished. Full Sphinx/MyST/domain
fidelity is incomplete. The same change repairs inline literal backtick escaping.
See the conversion reference for precise syntax and limits, and the context/audit
for evidence. Local 183 Rust tests, Clippy/formatting, strict docs and six Node
render fixtures pass; final-head native/hosted CI and merge remain pending.
PR #79 merged as `7a03f50` after all 16 checks passed on `8102782` in `42095877145`.
Both post-merge workflows passed (`42099649036`, `42099648776`); PR #78's main
`42090020416` passed too. Native rc.2 and production migration remain unpublished.


#### Sphinx literal resource follow-up — 2026-11-05

Source `6b5b1d2` preserves literal local image/download filenames before the
shared Markdown URL resolver. A source filename containing `%20` remains
separate from one containing a space; both original bytes and encoded emitted
URLs are covered. Root-relative resource paths retain source-root semantics.
Sphinx can also select localized image variants based on `language`, so nonempty
language settings are now rejected with guidance to choose resources explicitly
in a separate source copy. Absent/None/empty language settings remain supported.
The official [asset collector](https://github.com/sphinx-doc/sphinx/blob/master/sphinx/environment/collectors/asset.py)
was inspected read-only; no Python/Sphinx execution was introduced.

All 11 Sphinx integration tests, warnings-denied Clippy, formatting, strict docs
and all six Node rendering fixtures pass, including the percent/space ambiguity.
The expanded hosted export fixture checks real HTML URLs and copied bytes too.
Local distinct Rust coverage is now 184 tests. Initial PR #80 CI on `a850843`
had 14 passing jobs and no failures when this correction was prepared; that
older run does not validate this source. New final-head CI and merge are pending.


### 2026-11-05 — Cloudflare Pages delivery

Current source implements `deploy cloudflare`, `cloudflare` / `cf`, saved
publish/deploy, ship and GitHub CI generation for existing Pages projects.
Preview branch `zel-preview` is the default; production requires explicit intent.
Provider changes do not reuse another provider's destination. Environment token
and account ID stay outside configuration. Static-only limits, credentials and
failure behavior are documented in [deployment](../examples/zellij-docs/content/02-cli/deploy.mdx#cloudflare-pages).
Live Cloudflare account acceptance remains pending; published rc.1 is unchanged.
The context and audit retain protocol references, precise scope and evidence.
PR #80 merged as `c067b5a` after all 16 final-head checks passed on `fb622d1`.
Post-merge Windows preview encountered EPERM during a fixture directory rename;
this follow-up retries only transient Windows rename failures with a deadline.

Local validation: all **471 Rust workspace tests passed**, with the two real
hosted deployment tests deliberately ignored locally and selected explicitly
in CI. Final CLI/unit and CI-generator rechecks cover updated provider-token
provisioning. Warnings-denied workspace Clippy, Rust formatting, Node syntax,
workflow actionlint (including an emitted Cloudflare workflow), and strict/deep
Zellij documentation checks passed (55 routes, 23 examples). No local Docker or
browser process was started. Final-head hosted/native CI and merge remain pending.

Implementation commit **`8898a30`** contains the Cloudflare adapter, provider
resolution/CI changes, matching documentation and bounded Windows fixture rename
retry. Local evidence: `/private/tmp/zellij-cloudflare-workspace.log` (471 passed),
`zellij-cloudflare-final-tests.log` (40 CLI unit + 16 workflow tests passed),
`zellij-cloudflare-clippy.log`, and `zellij-cloudflare-docs.log`.
Final-head GitHub checks, hosted export acceptance and merge are pending.

Cloudflare follow-up **`6cd0517`** rejects API deployments marked `is_skipped`,
even if their stage otherwise appears successful. The protocol fixture covers
this case; all 13 local Cloudflare tests, Clippy and strict docs pass. The CLI
overview now explicitly lists both static providers. Initial PR #81 CI
`42118339455` on `8f2f327` had 11 passing jobs and no failures when this final
correction was prepared; it does not validate this follow-up. Final-head CI
and live Cloudflare account acceptance remain pending.

### 2026-11-05 — Vercel static deployment

Current source implements `deploy vercel`, `vercel` / `vc`, saved static
publication, import-and-ship and GitHub workflow generation. Existing project
IDs/names use `VERCEL_PROJECT_ID`; credentials use `VERCEL_TOKEN` and optional
`VERCEL_TEAM_ID`. Prebuilt static output is uploaded directly, with previews as
the default and production explicit. No Vercel CLI or remote source build is
required. Documentation covers limits and failure behavior; live Vercel account
acceptance remains pending. See the latest durable-context/audit entry for
protocol details and verification. Native rc.1 remains unchanged.

PR #81's Cloudflare implementation merged as `9cde253`, after all 16 checks on
`b1bf1da` passed in `42119403884`; the tested/merged tree is identical.

Vercel implementation commit **`28ea6f7`** passed all **487 local Rust workspace
tests**, with three real hosted-provider tests deliberately ignored locally and
selected explicitly in CI. Warnings-denied Clippy, formatting, strict/deep docs
(55 routes / 23 examples), workflow actionlint and an emitted Vercel workflow
passed. Evidence: `/private/tmp/zellij-vercel-workspace.log`,
`zellij-vercel-clippy.log`, `zellij-vercel-docs.log`, and generated fixture
`/private/tmp/zellij-vercel-ci-ba1_mb0e/.github/workflows/docs.yml`.
Final-head GitHub acceptance and merge remain pending. No local Docker/browser
or live provider deployment ran.
