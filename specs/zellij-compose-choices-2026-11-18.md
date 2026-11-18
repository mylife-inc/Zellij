# Composer selectors and feature explorers — 2026-11-18

Baseline `f9f1ceb`, source rc.11, public rc.10. This continues the user's approved
composer roadmap with independent selectors/explorers and two editable presets.
Multi-scene timelines and controlled section reuse remain subsequent work.

## Contract

- `selector` is a single-choice product presentation: labelled segmented choices
  or labelled colour swatches. It updates a matching heading, body, illustration,
  facts and optional CTA. It is not a shopping-cart or arbitrary form state API.
- `explorer` presents feature stories through horizontal tabs or a vertical list.
  Both nodes share one controller and the same finite panel vocabulary. No
  arbitrary nested sections or executable content in panels.
- Each node has 2–8 items with unique IDs and labels. `initial` names an existing
  ID; omitted means the first item. Swatch presentation requires every colour.
  Heading levels remain 2–4. Native/engine authoring enforce matching constraints.
- Controls enhance server-rendered sequential panels only after hydration. Every
  panel remains readable without JavaScript and in print. Selection does not
  animate, autoplay media, move focus into content, or modify the URL. Instances
  have isolated IDs/state. Keyboard arrows, Home/End and roving focus follow the
  radio/tab patterns; horizontal tabs leave Up/Down available for page scrolling.
- Panels reuse inline Markdown, CTA and image rendering. Images keep alt text,
  aspect ratio and local asset resolution. Layout responds to each component's
  own available space and preserves reading order on small screens.
- `product-selector` and `feature-explorer` expand into editable compositions via
  `zel ns compose -p <name>`. Existing four presets remain unchanged.

## UI review

| Before | After | Why |
| --- | --- | --- |
| Accordion stories are the only selectable composer media | Independent product choices and feature tabs | Match product choices and feature browsing to their own semantics |
| Generic tabs hide unselected content in server output | Progressive controls over sequential panels | Preserve reading without JavaScript and in print |
| Repeated product variants need repeated authoring | Two editable presets with explicit local asset placeholders | Make the new vocabulary discoverable through short CLI commands |

Reference patterns: [WAI-ARIA radios](https://www.w3.org/WAI/ARIA/apg/patterns/radio/)
and [tabs](https://www.w3.org/WAI/ARIA/apg/patterns/tabs/). Browser tests must verify
actual keyboard, focus and visibility behavior; these references are not tests.

## Verification plan

Schema/reference tests, render tests, native rejection-before-write and real
engine/native authoring parity. Hosted desktop/mobile Chromium and mobile WebKit:
selection/content synchronization, multiple instances, RTL, keyboard and focus,
responsive/nested layouts, light/dark/forced colours, print/no-JS, invalid-state
handling and browser errors. Review captures; retain docs preservation checks.
Full CI remains the merge gate. No local GUI/browser/container/LaTeX operation,
production deployment or release mutation is part of this increment.

## Early verification findings

Implementation `71da152` passed 516 engine tests, workspace types, engine build,
20 release-policy guards and deep strict checks. The focused hosted run
`44269010997` passed 72 of 75 composer cases with no retries; all three failures
were the print fallback. Tailwind's important base-layer `[hidden]` rule took
precedence over an unlayered important rule. The correction follows the existing
media accordion by placing the print override in the base layer.

The native suite found that interactive `feature-explorer` authoring could not
traverse an item's facts. Generic authoring selected a union branch by field name
alone: several composer nodes have `items`, but different item fields and bounds.
The correction selects the authored `type`/`kind` before field traversal, including
set, count and unset. Regression coverage exercises choices, lists, galleries and
layer backdrops; existing interactive preset coverage remains enabled. The initial
full CI run `44269051521` was cancelled after these failures were understood.

Visual review also found a light-theme inverted-card heading taking the global
heading colour instead of its local foreground. The correction gives choice
headings the same surface-aware rule as other composer headings, with browser
assertions in both colour schemes. The large-text probe now changes the root font
size so rem-based controls are actually enlarged. The native unknown-initial-ID
case removes selector-only fields before checking an explorer.

| Before | After | Why |
| --- | --- | --- |
| Inverted-card choice heading used the global foreground | Heading uses the local composition foreground | Keep light-theme dark cards readable |
| Important print rule was outside the base layer | Override uses the same layer as the hidden reset | Preserve every choice in printed output |

## Capture retention follow-up

Corrected source `1f86514` passes all 75 focused composer cases without retries
in run `44270115427`; all six documentation hydration-preservation cases pass.
Full-run showcase job `134684083280` passes 475 cases without retries (140
conditional skips). Its artifact retained the existing composer images but
omitted the twelve new `compose-*.png` captures because the upload pattern only
matched `composition-*.png`. All new captures are retained by the focused run.
The CI follow-up adds the missing artifact pattern without changing rendering,
authoring, content or tests; a fresh full run verifies final retention.
