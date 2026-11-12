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
