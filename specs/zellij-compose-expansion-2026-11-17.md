# Composer layers, semantic content and scroll stages — 2026-11-17

Baseline `c8e9842`, source rc.11, public rc.10. The user explicitly authorized
all three remaining capability groups. This increment retains format version 1
and the finite tree (three nested groups, 128 nodes, one primary heading).

## Contract

- `layers` lays content over an image or user-controlled silent video backdrop.
  Nine named `place` values position distinct children in separate desktop grid
  cells; duplicate cells fail validation. Groups collect content sharing a cell.
  Narrow containers and print retain source order. Backdrop playback is opt-in,
  with an accessible control outside the foreground grid and a still poster.
- Semantic leaves: `prose` (Markdown, no MDX/raw HTML), `list`, `quote`, `icon`,
  `code`, `table`, `disclosure`, `carousel`, `gallery`. Reuse the existing carousel
  track and lightbox. Tables retain row/column headers; code remains selectable;
  galleries retain ordinary image links without JavaScript. No arbitrary section
  embedding or authored executable code is introduced.
- Root/group `scroll` declares a bounded range and transform/opacity endpoints,
  optional pinning and scroll distance. It operates only with expressive motion,
  a wide viewport and content that fits the usable viewport. No nested scroll
  stages. Reduced motion, print, forced colors and no-JS retain ordinary content;
  focused content is never faded/transformed. Cleanup restores ordinary layout.
- Native authoring and engine share range, placement, table and nested-stage
  constraints; schema annotations preserve deep asset/link diagnostics.

## UI review

| Before | After | Why |
| --- | --- | --- |
| Composer media and copy only sit beside one another | Bounded backdrop layouts and named foreground cells | Enable layered stories without hiding mobile reading order or focus targets |
| A text grid cannot provide table, disclosure or dialog semantics | Explicit semantic nodes and shared interaction components | Preserve meaning and keyboard behavior |
| Only a fixed heading reveal is authorable | Bounded stage ranges and transform endpoints with static fallbacks | Express scroll stories without making content depend on animation |

## Verification plan

Local schema/rendering/rule tests, native authoring and actual-engine parity,
workspace types/build, strict content and Rust checks. Hosted acceptance must
cover placement/narrow layouts, long content, controls and keyboard focus,
no-JS/print/reduced motion, multiple gallery instances, scroll reversal/ranges,
resizing, source order and documentation preservation. Review desktop and phone
captures. No local browser, container or LaTeX runs. Public release and production
hosting are unchanged. No claim of arbitrary-section or zero-bug completeness.

Implementation and final verification are in progress.

## Early verification findings

All 509 engine tests and workspace types pass. The first native authoring test
attempt accidentally combined parent and child JSON assignments, which the CLI
correctly rejects before schema validation; the test now submits complete invalid
values to exercise the intended cross-field rules. Strict docs checks caught the
new examples referencing an original showcase SVG not yet copied into the docs;
the same original asset is now included. No product regression is inferred from
these test/fixture findings.

Implementation notes: print uses the standard
[`::details-content`](https://developer.mozilla.org/en-US/docs/Web/CSS/Reference/Selectors/::details-content)
content slot. Sticky ancestors retain visible overflow, following
[CSS positioning](https://developer.mozilla.org/en-US/docs/Web/CSS/Reference/Properties/position).
Hosted browser tests must verify the actual results; these references are not
substitutes for browser acceptance.


## Review corrections before acceptance

- Focused browser run `44220395935` passed 50 of 51 cases. Mobile WebKit
  kept closed native details content hidden in print despite `::details-content`
  styling. The final renderer provides a print-only summary/body sibling and
  hides the interactive details in print; screen readers and keyboard users see
  only the interactive copy on screen. Both JavaScript and no-JS print paths are
  covered. The earlier CSS approach above is retained as history, not the final
  implementation.
- Two plain-Markdown blocks using the same footnote label produced duplicate
  generated IDs. Each synchronous prose instance now namespaces those IDs with
  React `useId`, rewrites local links/ARIA references, and preserves anchor
  attributes when rendering through Next Link. Unit and hosted checks exercise
  two repeated footnotes in separate blocks.
- Initial screenshot selection included hidden dialog images. It now checks
  visible content images; dialog image behavior is exercised separately.
- Full native tests pass 812 cases with five hosted-only cases ignored. Clippy
  initially rejected a helper after the test module; moving it before the module
  resolves that warning. Warnings-denied Clippy and native/engine authoring parity
  now pass. The engine passes 510 tests and workspace types.

- First full CI run `44221389302` caught deep validation still using the MDX
  reference annotation for plain prose. That incorrectly treated local footnotes
  as global anchors and would misinterpret braces/HTML. The run was cancelled
  after diagnosis. Plain prose now has its own Markdown annotation; rendering and
  deep checking share a parser. Regression tests cover repeated footnotes, literal
  braces, dropped HTML, real route/asset errors and absent heading anchors. This
  fixes validation semantics rather than suppressing duplicate-ID diagnostics.
