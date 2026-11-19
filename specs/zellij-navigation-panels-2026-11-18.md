# Desktop navigation panels and theme previews — 2026-11-18

## Report and cause

A local site review found narrow desktop menus and identically coloured theme
swatches. Flat menu leaves deliberately used a compact dropdown, while nested
groups already used a full-width panel. A site `overrides.accent` value was
emitted inside every scoped theme rule, so all preview dots inherited that same
brand colour.

## Implementation

- All desktop menu panels, including a flat More overflow menu, now span the
  header width and open flush below it. Flat leaves use a responsive grid;
  existing groups retain headed columns and prominent first-column typography.
- The opening animation reveals downward without translating the panel across
  the bar. Both reduced-motion preferences and `motion: none` disable it.
- Panels scroll within the actual available height. A header ResizeObserver and
  viewport/scroll updates account for announcements and short windows. Existing
  hover intent, keyboard entry, dismissal and centered navigation remain.
- Theme previews receive their original palette's accent/background/foreground
  tokens separately from the site's active branded tokens. Light, dark and both
  system-mode cases keep accurate swatches while preserving site overrides.
- WebKit pointer activation did not focus the mobile menu trigger, so the focus
  trap sometimes restored the body on close. Opening now explicitly focuses the
  trigger without scrolling. The former WebKit skip for this behaviour is removed.

The menu and theme guides distinguish the next engine update from released
rc.11. No native CLI command or configuration syntax changes. No public release
or production deployment is part of this local fix. Private acceptance content
and artifacts remain outside this repository.

## Verification

- All 75 targeted theme/navigation engine tests pass; engine typecheck and
  package build pass.
- The production showcase builds with the locked Next 16.4.6. All four relevant
  browser specs pass on the final source: **127 passed, 89 conditional skips,
  zero failures and zero retries**, using desktop/mobile Chrome and WebKit.
- Coverage includes flat and grouped panels, More overflow, short-viewport
  scrolling, hover travel, centering, keyboard navigation, reduced motion,
  original theme accents under a site override in four mode/OS combinations,
  and mobile section/back/focus behaviour.
- The documentation check passes for 59 routes and 33 validated examples.
- `git diff --check` passes. No public release or deployment was performed.

See [the machine-readable evidence](evidence/zellij-navigation-panels-2026-11-18.json).
Private consumer acceptance and its packaged candidate are tracked separately.

## Retained diagnostic history

The first browser attempt had one real panel/header overlap during entrance and
61 WebKit launch failures because the required browser revision was absent.
The overlap was removed from the animation; the matching browser was installed.
A subsequent run completed 125 cases with 90 conditional skips and one WebKit
focus-restoration failure. That failure prompted the trigger-focus correction;
it is not counted as a passing gate.

The local dependency installation initially contained Next 16.4.2 despite the
lockfile pinning 16.4.6. `npm ci --ignore-scripts` restored the pinned dependencies;
no manifest or lockfile change was needed. Rebuilt verification uses 16.4.6.

The separate linked-heading/nested-anchor hydration issue recorded in
`ZELLIJ_CONTEXT.md` remains open. These navigation changes do not fix or conceal it.
