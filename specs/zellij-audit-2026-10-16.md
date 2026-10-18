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
