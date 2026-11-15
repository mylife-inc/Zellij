# Centered site header correction

Date: 2026-11-15. Baseline: `22345d0` on main; rc.9 source, rc.8 public.

The user clarified the layout after reviewing the new website template images:
company identity at the far left, product/home navigation and menu centered,
search and theme controls at the far right. The previews inherited `nav.align`
left. The previous center option aligned only the link list within leftover
space, so unequal branding and controls displaced it from the viewport center.

The shared hosted header separates company identity from the product/menu group.
Centered layout reserves the larger side width at both edges; overflow budgets
come from the entire header rather than the current visible link widths. This
keeps hidden entries able to return when the viewport grows. Explicit left
alignment remains supported. Mobile keeps company/product identity and the full
drawer, with right-aligned controls. The default changes to center in rc.9.

The three unpublished website starters and active documentation opt into center.
The five already published template archives remain unchanged. Native portable
books retain their separate sidebar navigation; they have no desktop top-menu
row to center. No public release or production documentation deployment is part
of this correction.

Verification covers exact desktop centering, optional company identity, unequal
utility widths, 320–1600px viewports, More/restore behavior, large text, keyboard
focus and complete mobile navigation. Actual website export checks include
header geometry in both palettes with and without JavaScript. Updated screenshots
and CI/merge evidence are retained in the evidence record.

The first hosted template job passes all 97 captures and exact centering checks.
Showcase acceptance exposed a crowded 320px company header and older tests that
assumed all five menu items remained visible at a fixed viewport. Mobile moves
the GitHub utility into the drawer, preserves its destination, and allows the
product image to shrink beside a maximum-width company icon. The company slot
uses its intrinsic width rather than stretching to the grid track. Menu tests
now select wide-screen versus overflow cases explicitly and the compact-panel
fixture derives its width from the measured labels and controls. Failed cases
remain recorded; no assertions are waived.

## Verified result

All 30 jobs in [43764569035](https://github.com/mylife-inc/Zellij/actions/runs/43764569035)
pass at `b202c5f1c055a238149dfb9b4e5b41c43424d67d`. Showcase checks pass 382
cases with 140 existing conditional skips and no retries; prefix and development
reload acceptance also pass. Local checks pass 491 engine tests, type checking,
engine packaging, six native template integration cases, strict deep docs
validation and deterministic packaging/parity for eight templates. The five
released archives retain their hashes.

The template job checks actual header geometry with and without JavaScript and
produces 97 captures. Six refreshed hero previews are visually reviewed and
retained in `specs/evidence/centered-header-2026-11-15/`. See
[the evidence](evidence/zellij-centered-header-2026-11-15.json) for hashes,
initial failures and fixes. The final records-only update changes no runtime,
active-guide, template, test or workflow input from the verified head. Merge
and development-image publication follow; public rc.8 stays unchanged.
