# Zellij — implementation audit and completion backlog

**Date:** 2026-10-16 (America/Los_Angeles)  
**Audited code:** local `main`, `6356b05deaddf5421b24d7c8a1e30f868d4923c0`  
**Scope:** study source/documentation, establish reusable context, identify
unfinished work and defects. **No implementation fixes or remote changes.**

Start with [ZELLIJ_CONTEXT.md](../ZELLIJ_CONTEXT.md) for the architecture and
source map. This report records what was observed at this commit; future fixes
should be recorded with their commit and verification rather than rewriting
these historical observations as if they never occurred.

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
