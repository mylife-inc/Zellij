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

## Implementation record — development preparation watching

Source commit `8674908` completes **S9's preparation watching**. `zellij dev`
prepares content and owns a polling watcher plus the installed Next dev process.
`zellij prepare --watch` exposes the same watcher independently. Generated apps,
the four examples and consequently the base-image wrapper use this path; Rust
`zel dev` retains its existing invocation. Existing wrappers need the updated
script or standalone watcher. No native CLI release is part of this milestone.

Polling metadata works across bind mounts, waits for stable saves, serializes
updates and picks up edits during preparation on a later scan. Asset and content
snapshots are separate, so prose edits do not recopy media. Custom paths, legacy
config root changes, symlinked roots, asset removal/recreation and generated-file
exclusion are covered. Nested symlinks are not traversed. Failed preparation
remains observable and recovers on the next corrected edit; startup errors fail
before Next launches. Pending asset changes are retained across prose failures.

Prepared JSON files use temporary-file replacement. Asset copying/hashing occurs
before replacing the old tree. There is a brief directory rename gap and no
whole-site transaction. The supervisor forwards signals, propagates child exit
codes and stops its watcher. Development search fetches fresh data on reopen;
production behavior is unchanged. Browser reload for page/images is still
manual. Deployment-prefix/config/feature changes require restart. This is not
full browser live reload or the requested portable Markdown renderer.

Local verification: **324 engine tests**, all workspace typechecks, packaging,
isolated installed-CLI/process lifecycle checks, existing packaged preparation
and deep/strict documentation checks pass. Real webpack/Chromium dev acceptance
verifies search without page reload, rendered updates on reload, page/asset
addition/deletion, malformed-MDX recovery and a released server port on shutdown.
CI adds actual Rust/container dev, installed-package and Chromium/WebKit dev
acceptance under normal Turbopack. See this source commit's PR for hosted results.

The browser's accessibility tree revealed new **S15**, recorded below. It was
not fixed by changing the watcher test to select the actual heading ID/text.

PR #8 merged as `d4615ca`; PR CI `39036985726`, main CI `39037525173` and image
run `39037525149` passed. Both browser runs had 310 initial passes and one
successful retry of the known mobile-WebKit FAQ interaction, plus 124 conditional
skips. Chromium/WebKit prefix acceptance passed. All four AMD64/ARM64 image
variants published and the published default-image build/serve smoke test passed.

## Implementation record — reproducible theme fonts

Source commit `7963587` removes the build-time Google Fonts dependency (**S13**)
using 75 pinned WOFF2 files across the twelve existing families. Original CSS,
Unicode ranges, styles/weights, fallback metrics and OFL licenses are preserved;
SHA-256 and source provenance are recorded in the manifest. Font files remain
unmodified. Generated CSS rewrites relative asset URLs and family names while
retaining public variables and non-preloaded loading behavior. A CSS URL
emits the combined full license notice as a deployed asset as well as retaining
the original notices in the package.

The package build verifies hashes and deterministic CSS offline and copies the
assets beside both compiled module formats. Git attributes preserve original
upstream bytes across line-ending settings. No normal build fetches Google
Fonts; npm dependency installation is a separate network requirement.

Local verification: 324 engine tests, all workspace typechecks, package build,
strict/deep docs check, and npm archive font/license contents pass. Actual
webpack/Chromium production and development scenarios decode all font faces,
verify lazy loading and prefixed local URLs, and retain deployed notices. The
existing prefix/navigation/search/asset and watcher recovery checks also pass.
Hosted CI adds a fresh Turbopack build with Docker network disabled after npm
installation and Chromium/WebKit font acceptance. See the source commit's PR
for hosted completion; no native binary release is part of this milestone.

First hosted run `39053315323` passed engine/schema/CLI jobs and the network-disabled
Turbopack build. Production browser tests had 310 initial passes and one successful
retry of the known mobile-WebKit FAQ interaction, with 124 conditional skips.
The production font gate correctly failed when Turbopack stripped license
comments. Follow-up `21479d3` replaces the comment-only notice with an emitted
asset referenced by the stylesheet and verifies its full HTTP response. Local
webpack/Chromium dev acceptance passes. Final results are tracked on
[PR #10](https://github.com/mylife-inc/Zellij/pull/10).

Run `39054096571` passed 311 production browser tests without retries. The font
check then fetched a stylesheet-relative notice URL relative to the page and
received 404. Its trace confirmed `url(../media/licenses.3-xnmaabgbyei.txt)` in
`/_next/static/chunks/...css`. The check now resolves against that stylesheet,
retaining full-byte notice and deployment-prefix assertions. Local webpack
production acceptance passes the emitted license asset.

Run `39054561760` passed engine/schema/CLI jobs and both production browser/font
gates. Chromium dev passed; WebKit dev requested Next's own debug-overlay Geist
font at `/__nextjs_font/geist-latin.woff2`, triggering the site's prefix assertion.
Inspection of Next's installed `get-dev-overlay-font-middleware.js` confirms its
four allowed local Geist paths intentionally bypass basePath. The development
check now recognizes only those framework paths; all other font URLs retain the
local-prefix assertion, and production has no exception. Production browser tests
had 310 initial passes plus one successful known FAQ retry and 124 skips.

PR #9 merged as `bd01bdb`. Main CI `39052261993` passed all jobs with 311 browser
passes, 124 conditional skips and no retries, plus Chromium/WebKit prefix and
dev acceptance. Image run `39052261989` published all four AMD64/ARM64 variants
and passed the published default-image build/serve smoke test.

PR #10 merged as `5a9e12c`. Exact-head CI `39055211438` passed all four jobs,
324 engine and 117 Rust tests, 311 browser tests with 124 conditional skips and
no retries, Chromium/WebKit production/dev font-license-prefix gates and the
network-disabled Turbopack build. Main CI `39055788092` also passed, with one
successful retry of the known mobile-WebKit FAQ interaction (310 initial passes,
124 skips). Image run `39055788338` passed all eight platform/feature builds,
four manifest publications and the default-image build/serve smoke. S13's
hosted acceptance is complete; native CLI binaries have not been released.

## Implementation record — CLI inspection output

Source commit `4d659b4` fixes **R9**. `zel context --json` returns the engine's
embedded JSON, including a generated theme catalog. `list themes` now uses the
same names/labels/descriptions as the engine's renderer. JSON is supported for
all lists, explain (complete section schema) and detection (ordered matches).
Context file output respects the selected format and sends JSON-mode
confirmation to stderr. Errors keep stdout empty and return nonzero.

Unsupported `--json` commands fail with exit 2 before execution. The unused
`--verbose` / `-v` parser option is removed and rejected. This implements the
inspection subset and honest refusal required by R9, not universal structured
diagnostics or verbose tracing. Detection now rejects missing/non-directory
paths; valid folders with no match return an empty JSON array successfully.
Docs/specs explain the current contracts and preserve the remaining plans.

Local acceptance: 125 Rust tests (eight new binary integration tests),
warnings-denied Clippy, formatting, 325 engine tests, workspace typechecks,
engine build and deep/strict docs checks pass. Tests compare all 23 section
schemas and all 14 themes to the embedded engine model, exercise file and
stream output, and reject flags before writers/container commands run, with
external tools absent from PATH. The engine suite passed with two local workers
after two existing math tests timed out during concurrent Rust compilation;
timeouts and assertions were unchanged. Hosted full acceptance is pending.
Native binary releases and portable build/serve/view are separate milestones.

Hosted completion: [PR #11](https://github.com/mylife-inc/Zellij/pull/11)
merged as `0a89131` after final source/help head `c411317` passed CI
`39059091256`. Main CI `39059796288` also passed all four jobs: 125 Rust tests,
325 engine tests, typechecks/schema sync, zero npm advisories, and **311 browser
passes with 124 conditional skips and no retries**. Container/scaffold/conversion,
font/license, production-prefix and development-watcher acceptance passed.
Image run `39059796306` passed all 13 jobs, publishing all four feature variants
for Linux AMD64/ARM64 and passing the default-image smoke test. No native binary
release was issued.

## Implementation record — native portable Markdown build

Source commit `8f72022` adds the native `zellij-docs` crate and
`zel build --target portable`. A plain Markdown folder selects it automatically;
discovered structured Zellij content retains the hosted static default. The
renderer parses Markdown with pulldown-cmark, highlights code with syntect, and
writes full HTML pages with ordinary relative links. Embedded local scripts
provide search, theme selection and mobile navigation without fetch/router data.
The 14 themes are generated from engine tokens; the same pinned 75-font snapshot
and license notices are embedded by Cargo. Building needs no Node, Docker or
network. Source checkout builds do require the engine font snapshot in this repo.

Output defaults to a sibling `<input-name>-site`; `--out`, `--name` and `--theme`
configure publication. Index/README home selection, optional front-matter title/
description/order, directory links, heading/footnote fragments, referenced
assets, synthetic home, breadcrumbs and previous/next are implemented. The
shared staged publisher moved from `zellij-engine` to `zellij-core::output`
without changing the ownership/failed-build-preservation contract. Source and
output are disjoint; nested symlinks, missing references, duplicate outputs and
Windows-incompatible names fail before replacing output. URL paths normalize
Windows separators at the filesystem boundary.

Current limits: raw HTML is displayed literally with a warning; MDX is rejected.
Math rendering, Mermaid execution, custom navigation/config, portable watch/reload,
`serve` and `view` remain unfinished. External media is not downloaded. The
[portable contract](zel-portable-docs.md) and CLI build docs carry the exact
behavior and capability comparison. This is not complete MkDocs compatibility.

Local acceptance: **137 Rust tests**, formatting, warnings-denied Clippy,
generated-theme parity, deep/strict docs checks and Chromium file acceptance
pass. The browser fixture builds without tools on PATH, preserves an old export
on failure, moves output to a space/Unicode path, opens home and deep links with
networking blocked, and checks search/anchors, local fonts/media, keyboard,
themes, mobile layout and JavaScript-disabled content. Review caught and fixed
Escape clearing the search field instead of closing its dialog. CI now includes
native portable build/CLI/browser jobs on Linux, macOS and Windows, each with
Chromium and WebKit. Hosted results are pending at this record; these jobs do
not establish full native Windows container support or release/install coverage.

Browser follow-up: the first hosted run passed Chromium but WebKit rejected
its initial file navigation with offline emulation enabled. A minimal local
file containing only an H1 reproduced that failure; HTTP-blocked file loading
without emulation passed. WebKit acceptance therefore blocks HTTP requests
explicitly and asserts that every observed request uses file/data URLs; Chromium
also retains offline emulation. Both JavaScript and no-JavaScript contexts use
these guards. No file-security flags or HTTP fallback are used. Both engines
now pass locally. WebKit additionally exposed missing focus restoration after
pointer-opened search; the runtime now records and restores the opening control.

Portable-build completion: [PR #12](https://github.com/mylife-inc/Zellij/pull/12)
merged as `680779f` after final head `99c0f54` passed all seven jobs in
`39063380795`. Native Linux/macOS/Windows portable checks passed in Chromium
and WebKit. PR browser acceptance had 310 initial passes plus one successful
retry of the existing mobile-WebKit FAQ test, with 124 conditional skips.
Main CI `39064011730` passed all seven jobs, including 138 Linux Rust tests,
325 engine tests and **311 browser passes, 124 skips, no retries**. Image run
`39064011643` passed all 13 jobs and published four AMD64/ARM64 variants. The
local debug binary was rebuilt from main; no native release was issued.

## Implementation record — native serve and view

Source commit `e33cd92` implements `zel serve` and `zel view`.
`zellij-docs::server` uses tiny_http to serve existing static exports on IPv4
loopback. It supports selected and ephemeral ports, MIME types, GET/HEAD,
extensionless HTML, directory indexes/redirects, query preservation, custom
404 responses and an explicit `--base-path` mount. It serves bytes unchanged,
disables caching for preview and requires a regular root index.html. Traversal,
malformed encodings, metadata paths, nested symlinks and Windows junctions are
rejected; foreign Host headers are refused. There is no directory listing,
home-page fallback, or automatic prefix rewriting.

`zellij-cli/src/preview.rs` orchestrates the workflow. `view` reserves its port
before building and calls the same portable helper as `build --target portable`.
It builds plain Markdown only, prints the actual URL, and launches the system
opener in a separate thread. `--no-open` suppresses launch; `--browser` selects
one executable receiving the URL as one argument, without a shell. Missing or
failing launchers warn while HTTP keeps running. Source files remain unchanged;
build failures preserve owned output, and port conflicts fail before rebuilding.

Ctrl-C (plus POSIX termination signals) stops normally with exit 0 and releases
the listener. Response writers are bounded and do not block the control loop;
shutdown can interrupt in-flight responses and retains all exported files.
Neither serve nor view watches/rebuilds/reloads on edits. Portable raw HTML/MDX,
math/Mermaid, richer navigation/config, and native release/installer work remain
open. This preview server is not a public-hosting/TLS/authentication service.

Local evidence: **143 Rust tests**, warnings-denied Clippy, formatting and
updated deep/strict docs validation pass. Six new real-HTTP tests cover routing,
MIME/HEAD/methods, port and prefix behavior, path/symlink containment and shutdown.
`scripts/check-preview.mjs` checks the actual CLI without external tools on PATH,
both browsers under a URL prefix, launch readiness/errors/no-open, source and
failed-build preservation, and Ctrl-C with slow/incomplete requests. A compiled
browser substitute requests the ready URL before recording launch; automated
acceptance does not claim to exercise each machine's actual default GUI browser.
Windows CI uses `scripts/fixtures/preview-console.rs` to create a private console
for the tested child and send a real Ctrl-C event, rather than Node's force-kill.
Cross-platform hosted acceptance is pending at this record.

Native-preview completion: [PR #13](https://github.com/mylife-inc/Zellij/pull/13)
merged as `bc95346` after head `4b4da4c` passed all seven jobs in
`39066025208`. Main CI `39066685507` also passed all seven jobs: 144 Linux
Rust tests, 325 engine tests, 311 website browser passes, 124 conditional skips,
and no retries. Both Chromium and WebKit passed native direct-file and HTTP
preview on Linux/macOS/Windows, including launcher failure recovery and real
Ctrl-C. Image run `39066685512` passed all 13 jobs and published all four
AMD64/ARM64 feature variants. No native binary release was issued.

## Implementation record — portable watch and live reload

Source commit `2c44098` implements `zel view --watch`. Ordinary `view` remains
one-time build/serve/open; `serve` remains unchanged-file preview. The watcher
captures the resolved source before the initial build, polls streamed file-byte
hashes, waits for two matching scans, and serializes full rebuilds. Hidden entries
and node_modules are ignored. Same-size/preserved-timestamp edits are detected.
Nested links, Windows junctions, special files and replacement root links are
refused. Missing roots and invalid documents/assets recover on subsequent edits.
Large trees require full scans/builds; no incremental compiler is implemented.

`zellij-docs::prepare` returns a staged `PreparedBuild`; ordinary `build` publishes
it immediately. Watch mode publishes under the shared `LiveReload` write lock,
while HTTP holds a read lock through file resolution/opening and revision
attachment. Responses do not hold it while streaming to clients. This excludes
the publication rename gap from preview reads, without making multiple HTTP
requests a single generation transaction. Failed builds retain the last export
and revision and report their error in the terminal and a browser notice.

Only watch HTTP responses receive `_zellij/preview.js` and revision polling.
The version embedded in each HTML response detects updates even before its
first poll. Success reloads the current URL, preserving query/fragment; deleted
page 404 previews can recover on restoration. Files on disk receive no preview
client or endpoint and remain movable/offline. Ctrl-C joins the worker and
cleans staged work before reporting completion; exported files remain. Name,
theme, output and port changes require restarting the command.

Local evidence: **147 Rust tests**, formatting, warnings-denied Clippy, strict/
deep docs validation, and actual CLI acceptance in Chromium and WebKit pass.
Four added Rust regressions cover snapshots (including preserved timestamps and
links), staged publication, HTTP revisions/endpoints/HEAD and unchanged output.
The browser/process fixture additionally exercises the first-poll race, atomic
saves, added/deleted pages, changed/missing assets, search refresh, error notice
and recovery, temporary source loss, symlinks/junctions, ignored trees, continuous
HTTP availability during publication, Ctrl-C, and moved offline exports. CI
uses the same native Linux/macOS/Windows matrix and real Windows console helper.
Hosted acceptance is pending at this record. No native release is issued.


Hosted follow-up: run `39068473117` exposed a race-fixture deadlock on Linux
and macOS. The test held the first preview-status response but awaited the
page's full load before allowing that response to continue. It now waits for
DOMContentLoaded plus the actual intercepted request before editing, then
releases the response only after publication. Reload assertions and timeouts
are unchanged; production code is unchanged. Final hosted status belongs to
[PR #14](https://github.com/mylife-inc/Zellij/pull/14).


Run `39068818910` passed the corrected race on Chromium, but Linux WebKit also
held DOMContentLoaded while a network request was intercepted. The final fixture
therefore delays only the first fetch initiation in the page. It waits for the
client to attempt that poll, changes the source, observes actual publication,
and then permits the real HTTP response. A session marker removes the delay on
reloaded documents. Production code, reload assertions and timeouts are unchanged.


Run `39069154907` passed Linux/macOS native acceptance, all broader checks,
and Windows Chromium watch acceptance (including real Ctrl-C). Windows WebKit
still timed out awaiting DOMContentLoaded in the deliberately paused first-poll
scenario. The fixture now waits for navigation commit, then explicitly requires
the rendered heading and attempted poll before editing. Failed watch runs retain
Playwright traces plus browser readiness and CLI output for diagnosis. Full
post-reload navigation, search, image and offline assertions remain in place.


The trace from `39069833700` confirmed successful publication and browser reload:
the page contained the updated heading and the new revision. It exposed an early
search shortcut sent before deferred page enhancements initialized. The fixture
now waits for the controls that site.js reveals before pressing the shortcut;
static heading visibility alone is not JavaScript readiness. Production code is
unchanged, and the first-poll race still must complete before these search checks.


HTTP transport correction: traces from `39070119126` showed requests for
site.js/search.js receiving no response, so the browser could not start its
first poll. The earlier lifecycle/interception explanations above were
provisional and did not explain the underlying failure. A standalone local
64-connection burst reproduced missing responses in ordinary `serve`; closing
answered keep-alive sockets immediately released the stuck requests. Inspection
of tiny_http's task queue found that queued connections can outnumber idle
workers, leaving connections unprocessed while those workers wait on persistent
clients. This affects PR #13's server as well as watch mode.

Source commit `6f14500` replaces the transport with Hyper HTTP/1 and Tokio:
idle sockets are async tasks, file
lookups use a bounded 16-thread blocking pool, and file bodies stream with
backpressure. A maximum of 256 connections, a 10-second header-read timeout and
cancellation on shutdown bound the preview service. Existing routing, host/path
checks, byte-preserving serving, HEAD lengths and watch publication locks remain.
The new regression queues 64 persistent requests plus idle preconnections,
requires all responses without closing other clients, then reuses the sockets
and shuts down with them open (eight bursts). Its initial 64-client version
failed against tiny_http after five seconds; the replacement passed the server
suite. Full local validation now passes: **148 Rust tests**, warnings-denied
Clippy, formatting, deep/strict docs checks, and Chromium/WebKit actual CLI
HTTP/watch plus moved direct-file acceptance. Cross-platform hosted CI remains
pending at this entry; see PR #14 for the final exact revision and results.

Watch/reload completion: [PR #14](https://github.com/mylife-inc/Zellij/pull/14)
merged as `142965f` from final head `4f20092`. PR CI `39071150585` and merged-main
CI `39071818383` each passed all seven jobs. Native Linux/macOS/Windows passed
both Chromium and WebKit offline/HTTP/watch acceptance, including real Windows
Ctrl-C. The CLI job passed 149 Linux Rust tests; engine tests remained 325, and
website browser acceptance passed 311 tests with 124 conditional skips and no
retries. Image run `39071818346` passed all 13 jobs and published all four
AMD64/ARM64 variants. Native release packages were not issued.

## Implementation record — portable configuration, navigation and branding

Source commit `4ba149f` adds optional `<Markdown-root>/zel.yaml` for site name,
initial theme, local company logo/link and nested navigation. It has no parent
search. A present file selects portable automatic build dispatch before hosted
marker discovery; explicit targets still win. CLI name/theme flags override
file values, then folder-name/cupertino defaults apply. The entire file must
remain valid even when some settings are overridden.

Navigation entries contain exactly one of `page`, `children`, or HTTP(S) `href`.
Source-relative page filenames are exact and retain literal spaces/Unicode.
Groups/links require labels; duplicate or missing pages, empty groups, traversal,
unknown/duplicate keys and incorrectly typed values fail. Nesting is bounded at
16 groups. Configured order and aliases drive sidebar and previous/next links;
unlisted pages are still rendered, validated and searched, but have no paging.
Omitting nav keeps automatic ordering; an empty list intentionally hides links.
A synthetic home lists authored navigation. Folder breadcrumbs stay unchanged.

Logo paths are checked for containment, image type and symlinks/junctions, copied
locally, and linked with encoded relative URLs. The company logo can open its
HTTP(S) website while the adjacent title links to documentation home. Mobile
controls occupy a separate row to preserve title/logo space. Navigation and
branding remain usable without JavaScript and after moving a direct-file export.

Watch mode rereads configuration on creation, edit and deletion. Invalid settings
preserve the last export and show the existing preview error notice; recovery
rebuilds/reloads. Browser preferences now save only explicit visitor choices,
so new project defaults take effect until overridden. The new versioned storage
format may reset an older saved choice once. General `zel.toml` defaults remain
planned; this portable format is distinct from hosted `site.yaml`.

Local verification: 154 Rust tests, warnings-denied Clippy, formatting, deep/strict
documentation validation and a fresh CLI build passed. Chromium/WebKit passed
moved file://, HTTP and watch acceptance, including config addition/removal,
error recovery, theme precedence, logo loading, nested navigation, alias paging,
unlisted search, mobile layout and no-JavaScript use. Visual inspection caught
mobile brand compression; the fixed layout and geometry assertions pass in both
browsers. The existing three-platform CI matrix runs these added checks. Hosted
acceptance is pending at this dated entry; consult the PR containing `4ba149f`
for its final revision/results. No new native release packages are issued.

Configuration completion: [PR #15](https://github.com/mylife-inc/Zellij/pull/15)
merged as `7b7222f` from head `92f2d10`. PR CI `39075396282` and main CI
`39076012865` each passed all seven jobs, including Chromium/WebKit portable,
HTTP and configuration/watch acceptance on native Linux/macOS/Windows. PR CI
passed 155 Linux Rust tests and 311 website browser tests (124 conditional skips,
no retries). Image run `39076012860` passed all 13 jobs and published all four
AMD64/ARM64 variants. The local binary was rebuilt; no native release issued.

## Implementation record — portable publication paths

Source commit `90befda` fixes a publication/watch inconsistency found during
final review. At `7b7222f`, a configured `node_modules/logo.svg` was successfully
copied into an export, but watch snapshots exclude that directory and would not
notice edits. Explicit Markdown asset links also bypassed hidden/dependency
exclusions. Discovery, watch snapshots, logo configuration and referenced assets
now share the same excluded-name rule. Hidden paths and `node_modules` cannot be
published through those references. Authors must move published assets into a
normal source folder such as `assets/`.

Local verification: 18 configuration/portable/watch tests, warnings-denied Clippy,
formatting and deep/strict docs validation passed. Regression fixtures contain
real excluded files and check image/download references, logo configuration,
source preservation and preservation of the previous export. Hosted acceptance
is pending at this dated entry; consult the PR containing `90befda` for results.

Publication-path completion: [PR #16](https://github.com/mylife-inc/Zellij/pull/16)
merged as `a6a9a7d` from head `e188a13`. PR CI `39076533168` and main CI
`39077026320` each passed all seven jobs; image run `39077026408` passed all
13 jobs and published all four AMD64/ARM64 variants. Linux ran 156 Rust tests.
Native Linux/macOS/Windows Chromium/WebKit file, HTTP and watch checks passed.
The website suites each had 310 initial passes, 124 conditional skips and one
successful mobile-Safari retry: the PR's testimonial-carousel slide did not
become active within 5 seconds; main's FAQ answer remained hidden after its
first click. These are recorded flakes, not diagnosed or fixed by this work.
No native package release was issued.

## Implementation record — native portable validation

Source commit `7457daf` adds `zel check [dir] --target portable|static`, native
strict warnings and portable JSON reports. Automatic selection shares build's
helper: a root `zel.yaml` entry wins, then hosted discovery, otherwise portable.
The existing hosted engine checker remains available; explicit static is useful
for unmarked hosted folders. Hosted JSON is rejected before container execution.

`zellij-docs::check` and build now share a complete in-memory compiler. Check
performs configuration/discovery/Markdown/navigation/reference/render/output-path
validation without staging or writing files. Build then stages/publishes as
before. Virtual file/directory output collisions are caught before filesystem
writes, so checks can detect them too. Existing output ownership is irrelevant
to a check; it does not promise output permissions, disk capacity or publication.

Errors are collected per document during read/parse, then across references
when parsing succeeds. Fatal config/discovery/render/output errors stop their
phase. Raw HTML stays a warning; strict makes it fail without changing severity.
`--deep` is accepted but redundant for portable content, whose local references
are always checked. MDX and external URL availability remain outside this target.

Portable JSON emits one stdout report on validation success/failure with no
stderr diagnostics. Schema version 1 includes target/root/strict/ok, nullable
HTML page count and diagnostics with severity/code/nullable source-relative file/
message. No guessed line/column fields. Codes: source, configuration, markdown,
reference, output-path, rendering, raw-html. Exit 0 success, 3 content/strict,
1 returned I/O failure; first fatal error controls status. Usage errors use
stderr and exit 2. See the portable contract and `zel check` documentation.

Local verification: **164 Rust tests**, warnings-denied Clippy, formatting,
updated deep/strict docs checks, and fresh binary build passed. Chromium/WebKit
passed moved file:// navigation/search/branding/no-JS, HTTP preview and full
watch/configuration/recovery/Ctrl-C acceptance with the refactored compiler.
Nine new library/CLI tests verify check/build parity, multi-file errors,
structured output/exit behavior, target selection, strict warnings, collision
failures, and byte-preservation of source and existing output. The CLI tests
remove external tools from PATH. All-platform CI includes native checker tests.
Hosted acceptance is pending at this dated entry; consult the PR containing
`7457daf` for its final head/results. No native package release is issued.

Native validation completion: [PR #17](https://github.com/mylife-inc/Zellij/pull/17)
merged as `c391dc3` from head `df82cdf`. PR CI `39078577656` and main CI
`39079171966` each passed all seven jobs, including native Linux/macOS/Windows
portable validation and Chromium/WebKit acceptance. Linux ran 165 Rust tests;
the engine ran 325. PR website acceptance had 310 initial passes, 124 skips and
one successful mobile-Safari FAQ retry; main passed 311 tests with 124 skips and
no retries. Image run `39079171941` passed all 13 jobs and published all four
AMD64/ARM64 variants. Local main/binary were updated; no native release issued.

## Implementation record — progressive reveal and FAQ interaction

Source commit `9acd934` fixes the audit's **S4** entrance-content visibility
failure and a reproduced FAQ scrolling race. The changes affect the hosted
engine; native portable rendering is unchanged.

The browser investigation distinguishes established defects from the earlier CI symptom. At `c391dc3`, 30 instrumented ordinary mobile-WebKit FAQ trials and 60 original FAQ/keyboard/carousel repetitions all passed locally. The earlier CI first-click FAQ/carousel failures therefore have not been reproduced exactly or conclusively attributed to one cause.

A related lost-click race is reproducible with a 200 ms pointer-down/up interval: 4 of 20 mobile-WebKit FAQ trials failed to close the answer. Event traces show the summary moving hundreds of pixels during a press and the click being retargeted to a surrounding section/div. A comparison changing only document scroll-behavior from smooth to auto passed 20/20. The product now uses immediate document scrolling to prevent default focus/scroll-into-view movement continuing through a press; entrance effects and horizontal carousel rules remain. The existing FAQ test now uses a real 200 ms press without action retries.

The audit's S4 is independently confirmed: the new actual JavaScript-disabled browser regression fails against the baseline because entrance wrappers have opacity zero. Reveal now emits visible `pending` server markup, installs its observer before arming offscreen entrances, and never hides content already in view when hydration arrives. The six new cross-project browser cases verify visible wrappers with JavaScript disabled and native FAQ operation, plus opening a FAQ while Next scripts are held and preserving that state when scripts are released.

CI now retains first-failure traces rather than tracing only a retry, and uploads failure traces/error context even when a later retry succeeds. This enables evidence-based diagnosis if the historical first-click flake recurs. Passing current acceptance is not proof that every prior flaky failure had the same cause.

Local verification: **325 engine tests**, all workspace typechecks, engine
packaging, a normal Turbopack production build, and deep/strict documentation
validation passed. Full production browser acceptance passed **317 tests**,
with **124 conditional skips and no retries**, across desktop/mobile Chromium
and mobile WebKit. The corrected product passed **20/20** instrumented 200 ms
press trials with its actual styles (no injected scrolling override). The six
new no-JS/delayed-hydration cases also passed directly. No-JS coverage asserts
computed opacity/transform, not only Playwright's visibility check, which allows
opacity zero. See [Playwright's definition](https://playwright.dev/docs/actionability#visible).

Baseline failure traces and experiment logs were captured under
`/private/tmp/zellij-faq-*` and `zellij-reveal-*` on this machine. The important
reproduction and limits are recorded here because temporary evidence may expire.
Hosted acceptance is pending at this dated entry; consult the PR containing
`9acd934` for its final revision/results. Native installers, S5/S14/S15 and the
remaining portable capabilities are still open; the original sporadic CI
first-click failures must not be marked conclusively diagnosed by this evidence.

## Implementation record — native portable starter

Source `6af6a48` closes the portable contract's missing project-scaffolding
capability through `zel new portable [dir]`. It writes native Markdown and
`zel.yaml`, with a working guide/local-image example or a two-file `--minimal`
variant. Name/theme validation precedes writing; unusual display names are
quoted metadata, never executable Markdown/HTML/shell syntax. Shared write
preflight protects existing content, with no overwrite flag; dotfiles remain.
Hosted scaffold behavior is unchanged. Rendering extensions and release/install
packages remain separate work, not completed by this starter.

Local acceptance: **169 Rust tests**, formatting, warnings-denied Clippy and
deep/strict product-documentation checks passed. The real command's output passes
Chromium/WebKit after relocation with HTTP blocked, including full/minimal names,
navigation, fragments, local assets, search and JavaScript-disabled reading.
CLI tests cover all 14 themes, names, current-directory/default paths, invalid
options and preservation with external tools absent from PATH. The Linux/macOS/
Windows portable CI gate now runs these tests and generated-output browser checks.
Consult the PR containing `6af6a48` for its final hosted acceptance; it was pending
when this entry was committed. No native release is issued here.

Prior milestone: PR #18 merged as `ec0394a`; PR/main CI `39080731998` /
`39081222627` passed all seven jobs and 317 browser cases (124 conditional skips,
no retries). Image run `39081222598` passed all 13 jobs and published four variants
for both Linux architectures. The historical first-click flake's full cause
remains unproven; S4 and the controlled scrolling reproduction are fixed.

## Implementation record — native packages and installers

[PR #20](https://github.com/mylife-inc/Zellij/pull/20), source `94b2d8d` and
follow-up `489ad35`, implements six native packages (x64/ARM64 GNU/Linux, macOS and
Windows), a shared CI/release packaging workflow, tag/workspace-version preflight,
ZIP/checksum publication and a Windows PowerShell 5.1/7 installer. The archived
binary itself is installed and tested, including CPU architecture, version,
portable starter/check/build, relocated output, HTTP serve/view and real Ctrl-C.
Ordinary CI retains tested artifacts without publishing a version.

Source review found the Unix installer replaced the installed executable before
checking its version/execution and used a fixed `zel.new` staging file. It now
checks the candidate first and uses a unique destination-side stage. Regression
fixtures verify combined/per-file checksums, corrupt/missing checksums, unsafe
archives, mismatched versions, upgrades and preservation of installed/authored
files. Neither installer modifies PATH. Windows refuses arbitrary ZIP paths and
uses native machine architecture even from an emulated shell.

Local real macOS x86_64 archive installation and installed-command acceptance
passed, as did checksum generation, shellcheck, actionlint, script syntax and
deep/strict product-documentation validation. Other platform acceptance was
pending when this entry was committed; consult PR #20 for final revisions/results.
The initial Windows jobs built both architectures, then exposed unavailable
PowerShell 5.1 `Get-FileHash` discovery. `489ad35` uses .NET hashing, explicit
.NET null for the backup path, and adds malformed-mirror/token-isolation checks.
Both Linux targets and Apple Silicon passed package acceptance in that first run.
Run `39084948928` subsequently passed all six package/install jobs. A separate
macOS watch test failed when raw image evaluation lost its execution context
during reload; the assertion now uses Playwright's retrying locator property
check. Required image widths and timeouts remain intact. Final complete-suite
acceptance is recorded on PR #20. [Distribution contract](zel-native-distribution.md)
records tested OS baselines and remaining signing, broader OS, container and
release-publication limits. No existing v0.1.3 asset is replaced or new tag issued.

Prior milestone completion: PR #19 merged as `4735144`; PR/main CI `39082442519` /
`39082937391` passed all seven jobs, 317 browser cases, 124 conditional skips and
no retries. Image run `39082937383` passed all 13 jobs and published four variants
for both Linux architectures. Portable scaffolding acceptance is complete.

## Implementation record — theme preferences, prefixed video and guide accessibility

Source `f35cc92` resolves S5/S14/S15 in the hosted engine:

- **S5:** restore mode and theme independently when their controls are enabled;
  allow only known values and tolerate blocked storage. Disabled controls retain
  their configured defaults without deleting shared saved preferences. The fixed
  initialization script lives outside client-component modules and reads escaped
  data attributes. System-mode changes also update the toggle's accessible label.
- **S14:** pass the site's deployment prefix to local video and poster URLs,
  including posters for external embeds. Preserve external media URLs. Two local
  media unit cases fail against the old renderer; all four pass after the fix.
- **S15:** remove the generic label that replaced heading text; keep formatted
  h2/h3/h4 titles as their accessible names. Render the decorative hash in an
  aria-hidden span, visible on hover and keyboard focus, and keep Enter-operated
  fragment navigation.

Local evidence: 329 engine tests, all workspace typechecks, engine package build
and deep/strict product-documentation checks passed. The production basePath gate
passed Chromium and WebKit with independent mode/theme settings before hydration,
navigation/reload persistence, invalid/blocked storage, system color changes,
formatted heading role/name assertions and keyboard links, plus real prefixed
poster/video decoding. Initial new-fixture failures concerned metadata routing,
route cleanup and about:blank storage access; they were corrected and are not
evidence of fresh browser reproductions of S5. S5 was source-confirmed; S15 retains
the prior browser-tree reproduction below. Manual screen-reader checks and the
broader accessibility/SEO/performance gate remain outstanding. Consult the PR
containing `f35cc92` for the final checked revision and hosted acceptance.
[PR #21](https://github.com/mylife-inc/Zellij/pull/21)'s initial run `39087718901`
passed the new production checks but exposed an old watcher assertion expecting
raw heading text without the new decorative hash. Its accessibility snapshot
already had the correct title. The watcher assertion now requires the heading
to be visible and have that exact accessible name. The corrected development
watcher acceptance passed locally in Chromium and WebKit; the other 12 jobs
passed on the initial hosted run.

Prior milestone completion: [PR #20](https://github.com/mylife-inc/Zellij/pull/20)
merged head `434f5eb` as `937331c`. PR/main CI `39085300006` / `39085863182` passed
all 13 jobs, including six native package/install targets and Windows PowerShell
5.1/7. Each showcase run had 317 passes, 124 conditional skips and no retries.
Image run `39085863219` passed all 13 jobs, publishing four variants for both Linux
architectures and passing the published-image smoke test. The public native
release remains `zellij-v0.1.3`; no new binaries were tagged or published.

## Implementation record — page themes and Plausible analytics

Source `2823164` resolves S3's silent page-theme and analytics no-ops.

- A sections page's authored theme supplies scoped palette/structural tokens and
  presentation inside its main content. The shared header/footer and guides keep
  the site theme. Global light/dark/system mode and site overrides still apply;
  the explicit page theme wins over the site picker within its content. CSS scope
  boundaries prevent ancestor-theme presentation from bleeding into that content.
  It works without JavaScript, does not write storage, and preserves actual
  shared-header anchor offsets even with the full theme picker enabled.
- Opt-in production analytics uses the official Plausible NPM tracker 0.4.6,
  initialized once per document with automatic history handling. The domain is
  required and nonempty. Default none/dev builds do not initialize it; provider
  localhost/file and localStorage opt-out behavior is preserved. Failed analytics
  requests do not interrupt reading or navigation. No authored site was opted in
  by this change; native portable output has no analytics integration.
- The generated Rust JSON schema represents both analytics variants, and Markdown
  context generation expands both object alternatives. Product docs now state the
  actual contract, remove the unsupported Umami option and correct footer-social
  YAML. Official API checked: [Plausible tracker types](https://github.com/plausible/analytics/blob/master/tracker/npm_package/plausible.d.ts).

Local verification: 338 engine tests, all workspace types, final package build,
deep/strict docs and 10 Rust core tests passed. Four initial tests failed on the
old source: missing/empty Plausible domains and absent page-theme server output.
Production Chromium/WebKit passed computed-style/font/presentation checks, picker
on/off, overrides, system/mode changes, client navigation/reload, ordinary/guide
destinations and JavaScript-disabled rendering. The real tracker passed intercepted
pageview-count/domain/history, disabled/ignored/localhost and blocked-request checks.
Both dev browser engines verified analytics does not initialize. No real analytics
events were transmitted; live account/dashboard acceptance is not claimed. New
fixture failures (too many hero CTAs and reloading before navigation completed)
were corrected. Broad legacy-browser, accessibility/SEO/performance and native
release acceptance remain separate work. Consult the PR containing `2823164` for
the final checked revision and hosted results.

Prior completion: [PR #21](https://github.com/mylife-inc/Zellij/pull/21) merged
`1cb9d9a` as `8f10224`. PR/main CI `39088215253` / `39088773090` passed all 13 jobs,
with 317 showcase passes, 124 conditional skips and no retries in each. Image run
`39088773074` passed all 13 jobs, publishing four variants for both Linux
architectures and verifying the published-image build/serve smoke test. Latest
native release is still `zellij-v0.1.3`; repository is public/default-main.

## Implementation record — native portable math

Source `80174ac` extends the user's native documentation workflow with optional
build-time MathML. `markdown.math` in `zel.yaml` defaults to false; enabled `$...$`
and `$$...$$` expressions render using pinned `math-core` 0.9.0 and the bundled
official STIX Two Math font. No server, Node, TeX or browser math library is
required. Only math pages load the CSS; sites without equations omit its assets.
Provenance, checksums and licenses are stored alongside the embedded assets.

Source text remains available to titles, anchors/TOC, search, image alt text and
MathML annotations. Invalid/unsupported formulas and unresolved references fail
with source-file/expression diagnostics before publication. Each expression has
its own label namespace and local definitions/numbering/references. Existing
exports survive errors; watch mode handles config changes and recovery. The
CLI/docs source-build minimum becomes Rust 1.96, covered by a new CI check;
three nested-if cleanups keep the newer Clippy minimum clean. Helper-crate
minimums and binary runtime requirements are unchanged.

Local validation passes 176 Rust tests, formatting, warnings-denied Clippy,
47-route/23-example deep-strict docs checks, and real Chromium/WebKit offline and
HTTP/watch acceptance. Tests verify bundled font bytes/decoding, fractions and
matrices, escaped HTML-like text, heading/search navigation, mobile scrolling,
JavaScript-disabled rendering, structured CLI diagnostics, invalid-formula
preservation and recovery. The new browser fixture corrected theme-parameter
and viewport-settling assumptions. Consult the PR containing `80174ac` for
hosted platform/minimum-version and final merge results.

Limits remain explicit: no full TeX/MathJax parity, cross-expression equation
references, Mermaid execution, exhaustive legacy-browser or manual screen-reader
acceptance. No new native release is published. See the updated
[portable contract](zel-portable-docs.md).

Prior milestone completion: PR #22 merged as `0a52fdd`; PR/main CI
`39090482991` / `39091005544` and image workflow `39091005435` each passed all
13 jobs. Showcase passed 317 tests with 124 conditional skips on both revisions,
and production-theme/analytics/prefix, development-watcher and published-image
acceptance passed. Repository remains public/default-main and native release
remains `zellij-v0.1.3`.

## Implementation record — native portable mindmaps

Source `a140800` extends the native Markdown compiler with built-in YAML
`mindmap` fences. They generate finished SVG plus a semantic nested-list outline
without new dependencies or an external renderer. Single-root mappings, sibling
lists and scalar leaves follow the documented hosted syntax. Portable validation
adds strict string keys, no custom tags/duplicate keys, bounded YAML expansion,
and per-diagram limits of 64 KiB source, 512 nodes, 32 levels and 256-character
labels. HTML-like text stays literal in SVG, accessible names and outlines.

Source-file/diagram-number errors occur before output publication; failed
watch rebuilds retain the prior artifact and revision and recover after edits.
Only diagram pages load CSS; obsolete CSS is removed when diagrams disappear.
The active theme supplies SVG colors without redraw. Layout uses conservative
fixed-font width estimates and preserves readable sizes through scrolling.
An explicit keyboard-scroll enhancement resolves a reproduced WebKit native
overflow-scroll limitation. Without JavaScript, SVG still renders and readers
can open the complete text outline by keyboard. This is not a Mermaid runtime,
hosted-layout/palette parity or exhaustive complex-script/screen-reader support.

Local verification passes 182 Rust tests, formatting, warnings-denied Clippy,
47-route/23-example deep-strict documentation checks, Chromium/WebKit relocated
offline browser acceptance and HTTP/watch failure-preservation/recovery tests.
Browser assertions check actual SVG/text bounds, hierarchy, escaping, theme
colors, search/navigation, mobile containment, keyboard scrolling and disabled
JavaScript. Four initial native feature tests failed on the old implementation.
Review reproduced mobile page overflow from a maximum-depth/label text outline.
The follow-up contains outlines in a focused scroll region and adds both-browser,
JavaScript-enabled/disabled boundary acceptance. Consult the PR containing
`a140800` for final hosted acceptance and merge state.

Prior milestone: PR #23 merged as `9fa1866`; PR/main CI `39093178918` /
`39093718016` and image workflow `39093718039` each passed all 13 jobs. Minimum
Rust, six package/install targets, three-OS portable tests, showcase (317 passes,
124 conditional skips) and published-image acceptance passed. The repository
remains public/default-main and latest native release remains `zellij-v0.1.3`.

## Implementation record — portable Mermaid

Source `a9767fa` adds opt-in `markdown.mermaid` with a locally bundled browser
renderer. Native build/check preserve escaped source and enforce nonempty input,
64 KiB per fence and 32 diagrams per page. Mermaid syntax/layout validation takes
place in the browser, not native `check --strict`. Failed diagrams retain source
and an inline error while other diagrams continue. Strict settings, modern
MathML and a 500-edge limit are protected against author overrides.

The classic IIFE bundles diagram chunks and KaTeX, so relocated/nested `file://`
exports need no server or network. Only enabled diagram pages load the runtime;
unused assets disappear after rebuilds. JavaScript-disabled/missing-runtime
fallbacks remain readable. Theme/mode/system changes serialize redraws and discard
stale results. Review found duplicate IDs in upstream mindmap output and native
CSS selectors that would affect Mermaid; both are contained in this implementation.
Long source disclosures now scroll within a bounded, focusable region.

The reproducible generator and CI verification preserve package and upstream
embedded notices. Inspection found bundled js-yaml 4.3.0 that npm audit does not
see; an exact-module shim substitutes patched 4.3.2. The old embedded Lodash
subset excludes known affected functions, enforced by source-map assertions.
This is not a zero-advisory claim. The bundle README records provenance, hashes,
review limits and upgrade instructions. The hosted runtime is unchanged.

Local validation: 186 Rust tests, formatting, warnings-denied Clippy, bundle
reproduction, deep-strict documentation validation (47 routes/23 examples) and
production documentation build pass. Chromium/WebKit pass relocated offline
rendering, math labels, source/edge failures, security directives, unique IDs,
rapid theme/system-mode changes, search/navigation, mobile and no-JavaScript/
missing-runtime fallbacks. Real HTTP/watch tests pass opt-in/config/source edits,
failed native rebuild preservation and recovery. Consult the PR containing
`a9767fa` for final hosted acceptance and merge status. Remote diagram resources,
image discovery inside diagram source, plugins, server rendering and exhaustive
browser/screen-reader/security coverage remain outside this bounded milestone.

Prior milestone: PR #24 merged as `1bddf89`; PR/main CI `39095187808` /
`39095732531` and image workflow `39095732463` each passed all 13 jobs. Rust 1.96,
six package/install targets, three-OS portable acceptance, showcase (317 passes,
124 conditional skips, no retries) and published-image acceptance passed. The
repository is public/default-main; latest native release remains `zellij-v0.1.3`.

## Implementation follow-up — Mermaid theme font readiness

Source `ea39bfb` fixes a race reproduced after PR #25 merged: switching to Slate
while diagrams were rendering produced SVG while its body-font request was still
pending (`document.fonts.status === "loading"`). Font readiness was only awaited
before the render loop, so a later theme revision could use fallback metrics.
The runtime now loads the active revision's body font, awaits readiness and
rechecks staleness before layout. Optional font-load failures fall back rather
than aborting all diagrams. Regenerated bundle provenance stays reproducible.

The reproduction now reports no pending requests and `loaded`. Chromium/WebKit
HTTP/watch regression tests delay real font responses, switch themes during an
active render pass and verify eight diagrams draw after readiness. Both browsers
also pass focused offline Mermaid acceptance. PR #25 merged as `113283a` after
13/13 PR checks (`39097514862`); its image workflow `39098016565` passed 13/13.
Consult PR #25 for final main CI `39098016595` evidence and the PR containing
`ea39bfb` for acceptance/merge status of this correction.

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

**Resolution:** `4d659b4` implements the inspection JSON/theme contracts and
rejects unsupported modes before execution. See the implementation record
above for validation and remaining CLI scope.

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

Additional evidence from PR #9 run `39048613896`: normal production and prefix
builds passed, but the fresh Turbopack dev wrapper returned HTTP 500 before the
watcher browser assertions. IBM Plex Sans failed with
`next/font/google queries have exactly one entry` and unresolved internal font
URLs. This matches [Next issue #99114](https://github.com/vercel/next.js/issues/99114),
which reproduces both the earlier webpack null-regex error and this Turbopack
query-parsing error with Google Fonts' extensionless `/l/font?kit=...` responses.
The installed Next 16.4.2 loader still has the unsafe filename-extension regex.
The exact Google response from this CI failure was not retained, so the URL
shape is an upstream-supported explanation rather than a captured response.

[Upstream fix #100158](https://github.com/vercel/next.js/pull/100158) merged on
2026-10-15 as `98a73ead5bf711183e07398983c5c943f86328e7`. On 2026-10-17 the npm
stable dist-tag was still 16.4.2; no stable dependency update carrying that fix
was available. A fresh browser-job retry is verification of the watcher, not a
font fix. Prioritize a compatible patched release or a reproducible local-font
distribution; do not hide the failure by weakening the browser/build gate.

**Implementation:** `7963587` removes this external font-loading path using a
pinned local distribution. See the implementation record above and its PR for
verification; the historical failure evidence is retained here.

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
the renderer passes the right prefix to an existing asset. **Resolved in `f35cc92`;**
see the implementation record above for unit and production browser evidence.

### S15 — P2: guide headings lose their visible title in the accessibility tree

Observed in Chromium's accessibility snapshot while exercising development:
`## Updatedchromium` rendered visibly with the correct heading ID/text, but the
h2 and its link were both named “Link to this section.” In
`src/mdx/components.tsx:48`, the link wraps all heading text and sets that generic
`aria-label`. This replaces the text used to compute the heading's accessible
name. The same renderer handles h2/h3/h4, so heading navigation loses meaningful
section names for assistive technology. This is browser-tree evidence, not a
manual screen-reader test.

**Acceptance:** preserve each heading's visible title in its accessible name
while keeping its permalink discoverable and keyboard operable. Verify formatted
headings and h2/h3/h4 in Chromium/WebKit accessibility trees. **Resolved in `f35cc92`;**
see the implementation record above for role/name and keyboard verification.

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
  `list themes` and JSON context were advertised before they worked; the R9
  implementation record documents their correction.
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


## Implementation record — configured CLI sites (2026-10-18)

Source commit `efe8946` implements the agreed `zel.yaml` / `content/` / `site/`
layout, cwd/repository default selection, renderer-aware dispatch, output/port
configuration and site scaffolds. Legacy explicit sources remain usable. Watch
tracks sibling configuration without scanning output and preserves the last
successful build on invalid changes. See the durable context for scope and
verification; this record does not close the remaining CLI feature backlog.

The previous main CI container failure (`39109606782`) selected Podman against a
Docker-local test image. Container acceptance now pins its creating Docker
context. This is a CI fixture correction; runtime fallback policy is unchanged.

## Implementation record — CLI shortcuts and completions (2026-10-18)

Source `271493f46d3b97ceed12623524505374e3e12190` adds 22 top-level
shortcuts, `ci run` with compatible `ci publish`, and five-shell completion.
Canonical and short commands share argument definitions and handler dispatch.
This advances stage 4's usability/help work; it does not implement the remaining
image, deployment, workflow-generation, or registered stub capabilities.
Local verification: 222 Rust tests, formatting, warnings-denied Clippy, strict
49-route/23-example docs validation, Bash completion execution and Zsh syntax.
See durable context and the PR for browser and hosted CI completion evidence.


## Implementation record — static-site images (2026-10-18)

Source `5ed8fb49126ce3919abedbe532f6ec7ba63506e4` implements
`image static` / `si` for portable and hosted sources, plus renderer-aware
`idf --kind static|next`. Portable packaging preserves the native file-viewable
export; hosted packaging preserves basePath. The final image runs NGINX without
root on port 3000. Native temporary output is cleaned up on failure; existing
source and site output are preserved. Legacy Next paths and tag defaults remain.
Local checks: 226 Rust tests, fmt, Clippy, strict/deep 49-route/23-example docs.
Real-container acceptance is tracked by the subsequent PR; consult the durable
context for results. The Markdown-to-Next adapter and broader CLI backlog remain.


### Implementation record — 2026-10-18, Next image adapter

The requested `image next` / `ni` path now packages either renderer. The native
adapter preserves pages omitted from custom navigation, external sidebar links,
separate nav labels, validated links/anchors/assets, metadata and Markdown
semantics without treating plain Markdown as executable MDX. Sources and previous
output stay intact; temporary contexts are cleaned on failure. Adjacent fixes
make Next health checks prefix-aware and prevent `//page` URLs in root-mounted
guides. Source/PR and final validation evidence are tracked in ZELLIJ_CONTEXT.md.
Remaining delivery, CI and command-family work is unchanged by this milestone.


### Implementation record — 2026-10-18, saved image publication

Shared versioned image/publication settings and `publish` / `pub` now cover
static and Next images for both renderers. Intent resolution is separate from
execution, and existing image builders are reused without repeated builds.
Publication requires an explicit destination; ordinary image builds never push.
Failures stop at the failed stage, preserve source/output, remove temporary
contexts, and distinguish unsuccessful registry operations from invalid content.
Tests cover malformed settings, override precedence, no-tool dry runs, missing
runtime, no push after failure, exact destination and cleanup. CI also publishes
to a disposable loopback registry, removes the local image, pulls it back, and
runs the existing HTTP/browser acceptance against the retrieved static and Next
artifacts. Source and completed acceptance are tracked in the durable context.
The provider/Netlify and remaining CLI backlog are still open.

Fixing source for the saved-publication milestone:
`5b8bf91d731af36321f725f4dc4590519e1ba480`. Local checks passed 237 Rust tests,
formatting, warnings-denied Clippy and strict/deep docs validation. Use the
associated PR's latest checks for real registry and all-platform results.


### Implementation record — 2026-10-18, native Netlify delivery

The first direct provider adapter now shares typed intent and build-once delivery
with saved publication. It supports native portable exports and hosted exports
mounted at their configured prefix, preserves previous output, uses environment
credentials, and distinguishes preview from explicit production. Local mock API
tests cover manifests/bytes, deduplication, errors, timeouts, redirects and no
publication after failed builds. CI adds a real hosted export through the same
upload path to a loopback API. This is protocol/renderer acceptance, not proof of
a live Netlify account deployment. Source/PR/check evidence is kept in the durable
context. Other providers, saved CI generation, ship and the remaining CLI backlog
are still open; no binary release or public documentation deployment is included.

Netlify implementation source: `37631df8dbb2455f3e52a4756bf1a50fb57708be`,
[PR #33](https://github.com/mylife-inc/Zellij/pull/33). Local checks pass 245 Rust
tests, formatting, warnings-denied Clippy and strict/deep docs validation.
Use the PR's latest checks for final hosted-container and all-platform results.


### Implementation record — 2026-10-18, configured CI workflows

Saved CI settings and both generation forms now produce a Git-root workflow
using renderer-aware build and native delivery operations. Pull requests build
without publication; non-PR gates respect branch/tag/schedule intent. Generation
writes local files by default, with explicit stdin-only Netlify provisioning,
serialized values, path-filter escaping, destination-link rejection and atomic
writes. Legacy options and dispatch aliases remain accepted. The old unpinned
installer/check-only PR/Node uploader paths are replaced. Local regression,
actionlint and generated-command execution evidence is in ZELLIJ_CONTEXT.md.

Release compatibility is deliberately unresolved operational work: current
0.1.3 is incompatible; generated files must pin a published compatible 0.2.0+
release before enabling them. No release or live provider deployment was made.
Other providers and the remaining CLI backlog remain open. Source/PR evidence
will be appended without removing the historical generator findings.


Workflow implementation source: `8904cdd11e17e17d2cb59bb481e9220741996a12`.
Final local follow-up checks passed the CI integration, core/scaffold suites,
warnings-denied Clippy, formatting and strict/deep documentation validation.
Cross-platform and real-container acceptance remains the PR merge gate; inspect
the GitHub checks associated with this source for final-head results.


### Implementation record — 2026-10-18, configured import and ship

The configured CI milestone merged in PR #34 at `4bc5ef6`; final source run
39159674967, merge CI 39160901027 and base-image run 39160901001 passed.

Configured imports now use explicit --layout site, preserving the default raw
content layout for existing convert/migrate scripts. Ship prepares an import,
saves explicit nonsecret Netlify/image intent, and executes shared delivery once.
Failures after import retain the converted site and report the completed stage;
conversion warnings remain visible. Source/output containment and complete
preflight apply to both layouts. Equal-confidence converter selection no longer
depends on registration order, and imported Windows junctions cannot bypass the
existing symlink boundary. Missing runtime/credentials do not trigger installation,
startup or account creation. Local and forthcoming CI evidence is recorded in
ZELLIJ_CONTEXT.md; release publication and the remaining CLI backlog stay open.


Import/ship implementation source: `0b59859` in
[PR #35](https://github.com/mylife-inc/Zellij/pull/35). The pull request records
final-head platform and container acceptance; consult its checks for merge status.


PR #35 run 39162890191 exposed a Windows configured-import path bug: joining
content/ with a generated relative path inserted a backslash rejected by the
portable write-plan validator. Prefixing the already-validated relative path
with a literal slash fixes the implementation; the Windows junction fixture also
uses OS-native path components for cmd.exe. All-platform rerun is required.


### Implementation update — 2026-10-18, editable Next.js wrappers

PR #35 merged at e7ff7e01bd291dd58f387f0c865c1111d8dfeb70. Its final source
cf8296b23f816c805575a77d0468e29c05b89b1e passed all 13 jobs in CI 39163178585;
merge CI 39164568012 and base image 39164567998 also passed. This closes the
pending Windows configured-import and final-head evidence above.

Current source implements `new app` / `na`. Scaffolding uses no Node, npm or
container runtime. Existing hosted content is selected by --site-root or normal
site discovery and referenced at its canonical absolute location, without copying.
--with-docs instead creates a configured hosted starter under docs/zellij in the
new app. Destination defaults to ., has no force override, and must be empty
apart from noncolliding dotfiles. Engine/content overlap and link protections
apply before writes. Portable and legacy marketing content are rejected with
explicit guidance; image next still supports portable adaptation.

A built local engine package is required because npm distribution does not exist.
--engine selects it; the sole implicit location is ./engine/packages/zellij in
the working directory. Identity, version, wrapper-config capability and required
CLI bundles are checked. npm file dependencies retain literal path punctuation;
percent encoding failed actual installation and was corrected. Windows extended
canonical prefixes are normalized for npm. Node/npm are required only to run the
created app. --name, --port and comma-separated --features are documented.

The engine initializer exposes wrapperTemplate(); scripts/prepare-app-scaffold.mjs
embeds its core files, dependency/override versions and feature metadata in Rust.
CI checks drift. Node initializer and Next images continue using that definition.
Versioned zellij.config.yaml contains version: 1 and content only; it is separate
from a documentation site's zel.yaml and content/site.yaml. Check, preparation,
Next bootstrap and rendering now agree on that selector. The previously defaulted
check content path was corrected. Unversioned legacy config keeps its validation.

Local validation: 259 workspace Rust tests passed; focused new-app tests and
warnings-denied Clippy passed after the npm-path/compatibility refinements.
Engine tests, workspace typechecking, engine build verification, template drift,
formatting, changed-workflow actionlint and strict/deep docs validation passed.
The isolated acceptance script installed the real copied engine through npm from
a path containing spaces/#/%, validated starter/external content, built Next with
Turbopack, typechecked, and exercised Chromium navigation, reload, assets, search,
prefixes and 404s. No workspace node_modules shortcut was used for that app.
Full-workflow actionlint also reports pre-existing shellcheck SC2046 warnings in
base-image.yml; the changed ci.yml passes its check.

CI adds native scaffold integration and isolated installed-wrapper acceptance
on Linux, macOS and Windows. Final PR/source/platform/container evidence will be appended.
No local daemon was started, live provider/registry publication performed, or
native release/public docs deployment made. Latest public native release remains
zellij-v0.1.3; all current CLI additions still need a compatible release. Remaining
work includes fmt/watch, portable dev, hosted view, other providers and maintenance.


Editable-wrapper implementation source: `3877db8fc6e68bc4ceee94534cd95f0305df0c54`,
in [PR #36](https://github.com/mylife-inc/Zellij/pull/36). That PR tracks final-head
Linux/macOS/Windows installed-app acceptance and existing container checks.


### Implementation update — 2026-10-18, YAML formatting and validation watch

PR #36 merged as d773c1832918a0c426513e10949367105d9e00b6. Final head
3b523091dd37af96bb6cc167681adf166d0df835 passed all 13 jobs in CI 39170572087.
Merge CI 39171833213 and base image 39171833271 also succeeded, closing the
previous editable-wrapper milestone's pending platform/container evidence.

Current source implements fmt and watch with configured-site discovery. Fmt
selects hosted YAML plus zel.yaml, or only zel.yaml for portable sites, excluding
attachments/generated output and refusing links/junctions. It uses pretty_yaml
0.6.0 / yaml_parser 0.3.0 syntax trees, schema ordering and compact collections,
with comments/anchors protected from reordering. An upstream normalization of
|+ trailing blank lines was caught by regression tests: authored block scalars
are now restored with adjusted structural indentation, and parsed-value equality
is required before writes. Staging closes file handles before publication, keeps
permissions, and checks concurrent edits. Invalid late input writes nothing;
replacement is atomic per file, not filesystem-wide on storage failure.

Watch reuses portable checking and the hosted engine. Byte snapshots detect
same-size/timestamp-preserving edits; debounce and checks are serial. A configured
session pins layout and renderer and recovers after invalid content/config edits
without scanning unrelated repository/output paths. Ctrl-C waits for current
validation, then exits0. Hosted runtime failure is startup exit4; portable checks
and formatting never probe external tools. No source/output/build/publication
side effects are part of validation watch.

Local tests cover formatting scope, check-only writes, late errors, permissions,
low descriptor limits, multiline values, comments, scalar types and idempotence.
Disposable copies of showcase, zellij-docs and codeseal-docs remain strictly/deeply
valid after formatting; Markdown/media stay byte-identical. The actual portable
watch process recovers from initial bad content, restored-size/timestamp edits,
malformed/deleted config and changed renderer, then exits cleanly on Ctrl-C.
All 265 workspace Rust tests, warnings-denied Clippy, formatting, changed-workflow
actionlint and strict/deep local documentation validation passed. CI adds formatter
corpus and portable-watch acceptance on Linux/macOS/Windows, including Windows
junction and console-signal coverage, plus hosted watch with the CI-built engine.
The portable matrix timeout is 30 minutes to allow cold installed-wrapper builds
and these added checks. Final source/PR/platform evidence will be appended.

No local daemon was started or live provider/release/public docs publication
performed. Public native release remains zellij-v0.1.3. Remaining work: portable
dev, hosted view, additional providers and maintenance, plus compatible release
and documentation publication. Historical audit findings above remain evidence,
not a claim that the product is complete or bug-free.


Formatting/watch implementation source: `47fbcfbb17d435d584a758093415160d42513bc7`
in [PR #37](https://github.com/mylife-inc/Zellij/pull/37). That PR tracks final-head
Linux/macOS/Windows formatting/watch and real-container validation acceptance.


PR #37 run 39175688739 caught a Windows formatting publication failure: the
contained-path validator expects portable relative separators, while WalkDir
returns OS-native paths. Splitting actual path components and joining with /
fixes that preflight without reinterpreting Unix backslashes. Windows junction
refusal and no-write tests already passed; all-platform acceptance must rerun
on the fix before merge.


### Implementation update — 2026-10-18, portable development and hosted preview

PR #37 merged as b2771c81a2e69871abb69557c16b999dcaddd93c. Final head
aaad79fa7b9aee3ab9073bcca8d93fcf54ab11cf passed all 13 jobs in CI 39176083582,
including the Windows formatter correction. Merge CI 39177511838 and base-image
run 39177511802 also succeeded. This closes the previous pending evidence.

Current source makes portable dev equivalent to view --watch --no-open, with
native validation/building, configured discovery/output/port, browser reload and
no external-tool probe. Hosted dev retains Next.js development; port 0 now fails
with guidance to use view --watch --port 0 instead of advertising an unusable URL.

View/v now supports hosted static exports, with a native loopback HTTP server
and an optional browser. The container engine returns a staged flat export and
its actual resolved basePath. Index/prefix validation and a before/after source
snapshot precede publication. Missing runtime, occupied port, export failure,
changed source during build or cancellation preserves existing output. Metadata
is removed before publication; ordinary build and deployment layouts are retained.

The shared watch worker serializes/debounces builds and serves the last good
site after errors. Hosted watches pin renderer/layout and basePath. Config or
prefix changes show diagnostics; restoration recovers. Changed inputs during a
hosted build discard its stage and trigger a new build. Ctrl-C stops HTTP and
waits for the active build to finish cleanup, discarding its result; it does not
immediately kill a long container build. Output/port remain fixed until restart.
The reload client is injected only into HTTP responses. Portable exports retain
file:// support; hosted exports continue to require HTTP serving.

Local validation: all 265 Rust tests and warnings-denied Clippy passed on the
final implementation, including the hosted-dev port-zero guard.
Chromium/WebKit portable preview and dev acceptance passed. A standalone Rust
runtime-protocol fixture exercises real native CLI orchestration and Chromium:
prefix serving, browser readiness, reload/error recovery, configuration recovery,
stale-build refusal, initial/rebuild cancellation and previous-output preservation.
That fixture does not replace the real container renderer. CI now runs it on
Linux/macOS/Windows and adds a separate actual hosted-engine browser test.
Strict/deep local documentation validation and changed-workflow actionlint passed.

No local daemon was started; no live provider, native release or public docs
publication occurred. Public native release remains zellij-v0.1.3. Remaining work
includes other deployment providers, maintenance commands, additional dev flags,
compatible release distribution and documentation publication. Final source/PR
and platform/container results will be appended.


Portable-dev/hosted-preview implementation source:
`e9e47deb23f68ee9dafcada6b4fe657377e4db05`, in
[PR #38](https://github.com/mylife-inc/Zellij/pull/38). That PR tracks the final-head
platform tests and real-container browser validation before authorized merge.


PR #38 run 39180630629 caught a real Next export routing defect in the native
server: a guide/ directory for child routes or RSC payloads hid guide.html when
that directory lacked index.html. Resolution now falls through to the flat HTML
page in that case, preserving existing directory-index redirects and containment.
Nine HTTP server regression tests and warnings-denied Clippy pass. The real-engine
browser test now runs earlier in CI, verifies the initial home redirect, and
selects the preview error notice separately from Next's accessibility announcer.
Final platform/container checks must rerun on this fix before merge.

Local confirmation also served an existing real Next export through hosted view
with the reload client injected. Chromium passed parent-route loading, navigation,
reload, hydration and search without a local container daemon. The hosted CI
search check waits for the client handler to attach instead of sending a shortcut
immediately after receiving server-rendered HTML.


PR #38 run 39181963487 passed the real exported home redirect, parent/child
navigation and images with the routing fix. Its search step caught a fixture
omission: hosted nav.search is opt-in, and this fixture had not enabled it.
The fixture now explicitly sets nav.search: true. No renderer behavior changed
for that test correction; final-head acceptance must still complete.


PR #38 run 39182453254 passed real hosted preview end to end. The Windows
orchestration fixture instead selected the runner's installed Docker client,
exposing an existing engine bind-path defect: Rust canonical Windows paths use
\\?\ prefixes that Docker rejects. Content and output mounts now use standard
drive/UNC syntax; Unicode, spaces and Unix backslashes remain intact. All 22
engine tests, Clippy and local hosted browser/lifecycle acceptance pass.
The native fixture now removes case variants of PATH/endpoint keys, explicitly
selects an isolated Podman executable, verifies that fixture performed the build,
and rejects verbatim mount paths. CI runs it before the slower wrapper tests.
Final-revision Windows and full-container validation are still required.

## Maintenance implementation — 2026-10-18

PR #38 completed at b83de924 after all 13 PR checks in run 39183818152 passed.
Post-merge base-image 39185352421 succeeded; CI 39185352441 succeeded after a
failed-job rerun cleared an unchanged Windows ARM PowerShell CLR runtime crash.

The next maintenance increment adds native version/schema/engine reporting,
explicit engine pulls through the shared runtime resolver, and verified native
self-upgrades. Missing/corrupt checksums, unexpected archive entries, wrong
CPU/OS, wrong executable version and cancellation preserve the installed binary.
Concurrent upgrades are serialized with an OS lock. Windows keeps the previous
executable as a recovery copy; Unix replaces by rename. Package creation now
suppresses macOS AppleDouble metadata after local acceptance caught an extra
._zel entry that violates the one-executable archive contract. All six native
package jobs execute real upgrades against isolated loopback release fixtures.
Final source commit and CI results will be appended; this is not a release or a
claim that advanced CLI/provider work is complete.

Maintenance source d4d8c01867f68bd3a439f93a47e9989b295bcdcf is in PR #39.
Subsequent dependency review selected patched zip 9.0.5 for the newly added
archive reader after RustSec's October 17 extraction advisories (0367–0369).
No unsafe extraction helper is used; exact-entry checks and explicitly created
output files remain the extraction policy. Final-head platform validation is
required after this dependency update.

Review also identified a stale-process downgrade race: a process started before
another replacement could compare the requested release against its old compiled
version, then snapshot a newer executable. Upgrade now verifies the on-disk
version under the installation lock before downloading. Native acceptance also
checks actual concurrent updater refusal during a stalled download.


### Eject and features implementation — 2026-10-18

The native CLI now registers `eject [site] --out <destination>` (or --with-docs)
and `features <selection> [--site-root <path>]`. Eject copies the selected engine
distribution's /app and /opt/zellij using a stopped runtime container. It stages,
validates, cleans up and publishes only into new/empty output. Vendored engine
sources use file:vendor/zellij; tsconfig excludes those source/test copies from
the wrapper's own typecheck. External hosted content is referenced; image starter
content moves to docs/zellij/content with zel.yaml. The historical 'no npm'
promise was wrong: the distribution includes sources, not installed dependencies.
Users run npm install afterward. Runtime discovery/prerequisites remain shared.
Cancellation/copy failures attempt cleanup; cleanup failure reports the container
name and does not publish. No local Docker daemon was started.

Features maps maths/diagrams to portable markdown.math/markdown.mermaid switches
or to a compatible editable wrapper's direct feature packages. Portable edits
preserve YAML comments and unrelated spelling; alias/anchor targets are refused.
Wrapper changes retain unrelated manifest data and lockfiles, with npm install
left explicit. Hosted content without a wrapper is refused with useful guidance.
The feature dependency mapping is shared with the generated engine template.

A pre-existing engine limitation was confirmed: npm optionalDependencies may be
installed even when a wrapper has no direct feature requirements. Hosted `none`
therefore removes direct requirements, not rendering capability or all installed
packages. Strict exclusion/explicit hosted capability switches remain follow-up
work. The current documentation and historical spec corrections state this;
portable toggles do control rendering. No zero-bug/completion claim is made.

Local checks: 277 Rust tests, warnings-denied Clippy, Rust formatting,
actionlint for the changed workflow, and deep/strict documentation validation
(55 routes / 53 guide pages) passed. Native runtime-protocol acceptance exercises
selected sources, digest overrides, movable engine references, feature changes,
nonempty output, failed copy, incompatible engine, links and real Ctrl-C cleanup.
CI adds the same extraction acceptance on Windows/macOS/Linux and a real-container
eject + npm install + validation + production build + typecheck/browser check.
Final CI evidence and merge status must be refreshed before relying on this work.
Remaining work includes other providers/advanced options, the hosted dependency
limitation, compatible native releases and documentation publication.


### Hosted feature selection follow-up — 2026-10-18

This supersedes the hosted limitation documented in the eject/features milestone.
Editable wrappers now save package.json zellij.features explicitly; native and
Node scaffolding/setters share the same feature map. Fresh wrappers default to
none, while legacy wrappers without metadata retain installed-package detection.
The engine uses optional peers rather than optionalDependencies, so fresh none
wrappers omit math/Mermaid libraries. Selected missing packages fail the Next
configuration step with npm install guidance. After a rebuild/dev restart, none
disables rendering even when packages remain installed for another consumer.
Native mindmaps remain enabled. Engine-scoped loaders and aliases support both
Turbopack and Webpack without intercepting unrelated app imports of those libraries.
The native compatibility check refuses engines without featureSelectionVersion: 1
before changing a wrapper. No npm/lockfile mutation or package deletion is implicit.

Fresh installation also exposed a hidden dependency: the engine's source types
needed d3-hierarchy/d3-shape declarations previously supplied through Mermaid.
Those declarations are now explicit engine dependencies, keeping mindmap consumers
independent of optional diagrams. Docker distributions carry the shared selection
helpers and typed loaders. CI adds standalone fresh-none, enabled-all and retained-
packages-none browser/build/typecheck acceptance.

Local checks: 364 engine tests, 277 Rust tests, warnings-denied Clippy, Rust
formatting, workspace typechecks, engine packaging, generated-template consistency,
extraction/cancellation acceptance, workflow lint and strict/deep documentation
validation passed. Final browser acceptance and commit/CI evidence follow below.
Other providers, advanced CLI options, compatible native releases and published
documentation remain separate unfinished work.


Final local hosted-feature acceptance passed: a standalone wrapper installed
without Mermaid/remark-math/rehype-mathjax and built/typechecked/rendered with
Turbopack; enabling all installed them and rendered real MathJax/Mermaid output;
switching to none with packages deliberately retained disabled both under Webpack.
Native mindmap SVG and browser-error checks passed in every state. Each npm state
is checked in a fresh Node process, avoiding stale negative package lookups.
No locked dependency versions changed. Source commit and hosted CI evidence will
be recorded after PR validation; no new native release or manual deployment occurred.


Hosted-feature source c27c64d3f4d1cfc3315383a83b9d43ef17d9c28a is reviewed in
[PR #41](https://github.com/mylife-inc/Zellij/pull/41). All local acceptance above
passed. Refresh the PR's final head/checks and merge status before relying on it.
The previous PR #40 post-merge CI 39200372898 and Base image 39200372897 both
completed successfully. Public native release remains zellij-v0.1.3 (2026-08-13).


PR #41 CI run 39202250897 passed the new standalone feature acceptance and all
container/image/export checks through section-template rendering. Its final
conversion check exposed a test-harness ESM/CommonJS interop error when bundling
the new shared config helper (dynamic require of node:fs). The conversion helper
now supplies Node createRequire, matching the production CLI bundle. Local MkDocs
and Docusaurus conversion acceptance passed after this correction, including
rendered routes, heading fragments and asset bytes. Final CI must be refreshed
on the fixing head; the earlier run is not a successful completion record.


### Shared working directory and site selection — 2026-10-18

Previous PR #41 completed on source 7a2e35322c9b058041e5d367665a769c6b5d004f:
all 13 jobs passed in run 39204069716. Merge 02eaae86132c68c21a450bfc90637c4e5b7e05bd
also passed CI 39205838063 and Base image 39205837958. This supersedes the
pending conversion-harness evidence above.

Fixing commit 58b7129ea18f4add46154fe8b123edc383626152 adds shared --cwd and
--site-root, closing their outstanding contract gap. Selection is normalized into
existing command arguments; legacy CI --content and positional paths still work.
Conflicting or unsupported selections fail before effects. Repeated global path
flags across subcommands now fail instead of silently keeping the deepest value.
CLI paths resolve from --cwd, while configuration paths keep their own bases.

Verification: 284 Rust tests passed, including subprocess regressions for nested
Git roots, Unicode paths, output locations and rejected writes. Warnings-denied
Clippy, formatting and strict/deep documentation validation passed (55 routes,
53 guide pages, 23 examples). CI already runs the affected native suites on all
three OS families; refresh the PR's final head/checks before claiming hosted
validation. No new native release was published. Explicit --config, remaining
advanced options, other deployment providers and documentation publication remain.


### Explicit config-file selection — 2026-10-18

PR #42's final source 1bb7e04eed7917bbb8cdc4a2166dc671156d21e5 passed all
13 CI jobs in 39207701478 and merged as 4cae19f628f8c8665a747452a8d56da8e20c1f2d.
Its Base image 39209473109 passed; post-merge CI 39209473176 was still finishing
when this record was prepared.

Fixing commit 1653d84afb2b7ad065e0eb00c60ddd8d700c49db closes the explicit
--config contract gap. Shared SiteInput preserves the selected file across
renderers, commands, previews, authoring and generated workflows. Invalid/deleted
files fail instead of falling back to zel.yaml. Regression coverage includes
invalid neighboring defaults, Unicode/spaced filenames, JSON diagnostics, config
edits, hidden legacy config watch tracking, output preservation, unsupported or
ambiguous selections, symlinks and CI quoting/delivery variants.

All 290 Rust tests, warnings-denied Clippy, formatting, changed-workflow lint and
strict/deep docs validation passed locally. Portable preview/browser/watch,
named validation watch, hosted preview fixture and standalone named-config
wrapper installation/build/typecheck/browser checks passed. Cross-platform and
real-container evidence must be refreshed on the final PR head. No new native
release, local Docker startup or manual live deployment occurred. Output controls,
other providers, remaining advanced/converter coverage and publication remain.


### Shared output-control implementation — 2026-10-19

PR #43 merged as 8912aee8f7ed037fe8b491a5fdf5d916fe997637 after all 13 CI
jobs passed on ea6ada4e2b77b86ecf014cfa1f7291d50a2f06ed (39211231453).
Its main CI 39213225423 and Base image 39213225268 both passed; PR #42's
previously pending main CI 39209473176 passed as well.

Fixing commit f7d141441652cc951a0f75ae98d83a672b1181f9 implements shared --quiet/-q,
--verbose and --no-color with alias/help/completion support. Quiet retains data,
errors/warnings and requested reports; verbose adds stderr diagnostics without
raw arguments/env; color filtering covers parser errors and child output.
NO_COLOR is honored, quiet/verbose conflict before work, and validation/output/
publish behavior is unchanged. Hosted check keeps its complete mixed report.
Child output uses independent file handles and anonymous temporary spooling,
with 64 KiB per-stream failure tails and normal cancellation/cleanup ownership.
Spooling consumes temporary disk while the command runs. See contract stage 8c.

Local: 299 Rust tests plus final focused output regressions, warnings-denied
Clippy, formatting, workflow lint and strict/deep docs validation passed. Actual
quiet preview/HTTP/watch/recovery/Ctrl-C, validation watch and no-color hosted
preview with an isolated runtime/browser passed. CI adds cross-platform output
acceptance, real no-color preview and quiet/no-color ejection. Refresh final PR
checks before treating the source as merged/cross-platform accepted. Additional
providers, remaining advanced/converter scope and native release/docs publication
remain; no local Docker startup or manual live deployment occurred.


### Image platform selection — 2026-10-19

PR #44 merged as 2ada5fe0ad1e1227c70eb963746cca929fc72af5 after all 13 CI
jobs passed on 52cc3620cdfbdff5f8db4d443abe9a925619cbb4 (39216062489).
Its main CI 39218117100 and Base image 39218117079 both passed.

Fixing commit c9db8dfc1edbe7e702f7585e2680fbceb002435a implements single-platform Linux
image selection (linux/amd64 or linux/arm64) across image static/next/build,
si/ni, build --target image, saved publication/deploy, ship --image and CI.
Optional image.platform supplies the default; CLI overrides it. image.name is
now optional for local image builds, while publication still requires an
explicit destination. Plans show the platform, ship retains it in zel.yaml,
and generated CI freezes it in both jobs. A follow-up uses native ARM64 runners
for generated workflows, avoiding known QEMU/Node failures in release builds.
HTML/Netlify flags and unsupported/combined platform values fail before work.

Podman builds omit Docker's unsupported --progress option. nerdctl pushes use
--all-platforms to preserve a foreign single-platform local image. Build failure
keeps diagnostics, explains base/emulation requirements and never proceeds to
push. No local emulator/service is installed or started. See contract stage 9a.

Local: 304 Rust tests in 38 suites, final focused delivery tests, warnings-denied
Clippy, formatting, workflow and generated-workflow lint, and strict/deep docs
validation (55 routes, 53 guide pages, 23 examples) passed. A first focused run
had one 3-second fake-runtime probe timeout; the complete run with four test
threads passed. CI adds real ARM64 portable build/push/pull/HTTP acceptance and
an AMD64 override. Refresh final PR checks before claiming hosted acceptance.

Remaining: combined multi-platform manifests, further advanced options/providers,
conversion fidelity, matched native release and documentation publication.
Foreign-architecture hosted/Next execution is not covered by the small ARM64
portable acceptance fixture; dispatch for all renderer/kind pairs is covered.
No manual live provider/registry deployment or native release was performed.

Follow-up be57372ec7e32c4edad9bce538ac78ec9c914d5f selects native ARM64 generated-workflow runners.
All nine workflow tests, scaffold tests, generated workflow lint, Clippy,
formatting and strict/deep docs validation passed after this refinement.


### Temporary build/image features — 2026-11-02

PR #45 merged as 42bf4c906d1ca6be64940da8aaddfb9d63f1192c after all 13 CI
jobs passed on 5c75428ca495a0afcae722a678607cafb6e4c368 (39220991457).
Refreshed on November 2: main CI 39223399959 and Base image 39223399906 both
passed. The latest public native release remains zellij-v0.1.3 (August 13).

Fixing commit aa179631e7371d9c7185a07ec1929452e2262af5 adds --features to build/b,
image static/next/build, si/ni and hosted image dockerfile. Accepted complete
selections are maths, diagrams, maths,diagrams (either order), all and none.
The override does not edit authored configuration or content. Portable builds
apply it in memory; hosted exports and image recipes apply it to the disposable
wrapper before npm install. Explicit none overrides an enabled base image.
Portable Next adapters retain the selected rendering features and install the
matching dependencies, including an explicit none for an empty selection.

A shared typed parser rejects malformed/unknown/duplicate selections before
work. Hosted feature selection requires featureSelectionVersion 1 from the
engine manifest; older engines fail clearly instead of silently mishandling
none. Previous exports survive build/compatibility failures. Portable static
Dockerfiles take prebuilt HTML and direct users to build/si for feature changes.
Preview/dev, delivery/ship and generated CI retain saved configuration or engine
image variants; this temporary flag is not implemented for those commands.
See contract stage 9b and the updated build/image/features references.

Local verification: all 310 Rust tests in 38 suites passed. After the final
engine compatibility guard, all 22 engine tests and 9 static-image tests passed;
warnings-denied workspace Clippy, formatting, workflow lint, script syntax and
strict/deep documentation validation passed (55 routes, 53 guide pages,
23 examples). Hosted CI now exercises all/none folder exports and static images,
enabling features in a hosted Next image, and refusal of an incompatible engine.
Cross-platform and actual-container results are pending for this branch; refresh
the final PR head/checks before claiming hosted acceptance or merge completion.

Remaining: combined multi-platform manifests, other advanced options/providers,
conversion fidelity, matched native releases and documentation publication.
No local Docker daemon was started, native release published or manual live
provider/registry deployment performed for this milestone.


### PR #46 dependency and schema compatibility follow-up — 2026-11-02

[PR #46](https://github.com/mylife-inc/Zellij/pull/46) is open. Its first run,
41500959209 at 5dee8187c858c3fac3cfc7f509d41cd14f8a97ce, failed npm audit:
Zod 4.4.3 is affected by [GHSA-mrhx-c7xc-m3w5](https://github.com/advisories/GHSA-mrhx-c7xc-m3w5).
Fixing commit 1c9374665c64bff29bfd76054c9d28a75f5f9134 raises the engine dependency
and lockfile to Zod 4.6.7. npm audit --audit-level=low now reports zero known
vulnerabilities as checked on November 2; this is a dated audit result.

The update exposed primitive unions emitted as JSON Schema type arrays instead
of anyOf. Section template generation now handles those arrays, and Rust field
descriptions retain their alternatives. The generated schema was refreshed;
text context and generated YAML starters remain unchanged. Regressions exercise
actual stats-band values and timeline dates, alongside every section starter.

All 365 engine tests and 311 Rust tests (38 suites), workspace typechecks,
engine packaging, warnings-denied Clippy, formatting and strict/deep docs
validation passed locally. Final CI on the updated PR head remains required
before merge; neither the initial failing run nor local checks establish hosted
acceptance. Authentication works with XDG_CONFIG_HOME=/Users/hamid/Dev/config.


### Preview/development feature selection — 2026-11-02

PR #46 merged as eea28d6053622f9d692d6b56aa4150492d9f0f21 after all 13 CI
jobs passed on e5d619687e6460bb454a06a7a722715998bc9378 (41502232990).
Both post-merge runs now passed: CI 41506317359 and Base image 41506317357.
The latest public native release is still zellij-v0.1.3 (August 13), refreshed
on November 2; source capabilities below are not a claim about that executable.

Fixing commit 6ab08a3bcee51133a5fa176d3af2eb5bfb5bd554 adds --features to view/v
and dev for both portable and hosted sites. Typed complete selections match
build/image. Overrides survive every watched rebuild, portable configuration
edit and recovery, without changing authored files. Omitting the flag on a
later invocation restores saved/default selection. Serve still serves prebuilt
output and does not accept rendering options. See contract stage 9c.

Hosted dev and preview apply the override before dependency installation, using
the existing engine compatibility guard. Hosted export/dev dependency caches
now include the canonical explicit feature selection, preventing a build with
another selection from uninstalling a live dev session's packages. Equivalent
all and maths,diagrams share a cache; omission and none remain distinct. Old
caches are not deleted. Hosted dev now distinguishes content-validation errors
from dependency/startup failures instead of reporting all failures as bad content.

Local verification: 312 Rust tests in 38 suites, warnings-denied Clippy,
formatting, workflow lint, JavaScript syntax and strict/deep docs validation
passed (55 routes, 53 guide pages, 23 examples). Actual Chromium preview/watch
acceptance covers config edits, rendering, failure preservation, recovery,
source immutability, default restoration and Ctrl-C. Hosted orchestration with
an isolated runtime fixture also passed, including feature propagation on
rebuilds and cancellation. An initial full Rust run had one existing fake-runtime
probe timeout while other checks were active; the isolated full two-thread run
passed. Do not treat this as a runtime-probe implementation fix.

CI now checks actual hosted preview math/Mermaid rendering through rebuilds,
then explicit none in a subsequent preview; hosted development checks both in
a browser before and after edits. Final cross-platform/container CI and merge
are pending for this branch. No local Docker daemon was started or native release
published. Remaining work includes delivery/CI temporary controls, advanced
build options, combined image manifests, providers beyond Netlify, conversion
fidelity, and matched native release/documentation publication.


### PR #47 development-origin regression — 2026-11-02

First CI run 41511951845 at c98e226e565da6dcd1c9b5eec263fb090fff6b6a
passed 10 jobs, including real hosted preview rendering and five native package
targets. The CLI job failed the new hosted dev Mermaid browser check. Both
macOS ARM jobs were cancelled before any step ran because GitHub reported
hosted runner capacity constraints; these cancellations are not test results.

The browser check exposed an existing hydration failure: Next binds 0.0.0.0
inside the container, but the engine excluded loopback addresses from allowed
origins. Opening 127.0.0.1 blocked the HMR connection and left client components
unhydrated. This was reproduced locally with the real Turbopack dev server.
Fixing commit b23e1d417dd93f2d708f5a4aa86c28a3b14033bf allows the exact IPv4
and IPv6 loopback hosts (127.0.0.1 and [::1]). Direct wrapper browser acceptance
now uses the container's wildcard binding and checks real math/diagram output,
editing, search, recovery and origin enforcement. Requests from unrelated
hosts, including a hostname resembling a loopback IP, still return 403.

After the fix, real local Chromium/Turbopack acceptance and all 365 engine tests
passed. Final CI on the updated PR head, including the actual container and
both macOS ARM jobs, is still required before merge. No local Docker daemon
was started. PR: https://github.com/mylife-inc/Zellij/pull/47.


### Rendering features in delivery and CI — 2026-11-02

PR #47 merged as 42641950fdcfd75d43737a1f40ecbdf64a457a48 after all 13 jobs
passed on final head 27415a15a1b8159d3fab19fdd5e18c84051cdb26 in CI run
41517322451. This includes the actual hosted-development regression and both
macOS ARM jobs. Post-merge Base image run 41520378666 passed; CI 41520378824
had 12 of 13 jobs passed with CLI still running at this checkpoint. Refresh that
run before relying on a final post-merge result. Latest public native release
remains zellij-v0.1.3 (August 13), refreshed on November 2.

Fixing commit bf78d8346b5149668d03b7773fadc73acf2b9d26 extends the shared typed
--features selection to publish/pub, deploy/netlify, ship and generated CI.
Delivery forwards the invocation-only selection to one portable/hosted static
or image build. Dry plans display it. Source, saved configuration and previous
local exports remain unchanged. Failed builds do not publish. Ship retains its
existing destination/platform settings but not this override, and explains how
to repeat the selection on a later publish after success or failure.

CI generation freezes the canonical selection in verification and publication
build commands for every supported delivery form, including named configs,
platform selections and legacy image generation. Validation and prebuilt image
push commands do not receive rendering flags. Regenerating without the flag
removes the override. Dispatch does not accept it. Existing compatible-release
pins, event gates and explicit credential provisioning are unchanged.
See contract stage 9d and updated publish/deploy/ship/ci/features references.
The deployment reference's stale claim that CI ignores saved delivery settings
was also corrected.

Local verification: all 315 Rust tests in 38 suites, warnings-denied Clippy,
formatting, script syntax and strict/deep docs passed (55 routes, 53 guide pages,
23 examples). Ten generated workflows passed actionlint. Tests execute generated
portable commands, inspect selected HTML uploaded to a loopback API, cover all
renderer/image combinations, and retain ship/config/output failure guarantees.
Hosted CI now checks features in Netlify export/upload, portable Next image
publication, and imported static/Next images through browser rendering. Final
cross-platform/container CI and merge are pending for this branch.

No local container daemon was started, native release published, real provider
site deployed or external registry image manually published. Remaining work:
advanced build options, combined multi-platform images, additional providers,
conversion fidelity and matched native release/documentation publication.


### Release candidate isolation and runner capacity — 2026-11-02

PR #48 merged as b1d636934d2be06b92e01a02b150c11dcbb3dfe9 after all 13 jobs
passed on b365ad6e97c7c524a4dd396419c3d38c53e8265f (run 41523296482).
The earlier PR #47 post-merge CI 41520378824 also passed. PR #48 post-merge
Base image 41526351348 passed, but CI 41526351538 failed: 11 jobs passed, the
CLI container group exhausted hosted runner disk during npm install in the
Next ship image, and macOS ARM native packaging was cancelled before its first
step. Neither the disk failure nor the cancellation proves a product regression;
neither counts as successful acceptance. Logs identify ENOSPC, including failure
to write the runner's own logs. Commit 7c0397c298935c0ce43fac01948d5a96e6e7bd04
reclaims unused Docker builder cache between large acceptance groups while
retaining tagged engine images. Hosted verification remains required.

Fixing commit 0e5638f3b24ca057f26465f01bb1bdbc514d62d2 centralizes release policy
in scripts/release-policy.mjs and applies it to tagged CI and local helpers:

- Candidates publish exact image tags only, become GitHub prereleases, and
  cannot update stable/latest image aliases, deploy production docs or replace
  shared installer scripts. Branch builds only update main feature variants.
- Both repository release pages now use version-pinned attached Unix/PowerShell
  installers and correct image tags without the leading v. The previous
  candidate notes invoked a stable installer and referenced nonexistent v-prefixed
  image tags. Existing published aliases are not changed retroactively.
- The emergency local publisher now uses the same policy, including --prerelease
  and --latest=false. Both release helpers reject workspace version mismatches
  before tagging/building. Candidate preflight never enables production docs.
- The release workflow still allows native publication after an independent
  image failure. Complete candidate acceptance requires checking all jobs and
  final downloaded artifacts; a release page alone is not proof of completion.

Local verification: 12 release policy tests passed, including actual publishing
shells and manifest commands with fake Docker/GitHub/Git executables; no remote
publication is possible through those fixtures. Workflow lint, Bash/Zsh syntax,
workspace version check and strict/deep documentation validation also passed
(55 routes, 53 guide pages, 23 examples). Tests run in ordinary CI and tagged
release verification. Final hosted CI and merge are pending for this branch.

See specs/zel-release-procedure.md for candidate preparation and stable promotion.
No version bump or release tag was made, no native release was published, no
manual provider/registry deployment occurred, and no local Docker daemon was
started. The intended next candidate is 0.2.0-rc.1, after reviewed versioning and
full CI. Current native source and latest public release still say 0.1.3.


Release review follow-up: c7931f250214117cf58b2955db86399866b66946 changes
zimage-push's default from latest to local and zimage-all to local feature tags,
matching zimage. Explicit raw pushes remain available; release helpers apply
channel policy. Generated release notes now show matching ZELLIJ_IMAGE settings
for both Unix and PowerShell. Fourteen release tests pass locally, including
execution of the branch manifest command and development image helpers.

PR #49: https://github.com/mylife-inc/Zellij/pull/49. Initial run 41533203275
passed all six native package/installer targets, engine and schema checks before
the final follow-up was pushed; it does not certify the updated head. Refresh
PR #49's final head/checks before merging. Read-only setup checks confirmed that
RELEASES_GITHUB_TOKEN, NETLIFY_AUTH_TOKEN and NETLIFY_SITE_ID exist by name;
DEPLOY_DOCS is unset. Secret values were neither read nor printed. Presence does
not establish token validity or destination permissions. The development repo
has no published releases; the shared repo's newest Zellij release is still
zellij-v0.1.3. No user setup action is currently needed for this milestone.


### Native 0.2.0-rc.1 preparation — 2026-11-02

PR #49 merged as d324f9ef51a6c36d47304ef765adb6725aa12bff after all 13 jobs
passed on 136a206de28a24e1bdf7232f9d9332f80abd23a5 (41534811176). Its post-merge
Base image run 41538755022 passed, confirming main-only publication. Post-merge
CI 41538755131 failed the engine watcher test: 364 of 365 engine tests passed;
the assertion read nested/new.svg after observing an earlier logo edit and could
race the later asset preparation. This is an invalid test completion signal,
not evidence of permanent asset loss. Production preparation still serializes
updates and uses the existing short directory swap; it is not a whole-site
transaction. The test now forces separate edit batches and polls the requested
new asset. Refresh the complete post-merge run for other jobs' final results.

Preparation commit 03ec06f2c2bf93aee88c629449dd3a5bc2ce36f0 sets the Rust workspace
and its six package lock entries to 0.2.0-rc.1. Bare zel ci now defaults to its
own compatible version, and its regression checks this pin plus preservation on
invalid input. Release notes in specs/releases/v0.2.0-rc.1.md describe supported
commands, platforms and remaining limits and are included in both release pages.
The public stable release remains zellij-v0.1.3 until a new publication occurs.

The release workflow and local ztest/zcheck helpers also used npx vitest at the
repository root, but the lockfile installs Vitest inside the engine workspace.
That command failed locally with vitest not found. They now use the existing
npm test workspace command, matching ordinary CI.

Published release acceptance is a new reusable/manual-tag workflow. After
publication it downloads the exact archive, per-file checksum, SHA256SUMS and
both attached installers from the development repository and (when published)
mylife-inc/releases. It checks hashes and source installer bytes, compares the
two repositories, installs through their actual public URLs, and runs the native
archive/installer/upgrade/build/serve/view acceptance on six native targets.
Windows public installation uses both PowerShell 5.1 and PowerShell 7. Installer
line endings are fixed to LF so source comparisons agree on Windows. The four
published image variants are pulled and executed on native Linux AMD64 and ARM64;
architecture, engine contract and feature selection are checked. Acceptance jobs
have read-only repository permissions and receive no publishing/cloud secrets.
They can be rerun on a published tag without republishing assets.

Local verification: 365 engine tests, 315 Rust tests in 38 suites, 25 release
policy/download tests, workspace typechecks, engine packaging, warnings-denied
Clippy, Rust formatting, workflow lint and strict/deep docs passed (55 routes,
53 guide pages, 23 examples). Download tests use fake HTTP responses and image
tests use a fake Docker executable; no local Docker daemon was started. Locked
npm ci restored dependencies and reported zero advisories at this checkpoint.

Next: final-head PR CI and merge, then tag the reviewed candidate and monitor
all release and post-publication acceptance jobs. Do not call a release page
alone complete: native publication can still succeed while image jobs fail.
No tag, native release or production website deployment has occurred in this
preparation commit. Repository visibility and the old sealed repository are
unchanged; development remains in public mylife-inc/Zellij.


### Published 0.2.0-rc.1 and download recovery — 2026-11-02

PR #50 merged as 61881e805fdf34af12757e33ca4c1703c314825e after all 13 CI jobs
passed on 7b6838154cce0eaf69788c58a63abe0d1a2e5efa (41542526979). Annotated tag
v0.2.0-rc.1 points to that merge commit. The candidate is now public in both
repositories, each with six archives, six per-file checksums, SHA256SUMS and
both installers (15 assets):

- https://github.com/mylife-inc/Zellij/releases/tag/v0.2.0-rc.1
- https://github.com/mylife-inc/releases/releases/tag/zellij-v0.2.0-rc.1

Release run 41546241822 built all six native targets, published all four exact
engine image variants for Linux AMD64/ARM64, and built the documentation.
Production Netlify deployment and shared stable-installer updates were skipped.
Both release pages are prereleases; scoped stable Zellij remains zellij-v0.1.3.
Main Base image run 41546163136 also passed. Main CI 41546163311 had 11 passing
jobs with CLI and Windows portable acceptance still running at this checkpoint;
refresh it before describing its final result.

The original release run is NOT green: 27 jobs passed, while both Windows
public-acceptance jobs failed after completing the development-repository
installation and native suite. Their next fetch, from the shared repository,
failed with ECONNRESET/UND_ERR_SOCKET. A failed-jobs retry reproduced this.
This was an acceptance downloader failure; no checksum, installer or binary
assertion failed. Do not erase these failures or move the published tag.

Fix f9c6b45 adds at most three attempts for connection errors/timeouts. Each
attempt discards partial bytes and retains size, checksum and installer/source
checks. Missing/oversized assets and integrity failures are not retried. Fix
29029e16e2caf30b1b1dbb99e43f64b95d5ee76d permits an explicit published version in
manual acceptance runs. The harness checks out refs/tags/<version> separately,
so source comparisons and native tests still use the actual release commit.

Published release acceptance 41548199714 PASSED all eight jobs using harness
29029e1 against unchanged source tag v0.2.0-rc.1 at 61881e8. All six native
targets downloaded both repositories' assets, verified byte equality/checksums,
and exercised public installation. Windows ran PowerShell 5.1 and 7; its x64
log confirms a reset was recovered on attempt 2. Both native Linux architectures
pulled and executed all four published image variants. Use this successful
follow-up run together with the original publication run as candidate evidence:
https://github.com/mylife-inc/Zellij/actions/runs/41548199714

Local verification: 29 release/download tests, workflow lint and diff checks
passed. The actual published Intel macOS archive also passed temporary
installation from both repositories, replacement/refusal/upgrade tests, portable
build, moved-output preview, serve/view and Ctrl-C. No local Docker daemon was
started, and no global zel installation or PATH was changed. The download fix
changes acceptance infrastructure; candidate archives/installers/images were
not rebuilt or replaced. The fix branch still requires final PR CI and merge.

Next: finish the workflow-fix review, then exercise candidate-generated CI and
provider delivery against explicitly selected test destinations before stable
promotion and matched production documentation. Other tracked scope remains:
advanced build options, combined multi-platform site images, additional
providers and converter fidelity. Candidate acceptance does not mean the full
product has zero bugs or that every historical feature proposal is implemented.


Candidate documentation follow-up: local installation, quickstart, CLI and
contract pages now identify the published 0.2.0-rc.1 candidate, show pinned
attached Unix/PowerShell installers and the matching engine image, and use the
available candidate in copyable CI pins. Stale statements that source still
reports 0.1.3 or Windows packages await publication were corrected. Stable
0.1.3 remains explicitly distinguished from the candidate. The current-source
sections were refreshed while dated implementation history was preserved.
Strict/deep documentation validation passed: 55 routes, 53 guide pages and
23 examples. Both release pages also link the successful acceptance run and
retain the original publication failures. Production docs remain undeployed.


### Candidate acceptance closeout — 2026-11-02

[PR #51](https://github.com/mylife-inc/Zellij/pull/51) merged as
49d9a148f40df2b747d7ce6a0f46efc5a16c3954 after final CI
[41549182222](https://github.com/mylife-inc/Zellij/actions/runs/41549182222)
passed all 13 jobs on 9ab6232758f535cf517acff263523fc5ee96353b. This includes all
six native package/installer jobs, three portable/browser platforms, engine,
schema, showcase and full container delivery acceptance. The separate candidate
source main CI 41546163311 also finished with all 13 jobs passing, superseding
the pending result in the preparation record above.

Published-artifact acceptance remains 41548199714: eight passing jobs against
unchanged v0.2.0-rc.1 source 61881e8, using download harness 29029e1. Both release
pages now link that result and preserve the original Windows download failures.
An additional local smoke used the actual published Intel macOS executable to
generate portable `zel ci` and hosted `zel new project` workflows in temporary
repositories: both pinned 0.2.0-rc.1 and its matching engine and passed actionlint.
This checked generation; live execution of those newly generated workflows and
real provider delivery at designated test destinations remain the next gate.

The candidate is available for acceptance use. Stable 0.1.3, production docs,
and the published candidate binaries/installers/images retain their prior state.
No local Docker daemon was started. The next work is live candidate delivery
acceptance and an explicit stable-promotion decision, alongside the tracked
advanced build/provider/conversion backlog. Do not infer a zero-bug or complete
historical-feature claim from these passing checks.


### Generated workflow and delivery acceptance — 2026-11-02

Refreshed PR #51 post-merge status: CI 41552573220 and Base image 41552573125
both passed on 49d9a14. Candidate v0.2.0-rc.1 remains at 61881e8; no new release
or stable promotion is part of this source change.

Fixing commit 0a79f9504b233b39ad311eb112789e4299290a88 addresses a multi-site CI
queue collision. Every generated workflow previously had the display name
Documentation and used github.workflow in its concurrency group, so distinct
site workflows on the same ref shared a queue. GitHub allows only one pending
run per group by default, allowing a new site's run to replace another pending
site run. See https://docs.github.com/en/actions/concepts/workflows-and-actions/concurrency.
The generator now names workflows by their configured filename and derives a
bounded group key from SHA-256 of that filename plus github.ref. The hash
preserves case-sensitive file identity even though GitHub group names are
case-insensitive. sha2 was already locked for the CLI; no third-party version
changed. Tests cover independent sites, stable regeneration and case-only names.
This generator fix is newer than the published candidate and needs a later
native release. Existing workflows need deliberate regeneration/review.

Six committed acceptance workflows are generated without hand-editing from
scripts/fixtures/release-delivery. They use the corrected source generator,
but install and execute the public 0.2.0-rc.1 binary and exact engine image.
They cover native portable and hosted static builds, static/Next image delivery,
and portable/hosted Netlify delivery. PR events only validate/build. Publication
requires manual dispatch on main; Netlify never receives --prod. Dedicated image
destinations are ghcr.io/mylife-inc/zellij/acceptance-static:0.2.0-rc.1 and
acceptance-next:0.2.0-rc.1. A separate manual read-only inspection workflow pulls
those packages on fresh runners and verifies HTTP pages/assets, non-root users,
health checks, registry digests and source-file isolation.

The preparation helper regenerates workflows in a temporary Git repository,
compares their exact bytes, requires independent queues and checks PR/manual
triggers plus draft-only publication. Ordinary CI checks generated-workflow
drift. The shared HTTP checker supports either public draft renderer and rejects
missing pages/styles, altered assets and exposed sources. No local Docker daemon
was started. No actual registry or Netlify publication has occurred at this
preparation checkpoint. A user question about the Netlify test destination is
pending: existing documentation-site draft previews or a separate supplied site.
Do not manually dispatch Netlify delivery until that target choice is settled.

Local verification: 317 Rust tests in 38 suites, warnings-denied Clippy, Rust
formatting, all seven acceptance workflows plus CI through actionlint, exact
workflow regeneration, and three HTTP-checker tests passed. The HTTP checker
also passed against a real native portable build/temporary server. Hosted
fixture deep/strict validation passed (3 routes, 2 guide pages), and product
docs passed (55 routes, 53 guide pages, 23 examples). Full PR CI, actual generated
GitHub execution, registry publication/pulls and selected Netlify drafts remain
pending. Preserve their eventual run IDs and distinguish source-generator
acceptance from capabilities actually shipped in the candidate binary.


### Live delivery results and Netlify account discovery — 2026-11-02

PR #53 merged as bc3955bbd14a586a9d2da8375ff6f0f8717cb95b after all 19 executed
checks passed on acbdb94: full CI 41561583999 (13 jobs) and six actual generated
workflow checks (41561584047, 41561584073, 41561584038, 41561584027,
41561584019, 41561584037). The four PR publication jobs were correctly skipped.
Post-merge Base image run 41564319833 passed; CI 41564319708 was still running
at this checkpoint and must be refreshed before stating its result.

Manual generated workflows installed the public 0.2.0-rc.1 executable and
published dedicated test packages. Static run 41564355308 passed. Next run
41564361875 initially uploaded layers but GHCR denied the manifest; attempt 2
passed with no code or permission changes. The first failure remains evidence
of an undiagnosed registry failure, not a fixed product defect.

Fresh-pull inspection initially exposed an acceptance-checker mistake:
41565212699 passed static and could not find the not-yet-published Next tag;
41565578836 passed static but probed /index.html for Next, whose adapted home
route is /. Fix d2844ab9dcf28851e838aa9464cc06642c026d4b checks the HTTP root,
adds a regression fixture with no /index.html route, and retains startup status
and bounded container logs on failure. Four local HTTP tests passed (the new
case failed before the fix). Corrected run 41565958521 passed BOTH jobs against
the unchanged published images, checking AMD64, non-root users, health checks,
actual pages/assets and source-file isolation. Registry digests:

- ghcr.io/mylife-inc/zellij/acceptance-static@sha256:831a2eaec96ffa66d23133659fecdbb104128461f7b57f67e817b9c29a923740
- ghcr.io/mylife-inc/zellij/acceptance-next@sha256:fd0cf3db3087c70e0bcfd3d9a452231f7874bd964f252de569fe36864ee6fb50

The user authorized drafts on the existing Zellij Netlify site. The official
OpenAPI contract was checked: createSiteDeploy takes a deployFiles JSON body
with draft/async fields; the client sends draft: true without --prod.
https://github.com/netlify/open-api/blob/master/swagger.yml
Portable draft run 41564369100 failed GET-site preflight with HTTP 404 BEFORE
building or creating any deployment. The user's screenshot confirmed both
secret names exist, but not their current access. A temporary repository
variable selected the known public domain zellij.shebka.net; run 41565327817
also failed preflight with 404. That temporary variable was removed and the
existing secrets were never read or changed. Hosted draft dispatch was held.
No draft URL exists and live Netlify acceptance is NOT passed.

The user then reported separate GitHub accounts (hamidbm and mylife-inc), with
the existing Netlify deployment originating from hamidbm, and proposed a new
Netlify project associated with mylife-inc. This is a migration discussion,
NOT authorization to delete the live site or move its domain yet. Preserve
existing production while comparing every published route and its information
with current local documentation; prepare a separate staging destination before
any cutover. Netlify account/token access, not GitHub repository identity alone,
determines API access. User requires no loss of published information.

Public response fingerprints for /, /cli, /guide/quickstart were identical
before and after the failed preflights (130579, 124251, 159412 bytes). SHA-256:
8e1204979df3a294646e794f378d91e02f4186e289522890a26b161b22cb47f7;
8a895d38e705e5c6f5ab502996b3c316757d3a2683b913f0a935c28dc6d01247;
fdc8a5c4d7f67e5ced91a019c9693d165919fe4bbac0852df2f6a5042a83022d.
This proves those three bodies matched, not whole-site deployment identity.
No local Docker daemon was started. Candidate assets/tag, stable channel and
production docs were not promoted. The generator fix still needs a later native
release; the checker fix requires PR CI and merge. Full content parity and the
Netlify account migration remain separate unfinished work.


### Netlify staging and published-content preservation — 2026-11-02

PR #54 merged as 35a2261f8112f05a3365c64e8659712dd6e28f64 after all 13 CI jobs
passed in 41569977319. PR #53 main CI 41564319708 also passed. Both published
acceptance images remain verified by 41565958521; Netlify drafts have not passed.

The user reports separate hamidbm/mylife-inc identities and confirmed that the
intended destination is unavailable in Netlify's transfer dialog. Proceed with
a separate staging project under the intended account, while retaining the
existing production project and zellij.shebka.net domain. Deletion, transfer,
domain cutover and production replacement remain unapproved.

Source fix 8370ec4fc43ef2af6e775096df7010b624b608d9 preserves old documentation
bookmarks and corrects repository links. A fresh public inventory found 46
pages; all exist in the 55-route local site. Thirteen normalized page bodies are
identical and 33 contain reviewed corrections/expansion/reorganization. All 29
referenced content images match source bytes. The initial comparison identified
82 lost heading IDs; 74 static aliases and eight restored headings now retain
them. Local Chromium verified all 282 original heading IDs, 46 original and nine
new routes, 29 image hashes, fragment navigation, search and current GitHub links.
Strict/deep validation (55 routes, 53 guide pages, 23 examples), static webpack
build, workflow lint and diff checks passed. Normal Engine CI now runs this
preservation check. Full PR CI/merge for this source fix remain pending.

See specs/zellij-netlify-migration-2026-11-02.md and specs/evidence for the
comparison, legacy mapping, limitations and migration gates. Local ignored
artifacts: backups/zellij-production-2026-11-02.tar.gz (public crawl, not a full
Netlify backup) and backups/zellij-staging-2026-11-02.zip (validated 55-page
static upload). The report records their hashes. A full production deploy ZIP
from Netlify is still needed before cutover; the crawl cannot discover unlinked
files or provider settings. All generated build-only source changes were restored.

The user has the staging upload and instructions to create a separate project,
keep its temporary netlify.app address, and update the existing GitHub Netlify
secrets for it. Await the new URL and readiness before running draft acceptance
there. No credentials were read, no local Docker daemon started, and no existing
production content replaced. Native candidate/stable channels are unchanged;
the multi-site workflow fix still requires a later native candidate.


### Preservation acceptance and staging handoff — 2026-11-02

PR #55 merged as f1ca1ff04f049366a1175b4dfe2bd6e78a6c34a1 after all 13 jobs
passed in 41573372928 on final head 27947739e2bfdfa126f323aca847e970627c71bc:
https://github.com/mylife-inc/Zellij/actions/runs/41573372928
Source fix 8370ec4 preserves 82 legacy bookmarks and corrects repository links.
The Engine gate verified 46 original routes plus nine additions, 282 original
heading IDs, 29 byte-identical images, legacy navigation and search. All six
native package targets, three portable platforms, container delivery, schema
and showcase checks passed. This closes the pending PR result above; it does
not establish remote Netlify acceptance. PR #54 main CI 41572436263 also passed.
PR #55 post-merge CI 41576036593 and Base image 41576036573 remain running at
this closeout; their outcomes must be refreshed independently.

Read-only DNS inspection: zellij.shebka.net CNAME zellij-docs.netlify.app;
shebka.net nameservers ns1.hostmonster.com and ns2.hostmonster.com. No settings
changed. The validated staging ZIP is ready; await the new project's temporary
URL and confirmation that GitHub's Netlify secrets target it. The prepared
remote checker has only been syntax-checked. Both CLI draft deployments and
remote documentation preservation remain unverified pending that destination.

Retain the original live project/domain. A complete production deploy ZIP,
deploy ID and relevant settings remain prerequisites before a separately
approved cutover. The public crawl is not a complete provider backup. Candidate
0.2.0-rc.1 assets and stable 0.1.3 remain unchanged; the workflow queue fix still
needs a later native candidate. See the migration report for the concrete user
handoff. No local Docker daemon or production deployment was involved.


### Live Netlify draft acceptance and staging browser finding — 2026-11-02

The user created https://zel-stage.netlify.app, made the main site public,
updated NETLIFY_SITE_ID and generated a new NETLIFY_AUTH_TOKEN. Secret names and
update dates were checked without reading values; no repository variable
supersedes the site-ID secret. RELEASES_GITHUB_TOKEN already exists and was
left unchanged. The user subsequently made previews public after the first
portable preview request returned HTTP 401. Netlify treats main-site and
preview visibility separately.

Both actual generated workflows ran on 311421951ac9c6d3bf1904f150b0e1c79d85ab81
using the public 0.2.0-rc.1 CLI and exact engine image. No --prod flag was used.

- Portable run 41587034632 passed; draft:
  https://6ae97831f41a2555aa3f49e9--zel-stage.netlify.app/
- Hosted run 41587142130 passed; draft:
  https://6ae978a309307ab3058e7340--zel-stage.netlify.app/

After preview visibility changed, both passed scripts/check-delivery-http.mjs:
actual pages, exact logo bytes, linked styles/scripts/fonts and 404 responses
for source/configuration paths. In the normal in-app browser, a search for
zirconium opened /guide/next.html in the portable draft and /manual/next in the
hosted draft. Neither flow emitted a captured console error. These small test
fixtures are separate from the complete documentation at the staging root.
Before/after SHA-256 fingerprints for /, /cli and /guide/quickstart matched on
both the old production site and the staging main URL. This covers six public
responses, not whole-site deployment identity.

The full 55-route staged documentation matches the validated upload after
removing only Netlify's observed added hosting comment and HUD script; raw
responses are not all byte-identical. All 29 authored image hashes match.
The paced headless browser verified all 46 original routes and 282 heading IDs,
then received a Netlify connection-verification page (HTTP 403) on the first
added route. The earlier run stopped at /cli/serve with the same status. These
runs are incomplete, not clean full-browser passes. A normal in-app browser
opened all nine new pages, checked all 82 compatibility targets/scroll margins,
followed an old fragment on each of the 16 affected pages, and navigated search
to /cli/doctor#runtime-selection.

That browser also recorded repeated React #418 hydration errors on the full
documentation. Controlled local comparison of /cli/completions found no errors
for the original build, an error with byte-identical Netlify HTML, an error with
only the observed hosting comment at its original position, and no error with
only the badge script. Thus the injected comment is sufficient to reproduce
this failure; the isolated script is not. This is a hosting interaction, not
missing authored content, and has not yet been resolved. The user was asked to
turn off the staging Powered by Netlify badge so we can test whether its setting
also removes that comment. Do not claim the setting has changed or the error
is fixed until the actual remote response and browser have been checked.

Evidence: specs/evidence/zellij-netlify-acceptance-2026-11-02.json and ignored
backups/zellij-netlify-acceptance-2026-11-02. The latter retains raw logs,
fingerprints and diagnostic scripts. No authentication challenge was bypassed,
credentials read, local Docker started, production site replaced, domain moved
or stable release promoted. Original full Netlify deploy ZIP/settings and an
explicit production cutover decision remain outstanding. The workflow queue
fix still needs a later native candidate; published rc.1 assets are unchanged.

Refresh of earlier pending checks: PR #55 post-merge runs 41576036593 and
41576036573 were cancelled and must not be called passed. They were superseded
by main CI 41576279817 and Base image 41576279748 on PR #56 merge 3114219;
both completed successfully.


### Badge-off result and whitespace isolation — 2026-11-02

The user disabled Powered by Netlify badge. Fresh HTTP checks on `/`,
`/cli/completions` and `/guide/quickstart/new-project` returned 200 with the
badge script absent but the hosting comment/newline still present. Removing
only that observed block makes each response identical to the validated upload.
A fresh normal-browser visit to `/cli/completions` still emitted React #418.

Further controlled tests isolate the leading LF: newline-only before viewport
metadata reproduces #418; the same Netlify comment without its leading LF is
clean. The saved loopback reproduction also verified clean original HTML,
one error with a newline, and no error with a minimal comment. Hydration was
confirmed via the mounted theme control. A development build reproduces the
mismatch at `<style id="zellij-theme">`; installed React DOM's hydration scan
stops at the extra text node. This refines the earlier comment-level finding;
the badge setting did not fix it.

See specs/zellij-netlify-hydration-2026-11-02.md for the ready-to-send Netlify
support request and specs/evidence/serve-head-whitespace-repro.mjs for the
read-only reproduction. The report has not been sent. No engine workaround or
warning suppression was added. The preservation checker currently observes
uncaught page errors, not recoverable errors logged through console.error;
extend that coverage when implementing the remedy. Full-docs remote hydration
acceptance remains open. No production/domain/release/secret changes occurred.
Temporary development servers were stopped and their generated source changes
were removed. The source branch started from PR #57 merge df07f28.


### Browser acceptance hardening — 2026-11-02

The user has filed the Netlify issue and emailed support@netlify.com. Provider
resolution remains pending. The acceptance gap found during staging is addressed
in scripts/check-docs-preservation.mjs: capture console.error/pageerror with
route context and wait for the mounted theme control after every full navigation.
Two integration regressions invoke the real checker on copied built HTML with
an extra head newline or an uncaught exception. Each must fail with its expected
error channel and diagnostic. The positive full-docs check remains the preceding
Engine CI gate. Local syntax/actionlint/diff checks pass; final hosted results
and fixing commit are to be recorded after verification. No application HTML
cleanup or error suppression was introduced.


PR #59 first run 41594298255 passed full-docs preservation with the new error
capture and hydration wait, but its negative hydration test assumed that React
always uses console.error. Hosted Chromium reported it through pageerror.
The test now accepts either channel for React #418; a third control modifies
only an existing JavaScript asset to emit a console-only sentinel. That control
specifically guards the old console-capture gap without changing the HTML tree.
The uncaught-exception control passed on the first run. Final CI is pending.


### Candidate 0.2.0-rc.2 preparation — 2026-11-02

The user authorized continuing with browser acceptance and the next candidate.
Workspace version and only the six local package versions in Cargo.lock are
0.2.0-rc.2; external dependencies are unchanged. Reviewed candidate notes list
the per-workflow queue fix, public-download recovery, preservation corrections,
and strengthened hydration acceptance. The hosting newline issue remains open
with Netlify and is explicitly not claimed fixed. rc.1 assets are immutable;
normal installation documentation continues to target the published candidate.

Local release/download tests: 29 passed. Native CLI unit/workflow tests: 26 unit
and 11 integration tests passed; one real-container test remains intentionally
ignored locally and is a CI gate. The first sandboxed unit run failed seven
local API fixtures because loopback binding was denied; the identical rerun
with loopback permission passed. Version preflight confirms v0.2.0-rc.2.
Final-head CI, review/merge and publication remain pending; no rc.2 tag exists
and stable/production channels have not changed.


### Browser acceptance complete; rc.2 prepared — 2026-11-02

PR #59 merged as 93dca084a018f7ff56e6259441beca4a82874c32. Source commits
68db620 and 97f9f9f capture both browser error channels and wait for mounted
controls. All 13 checks passed on final head 97f9f9f in run 41594855467. Engine
verified all 55 routes, 282 original headings, 82 compatibility mappings, 29
original image hashes and search with no captured browser errors after
hydration. All three negative controls passed: head newline/React #418,
uncaught exception, and a console-only error in an existing JavaScript asset.
This resolves the acceptance coverage gap, not Netlify's HTML transformation.
The first run 41594298255 was superseded/cancelled after its Engine negative
test failed on an overly specific reporting-channel expectation; retain that
failure separately from the passing final run.

PR #60 merged as 0fbf0a12c3cf7074c4620e2bb84cc5d8e5fd7469. All 13 checks
passed on final head 04653b0 in run 41595462484. The merge and checked head both
have tree 4353ca74faba1aad6f6d8e9a4444ad1aa95cf2d0. Version preparation is in
249d0b1, with final scope/verification notes in 04653b0. The reviewed release
notes are specs/releases/v0.2.0-rc.2.md. All six archive/installer platforms,
three portable platforms, engine, schema, real containers and showcase passed.

Automatic approval review rejected executing `zrelease v0.2.0-rc.2`: creation
and publication of a public tag/assets requires explicit user approval beyond
preparation/validation. The command did not execute. No tag, release assets,
versioned images, stable promotion or production deployment was created by this
attempt. Local non-publishing `zcheck` was started separately; its final result
is recorded in the follow-up below. The source remains ready for a specifically
authorized prerelease from 0fbf0a1. After approval, refresh checks and publish to
mylife-inc/Zellij and mylife-inc/releases with matching exact GHCR tags, then
verify the downloaded public artifacts and released delivery workflows.

The user reports filing the Netlify bug and emailing support@netlify.com; a
provider response remains pending. Preserve the old project/domain and obtain
the full original deploy backup before any separately approved cutover.


#### Final local validation — 2026-11-02

The first full local `zcheck` passed typechecking, 365 engine tests, schema
consistency, Rust formatting and warnings-denied Clippy, then failed four
`static_images` mock-client probes at the three-second deadline. Repeating that
suite yielded two timeouts; one affected test passed in isolation. A separate
experiment, without Zellij, launched 12 new exit-zero shell scripts concurrently:
five exceeded three seconds and the slowest took 4.841 seconds. Sequential
re-execution took 0.012–0.022 seconds. This establishes startup latency outside
the product; the OS-level cause was not diagnosed.

The entire `zcheck` was then rerun with `RUST_TEST_THREADS=1`: exit 0, all
365 engine tests and 317 Rust tests passed, with one intentionally ignored
real-container test covered by CI. All ten mock-image tests passed. Product deadlines and CI
parallelism are unchanged, and no local container daemon was started. Default
parallel local execution remains a documented environment limitation, not a
claimed product fix. Raw logs are retained under ignored
backups/zellij-rc2-preparation-2026-11-02/ with hashes in the evidence JSON.
Publication still requires the specific user approval described above.

At this record, post-merge Base image run 41598251612 passed. Main CI rerun
41598251651 has 11 successful jobs, with CLI and Windows portable checks still
in progress; it is not recorded as passed. Both final PR runs already passed
all 13 jobs. Refresh the main run before publication.


### Hosted development auto-reload — 2026-11-03

The user asked to continue implementing the remaining CLI capabilities. Hosted
`dev` and updated editable wrappers now publish an opaque development revision
only after successful search/asset preparation. Development layouts poll that
revision and reload the current URL; image-only edits are included. Invalid
edits do not announce a reload. The development runtime cache is keyed by the
revision so a reload inside the normal 250 ms memo window cannot reuse the old
revision. Production layouts omit the poller. Watcher shutdown removes its
signal; one-shot preparation removes a signal left by an interrupted process.

Focused regressions and all 369 engine tests pass, as do workspace typechecks,
the packaged engine build, installed-CLI watcher/child lifecycle acceptance,
actionlint, syntax/diff checks, and deep/strict validation of all 55 docs routes
and 23 examples. Browser acceptance now requires automatic content and image-only
reloads, no reload on invalid edits, recovery and removed navigation entries,
and no production polling. Final hosted CI and fixing commit remain pending.
No local container daemon was started and no release was published.

This extends the unpublished rc.2 candidate notes. The earlier proposed release
commit 0fbf0a1 predates this work; any later release proposal must use a reviewed
commit containing its final advertised scope. Publication approval is still
pending. Remaining CLI work includes hosted JSON validation, advanced options,
combined image manifests, additional deployment/CI providers and converters.

PR #62's first browser run (41775973925) passed Chromium reload/recovery but
failed WebKit's image check: the updated document retained a cached 140px image
instead of the new 120px image. Markdown images used public asset URLs without
fingerprints. The follow-up applies prepared hashes to those URLs while keeping
encoded filenames, query values and fragments; it also invalidates the asset
manifest cache on atomic replacement instead of waiting 250 ms. A fixed-clock
regression checks replacement, removal/recreation and production memoization.
The original failure is retained here; final browser verification is pending.

Run 41777437141 subsequently passed both browser engines and real-container
development reload/recovery. Its last conversion-export assertion still expected
an unversioned Markdown image URL and failed after the successful export. The
acceptance script now requires the exact SHA-256 content fingerprint for both
MkDocs and Docusaurus image URLs, while continuing to verify exported bytes and
links. This updates the expected URL contract; the fingerprint implementation is
unchanged. The full run is retained as failed until the corrected check passes.


### Hosted JSON validation — 2026-11-03

Hosted `zel check --json` now consumes a versioned engine report. It preserves
real content/deep findings and warning severity, including strict warning-only
failures. Native success is exit 0, content/strict failure 3, unavailable runtime
or inaccessible mount 4, configuration conflict 2, and incompatible/failed engine
report 1. Normal JSON mode writes one stdout report and no stderr diagnostics
after site/checker selection; explicit verbose diagnostics still use stderr.
The Node check command emits the same validation schema but retains exit 1 for
failed validation. Hosted `pages` counts loaded routes, not compiled HTML files.

The protocol validates version, target, mounted root, requested options, status
and findings before accepting success. The structured host root preserves
quotes, backslashes and Unicode; diagnostic messages stay verbatim so authored
URLs cannot be mistaken for container paths. Older engine
images fail with image-selection/pull guidance rather than parsing terminal
text. Runtime/configuration failures also receive structured reports. Content
mounts remain read-only. Invalid CLI flags and discovery failures before the
checker is selected retain ordinary CLI errors.

Regressions cover engine findings, strict warnings, bootstrap failures, malformed
or inconsistent reports, native option forwarding, host path escaping and
missing runtimes. The installed-package/real-container acceptance script now
exercises JSON success, deep failure, strict warnings and recovery.

Implementation commits: `9a3d740`, `06e9d1e`, `9d89dd5`; review and hosted
acceptance are tracked in [PR #63](https://github.com/mylife-inc/Zellij/pull/63).
Local verification passed: 321 Rust tests (one real-container test intentionally
ignored locally), warnings-denied Clippy, formatting, 372 engine tests,
workspace typechecks, engine package build and installed JSON checks. All 55
documentation routes and 23 examples pass deep/strict validation; actionlint,
JavaScript syntax and diff checks pass. Rust fixtures used serial execution and
loopback access for mock HTTP services. Two obsolete tests expecting hosted
JSON rejection were updated to assert its structured runtime failures.

The dependent reload implementation is `3ddd2cd` plus image-cache correction
`8d3d4f1`, tracked in [PR #62](https://github.com/mylife-inc/Zellij/pull/62).
Both Chromium and WebKit passed automatic content/image reload, invalid-edit
preservation and recovery in job 126401231393 of run 41777437141. Existing
showcase browser coverage also passed 317 tests. Current full platform/container
results remain available in the PR checks; refresh them before relying on a
release source. Neither implementation publishes a release or deploys production.

Remaining CLI scope includes additional deployment and CI providers, combined
multi-platform image manifests/build arguments, broader import fidelity,
hosted dev host/browser options, and advanced build overrides. The Netlify
hosting-injected hydration failure remains a separate provider issue.


### Development host and browser options — 2026-11-03

Implementation `222ffda` adds `zel dev --host`, `--open` and
`--browser <executable>` for portable and hosted sites. Defaults remain portable
loopback (`127.0.0.1`) and hosted IPv4 wildcard (`0.0.0.0`). `localhost` and
literal IPv4/IPv6 addresses are accepted; wildcard browser URLs use loopback.
Portable preview validates request hosts against the connection's local address,
including dual-stack address normalization, while retaining path/source guards.

Hosted development owns a uniquely named container. Successful engine preparation
publishes its session and current `basePath` with the development revision. The
CLI confirms both container metadata and the matching HTTP response before
announcing/opening the URL, once per session. Failed startup, occupied ports and
unrelated listeners cannot cause a browser launch. Current engine metadata and
a reachable published port are required; after five minutes without confirmation,
manual-opening guidance is printed and development continues. Ctrl-C uses bounded
runtime stop/removal, reaps the foreground client and retains the dependency cache.
Quiet cancellation does not replay runtime logs as an unexpected failure.

Local verification: 322 Rust tests passed (one real-container test intentionally
ignored), warnings-denied Clippy, formatting, 374 engine tests, workspace
typechecks and package build. All 55 documentation routes and 23 examples pass
deep/strict checks. Native fake-runtime/browser acceptance passed IPv4/IPv6,
readiness delay, prefix handling, one-time launch, wrong-session rejection,
startup/port/browser failures, quiet mode and cancellation during startup/serving.
No local Docker or GUI browser was used. The new native acceptance runs on all
three CI operating systems; real-container/browser coverage now checks explicit
loopback publishing, prefixed browser opening and container/port cleanup. CI
results must be checked before merging; this entry records local evidence only.

Prior milestones are now merged: PR #62 as `e6a83bbc7fff5ed383d146a04fa2b5c25d83bb29`
with all 13 checks passing in run `41780233137` on `caf5e3c`; PR #63 as
`a3a1571a2a479b8d8dd679a30b31a23b1c3afe21` with all 13 passing in `41780276435`
on `09d1b65`. Each merge tree matches its tested head. Main `a3a1571` CI
`41782982430` and Base image `41782982399` also passed. Earlier failed runs remain
part of the dated history above.

Remaining CLI scope: advanced build overrides, image build arguments and combined
multi-platform manifests, additional deployment/CI providers, and broader import
coverage/fidelity. The Netlify hosting-injected hydration issue remains external.
No release publication, production deployment or domain change is included.


### Development options — acceptance follow-up, 2026-11-04

PR #64's core implementation is `222ffda`. Follow-up `d0eae61` releases the
session's temporary identity before the inaccessible-mount diagnostic exits;
native regressions verify cleanup for invalid content and inaccessible mounts.
Run `41795843334` on `d0eae61` passed 12 of 13 jobs, including all three native
development-option suites and six native package/install jobs. Its CLI job
failed before starting a container: the new browser fixture had made the app
directory nonempty before scaffold initialization. This was an acceptance setup
error, not evidence that container startup or cleanup had passed.

Harness correction `3b95629` initializes the app first, waits for shutdown output
to drain with a bounded deadline, and runs real development acceptance before
long image-publication checks. The scaffold/launcher setup, JavaScript syntax,
actionlint and diff checks pass locally without Docker or a GUI browser. Final
container results are tracked in [PR #64](https://github.com/mylife-inc/Zellij/pull/64).
Earlier run `41794505393` was canceled after the startup cleanup correction;
it did not establish final acceptance. No release or production deployment was made.


### Development options — final acceptance, 2026-11-04

PR #64 merged as `706f5430772a13ba9428aeef0d60f3237fd2dd4f` after all
13 checks passed in run `41798421655` on `bae81ce2cdc63616ed20c8b8c39b71dd9653b49b`.
The tested head and merge have identical trees. Refreshed main CI `41801848062`
and Base image `41801847816` both passed. This supersedes the pending acceptance
status above while retaining its earlier failure/correction history.

### Hosted build overrides — local implementation, 2026-11-04

The current branch adds invocation-only `--base-path`, `--site-url` and
`--no-check` to hosted `build`/`b`, static/Next image commands (`si`/`ni`), legacy
Next image builds and hosted Dockerfile generation (`idf`). Portable Markdown
adapted into a Next image accepts them; pure portable HTML/static images reject
them before runtime probing. Deploy/publication/CI generation do not yet accept
these three flags. Existing delivery callers retain default behavior.

An explicit prefix overrides the authored/wrapper prefix; `/` clears it. The
public site URL is an HTTP(S) origin only, with the path supplied separately.
Both reach rendering, SEO metadata, image layout and health checks. Next images
retain settings at runtime, including unknown-route content loads. Authored
configuration is validated before applying overrides and is never rewritten.
`--no-check` removes only the separate preflight; preparation, schema, rendering
and output ownership validation remain active. Failed exports preserve the last
successful site.

A typed Rust settings value crosses the CLI/engine boundary as versioned,
hex-encoded JSON in `ZELLIJ_BUILD_OVERRIDES`. User strings never become shell or
Dockerfile instructions. Engine capability `buildOverridesVersion: 1` is checked
before installation; older images fail with upgrade guidance. The shared Node
bootstrap validates/normalizes the protocol and supplies both Next configuration
and content sources. The production runtime cache keys include these settings.

Implementation: `62ec4f1`. Local verification passed: 323 Rust tests (one real
container test intentionally ignored locally), warnings-denied Clippy, formatting,
415 engine tests, workspace typechecks, package build and isolated installed
checker acceptance. All 55 documentation routes and 23 examples pass deep/strict
validation. Native fake-runtime acceptance covers aliases, generated recipes,
root reset, preflight selection, invalid arguments, portable rejection and source/
output preservation. JavaScript syntax, actionlint and diff checks pass. Rust
fixtures ran serially with loopback access for their existing mock services.
No local Docker or GUI browser was used. CI adds the native fixture on all three
operating systems plus real export/static-image/Next-image URL and health checks;
final platform/container results are pending.
Remaining CLI scope includes image build arguments and combined multi-platform
manifests, propagation of these options to delivery/CI where appropriate,
additional deployment/CI providers and broader import fidelity. Netlify's
hosting-injected hydration problem remains external. No release publication,
production deployment or domain change is included.


### Hosted build overrides — first CI follow-up, 2026-11-04

[PR #65](https://github.com/mylife-inc/Zellij/pull/65) tracks implementation
`62ec4f1` and context/probe regression `23d471e`. In run `41814544611`, the real
hosted override export checks passed, including canonical/sitemap/robots URLs,
root reset, invalid-content preservation and old-engine rejection. Engine,
showcase and all six native package/install jobs passed. The Windows portable
job failed earlier in the existing development-options fixture with an HTTP
`ECONNRESET`, before reaching the new override fixture.

Correction `fc3397f`: the fake development server used a single socket read, which could respond and
close before a complete request arrived. The correction explicitly sets accepted
sockets to blocking mode (Windows can inherit listener mode), consumes bounded
complete headers, adds a deliberately fragmented request regression, and labels
HTTP failures. It adds no blanket retry and does not change production server
behavior. The superseding CI run must verify Windows and full image acceptance;
this partial run is not claimed as complete acceptance.

The corrected native development suite, including fragmented headers, passes
locally with fake browser executables and loopback sockets. JavaScript syntax,
actionlint and diff checks pass. The new build-override fixture now runs before
longer native preview fixtures in each platform job.


### Hosted build overrides — final acceptance, 2026-11-04

PR #65 merged as `58623ff683e7597471607c59825966dec44d158e` after all 13
checks passed in run `41815732030` on `a92b80c59bee232773267c20dc07ea98015e42d0`.
The merge and tested head have identical trees. This includes Windows acceptance
after fixture correction `fc3397f`, real hosted exports, both image kinds and
browser/health checks. Earlier failed/canceled evidence remains above.
Main Base image run `41819057643` passed; main CI `41819057870` was still running
when refreshed for this next milestone. Native rc.1 remains the latest published
release; no rc.2 publication or production/domain change was performed.

### Build overrides in delivery and generated CI — 2026-11-04

Implementation `c20d48b` extends `--base-path`, `--site-url` and `--no-check` to
`netlify`, `deploy`, `publish`/`pub`, `ship`, and GitHub workflow generation.
The same typed options reach the single hosted export or image build before
upload/push. Pure portable static delivery rejects them before runtime or remote
work; portable-to-Next publication accepts them. Dry runs show normalized values,
including `/` for an explicitly cleared prefix, and the skipped content preflight.
Netlify credentials/site access still precede the build. Failed builds never
create a deployment or push. Temporary exports, authored files and prior local
output retain their existing protections.

`ship` keeps its converted site and saved delivery destination after failure,
but never saves invocation-only build settings. Both success and failure guidance
say to repeat the applicable overrides. Bare saved `deploy` uses discovery or
`--site-root`; its first positional remains the explicit provider selector.

Generated workflows serialize URL settings as environment values and pass quoted
arguments only to build/delivery commands. GitHub expression syntax in values is
rejected. `/` retains its root-reset meaning. `--no-check` removes the separate
workflow deep/strict check and skips the build's preflight; preparation/schema/
rendering, credential/runtime checks and publication event gates remain active.
Without it, the workflow validates authored content as before. The generator
requires version 0.2.0-rc.2 or later when these flags are used; it still does not
publish or verify release availability. Omission on regeneration restores default
checks/settings. Existing generated release-acceptance workflows are byte-identical.

Local validation passed: 325 Rust tests (one real-container test intentionally
ignored locally), warnings-denied Clippy, formatting, all 12 workflow tests and
11 parser tests after the command enum allocation correction. Native fake-runtime
coverage verifies publication aliases, no push on failed builds, imported-site
retention, dry plans, root reset, portable rejection and unchanged source. A Bash
replay verifies shell-looking prefix values remain literal; workflow failures
preserve the previous file. Mock Netlify tests retain authentication preflight.
Generated workflows pass actionlint with and without preflight; all 55 docs
routes and 23 examples pass deep/strict validation. Script syntax/diff checks pass.
No local Docker or GUI browser was used.

CI now exercises prefixed imported static and Next images through push/pull,
browser navigation, canonical/sitemap metadata and health checks. Hosted Netlify
acceptance uses an overridden deployment prefix/origin and verifies invalid
content under `--no-check` cannot create a deployment. Final platform/container
results must be refreshed from the pull request before merging.

Remaining CLI work includes image build arguments, combined multi-platform image
publication, additional deployment/CI providers and broader import fidelity.
The Netlify hosting-injected hydration issue remains external. Candidate release
publication and production migration remain separate, pending decisions.


### Delivery overrides — CI harness correction, 2026-11-04

[PR #66](https://github.com/mylife-inc/Zellij/pull/66) tracks `c20d48b` and
context record `d634507`. Run `41821800581` failed the macOS workflow test because
its system Bash cannot execute the existing `${REPOSITORY,,}` expression in the
Ubuntu-only legacy image job. Correction `0c1802a` runs full workflow-shell replay
on Linux, matching the generated runner. All operating systems still check the
workflow data, option placement, release compatibility, rejection/preservation,
and native delivery forwarding. No product behavior or generated workflow changed.
The focused regression, formatting and diff checks pass locally after correction.
Final CI acceptance is pending on the replacement head; the first run is not
claimed as complete acceptance.


### Delivery overrides — final acceptance and main follow-up, 2026-11-04

PR #66 merged as `b7a076af0a72302f9c44f66e295ae9b208983ed8` after all 13
checks passed in run `41822262323` on `c3b94705fdb096ce5f699e9c30172b414aa937c4`.
The tested head and merge have identical trees. The changed Netlify export/upload
and static/Next ship image tests passed, as did Linux/macOS/Windows acceptance.
The earlier macOS harness failure remains recorded above.

Main Base image run `41826282353` passed. Main CI `41826282286` passed 12 jobs;
Windows ARM packaging compiled Rust successfully, then PowerShell crashed with
`Internal CLR error (0x80131506)` while creating the ZIP. Only that failed job
was rerun; attempt 2 was queued when this entry was written. This is not claimed
as a source fix or a fully successful main run. Native rc.1 remains the latest
published release; no rc.2 tag/release or production/domain change occurred.

### Image build arguments — 2026-11-04

Implementation `86b6d2d` adds repeatable explicit `--build-arg NAME=VALUE` to
static/Next image commands and aliases, legacy image builds, printed Dockerfiles,
image publication, image `ship`, and generated GitHub image workflows. Values
are typed argv data. Names are ASCII identifiers; values allow Unicode, spaces,
additional equals signs and literal shell syntax, but no control characters.
Empty values are explicit and repeated names use the last value. Environment
lookup is not implicit. Zellij-owned settings, historical FEATURES and automatic
platform arguments are reserved for their dedicated flags.

Recipes declare names only, before builder commands. Hosted final runtime stages
do not redeclare arguments or receive added ENV settings. Portable static images
apply them only during container packaging, after native Markdown compilation.
Proxy/BUILDKIT arguments are forwarded without extra declarations. Printed
recipes require the values again when built. Build arguments are not secrets:
builders can retain them in metadata, and build scripts can bake them into files.
Authored content and saved configuration are unchanged. Dry plans omit values;
ship success/failure guidance explains that arguments must be repeated.

Static-folder builds, Netlify delivery and Netlify ship reject image arguments
before output, credentials or remote requests. Image build failures do not push.
Generated workflows pass numbered environment assignments as quoted arguments
only to build/publication commands, never checks or prebuilt pushes. GitHub
expressions are rejected before writing; older CLI pins are rejected (minimum
0.2.0-rc.2, still unpublished). Regeneration without arguments removes them.

Local verification: 328 Rust tests passed, one real-container test intentionally
ignored locally; warnings-denied Clippy and formatting passed. Native fake clients
verify Docker/Podman/nerdctl argv, aliases, portable/hosted recipes, publication,
ship retention, omission/reset, dry plans, early rejection and unchanged source.
Existing override acceptance and all six generated release workflow comparisons
pass. Generated static/Next/legacy image workflows pass actionlint with combined
build arguments, features, URL overrides and ARM64 selection. All 55 docs routes
and 23 examples pass deep/strict checks; script syntax and diff checks pass.

CI adds the native fixture on all three operating systems. Real static/Next ship
acceptance uses a custom builder script that consumes an exact literal argument
and emits a public JSON file, then verifies the pulled artifact over HTTP and
checks that argument names are absent from runtime ENV. No local Docker or GUI
browser was used. Final CI acceptance must be refreshed before merging.
Remaining work includes combined multi-platform image publication, additional
providers and broader import fidelity. Netlify's hosting-injected hydration issue
and the separate release/production-migration decisions remain pending.


### Image arguments — first CI correction, 2026-11-04

[PR #67](https://github.com/mylife-inc/Zellij/pull/67) tracks implementation
`86b6d2d` and context `bcb0401`. Run `41880580238` failed the new Windows native
fixture during the fake Docker info probe, before building. Rust on Windows
searches the executable and system directories before inherited PATH; setting
only the fixture PATH does not isolate an installed system Docker. Correction
`45f44bf` copies zel beside its fake clients and verifies each selected probe
writes the fixture marker. Production runtime discovery is unchanged. Corrected
native acceptance, JavaScript syntax and diff checks pass locally; final Windows
and real-container acceptance remain pending on the replacement head.

The earlier main CI `41826282286` now passed on attempt 2: Windows ARM packaging
job `126744911367` succeeded without source changes after the isolated PowerShell
CLR crash. This is retry recovery, not a diagnosed product fix. No release was
published and no production site changed.


### Combined image publication and smaller runtimes — 2026-11-04

The user authorized the next CLI milestone and requested smaller images.
Implementation `a29a792121ce0cb3fa6ba027e243173d6e0a40e7` adds `--platform all`
(and either ordering of the explicit AMD64/ARM64 comma list) to registry
publication and ship. `image.platform: all` saves the default. Single-platform
local commands remain compatible; combined selection on a local-only image
command fails before runtime discovery with publication/override guidance.
Netlify still rejects explicit image platform flags.

Both renderers and static/Next image kinds share the same recipe/argument paths.
Docker uses the caller's selected Buildx builder and direct registry output;
Podman uses an invocation-owned lowercase manifest, pushes all architectures,
and cleans the manifest after success/failure; nerdctl uses its image exporter.
No builder, emulator or daemon is installed/started. Combined publication does
not promise a local image; failure guidance acknowledges possible remote layers
and recommends inspecting the registry before retrying. Ship retains its
converted site and saved architecture choice after delivery failure.

Generated combined CI verifies both architectures without credentials on PRs.
Publication events use native AMD64/ARM64 runners, unique per-run/attempt staging
tags, and a dependent final job joining immutable digests only after both jobs
succeed. Staging tags remain for registry retention policies. Existing event
restrictions, permissions, explicit configuration selectors and build overrides
are retained. Minimum CLI pin is 0.2.0-rc.2 (still unpublished); single-platform
workflows, including all six release acceptance files, are unchanged.

Static images use upstream NGINX Alpine slim without optional dynamic modules.
Next images trim npm, Yarn and Node headers in a separate stage and copy that
filesystem into a fresh runtime, so removed tools do not remain in inherited
layers. Node's matching Alpine libraries, certificates, licenses and shell are
kept, alongside traced app dependencies, public assets and runtime content.
New CI jobs build/run both image kinds on both native architectures, replay the
generated publication/manifest commands against loopback registries, and compare
uncompressed image size against the old runtime bases with identical site data.
The existing container job also tests direct combined Buildx static publication,
both architecture pulls/HTTP serving, and the existing Next browser suite.

Local verification: 329 Rust tests passed, one real-container test ignored;
warnings-denied Clippy, formatting, isolated native runtime acceptance, existing
image-argument acceptance, all six generated release-workflow comparisons, and
12 combined generated workflows through actionlint passed. Docs deep/strict
checks passed all 55 routes and 23 examples. An initial parallel Rust run hit
three known local fake-shell info-probe timeouts; the full serial run passed.
No local Docker or browser was used. Actual image sizes and final CI acceptance
remain pending for this branch; do not claim measured savings or a merge yet.
Remaining CLI areas include additional providers and broader import fidelity;
release publication and Netlify production migration remain separate decisions.

Prior milestone evidence: PR #67 merged as `557d8b6` after all 13 checks passed
in run `41882538968` attempt 2 on `ba37a8a`; merge/head trees match. The first
attempt's existing Docker info timeout recovered on retry without a code change.
Main base-image run `41889745285` passed; main CI `41889745157` was still running
at this entry. The Netlify hosting-comment hydration issue remains external.


### Combined publication — first CI fixture correction, 2026-11-04

[PR #68](https://github.com/mylife-inc/Zellij/pull/68) tracks this milestone.
Compatibility correction `c215266` preserves Node's original container entrypoint
in the trimmed runtime; all 25 engine tests passed afterward. Run `41893692905`
on `ec35452` built/pushed/pulled static and Next images on both native Linux
architectures. Static serving and size comparison passed. The new Next acceptance
fixture then requested `/index.html`, but the portable-to-hosted adapter maps the
home page to `/` (confirmed in source and the generated route table). The server
started successfully; the fixture's incorrect route caused both failures.
The fixture now requests the renderer's actual home and reports HTTP status on
failure. Full container acceptance, final size figures and merge remain pending.


### Combined publication — native acceptance and size accounting, 2026-11-04

Run `41894634123` on `eec6ca1` passed both native image jobs and the dependent
combined-manifest job (`126803607170`, `126803607619`, `126805836087`). Both static
and Next images build, push, pull, serve both documents, deny Markdown-source
access and pass health checks on AMD64 and ARM64. The generated final command
joins the exact platform digests. Node/npm/Yarn/header checks also pass.

The first size report used Docker's Size field and called it uncompressed;
that label is not portable between classic and containerd image stores. The
runner-local comparison showed reductions, but raw AMD64 and ARM64 counts must
not be compared directly. Acceptance now also pushes the identical-payload
baseline to the disposable loopback registry and sums compressed layer sizes
from its platform manifest. This supplies consistent download-size comparisons
and keeps Docker-reported sizes explicitly labeled. Final compressed figures
and full final-head CI remain pending. No live external registry was used.

Prior main CI `41889745157` is now successful. Superseded PR run `41893692905`
required force-cancellation after its macOS preview job did not stop following a
normal cancellation request; this unblocked the corrected run, not a product fix.


### Combined publication — final runtime footprint pass, 2026-11-04

Read-only inspection of the tested AMD64 image archive confirmed that most Next
runtime space is Node itself and traced app dependencies, with negligible source
maps. It also exposed retained Alpine package-index caches. The final recipes
remove those caches. Static packaging additionally removes the unused alternate
NGINX debug executable and copies the prepared filesystem into a fresh stage so
upstream deleted/cache layers do not inflate downloads. NGINX's entrypoint,
non-root user and SIGQUIT shutdown signal are explicitly retained. Node's
entrypoint remains intact. Required runtime libraries and traced app dependencies
are not pruned. Native CI verifies cache absence, NGINX startup metadata and the
existing serving/health behavior, and measures compressed registry layers against
the former runtime bases. No local container or browser was started.


### Import heading anchors and completed image milestone — 2026-11-04

PR #68 merged as `e7b0d7987787ee4e0856bb3d3edb280eaec0c9ae` after all
16 checks passed in run `41897666269` on `f5c0a5c`; merge and tested trees match.
Both native image jobs and the generated combined-manifest job passed, as did
direct combined Buildx publication, native package/install checks on six targets,
portable acceptance on three OS families, Next browser acceptance and the full
CLI/container suite. Base image post-merge run `41902611084` passed; main CI
`41902611421` had 14 of 16 successful jobs when this record was written.
The final compressed-layer measurements are in
`specs/evidence/zellij-image-publication-2026-11-04.json`: static images decreased
from 27.1–27.5 MB to 6.6–7.0 MB (74.7–75.6%), and Next images from about 84.3 MB
to 75.8–75.9 MB (10%). These compare identical fixture payloads and exclude small
manifests/configuration and attestations; they are not universal site sizes.

Source `d5b6fac` addresses another portion of audit R5: imports now preserve
explicit document-level ATX/setext heading IDs in `{#id}` notation and Docusaurus
MDX/HTML comment forms. The converter recognizes headings with Markdown parser
offsets, preserves inline formatting/rewritten links, and emits literal MDX
`h1`–`h6` elements. A leading H1 becomes a literal anchor below the layout's
single title. Title extraction ignores code examples and removes ID syntax.
Duplicate/unsupported IDs and nested identified headings fail before output
writes; the error identifies the source document. Classes and other heading
attributes are not carried over. Automatic source/plugin slugs and implicit
leading-H1 anchors still need review; this does not close all of R5.

The engine's shared heading reader now indexes literal JSX heading IDs for TOC
and search without executing expressions or assuming spread/dynamic/component
IDs. Existing deep validation checks the actual literal IDs, including collisions
with generated anchors. No new runtime dependency was added.

A new regression exposed the old fence scanner treating shorter or different
fences inside examples as closing fences. Shared marker/width tracking fixes
that in both Material demotion and MDX escaping. Balanced markup spanning a
replaced heading still uses the complete document as its recognition context.

Local validation: 56 converter unit tests and 29 CLI conversion/import/write
safety tests passed; 54 engine heading/TOC/search/deep-check tests passed.
Warnings-denied workspace Clippy, formatting, engine typechecking/packaging,
and the docs deep/strict check (55 routes, 23 examples) passed. The real native
CLI-to-engine fixture passed for MkDocs and Docusaurus: explicit title IDs,
formatted and linked headings, Unicode IDs, reference URLs, unique rendered IDs,
search/TOC targets, strict links and asset bytes. Its existing hosted CI step
also checks full static exports, one page title and rendered TOC links.
No local container or browser ran. Final-head CI and merge are pending for
this import milestone; source functionality remains unpublished rc.2.

Other providers, broader source-format fidelity and release/production migration
remain separate work. Netlify's hosting-comment hydration issue remains external;
the old production site/domain is retained. No new release was published.


### Imported metadata and Docusaurus document IDs — 2026-11-04

PR #69 merged as `b1ff5081013a89b8c98ed95715d8bc114c8a1fe5` after all
16 checks passed in run `41905598125` on `fb02026`; the merge and tested trees
match (`fda6beec21e54a0813a023a2ee256b155277e91c`). Its post-merge base-image
run `41910675985` passed; main CI `41910675972` was still running when this
record was written. Both post-merge runs for PR #68 (`41902611421` and
`41902611084`) finished successfully.

Source `9e296cc` addresses another portion of audit R5. Both importers now use
an authored YAML title and description as active page metadata, retaining the
original metadata in comments. Title precedence is front matter, document H1,
sidebar label, then source path without extension. Sidebar labels are emitted
independently using the engine's existing page/label entries. The shared inert
YAML reader handles BOMs, CRLF and `...` delimiters, reports malformed metadata
with its source path, and validates the types of supported fields. Retained
comments also handle Unicode YAML line separators so their text cannot become
active output fields. No runtime dependency or engine schema was added.

Docusaurus imports inventory the actual `.md`/`.mdx` sources and resolve custom
IDs using the default numeric-prefix rules and `parse_number_prefixes: false`.
A custom ID replaces the basename only. `sidebar_label` overrides a sidebar
item label. Legacy filename IDs remain accepted when unambiguous; conflicts
between canonical IDs and legacy names, duplicate IDs, missing sidebar targets
and ambiguous `.md`/`.mdx` stems fail before writes. Routes and relative links
still derive from physical source paths; custom `slug` values, plugin paths and
custom JavaScript prefix parsers are not applied. Plain `sidebars.json` is now
parsed directly. The inventory rejects symlinks/junctions before reads and
normalizes native path separators for portable lookup.

Draft/unlisted documents are excluded and reported, including explicit sidebar
entries and folder-derived navigation. An explicit empty sidebar stays empty.
Unsupported autogenerated branches leave omitted documents in the report;
they no longer cause all documents to be promoted when no supported entries
remain. Empty navigation and groups emit YAML arrays rather than null values.
Dynamic/missing sidebars retain the documented alphabetical fallback, with
exclusions applied. This remains a partial fidelity improvement, not full
Docusaurus execution or closure of R5.

Local verification: 60 converter unit tests and 37 CLI metadata/reference/
import/write-safety tests passed. Warnings-denied workspace Clippy, formatting
and docs deep/strict checks passed (55 routes, 23 examples). Both real CLI-to-
engine fixtures passed for metadata, distinct sidebar labels, custom IDs,
physical routes, rendered MDX, heading/search/TOC targets and asset bytes. The
metadata CLI suite is now in Linux/macOS/Windows CI; the existing full-export
fixture additionally checks page metadata and sidebar labels in generated HTML.
No local container or browser ran. Final-head hosted CI and merge are pending.

This remains unpublished rc.2 functionality. Release preparation must be
refreshed before any publication, which still needs explicit approval following
the earlier automatic approval rejection. Netlify's hosting-comment issue,
production cutover, other deployment/CI providers and broader import fidelity
remain separate work. The existing production site/domain remains intact.


Follow-up review fix `0fb7e14`: plain JSON sidebar strings are always parsed as
inert data, so labels containing ellipses, backticks or `require()` do not
trigger JavaScript fallback and accidentally include omitted files. All nine
metadata CLI fixtures pass locally after this correction (38 CLI tests across
the milestone's four suites, plus 60 converter tests). PR #70 hosts the final
native/full-export acceptance; CI on the corrected head is pending.


Native CI follow-up: run `41913945831` on `1bc4b1f` passed eight metadata
fixtures on Windows but failed while creating the ninth fixture's junction,
before invoking the importer. `d3db17c` builds that path with native Windows
separators, matching the existing passing junction fixtures, logs setup errors,
and removes the junction before cleanup. All nine fixtures pass locally after
the correction; final-head Windows and full acceptance must still pass.


### Import metadata closure and autogenerated sidebars — 2026-11-04

PR #70 merged as `5001718ed39f022247494b5452e1352642c1152b` after all
16 checks passed in run `41914893785` on `b550285`. Tested and merged trees
match (`87a230cde310e97dada489935212e19aa1e5eb29`). Post-merge base-image run
`41918572383` succeeded. Main run `41918572402` also succeeded after retrying
only its macOS ARM64 packaging job: the original job was cancelled while
queued, with no runner assigned or steps executed. This was not a product
failure. The previous PR #69 main run `41910675972` finished successfully.

Source `94296ac` expands data-only Docusaurus `autogenerated` sidebar slices.
Each selected directory contributes its contents, with nested folders becoming
categories. Manual entries and separate slices retain their relative positions.
Within a slice, finite `sidebar_position`, category metadata positions and
numeric filename prefixes determine order, with source names breaking ties.
Category `_category_.json`, `.yml` and `.yaml` labels/positions are supported;
category metadata overrides linked-index metadata, then folder-derived defaults.
Index/README/same-folder-name conventions and explicit local/full document ID
links become the first child page, or a single page link for a lone index.
`link: null` disables the index convention. Repeated navigation references now
emit each source page only once.

Drafts, unlisted pages, partials and hidden Markdown are excluded and reported;
partials/hidden documents are skipped before reading their contents. Explicit
empty sidebars remain empty. Missing or dynamic sidebars retain alphabetical
fallback. JSON/JSON5 sidebar data is parsed before dynamic-code detection, so
JavaScript-looking labels/comments remain inert data. Invalid positions,
malformed/duplicate category metadata and unsafe or missing directories fail
before output writes. Generated paths use portable separators on Windows.

Local verification: **58 converter unit tests and 47 CLI integration tests**
passed, covering metadata, references, import delivery, autogenerated sidebars
and safe writes. Warnings-denied workspace Clippy, formatting and documentation
deep/strict validation passed (55 routes, 23 examples). Both actual CLI-to-engine
fixtures passed, including generated hierarchy, labels, routes, MDX rendering,
heading/search/TOC targets and asset bytes. CI now runs the new suite on Linux,
macOS and Windows, and the full-export fixture checks the generated category
label in HTML. Hosted CI and merge are pending; no local container or browser
was used.

This is a further partial resolution of audit R5, not complete Docusaurus
compatibility. Generated category landing pages, manual category links,
category styling/collapse fields, custom JavaScript generators, custom slugs
and plugin routing remain unsupported and documented for review. This remains
unpublished rc.2 functionality; release preparation and approval, Netlify's
hosting-comment issue/production cutover, and additional deployment/CI providers
remain separate work.


CI follow-up: run `41955787336` on `8fa5b20` passed 15 jobs, including all
native import suites, but the CLI job failed an existing development-browser
fixture assertion: navigation count changed from 9 to 10 after the test used
Back and then wrote an invalid edit. Both the search index and server revision
remained unchanged across that edit. The fixture recorded its baseline before
waiting for the restored page to acknowledge the last valid asset revision.

Fix `a4c4e44` waits for two successful revision polls in the same document
before recording that baseline, and rechecks the valid heading/image. It keeps
the existing assertions that invalid edits preserve the revision, navigation
count, content and image. No product runtime was changed. JavaScript syntax and
12 deterministic reload/watcher unit tests pass locally. Browser confirmation
and all final-head CI gates remain pending; no local browser/container ran.


### Category landing pages and manual category links — 2026-11-04

PR #71 merged as `eb9070595acf23299eeb01167db1d68f3c762866` after all 16
checks passed in run `41957491288` on `85778f2`. Tested and merged trees match
(`46d90f4f77bfa88c1aa55b23605b5cd1fb643f38`). Both standalone and container
versions of the corrected reload fixture passed, including its invalid-edit
preservation assertions. Post-merge base-image run `41960754932` succeeded;
main run `41960754928` was still running when this entry was prepared.

Source `40d665e` adds Docusaurus generated category indexes and manual category
document links. Manual links resolve full document IDs (with existing unambiguous
legacy filename compatibility), import linked documents outside the child list,
put the linked page first, remove a duplicate direct child, and collapse a lone
index to one link using the category label. Draft/unlisted linked pages remain
excluded. Autogenerated category document links retain local-ID-first lookup.

Generated-index metadata from manual categories and JSON/YAML folder metadata
creates ordinary editable MDX pages with titles, descriptions and nested lists
of imported child links. The title defaults to the category label. Pages live
under `01-guide/category/<normalized-label>/index.mdx`; numbered alternatives
avoid routes occupied by generated pages or inventoried authored documents,
including drafts, unlisted and orphan sources. Repeated autogenerated slices
reuse the same folder's generated page. Empty/fully excluded categories report
that no landing page was created. No source files are changed.

The Docusaurus adapter keeps synthetic pages in memory and feeds them through
the shared emitter/reference resolver. Relative references are rewritten using
the same normalized routes as authored documents. Metadata prose is encoded as
literal text, including markup, Markdown punctuation and module-looking text,
so descriptions/labels cannot introduce MDX code or extra links. No dependency,
engine schema or public CLI syntax changed.

Local verification: **58 converter unit tests + 53 CLI integration tests**
passed, plus formatting, warnings-denied workspace Clippy, documentation deep/
strict validation (55 routes, 23 examples), and both real CLI-to-engine fixtures.
The fixtures validate manual category links, nested generated pages, descriptions,
search, plain-text rendering, routes and asset bytes. CI adds the new native
suite on Linux/macOS/Windows; full-export acceptance verifies generated pages
and links with a deployment prefix. Hosted CI/merge remain pending. No local
container or GUI browser ran.

This is another partial resolution of audit R5. Generated-index source slug,
image, keywords and other unsupported link fields are reported; custom source
URLs, plugins/generators, category styling and embedded DocCardList components
remain unsupported. The generated output is a nested Markdown link list, not
Docusaurus cards. Source rc.2 remains unpublished; release preparation/approval,
Netlify's hosting-comment issue and production cutover, additional cloud/CI
providers and broader conversion fidelity remain separate tasks.


### Explicit Docusaurus URL slugs — 2026-11-04

Prior milestone closure: PR #72 merged as `ab768ea635750b13a092ad97c43f4b3e6782b0cd`
after all 16 checks passed in run `41963452333` on `1d4e5b6`. Tested/merged
trees match (`7e00781c79dc7d72c25ca36da857f8e1aefa4b9c`). The post-merge
base-image workflow `41966348546` passed. Main workflow `41966348612` has a
Docker `info` timeout in CLI job `127044130186`, before the first hosted build;
the Windows portable job is still running. A targeted retry was refused while
the run remained active; revisit after completion. This is not a diagnosed
product fix. PR #71 main `41960754928` and base `41960754932` both passed.

Source `6e9f942` addresses another part of audit R5. Explicit Docusaurus document
and generated-category slugs become validated `route` front matter beneath
`/guide`, using the engine's existing route override. Absolute document slugs
start at that guide root; relative slugs resolve from the physical document's
parent directories, applying default numeric-prefix removal unless disabled
for that document. Literal relative dot segments may stay within the docs root.
Category slugs resolve from the docs root regardless of their physical folder.
Only explicit slugs change routes; other documents retain existing defaults.

Imported filenames, sidebar source references and asset bases stay unchanged.
Converted file links, category child links, navigation and search use the new
routes. Case is retained; percent encoding is decoded once, Unicode is normalized
to NFC and URL links are encoded. Trailing slashes are removed. Colliding routes
(including case-only and Unicode-equivalent paths), malformed encoding, encoded
separators/dot segments, root escapes, queries/fragments/hosts and nonportable
segments fail before writes. Unknown source `route` metadata remains inert;
MkDocs slug metadata is unchanged. Explicit category slugs are reserved before
generated defaults, so defaults receive suffixes independently of sidebar order
without changing their physical filenames. No engine schema/dependency changed.

Local verification passed: **60 converter unit tests + 60 CLI integration tests**,
workspace Clippy with warnings denied, formatting, deep/strict docs validation
(55 routes, 23 examples) and both real CLI-to-engine fixtures. These fixtures
exercise actual route loading, MDX rendering, sidebar/search links, category
pages, heading anchors and asset bytes with uppercase and Unicode/space URLs.
CI adds native slug coverage on Linux/macOS/Windows and full static-export HTML
checks with `/products/docs`. Final-head hosted acceptance and merge are pending.
No local container or GUI browser ran.

Remaining import work includes original deployment prefixes/root-relative source
URLs, plugin routing/generators, category styling, generated-index images/keywords
and embedded DocCardList components. R5 is still only partially resolved. Source
rc.2 remains unpublished; release preparation/approval, Netlify's hosting-comment
issue/production cutover and additional deployment/CI providers remain separate.


#### Unicode static-export follow-up — 2026-11-04

PR #73 run `41968082371` on `a9251b6` passed 13 jobs, including all native
import suites and packages. ARM64 image acceptance failed before image building
with another Docker `info` timeout. The CLI job passed its prior acceptance
steps but failed the new Unicode HTML assertion in the last conversion fixture.
The combined manifest job was skipped because ARM64 had failed. Do not claim
this run as final acceptance or the runtime timeout as fixed.

A native command-line Next 16.4.2 webpack export reproduced the Unicode page
as a 404 HTML shell. Instrumentation showed page params containing encoded
`Caf%C3%A9`/`Release%20Notes`, while metadata received decoded `Café`/`Release Notes`.
The hosted CI build using Next 16.4.5/Turbopack showed the same missing page link.
Fix `9eb97ef` matches exact registered routes first, then tries one safe decoding
step for page lookup and active sidebar navigation. Literal percent routes retain
precedence; malformed encodings, encoded delimiters and dot segments do not
become different routes. The route inventory and static params stay unchanged.

All **423 engine tests** and engine typechecking pass. The actual native webpack
export now contains the Café H1, the prefixed document link and an active sidebar
link, with no Next error shell. This was a command-line build only: no local
container or browser ran. Full-export acceptance retains its original links and
adds explicit page-title/active-link checks. The same conversion check now runs
immediately after the CI engine image is ready, before the long container suite;
no acceptance steps were removed. All final-head CI gates must pass again.

Prior PR #72 post-merge main workflow `41966348612` subsequently passed on its
targeted CLI-job retry, with no source change; base `41966348546` also passed.
Recurring Docker info timeouts remain a follow-up. Docker's official
[info implementation](https://github.com/docker/cli/blob/master/cli/command/system/info.go)
probes client plugins, and the failed ARM64 log left a Compose helper to clean
up. This is a diagnostic lead, not a proven cause or fix.


#### Selected-runtime budget and CI prerequisite correction — 2026-11-04

The next run, `41971949013` on `9eea726`, again hit the three-second Docker
`info` limit on ARM64 immediately after Docker had built the engine image and
started the local registry. It also exposed an error in the preceding CI edit:
the earlier conversion-export step had been placed in the native portable job,
which does not build `zellij-ci`, so it failed trying to pull that missing image.

Source `0d605a6` restores the conversion check to the CLI container job after
image creation/runtime selection and before the longer hosted tests. A parsed
workflow assertion verifies that ordering and that the portable matrix does not
contain the container-only step. All existing acceptance steps remain.

The same source lets an explicitly selected runtime use the existing full
**twelve-second discovery budget**: there are no fallback clients/sockets to
reserve time for in that mode. Automatic discovery still allows three seconds
per probe within twelve seconds overall. Metadata probes remain two seconds,
endpoint selection/secrecy remain unchanged, and no service is started. This is
bounded tolerance for slower healthy runtimes, not a claim to have diagnosed or
fixed Docker's plugin latency. A real CLI fixture with a four-second fake Docker
info response proves the selected endpoint succeeds without calling Podman.

Local verification: 9 runtime-requirement CLI tests and 25 engine Rust unit tests
passed, alongside the prior 120 import/safety Rust tests, 423 engine JavaScript
tests and native Unicode export. Clippy, formatting and refreshed deep/strict
documentation checks passed. Final-head CI must pass before merging PR #73.


### 2026-11-04 — import asset policy and optional validation

PR #73 merged as `a4b575ddbe35b9a0f4c176526bd1a704e682194d` after all 16
checks passed in [CI 41972577587](https://github.com/mylife-inc/Zellij/actions/runs/41972577587)
on final head `208e339234ae336489cb4e46f573c53530a10a95`. Tested/merged tree:
`02a8c4c6c7eee05e05ab8a1403698d9c3d17e31c`. Both post-merge workflows passed:
main CI `41975635129` and base image `41975635128`. This closes the preceding
pending status; earlier failed runs remain recorded above.

Source `ae7c611` exposes the converter library's existing asset-copy policy as
`--assets copy|skip` on `convert`/`cv`, `migrate`/`mig` and `ship`. Copy remains
the default. Skip preserves original asset references and reports omissions;
document links still resolve to imported routes. Path containment, symlink and
hidden-source protections still apply before writing. Omitted assets may need
manual placement/reference repair before publishing. The original design's
`link` policy is not implemented; Clap rejects it before any output is written.
This import-only policy is not saved as a delivery/build setting.

`convert`/`cv --validate` uses migrate's existing hosted check after writing
content or site layout. Missing runtimes return exit 4, content-check failures
return exit 3, and both retain the completed import for review and `zel check`.
Migrate always validates; accepting the shared flag does not run a second check.
Dry runs never validate, write, probe runtimes or invoke delivery. This supersedes
the original design's suggestion to report validation as skipped on an unavailable
runtime: an explicitly requested check must not report success when it did not run.

Local verification passed: 54 CLI import regressions, 27 CLI unit tests (one
existing ignored test), 60 converter tests, warnings-denied workspace Clippy,
formatting, deep/strict docs (55 routes, 23 examples), and both real CLI-to-engine
conversion fixtures. Unit tests requiring loopback sockets passed outside the
socket-restricting sandbox. No local Docker daemon or browser was used. CI's
real export fixture now tests `convert --validate` alongside migrate, and image
publication acceptance exercises explicit asset copying. Final-head native
three-OS and hosted CI remain required before merge.

Public release remains `zellij-v0.2.0-rc.1`; these controls are unpublished rc.2
source. No release, production Netlify deployment or domain change was requested.
Netlify's hosting-comment hydration defect remains with support. Import source
URL/plugin fidelity and additional deployment providers remain incomplete.


### 2026-11-05 — website-link import and CLI completion status

PR #74 merged as `74abf77a6592633288692a1af21fee9905f3ace6`, after all 16
checks passed in [41991356062](https://github.com/mylife-inc/Zellij/actions/runs/41991356062)
on `5846c16c698aaa1212df6a42a89afe83c54aa877`. Tested/merged tree:
`4f698704576125ff784b72469b8786c1a18bb430`. Both post-merge workflows passed:
main `41995023512` and base image `41995023365`.

The user asked whether all CLI commands are implemented and authorized the
suggested website-link work. Every currently registered command family has
handlers, but the entire original specification is not complete. Source
`8f46bd4` replaces the generic dispatch stub with an exhaustive match; normalized
aliases are the only unreachable forms. Future missing handlers now fail to
compile. The current status table is `specs/zel-cli-status-2026-11-05.md`;
additional providers, converters, CI adapters and interactive authoring remain.

The same source maps root-relative and configured same-origin HTTP(S) Markdown
links through source URL conventions to emitted Zellij routes. MkDocs uses
site_url/use_directory_urls, including README/index and direct HTML addresses.
Docusaurus reads direct literal url/baseUrl and a single classic preset/docs
plugin routeBasePath. Default IDs, number-prefix and index conventions, explicit
slugs and explicit category slugs participate. Queries/fragments are unchanged;
URL paths preserve case, normalize Unicode and decode percent encoding once.
Referenced website assets use contained source roots and the existing copy/skip
policy. Other origins are untouched; no URLs are fetched and no JS is executed.

Known omitted or colliding source URLs remain unchanged with reports. Invalid
supported routing settings and unsafe local paths fail before writes. Computed,
custom, localized, versioned and multiple-instance Docusaurus routing is reported
for review, as are implicit generated-category URLs. Relative HTML URLs and
HTML/JSX attributes remain outside the new mapping. Destination deployment
settings are not inferred from the old site's prefix.

Local checks passed: 60 converter unit tests, 63 CLI import tests, 7 alias tests,
workspace Clippy with warnings denied, formatting, deep/strict documentation
(55 routes, 23 examples), and both CLI-to-engine fixtures. CI now includes the
new tests on three native OSes and real hosted exports containing source website
links. No local Docker or browser was used. Final-head CI is required before merge.

Upstream rules were verified against the MkDocs configuration/file URL code and
Docusaurus create-doc/slug code; source links are in the conversion reference.
Refreshed public tag is `v0.2.0-rc.1` (some older entries incorrectly prefixed
that candidate tag with `zellij-`). rc.2 remains unpublished, and its prior
publication proposal predates this work. Netlify support's hosting-comment
hydration issue and production migration remain separate.


#### Windows path follow-up — 2026-11-05

Initial PR #75 CI `42000267890` on `d65ded6` passed the real converted-link
HTML export step but failed a new Windows URL regression. WalkDir returned
native backslash paths while the MkDocs listed-path inventory used forward
slashes. Nested listed documents were mistakenly added as orphans; their
inconsistent route spelling then made website URLs ambiguous. The same path
handling could also produce nonportable names for folder-derived navigation.

Source `74bacea` normalizes discovered relative path components to `/`, as the
Docusaurus inventory already does. The existing nested URL regression and an
additional nested, no-nav fixture cover both cases on every CI platform.
All 31 targeted local import tests pass after the correction, alongside the
prior converter/alias checks (131 distinct Rust tests total). Full final-head
CI, including the Windows rerun, is still required before merge. This is a
code defect correction, not a retry of an unexplained CI failure.


### 2026-11-05 — relative website links (source `d141846`)

Relative document URLs now use the original page URL directory for supported
MkDocs and Docusaurus routing. Explicit Markdown files and regular local assets
keep their prior semantics; unknown slash policies, omitted/ambiguous URLs and
conflicting extensionless filename guesses are reported unchanged. No new asset
search root or source execution is introduced. Unsafe paths fail before writes.
See the conversion reference and ZELLIJ_CONTEXT.md for compatibility details.

Local verification: 132 Rust tests, warnings-denied Clippy, formatting,
actionlint, deep/strict docs and both CLI-to-engine fixtures passed. Three-OS
native tests and actual container HTML-export acceptance are CI gates, pending
for this source. PR #75's all-16-check result and merge are recorded in the
context separately; they do not validate the new source. The full CLI design
remains incomplete; see zel-cli-status-2026-11-05.md.

### 2026-11-05 — PR #76 closure and VitePress import source

PR #76 merged as `e8f525d1e765f0a455de192b200866377f2dd45b` after all 16
checks passed in [42007978630](https://github.com/mylife-inc/Zellij/actions/runs/42007978630)
on `903c66d236bd9a48746a6be6070d9446482d2c93`. Merge and tested head share
tree `1f4e626af5a93b747c1b28471f04e4b57e704eab`. This closes relative website
links (`d141846`) and the Windows watch-announcement relay race (`a1003bd`).
Post-merge base-image run `42011273390` passed. Main `42011273809` has 15
successful jobs, including Windows preview; CLI acceptance is still running.

Source `38b9d41` adds the registered VitePress converter to `detect`,
`list formats`, `convert`/`cv`, `migrate`/`mig` and `ship`. It reads exactly one
`.vitepress/config.{js,ts,mjs,mts}`, accepting literal exports or the standard
`import { defineConfig } from 'vitepress'` wrapper without evaluation. Supports
contained `srcDir`, title/description, nested and multiple declarative sidebars,
inherited item bases, linked groups, page metadata, Markdown links, source
`base`/`cleanUrls` URLs and referenced source/public assets. Multiple sidebars
become prefix-labelled groups in lexical order; omitted documents are reported,
and explicit empty/false sidebars stay empty. Absent sidebars derive visible
Markdown navigation. Absolute HTTP(S) URLs stay external without an origin.

Dynamic config, custom routing/exclusions/locales, dynamic parameter files and
custom output/cache roots fail before writes. Vue components, top navigation,
home layouts and custom Markdown extensions remain manual migration work;
source text/metadata and reports are retained. Public assets use a separate
`assets/imported/public/` namespace; conflicting source/public root URLs fail
before writes, including with `--assets skip`. Existing shared preflight retains
source isolation, symlink/junction rejection, route collisions and inert dry
runs. This is a documented subset, not complete VitePress fidelity.

Local verification passed: **140 Rust tests** (62 converter unit tests and 78
import/inspection integration tests), Clippy with warnings denied, formatting,
actionlint and diff checks; deep/strict docs cover 55 routes and 23 examples.
The Node-only CLI-to-engine fixture passes for all three formats, checking
rendered routes, titles, heading/TOC/search anchors, query/fragment links and
copied bytes under a deployment prefix. Hosted export and three-OS VitePress
acceptance are added to CI; final-head hosted/native verification and merge
remain pending. No local Docker or browser/GUI tests were run.

Semantics were checked against VitePress 1.6.4 source and its
[routing](https://vuejs.github.io/vitepress/v1/guide/routing),
[sidebar](https://vuejs.github.io/vitepress/v1/reference/default-theme-sidebar),
[asset](https://vuejs.github.io/vitepress/v1/guide/asset-handling) and
[config](https://vuejs.github.io/vitepress/v1/reference/site-config) documentation.
Current vitepress.dev defaults to a newer development version; this milestone
does not claim that version's full behavior. The published native release
remains rc.1. Release publication and production Netlify migration are separate.

### 2026-11-05 — VitePress closure and Jekyll documentation imports

PR #77 merged as `0729235ed7d8e3a0835b324e045e5e19a15d79e3` after all 16
checks passed in [42014697065](https://github.com/mylife-inc/Zellij/actions/runs/42014697065)
on `155f09c0feb9cc77477f1d7fbc197fc8e2943012`. Its tree matches the tested
`3f5edcf76ad3ab9e5e5028a8643124200ae5c38b`. Final fix `155f09c` prevents a
URL in a sidebar query string from making a local VitePress page look external.
Both post-merge workflows passed: main `42017999796`, base image `42017999836`.
PR #76's previously pending main run `42011273809` also passed, completing
both of that merge's post-merge workflows. Local main was clean and synchronized
at the start of this milestone; native release remains `v0.2.0-rc.1`.

Source `0073bca` registers `jekyll` with detection, formats, convert/migrate and
ship. It supports one YAML config, contained source/data/output roots, front-
matter `.md`/`.markdown` pages, enabled documentation collections and literal
scoped defaults. Supported navigation data arrays (`name/link` or `title/url`,
optional children, optionally wrapped in `main`) retain order and nesting.
Without a navigation file, published pages precede enabled collections; literal
collection order lists precede filename order. Empty explicit navigation stays
empty; omitted/unpublished/navigation-excluded pages are reported.

Root-relative, same-origin and relative website links use source URL settings,
index conventions, supported permalink styles, literal page permalinks and
collection `:collection`/`:path`/`:output_ext` patterns. These map to existing
filename-derived Zellij routes, not destination permalink overrides. Referenced
static assets must be eligible under the Jekyll source inventory: excluded,
internal data/template and generated files cannot be published through links.
Canonical source roots fix macOS alias comparisons; BOM/CRLF template detection
is covered. Original metadata is retained alongside effective defaults.

Limits are explicit: no Ruby/Liquid/theme/plugin execution, blog migration,
dated collections, glob policies/scopes, forced includes, TOML import or
arbitrary routing patterns. Unsupported navigation/configuration fails before
writes; Liquid/theme presentation is reported for manual work. The importer
adds no dependencies. Hugo remains the next importer; Sphinx and other
provider/CI/interactive work remain in the broader CLI backlog.

Local verification passed: **155 Rust tests** (64 converter unit tests plus
91 import/inspection integration tests), final targeted Jekyll regressions,
warnings-denied Clippy, formatting, actionlint and diff checks. Deep/strict docs
cover 55 routes and 23 examples. Node-only rendering passes all four formats,
including a Jekyll collection, source-relative assets, anchors, search/TOC and
website links. Three-OS import acceptance and real hosted export join CI;
final-head hosted/native checks and merge remain pending. No local Docker or
browser/GUI test ran. User docs, current CLI status and unpublished rc.2 release
notes were updated with behavior, command examples and limits.

Source semantics were checked against Jekyll 4.4.1's
[configuration](https://jekyllrb.com/docs/configuration/options/),
[collections](https://jekyllrb.com/docs/collections/),
[navigation tutorial](https://jekyllrb.com/docs/step-by-step/06-data-files/),
[permalinks](https://jekyllrb.com/docs/permalinks/) and matching tagged source
for page/collection URLs, publication and scoped defaults. This remains a
conservative documentation subset. Production documentation deployment,
Netlify migration and native release publication are separate pending work.

#### Windows ARM64 acceptance follow-up — 2026-11-05

Initial Jekyll PR #78 run `42084376374` failed native ARM64 Windows job
`127445423055` after the binary built and both PowerShell installers passed.
The visible error was `EBUSY` deleting the installed executable from the
harness's disposable directory. The unconditional cleanup could replace an
earlier assertion error, so the original cause cannot be inferred from that
log alone. Source `e460f40` preserves both errors with `AggregateError`, includes
command arguments in failed-command diagnostics, and bounds retries only for
filesystem cleanup (10 retries with 100 ms linear backoff). No installer or
acceptance assertion is retried or skipped. Node syntax and diff checks pass;
new final-head Windows and full CI evidence are required before merge.

### 2026-11-05 — Jekyll closure and Hugo documentation imports

PR #78 merged as `6190e51bcfca91a874f508af3f2a523492319440` after all 16
checks passed in [42085346212](https://github.com/mylife-inc/Zellij/actions/runs/42085346212)
on `ffdeca7f8d245979e9dcd4b7d58e2d40a093ca92`. Its tree equals tested tree
`c594f2570f1664de72cdec025206748e965218d8`. Windows ARM64 job `127449174329`
passed both installers, native upgrades/refusals/cancellation, portable output,
HTTP serve/view and real Ctrl-C. The earlier masked cleanup failure remains
historical evidence; this passing run does not establish its original cause.
Post-merge base image `42090020402` passed. Main `42090020416` has 15 passing
jobs with CLI acceptance still running at this refresh. Local main was clean
and synchronized before the Hugo branch; published native candidate remains rc.1.

Source `5631e6c` registers Hugo detection and conversion through convert/cv,
migrate/mig and ship. One `hugo.*` or legacy `config.*` TOML/YAML/JSON file
selects contained content/static roots. Markdown pages accept optional front
matter in all three formats, case-insensitive keys, UTC/offset ISO dates,
publication flags and supported page build rules. Draft, future, expired,
headless and unlisted pages stay excluded unless a supported explicit build
flag permits them. Date decisions are made once at conversion time.

Hugo section `_index.md` maps to visible `index.mdx`; leaf bundles keep their
index pages and do not promote Markdown resources into pages. `DocTree` now
has an explicit source-to-output filename map, leaving original source keys
available for Markdown links and asset lookup. Shared emit/rewrite defaults
remain unchanged for existing importers. Derived section navigation uses
weight, descending date, case-folded link title and source path. Theme menus
and locale-specific collation are outside this subset.

Supported deployment prefixes, pretty URLs, regular-page slugs/URLs and
source-file links map to filename-derived Zellij routes. Eligible referenced
content/static resources retain their bytes; implicit sections preserve asset
filename case. Static/resource URL conflicts and unsafe paths fail before
writes. Explicit section/bundle route overrides, menus, split configuration,
modules, cascades, localization, custom date rules and complex routing require
manual migration. Shortcodes and aliases remain reported source metadata/text;
no Go, template or theme execution takes place.

Tagged Hugo v0.169.0 source inspection found that leading ASCII whitespace and
inline opening delimiters still introduce front matter. The importer matches
this behavior so draft rules cannot be bypassed; Org front matter and malformed
end delimiters fail explicitly. Official references:
[configuration](https://gohugo.io/configuration/introduction/),
[front matter](https://gohugo.io/content-management/front-matter/),
[bundles](https://gohugo.io/content-management/page-bundles/),
[publication options](https://gohugo.io/content-management/build-options/),
[URLs](https://gohugo.io/content-management/urls/) and the tagged
[front matter parser](https://github.com/gohugoio/hugo/blob/v0.169.0/parser/pageparser/pagelexer_intro.go),
[path helper](https://github.com/gohugoio/hugo/blob/v0.169.0/common/paths/path.go),
[ordering](https://github.com/gohugoio/hugo/blob/v0.169.0/resources/page/pages_sort.go).

Local verification passed: **171 distinct Rust tests** (66 converter unit,
105 import/inspection integration), including final targeted regressions after
parser/resource corrections; warnings-denied Clippy, formatting, actionlint and
diff checks; deep/strict docs covering 55 routes and 23 examples; and Node-only
CLI-to-engine rendering for all five importers. The Hugo fixture checks section
pages, a leaf bundle, links, Unicode asset paths, titles, headings/search and
static resources. The same fixture's hosted export and three-OS import tests
are in CI; final-head hosted/native acceptance and merge are still pending.
No local Docker, browser or GUI tests ran. The TOML and date dependencies use
minimal features; no runtime dependency installation is added for users.

The conversion reference, guides, detection/help overview, current CLI status,
and unpublished rc.2 notes were updated. Sphinx imports and the remaining
provider/CI/interactive/advanced-option backlog are still incomplete. Native
release publication and production Netlify migration remain separate work.

#### Hugo resource ownership correction — 2026-11-05

Source `f7f0121` keeps implicit top-level section resources independent of a
draft home page. A section's visible image remains eligible while a resource
owned by the omitted home page stays excluded. The expanded real-CLI fixture
passes all 14 Hugo tests; warnings-denied Clippy passes. This joins PR #79 and
requires the new final-head CI run before merge.

#### Hugo Windows path correction — 2026-11-05

PR #79 run `42094771583` failed Windows import job `127482680944`: native
`with_file_name` changed nested `_index.md` output to a backslash-containing
relative path, which the portable-path validator correctly refused. Source
`771e40e` normalizes generated output filenames from path components to `/`.
The existing section/link integration fixture covers the failure. All 14 Hugo
tests and warnings-denied Clippy pass locally; new Windows and full final-head
CI evidence are still required before merge. The safety validator is unchanged.


### 2026-11-05 — Hugo closure and Sphinx reStructuredText imports

PR #79 merged as `7a03f50c54421ba03a4a00f1be4cc5c747fccd65` after all 16
checks passed in [42095877145](https://github.com/mylife-inc/Zellij/actions/runs/42095877145)
on `8102782f0252c97d222236bd6b9b3161d71045ce`. Tested and merged tree:
`cf3541bcae5aeac20caf35e665a97758c95c6ef0`. Both post-merge workflows passed:
main `42099649036` (all 16 jobs) and base image `42099648776`. This closes the
Windows section-path correction `771e40e` and implicit-section resource fix
`f7f0121`. PR #78's previously pending main run `42090020416` passed too; both
of that merge's workflows are green. Main was clean and synchronized when this
branch began. The published native candidate remains `v0.2.0-rc.1`.

Source `968b469` registers Sphinx in detect/formats/convert/cv/migrate/mig/ship.
The directory containing `conf.py` is the source root. Configuration is parsed
as literal Python data with no Python interpreter or dependency installation.
Supported settings include project, root_doc/master_doc, .rst source_suffix,
UTF-8, explicit exclusions and html_baseurl. Executable/dynamic configuration,
extensions and unsupported content-transforming settings fail before writes.
Theme/presentation values are not applied as destination theme configuration.

Explicit toctrees preserve order, captions, labels and nested page groups.
Hidden trees define navigation without inline lists; self does not recurse.
Navigation cycles, excluded/missing targets, unsafe paths and route/label
collisions fail before output. Unlisted pages are reported and not published,
unlike Sphinx's ability to build orphan pages. The reStructuredText adapter
handles headings, paragraphs, emphasis, inline literals, flat lists, literal/code
blocks, standard admonitions, images, named hyperlinks and doc/ref/download
roles. Source HTML and directive-looking code remain literal. Section labels
and supported implicit IDs feed links, TOC and search. Alternate old heading IDs
require review. Unsupported syntax produces a source-specific preflight error;
this is a conservative subset, not full Sphinx/MyST/API-extension fidelity.

.rst links and HTML-builder website addresses map into filename-derived guide
routes. Contained referenced eligible assets retain bytes; source-root-relative
images/downloads do not inherit html_baseurl. Excluded sources, templates,
configuration and script files cannot be copied as downloads. Resource paths
are checked even under --assets skip. No source tool or configuration code runs.
No new dependencies were added.

The same source fixes shared MDX escaping of inline code containing shorter
backtick runs: literal spans match equal-length delimiters, preserving braces
and angle brackets instead of escaping code text. Regression tests cover this
and real Sphinx-to-engine rendering covers literal HTML, role text and code.

Local verification passed: **183 distinct Rust tests** (68 converter unit and
115 import/inspection integration), with final targeted Sphinx regressions after
resource/punctuation corrections; warnings-denied Clippy, formatting, actionlint
and diff checks; strict/deep documentation (55 routes, 23 examples); and Node-only
rendering for all six import formats. Hosted exports and the new three-OS Sphinx
suite are CI gates; final-head CI and merge are pending for this source. No local
Docker, browser or GUI tests ran. Logs use /private/tmp/zellij-sphinx-*; durable
scope and upstream references are in the conversion reference.

User documentation, quickstarts, CLI status and unpublished rc.2 notes were
updated. Remaining work includes richer Sphinx/other-format fidelity, provider
adapters, GitLab/CircleCI, interactive authoring and advanced options. Native
release publication needs separate explicit authorization; the old rc.2 proposal
predates this work. Production Netlify migration and the vendor's hosting-comment
hydration issue remain separate. No production deployment or domain change ran.


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


### 2026-11-05 — Cloudflare Pages native deployment

The next provider is Cloudflare Pages Direct Upload for generated static HTML.
`zel deploy cloudflare`, `zel cloudflare` / `cf`, saved `publish` / bare `deploy`,
`ship --provider cloudflare`, and `ci --deploy cloudflare` share provider intent.
The destination is an existing project name: explicit `--site` / `--site-id`,
then `CLOUDFLARE_PAGES_PROJECT_NAME`, then same-provider `publish.site`.
Changing providers never reuses the old provider's saved destination or env ID.
`CLOUDFLARE_ACCOUNT_ID` and `CLOUDFLARE_API_TOKEN` stay in the environment;
there is no account/project creation, Wrangler invocation or stored-login lookup.

Project preflight precedes the temporary build. Preview uses `zel-preview`,
independent of the local Git branch, and rejects a project whose production
branch has that name. Explicit `--prod` uses the preflight production branch.
The adapter inventories generated files, uses Cloudflare's BLAKE3/base64/extension
asset keys, upload JWTs, missing-hash lookup, bounded batches and multipart
manifest creation, then waits for deployment-stage success. Account tokens and
upload JWTs are never combined in a request. Identity/environment/URL checks,
redirect refusal, redacted provider errors and bounded requests apply. Creation
POSTs are never automatically retried; uncertain deployments require dashboard
inspection. Both renderers preserve source and previous output.

Supported exports contain at most 20,000 files, each at most 25 MiB. Symlinks,
junctions, special files and reserved Workers/Functions/custom rules are rejected;
this adapter does not deploy a Next server. Hosted URL/feature overrides retain
existing behavior. CI generates provider-specific variables/secrets, excludes
publication credentials from PR verification, supports explicit token provisioning,
and requires rc.2 or later. Published rc.1 remains unchanged. Docs explain setup,
preview behavior, unsupported forms and pending live-account acceptance.

Protocol references inspected read-only:
[deployment API](https://developers.cloudflare.com/api/resources/pages/subresources/projects/subresources/deployments/methods/create/),
[official upload implementation](https://github.com/cloudflare/workers-sdk/blob/main/packages/wrangler/src/pages/upload.ts),
[asset hashing](https://github.com/cloudflare/workers-sdk/blob/main/packages/deploy-helpers/src/deploy/helpers/hash.ts).
Rust mock HTTP tests cover uploads, caching, batches, Unicode/percent filenames,
validation ordering, failures, timeouts and portable output. GitHub CI adds a
real hosted-export/Cloudflare-protocol fixture. No live provider account was used.

PR #80 closure: all 16 checks passed on `fb622d1f84c3516e6ba6d0affbd21cd39b043b44`
in `42109434915`; merge `c067b5ae17a8a6f3e724e0c625383cccd041cd74` has identical
tested tree `550b8b21d753a5dc33e84848a391a76c5b2c6052`. Post-merge base-image run
`42113725968` passed. Main `42113725983` hit Windows EPERM in
`scripts/preview-watch.mjs:130` during the fixture's source-directory rename;
all Rust tests had passed. The follow-up adds Windows-only, bounded retries for
EPERM/EACCES/EBUSY on the two fixture renames; other errors still fail immediately.
Windows/hosted final-head CI must validate this follow-up before merge.

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

### 2026-11-05 — Vercel prebuilt static delivery

PR #81 (Cloudflare Pages) merged as `9cde253e29a50e3bb2ea7b90355f274be569e629`
after all 16 checks passed on `b1bf1dac48cd19541e976d8a3408cb62ea6570ed` in
run `42119403884`. Tested and merged trees match
`c3d659a4c13a245292d69a370f44fecde9380137`. Post-merge base-image run
`42123170846` passed; main `42123170594` was still running with no failures
when this entry was prepared. Cloudflare live-account acceptance remains pending.

Current source adds `zel vercel` / `zel vc`, `deploy vercel`, saved `publish`,
`ship --provider vercel`, and `ci --deploy vercel`. Destination precedence is
explicit `--site` / `--site-id`, `VERCEL_PROJECT_ID`, then same-provider
`publish.site`. IDs (`prj_…`) and project names work. Provider changes do not
reuse another provider's saved destination or environment ID. The shared saved
site deserializer now enforces string typing, with destination syntax checked
by the selected provider when Site loads; Netlify validation stays strict.

The native adapter lives in `cli/crates/zellij-cli/src/vercel.rs`. It uses
`VERCEL_TOKEN` and optional `VERCEL_TEAM_ID`, checks an existing project before
building, then uses the shared temporary static export builder. Portable output
requires no external runtime; hosted output retains container requirements and
URL/feature overrides. Both preserve source files and prior output. No Vercel
CLI, stored login, account/project creation, configuration/DNS changes or remote
source build is involved. Default deployments are previews; `--prod` is explicit
and ship never persists that choice. Production waits for alias assignment.

The protocol uploads SHA-1-addressed assets to `/v2/files`, then creates exactly
one `/v13/deployments?prebuilt=1` request containing `.vercel/output/static/*`
and generated Build Output API v3 configuration. Content hashes deduplicate
uploads; source/config files and Zellij's ownership marker are excluded. The
routing configuration retains literal `.html` links and adds existing-file
fallbacks for hosted clean paths and directory indexes. Full deployment polling
checks ID, project, target environment, failure/alias status and HTTPS deployment
hostname before reporting readiness. Provider bodies/tokens stay out of errors,
redirects are refused and request/deployment budgets are bounded. Creation is
never automatically retried; uncertain results require dashboard inspection.

Initial limits are 15,000 static files / 100 MiB total; provider plan limits may
be lower. Links/junctions/special files, reserved Vercel config/output paths and
ambiguous control/#/?/backslash paths fail before uploads. Unicode/space/percent
filenames remain intact. Server functions, middleware, custom Vercel rules and
live-account acceptance are outside this milestone. Native publication remains
separate; published rc.1 does not contain these changes.

GitHub generation uses only the Vercel token and project/team variables or secrets
on the publish step; PR verification has no deployment credentials. Explicit
`--provision` copies only `VERCEL_TOKEN` via stdin after writing the workflow.
A compatible rc.2-or-later pin is required, without claiming it is published.
CLI docs, current status and unpublished release notes describe setup and limits.

Official protocol references inspected read-only:
[REST deployment](https://vercel.com/docs/rest-api/deployments/create-a-new-deployment),
[file uploads](https://vercel.com/docs/rest-api/deployments/upload-deployment-files),
[project preflight](https://vercel.com/docs/rest-api/projects/find-a-project-by-id-or-name),
[prebuilt query in the official client](https://github.com/vercel/vercel/blob/main/packages/client/src/utils/query-string.ts),
[Build Output configuration](https://vercel.com/docs/build-output-api/configuration).
Authenticated response fields were cross-checked against Vercel's OpenAPI schema.

Local tests cover protocol/identity/target selection, uploads and duplicates,
credential redaction, redirects, deadlines, failures, output preservation, CLI
aliases/provider isolation, ship retention and workflow secret provisioning.
CI explicitly selects a real hosted-export/mock-upload test, including base paths,
assets and failed-build preservation. Final local workspace checks and final-head
GitHub acceptance are pending at this entry. No local Docker or browser ran.

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


### 2026-11-05 — GitHub Pages branch publication

Current source adds `deploy github-pages`, `github-pages` / `ghp`, saved static
publication, import-and-ship and GitHub workflow generation. The destination is
an existing GitHub.com `owner/repository`; `--site` / `--site-id` overrides
`GITHUB_PAGES_REPOSITORY`, then same-provider `publish.site`. Authentication uses
only `GITHUB_PAGES_TOKEN`, a dedicated token with Contents and Pages write access.
No gh-login discovery, implicit GITHUB_TOKEN selection or external Git process is
used for deployment. Generation still uses Git for repository discovery and
explicit provisioning uses gh stdin, as with the earlier adapters.

This adapter has no preview mode and requires `--prod`, even for a dry-run plan;
CI generation accepts `--prod` or saved `ci.production: true`. Ship saves provider
and destination but never production intent or credentials. The destination must
already use Pages branch publishing (`build_type: legacy`) from `gh-pages /`;
the repository cannot be archived or have gh-pages as its default/source branch.
No repository, branch, site, DNS, protection or settings changes are performed.

The entire gh-pages root is replaced by a static snapshot. Git trees use no base
tree, commits retain the old head as their parent, and the branch is updated with
force:false after another repository/Pages/ref check. Concurrent changes fail.
The command explicitly requests a Pages build and accepts success only for its
published commit, then rechecks configuration/ref. Stale success, missing build,
failed/superseded build and timeouts cannot report success. A failed request can
leave Git objects or a published commit; mutations are never automatically
retried, resumed or rolled back. Ten minutes bounds the upload/publish phase;
individual requests/connects retain 60/15-second limits and reject redirects.
Response bodies/token values are omitted from errors.

Hosted deployments derive invocation defaults for basePath and site origin from
the Pages URL, overriding authored defaults without modifying source. An explicit
prefix must equal or be below the site's mount. Upload strips that mount once,
so /repository links have branch-root files rather than a doubled path. Explicit
site origin remains an SEO override. Portable output stays relative. CNAME comes
from the existing API configuration; conflicting exports fail. .nojekyll is
added and .zellij-output excluded. Git/workflow/Jekyll configuration, links and
special files are rejected. Limits: 5,000 static files, 25 MiB/file, 100 MiB total,
200 distinct binary/large uploads, 6 MiB tree JSON. Small UTF-8 files are inlined
within a 2 MiB budget; other assets use Git SHA-1 blobs and base64 byte upload.

Official protocol references inspected: [Pages API](https://docs.github.com/en/rest/pages/pages),
[publishing sources](https://docs.github.com/en/pages/getting-started-with-github-pages/configuring-a-publishing-source-for-your-github-pages-site),
[Git trees](https://docs.github.com/en/rest/git/trees),
[Git references](https://docs.github.com/en/rest/git/refs),
[Git blobs](https://docs.github.com/en/rest/git/blobs), and
[Git commits](https://docs.github.com/en/rest/git/commits).
Tests use local API fixtures; a real hosted export test is explicitly selected
by CI using its already-running engine image. Local Docker and browser execution
remain excluded. Live Pages account acceptance is pending; no live site or branch
was deployed. Broader provider work includes S3, Fly and Railway; GitLab/CircleCI,
wizard and richer imports remain separate. Native rc.2 and production Netlify
migration remain unpublished.

Prior milestone closure: PR #82 merged as `c8b485c567ecf108bb30771039ad317b75950b8b`
after all 16 checks passed on `b12af984d6bfdc220b8c4e06ad6598fff38c183f` in
`42126815665`; tested/merged tree `c79bd527ff19ccb198ca5eeb8b0bec0589f43c0f` matches.
Post-merge base-image `42130753789` and main CI `42130753939` both passed.
PR #81 post-merge `42123170594` and `42123170846` also both passed. The published
native candidate remains v0.2.0-rc.1; no release action was taken this milestone.


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

### 2026-11-05 — GitHub Pages milestone closure and S3 implementation

PR #83 merged as **`e25d8d9c45e959c61da229fa8d3b2911021ab2f5`** after all
16 checks passed on final head `b11bdd697076a0005423da4967ddc887005c6db1`
in run `42135824730`. Tested and merged tree:
`c872806f12719cf8d366991514c56d837adf8b70`. Attempt 1 completed 15 checks;
the remaining Linux job stalled downloading Ubuntu packages at about 45 kB/s.
Only that unfinished job was cancelled and retried on the unchanged head;
attempt 2 passed. This was infrastructure delay, not a disabled assertion.
Post-merge base-image run `42139995450` and main CI `42139995418` both passed.
Evidence: `/private/tmp/zellij-pr83-premerge.json`, `zellij-pr83-ci.json`,
`zellij-pr83-merge.json` and the final PR description.

Current work adds native AWS S3 website uploads to `deploy s3`, shortcut `s3`,
saved publication, ship and GitHub CI generation. Destination is an existing
commercial-region general-purpose bucket, optionally with an ASCII prefix.
`--prod` is mandatory even for plans; CI may use saved `ci.production: true`.
Environment credentials, `AWS_REGION` and `S3_EXPECTED_BUCKET_OWNER` are required.
Temporary access keys require `AWS_SESSION_TOKEN`. No AWS CLI/profile/metadata
credential lookup or custom endpoint is used. Secrets never enter configuration
or argv. Explicit CI provisioning validates all keys before writing, then sends
required and supplied optional secrets individually over stdin; partial failures
retain the workflow and may have set some secrets.

The native SigV4 adapter checks bucket region, expected owner and website setup
before building. It requires index.html with no redirects/routing rules. It
builds once into temporary output and retains source/previous local output.
Portable keys receive the destination prefix; hosted invocation defaults derive
from that prefix and the website origin. Hosted files already contain the prefix
and are uploaded without duplication. Non-index HTML also gets extensionless
copies for clean links; collisions fail before uploads.

All key ETags/missing states are captured before conditional PUTs. Assets upload
first, then HTML, selected index last. SHA-256 receipts are verified. Configuration
is rechecked before and after upload. S3 publication remains non-atomic; failures
can leave partial changes. There is no automatic retry, rollback, prune, bucket
configuration, public-access/policy/ACL change, DNS or CloudFront setup. Old keys
remain. Website URLs use HTTP; uploads use HTTPS. Success means objects uploaded,
not that public website access was verified. Limits: 5,000 objects, 25 MiB/file,
100 MiB total including clean-route copies, 1,024-byte keys, 512-byte destination
prefix. Redirects are refused, provider bodies redacted, responses bounded to
256 KiB, with 60/15-second request/connect limits and a 15-minute upload budget.

Docs, command help, inventory, contract, current status and draft release notes
are updated together. Tests cover published AWS signing vectors, byte/path
preservation, sessions, conditional writes, failed receipts, drift, quotas,
preflight-before-build, source preservation and explicit multi-secret provisioning.
The hosted real-export fixture is selected only in GitHub CI. No local Docker,
Playwright or live AWS mutation was run. Live AWS/Cloudflare/Vercel/Pages acceptance,
Fly/Railway, other CI systems, wizard and advanced imports remain separate work.
Native rc.2 remains unpublished pending explicit release authorization. Netlify's
hosting-comment hydration issue and production migration remain pending.

Local verification: **528 distinct Rust tests passed** (524 in the full workspace
run, plus four added regression cases in the final focused runs). The final CLI
binary suite passed 78 tests with five real hosted-provider tests intentionally
ignored locally and explicitly selected in CI. The final delivery/import/workflow
suites passed. Warnings-denied Clippy, Rust formatting, strict/deep documentation
(55 routes / 23 examples), repository actionlint and an emitted S3 workflow passed.
Evidence: `/private/tmp/zellij-s3-workspace.log`, `zellij-s3-final-tests.log`,
`zellij-s3-bin-final.log`, `zellij-s3-clippy.log`, `zellij-s3-docs.log`, and
`/private/var/folders/1p/q48lnmt15932t129b48z_mrr0000gn/T/zellij-s3-ci-gjue2ymx/.github/workflows/docs.yml`.
Final-head hosted/native CI and merge remain pending, as does live AWS acceptance.

S3 implementation commit **`db164b5`** contains the adapter, CLI/CI integration,
regression tests and matching documentation. Local verification above covers this
implementation; final-head native/hosted CI and live account acceptance remain
pending. This subsequent context-only commit records the source identity.


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

## 2026-11-06 — Railway closure and GitLab/CircleCI generation

Railway PR #86 merged as `b24b3dc`, after all 16 checks passed on `fdf2f6a` in
run `42178970612` (attempt 2). Windows ARM PowerShell 7 encountered an internal
CLR error in the first attempt; its unchanged-head retry passed. Tested/merged
tree: `5918d4e341abfe3e566f887d218642809ff8d912`. Both post-merge workflows passed
(main `42183211689`, base image `42183211386`). No live Railway deployment or
native release was made.

The next implementation addresses the missing GitLab/CircleCI generators with
standalone configuration files, saved/explicit system selection, protected
publication scopes, native AMD64/ARM64 builds and immutable manifest receipts.
Remote dispatch/provisioning remain GitHub-only; schedules require separate
provider setup. The current documentation records runner prerequisites,
credential scope and live-run limitations. See the corresponding dated durable
context entry for architecture and source references. Final source identity and
verification follow once the implementation checks finish.

Local verification: 564 Rust tests pass, five existing hosted-provider fixtures
remain CI-only. The final parser adjustment passed 11 parser and 29 workflow
tests. Clippy with denied warnings, formatting, strict/deep docs, JavaScript
syntax and actionlint pass. Four generated cases for each new service pass its
official JSON schema; the equivalent GitHub cases pass actionlint. Live CI
service acceptance is distinct from these structural and script checks. Final
GitHub native image acceptance and merge remain pending.

Implementation commit **`9d3ba34`** contains this milestone and its regression
coverage. The following context-only commit records that source identity.

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


## 2026-11-06 — PR #89 closure and doctor preflight follow-up

PR #89 merged as `479eac4b899266311af84a5b4f8f55ea1b881356`; all 16 checks
passed in `42267735005` on `80ba76c5f58f191b33e5dc2f8e753966f105e4e6`,
without retries. Tree equality was verified. Windows x64 and ARM64 each passed
eight upgrade tests and actual self-upgrade acceptance. Hosted full-starter
export passed 26-route/section/media assertions. This closes the pending
acceptance recorded above for sources `608e5e8` and `54f601d`.

The post-merge base-image workflow `42272858763` passed. Main run `42272858943`
passed 15 jobs; Ubuntu portable job `128081619870` timed out downloading
Playwright system dependencies after native tests passed. Only that job was
retried at the unchanged merge SHA. Its result is pending, not presumed green.

Doctor previously probed the runtime before resolving content, could not accept
site/config selectors, and failed without Docker even for a configured portable
site. The current follow-up adds shared selection, native settings preflight,
portable runtime bypass, useful site details on hosted runtime failure, and
structured JSON. It preserves runtime-only diagnostics outside a site, existing
bounded probes and hidden endpoint values. Configuration failures occur before
runtime discovery. Tests use fake clients; no local Docker/browser is started.
See the current doctor reference and durable context for the precise boundaries.
Final source identity and verification follow after local checks.


The PR #89 targeted retry completed: run `42272858943`, attempt 2, all 16 jobs
successful at unchanged `479eac4`. This closes the pending post-merge acceptance
without erasing the first attempt's dependency-download timeout.

Doctor local verification passed **586 Rust tests**, with five existing real
hosted-provider fixtures ignored and reserved for CI; all 16 runtime integration
tests passed. Warnings-denied workspace Clippy, formatting, actionlint and
strict/deep documentation (55 routes, 23 examples) passed. No local Docker or
browser ran. Logs: `/private/tmp/zellij-doctor-{workspace,clippy,docs}.log`.

Doctor implementation: **`1c2091b922e136b70118ba59549eedb00d2c35c0`**. The local verification above
covers this source. Final-head GitHub CI and merge remain pending.


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


Sphinx table browser-fixture follow-up: initial PR #91 run `42314567470` on
`926f76a` failed because the new fixture server rejected `/products/docs`
without a trailing slash. The hosted Sphinx export had succeeded; browser
navigation requested the prefix root and the unchanged HTTP/console assertions
correctly caught its 404. Fix **`aa409e6393344f505655afbfcbb796e571c52554`**
serves that exact root while retaining prefix containment and adds an explicit
root-response check before browser launch. No product parser or error assertion
changed. Node syntax and diff checks passed locally; no local browser or Docker
ran. The corrected final head requires a new complete CI run before merge.


Sphinx table browser acceptance then exposed a product bug in run `42315268885`
on `e910edb`: the common MDX link renderer passed copied download URLs through
Next Link. Its page-data prefetch requested a nonexistent path such as
`/_zellij/assets/imported/docs/notes.txt/__next._tree.txt`, producing HTTP 404.
The table semantics, local image and document navigation assertions had already
passed; the final no-browser-errors assertion caught the download prefetch.

Fix **`ba8a68ecb9704823706967956c8962acc6adfeed`** renders URLs in the reserved
`/_zellij/assets/` namespace as ordinary anchors with the site's basePath.
Page links retain Next navigation; external/fragment handling and source query,
fragment and encoded filename text are retained. This applies to all converters.
Node-only regression checks cover root/prefixed sites and all six importer
fixtures; the CI browser test now clicks the Notes link, requires a native
navigation response and compares the exact bytes. It still rejects HTTP,
console and runtime errors. The [Next Link documentation](https://nextjs.org/docs/app/api-reference/components/link#prefetch)
confirms its production page-data prefetch behavior; the failure log establishes
this reproduction.

All 426 engine tests, workspace typechecks, Node-only conversion rendering/deep
checks for all six importers, strict/deep docs (55 routes, 23 examples), script
syntax and diff checks passed locally. Evidence:
`/private/tmp/zellij-tables-{engine-tests,typecheck,asset-render}.log`.
No local browser or Docker ran. Final-head CI remains required before merge.


## 2026-11-06 — Sphinx table closure and list/CSV directives

PR #91 merged as **`6b3b4b4c25d5f20c5daefd6ffce0c6760f7e5d80`** after all 16
checks passed on **`eda6ab6328d1cf978f7a9a862d08c7b70a56afe0`** in
[42316324469](https://github.com/mylife-inc/Zellij/actions/runs/42316324469),
attempt 1. Tested and merged tree: `6af059aa6f40eaec855351210567f5b0db692eef`.
The table cases passed on Windows/macOS/Linux; browser acceptance confirmed
hydration, semantic cells, image loading, document navigation and native asset
navigation with exact bytes. All six native packages, both image architectures,
combined manifests and broader hosted/browser tests passed. Preservation retained
46 original routes, 282 heading IDs, 82 compatibility mappings and 29 images.
The parser (`2b06d93`), fixture root fix (`aa409e6`) and shared download fix
(`ba8a68e`) are recorded above with their original failures and verification.

Post-merge base-image run `42319859200` passed. Main `42319859257` completed
15 successful jobs; its showcase job `128241510788` hit the 20-minute limit
while downloading Ubuntu browser dependencies, before application/browser tests.
The log ends in the `playwright install --with-deps` apt download, with no test
failure. A targeted retry on unchanged `6b3b4b4` was requested; its outcome is
pending. Native rc.1 remains the newest published candidate; no release or
production migration was attempted.

Current work adds `list-table` and inline `csv-table` to the Sphinx converter.
`cli/crates/zellij-convert/src/sphinx/table_directives.rs` parses the two-level
row/cell bullet structure and strict quoted CSV without new dependencies or
source execution. Both support captions, `header-rows`, `stub-columns` and
`widths: auto`; CSV adds supplemental/multiline `header`, literal Unicode
`delim`/`quote`, `delim: space` and `keepspace`. Short CSV rows are padded,
while list rows must be uniform. A body row and a data column must remain after
headers/stubs. The existing 128-column/10000-row/100000-cell and 16-level limits
apply. The common RST table node now carries a stub count; body row headings
render as `<th scope="row">` through the existing MDX pipeline.

Cell bodies retain the existing supported RST constructs, reference resolution,
asset containment and search behavior. Option parsing now distinguishes a field
from a role at the start of CSV content, preserving `:download:` cells, and reads
CSV header continuation lines without treating them as body data. External CSV
file/URL/encoding options, custom escape rules, character-code notation, numeric
widths, alignment/classes/names, spanning cells and unsupported nested constructs
remain explicit errors before output. No CSV external input is read or fetched.

The shared fixture now contains eight tables, four header groups, 17 column
headings and seven row headings. Native regression coverage checks retained
content, literal code, links, exact assets, dry runs, skip mode, invalid directives,
unsafe paths and the list-table nesting bound. CI's existing browser helper now
checks row-header roles, supplemental headers, padded cells and a second local
image while keeping navigation/download and error assertions. Documentation,
CLI status and the contract describe the accepted syntax and remaining limits.
Local final verification and final-head CI remain pending.

References: [Docutils list-table](https://docutils.sourceforge.io/docs/ref/rst/directives.html#list-table),
[CSV table](https://docutils.sourceforge.io/docs/ref/rst/directives.html#csv-table),
and the [upstream parser](https://github.com/docutils/docutils/blob/master/docutils/docutils/parsers/rst/directives/tables.py).


List/CSV implementation: **`bc70c45296d01c5b7369142db610c1da2fba15f1`**.
Local verification passed **600 Rust workspace tests**, with five existing
hosted-provider fixtures reserved for CI. All 18 Sphinx CLI cases and four new
table-directive unit cases pass. Warnings-denied workspace Clippy, formatting,
strict/deep docs (55 routes, 23 examples), script syntax, and Node-only rendering,
deep checks and searchable-cell assertions across all six converters pass.
Evidence: `/private/tmp/zellij-directives-{workspace,clippy,import,unit,docs,search}.log`.
No local Docker/browser or live provider operation ran. Final-head CI and merge
remain pending; main's unchanged-head showcase retry is still running.


### 2026-11-06 — Directive boundaries and prior main-CI closure

PR #91 main run `42319859257`, attempt 2, now has all 16 jobs successful on
unchanged `6b3b4b4`. Retried showcase job `128249568949` passed. This closes
the earlier dependency-download timeout without a source change; the original
failure and its limited verification are retained above.

Review of the new Sphinx directive parser found that leading blank lines were
being discarded before option parsing. A code block beginning with `:header:`
could therefore be rejected as if it contained directive options. CSV header
continuations could also consume the separator and misclassify following data.
Fix **`6b695697e7b95c476225a29e8aca079d64f6dd6d`** retains that boundary,
requires a blank line between options and content, and rejects unsupported
multiline table captions instead of silently importing them as CSV rows.

Regression coverage checks literal option/include-looking code, multiline CSV
headers followed by inline roles, missing separators and multiline captions.
The shared native/rendered/browser fixture includes the literal field text.
All 19 Sphinx CLI cases and 75 converter tests pass, as do warnings-denied
Clippy, formatting, strict/deep docs (55 routes, 23 examples), script syntax
and Node-only rendering/search checks for all six importers. The final complete
Rust workspace suite passed **601 tests**, with five existing hosted fixtures
reserved for CI. Fresh final-head CI and merge remain pending.

Evidence: `/private/tmp/zellij-directives-final-{import,convert,clippy,docs}.log`
and `/private/tmp/zellij-directives-exact-{workspace,render}.log`. No local Docker/browser
or production/provider mutation ran. PR #92's earlier CI head does not validate
this follow-up; final-head CI is required before merge.

The option/content distinction follows the [Docutils directive block parser](https://github.com/docutils/docutils/blob/master/docutils/docutils/parsers/rst/states.py).


## 2026-11-06 — Sphinx directive closure and nested lists

PR #92 merged as **`0c4d79e80a232b403282d56cb56f2e7841c5fdc5`** after all
16 checks passed on **`2eeb9c6d967274484f7f30015ed1662859c70427`** in
[42324258639](https://github.com/mylife-inc/Zellij/actions/runs/42324258639),
attempt 1. Tested and merged tree: `fe9232807c1325496e1a97b9f99eb479950dd56e`.
All 19 Unix/18 Windows Sphinx tests passed. Hosted browser acceptance covered
semantic tables, column/row headings, local images, navigation and exact native
asset downloads. Documentation preservation retained 46 original routes, 282
heading IDs, 82 compatibility mappings and 29 byte-identical images. Both
post-merge workflows passed on unchanged `0c4d79e`, attempt 1: main
`42326727478` (all 16 jobs) and base-image `42326727358`. Native rc.1 remains
the latest published candidate; no native release or production migration ran.

Nested-list source **`27889f655594059eb528383f3abd23b68acafb2a`** adds
`sphinx/lists.rs` and a recursive `Block::List`. RST list items now retain
nesting, wrapped/multiple paragraphs, literal code, tables, admonitions, images
and references. Supported markers are ASCII and standard Unicode bullets,
decimal `1.`/`1)`/`(1)` forms and corresponding `#` auto-enumerators. Explicit
start values and sequence/style restarts become separate semantic HTML lists.
Empty items are retained. Unicode bullets use display-column indentation,
not UTF-8 byte length. Rendering shares the existing reference/asset and search
pipeline through convert/cv, migrate/mig and ship. No dependency was added.

Malformed indentation or block separators, alphabetic/Roman enumeration,
out-of-range counters and unsupported nested constructs fail before output.
Decimal counters must fit 0–2147483647; each list has at most 10000 items.
Lists share the existing table/admonition depth limit of 16, including empty
items. Existing external CSV, merged-cell, extension and source-execution limits
remain. The conversion reference and CLI contract/status now describe this scope.

A shared native/render/browser fixture covers six unordered lists, five numbered
lists, numbering restarts, Unicode, an empty item, literal code, lists in tables
and a table in a list. The hosted CI helper checks the actual parent/child DOM
structure, numbering, nested media, document navigation and native downloads
while retaining hydration and console/HTTP error assertions.

Local tests caught and fixed a Unicode-marker byte/column mismatch before commit.
The first additional-table rendering assertion assumed a caption had no nested
paragraph element; the corrected assertion checks the retained caption text and
table count, with browser containment checks. Neither failure was waived.
Focused validation passed 23 Sphinx CLI cases and 78 converter unit tests,
warnings-denied workspace Clippy, formatting, strict/deep docs (55 routes,
23 examples), Node syntax and Node-only rendering/search checks for all six
importers. The complete Rust workspace suite passed **608 tests**, with five
existing hosted-provider fixtures reserved for CI. Final-head CI and merge
remain pending. Evidence: `/private/tmp/zellij-lists-{workspace,import,unit,clippy,docs,render-verified}.log`.
No local Docker, browser, GUI or live provider operation ran.

Reference: [Docutils list syntax and indentation](https://docutils.sourceforge.io/docs/ref/rst/restructuredtext.html#bullet-lists).


Nested-list hosted assertion follow-up: initial PR #93 run `42333097022` on
`967df6a` failed in CLI job `128283842248` after the Sphinx export succeeded.
The new list-count assertion scanned the complete exported page (12 unordered
lists, including navigation/sidebar), while its expected six lists describe
only the authored article. The earlier MDX-only render had correctly passed.

Fix **`eb4b83154d3082412bb37d4f178338b6d915e7d5`** requires and selects the
exported `<article>` before counting. Expected counts, numbering, content and
all browser assertions remain unchanged; no product code changed. Node syntax
and diff checks pass. No local browser or Docker ran. The initial run did not
reach the new nested-list browser assertions, so a fresh full final-head CI run
is required before merge.


## 2026-11-06 — Nested-list CI closure

PR #93 merged as **`2ed992d977929ad15509618fcbf163b287139f54`** after
all 16 checks passed on **`1b29b4a52df87c8ad6663f4f7c506aa87d253bbd`**
in [42333764306](https://github.com/mylife-inc/Zellij/actions/runs/42333764306),
attempt 1. The tested and merged tree was
`42ccffe4af1735815377a5840e2b0cfffde6362f`. Hosted browser acceptance passed
nested list structure, numbering, literal code, local images, document navigation,
exact native downloads and hydration/error checks. Documentation preservation
retained 46 original routes, 282 heading IDs, 82 compatibility mappings and
29 byte-identical images. All six native packages and both Linux image
architectures/combined manifests passed. Both post-merge workflows passed on
unchanged `2ed992d`, attempt 1: main `42336073179` (all 16 jobs) and base-image
`42336073120` (all 13 jobs). The initial assertion-scope failure and correction
remain recorded above. Native rc.1 remains the latest published candidate.


## 2026-11-06 — Sphinx alphabetic and Roman numbering

Source **`67b313f955bb7c9ab3d4af71381ab3fef5d019a0`** adds lower/upper alphabetic and Roman list styles to the
shared Sphinx importer. All three punctuation forms and matching automatic
markers retain starts, case, nesting and the existing block/reference pipeline
through native HTML list attributes. Context resolves ambiguous I/V/etc.;
wrapped alphabetic/Roman prose remains prose. Canonical Roman values are
limited to the browser counter range 1–3999 and alphabetic values to 1–26.
Malformed/mixed-case Roman markers and automatic overflow fail before output.
No Python, source evaluation or new dependency was introduced.

The same change fixes a decimal-list bug: an explicit enumerator after `#`
must start a new list even if its value happens to be consecutive, matching
Docutils. It also corrects obsolete overview/quickstart claims that tables and
nested lists are unsupported, and clarifies that the remaining automatic
numbering limitation concerns toctrees. Existing containment, shared depth and
item bounds remain. Other Sphinx extensions, spanning cells, substitutions and
source-execution limits remain documented; this does not complete all imports.

Local focused validation passed **24 Sphinx CLI tests** and **81 converter unit
tests**, warnings-denied workspace Clippy, formatting, strict/deep documentation
validation (55 routes, 23 examples) and Node-only rendering/search for all six
importers. A fixture heading underline initially failed validation; the fixture
was corrected and rerun successfully. The final conversion-notice wording also
passed the 24 Sphinx CLI cases. Full workspace verification passed **612 tests**,
with five existing hosted-provider fixtures reserved for CI. Final-head
hosted/native CI and merge remain pending.

The shared native/render/browser fixture now contains nine ordered lists,
including all four letter/Roman styles. CI additionally checks computed browser
styles on lists and items, nested typed-list containment and a section link,
while retaining existing media/navigation/hydration/error assertions. No local
Docker, browser or GUI ran, and no native release or production migration ran.
Evidence: `/private/tmp/zellij-numbering-{unit-final,import-final,message-check,clippy,docs,render,workspace}.log`.
References: [Docutils enumeration](https://docutils.sourceforge.io/docs/ref/rst/restructuredtext.html#enumerated-lists),
[sequence parser](https://github.com/docutils/docutils/blob/master/docutils/docutils/parsers/rst/states.py),
[Roman browser counters](https://www.w3.org/TR/css-counter-styles-3/#lower-roman).


Numbering browser-selector follow-up: PR #94 run `42338890305` on `863d301`
failed in CLI job `128302002257` after all importer exports and the preexisting
Sphinx browser assertions succeeded. The new `ol[type="a"]` selector matched
both `a` and `A` lists because HTML type attribute selectors compare without
case, causing a count of two where one exact-case sequence was expected.

Fix **`496f97984810ed87b382eef536cc7b17f216c95b`** reads the four typed lists' exact DOM attribute values
and compares them in JavaScript. It retains exact starts, direct item counts,
computed list/item styles and parent-child type assertions, and also requires
exactly four typed lists. Product output and Rust code are unchanged. Node
syntax and diff checks pass. The earlier run did not reach the new computed
style assertions, so fresh final-head CI remains required. No local browser
or Docker ran. The 612-test Rust result remains valid for unchanged Rust code.


Numbering display follow-up: PR #94 run `42339535816` on `00be13b` failed in
CLI job `128303947326`. Exact-case DOM selection succeeded, but the first
alphabetic list and its item computed to `list-style-type: none`. Inspection
identified Tailwind Preflight's general list reset, which also hides ordinary
prose bullet/decimal markers. The semantic attributes alone did not establish
visible numbering; the initial implementation's browser acceptance was incomplete.

Fix **`21c663b69a1c2219b2123915f863837020c84b5b`** adds a prose-only `list-style-type: revert-layer` rule inside
Tailwind's `base` layer. This undoes that layer's reset while preserving HTML
presentation hints (including case-sensitive ol types) and native nested bullet
styles. A plain `revert` would also discard author presentation hints. Navigation
and component styling retain their existing rules. CI now also checks ordinary
bullet/decimal styles and typed items' `display: list-item`; none of the original
computed-style expectations were relaxed.

Local Tailwind/PostCSS compilation passed with the rule retained in the built
stylesheet (`/private/tmp/zellij-numbering-styles.css`), as did script syntax and
diff checks. Rust code remains unchanged and its 612-test suite remains valid.
The stylesheet change requires fresh final-head hosted browser acceptance.
No local browser/Docker ran. Final CI and merge remain pending.
References: [Tailwind Preflight](https://tailwindcss.com/docs/preflight),
[CSS presentation-hint origins](https://www.w3.org/TR/css-cascade-5/#preshint),
[layer rollback](https://www.w3.org/TR/css-cascade-5/#defaulting-keywords).


## 2026-11-06 — Numbering and prose-marker CI closure

PR #94 merged as **`3f14b99b533e9e63b6f25d40c8ce00c2a2cb4fc2`** after all
16 checks passed on **`fcbaa6b91e477a177f604a710f5c867228cbe9ff`** in
[42340198858](https://github.com/mylife-inc/Zellij/actions/runs/42340198858),
attempt 1. Tested/merged tree: `f13f7b7394f52a28a4d28ce84f27709404df6db8`.
Hosted browser checks passed exact-case alphabetic/Roman styles, ordinary
bullet/decimal markers, list-item display, nesting, media, navigation and native
downloads with no hydration errors. Sphinx tests passed 24 cases on Unix and
23 on Windows. All six native packages and both image architectures passed.
Documentation retained 46 original routes, 282 heading IDs, 82 compatibility
mappings and 29 byte-identical images. Both post-merge workflows passed on
unchanged `3f14b99`, attempt 1: main `42342065734` (all 16 jobs), base-image
`42342065709` (all 13 jobs). Earlier selector/display failures remain recorded.
Native rc.1 remains the latest published candidate; no release or production
migration ran.


## 2026-11-06 — Sphinx reusable text and images

Source **`958b87743468d47057a967e8e91654cad00fb787`** adds document-local
`replace`/`image` substitutions, forward/nested references and named substitution
hyperlinks to the shared Sphinx converter. Names normalize whitespace, prefer
exact case and otherwise require a unique lowercase match. Images retain alt
text and direct/named targets. URL targets ending in underscores remain URLs.
Literal `version`/`release` values come from configuration and can be overridden
by authored definitions. Code blocks, inline code and escaped pipes stay literal.
Existing reference/asset handling applies through convert/cv, migrate/mig and
ship, including copy/skip/dry-run containment checks. No source is executed.

All definitions in published documents are validated before output, including
unused ones. Missing/ambiguous names, duplicate definitions, cycles, nested links
and unsupported constructs fail. Bounds are 1000 authored definitions per page,
256-byte names, 64 KiB replacement paragraphs, and (for each validation/render
pass) depth 16, 10000 references and 4 MiB processed expansion text. Headings and
navigation labels still need plain text; global transforms, anonymous links,
date/unicode and unsupported options remain manual migration tasks. The current
conversion docs, quickstart, contract and CLI status describe this scope.

The shared MDX pipeline marks Markdown images within prose for inline display,
including linked images; standalone figures keep their block presentation.
This corrects the CSS reset's default block behavior for inline images without
adding browser JavaScript or dependencies. The native/render/browser fixture
covers formatted reusable text, nested and forward definitions, copied images,
section/download links, lists/table cells/captions, literal code and page scope.

Focused checks pass **28 Sphinx CLI cases**, **86 converter unit tests**,
**427 engine tests**, workspace type checking, warnings-denied Clippy,
formatting and engine package build. All six converters pass Node-only
render/search checks; strict/deep docs retain 55 routes and 23 examples.
The final-source full Rust workspace rerun is in progress; its result and
final-head hosted/native CI must be checked before merge.

Local regression work caught a discarded blank separator that could admit a
second replacement paragraph; parsing now rejects it. The first rendered-code
assertion assumed syntax highlighting left a directive as contiguous HTML text;
it now checks the code's retained text, with the same browser assertion. Clippy
also caught test-module placement, corrected before the final lint pass.
Evidence: `/private/tmp/zellij-substitutions-{import-exact,unit-final,engine-full,typecheck,package,clippy-final,render-exact,docs,workspace-final}.log`.
No local Docker/browser/GUI, native release or production/provider mutation ran.
References: [Docutils substitution definitions](https://docutils.sourceforge.io/docs/ref/rst/restructuredtext.html#substitution-definitions),
[replacement text](https://docutils.sourceforge.io/docs/ref/rst/directives.html#replacement-text).


#### 2026-11-06 — Substitution final-suite and inline-layout review

The final Rust workspace run on `1041870` completed successfully: **621 passed,
5 ignored**, across 51 suites. Source **`26d2d01`** then corrected the shared
image-layout helper after a new SSR regression reproduced missing inline classes
in compact Markdown lists and GFM table cells. It recognizes text in the same
inline flow, excluding nested blocks, so image-only figures stay blocks even
when a nested list contains text. The expanded regression passes, as do all
**427 engine tests** and workspace type checking. No Rust source changed after
the full Rust result. Earlier passing checks remain recorded above.

Evidence: `/private/tmp/zellij-substitutions-workspace-final.log`,
`zellij-substitutions-image-review.log` (reproduced failure),
`zellij-substitutions-image-review-fixed.log`,
`zellij-substitutions-engine-review.log`, and
`zellij-substitutions-typecheck-review.log`. Initial PR #95 CI `42348865898`
on `1041870` does not validate this follow-up. Final-head hosted/native
acceptance and merge remain pending. No local browser/container was run.


## 2026-11-06 — Substitution merge and post-merge follow-up

PR #95 merged as **`4dcb12c99dffd30d86f20fdeed9c609c465aa7f7`** after all
16 checks passed on **`d9b47a0e0659aa2c4097dded139b7a1c03a1f8ec`** in
[42349216986](https://github.com/mylife-inc/Zellij/actions/runs/42349216986),
attempt 1. Tested/merged tree: `f1a0108156e95189cc86b5be30ac2759a2997a6a`.
Hosted browser acceptance passed substitution text/markup, inline/standalone
images, named/image navigation, literal code and exact native downloads, with
no captured hydration errors. Sphinx cases passed 28 on Unix and 27 on Windows.
All six native packages, both image architectures and combined manifests passed.
Documentation retained all 46 original routes, 282 heading IDs, 82 compatibility
mappings and 29 byte-identical images. Local main matched the tested tree.

Post-merge base-image run `42351594623`, job `128340710709`, built maths/amd64
successfully but failed actions/upload-artifact during FinalizeArtifact with
`ECONNRESET`. A targeted failed-job retry on unchanged `4dcb12c` passed all 13
base-image jobs (attempt 2), including published-image acceptance. Main run
`42351594650` has 15 passing jobs and CLI acceptance still running at this
checkpoint. This was an artifact transport failure; no product fix or relaxed
assertion was needed. Native rc.1 remains the latest published candidate.


## 2026-11-06 — Native site-creation wizard

Source **`b556b88`** adds opt-in `new portable` / `init -i` and `new docs` /
`nd -i` creation. Native prompts choose name, embedded-catalogue theme, starter
and a separate hosted guide. Explicit flags skip matching questions; ordinary
commands retain their defaults and never prompt. Names/themes validate before
writes, fixing hosted creation's acceptance of invalid values. The shared
line-oriented prompt remains on stderr, including under quiet; EOF/:q cancels.

A read-only scaffold preflight checks the root/common configuration before
questions. The final plan checks every destination before writes; cancellation
with force leaves old content intact. Portable output still has no overwrite
flag. Existing templates generate the files, with no new dependency, client code,
installer, network or runtime. Fixed relative next-step suggestions also avoid
interpolating a destination path into a copied shell command. Docs, shortcuts,
CLI status and contract describe exact defaults, scope and remaining page/app
wizard work.

All **629 Rust workspace tests pass** (5 existing hosted-provider cases ignored
locally and selected in CI), including 8 site-wizard cases on Unix and all prior
section/starter/alias tests. Tests cover every starter/guide combination, byte
equivalence with flags, literal Unicode metadata, CRLF and numeric selections,
invalid answers, quiet prompts, cancellation at every question, force preservation,
symlink containment and native portable builds without tools on PATH. The initial
new HTML assertion used `data-theme` instead of the established
`data-zellij-theme`; corrected to the actual contract before final verification.

Warnings-denied Clippy, formatting, workflow actionlint and script syntax pass.
Node-only engine checks validate full ordinary/wizard starters with all 23
section types and exact media; strict/deep project docs retain 55 routes and
23 examples. CI extends the existing Linux/macOS/Windows fixtures: portable
wizard starters are built, moved and viewed offline with/without JavaScript,
and a hosted wizard full starter exports all 26 routes. Final-head CI and merge
are pending. No local Docker/browser/GUI or live provider mutation ran.
Evidence: `/private/tmp/zellij-site-wizard-{focused-final,workspace,clippy,engine,docs}.log`.


### 2026-11-07 — Windows installer harness desktop-startup correction

Initial PR #96 CI `42353311383` on `306f16c` failed in Windows ARM package job
`128345889969`. The Windows PowerShell installer checks passed; the subsequent
`pwsh` invocation crashed with `Internal CLR error (0x80131506)` in
`TaskbarJumpList.CreateElevatedEntry`, on its background desktop initialization
thread. The failure was retained; the installer assertions were not relaxed.

Source **`4df0d31`** starts only automated PowerShell child processes with Node's
`windowsHide: true` in native package/installer fixtures. The PowerShell source
checks startup visibility before its JumpList COM calls; libuv maps this option
to `SW_HIDE`, so the correction targets that desktop-only path. The CLI and
console/signal fixture launch behavior is unchanged. Both Windows shells still
run the full checksum, malformed-archive, preservation, installed-binary and
upgrade checks. There is no catch-and-ignore or automatic retry around assertions.

Node syntax and diff checks pass. The 629-test Rust result remains valid for
unchanged Rust code; final-head Windows/native and full CI must pass before
merge. No local browser/container/GUI ran. References:
[PowerShell JumpList guard](https://github.com/PowerShell/PowerShell/blob/master/src/Microsoft.PowerShell.ConsoleHost/WindowsTaskbarJumpList/TaskbarJumpList.cs),
[libuv process startup](https://github.com/libuv/libuv/blob/v1.x/src/win/process.c).


### 2026-11-07 — Preserve hosted name compatibility in the site wizard

PR #96 run `42353774620` on `05a412e` passed 15 jobs, including all six
native packages. Windows ARM job `128347405063` passed both `powershell.exe`
and `pwsh` installer/checksum/archive/upgrade checks after `4df0d31`; its
console and signal assertions stayed unchanged. All three portable-platform
jobs passed wizard-generated standard/minimal offline navigation, assets,
search and no-JavaScript acceptance in Chromium and WebKit.

The CLI job `128347404872` failed at the existing scaffold-name acceptance:
new validation rejected a hosted `--name` containing newline/tab characters.
Those names were already supported through escaped YAML, so treating them as
invalid was a compatibility regression. Source **`65de2c8`** restores literal
hosted name values, including with `-i` when the name flag skips its question.
Interactive name answers and portable names still exclude control characters;
all names must be nonempty, and themes must be in the embedded catalogue.
The user reference and CLI contract now distinguish these cases. This corrects
the broader name-validation claim in the initial wizard entry above.

All 9 focused wizard tests pass, including the new literal-name/interactive
regression. The unchanged Node-only scaffold acceptance passes all 40 Rust
starters, 10 Node starters and 10 guide titles with exact metadata and rendered
links. Strict/deep project documentation validation passes 55 routes and
23 examples. Full Rust rerun and final-head hosted/native CI remain pending;
the earlier run cannot validate this correction or the hosted wizard export.
No local Docker/browser/GUI, native release or provider mutation ran.
Evidence: `/private/tmp/zellij-site-wizard-compat-{focused,scaffolds,docs}.log`.

PR #95's post-merge main run `42351594650` subsequently passed all 16 jobs on
unchanged `4dcb12c`, so both its post-merge workflows are now green. Its base
image workflow retained the previously recorded artifact-upload retry.


## 2026-11-07 — Site wizard merge and page authoring follow-up

PR #96 merged as **`0849faf348526ef8a27225f89fc01f0e032864ed`** after all
16 checks passed on `b98811bd894bf5eb82d79f78bba35877167e8870` in
[42356363457](https://github.com/mylife-inc/Zellij/actions/runs/42356363457),
attempt 1. Tested/merged tree: `136c390adae14417580ab323d44c778f192d0527`.
The final local suite passed 630 Rust tests (5 existing hosted cases selected
separately in CI), and CI passed all 427 engine tests. Hosted literal names
were restored by `65de2c8`; Windows installer startup was corrected by `4df0d31`.
All six native packages passed, as did wizard-generated portable browser/no-JS
acceptance on Linux/macOS/Windows and the 26-route hosted wizard export.
Documentation retained all 46 original routes, 282 heading IDs, 82 compatibility
mappings and 29 byte-identical images with no captured browser errors.

Post-merge base-image run `42357950120` passed all 13 jobs. Main run
`42357950211` has 14 passing jobs, with Windows portable and CLI acceptance still
running at this checkpoint; no failures. Native rc.1 remains the latest published
candidate. No native release or production/provider change was made.

Source **`dd7c625`** adds optional `new page -i` / `np -i`. It asks only for a
missing name and missing ordered section list, accepting exact catalogue names
or displayed numbers; Enter selects hero. Unknown/duplicate types retry, while
invalid supplied selections fail before questions. Ordinary commands keep their
required name, default hero, templates and printed registration report.
The shared Prompt stays on stderr, including with quiet; the pages.yaml report
stays on stdout. EOF/:q/interruption during questions writes no section files.
The entire writer plan still preflights collisions and containment before writes.
Authored pages.yaml/menu.yaml remain untouched; manual registration, TODO content
and placeholder media review are explicit in the docs. No dependency, runtime,
network or new renderer behavior was introduced.

All 8 focused page-wizard tests pass on Unix (7 portable to Windows), covering
flag equivalence, ordered references, Unicode/CRLF, numeric choices, defaults,
aliases, quiet reports, cancellation, late collisions, symlinks and global
site/config/cwd selection. The 7 existing alias and 10 safe-write cases pass.
Initial new test fixtures used the nonexistent type `features` instead of
`feature-grid` and an unsupported custom content-folder setting; corrected to
the established catalogue and content/ convention before the passing run.
Node-only engine acceptance validates a wizard page with all 23 types in order,
checks exact templates and preserves authored registration bytes. CI runs this
on all three native OSes and checks the ordered section markers in hosted HTML.
Warnings-denied Clippy, formatting, actionlint, diff checks and strict/deep docs
(55 routes, 23 examples) pass. Full Rust workspace rerun and final-head CI/merge
remain pending. No local Docker/browser/GUI ran.
Evidence: `/private/tmp/zellij-page-wizard-{focused,focused-fixed,engine,clippy,docs-final,workspace}.log`.
App wizards, arbitrary YAML editing, broader imports and live-provider acceptance
remain separate work; the published rc.1 does not include this feature.


## 2026-11-07 — Page wizard merge and app creation follow-up

PR #97 merged as **`c10c1e1f822bfe6649a2f7ef8715137ca9b6b23c`** after all
16 checks passed on `ad9e7672d3d3720fee1fc4e7227e374f826200e3` in
[42359044185](https://github.com/mylife-inc/Zellij/actions/runs/42359044185),
attempt 1. Tested/merged tree: `8ecc554b938d75b959a129e0a1d1222a961b1207`.
The local suite passed 638 Rust tests (5 existing hosted cases selected separately
in CI), and CI passed all 427 engine tests. All six native packages, both image
architectures and the combined manifests passed. The real hosted export rendered
the wizard page's 23 sections in their selected order. Documentation retained all
46 original routes, 282 heading IDs, 82 compatibility mappings and 29 byte-identical
images with no captured browser errors. Both post-merge workflows passed on their
first attempts: main `42361341195` (16 jobs), base image `42361341102` (13 jobs).
PR #96's main `42357950211` also completed successfully on its first attempt;
both of its post-merge workflows are green.

Source **`326c2b3`** adds optional `new app -i` / `na -i`. It asks only for missing
engine path, content mode/source, display name, dev/start port and optional
features. Existing hosted content stays referenced in place; the starter choice
creates docs/zellij inside the new wrapper. Flags skip their questions, numeric
choices are supported, and invalid answers retry. Standalone --features none
explicitly selects no optional dependencies. The shared single-choice Prompt
retains the site wizard's behavior and stderr/quiet semantics.

All planning is read-only. EOF/:q/interruption while answering writes no wrapper,
and the existing full-plan writer still enforces destination, overlap, symlink
and collision checks before writes. Explicit configuration and --cwd semantics
are preserved. No package installation or runtime/server starts during creation;
the engine must already be built. Existing app creation's nonempty-destination
error now says to choose another destination instead of suggesting unsupported
--force. Ordinary flags and generated wrapper templates retain their behavior.

All 12 app-scaffold tests pass on Unix (10 are portable to Windows), including
both content modes and all feature choices, byte equivalence with flags, inherited
ports, quiet prompts, Unicode/CRLF, numeric selections, explicit config/cwd,
cancellation at each question, collisions and symlink preservation. Existing
alias/site/page/section tests pass (42 focused cases total). One initial expected
feature-metadata order was corrected to the existing maths/diagrams order; product
ordering was unchanged. A wizard-generated starter with the real built engine
passes strict/deep validation without installing dependencies. Warnings-denied
Clippy, formatting, Node syntax and diff checks pass; strict/deep project docs
validate 55 routes and 23 examples. The final Rust workspace suite passed all
646 tests, with 5 existing hosted cases selected separately in CI.

CI's existing native app acceptance now installs and builds wizard-generated
starter/external wrappers and tests production browser navigation, reload, media
and search on Linux/macOS/Windows. Final-head CI and merge remain pending. No
local Docker/browser/GUI, native release or provider mutation ran. Published
rc.1 predates these wizards; arbitrary YAML editing, broader imports and live
provider acceptance remain separate work.
Evidence: `/private/tmp/zellij-app-wizard-{focused-fixed,final-focused,engine,clippy,docs,workspace-final}.log`.


## 2026-11-07 — App wizard closure and structured section authoring

PR #98 merged as **`2d7e4a58a16fb379290109f67950266e7fb982f9`** after all
16 checks passed on `268c9361ae12940cfd135f59a9e5d8deb980671e` in
[42390317045](https://github.com/mylife-inc/Zellij/actions/runs/42390317045),
attempt 1. Tested/merged tree: `b26ef496a637bc38bb171fd30f122ad6318fa3a4`.
The local suite passed 646 Rust tests (5 existing hosted cases selected separately
in CI), and CI passed 427 engine tests. Wizard-generated wrappers passed real
installation, production build, navigation, reload, media and search on Linux,
macOS and Windows. All six native packages, both image architectures and combined
manifests passed. Documentation retained all 46 original routes, 282 heading IDs,
82 compatibility mappings and 29 byte-identical images without captured browser
errors. Both post-merge workflows passed on attempt 1: main `42392744587`
(16 jobs), base image `42392744541` (13 jobs). Remote main and native releases were
refreshed before the follow-up; rc.1 remains the latest published candidate.

Source **`f6e2c7e`** extends new section / ns with repeatable --set-json FIELD=JSON
and --unset FIELD. Complete JSON values, including objects and lists, replace the
selected field after embedded-schema validation. Optional properties can be
removed without creating missing parents; required properties, list elements,
unknown fields and out-of-range indexes fail. Engine defaults still apply to
omitted settings. These options create new sections, not edits to existing YAML.

Processing order is counts, JSON replacements, scalar refinements, then removals.
A scalar/removal can refine a descendant of a JSON object/list. JSON replacements
cannot overlap; scalar JSON values cannot be converted to objects by a descendant
override. Duplicate paths, contradictory counts (now including --tiers), and
media selections that discard another supplied value fail before writing.
Complete tiers/tier-object values suppress the highlight question and conflict
with --highlight; their highlighted values can be supplied directly. The wizard
retains complete subtrees, refinements and omissions. Matrix and media rules,
new-file output, cancellation, symlink and collision protection remain in force.

All 12 section-authoring CLI tests pass on Unix (11 portable to Windows), covering
complete nested values, typed scalars, deterministic refinements/removals, invalid
JSON/schema/path cases, conflicts, quiet/cwd, cancellation and preservation.
The 7 library authoring cases and existing alias/site/page/app tests pass (48
focused CLI cases total). Node-only acceptance round-trips all 23 authored section
types through JSON and the engine schemas, including literal Unicode/multiline
text and explicit omissions. The expanded 25-route template fixture validates a
structured hero; CI additionally asserts its final headline, link and anchor in
hosted HTML alongside the unchanged 23-section wizard page.

Warnings-denied Clippy, formatting, Node syntax, actionlint and diff checks pass;
strict/deep project docs retain 55 routes and 23 examples. The complete Rust
workspace passed 655 tests, with 5 existing hosted cases selected separately in
CI. Final-head CI and merge remain pending. No local Docker/browser/GUI,
native release or live provider mutation ran. User docs, CLI contract, specification
and status distinguish new-section authoring from general existing-file editing.
Evidence: `/private/tmp/zellij-structured-section-{focused,focused-final,library-final,engine,templates,clippy-final,docs,workspace}.log`.


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


### 2026-11-09 — Refreshed rc.2 preparation (unpublished)

PR #103 merged public asset URL linking as
`fa0b20683855df69b30d0dde1366c22fe0bf3e15`, with the identical tested tree
`9c6250dd38090ae996e9542f0a65a99b20ac7a6c`. All 16 checks passed on final
head `ff90234de9d80af75de991e7087e51515acad980` in run `42557158727`.
Both post-merge workflows passed: CI `42560161757`, Base image `42560161729`.

The candidate notes now cover source through PR #103, including all six
importers, public asset URLs, expanded Sphinx support, provider adapters,
GitLab/CircleCI generation, native wizards and recent bug fixes. The old rc.2
target `0fbf0a1` is superseded; its 2026-11-02 evidence remains historical.
Source and lockfile still agree on unpublished `0.2.0-rc.2`. Both candidate
tags are absent, the shared release secret is present (permissions untested),
and generated release pages pin installers and all four candidate image tags.
No stable tags/shared installer files or production documentation are changed.

The latest main run passed all 16 jobs, including installation of six native
packages on their matching OS/architecture. All six downloaded artifact ZIP
digests, contained package checksums and single-executable archive contracts
were independently verified. These baseline CI packages are not public-release
acceptance; release builds need their own final hashes. All 29 local release
policy/download tests pass using fake publication commands and loopback fixtures.
No local container, browser, live provider deployment or native publication ran.

See `specs/evidence/zellij-rc2-preparation-2026-11-09.json` and the refreshed
`specs/releases/v0.2.0-rc.2.md`. This documentation preparation still needs its
own final-head CI. The exact resulting reviewed merge commit must be recorded
in the PR closure and approved before publication; the earlier approval-review
rejection explicitly required authorization for public prerelease tags/assets.
Known live-provider, runner, importer and Netlify limits remain in the release
notes and CLI status. The graph remains a pre-asset-link CLI snapshot and excludes
root `examples/**` under the user's explicit graph scope.


### 2026-11-09 — rc.2 publication and delivery acceptance refresh

The user explicitly approved `v0.2.0-rc.2` from
`54e636faf2162c36e50a91507f1f35be63a660d0`. Preparation PR #104 passed all 16
checks on `cb9b863aa78eaeb7aa09026b5f9f33799601eead` in run `42762346632`;
its merge retained tree `189c93d4edba6262abf15f06044b1178d14f8928`.
Both post-merge workflows passed: CI `42765943453`, Base image `42765943694`.
The standard release helper passed typechecking, schema drift, formatting,
warnings-denied Clippy, 453 engine tests and 681 Rust tests. Five real hosted
cases were intentionally selected in CI, not run locally. The approved tag
was pushed without modifying the reviewed source.

Release run **`42768544795` passed all 29 jobs**. Native archives/installers are
published in `mylife-inc/Zellij` and `mylife-inc/releases`; all six native
public-download/install jobs passed. All four engine variants are published
for Linux AMD64 and ARM64; both native public-image jobs passed. Public archive
checksums match across repositories, and attached installers match the tagged
source. The source tag and public image digests are fixed in
`specs/evidence/zellij-rc2-release-2026-11-09.json`. Production documentation and
shared stable installer updates were skipped. The old rc.1 release and the
historical rc.2 preparation/approval-rejection records are preserved.

Current follow-up retargets six saved delivery fixtures and their generated
GitHub workflows to rc.2, with dedicated new static/Next acceptance image tags.
Active CLI/quickstart documentation now installs and describes rc.2; remaining
provider, runner, importer and Netlify limitations stay explicit. All six
workflows match actual CLI generation; strict/deep docs pass (55 routes and 23
examples), and all 33 release/download/HTTP helper tests pass. This follow-up
still needs final-head PR checks and manual delivery acceptance after merge;
public package acceptance is not evidence of those separate delivery runs.

No local container/browser/GUI, production Netlify deployment or domain change
ran. Other provider secret/variable names are absent from this repository; the
user has been asked for an existing non-secret live-test destination while the
release and approved Netlify staging work continue.


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
