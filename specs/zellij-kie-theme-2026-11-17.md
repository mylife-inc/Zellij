# Kie theme and API-platform starter — 2026-11-17

The user requests a theme identical to the visual design of https://kie.ai,
after the current book work. This source rc.10 addition introduces `kie` as a
selectable theme and `api-platform` as a matching website starter. It retains
the user's centered product/menu group, far-left optional company link and
right-hand controls, plus the existing two-level mobile drawer.

## Reference and scope

Normal HTTPS retrieval of the public server-rendered HTML/CSS succeeded on
2026-11-17. Live hosted browser navigation returned a Cloudflare 403; that page
was rejected as a reference. Run `44072718969` renders the public main/CSS with
scripts disabled, using the same declared Space Grotesk family from Zellij's
pinned local inventory. Its light/dark desktop/mobile captures are retained
privately for comparison. This method does not verify live interactions or
missing dynamic media. Reference HTML/CSS and company/model artwork are not
redistributed in the repository.

Measured design: light background #fafafc, foreground #0b1220, secondary text
#44516b, accent #2d6cff, border #e3e8f0; dark background #060814, foreground
#eceff7, secondary text #bfc6d8, accent #4d86ff, border #1a2138. Space Grotesk
600 headlines use 72px/1.04 at desktop and 38px on phones, tracking -0.032em;
section titles scale to 56px/1.05. Hero measure is 920px, content 1200px,
header 60px, pill actions 44px (52px in hero). Cards use 20px corners.

The theme applies those typography, palette, geometry and blue-light treatments
to existing components. Author overrides, dark/system mode, local fonts,
page themes, reduced motion and no-JS content remain supported. Portable docs
receive the same token palette/fonts; marketing sections use the hosted engine.
The dark primary button uses a dark readable foreground. The starter has
original copy/art and no remote API, customer endorsements, auth or checkout.
A theme cannot guarantee pixel identity for different content or replace a
third-party application's behavior. No unverified 100% identity claim is made.

## Starter and compatibility

`zel create my-api -t api-platform` creates `zel.yaml`, `content/`, four routes
and local SVG illustrations. Sections include a wide hero, stats, pricing,
model carousel, integration steps/terminal, feature grid, FAQ and final CTA.
Template version 0.1.0 requires CLI and engine rc.10. Creation is offline;
builds use the hosted runtime and the output needs HTTP. Existing eight
published template archives remain immutable. No new dependency or font is
needed. Public rc.9 is unchanged until an explicitly verified release.

## Acceptance

Initial acceptance plan: engine/native tests, schema and token freshness, exact archive
preservation, strict authored-content validation, actual source engine builds,
desktop/mobile light/dark captures, local fonts, FAQ keyboard use, no-JS content,
reduced motion, full shared navigation and image/network checks. All actual
browsers and containers run in hosted CI; no local browser/container is started.

First website run `44074881310` catches the Kie hero at the base 64px/centered
style instead of its 72px/start design. Hosted Chromium/WebKit probe
`44076164327` isolates an existing shared-theme bug: bare attribute selectors
inside `@scope` implicitly select descendants, excluding the theme-bearing root.
Using `:scope[data-zellij-theme=…]` restores the presentation rules while retaining
nested theme boundaries. This also restores the previously inert Cupertino,
Mono, Porcelain, Aurora and Carbon rules; their website visuals require review.
A durable two-browser check covers root, nested and re-entered scopes and these
older presentation rules. Full CI also catches the missing generated Markdown
context snapshot; both generated forms now include Kie.

The preservation gate in `44076881549` catches a renamed existing theme-list
heading. The original `#the-fourteen` anchor is restored; it still describes
the public rc.9 inventory, with Kie documented in a separate source rc.10 section.

The corrected desktop captures confirm the measured typography and placement.
The mobile driver then catches its own incomplete loading strategy: vertical
scrolling does not reveal lazy images in a horizontal carousel. It now visits
each card, verifies decoded images and restores the track before screenshots.
The new template is formatted as editable YAML, and hero/stat band colors and
mobile stat columns are refined. Arabic comparison captures request reduced
motion to avoid recording a header halfway through a color transition.

Run `44078723389` passes the nine-template job and strict documentation gate.
The showcase still expected 14 theme choices; it now expects 15 and checks Kie
by name. Arabic skip-link acceptance polls its layout with the same one-pixel
tolerance as the page-overflow check and includes measured bounds on failure.
This does not change the skip-link CSS or relax the requirement to show it.

The final Next browser measurements in `44080427260` explain the skip-link
assertion: focus state arrives before its CSS placement on three of four cases.
Initial x coordinates still contain the offscreen 9,999px offset; settled bounds
are exactly x=1141 at 1280px and x=251 at 390px, each 139px wide. Polling rendered
layout resolves this harness race without changing the product's logical CSS.

## Verified theme result

All 32 jobs in `44080427260` pass at `4604cfc`. The showcase passes 418 cases
with 140 conditional skips and no retries; additional prefix/reload/page-theme
checks pass. The scoped-theme regression passes Chromium/WebKit on all three
OS families. The template job retains 132 captures and each native website
architecture retains 112. All 356 captures retain the reviewed earlier bytes.
Cross-architecture geometry is identical; 35 images match every pixel and the
remaining 77 differ by at most one RGB-channel step.

Local checks pass 810 Rust tests (five hosted-only cases ignored), 494 engine
tests, 104 helpers, warnings-denied Clippy and strict 59-route/25-example docs.
All nine templates pass creation/packaging/content acceptance; eight previously
published archives retain their exact bytes. Reference/design and final previews
are retained in [the evidence](evidence/zellij-kie-theme-2026-11-17.json).
A final CSS review leads to the separate [card-surface follow-up](zellij-card-surfaces-2026-11-17.md).

PR #196 merges as `5edac83`, identical to tested `4604cfc`. Development-image
run `44091260003` passes all 13 jobs, publishing four amd64/arm64 variants and
passing the serving check. Card follow-up PR #197 merges as `84dcce6`; its
32-job CI passes and all 356 existing captures retain exact bytes. Final records
include retained previews and a strict-checked guide correction distinguishing
published rc.9 from source rc.10 multilingual support.
