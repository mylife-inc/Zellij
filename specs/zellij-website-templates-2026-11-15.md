# Website template collection

Date: 2026-11-15. Baseline: `2d44c4a` on main, published candidate rc.8.
The user prioritizes beautiful website templates alongside the completed book
starters. Source advances to rc.9 (unreleased); public rc.8 is unchanged.

## Design and scope

The existing `product-studio@0.1.0` is a small generic landing page. Keep it
immutable and add three distinct original starters, each at 0.1.0:

- `product-launch`: fictional Forma One audio product, full-width sculptural
  illustration, restrained typography, gradient heading, material details,
  domestic setting, and a separate Design page with media accordion and FAQ.
- `software-platform`: fictional Relay software, dark interface illustration,
  priorities and workflow narrative, plus a dedicated Workflow page.
- `creative-studio`: fictional Atelier North practice, large serif composition,
  original geometric identity art, and two expanded project studies.

Each has two landing pages and two guide pages. Local SVGs are original Shebka
art, listed in the manifest and ASSETS.md. There are no external images, copied
brand marks, unsubstantiated customer statistics, forms or signup services.
All sample CTAs resolve to local pages. Authors replace the fictional identities
and prose before publishing. The title variable changes the site identity and
home page title; it deliberately does not substitute prose or artwork text.

Current official reference pages inspected on 2026-11-15:
[Apple](https://www.apple.com/) and [MacBook Pro](https://www.apple.com/macbook-pro/)
use image-led product stories and paired actions;
[Linear](https://linear.app/) uses product-interface and workflow storytelling;
[Stripe](https://stripe.com/en-be) illustrates connected product capabilities.
These informed the narrative patterns; no assets, copy or proprietary UI were
copied. The editorial studio is an original complementary direction.

## Authoring and compatibility

Use `zel create <folder> -t <id> [--name <title>]`. Existing aliases, immutable
archives, receipts, remote catalogs and offline caching apply unchanged.
The source catalog has eight entries; the published rc.8 catalog still has five.
Before rc.9 release, use a source-built CLI and matching engine. The new
starters require rc.9, including the bento surface foreground fix. All hosted builds require a supported runtime and HTTP viewing. No
portable-file, book PDF or pixel-identical clone claim is made.

The templates compose existing hero, split hero, showcase, bento, zigzag,
compose, steps, media accordion, FAQ and CTA sections. No engine API, dependency,
font payload or published template bytes change. Bento tile headings and body
foregrounds now follow their own surface instead of the surrounding section. New art
is vector-only. Theme controls, reduced-motion handling and progressive content
use the existing engine. Source version and the standalone recipe advance to
rc.9; public installers and release pins do not change.

## Verification contract

Native tests cover all eight catalog entries, aliases, exact versions, custom
names, receipts and offline creation with no external tools. The packaging
helper enforces published archive hashes, builds new archives twice, compares
local/bundled/remote file receipts and replays offline cached creation.
Every section is parsed with its real engine schema and checked for discarded
fields, then every hosted site receives strict deep content/link validation.

The hosted template job builds the actual CLI output and checks every new page
at desktop/mobile widths in both palettes. It checks one h1, overflow, loaded
local images, browser/network errors, actual CTA navigation, keyboard media
accordion selection and no-JavaScript content. Screenshots are retained for
visual review. The existing book/template checks move intact from the long
CLI job to the independent template job. Native package tests check the exact
catalog IDs on all supported targets.

Initial local checks found guide-relative links resolving outside their guide
and an unlisted customization page. Absolute `/guide/...` links and explicit
sidebar entries fix both. Strict deep checks then pass for all eight starters.
Final source/hosted results and visual findings are recorded in
[the evidence](evidence/zellij-website-templates-2026-11-15.json).

The first hosted pass exposed a visual bug: inverted bento headings used the
global dark heading foreground on a dark card. Tile foregrounds and heading
inheritance now follow the tile surface, including when the enclosing section
is inverted or accent-colored. Browser coverage checks all 12 section/tile
combinations in both palettes. This requires the rc.9 engine for these starters.
The initial no-JavaScript harness also relied on a page-side animation-frame
promise, which cannot settle with scripting disabled. Driver-side scrolling
and bounded image polling fix the harness; the failed run remains in evidence.

Visual review of the complete 97-capture run prompted a final artwork pass:
the studio has a dedicated portrait poster and tighter section spacing; software
illustrations use a 4:3 frame so the sidebar and workflow nodes stay visible in
split layouts. Porcelain's light muted text changes from #8a8a8a to #6e6e6e,
with computed browser contrast checks for the studio's muted prose in both
palettes and on normal/tinted surfaces. This targets the 4.5:1 ordinary-text
threshold in [WCAG contrast guidance](https://www.w3.org/WAI/WCAG22/Understanding/contrast-minimum.html);
it is not a claim of a complete accessibility audit.

The product template's final desktop pass gives both material cards two grid
columns, filling the four-column row, and places the detail illustration's
caption inside its narrowest crop. These are unpublished template refinements;
all previously published archives retain their hashes.

## Verified result

All 30 source CI jobs in [43755151229](https://github.com/mylife-inc/Zellij/actions/runs/43755151229)
pass at `40ab8b78f1c8de7e1ae593322e56ec9e3fada103`. Local checks pass 802 Rust,
490 engine and 49 release-helper tests. The hosted template job produces 97
captures, including the previous book starters; six reviewed website hero
previews are retained in `specs/evidence/website-templates-2026-11-15/` and match
the final run byte for byte. All eight template archives pass deterministic
packaging and local/bundled/remote/cached parity; the five published archive
hashes remain unchanged. The final records-only update does not alter any
runtime, active-guide, template, test or workflow input from the tested head.

PR #185 merges as `aaf14e3`, with the same tree as reviewed `e807955`. All 13
jobs in development-image publication
[43759089541](https://github.com/mylife-inc/Zellij/actions/runs/43759089541)
pass at the merge. Four `main` feature variants publish both amd64/arm64; the
published default image builds and serves a site. The initial readiness probe
reset and the next succeeded within the existing bounded loop. Manifest digests
and job results are retained in evidence. This records-only follow-up changes
no runtime, active-guide, template, test, workflow or image input.

This milestone does not publish rc.9 or change production hosting; the public
release remains rc.8.
