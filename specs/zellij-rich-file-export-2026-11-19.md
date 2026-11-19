# Rich file export — 2026-11-19

The native `portable` renderer met basic double-click reading needs but did not
meet the user's requirement for the same rich website offline. This is a product
gap, not a broken manual export. The new hosted **file** target addresses it in
the shared engine. It is local source work after rc.11, not a published release.

## Implementation

Fixing commit: **`053fbfc`** (rich file renderer and linked-heading correction).

`zel build <site> --target file` uses the normal hosted validation and Next export,
then runs `zellij export-offline` before staged output publication. A capability
check rejects old engine images before rendering. The existing static target
continues to produce its HTTP-oriented Next output.

The exporter evaluates existing server components at build time and serializes
their React element tree with explicit, non-executable value tags. An enumerated
registry retains the engine's interactive component boundaries. Both SSR and
browser hydration use those same components. The registry has a coverage test
so new client components cannot silently become non-interactive offline.

Offline adapters replace Next links, routing and images with normal local file
navigation. Routes become `route/index.html`; asset URLs and CSS references are
relative. The export keeps the normal CSS, fonts, section views and guide views.
The page payload is escaped inert JSON, not evaluated Markdown or JavaScript.
Classic scripts supply the browser runtime and lazy search/diagram libraries;
there is no runtime file fetch, ES-module loading, hidden server or relaxed
browser security flag. Full dependency notices accompany the bundles.

Unused Next scripts and RSC text are removed. Old flat route HTML becomes local
redirects. Analytics and development polling are omitted. Remote media, external
links and service integrations are not downloaded or made available offline.
Custom Next server endpoints and arbitrary application components are outside
this engine-content exporter; non-serializable component props fail explicitly.

The linked-heading defect recorded in the November 18 context is also fixed:
authored heading links remain siblings of their permalink, avoiding nested
anchors and hydration errors. Its initial finding and private workaround remain
in the dated history.

## Verification

- Engine package build and TypeScript check pass.
- 53 scoped engine tests pass, including URL encoding/base paths/traversal,
  inert script escaping, component serialization/IDs/registry and linked headings.
- Scoped Clippy passes with warnings denied.
- 153 Rust tests pass; five existing tests remain ignored. Runtime requirements,
  target inventory and template compatibility are covered.
- The public showcase's 16 routes pass in Chromium and WebKit under `file://`,
  from a directory with spaces and non-Latin characters. HTTP requests are
  blocked; search navigation and images pass. A second export compiled with
  `/company/docs` as its deployment prefix also passes. CI runs file acceptance
  separately.
- A private 200-route fixture passes with 64 diagrams and no recorded page errors,
  missing images or failed requests. Its 15 themes × two modes, 18 menu icons,
  full-width desktop panels, mobile drawer/back/focus, search and cross-page
  theme persistence pass in Chrome and WebKit.
- The private homepage's 1440 × 1000 screenshots match its existing HTTP site
  pixel for pixel in each browser. Private source data and evidence stay outside
  this repository.
- Real Docker builds pass for the new target and the client regeneration helper.
  An old engine fails the capability guard and preserves the previous output.
  A fresh extracted private ZIP regenerates from a relocated directory with
  spaces/non-Latin characters, with all 200 route HTML files byte-identical.
  The first Docker output attempt under `/private/tmp` encountered Colima's
  unshared mount protection; repeating under the shared home directory passes.

The client helper intentionally pairs verified rc.11 Mac/Windows binaries with
an archived local engine whose postbuild runs the exporter. This enables the
handoff before a public CLI release; it is not a claim that stock rc.11 has the
new target. Full regeneration was executed on Intel macOS. Windows/ARM native
execution and a public release remain separate checks. No client material belongs
in public tests, commits, images or release artifacts.

## Follow-up: duplicated headings during scroll

Fixing commit: **`8021a66`**.

A consumer found a section heading displayed three times. The authored section
and generated HTML contained one copy. `decode()` had always passed the wire
key to `createElement`, including `null` for unkeyed elements. React converts an
explicit null key to the string `"null"`; unrelated siblings therefore collided
during reveal state updates. This could duplicate visible content without a
browser exception. The decoder now omits the key property when it is absent,
while preserving an explicitly authored literal `"null"` key.

The initial acceptance results above remain historical evidence, but their
initial viewport comparisons and error checks did not cover this state-driven
duplication. A new unit regression failed on the original decoder and passes on
the fix. The public file-browser harness compares section header text against
the source HTML before and after scrolling every visible header into view.

Verification: 54 scoped engine tests, TypeScript/package build, and all 16
showcase routes in Chromium and WebKit pass. The consumer homepage also passes
in both browsers at desktop and mobile sizes with HTTP blocked: repeated scrolls,
audience tabs, theme switching and search retain exactly one copy of each
authored heading. The local client engine archive and generated site were
rebuilt. A fresh offline crawl also passes all 200 consumer routes and 64
diagrams with no missing images, failed requests or page errors. Checksums
confirm all 443 files in the editable source collection and all four native
binaries are unchanged. No private content was added to the public regression
fixtures.
