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
and final CI/merge evidence will be recorded after the hosted checks pass.
