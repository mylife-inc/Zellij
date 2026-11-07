# zel CLI implementation status — 2026-11-05

All registered command families have implementation handlers. The full CLI
design is **not complete**. Provider, format and advanced-option coverage must
be assessed separately from the existence of a command. This is the current
summary; the [original inventory](zel-cli-inventory-2026-10-17.md) retains the
dated audit and implementation history.

| Area | Implemented command forms | Remaining scope or limits |
| --- | --- | --- |
| Creation | `new portable` / `init`, `new docs`, `new app`, `new section`, `new page`, `new guide`, `new project`, with documented aliases | `new docs --full` adds all 23 section examples and local media. Opt-in `init -i` / `nd -i` site wizards choose name, theme, starter and an optional hosted guide. `np -i` chooses a page name and ordered section types; registration remains a printed `pages.yaml` entry. `na -i` chooses the local engine, content source, app name, port and optional features without installation. Section wizard, scalar `--set`, list `--count`, pricing `--tiers`/`--highlight` and positional output are implemented. Arbitrary YAML editing through flags remains outside this scope. |
| Inspection | `explain`, `list sections/themes/formats/targets`, `context`, `detect`, `doctor`, `version` | Doctor resolves site settings, skips runtime probes for portable sites, and offers JSON preflight reports. Content validation and provider credentials remain separate. |
| Validation and local work | `check`, `fmt`, `watch`, `dev`, `build`, `serve`, `view` | Hosted operations need their documented runtime/engine prerequisites. JSON is supported per command, not universally. |
| Images | `image static`, `next`, legacy `build`, `dockerfile`, `run`, `push`; shortcuts including `si` and `ni` | Linux amd64/arm64, including combined publication through publish/ship/CI. These commands require a container runtime. |
| Delivery | `deploy`, `netlify`, `cloudflare` / `cf`, `vercel` / `vc`, `github-pages` / `ghp`, `s3`, `fly`, `railway` / `rw`, `publish`, `ship` | Netlify, Cloudflare Pages, Vercel, GitHub Pages and S3 static adapters plus image registry publication are implemented. Cloudflare, Vercel, GitHub Pages and S3 live-account acceptance is pending. Fly supports existing configured web Machines with static/Next images; live account acceptance is pending. Railway supports existing image services with an explicit project/environment and registry image; live account acceptance is pending. |
| CI | `ci`, `ci init github\|gitlab\|circle`, `ci run` and legacy `ci publish` | All three generators are implemented in current source. Dispatch/provisioning remain GitHub-only; live GitLab/CircleCI runner acceptance and the next native release remain pending. |
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
Supported headings, labels, nested bullet/decimal/alphabetic/Roman lists, code, admonitions, images and rectangular
grid/simple tables, list tables, inline CSV tables and document-local text/image
substitutions retain content;
document/section/download references and HTML-builder URLs map to
hosted guide routes and eligible assets. Unknown executable configuration,
extensions, MyST, API domains, includes, spanning cells, external CSV sources,
custom table presentation/escape options, global content transforms and
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

PR #82 initial run `42126186780` exposed a Windows-only fixture setup error:
`bad?.html` is not a legal Windows filename, so the test failed before calling
Zel's validation. Fix **`4475d66`** uses `bad#.html`, legal on all supported OSes
but rejected by the same upload-path rule. No assertion or Windows test is
skipped. All 11 local Vercel tests pass after the correction (one real hosted
test remains CI-only); formatting passes. Log:
`/private/tmp/zellij-pr82-windows.log`, with local recheck in
`/private/tmp/zellij-vercel-final-tests.log`. Final-head CI is still required;
the initial run does not validate this correction.


### 2026-11-05 — GitHub Pages publication

Current source implements `deploy github-pages`, `github-pages` / `ghp`, saved
publication, ship and GitHub workflow generation. Existing gh-pages root sources
and explicit production intent are required; this provider has no draft mode.
Hosted URL defaults follow the existing Pages mount. Non-forced commit updates
preserve history and CNAME; success requires the matching commit's Pages build.
See [deployment setup and limits](../examples/zellij-docs/content/02-cli/deploy.mdx#github-pages)
and the latest durable-context/audit entry for implementation and evidence.
Live Pages account acceptance and final-head CI remain pending. PR #82 merged
as `c8b485c` after all 16 checks passed; both its post-merge workflows passed.
Published native rc.1 remains unchanged; rc.2 is not published.


Local verification: all **505 Rust workspace tests passed**, with four real
hosted-provider tests intentionally ignored locally and explicitly selected in
CI. The final CLI/import/workflow recheck passed after credential-environment
isolation and production-before-conversion assertions. Warnings-denied Clippy,
Rust formatting, strict/deep docs (55 routes / 23 examples), workflow actionlint
and an emitted Pages workflow passed. Evidence: `/private/tmp/zellij-pages-workspace.log`,
`zellij-pages-final-tests.log`, `zellij-pages-clippy.log`, `zellij-pages-docs.log`,
and `/private/tmp/zellij-pages-ci-zfwwsu3_/.github/workflows/docs.yml`.
Final-head GitHub native/hosted checks and merge remain pending. No live provider
publication, local Docker or browser process was started.


GitHub Pages implementation commit **`b72e851`** contains the adapter, CLI/CI
integration, regression tests and matching docs. Local verification above covers
that implementation; final-head native/hosted CI and live account acceptance
remain pending. The subsequent context-only commit records this source identity.


PR #83 run `42135062241` exposed a Windows response-read deadline diagnostic:
expiration while reading the HTTP body was reported as a generic read failure.
Fix **`5dc95ec`** recognizes both reqwest's wrapped I/O timeout and the operation
deadline, retaining response/token redaction. A regression delays the response
body after headers. The original stale/missing-build timeout assertions remain;
no Windows test or assertion was disabled. All 13 local Pages tests passed
(one hosted test remains CI-only), with warnings-denied Clippy and formatting.
Distinct local Rust coverage is now 506 tests. Evidence:
`/private/tmp/zellij-pr83-windows.log` and `zellij-pages-timeout-tests.log`.
The first run cannot validate this correction; final-head CI and merge remain
pending, as does live GitHub Pages account acceptance.

### 2026-11-05 — AWS S3 static website uploads

Current source implements `deploy s3`, `s3`, saved `publish`/`pub`, `ship` and
GitHub CI generation for an existing S3 website bucket. Destination is bucket
or bucket/prefix; production intent is mandatory. Native SigV4 uploads use
explicit environment credentials, region and expected owner. Hosted prefixes
are applied once and clean-route HTML copies are included. All object
preconditions are captured before conditional writes; uploads remain non-atomic
and old keys remain. No bucket, policy, ACL, DNS or CDN setup is performed.
See [setup and limits](../examples/zellij-docs/content/02-cli/deploy.mdx#aws-s3).
Live AWS account acceptance is pending. The published rc.1 lacks this provider;
rc.2 remains unpublished. Fly/Railway, other CI systems, wizard and advanced
import gaps remain separate work.


### 2026-11-05 — Fly image deployment to existing apps

Current source implements `fly`, `deploy fly`, saved publication, import-and-ship
and GitHub workflow generation. `--kind static|next` selects the image; saved
`publish.image` is supported only for Fly deployments. Static is the default.
Explicit `--prod` is mandatory. App selection uses explicit site, FLY_APP, then
same-provider saved site. FLY_ORG guards organization identity; FLY_API_TOKEN
supports one raw or scheme-prefixed token. Credentials are never saved in YAML.

The native Machines API adapter requires 1–10 already-started, healthy web
Machines, HTTP services on port 3000, autostop disabled, and no additional process
groups, mounts, injected files or entrypoint overrides. It preserves full config
and changes only the image. Existing shared image builders produce small static
or Next wrappers for linux/amd64; no local runtime is installed or started.
The selected runtime must already be authenticated to registry.fly.io. Unique
tags and a validated repository digest identify the image for deployment.

Preflight precedes building. After one build/push, identity, membership and
versions are rechecked. All leases are acquired before updates, with optimistic
version guards. Sequential updates wait for the new instance, matching digest,
and fresh passing checks. Final state is checked before releasing leases.
Five-minute rollout budget, ten-minute leases, bounded/redacted HTTP responses,
no redirects, retries or automatic rollback. Failures may leave a partial rollout
and retained images. This does not create/configure apps, Machines, networking,
secrets, scaling or Fly release records; fly.toml is not applied. Public DNS/TLS
and reachability are not acceptance claims.

GitHub PR jobs build the selected image without credentials; authorized publish
events log in with stdin, deploy once and log out. Provisioning remains explicit.
Live Fly account acceptance is pending. Shared real static/Next image acceptance
runs in existing CI; protocol/fake-client fixtures do not prove live Fly behavior.
Railway, other CI systems, wizard and advanced imports remain separate work.
Rc.2 is unpublished pending explicit release authorization; production Netlify
migration remains blocked by the reported hosting-comment hydration issue.

Protocol references: Fly [Machines API](https://docs.fly.io/machines/api/machines-resource),
[Apps API](https://docs.fly.io/machines/api/apps-resource),
[registry](https://docs.fly.io/blueprints/using-the-fly-docker-registry), and
[official SDK token normalization](https://github.com/superfly/fly-go/blob/master/tokens/tokens.go).
See [user setup and exact limits](../examples/zellij-docs/content/02-cli/deploy.mdx#fly).
Final source identity and verification are recorded in the subsequent entry.


PR #84 closure: S3 merged as `9e3dd79e48c2cd256e16f9e24085fcf5a45b78c4`
after all 16 checks passed on `d141aa8ed44fff8643601e2f1a93b21053cc9be2`
in run `42150590098`, attempt 2. Only the Ubuntu job was retried after slow
Azure package downloads exhausted its time limit; assertions were unchanged.
Tested/merged tree: `4d85791532ca267dff6a1bf028307f7196c1924e`.
Both post-merge workflows passed: main `42155106031`, base image `42155106146`.

Fly local verification: **541 distinct Rust tests passed** (538 in the full
workspace run, plus three added cases in final focused runs). Final CLI binary,
delivery, import and workflow suites passed 149 tests, with five real hosted
provider fixtures intentionally ignored locally and selected in existing CI.
Warnings-denied workspace Clippy, formatting, strict/deep documentation (55 routes,
23 examples), repository actionlint and an emitted Fly workflow passed.
Evidence: `/private/tmp/zellij-fly-workspace.log`, `zellij-fly-final-tests.log`,
`zellij-fly-clippy.log`, `zellij-fly-docs.log`, and generated workflow
`/private/var/folders/1p/q48lnmt15932t129b48z_mrr0000gn/T/zellij-fly-ci-22txs7eu/.github/workflows/docs.yml`.
No local Docker/browser or live Fly mutation ran. Final-head cross-platform CI
and merge remain pending; live provider acceptance remains a separate limitation.


Fly implementation commit **`a9b650c`** contains the native adapter, static/Next
selection, delivery/import/CI wiring, protocol and fake-tool regression tests,
and matching documentation. Local verification above covers this source.
Final-head GitHub CI and merge remain pending; live Fly account acceptance is
not implied by loopback fixtures or shared image tests.


Fly compatibility follow-up **`ecad3ca`** accepts the API's default init object
with `tty: false` while retaining rejection of custom command/TTY overrides.
Update receipts may omit `updated_at`; readiness then uses the subsequent GET's
timestamp only after verifying the new instance ID and exact requested config.
The baseline fixture now includes Fly's default init shape, and a new regression
covers sparse update receipts. All 10 Fly tests and warnings-denied workspace
Clippy pass. Distinct local Rust coverage is **542** tests. Evidence:
`/private/tmp/zellij-fly-compat-tests.log`, `zellij-fly-compat-clippy.log`.
PR #85's first run on `1b272fa` cannot validate this correction; the final head
requires its own complete CI run before merge. Live Fly acceptance remains pending.


### 2026-11-05 — Fly PR #85 closure

Fly implementation `a9b650c` and API compatibility correction `ecad3ca` merged
in PR #85 as `30a02721d6d703adffe9ccc93ecc5b4e9b7cf5a0`, after all 16 checks
passed on final head `101e5d4dd196e593a460b1375f32fa2b0bac2347` in run
`42160670652`. Tested/merged tree: `9ab7fa9981ded274ba132aa3a568d6e3fe419d44`.
Both post-merge workflows passed: main `42163948890`, base image `42163948889`.
Native releases remain rc.1 and stable 0.1.3; no rc.2 tag/release was created.
Live Fly account acceptance remains pending.


### 2026-11-05 — Railway image deployment to existing services

Current source adds `railway` / `rw`, `deploy railway`, saved `pub`, imported
`ship`, and GitHub CI delivery. Static/Next images use the existing compact
builders with linux/amd64. `image.name` or `--tag` is required; supported remote
registries are GHCR, Docker Hub, GitLab, Quay and MCR. The image is built once,
pushed, and its unambiguous SHA-256 RepoDigest is used as the service source.
Bounded digest inspection is shared with Fly; Docker Hub's abbreviated receipt
names are normalized without admitting other repositories or ambiguous hashes.

The native public GraphQL adapter uses an environment-scoped project token in
`RAILWAY_TOKEN` and requires expected `RAILWAY_PROJECT_ID` and
`RAILWAY_ENVIRONMENT_ID`. Destination precedence is explicit service UUID,
`RAILWAY_SERVICE_ID`, then same-provider `publish.site`. `--prod` is mandatory;
`--kind` overrides same-provider `publish.image`, defaulting to static. Ship
requires an explicit tag and saves the registry/image/provider selection without
credentials or implicit production permission. CI gets the tag from image.name.

Preflight verifies token scope, project/environment/service identity, no staged
changes or attached volumes, a successful existing image deployment, startup
healthcheck, PORT=3000, compatible HOSTNAME, and no custom start/pre-deploy/cron,
repository config or sleeping. A complete volume inventory is required (at most
100 per environment). It rechecks the service after building, patches only the
environment-scoped image source, calls serviceInstanceDeployV2 once, validates
the returned deployment identity, polls within five minutes, and rechecks the
source/settings/latest deployment before success. Responses are capped, redirects
refused and GraphQL partial errors rejected without printing response details.
No automatic retry, rollback, resource creation or variable write occurs.

The API update/deploy sequence has no atomic lock; concurrent external writes
can still race. Failed operations can leave a new source or deployment running;
the registry image remains. SUCCESS reports Railway startup deployment status,
not independent attestation of the running digest, public DNS/TLS, reachability,
or continuing health. Live Railway account acceptance is pending.

GitHub verification jobs build the selected image without credentials. Publish
jobs log in to the explicit registry (GHCR GITHUB_TOKEN/packages:write, otherwise
REGISTRY_USERNAME/PASSWORD) and scope the Railway token to the deploy step.
Railway needs its own pull access; local registry login is not sufficient.
Release availability is not checked by generation; rc.2 remains unpublished.

Protocol references: Railway [authentication](https://docs.railway.com/integrations/api),
[services](https://docs.railway.com/integrations/api/manage-services),
[healthchecks](https://docs.railway.com/deployments/healthchecks), and
[official CLI schema at bd4d490](https://github.com/railwayapp/cli/blob/bd4d490c11d084c39d7cdea114708a9995d1ccc0/src/gql/schema.json).
The experimental environmentApplyChangeSet endpoint is not used.
See [setup and exact limits](../examples/zellij-docs/content/02-cli/deploy.mdx#railway).
Final source identity and verification will be recorded below.


Railway local verification: **556 Rust tests passed**, with five existing real
hosted-provider fixtures intentionally ignored locally and selected in CI.
The final Railway import/workflow fixtures also passed after isolating their
processes from inherited Railway environment variables. Warnings-denied workspace
Clippy, formatting, strict/deep documentation (55 routes, 23 examples), repository
actionlint and an actual emitted Railway workflow passed. Evidence:
`/private/tmp/zellij-railway-workspace.log`, `zellij-railway-final-focused.log`,
`zellij-railway-clippy.log`, `zellij-railway-docs.log`; the workflow path is saved
in `/private/tmp/zellij-railway-generated-workflow-path`.
No local Docker/browser or live provider mutation ran. Final-head GitHub CI and
merge remain pending; live Railway account acceptance remains separate.


Railway implementation commit **`3ace654`** contains the adapter, shared digest
handling, CLI/CI/import wiring, regression fixtures and matching user docs.
The local verification above covers this implementation. Final-head GitHub CI
and merge remain pending; no live Railway acceptance or native release is implied.


The final Railway timeout fixture leaves five seconds for loopback requests and
uses the polling interval to exhaust the remaining budget, avoiding a 100 ms
Windows scheduler race. Production deadlines are unchanged. All nine Railway
protocol tests passed again; evidence: `/private/tmp/zellij-railway-protocol.log`.
PR #86 must validate its final head before merge.

## 2026-11-06 CI system milestone

Railway PR #86 merged as `b24b3dc`; all 16 final-head checks and both post-merge
workflows passed. GitLab/CircleCI generators are now implemented in current
source, covering portable/hosted builds, static/Next images, combined native
architectures and the seven existing deployment providers. Saved `ci.system`
and short `ci --system` forms select them. These are standalone configurations,
not mergers of existing pipelines. `ci run`/`--provision` remain GitHub-only.
Schedules and protected credential scopes need provider-side setup; actual
GitLab/CircleCI project acceptance and native release publication remain pending.

## 2026-11-06 — CI generator closure and section authoring

GitLab/CircleCI PR #87 merged as `b590cb2dd1309601525f663285dfc98c1dc00406`
after all 16 checks passed on `3eb218812b33163d228cf0ef92bbb5d576081079` in
run `42201229972` (attempt 1). Tested and merged tree:
`7398f45e8e1e30797fba9c70436f5617efeac79d`. Both post-merge workflows passed:
main `42205322276` (16 jobs), base image `42205322053` (13 jobs). The native
AMD64/ARM64 jobs replayed generated provider publication and manifest scripts
against a disposable registry; actual GitLab/CircleCI runner acceptance remains
separate. The published native candidate remains rc.1; no rc.2 was published.

The section authoring milestone adds native prompts, scalar/list options and
pricing shortcuts, with shared engine/native cross-field rules and no-overwrite
output. Local verification: 574 Rust tests (five existing hosted fixtures remain
CI-only), 424 engine tests, all 23 types checked against actual engine schemas,
Clippy, formatting, TypeScript, strict/deep docs and actionlint pass. The durable
context and updated creation reference describe exact behavior and limits.
Final-head cross-platform CI and merge remain pending.

Implementation commit **`ff42301adc435f697b8bb729f2aceb91e11f8eca`** contains the section wizard, native
validation, shared engine metadata, cross-platform regression checks and user
documentation. The verification above covers this source; the following
context-only commit records its identity. Final-head GitHub CI is pending.

## 2026-11-06 — Section authoring closure and full starter

Section authoring PR #88 merged as `5078b51e8e01d326fc2b391da4381a00099bf126`
after all 16 checks passed on `fe69695b152c09e3a55a426bed91ebe8f7b6fadc` in
run `42210390950` (attempt 1). Tested and merged tree:
`578c819fd9c049a6d68c38e55111d1a151a90629`. The PR body records the verified
merge and source-tree equality. Post-merge base image `42214445273` passed all
13 jobs. Main run `42214445315` passed 15/16: Windows ARM installer job
`127880840580` failed upgrading a separate installation to the higher fixture
version with Windows error 32 after download. The log did not identify the
individual file operation or lock holder; it is not evidence of a schema or
section-authoring failure, nor a clean post-merge run.

Current source adds bounded retries for Windows sharing/lock violations (32/33)
during candidate permission changes, staged-archive removal and the Windows
rename/restore sequence. Each operation has a five-second budget; other errors
fail immediately. Errors identify the operation. Cancellation is checked before
the old executable moves, then commit/restoration complete without cancellation.
Existing checksum/version/architecture checks and backup retention are unchanged.
The Windows x64/ARM64 package jobs now run tests with actual exclusive file
handles as well as the existing real self-upgrade fixture. Unit cases also cover
non-Windows behavior, permanent errors, exhausted budgets and cancellation.
The exact original lock holder remains unknown; this handles the observed error
class rather than claiming a particular antivirus/process diagnosis.
References: [Windows error codes](https://learn.microsoft.com/en-us/windows/win32/debug/system-error-codes--0-499-)
and [file move/replacement sharing requirements](https://learn.microsoft.com/en-us/windows/win32/fileio/moving-and-replacing-files).

Current full-starter source adds a catalogue, all 23 editable section examples
and local SVG/MP4 media to `new docs --full` / `nd --full`, including `--guide`
and collision protection for binary files. See the durable context and creation
reference for architecture, coverage and limits. Final-head cross-platform CI
and merge remain pending.

Local verification: **579 Rust tests passed**, with five existing hosted-provider
fixtures reserved for CI. The final cancellation assertion also passed all seven
upgrade unit tests. Warnings-denied workspace Clippy and the final CLI Clippy
check pass, as do formatting, actionlint, strict/deep documentation (55 routes,
23 examples) and the full-starter engine boundary script. The first unprivileged
CLI-unit attempt lacked loopback socket permission; the full suite above ran
with loopback access and passed. Actual Windows lock tests and hosted full-site export still need final-head
GitHub acceptance.
Evidence: `/private/tmp/zellij-full-{workspace,starter-tests,upgrade-tests,
engine-check,clippy,final-clippy,docs}.log`.

Windows upgrade recovery source: **`608e5e8db1f7c77380493063f7fb73c467c9fe83`**.
It adds operation-specific diagnostics, bounded sharing-lock retries and native
Windows file-lock regression coverage. Final-head GitHub acceptance is pending.

Full starter implementation: **`54f601dc9701a11ef3d2819874b26c96950cac9f`**.
The preceding local verification covers this source and Windows recovery
`608e5e8`; this context-only follow-up records their identities. Final-head
GitHub acceptance and merge remain pending.


## 2026-11-06 — Doctor site/configuration diagnostics

Current source implements `doctor [site]`, shared site/config/working-directory
selectors, portable settings checks without a runtime, and JSON preflight reports.
Hosted failures retain selected site information. Outside a site the existing
runtime-only diagnostic remains available. Content, output readiness, credentials
and provider connectivity are not included. Regression tests cover fake clients
and native filesystem behavior; final-head cross-platform CI and publication
remain separate. See the [doctor reference](../examples/zellij-docs/content/02-cli/doctor.mdx).

Doctor implementation: **`1c2091b922e136b70118ba59549eedb00d2c35c0`**.
Local verification for this source: 586 Rust tests passed, five existing hosted
fixtures reserved for CI; warnings-denied workspace Clippy, formatting,
actionlint and strict/deep documentation (55 routes, 23 examples) passed.
Final-head GitHub CI and merge remain pending.

The preceding full-starter/Windows-recovery milestone merged in PR #89 as
`479eac4` after all 16 final-head checks passed on `80ba76c` in `42267735005`.
Its post-merge base-image run `42272858763` passed. Main run `42272858943`
passed on attempt 2 after retrying only an Ubuntu browser dependency-download
timeout at the unchanged merge SHA. The durable context/audit retain the
first attempt and complete acceptance evidence.


Doctor CI follow-up: run `42297958566` on `8ee56f9` caught a missing legacy
`/cli/doctor#exit-code` target after the heading was renamed to describe JSON.
The documentation now retains that ID as an explicit legacy anchor. Existing
content-preservation assertions remain unchanged. This failure was a real
bookmark regression; the corrected final head must pass its own complete CI.

Bookmark fix: **`22277623932783505d555da2a693bbb9c8ed05f7`**. Strict/deep documentation checks pass
(55 routes, 23 examples); the browser preservation check runs only in CI.


## 2026-11-06 — Doctor closure and Sphinx table imports

PR #90 merged as **`e6733067bb49ee7f831896f2659a2dfa7300c203`** after all 16
checks passed on **`0fb8da2fa97df6cbbdf035fe8b407780aed93b28`** in
[42298531414](https://github.com/mylife-inc/Zellij/actions/runs/42298531414),
attempt 1. Tested and merged tree: `28d7416e410217080dfc31793d6d2d25b9f83a80`.
Doctor cases passed on Windows/macOS/Linux; all six native packages, both image
architectures/manifests and broader hosted/browser tests passed. Preservation
retained 46 original routes, 282 original heading IDs, 82 compatibility mappings
and 29 byte-identical images. Source `1c2091b` and bookmark fix `2227762` are
identified in the preceding history. Post-merge main run `42303348733` passed
all 16 jobs; base-image run `42303348916` passed all 13. Both were refreshed
before this milestone. The current repository release inventory lists rc.1;
no rc.2 publication was attempted.

Current work adds rectangular Sphinx grid/simple tables and optional captions
through `.. table::`, without Python or a source build. The table parser in
`cli/crates/zellij-convert/src/sphinx/tables.rs` validates border geometry and
uses `unicode-width` (locked 0.2.3) for wide/combining character columns. Simple
rows preserve first-column continuation semantics, paragraph breaks and the
unbounded final column. Grid cells preserve their common margin and multiline
content. Both forms retain headerless tables and multiple header rows.

The RST block model now has a Table node; cells reuse the existing supported
block parser, sharing a 16-level table/admonition depth limit. Rendering emits
semantic HTML table/head/body/caption/cell tags with Markdown inside cells.
This preserves supported inline markup, lists, admonitions, code, images and
references through the existing MDX, link rewriting, asset, heading and search
pipeline. Source HTML stays literal. Tables have at most 128 columns, 10000
rows, 100000 cells and a 16384-column border. No filesystem links, outside reads,
source execution or network fetching are introduced.

Spanning cells, malformed geometry, list/csv tables, table directive options,
and headings/section labels/toctrees inside cells still fail with the source
filename before output is created. Existing unsupported RST content remains
unsupported inside cells; no lossy flattening is attempted. The conversion
reference and CLI status describe these limits. A small missing comma in the
JSON command reference is corrected too.

The shared `scripts/fixtures/sphinx-tables.rst` is exercised by native CLI
integration tests and `scripts/check-conversion.mjs`. Node-only MDX rendering
and deep checks passed locally across all six importers; the Sphinx fixture
verifies semantic headers, Unicode, captions, blocks, escaped source HTML,
code containing pipes/braces, links, exact assets and searchable cell text.
The same script checks the real hosted export in GitHub CI. Local Rust tests
cover malformed/spanning/unsafe forms, boundaries, dry run and asset skip.
No local Docker or browser ran. Final local suite and cross-platform CI remain
pending; live providers, native release and Netlify migration remain separate.

References: [Docutils table syntax](https://docutils.sourceforge.io/docs/ref/rst/restructuredtext.html#tables),
[table directive](https://docutils.sourceforge.io/docs/ref/rst/directives.html#table),
[Unicode width implementation](https://github.com/unicode-rs/unicode-width).


Local verification: the workspace suite passed **592 tests**, with five existing
hosted-provider fixtures reserved for CI. After tightening the depth guard for
empty cells, all **15 Sphinx CLI integration tests** passed, including the new
16-level acceptance/17-level rejection case (593 distinct native cases across
these checks). Final warnings-denied workspace Clippy, formatting, actionlint,
strict/deep documentation (55 routes, 23 examples), and Node-only MDX rendering
across all six converters passed. Logs are under
`/private/tmp/zellij-tables-{workspace,import-final,clippy-final,engine-final,docs}.log`.

CI additionally runs `scripts/check-imported-tables.mjs` from conversion export
acceptance. It checks browser hydration, semantic header/body cells, local image
loading and document navigation under the deployment prefix, capturing console
and runtime errors and rejecting external requests. The existing Chromium setup
step moves before conversion acceptance; it is not duplicated. This helper is
not imported by local server-render-only checks. Final-head CI and merge remain
pending.

Sphinx table implementation: **`2b06d934c3cc7d77456298368d930c492dfb3d5d`**. Local evidence above covers
this source, including the final empty-cell depth guard. Final-head GitHub
acceptance and merge remain pending.
