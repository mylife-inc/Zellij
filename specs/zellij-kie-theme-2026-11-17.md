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

Pending: engine/native tests, schema and token freshness, exact archive
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
