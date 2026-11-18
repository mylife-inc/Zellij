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
