# zel CLI implementation status — 2026-11-05

All registered command families have implementation handlers. The full CLI
design is **not complete**. Provider, format and advanced-option coverage must
be assessed separately from the existence of a command. This is the current
summary; the [original inventory](zel-cli-inventory-2026-10-17.md) retains the
dated audit and implementation history.

| Area | Implemented command forms | Remaining scope or limits |
| --- | --- | --- |
| Creation | `new portable` / `init`, `new docs`, `new app`, `new section`, `new page`, `new guide`, `new project`, with documented aliases | `new docs --full` adds all 23 section examples and local media. Opt-in `init -i` / `nd -i` site wizards choose name, theme, starter and an optional hosted guide. `np -i` chooses a page name and ordered section types; registration remains a printed `pages.yaml` entry. `na -i` chooses the local engine, content source, app name, port and optional features without installation. Section wizard, scalar `--set`, complete `--set-json` values, optional-field `--unset`, list `--count`, pricing `--tiers`/`--highlight` and positional output are implemented. These author new sections; editing existing YAML files remains outside this scope. |
| Inspection | `explain`, `list sections/themes/formats/targets`, `context`, `detect`, `doctor`, `version` | Doctor resolves site settings, skips runtime probes for portable sites, and offers JSON preflight reports. Content validation and provider credentials remain separate. |
| Validation and local work | `check`, `fmt`, `watch`, `dev`, `build`, `serve`, `view` | Hosted operations need their documented runtime/engine prerequisites. JSON is supported per command, not universally. |
| Images | `image static`, `next`, legacy `build`, `dockerfile`, `run`, `push`; shortcuts including `si` and `ni` | Linux amd64/arm64, including combined publication through publish/ship/CI. These commands require a container runtime. |
| Delivery | `deploy`, `netlify`, `cloudflare` / `cf`, `vercel` / `vc`, `github-pages` / `ghp`, `s3`, `fly`, `railway` / `rw`, `publish`, `ship` | Netlify, Cloudflare Pages, Vercel, GitHub Pages and S3 static adapters plus image registry publication are implemented. Cloudflare and GitHub Pages portable/hosted live acceptance passed on 2026-11-10; Vercel and S3 remain pending. Fly supports existing configured web Machines with static/Next images; live account acceptance is pending. Railway supports existing image services with an explicit project/environment and registry image; static/Next live acceptance passed on 2026-11-10. Cloudflare/Railway live fixes require current source after rc.2. |
| CI | `ci`, `ci init github\|gitlab\|circle`, `ci run` and legacy `ci publish` | All three generators are implemented in current source. Dispatch/provisioning remain GitHub-only; live GitLab/CircleCI runner acceptance remains pending. Candidate `0.2.0-rc.2` includes all three generators. |
| Imports | `convert` / `cv`, `migrate` / `mig`; shared conversion in `ship` | MkDocs, Docusaurus, declarative VitePress, YAML/TOML Jekyll, declarative Hugo and a conservative Sphinx/reStructuredText subset. Full Sphinx/MyST/domain/extension fidelity remains incomplete. Plain Markdown is directly supported by the portable renderer. `--assets copy\|link\|skip` is implemented in current source; link requires `--asset-base-url` and retains assets on that existing HTTP(S) server. |
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
custom Markdown presentation remain manual migration tasks. VitePress is included in candidate `0.2.0-rc.2`.

Jekyll imports accept YAML or TOML configuration, front-matter Markdown pages, enabled
documentation collections, literal scoped defaults and supported navigation
arrays. Links use source permalinks and deployment prefixes; eligible referenced
static assets respect exclusions, internal data/templates and generated output.
Liquid/Ruby/theme code is not executed. Derived collection navigation supports
scalar `sort_by` after defaults; explicit order arrays take precedence and stale
entries are ignored/reported. TOML and collection sorting are source-only after
rc.2; see the latest dated entry for final-head verification and merge status.
Blog posts, dated collections,
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
grid/simple tables, list tables, inline/local-file CSV tables and document-local text/image
substitutions, local parsed `include` fragments and UTF-8 `literalinclude` excerpts retain content;
document/section/download references and HTML-builder URLs map to
hosted guide routes and eligible assets. Unknown executable configuration,
extensions, MyST, API domains, alternate include parsers, spanning cells, remote CSV sources,
custom table presentation/escape options, global content transforms and
custom builders still require manual migration. Unlisted documents are omitted
and reported. The conversion reference specifies the exact accepted forms.

The newest published native candidate is **`v0.2.0-rc.2`**, from approved commit
`54e636faf2162c36e50a91507f1f35be63a660d0`. All 29 Release jobs passed, including
public downloads/installers for six native targets and four image variants on
both Linux architectures. See the [release evidence](evidence/zellij-rc2-release-2026-11-09.json).
PR #105 merged the rc.2 delivery pins and active documentation as `5677577`.
All six installed-CLI manual delivery runs passed, including static/Next image
publication and portable/hosted Netlify drafts. Both draft HTTP checks and both
fresh-pull site-image checks passed; see the [delivery evidence](evidence/zellij-rc2-delivery-acceptance-2026-11-09.json).
Production Netlify migration remains pending the reported hosting-comment
hydration issue; HTTP fixture acceptance does not resolve it. Passing tests do
not establish zero bugs or completion of the whole design.

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


## 2026-11-07 — Structured section merge and Sphinx literal inclusions

PR #99 merged as **`6c01f4a30c9314a03132a8bda2f0fce6bd30cdb4`** after all
16 checks passed on `a3681ff5623b46253e3b5f73e48cc31c516c47c0` in
[42402060708](https://github.com/mylife-inc/Zellij/actions/runs/42402060708),
attempt 1. Tested/merged tree: `cea5d6fda64d9d7743a8c109b67e3135117d8111`.
Local main and origin/main were equal and clean before this follow-up. Both
post-merge workflows passed on attempt 1: main `42404134767` (16 jobs), base
image `42404134783` (13 jobs). The final local suite passed 655 Rust tests
(5 hosted cases selected separately in CI); CI passed 427 engine tests, all six
native packages, both image architectures and combined manifests. Documentation
retained 46 original routes, 282 heading IDs, 82 compatibility mappings and 29
byte-identical images without captured browser errors. Native rc.1 remains the
latest published candidate.

Source **`4cbbc7a6d10f4c97aae200dc43dd3122bf8c25f8`** adds Sphinx
`literalinclude` for local UTF-8 code/text files. The parser records explicit
include nodes; after navigation selects published documents, a bounded resolver
reads contained regular files and replaces those nodes with literal code blocks,
including nested lists/admonitions/table cells. No Python, included RST, remote
file or source code is executed. Inclusion alone does not publish a raw source
asset. File paths are document-relative or source-root-relative, with literal
Unicode/space/percent/hash filenames; hidden, excluded, symlinked and escaping
paths fail. Unpublished pages do not open their include targets.

Language/encoding, tab stops, start/end markers, ordered one-based line ranges
and numeric/common-prefix dedent are supported. Filters run in Sphinx order;
missing markers, out-of-range lines and non-whitespace dedents fail rather than
silently losing content. UTF-8 BOM handling follows the chosen encoding; line
endings normalize to LF. Bounds are 8 MiB/file, 32 MiB unique input and expanded
text, and 100,000 explicitly selected lines. Unsupported options (line numbers,
captions, pyobject, diff, prepend/append, etc.) fail before writes. Parsed includes,
MyST, extensions and untitled RST fragments remain unsupported. Sphinx's
[directive documentation](https://www.sphinx-doc.org/en/master/usage/restructuredtext/directives.html#directive-literalinclude)
and upstream directive source were inspected read-only to check filter semantics.

All 37 Sphinx CLI tests pass on Unix (35 on Windows), including nine new tests
covering options, selection, nesting, resource bounds, read containment, literal
content, source immutability and no-output failures in normal/dry-run/assets-skip
modes. Node-only real-engine acceptance passes all six importer fixtures; the
expanded Sphinx fixture retains nine routes, five literal blocks, selected code
text, ordinary prose search, heading anchors and existing assets. The first Node
run exposed an incorrect test assumption that fenced code is indexed; the test
now checks prose, consistent with the engine's documented search policy. No
search behavior changed. CI additionally checks exported HTML, hydration,
nested DOM structure and reload without executing the included examples.

Warnings-denied workspace Clippy, formatting, Node syntax, actionlint and diff
checks pass. Strict/deep docs retain 55 routes and 23 examples. The full Rust
workspace run and final-head CI/merge remain pending. No local Docker/browser/GUI,
native release, Netlify production or live provider mutation ran. User docs,
CLI contract/specification/status and this context describe the supported subset.
Evidence: `/private/tmp/zellij-literalinclude-{focused,engine-final,clippy,docs,workspace}.log`.


### Exact filename inventory follow-up

The first complete local Rust run passed 664 tests (5 hosted cases selected
separately in CI). Review then identified a case-insensitive-filesystem exclusion
bypass in the initial literalinclude resolver: a differently capitalized target
could resolve an excluded file. Fix **`e4be0eaca31f4fb07186120337279740b0d8ad5f`**
requires an exact match in the already-filtered source file inventory, before
contained-path and bounded-file reads. Regression cases cover both excluded-file
case aliases and differently capitalized eligible files. The user reference now
requires exact filename case on every OS. All 37 Sphinx CLI tests pass after the
fix. Final workspace/engine checks and final-head GitHub CI remain pending.


### Final local verification and Windows fixture correction

The complete workspace rerun after the inventory fix passed **664 Rust tests**
(5 hosted cases selected separately in CI). Final Clippy and Node conversion
checks also passed; strict/deep docs retain 55 routes and 23 examples. Fixture-only
commit `766c889` avoids the reserved Windows NUL filename for the control-byte
case. Run `42412362640` then exposed a Windows assertion expecting raw CRLF
checkout bytes where conversion intentionally produces LF. Test correction
**`317e5072200c3102bc001a593e0f4e0fc99b3993`** explicitly writes CRLF on every OS,
checks normalized output, and checks that source bytes remain untouched. Node
acceptance normalizes the expected fixture similarly. All 37 Sphinx tests and
all six importer engine fixtures pass after this correction. No product behavior
was changed for this test fix. Final-head CI and merge remain pending.
Evidence: `/private/tmp/zellij-literalinclude-{workspace-final,clippy-final,docs-final,engine-inventory,portable-fixture,crlf,engine-crlf}.log`.


## 2026-11-07 — Literal inclusion closure and search code boundaries

PR #100 merged as **`9fb29c8f5a3abe0804d151cef6d6be710ffa1bc3`** after all
16 checks passed on `77de5e41ae166974f49677d4eb681b645ab5062d` in
[42412853970](https://github.com/mylife-inc/Zellij/actions/runs/42412853970),
attempt 1. Tested/merged tree: `a0e48d25c9c9e90c4074fd463f45afea78e1dfb8`.
Main and origin/main matched and were clean before this follow-up. Both automatic
post-merge workflows passed on attempt 1: main `42415654053` (16 jobs) and base
image `42415653990` (13 jobs). Final-head acceptance included 664 local Rust tests
(5 hosted cases selected separately in CI), 427 engine tests, all six native
packages, both image architectures and combined manifests. Sphinx acceptance
passed 37 CLI tests on Unix and 35 on Windows; real exported code, nested DOM,
hydration and reload passed. Documentation retained 46 original routes, 282
heading IDs, 82 compatibility mappings and 29 byte-identical images without
captured browser errors. Remote state/releases were refreshed; rc.1 remains the
latest native release. No native tag/publication or production deployment ran.

Source **`f9676f5261781b1fe854fd93aa0185cb7a50d825`** fixes a confirmed MDX search bug: fixed-three-character
regular expressions could close a longer code fence too early and index example
text. A four-backtick or four-tilde block containing a three-character fence and
`fenced_only_sentinel` previously leaked that token into prose search. The initial
regression run reproduced 18 failures before the implementation changed.

The existing shared MDX/heading parser now records original Markdown code-block
and inline-code ranges in one full-document parse. Search masks those ranges
without changing UTF-16 offsets or CR/LF characters, then slices passages using
the same rendered heading IDs. This handles longer/mixed/unfinished fences,
quotes/lists/components, inline backtick runs, literal escaped/unmatched markers,
JSX attributes and optional math parsing without evaluating authored JavaScript.
Heading labels retain their code text. A section inside a component is never
reparsed as an incomplete MDX fragment. Standalone `mdxToText` accepts complete
MDX with optional parser plugins and reports malformed syntax; portable document
payloads continue through their existing separate indexing path.

The packaged CRLF fixture also exposed a pre-existing guide-loader bug: the
leading H1 was not removed under CRLF, duplicating the title and shifting repeated
heading anchors. The same source commit handles LF, CRLF and CR when removing the
leading title, retaining later headings and literal code unchanged. Loader cases
and the actual packaged preparation fixture verify stable canonical anchors.

All **453 engine tests** pass (26 more than the baseline), including the 40 search
cases, shared rendering/heading coverage and three line-ending loader cases.
All 37 Sphinx CLI tests still pass with the extended fixture. All six Node-only
importer/rendering fixtures pass; packaged preparation passes with actual bundled
CLI artifacts, CRLF input and optional math. Engine build verification, workspace
typechecking, strict/deep docs (55 routes, 23 examples), Node syntax, actionlint
and diff checks pass. CI additionally reads exported search JSON and exercises
the real search dialog: prose finds the correct heading, while a code-only token
returns no results. No local container/browser/GUI was used. User guide, engine
specification and durable context/status are updated. Final-head CI/merge remain
pending; broad import/provider/release gaps remain as previously documented.
Evidence: `/private/tmp/zellij-search-boundaries-{before,focused,engine-final,typecheck-all,build-final,sphinx,conversion,packaged-final,docs-final}.log`.


### 2026-11-08 — Local Sphinx CSV table files

`csv-table :file:` now imports local UTF-8 data through the same bounded reader
as `literalinclude`, after navigation chooses published documents. Headers,
stub columns, captions, nested cell markup, substitutions, links, images and
searchable text survive. CSV source files are not copied unless explicitly
linked; copy/skip/dry-run share the same input checks. UTF-8/BOM policy, CRLF/CR
normalization and tab delimiters are documented in the conversion reference.
Exact-case inventory lookup, containment, exclusions and symlink rejection apply.
Files are limited to 8 MiB; both resource types share 32 MiB unique-input and
expanded-text budgets, plus existing table dimensions and a 16-level nesting cap.
No source Python/Docutils, remote CSV URL or authored code executes.

Docutils assigns CSV cell source paths to the data file, while supplemental
`:header:` rows retain the directive source. Nested CSV files follow those paths;
Sphinx roles/images/downloads/literalinclude retain the original RST context.
A regression also exposed loss of indentation inside multiline CSV header
options; continuation indentation is now relative to the first-line value.
References: [Docutils table implementation](https://github.com/docutils/docutils/blob/master/docutils/docutils/parsers/rst/directives/tables.py),
[Sphinx CSV patch](https://github.com/sphinx-doc/sphinx/blob/master/sphinx/directives/patches.py),
[asset collectors](https://github.com/sphinx-doc/sphinx/blob/master/sphinx/environment/collectors/asset.py),
and [literal includes](https://github.com/sphinx-doc/sphinx/blob/master/sphinx/directives/code.py).

The initial positive regression failed against the prior implementation. Local
46 Sphinx CLI tests and 453 engine tests pass, along with all six Node import/render
fixtures and strict/deep docs (55 routes, 23 examples). The first broader Rust
run found an obsolete test expecting tab delimiters to fail; that expectation
was removed after actual tab-delimited import passed. The final workspace run,
final-head hosted/native CI and merge remain pending at this entry. No local
Docker/browser/GUI, native release, live provider deployment or production
migration ran. Remote CSV and broader Sphinx/domain/include fidelity remain out
of scope, as do the pending live-provider/release tasks in the CLI status.

Previous milestone closure: PR #101 merged as `cac41b71c9cfbc6767b5f718a0a163beb3b36d35`,
with the identical tested tree `f44ae2e59657a33789f9e06def3c0ccc85106e7f`.
All 16 checks passed on `7ce9419ce0aab665f32d20d6aa3e2a25bbe0c21c` in
run `42418585801`. Both post-merge workflows passed: CI `42421313719` and base
image `42421313696`. Native candidate rc.1 remains the newest published release.


Local CSV implementation **`a7b32b017525626c8fc5c4ce26a037e6b2df57bf`**
passes the final local workspace run: **673 Rust tests**, 0 failures, with 5
hosted-export tests intentionally ignored locally and selected separately in CI.
All 46 Sphinx CLI cases, 453 engine tests, warnings-denied workspace Clippy,
formatting, six actual-CLI Node/rendering fixtures, strict/deep docs (55 routes /
23 examples), script syntax and diff checks pass. Evidence is under
`/private/tmp/zellij-local-csv-{workspace-final,focused-final,engine,clippy-final,render-final,docs-final}.log`.
The multiline-header regression initially failed and passes after preserving
relative option indentation. Final-head cross-platform/native/hosted CI and merge
remain pending; these local results do not establish a bug-free or complete CLI.


### 2026-11-08 — Public asset URL linking

The user selected existing public HTTP(S) URLs for `--assets link`, with an
explicit `--asset-base-url`; copying remains the default. Current source adds
this policy to convert/cv, migrate/mig and ship through the shared converter.
Eligible local asset paths become base-plus-root-relative URLs; copy-only
namespace prefixes are omitted. Queries/fragments and encoded filenames survive;
document links retain Zellij routing. No asset bytes are read/copied in link
mode, no symlinks are created and no remote URLs are fetched. Distinct local
files mapping to one public URL fail before writes. Missing/excluded resources
retain the usual reports and containment checks. CSV/include content inputs
remain readable independently of asset policy. Absolute URLs persist in content
for subsequent build/publish commands; the public server remains required.

The URL base must be HTTP(S), with a host and no credentials/query/fragment,
whitespace, backslashes, malformed escapes or dot segments. It is required only
for link mode; other policy/base combinations fail. Reports list linked files
and public URLs, including in dry runs. The user reference documents source
roots and the lack of source-tool hashed/CDN layout emulation. Native release
rc.1 remains unchanged. Full validation and final-head CI/merge are pending.

PR #102 closure: all 16 checks passed on `6d0b5c321d831606e383c14c127e20f579bdaca3`
in run `42530352694`, attempt 1; merge `245eacd2bfde84492d3a337020d0a75e1784aeb0`
has identical tree `56cf10ab6ce261ad070ccdc2f677e8e41abf20f1`. Both post-merge
workflows passed: CI `42532456398` and Base image `42532456352`.


Public asset linking implementation **`1b2e2e4fca5d6d46ea24708276e6636d9193a1a0`**
passes all **681 Rust workspace tests** (5 real hosted-export cases intentionally
selected only in CI). All 25 import/delivery integration tests pass, including
all six importers, public/static asset roots, URL encoding/suffixes, invalid
options, collisions, missing targets, symlinks and no asset-byte reads. The
initial end-to-end test failed against the old CLI. One later test fixture
needed a Markdown paragraph break before its reference definition; product
behavior was unchanged by that fixture correction.

Warnings-denied workspace Clippy, formatting, strict/deep docs (55 routes / 23
examples), Node script syntax and diff checks pass. All six real CLI-to-engine
Node fixtures verify copy and link policies with unchanged routes, remote URL
rendering and no local asset copies. CI adds an actual linked static export and
Next image browser/download acceptance through a separate loopback asset server.
No local browser/container/GUI, live provider mutation or release publication
ran. Evidence: `/private/tmp/zellij-public-assets-{before,focused-final,workspace,clippy,render,docs}.log`.
Final-head GitHub CI and merge remain pending.


### 2026-11-09 — rc.2 installed-CLI delivery acceptance closure

PR #105 merged as `5677577c75a4307cec5d4b0c7cd949f99aeb618a`, retaining tested
tree `58d0ed02fd77c5b94ad3960669a9ce0630c179d1` from head
`9fb6bbd32602856c0b75050335c53b912f2eea2c`. All 16 main PR checks passed in
`42770299276`, and all six generated-workflow PR verification runs passed.
The saved workflows now install published rc.2, use its hosted engine image,
and publish only to the dedicated new rc.2 acceptance image tags or approved
Netlify staging drafts. Active installation and command documentation was
updated with the release.

All six manual workflows passed on the merged source: portable `42773738576`,
hosted `42773741515`, static image `42773744556`, Next image `42773747457`,
Netlify portable draft `42773751667` and Netlify hosted draft `42773754795`.
Both public draft URLs passed HTTP page, linked-asset and source-file isolation
checks. Fresh runners pulled both newly published site images and passed HTTP
acceptance in `42774184245`; the static and Next immutable image digests,
draft URLs and job/source identities are retained in
`specs/evidence/zellij-rc2-delivery-acceptance-2026-11-09.json`.

Post-merge Base image `42773711571` passed; main CI `42773711512` was still
running when this entry was prepared. The complete final-head PR suite and
all manual acceptance runs above passed on the same source tree. This entry
changes audit/status evidence only, without modifying that tested runtime,
workflows, fixtures, authored site documentation or immutable release assets.

Live acceptance for the other provider accounts and actual GitLab/CircleCI
runners still needs suitable destinations. Advanced importer limits remain
in the CLI status. The Netlify hosting-comment hydration issue remains open:
HTTP fixture acceptance is not a browser hydration test or production migration.
No local Docker/browser/GUI or production Netlify/domain change ran.


### 2026-11-09 — Jekyll TOML configuration imports

Source `c9f1fbf7e67b6f87908bfd0db879bd61a2fb94e0` adds `_config.toml` to the Jekyll conversion
path used by `convert`/`cv`, `migrate`/`mig` and `ship`. Exactly one YAML or TOML
configuration is accepted; ambiguous files still require explicit consolidation
in a source copy. TOML keys remain case-sensitive, tables and arrays retain
structure, booleans/numbers retain their types, and date/time values become
text rather than Serde's private datetime map. The existing settings/defaults,
collection ordering, visibility, navigation, URL and asset validation applies.
Configuration parsing is data-only, bounded to 64 levels, and does not execute
Ruby, Liquid, plugins or source commands. Page front matter remains YAML; blog
posts, dated collections and config merging remain outside the supported subset.

The regression first failed against the previous TOML rejection. All 15 Jekyll
integration tests now pass, covering metadata, routes, URLs/assets, nested
defaults, ordering, case/type preservation, BOM/CRLF, ambiguity, invalid syntax
and settings, symlinks, unchanged sources and dry runs. All **684 Rust workspace
tests** pass; 5 real hosted cases are intentionally selected in CI only. The
final focused parser run also checks the documented nesting boundary.
Warnings-denied workspace Clippy, formatting, strict/deep documentation
(55 routes / 23 examples), Node syntax and diff checks pass.

The actual CLI-to-engine fixture now covers YAML and TOML Jekyll sites, alongside
the five other importers, under copy and public-link asset policies. All seven
Node rendering fixtures pass. CI will also export both Jekyll variants through
the real engine image. Conversion reference and quickstart document TOML with
an example and explicitly mark it as source-only after rc.2. Final-head PR CI
and merge remain pending for this change. No local Docker/browser/GUI, provider
deployment or new native publication ran; published rc.2 remains immutable.

PR #105's previously pending post-merge checks are now closed: all 16 main CI
jobs in `42773711512` passed, as did Base image `42773711571`, both on
`5677577c75a4307cec5d4b0c7cd949f99aeb618a`. The rc.2 delivery evidence now appends
that closure while preserving the original in-progress snapshot. PR #106 merged
the prior audit-only record as `1650e7d9318b311c72f857d216ca1815493108b7`.


### 2026-11-10 — Jekyll TOML acceptance closure

[PR #107](https://github.com/mylife-inc/Zellij/pull/107) merged as
`55639024c75cc8e50e1b6a923f9f023a5c5edfb5` after all 16 checks passed on
`db3aa01afb03a4e523cd6012cc6d6df811e30fec` in run `42778216082`,
attempt 1. The merge retains tested tree
`443cb5a2003d2aca3ddd9adfab6896746e984b63`. This includes all six native package
checks, native Windows/macOS/Linux regressions, both Linux image architectures
and real-engine export of both YAML and TOML Jekyll fixtures. Source implementation
is `c9f1fbf7e67b6f87908bfd0db879bd61a2fb94e0`; local 684 Rust tests, Clippy,
formatting, seven conversion/rendering fixtures and strict documentation checks
passed. The new configuration format is merged source, newer than published rc.2.

Automatic post-merge CI `42782791785` and Base image `42782792095` are not
yet complete at this refresh. They remain separate from the successful PR
checks; consult their current results before claiming they passed.
This closure edits audit/status documents only. Published rc.2 assets and tags
are unchanged. Other provider/runner acceptance, advanced importer limitations
and the reported Netlify hosting-comment hydration issue remain open.


### 2026-11-10 — Jekyll collection metadata sorting

Source `a3c3663ceae66daf7a20f3d01c50def1390e5a66` adds collection `sort_by` to
Jekyll imports in the shared convert/migrate/ship path. Derived navigation uses
effective front matter after scoped defaults, sorting comparable numeric or
text values ascending, with missing/null values last and source paths as ties.
64-bit integers retain exact ordering against integers and finite floats.
Literal, case-sensitive keys are not interpreted as nested paths. Generated
slug/extension and ASCII-title fallbacks follow source basenames; non-ASCII
generated titles need explicit metadata. Mixed families, nonfinite values,
structured/boolean values, generated date/category/tag/excerpt fields and
date/time-shaped values have explicit pre-write diagnostics and a numeric-rank
or literal-order workaround. This remains a data-only subset of Jekyll sorting.

The same change fixes explicit collection order lists: an array, even empty,
takes precedence over sort_by. Missing or unpublished entries are skipped and
reported rather than aborting conversion; unlisted pages retain filename order.
Explicit navigation data remains authoritative. Source files are unchanged,
and dry-run/error cases do not create output. User documentation covers these
rules and marks the new capabilities as source-only after published rc.2.

The new regression failed against the old sort_by rejection. All **691 Rust
workspace tests** pass (5 real hosted cases remain intentionally selected only
in CI), including **21 Jekyll integration tests** and exact numeric boundary
checks. Workspace warnings-denied Clippy, formatting, Node syntax, diff checks
and strict/deep documentation validation (55 routes / 23 examples) pass.
All **eight CLI-to-engine fixtures** pass locally: the existing seven plus a
TOML collection fixture asserting inherited ranks, ties, missing values and
engine sidebar order. CI also exercises that fixture through real hosted export.
Final-head CI and merge are pending for this milestone.

PR #107's previously pending post-merge workflows are now verified successful:
all 16 main CI jobs in `42782791785` and all 13 Base image jobs in `42782792095`
passed. PR #108 merged its audit closure as
`ea097410403b57c65b774d46e4d81ec2d0271c0b`. These earlier results do not stand
in for this milestone's final-head checks. No local Docker/browser/GUI,
provider deployment, production-domain change or new release publication ran.
The published `v0.2.0-rc.2` tag/assets remain immutable at `54e636f`.
Live provider/runner acceptance, advanced importer limitations and the Netlify
hosting-comment hydration issue remain open; passing tests do not establish
zero bugs or completion of the entire CLI design.


### 2026-11-10 — Jekyll collection sorting acceptance closure

[PR #109](https://github.com/mylife-inc/Zellij/pull/109) merged as
`47ceb61bbb77c19faf24b0ab08ac6f9069248050` after all 16 checks passed on
`37a5bc4c4c5f9f3f69beb0b0f1053dc79df207e1` in run `42795255252`,
attempt 1. The merge retains tested tree
`78065d94d21333b3e363df05f3b5bd33596fff2e`. Source implementation is
`a3c3663ceae66daf7a20f3d01c50def1390e5a66`. Native Windows/macOS/Linux regressions,
all six native package checks, both Linux image architectures, browser checks
and actual engine exports passed. The new collection fixture also verifies
inherited numeric ranks, missing values and tie ordering in exported sidebar
links. Local 691 Rust tests, warnings-denied Clippy, formatting, eight conversion
fixtures and strict documentation checks passed.

Automatic post-merge CI `42799993837` and Base image `42799993455` are not
yet complete at this refresh. They remain separate from the successful PR
checks; consult their current status before claiming they passed. This closure
updates only audit/status documents. Collection sorting, stale-order handling
and the preceding TOML work are merged source newer than the immutable rc.2
release. Provider/runner live acceptance, advanced importer limitations and the
Netlify hosting-comment hydration issue remain open. No local Docker/browser,
production-domain change or new release publication ran.


### 2026-11-10 — live provider acceptance and Cloudflare corrections

Source `21fb99c` fixes two problems found using a disposable Cloudflare Pages
project. Successful asset acknowledgement responses can omit `result`; only
acknowledgement endpoints now accept that form, while data payloads and the
success flag remain required. Portable exports without a top-level `404.html`
were served with Cloudflare's SPA fallback, returning home HTML with status 200
for nonexistent paths. The deployment's private staging directory now receives
a fallback 404 page if needed, preserving existing regular 404 files and
rejecting links/non-files. These fixes are newer than immutable rc.2.

Both regressions failed before their fixes. Focused tests pass (16 Cloudflare
cases; the real hosted case is selected in CI). Current-source portable
Cloudflare and GitHub Pages live checks pass for nested pages, exact authored
logo bytes, linked assets, source-file isolation and missing-page status.
Hosted and Railway live tests are still in progress. The manual acceptance
workflow now supports explicitly identified disposable provider destinations,
retains evidence and deletes only exclusively owned test image versions.

The user authorized test deployments and removal afterward. Cloudflare uses
existing authorized Ziryab credentials without changing Ziryab. GitHub Pages
uses a separate disposable repository; the initial temporary Pages setup on
mylife-inc/Zellij was removed and absence verified. Railway has an isolated
project/service, but creating its project token returned Not Authorized; a
user-created environment token is needed for `zel rw`. Track cleanup and exact
live results before claiming this milestone complete. No production domains
or existing customer projects were modified.

PR #110's main `4b782c6` post-merge workflows both passed: CI `42800161133` and
Base image `42800161483`. Earlier runs `42799993837` and `42799993455` were
superseded/cancelled, not successful. Full validation, final-head CI and merge
for the new Cloudflare corrections remain pending at this entry.


### 2026-11-10 — Cloudflare, GitHub Pages and Railway live acceptance closure

[PR #111](https://github.com/mylife-inc/Zellij/pull/111) merged as
`d1d03d4c75023689946318c08b4ecbcdce0c0a2a` after all 16 final-head checks passed on
`98b5d0cd94c931fce30d6efffb3fe2be90c7baab` in run `42898053745`.
Source `21fb99c` fixes Cloudflare success-only asset acknowledgements and adds
an upload-only fallback 404 page. Source `2c3f794` replaces Railway's nullable
change-count check with explicit empty staged-patch verification, without
requesting decrypted patch variables. Source `a5ee146` allows all instances to
be stopped during startup and requires running instances at SUCCESS. Scope,
malformed-state, staged-change, configuration-drift and failure checks remain.
Both Railway regressions were reproduced against the previous implementation.
Source `98b5d0c` updates the hosted-prefix fixture to allow only the required
root 404 outside the mount and verifies preservation of its prefixed 404; the
previous assertion failed in CI `42895754326`. Runtime behavior is unchanged
from the live-tested source.

All **six live cases passed**: Cloudflare and GitHub Pages portable/hosted sites
in `42891873828` (source `21fb99c`), plus Railway static/Next images in
`42895763990` (source `a5ee146`). Checks fetched home/nested pages, exact authored
logo bytes and linked assets, and required 404 responses for source and missing
paths. Railway reported SUCCESS for the exact newly returned deployment IDs;
HTTP checks then verified their public routes. These are current-source fixture
checks, not a retest of the unchanged rc.2 binary or live browser hydration.
Normal CI separately passed the browser and cross-platform acceptance suites.
Local final-head verification: **695 Rust tests**, 5 hosted-only ignored cases,
warnings-denied Clippy, formatting, seven Node delivery/cleanup checks, workflow
lint and strict/deep documentation validation (55 routes / 23 examples).

Cloudflare's project, GitHub Pages' publishing branch/site and the entire
Railway project were removed. Public Cloudflare test hostnames no longer
resolve; GitHub and Railway test URLs return 404. All three temporary Actions
secrets were removed. Registry cleanup `42896298737` deleted the two exclusively
tagged Railway test image versions after project deletion. An earlier failing
harness had deleted an image while its deployment could still be starting;
cleanup now runs as a separate explicit operation after service removal, and
shared/release versions are protected. The empty disposable GitHub repository
`mylife-inc/zellij-live-20261110-684141` remains: deleting it returned HTTP 403
because the current OAuth token lacks delete_repo. Its website is removed.
The original mylife-inc/Zellij Pages setup was also removed and absence verified.

See [the evidence](evidence/zellij-live-provider-acceptance-2026-11-10.json)
for source/run IDs, deployment receipts, immutable image digests and cleanup.
The user supplied the Railway project token after API token creation returned
Not Authorized; this credential blocker is resolved. The disposable project
and its temporary GitHub secret are now deleted. Existing Ziryab projects, Netlify production/staging sites and
custom domains were unchanged. Published rc.2 remains immutable; these four
fixes require current source or a later release. Fly, Vercel, S3 and real
GitLab/CircleCI runner acceptance, advanced importer limitations and Netlify's
reported hosting-comment hydration issue remain open. Post-merge CI
`42903488658` and Base image `42903488649` are still running on `d1d03d4`
at this audit refresh and must be checked separately; this record does not claim zero bugs or a complete
implementation of every planned CLI capability.


### 2026-11-10 — live-provider cleanup and post-merge verification completed

The user deleted the empty disposable repository
`mylife-inc/zellij-live-20261110-684141`; a fresh GitHub repository API lookup
returns HTTP 404. This closes the prior delete_repo permission blocker. The
earlier HTTP 403 remains recorded in the evidence as the initial deletion
attempt. All temporary provider destinations, Actions secrets and exclusively
tagged test image versions from this acceptance session are now removed.

Both post-merge workflows for PR #111 merge
`d1d03d4c75023689946318c08b4ecbcdce0c0a2a` passed: all 16 CI jobs in
`42903488658` and all 13 Base image jobs in `42903488649`. These results close
the pending checks in the preceding entry. PR #112 merged the initial live
acceptance documentation as `a5daa5f4af44f063d82075e492369a5b37f1d446`.
This update changes only context/status/evidence; published rc.2 is unchanged.
Fly, Vercel, S3 and real GitLab/CircleCI runner acceptance, documented importer
limitations and the reported Netlify hosting-comment hydration issue remain open.


### 2026-11-10 — local Sphinx document includes

Source `3a19947a8b6dfb5a236e813468aca1feba04ee1e` adds parsed local RST `include` fragments through
convert/cv, migrate and shared ship conversion. Included headings, explicit
labels, links, images and substitutions join the owning document; supported
toctrees and nested lists/admonitions/tables retain their existing behavior.
Navigation-driven parsing permits untitled include-only `.rst` files without
publishing them as pages. Unlisted source documents are reported but no longer
parsed or used to read dependencies. Referenced fragments are not misreported
as unconverted orphan pages.

Includes use the same bounded resource reader as local CSV/literalinclude: exact
case, containment, exclusions, hidden/internal-tree and symlink checks; UTF-8
text only; 8 MiB per file and shared 32 MiB input/expanded budgets. Repeated
fragments are allowed, while cyclic paths, chains over 32 active source files,
more than 10,000 expansions and duplicate headings/labels fail before writes.
Options support UTF-8/BOM policy, Python-style zero-based line slices, nonempty
literal start-after/end-before markers and bounded tab expansion. Parser/code/
literal/presentation options, standard include libraries and explicitly excluded
fragments remain unsupported; code excerpts use literalinclude.

The official Sphinx Include/relfn2path implementations and Docutils include
reference were inspected. Sphinx include paths, document roles and assets stay
relative to the owning page, including nested fragments; CSV file directives
retain the actual containing fragment/cell source. Heading state is shared
across includes, while nested block limits remain enforced. Configuration
version/release substitutions now fill gaps after all declarations are parsed:
a regression reproduced the previous default/CSV-included-declaration collision.
No Python, Docutils, source extensions or remote source fetches are executed.

Local validation: all 53 Sphinx CLI tests pass, including seven new include
regressions; warnings-denied workspace Clippy, formatting, actual CLI-to-engine
rendering fixtures and strict/deep docs (55 routes / 23 examples) pass. The
initial broader workspace run passed 702 tests (5 hosted-only cases ignored);
the exact final source is being rechecked after the substitution-order fix.
GitHub final-head hosted/browser/native verification and merge remain pending.
Browser coverage adds included-heading hydration, images, navigation and search;
it runs only in CI, with no local Docker/browser/GUI. Published rc.2 is unchanged.

References: [Sphinx include](https://github.com/sphinx-doc/sphinx/blob/master/sphinx/directives/other.py),
[document-relative paths](https://github.com/sphinx-doc/sphinx/blob/master/sphinx/environment/__init__.py),
[Docutils include](https://docutils.sourceforge.io/docs/ref/rst/directives.html#including-an-external-document-fragment).


### 2026-11-10 — Sphinx include acceptance closure

[PR #114](https://github.com/mylife-inc/Zellij/pull/114) merged as
`ee866afd634cea5107941520c231bdc3fa6b270d` after all 16 checks passed on
`8f360bb13f6428680925c05e76441fddc81616a4` in run `42925938421`.
The merge retains tested tree `d5005a3ee4c22218ac5814e398667134bba85364`.
Source `3a19947` implements parsed document fragments and resolves the
configuration-default/substitution collision. Browser assertion follow-up
`8f360bb` uses the accessible heading name and verifies its ID independently
of the decorative anchor marker; superseded run `42925681071` was cancelled.

Final-source local validation passed all **702 Rust tests** (5 hosted-only
cases reserved for CI), including 53 Sphinx CLI cases, plus warnings-denied
Clippy, formatting, actual CLI-to-engine rendering and strict/deep docs
(55 routes / 23 examples). Final-head CI passed the six native package checks,
Linux/macOS/Windows regressions, both image architectures and real hosted
exports/browser tests. Included heading IDs, images, document navigation and
search passed in the generated site under its deployment prefix.

Automatic post-merge CI and Base image workflows are still running at this
refresh; their exact IDs/statuses are in the
[evidence](evidence/zellij-sphinx-includes-2026-11-10.json). They are separate from the successful PR checks.
Parsed includes require current source or a later release; published rc.2
remains unchanged. Existing include limits, pending live Fly/Vercel/S3 and
GitLab/CircleCI runner acceptance, and the reported Netlify hosting-comment
hydration issue remain open. No local Docker/browser/GUI or live provider
deployment ran for this milestone.
