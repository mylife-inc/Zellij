# Zellij — implementation audit and completion backlog

**Date:** 2026-10-16 (America/Los_Angeles)

**Audited code:** local `main`, `6356b05deaddf5421b24d7c8a1e30f868d4923c0`
**Scope:** study source/documentation, establish reusable context, identify
unfinished work and defects. **The initial audit made no implementation fixes or remote changes.**

Start with [ZELLIJ_CONTEXT.md](../ZELLIJ_CONTEXT.md) for the architecture and
source map. This report records what was observed at this commit; future fixes
should be recorded with their commit and verification rather than rewriting
these historical observations as if they never occurred.

## Implementation record — first safety and CI milestone

The user subsequently authorized implementation and repository setup.
[PR #1](https://github.com/mylife-inc/Zellij/pull/1) carries the three previously
unpublished main commits plus the following changes. The findings below remain
the historical evidence at `6356b05`; this table records their disposition.

| Findings | Resolution and evidence |
| --- | --- |
| R1, R2: conversion containment and overwrite | `1a2e26b`: validate source and output paths, reject traversal/symlinks/overlap, require new/empty output, and preflight the complete write plan. CLI integration tests cover both converters and existing authored files. |
| R3: page/guide overwrites | `1a2e26b`: add files without overwriting existing destinations; preflight catches late collisions before earlier sections are written. |
| S1: destructive static output | `1a2e26b`: stage beside output and replace only empty or owned exports; failures retain the previous export. Unit tests cover overlap, unowned output, missing output mounts, rebuilds, and destination changes. CI adds real-container initial/failing/replacement builds. |
| S7: authored `.zel-dockerfile` deletion | `1a2e26b`: stream Dockerfile on stdin; fake-runtime regression verifies both success and failure preserve authored content. |
| R14: Clippy | `1a2e26b`: resolve the warnings-denied gate; local and hosted checks pass. |
| Clean browser-job setup | `deda7a0`: build the engine helpers in the browser job's own checkout before the showcase build. Hosted run `39000366953` passes. |
| Large-import file limits | `bd6b6c6`: close staged file descriptors before publishing; a 96-page conversion succeeds under a 64-descriptor limit. |
| Current CLI/engine compatibility | `bd6b6c6`: build the CI engine image from the current checkout, including the previously unpublished React-range fix in `5001c37`. The old published image failed installation by selecting React 19.3 against the engine's 19.2 peer range. |

Verification: 99 Rust tests (17 added), warnings-denied Clippy, formatting,
227 engine unit tests, all workspace typechecks, engine package build, and
showcase production build passed locally. Hosted browser run `39000366953`
passed 294 tests, with 114 pre-existing conditional skips for device/browser
cases. Check the PR's latest CI results for its real-container test and final
merge status; these counts do not imply all product acceptance is complete.

Repository setup: `mylife-inc/Zellij` is now public and defaults to `main`.
Gitleaks 8.30.1 scanned all 134 reachable commits before publication and reported
no detected leaks. The user's local AI settings were excluded. No history was
rewritten. Native binaries have not been released by this milestone.

The new static-export marker is an explicit ownership contract: every file
inside an owned output is replaceable. Old unmarked exports need a new/empty
output directory for their first build; never add the marker to authored data.
Write preflight protects against known path/collision failures; it is not a
claim of a filesystem-wide transaction under arbitrary concurrent modification
or storage failure.

Additional follow-up: locked dependency audit found 19 advisories (3 critical,
6 high, 6 moderate, 4 low), including Next.js and Vitest. Remediate these with
compatibility tests in a separate change. R4–R13 and S2–S6/S8–S10 remain open,
as do the requested header work and native Windows support. See
[project context](../ZELLIJ_CONTEXT.md) for the current next-work order.

## Implementation record — dependency and section-template milestone

The paragraph above records the first milestone's handoff. Its dependency
and R4 follow-ups are implemented in `17d5a23`, tracked in
[PR #2](https://github.com/mylife-inc/Zellij/pull/2).

- **Dependency advisories:** the locked repository tree and a fresh standalone
  wrapper with all features now audit with zero reported advisories. Next.js
  16.4.2, Vitest 4.1.12, Mermaid 11.17.2 and patched transitive dependencies
  replace the affected versions. Production math/diagram overrides propagate
  into generated app roots, because npm ignores dependency-owned overrides.
- **Front matter parsing:** remove gray-matter and its obsolete YAML chain.
  The shared YAML parser rejects executable language tags and malformed
  metadata; tests cover prose preservation, nested/multiline data, BOM/CRLF,
  missing delimiters, and unsupported languages.
- **R4:** the engine generates each section's ready-to-edit YAML and validates
  it with the full Zod schema before embedding it in Rust. Arrays respect
  minima, objects preserve their shape, matrix dimensions agree, and media
  alternatives satisfy refinements. The CLI no longer guesses from flattened
  field descriptions. Authored media are still explicit placeholders to replace;
  CI supplies real image/video fixtures when exporting the generated content.

Verification at this implementation: 260 engine tests, 99 Rust tests,
warnings-denied Clippy, formatting, workspace typechecks, engine packaging,
all four example content checks, all 23 actual Rust-emitted template checks,
and a showcase webpack production build pass locally. CI adds complete
23-template static export and math/diagram browser regression checks. The
normal Turbopack build cannot run inside this local sandbox's helper-process
port restriction; see PR #2's latest hosted checks for that build, export and
browser results before treating the change as release-ready.

Hosted run `39004471983` at `d8c9f5d` subsequently passed every job, including
the normal Turbopack production build, initial/failing/replacement exports,
and static export of all 23 generated templates. The browser suite passed
**297 tests**, with **114 conditional skips and no retries**.

PR #1 merged as `6950589`. All four base-image feature variants were published
for AMD64 and ARM64 by run `39001111783`; no new native binaries were released.
Post-merge browser run `39001111736` had mobile-WebKit navigation timeouts on
its first attempt and passed 294 tests with 114 conditional skips on its retry.
The cause remains unproven. PR #2 retains failure artifacts to aid diagnosis.
The automated security-review plugin's commit review failed with API HTTP 400,
so there is no successful result from that separate review service.

R5–R13 and S2–S6/S8–S10, the requested header changes, and native Windows
support remain work for later milestones. Historical findings below retain
their original wording and reproductions.

## Implementation record — conversion links and assets

Commit `4196e6b` ([PR #3](https://github.com/mylife-inc/Zellij/pull/3)) fixes
R5's reproduced Markdown link/asset failure. Both converters map documents to
the engine's canonical routes and copy referenced binary files. Reference-style
links, directory indexes, nested/encoded paths, query and fragment text, and
Docusaurus default static assets are supported. Markdown asset images apply
the site's basePath. Unsafe or conflicting plans fail before output; missing,
ambiguous and unsupported references are reported, and unlisted pages are not
copied as downloads.

R5 remains **partially addressed** for broader source compatibility: custom
heading IDs, fragments to the removed leading H1, source route overrides,
HTML/JSX attributes and imports, and custom static directories need manual
migration. Ordinary heading fragments are preserved and a real compiled H2
anchor is checked; this is not comprehensive anchor validation.

Local verification: 109 Rust tests, 261 engine tests, formatting, Clippy with
warnings denied, workspace typechecks, engine package build, updated docs
content validation and both CLI-to-engine conversion fixtures pass. CI adds
actual static exports with matching Next/site basePath and binary comparisons.
Consult PR #3 for hosted evidence before treating exports/browser checks as
passed. No native CLI binary release is issued by this milestone.

## Implementation record — scaffolding and discovery

Commits `fe4240c` and `ee68282`
([PR #4](https://github.com/mylife-inc/Zellij/pull/4)) address **R6, R7 and R8**
while preserving the original reproductions below:

- **R6:** Rust content generation and conversion share a string serializer;
  Node content initialization uses equivalent YAML scalars. Quotes, hashes,
  colon-space, implicit booleans/numbers, backslashes, CR/LF, tabs, Unicode
  line separators and non-printable characters retain their exact values. Copyright strings are serialized
  as whole values, and guide front matter uses the same policy.
- **R7:** plain and minimal starters have no guide CTA unless a guide is
  generated. The integration fixture loads actual CLI output with the engine
  and renders Hero to verify each link against the generated route inventory.
- **R8:** remove the CLI's duplicate two-file detector and use core discovery
  for check/dev/build and other resolving commands. Preserve `content/` before
  the current root, then support `docs/` and `documentation/`. Recognize
  menu-only/sections-only/guide-only roots, reject misleading filesystem marker
  types and hidden/reserved guide folders, and never search ancestors. Explicit
  directory selection remains exact. Content validity still belongs to the
  engine, and an empty menu-only root appropriately warns about no front page.

Validation: **117 Rust tests**, **262 engine tests**, formatting, Clippy with
warnings denied, workspace typechecks, engine package build, documentation
content check and both existing conversion integration fixtures pass locally.
The new scaffold integration check passes **40 Rust starter combinations,
10 Node content starters and 10 guide titles**, checking metadata and rendered
links. CI adds real-container discovery/check acceptance for four guide-only
locations and a menu-only root; read the PR's checks for hosted outcomes.
No native CLI release is issued here. Workflow input serialization and npm
package-name validation are outside this content-metadata fix.

The automated security-review plugin could not read its model response during
the source commit; its review did not complete. The source commit succeeded.

PR #3 subsequently merged as `dd99c2c`; main CI `39007109533` and image run
`39007109576` passed. All four image variants published for AMD64/ARM64. The
browser run had 296 initial passes, one successful retry and 114 conditional
skips: a mobile-WebKit `/guide/quickstart` navigation error. Its cause, and the
different FAQ interaction retry in PR CI, remain unproven.

## Implementation record — header layout and width-aware overflow

Commit `da1c222` addresses the requested header placement and width budgeting.
The bar now spans the viewport with bounded edge padding, company branding at
the left and controls at the right. Existing logo/home-link configuration is
preserved and covered by a server-rendering test; the configured company link
and light/dark asset URLs survive a deployment prefix.

Desktop overflow uses measured label/control space instead of relying only on
item count. Hidden links and nested groups remain available in More; the
mobile drawer retains the complete resolved menu. Focus is restored when items
move, stale panels close across the mobile breakpoint, and the drawer releases
scroll lock. More panels support scrolling in short viewports; compact panels
are bounded at both horizontal edges. Default mobile controls are 44px.

Evidence: 268 engine unit tests, workspace typechecks, engine packaging and
documentation validation passed locally. Focused desktop/mobile Chromium
navigation passed 34 tests with 26 conditional skips. Six new desktop header
cases pass, including the separately rerun compact-panel boundary regression;
these cover 320–1600px, larger text, a wide brand, preserved nested/external
links, focus and drawer cleanup. Final engine typechecking and the six focused
unit tests pass after the positioning adjustment. The initial failures in the
new stress/boundary tests were incorrect fixture widths, corrected to exercise
the intended overflow type. Full production/browser/WebKit results belong to
the PR containing this commit. This does not close the separate S2/S4/S5/S6/S9
findings or constitute manual assistive-technology acceptance.

The original UI observations below are retained as historical evidence; their
centered-container/count-only implementation is superseded by this record.
Search remains an icon opening a dialog. Site owners still supply their own
company assets and destination through the existing branding configuration.

PR #4 subsequently merged as `4be61dc`: PR CI `39010124427` passed 297 browser
tests with 114 skips and no retries. Main CI `39010596721` passed with 296
initial passes, one successful mobile-WebKit FAQ retry and 114 skips. The FAQ
interaction flake remains undiagnosed. Image run `39010596751` published all
four variants for AMD64/ARM64 and passed published-image smoke acceptance.
No native CLI release was issued.

## Implementation record — canonical search and content preparation

Commit `29654d3` resolves **R10–R12** and **S2**. Preparation and runtime share
content-source/path resolution. Search consumes canonical `loadSite` pages,
including sidebar/frontmatter route overrides and title-stripped bodies. A shared
nonexecuting MDX parser uses the renderer's slug plugin for search and TOC IDs.
Formatted, duplicate, setext, quoted, indented and mathematical headings are
covered against actual rendered HTML; deeper subsections remain searchable.
Optional math plugins stay dynamic in the CLI bundle so MathJax resolves its
runtime files correctly. Both installed-math and isolated plain-content bundles
are exercised.

All three Node preparation commands now accept `--content`/`--config`, and their
helpers accept the runtime options object as well as the existing app-root string.
Missing roots, malformed asset directories and overlapping source/output paths
(including symlinked parents) fail before removing prepared assets. Rerunning
sync after deleting the source assets directory clears stale assets and hashes.
This resolves only the deletion half of **S9**; automatic dev watching remains.
Guide-root and home-redirect route collisions are rejected, while a guide's own
root/index identity is allowed. Search load cancellation is restartable, failed
requests retry on reopening, and successful data is cached by deployment prefix.

Evidence: **285 engine tests**, workspace typechecks, engine packaging, preparation
and checks for all four examples, and packaged-CLI integration pass locally.
The four focused desktop/mobile Chromium recovery cases pass without retries.
An initial broad dev-server run timed out on the existing `networkidle` condition;
production/WebKit checks remain hosted gates. The new recovery fixture initially
clicked before hydration; it now uses the existing retrying shortcut helper.
CI adds a real static export and verifies each search URL and heading ID against
its emitted HTML. Consult the PR containing this commit for hosted outcomes.

**R13 remains open:** contradictory fast-check promises are corrected, but deeper
prose-link/asset validation is still missing. S8 runtime memo isolation and S12
basePath synchronization/acceptance are unchanged. Search's prose cleanup remains
regex-based. No native CLI release is part of this milestone. Historical source
locations and reproductions below describe the original audit revision.

Prior milestone completion: PR #5 merged as `7fb5178`. PR CI `39019128511` and
main CI `39019556088` passed, with 305 browser passes, 124 conditional skips,
no retries, 268 engine tests and 117 Rust tests. Image run `39019556101` published
all four feature variants for AMD64/ARM64 and passed build/serve smoke acceptance.

## Implementation record — deployment prefix and runtime isolation

Commit `8463720` resolves **S6, S8 and S12** while preserving the original
findings below. `withZellij()` and rendering share native content-root/prefix
resolution. Content's basePath configures Next automatically; a wrapper may
provide the value when content omits it. Explicit disagreements fail, including
empty/nonempty conflicts. Normalization rejects malformed prefixes. Custom
roots are supplied explicitly to preparation, runtime and Next configuration.
The effective Next build value is embedded in reserved `ZELLIJ_BASE_PATH`, and
runtime options cannot silently disagree with it. Prefix changes need a rebuild.

The runtime cache now separates absolute application/configuration/content
paths and prefix settings, preserves the development TTL, and evicts failed
loads. OG/Twitter images receive the deployment prefix. Shared native bootstrap
files are included in package and container output. Conversion CI exports use
the ordinary image instead of a special prefix-config override.

The stale showcase verifier now runs an isolated generated wrapper with a
content-owned prefix and custom content folder. Its real production browser
checks cover redirects, nested navigation, ordinary and optimized images, Next
resources, metadata, sitemap/robots and search-result navigation. Failed browser
checks retain traces. Hosted CI exercises Chromium and WebKit with Turbopack.

Local evidence: **303 engine tests**, workspace typechecks, engine packaging,
packaged-CLI integration and preparation/checks for all four examples pass.
Production prefix acceptance passes Chromium with webpack and local font
responses. The first normal-font build succeeded but exposed an overly strict
fixture assertion that rejected a valid bare-prefix home URL; that assertion was
corrected. A later normal-font build failed in Next's Google Fonts loader (S13).
The temporary font fixture is local verification infrastructure, not a fix or a
shipped fallback. Read this commit's PR for normal-font/Turbopack/WebKit results.
No native binary release is issued here.

PR #6 subsequently merged as `d115a9a`. PR CI `39022447464` passed all four jobs,
285 engine tests, 117 Rust tests and 311 browser tests with 124 conditional skips
and no retries. Main CI `39022871779` passed with 310 initial browser passes and
one successful retry of the existing mobile-WebKit FAQ interaction. That flake
remains undiagnosed. Image run `39022871680` published all four feature variants
for AMD64/ARM64 and passed published-image build/serve smoke acceptance.

New user scope is recorded in [the portable documentation contract](zel-portable-docs.md):
build a plain Markdown folder into a navigable direct-file website, serve a
generated site over HTTP, and compose build/server/browser launch in `view`.
These capabilities are **not implemented by this prefix milestone**. A Next
HTTP static export is not direct-file acceptance. Core MkDocs capabilities must
be compared during CLI work; plugin-specific parity is a separate scope choice.

## Implementation record — deep and strict content checks

Commit `86e5e2d` adds `--deep` and `--strict` to the Rust and Node check commands.
It addresses **R13's static-reference/syntax portion**, preserving the default
fast check and the original reproduction below. Build/dev/migrate do not silently
enable the new modes. The original invalid fixture now reports its missing local
image, nonexistent CTA route and malformed MDX in one run. Strict mode returns
failure for warnings as well as errors.

Deep analysis reads canonical routes, follows home/guide redirects and uses the
renderer's heading/GFM IDs. It checks Markdown/reference-style links and images,
literal HTML/JSX references, section CTAs/media, inline-prose links, social/brand
media and SectionPreview examples. Schema metadata defines reference semantics;
the Rust reference is regenerated from those same schemas. Inline tokenization
is shared with rendering. Duplicate reference definitions use the first value,
code examples stay literal, and authored MDX JavaScript is never evaluated.
It detects missing/recursive previews and duplicate rendered IDs. Physical asset
containment rejects escaping paths/symlinks and directories masquerading as files.

The checker found four additional broken showcase anchor links to `#display`,
`#chip` and `#battery`; these now point to existing highlights/specifications
content. Fast navigation validation also stops mistaking query strings for part
of a route. Error messages identify the source file and field/body position.

Local evidence: **317 engine tests**, including 14 new validation regressions,
**117 Rust tests**, formatting, warnings-denied Clippy, workspace typechecks,
engine packaging, isolated packaged-checker/preparation integration, and all four
local examples with `--deep --strict` pass. CI adds checks for the three fully
tracked examples and a real Rust/container validation fixture whose accepted
routes, anchors and assets are verified in a static export. Read this commit's
PR for hosted browser/container results. No native binary release is included.

R13's remaining boundaries are documented rather than hidden: parsing is not
component/JavaScript execution; external URLs and literal menu `url:` are not
requested/validated. Dynamic MDX, custom components, component ID props, srcSet
and links without a page context warn. Strict mode fails these warnings. Custom
wrapper public resources and SVG-internal fragments need build/browser checks.
The optional math parser must be installed in the checking environment; the bare
content-only image does not install optional packages. Complete execution and
feature-specific validation therefore remain part of release acceptance.

PR #7 merged as `ba30f9b`. PR CI `39025935776` and main CI `39026348416` passed
303 engine tests, 117 Rust tests and 311 browser tests with 124 conditional skips
and no retries. Chromium/WebKit prefix acceptance passed with normal fonts and
Turbopack. Image run `39026348423` published all four feature variants for
AMD64/ARM64 and passed published-image build/serve smoke acceptance.

## Assessment

The shared rendering engine is substantially implemented: 23 sections, 14
themes, four layouts, multiple guide trees, MDX, search, motion, branding,
assets, static output, and standalone containers. Four apps consume it.
The principal unfinished product area is the Rust authoring and shipping
workflow, including safe file writing, conversion fidelity, declared commands
that are stubs, consistent validation, platform acceptance, and release gates.

The unit suites are useful but insufficient evidence of product completion.
Several generated outputs are valid YAML yet invalid Zellij content, and
content checks accept some sites with broken links, assets, or MDX. These are
failures at boundaries between otherwise tested components.

## Where work stopped

The latest local implementation commit is **2026-09-25, `6356b05`**, applying
`menuOverflowAfter` to explicit menus. Its commit message acknowledges that
item-count overflow is incomplete: a few long labels can still run into the
search controls. Before that, `5001c37` fixed several MkDocs/Material-to-MDX
conversion cases and clarified container build errors; `e97099f` rejected
successful-looking builds with empty output.

All three commits are absent from the live remote `main` observed during this
review. Local `content-model` contains no commits missing from local `main`;
its ahead-of-tracking count should not be mistaken for another unmerged feature.

### Repository and distribution observations

| Item | Observed state on 2026-10-16 |
| --- | --- |
| Existing source repository | [mylife-inc/Zellij](https://github.com/mylife-inc/Zellij), **private** |
| Default branch | `content-model` |
| Remote main | `70fc7c3e0e2489ee116a97ee7afd7ed6a4ff305b` |
| Remote content-model | `b34a8c343abfffd3b8e225ff6293f33a7b27cf38` |
| Local main | `6356b05`, three commits ahead |
| Latest observed Actions runs | Failed before job execution because of billing/spending limits |
| Public binary artifacts | [zellij-v0.1.3 in mylife-inc/releases](https://github.com/mylife-inc/releases/releases/tag/zellij-v0.1.3), published 2026-08-13 UTC |
| Binary targets | macOS arm64/x86_64; GNU/Linux arm64/x86_64; tarballs/checksums/install script |
| Windows | No native target; WSL documented instead |
| Engine base image | `ghcr.io/mylife-inc/zellij-base:latest`, live manifest includes `linux/amd64` and `linux/arm64` |

The [examined CI run](https://github.com/mylife-inc/Zellij/actions/runs/31653664552)
reports that recent account payments failed or the spending limit needs to be
increased. Jobs did not run their tests. This infrastructure failure is distinct
from the reproducible local Clippy failure below.

The inspected base-image index digest was
`sha256:cc2ececbb51e116f81372257b59c4f595f62039f22bbfde566e15bfe9e40697e`.
Its two `unknown/unknown` entries are attestations, not additional runnable
platforms. Individual feature-tag manifests and correspondence between the
published image and the three unpublished commits were not verified.

The desired repository name is already occupied. Before migration, choose
either to reuse/change the visibility of the existing private repository or to
rename/preserve it and create a new public repository with a reviewed starting
snapshot. The user explicitly rejects using `Zellij-Sealed` for development.
No creation, rename, visibility change, push, release, or deletion was performed.

Standard GitHub-hosted runners are free for public repositories; concurrency,
execution limits, and separately billed resources still apply. See
[GitHub Actions billing](https://docs.github.com/en/billing/concepts/product-billing/github-actions)
and [Actions limits](https://docs.github.com/en/actions/reference/limits).
Eventual deletion/encryption cannot retract source copies already obtained
while the repository was public. Separately, the current Dockerfile copies
`src` into the public base image; encrypting Git history alone would not change
that distribution behavior.

## Verification performed

| Check | Result and boundary |
| --- | --- |
| Engine Vitest suite | **227 passed**, 11 test files |
| npm workspace typecheck | Passed across engine and four apps |
| Engine package build | Passed; client-directive verification passed |
| Generated JSON schema and context | Fresh output matched both committed embedded copies |
| Rust unit tests, offline | **82 passed**: converter 47, engine adapter 11, scaffold 24; CLI/core have no unit tests |
| Rust workspace debug build, offline | Passed; fixture tests used this freshly built `zel` |
| Rust formatting | Passed |
| Rust Clippy, all targets, warnings denied | **Failed**, `material.rs:29`, `clippy::question_mark` |
| Local example content checks | All four passed; counts below |
| Scratch integration fixtures | Reproduced the findings labeled R below, including every generated section type |
| Showcase Next production build | Incomplete: first network restrictions blocked Google Fonts; network-enabled retry then failed because local `@tailwindcss/postcss` was absent |
| Playwright, Lighthouse, current visual acceptance | **Not completed**, because the production build prerequisite did not pass |
| Native installers/cross-platform execution | Artifact presence checked; binaries not installed or executed on all four platforms |

Local content checks reported:

| App | Section pages | Guide trees/pages | Routes |
| --- | ---: | ---: | ---: |
| showcase | 2 | 1 / 5 | 7 |
| zellij-docs | 2 | 3 / 44 | 46 |
| payos | 3 | 2 / 8 | 12 |
| codeseal-docs | 3 | 2 / 18 | 21 |

Zellij docs also validated 23 section examples. These counts describe this
local content state. PayOS generates some ignored documentation from external
sources, so this is not proof of a self-contained clean-clone build.

Environment: bundled Node 24.19.0 and local Rust 1.99.0. System
`/usr/local/bin/node` failed to load a dynamic library; it was not repaired.
The missing Tailwind plugin is declared in the app manifest, so the build
failure establishes an incomplete local dependency installation, not that a
fresh `npm ci` would necessarily fail. No dependencies or implementation files
were edited to get past these environmental blockers.

## Reproduced defects

**P1** means data-loss/security or release-blocking user-workflow risk;
**P2** means a functional defect to fix before calling the relevant workflow
complete. All R findings below were reproduced in disposable temporary
directories, not against the user's content. Source locations refer to the
audited commit and will move as fixes land.

### R1 — P1: conversion can write outside its destination

**Trigger:** a MkDocs navigation entry `../../escape.md` with the referenced
file present outside the docs folder. Conversion emitted a file at
`destination/01-guide/../../escape.mdx`, outside the destination, and exited 0.

The MkDocs reader joins paths without containment checks; `slug_for` preserves
parent segments; the emitter incorporates that slug into an output path; and
the CLI writes it without checking containment. A source configuration can
therefore make conversion read/write beyond the expected folders.

**Sources:** `cli/crates/zellij-convert/src/mkdocs.rs`, `emit.rs:171`,
`cli/crates/zellij-cli/src/main.rs:1475`.
**Acceptance:** reject escaping/absolute paths and unsafe symlink resolutions;
prove every intended output stays under its selected root before writing any
file. Preserve support for normal nested document paths.

### R2 — P1: conversion silently overwrites existing work

**Trigger:** convert into a nonempty destination containing an authored
`site.yaml`. The file was replaced and the command exited 0. Passing the source
itself as `--out` also succeeded, despite the help text promising never to do so.

**Source:** `cli/crates/zellij-cli/src/main.rs:1433` and the write loop at 1475.
**Acceptance:** preflight source/output overlap, existing destinations, and all
collisions before writing; use a deliberate overwrite policy with diagnostics.

### R3 — P1: `new page` overwrites existing authored sections

**Trigger:** an existing `sections/home/hero.yaml` was replaced by a TODO
scaffold after `zel new page home --dir <content>`, without an overwrite flag.
The documentation explicitly promises that creation will refuse collisions.

**Sources:** `cli/crates/zellij-cli/src/main.rs:1209`, especially the unconditional
`scaffold.write(&root, true)` at 1220; scaffold writer `src/lib.rs:53`.
**Acceptance:** adding files to an existing content root is allowed, but an
existing destination file must be protected. Preflight all generated files so
a late collision does not leave a partially applied scaffold.

### R4 — P2: 11 of 23 generated section templates are invalid

Each `zel new section <type>` output was inserted into an otherwise valid
minimal site's `pages.yaml`, then checked by the actual engine.

Failing types: **bento-grid, card-carousel, code-cards, gallery, hero-split,
logo-strip, matrix, media-panel, stats-band, steps-panel, timeline**.
Examples: arrays shorter than schema minima; `code-cards` emits a string where
an object is required; hero-split/media-panel/steps-panel omit required
mutually exclusive visual choices; matrix dimensions do not match.

**Sources:** `cli/crates/zellij-scaffold/src/section.rs:15`, `render` and
`placeholder`; engine `src/sections/schemas.ts`.
Existing Rust coverage only checks that the generated YAML parses.
**Acceptance:** every generated template must pass the engine's own schema and
render with its placeholder assets/content. Exercise nested objects, minima,
and schema refinements, not just required field names.

### R5 — P2: conversion leaves broken document links and drops assets

**Trigger:** source Markdown contained `[Install](install.md)` and an image
`assets/logo.svg`. Output retained these strings, no assets were copied, and
engine validation passed. Actual generated document routes included
`/guide/install`, not a `.md` URL.

**Sources:** `cli/crates/zellij-convert/src/emit.rs:139`, converter model/result
types, and engine `src/mdx/components.tsx` (ordinary links pass through).
**Acceptance:** create a source-to-route/asset mapping, copy supported media,
rewrite links and anchors, and report unresolved references. Do not silently
guess when a source feature cannot be represented.

### R6 — P2: ordinary site names can produce invalid YAML

**Trigger:** `zel new docs <scratch> --name 'Acme: Docs'` exited 0, but its
`site.yaml` failed parsing because the name was interpolated unquoted.
Other generated values, including titles and quoted legal text, also need
correct serialization rather than raw string interpolation.

**Source:** `cli/crates/zellij-scaffold/src/content.rs`, `site_yaml`, `pages_yaml`,
`hero_yaml`, and guide generation.
**Acceptance:** names/titles containing colon-space, quotes, hashes, Unicode,
and newlines serialize correctly and validate.

### R7 — P2: the default new site links to a guide that does not exist

**Trigger:** plain `zel new docs` without `--guide` creates only `/`, but its
hero CTA points to `/guide`. The engine check still passes.

**Source:** `cli/crates/zellij-scaffold/src/content.rs:115`, `hero_yaml`.
**Acceptance:** starter links target generated routes for plain, minimal, and
guided variants; verify rendered links as well as schema acceptance.

### R8 — P2: Rust content discovery rejects a valid guide-only site

**Trigger:** a root with only `01-guide/_sidebar.yaml` and `index.mdx` passes
the engine check. Running `zel check` from that root exits before invoking
the engine with “no Zellij content folder found at .”.

**Sources:** duplicate resolver in `cli/crates/zellij-cli/src/main.rs:490` checks
only `site.yaml`/`pages.yaml`; `zellij-core/src/project.rs` already has broader
discovery logic but is not used for that path.
**Acceptance:** all commands share discovery rules that match valid engine
content roots, including menu-only and guide-only cases.

### R9 — P2: advertised CLI behaviors are false

`zel list themes` exits 1 with “not implemented yet” despite its Available
documentation. `zel context --json` exits 0 but emits Markdown; JSON parsing
fails. `--json` and `--verbose` are parsed globals but not consumed by dispatch.

**Sources:** `cli/crates/zellij-cli/src/main.rs:26`, dispatch at 361, and
`write_context` at 1397; `examples/zellij-docs/content/02-cli/context.mdx` and
`list.mdx`.
**Acceptance:** implement the documented output contract or clearly reject
unsupported modes; a successful JSON mode must emit valid, stable JSON.

### R10 — P2: search indexes routes and anchors different from rendered pages

**Trigger:** `01-guide/_sidebar.yaml` sets `route: /manual`; its index page
contains `# Install` and `## Install`. The real site routes include `/manual`.
The search file instead contains `/guide` and `/guide#install-1`. The canonical
loader strips the leading title heading, while the search reader keeps it,
changing the duplicate-heading suffix too.

**Source:** `engine/packages/zellij/src/cli/write-search-index.ts:94`.
**Acceptance:** generate search from the canonical resolved pages/heading
semantics, covering sidebar route overrides, page routes, home routes, and
duplicate headings.

### R11 — P2: custom `contentDir` does not reach assets/search preparation

**Trigger:** content under `external/` validates and is usable by the runtime's
`contentDir` option. Calling prepare helpers for the wrapper still looks in
`./content`: no asset copied, empty search index. Only the legacy config offers
an alternate path to these helpers.

**Sources:** `src/cli/sync-assets.ts` and `write-search-index.ts`, each with its
own `resolveContentDir`; `src/config/runtime.ts` supports richer options.
**Acceptance:** wrappers and preparation tools use one explicit content-root
contract, without requiring deprecated configuration as a workaround.

### R12 — P2: a guide root can collide with a section page undetected

**Trigger:** section page at `/guide` and guide tree at `/guide` with no index
but an intro page. Validation accepts both. Route resolution selects the
section page, shadowing the intended guide-root redirect.

**Sources:** `src/content/site.ts:244` only claims a guide root if the route is
not already present; `src/render/page.tsx:31` checks section pages first.
**Acceptance:** validate ownership of synthetic routes as well as physical
pages, allowing only intentional same-guide index/root identity.

### R13 — P2: successful content checks leave important errors undetected

One fixture combined a CTA to `/not-a-page`, a missing local visual asset, and
an unclosed MDX component. The check returned success. Menu/footer reference
validation is useful but is not comprehensive site validation.

**Sources:** `src/cli/check.ts:35`, `src/content/site.ts` validation scope.
Some documentation correctly says `check` does not compile MDX; other language
claims everything rejected by a build is rejected by check, or that broken
links cannot ship. This is partly a missing validation feature and partly an
incorrect promise, not a reason to erase the documented fast-check distinction.

**Acceptance:** define fast check versus build/deep check explicitly; validate
local references/assets where promised; ensure CI also compiles MDX and checks
rendered links. Preserve intentional unchecked literal menu `url:` behavior.

### R14 — release gate: Clippy fails on current local toolchain

`cargo clippy --workspace --all-targets --offline -- -D warnings` fails with
`clippy::question_mark` at `cli/crates/zellij-convert/src/material.rs:29`.
The CI workflow uses this warnings-denied gate. Unit tests and a normal build
passing do not make it green. Recheck with the selected CI toolchain when fixing.

## Additional source findings and risks

These are supported by source review but were **not** exercised through a full
browser/container workflow in this audit. Keep this distinction when triaging.

| ID / priority | Finding and trigger | Source / follow-up |
| --- | --- | --- |
| S1 / P1 | Static build clears `--out` using `rm -rf /out/*`. No source/output overlap or non-build-directory guard is apparent. An output path naming authored content could delete it after a successful build. | `cli/crates/zellij-engine/src/container.rs:393`; add a safe scratch-container regression before releasing a fix. |
| S2 / P2 | Closing search before its initial load completes cancels the effect while leaving `started.current = true`; reopening never starts another load and can remain on Loading. | `src/search/search-modal.tsx:73` and cleanup; test with a deliberately delayed index response. |
| S3 / P2 | `pages.yaml` accepts a per-page `theme`, but rendering uses only the site theme. Analytics config accepts Plausible settings but no analytics integration consumes them. | `src/content/model-schema.ts:95`, `src/render/sections-page.tsx`, `src/config/schema.ts:152`, `src/render/root-layout.tsx`. Implement or remove/document these silent no-ops. |
| S4 / P2 | For ordinary motion preferences, Reveal initially emits `data-zellij-reveal='out'`, and CSS sets opacity 0. With JavaScript unavailable, affected content has no visible fallback. | `src/motion/reveal.tsx:54`, `src/styles/sections.css:16`; test JS-disabled and hydration-failure cases. Reduced-motion CSS already restores visibility. |
| S5 / P2 | Stored theme restoration is injected only when `themeToggle` is true. A theme switcher enabled alone does not get that restoration. Conversely the script restores arbitrary stored theme names even when only one theme's CSS is emitted. | `src/render/root-layout.tsx:35`, `src/chrome/theme-toggle.tsx:20`, `src/themes/emit.ts`; test both controls independently and stale same-origin preferences. |
| S6 / P2 | OG/Twitter image resolution calls `assetHref` without the configured basePath, while canonical URLs include it. Shared-image URLs can be wrong under a subpath. | `src/render/page.tsx:85`; verify generated metadata under `/products/acme/docs`. |
| S7 / P2 | `image build` creates then deletes `<content>/.zel-dockerfile`; an existing file of that name would be overwritten and removed. | `cli/crates/zellij-engine/src/image.rs:105`; use collision-safe temporary files/cleanup. |
| S8 / P2 | Global runtime cache is not keyed by options; two content roots used by one process can receive the first site's data. | `src/config/runtime.ts`, `loadZellijRuntime`; define whether multiple runtimes per process are supported and test that contract. |
| S9 / P2 | Assets/search are prepared at startup; the dev content-cache TTL does not rebuild these derived files. If the entire source assets directory disappears, sync returns before deleting old public assets. | `src/cli/sync-assets.ts:24`, `write-search-index.ts`, container dev command; test edit/add/delete cycles. |
| S10 / product decision | Public base-image build copies full engine `src` despite a nearby comment claiming engine source stays behind. | `docker/base.Dockerfile:35`; current package source exports require this. Align future CodeSeal/source-distribution intentions with the real packaging design. |

The old single-platform comment in `scripts/publish.sh` is **not** evidence of
today's image-platform coverage; the live manifest contains both architectures.
Likewise, a missing local dependency is not evidence of a source-level build
failure in a clean environment.

### S11 — P3: feature selection does not produce the promised small dependency tree

Observed during the dependency milestone, after the initial audit: generate a
fresh wrapper with `zellij init ... --features none`, then resolve its lockfile
with `npm install --package-lock-only --ignore-scripts`. Despite no direct
math/diagram dependencies in the wrapper, the resolved tree still includes
Mermaid 11.17.2, remark-math 6.0.0, and rehype-mathjax 7.1.0. The engine's own
`optionalDependencies` request them, and ordinary npm installation includes
optional dependencies. Both minimal and all-feature wrappers audit clean;
this is a package-size/feature-selection defect, not a remaining advisory.

**Sources:** `engine/packages/zellij/package.json`, `src/cli/init.ts` feature
selection, `docker/base.Dockerfile`, and the container's ordinary npm install.
**Acceptance:** optional packages are absent from an installed minimal wrapper,
selected features actually install and render, and transitions between feature
sets do not retain unrequested packages. Verify installed trees and image sizes,
not only the wrapper's direct dependency list. This follow-up remains open.

### S12 — P2: prefix configuration and its verification script can disagree

Observed during the conversion milestone: the wrapper generated by
`src/cli/init.ts` calls `withZellij({ typedRoutes: false })`; neither that config
nor `next-config.mjs` derives Next's `basePath` from `site.yaml`. The content
runtime does read that setting. Setting it only in content therefore prefixes
ordinary asset URLs while Next links still use the unprefixed deployment.
Currently both configurations must be supplied explicitly. The conversion CI
fixture does so; that fixture is not evidence the default wrapper synchronizes
the setting automatically.

Separately, `examples/showcase/scripts/verify-basepath.mjs` uses literal string
replacements for a commented Next basePath and an empty site basePath that are
absent from the current files. Those replacements silently do nothing. The
script needs to assert/set the actual configuration before its result can be
used as prefix-deployment evidence.

**Acceptance:** decide and document one reliable configuration path for
content-only builds; verify matching Next/content settings and fail clearly on
disagreement. Exercise a served site under a nonempty prefix, including links,
assets, search and Next resources, without relying on stale string replacements.

### S13 — P2 follow-up: intermittent build-time font loading failure

Observed after the initial audit while verifying `8463720`: one local webpack
production build succeeded with normal Google Fonts, while a later build failed
for `IBM_Plex_Sans` with `TypeError: Cannot read properties of null (reading '1')`.
The installed Next loader at
`node_modules/next/dist/compiled/@next/font/dist/google/loader.js:122` indexes
`/\.(woff|woff2|eot|ttf|otf)$/.exec(googleFontFileUrl)[1]` without a null check.
This suggests a fetched font URL did not match its expected filename extension;
the original response/URL was not captured, so the provider condition and full
root cause remain unproven. Do not describe it as a diagnosed network outage.

**Source/follow-up:** `src/themes/fonts.ts` imports Google families at build time.
A local Next font-response fixture using an installed valid font allowed the
prefix browser acceptance to complete, but is not a product remedy. CI retains
the normal font path. Capture a failing response, assess a reproducible font
source/fallback strategy, and verify theme/font output in clean production builds.
Runtime self-hosted fonts do not remove this build-time external dependency.

### S14 — P2: video-embed omits the deployment prefix for local media

Found by source review during the deep-check milestone, not yet exercised in a
prefixed browser fixture. `src/sections/components/video-embed.tsx` does not take
`site` from its props and calls `assetHref(section.src)` / `assetHref(section.poster)`
without `site.config.basePath`. A local video under `/products/acme/docs` therefore
gets an unprefixed URL despite the shared deployment configuration. The media-panel
and other image sections already supply the prefix.

**Acceptance:** apply the site's prefix to local video/poster URLs; leave external
embeds intact. Verify a real local video and poster under a nonempty prefix in
production browser acceptance. Static content validation cannot establish that
the renderer passes the right prefix to an existing asset. This remains open.

## User-requested UI and platform work

The owner's logo is **already modeled and rendered** through
`brand.logo: { src, dark?, alt?, href?, height? }`. An external `href` uses an
ordinary anchor; `brand.title` can keep a separate link to the site's own home.
This supports the requested company-site link without inventing another brand
schema. It still needs the desired site's assets/configuration and acceptance.

The header has a centered `.zellij-nav-inner` with
`max-width: var(--zellij-container)` (`src/styles/chrome.css:53`). Controls are
to the right within that container, rather than at the far viewport edge.
The unfinished work is to agree on and implement the available header width,
reserve space for brand/actions, and collapse menus according to actual width.
`nav.align` controls menu placement; changing it alone does not solve the width
budget or long-label collisions.

Suggested acceptance: company logo at the requested left inset; search/theme
actions at the requested right inset; no overlap with long labels or larger
text; stable overflow on resize; usable keyboard menus, mobile drawer, search,
sticky/transparent states, and light/dark logo variants. Decide whether the
search control remains an icon opening the modal or becomes a persistent input.

For `zel`, the current release matrix covers four native Unix binaries. Native
Windows was explicitly removed in `6a44d37`, and the installer rejects
MSYS/MINGW/Cygwin with WSL guidance. Restoring native Windows requires more than
a runner entry: `.exe` packaging, an installation path/installer, runtime and
path handling, Docker Desktop/WSL integration expectations, and actual Windows
smoke tests. Clarify x64 versus ARM64 support and GNU/Linux baseline; Alpine
musl and other OS targets are not covered by the current four binaries.

## Remaining work to define a finished v1

This is a proposed order, not authorization to implement every old spec idea.

| Stage | Work | Completion evidence |
| --- | --- | --- |
| 1. Protect authored data | R1–R3, S1/S7; safe path containment, overlap/collision policy, preflight writes | Adversarial scratch fixtures cannot escape or overwrite protected input; failures leave no partial output |
| 2. Make authoring/conversion dependable | R4–R8, R10–R13; link/media mapping, valid templates, names, discovery, shared routing/preparation | All 23 templates validate/build; representative real conversions retain links/media/navigation; intentionally bad fixtures fail with useful errors |
| 3. Finish header and interaction behavior | Width-aware nav, company-brand configuration, S2/S4/S5/S6/S9, accessibility and responsive behavior | Browser coverage at 360–1440 px and long-label/larger-text cases; Chromium and WebKit; keyboard and reduced-motion checks |
| 4. Finish the chosen CLI surface | `list themes`, JSON/verbose output, config loading; then `new app`, `serve`, `fmt`, deployment/workflow commands selected for v1 | Help/docs match execution; commands have tested exit/output contracts and end-to-end examples |
| 5. Make distribution reproducible | Repository/default-branch decision; fix Clippy; CI/release gates; four existing binaries and images; native Windows if agreed | Green clean-clone CI, version/checksum matching, fresh installer/runtime smoke tests, documented platform support |
| 6. Complete acceptance and docs | BasePath, metadata/search/assets, clean app builds, performance/accessibility, docs/spec reconciliation, finite feature freeze | No known blocking defects; agreed acceptance matrix passes; remaining nonblocking issues explicitly tracked |

Important CLI decisions still open:

- Config filename/precedence (`zel.toml` versus `zellij.toml`), `--cwd`/`--config`,
  stable diagnostics, quiet/color/confirmation behavior, and JSON contracts.
- `--features`, feature autodetection, platform/build args, site URL/basePath,
  `--open`, full watch behavior, and asset/validation policies are described
  more broadly than the Rust CLI accepts today.
- Which deployment targets are v1 requirements? Seven advertised provider names
  do not mean seven working integrations. Generated Netlify workflows currently
  use Netlify's own CLI; `zel deploy` is a stub.
- Whether `publish`/`ship` convenience compositions, `eject`, upgrade/completions,
  and additional converters are required for v1 or later work. MkDocs and
  limited Docusaurus exist; VitePress/Jekyll/Hugo and broader sources do not.
- Version pinning: native binary, embedded schema, and engine image should be a
  known compatible set; mutable `latest` is not proof of that compatibility.
- Reconcile the temporary legacy-content adapter policy with real example
  consumers before removing compatibility.

Existing spec acceptance includes strict typing, static export, browser
coverage, basePath correctness, ≥95 Lighthouse performance/accessibility and
100 SEO. These numerical targets were **not measured** in this audit.
Reconcile obsolete fixture counts with an explicit coverage matrix rather than
assuming old milestone labels prove acceptance. Engine v1 non-goals remain
versioned docs, i18n/locales, remote content, CMS integration, comments, and A/B
testing unless the user changes scope.

## Documentation discrepancies to reconcile

- Section/theme/app counts have drifted across specs, README, generated starter
  copy, and CLI prose. Current inventory is 23/14/4.
- CLI spec introduction still describes an unimplemented tool; content-model
  design has an early awaiting-signoff label and a later implemented status.
- `new project` is implemented but listed as planned in part of the CLI docs.
  `list themes` and JSON context are advertised as available but do not work.
- Several labs demonstrate `--open`, config files, or feature flags the Rust
  parser does not support. CI documentation mentions options absent from the
  parser while omitting some implemented provisioning options.
- Guide-folder prose is inconsistent about unlisted pages; actual behavior is
  a generated page plus a warning, not exclusion from publication.
- Site config examples/promises include an invalid `footer.social: true` shape,
  unsupported analytics descriptions, and a per-page theme with no effect.
- Conversion/check claims exceed current asset/link/MDX behavior. The blanket
  “generated reference cannot be stale” claim does not protect manually written
  prose or omitted schema semantics in the scaffold generator.
- Some source/repository links still name `shebka/zellij`; the current source
  remote is `mylife-inc/Zellij`.
- Offline-build and source-free-container descriptions need to match dependency
  installation, fonts, source exports, and traced runtime contents.

## Evidence and limits

The study covered the three specifications, the documentation site's content
and labs, architecture-critical engine paths, schemas/registries, CLI dispatch,
scaffolding/conversion/container paths, CI/release/install scripts, example
configuration, test coverage, and recent Git history. Individual visual section
components were inspected selectively; this is not a claim that every source
line or every browser state has been exhaustively audited.

Scratch evidence on this machine was saved under
`/private/tmp/zellij-audit-repro.mjs`, `zellij-audit-reproductions.json`,
`zellij-audit-reproductions.log`, and `zellij-audit-*-tests/build/typecheck` logs.
Temporary files can disappear; the findings and reproduction conditions above
are the durable record. The fixture runner only wrote into newly created
temporary directories and exercised the freshly built CLI plus compiled engine
helpers. No source fix was made, no new implementation test was committed, and
no user content was overwritten.

Not established here: a vulnerability-free dependency tree, exhaustive security
review, a successful clean production build, cross-platform runtime correctness,
all deployed-site behavior, or absence of undiscovered bugs. “Finished” should
mean an agreed feature scope, passing release acceptance, and no known blocking
defects—not a technically unprovable guarantee of zero future bugs.
