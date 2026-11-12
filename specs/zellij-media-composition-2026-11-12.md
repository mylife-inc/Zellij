# Media accordion and section presets — 2026-11-12

Baseline: `cc199f3` on main, published CLI v0.2.0-rc.5. This implements the next
slice of the [expansion design](zellij-product-expansion-2026-11-11.md). It does
not imply the entire composer, book/PDF publishing or installation backlog is
complete. Runtime and native changes are source after rc.5 until a new release.

## Authoring contract

The concise `media-accordion` section and the `compose` node share one field
schema and renderer. Both have a required accessible label, two to eight items,
landscape/square/portrait image ratio, contain/cover fit, start/end media side
and item heading level 2–4. Each item has title, inline-Markdown body, image,
meaningful alt and optional inline-Markdown caption. The concise section also
has an optional header. The composed node accepts the existing entrance effect;
its heading is a separate ordinary node. All references use the shared asset and
inline-Markdown annotations, including strict unknown-field diagnostics.

`zel presets` / `zel list presets` lists four engine-owned compositions:
product-hero, split-story, feature-cards and media-accordion. JSON lists names,
section types and descriptions. `zel ns -p <name>` infers the section type;
`zel ns compose --preset <name>` is explicit. Both use existing editing/wizard
and no-overwrite rules. Use `--out` when omitting the type. Presets are expanded
into ordinary YAML, without runtime lookup, network, authored executable code or
an extra schema format. They differ from whole-site `zel templates` catalogs.
The existing `explain`, generated context, schema export and completions expose
the additions. A preset/type mismatch fails before prompting or writing.

## Rendering and accessibility

The visual treatment uses restrained editorial typography, generous image space,
thin separators and existing theme tokens. Original Fieldnotes vector artwork
illustrates focus, layered ideas and an overview; no Apple assets are copied.
The desktop companion changes with selection; narrow containers place each
image after its text. A CSS container query at 760px handles nested cards without
assuming a desktop viewport implies a wide component. Start/end placement does
not change reading order. No new dependency is introduced.
Caption content has one DOM instance beside the selected text (below the inline
image in a narrow container), so resizing cannot hide a focused caption link.

Headers follow the [WAI accordion pattern](https://www.w3.org/WAI/ARIA/apg/patterns/accordion/):
native buttons in headings, unique per-instance control/panel IDs, expanded
state and an explicitly non-collapsible active item. Enter/Space select; Tab
works normally; Up/Down and Home/End move header focus without changing selection.
There is no autoplay, focus jump or unnecessary live-region announcement.

Server output is a sequential reading view with all images/text and ordinary
headings. The first client render matches it; an effect enables the controls
and selects the first item. This deliberately allows a layout change during
enhancement to keep failed/disabled-JavaScript output complete. Print CSS restores
all hidden panels and inline images; it does not need a beforeprint script.
All bounded item images load eagerly to support instant selection and printing.
Authors should optimize image sizes. Reduced motion and motion:none suppress
image fades; forced-colour mode retains system text, borders and focus outlines.

## Verification checkpoint

Local engine tests and type checks pass at the first implementation checkpoint;
17 native authoring cases pass, including presets, typed refinements, invalid
inputs and overwrite protection. Final workspace, hosted browser/native checks,
screenshot review and merge remain pending. Hosted cases cover synchronization,
keyboard/focus, multiple instances, narrow containers, viewport changes, print,
reduced motion, forced colours, disabled JavaScript, image loading and hydration.
The native-to-engine and hosted export corpora include all 25 section types and
all four presets. Current-source docs add a working preview and reference guide.

No local browser/Docker, live deployment provider, production documentation/domain
migration or release publication is part of this implementation checkpoint.

First hosted run `43305489541` on `11ec16a`: 350 showcase cases passed, 124
existing conditional cases skipped, and six new cases failed across the three
browser projects. Print retained only the two selected panels because Tailwind's
important `[hidden]` rule in the base cascade layer outranked the unlayered print
override. The correction places the hidden-panel print override in that same
base layer. The other three failures expected a keyboard outline after pointer
interaction followed by programmatic focus; the test now uses the supported Home
key and asserts actual keyboard focus before checking the outline. A separate
review keeps captions in one DOM instance and tests linked-caption focus across
resizing. Final-head hosted verification is required; this is not a passing gate.

Local verification history: one concurrent full engine run timed out in four
existing MDX tests; a lower-concurrency rerun is pending. The Rust workspace's
unit/integration suites passed, but Homebrew rustdoc aborted on a missing z3
dynamic library before doctests. The doctests are being rerun with rustup's
rustdoc, without modifying product code or the host's package installation.

Run `43307234808` on `0dac1d3` passed all 356 showcase browser cases (124 existing
conditional skips, no retries), including print and caption-focus corrections.
All six light/dark desktop and phone captures were reviewed. They exposed body
and caption links that inherited ordinary text styling without a persistent
visual cue. The follow-up underlines those inline links and asserts that cue in
keyboard/caption browser cases. A final-head run is required after this change.
The lower-worker engine run passes all 489 tests. Rust doctests pass with a
matched rustup compiler/doc pair in an isolated build directory; mixing only the
rustdoc replacement with Homebrew-built metadata failed and remains recorded.
All 17 native authoring cases pass with the matched toolchain.

## Merged acceptance — 2026-11-12

PR #147 merged as `2373c5e`, with exactly the tree tested at `d8c6b7c`.
All 20 final-head jobs in `43308742913` passed without job reruns. All 356
showcase cases passed without retries (124 existing conditional skips), including
21 new cases across desktop Chromium, mobile Chromium and mobile WebKit. Six
final light/dark captures were reviewed at full resolution where needed.
The full export corpus renders all 25 section types and all four native presets.

Documentation acceptance preserves 46 original routes, 282 heading IDs, 82
compatibility mappings and 29 byte-identical images; all 12 added routes also
pass. The new guide's source/preview tabs and selected media render correctly.
Local gates pass 489 engine tests, workspace types/build, schema drift checks,
warnings-denied Clippy, 747 Rust unit/integration cases (five existing ignored),
matched-toolchain doctests and 17 native authoring cases. The previous failed or
superseded checkpoints above remain historical; they are not passing gates.

All 13 development-engine jobs in `43313219720` pass on the merge. The four
`main` feature variants are published for Linux amd64/arm64; the published image
also builds a site successfully.
The [machine-readable evidence](evidence/zellij-media-composition-2026-11-12.json)
records exact source/merge identities, logs, captures and run results. Published
native rc.5 and the production documentation/domain are unchanged. Remaining
composer primitives, book/PDF/LaTeX, technical/study editions and additional
installation channels remain follow-up work.
