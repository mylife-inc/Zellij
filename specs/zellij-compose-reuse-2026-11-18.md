# Controlled section reuse — 2026-11-18

Baseline `bf38c63`, source rc.11, public rc.10. This continues the approved composer
roadmap after selectors and scroll stories. No release or production deployment
is part of this increment.

## Contract

A leaf `type: section` has a required accessible `label`, `headingLevel` 2–4
(default 2), and inline `content` whose `type` is one of eight reviewed built-ins:
`feature-grid`, `pricing-table`, `stats-band`, `cta-banner`, `bento-grid`,
`timeline`, `team-grid`, `logo-strip`. It uses the same semantic renderer and
schema as the standalone section. Parent composition/group owns the surface,
padding, anchor and motion. Nested `id`, `motion`, `background`, `padding` and
unknown fields are rejected. `titleGradient` remains supported. No `place` or
`effect` on the node; no section below layers or a scroll stage, including
intervening groups. Ordinary stack, split and grid groups remain supported.

Headings never introduce an H1. A section heading uses `headingLevel`; item
headings use the following level when the section has its own heading, otherwise
they use `headingLevel`. Logo-strip's heading retains its existing paragraph
semantics. Standalone heading markup remains unchanged. No author IDs are emitted
by the adapter; existing controls retain their own React-generated identities.

New embedding bounds: 16 logos; 12 feature/bento/timeline items or people; eight
links per person; 24 features per pricing tier. Existing 1–4 tiers and 3–5 stats
remain. Existing format 1, three group levels and 128 composer nodes remain.
Reference annotations survive at each nested asset, inline Markdown and href.
Engine and native authoring enforce the same contract before file publication.

Source map: `reusable-section-schemas.ts` holds the eight unchanged standalone
schemas, re-exported by `schemas.ts`; `reuse-schema.ts` derives their bounded
embedded forms. `components/section-content.tsx` dispatches only those reviewed
renderers. This avoids a cycle through the central schema catalog; rendering
does not dispatch arbitrary registry entries. Rust's mirrored rules remain in
scaffold `authoring.rs`.

Reuse means embedding built-in content, not referencing another YAML file. Other
registered section types, recursive `compose`, arbitrary HTML/CSS/scripts and MDX
are not accepted by this node. Existing composer primitives remain available for
other semantic/interactive content; this is not unrestricted universal rendering.

## UI review

| Before | After | Why |
| --- | --- | --- |
| Authors reconstruct pricing, linked feature cards and numeric statistics from small nodes | Embed the actual built-in renderers | Preserve their semantics, copy control, count-up behavior and tier treatments |
| Dedicated grids follow viewport width | Embedded grids follow their containing column | Avoid squeezing three cards into a narrow split or nested group |
| Dedicated sections own H2/H3 and full-page spacing | Explicit subordinate heading levels and composition-owned spacing | Keep one page heading and a predictable reading outline |
| A copy button could appear before JavaScript or clipboard support was available | Show it only when usable; retain selectable command text | Preserve an honest fallback |
| Dark print backgrounds and transparent gradient text could obscure copy | White compositions, dark text, stable statistics and no sticky header in print | Keep the authored content readable |

No interaction animation is added. Existing motion remains opt-in and subject to
reduced-motion preferences. No-JS and print must retain all content. Keyboard
navigation, independent copy controls, source order, theme contrast, narrow
containers and actual built-in behaviors are hosted browser acceptance gates.

## Verification plan

Schema bounds/references and server-rendered heading/list/CTA/media semantics;
CLI structured/scalar edits and rejection before writes; parity of standalone
schemas and prior presets; strict deep docs/showcase checking; hosted light/dark,
mobile, narrow-container, print, forced-colour, no-JS and interactive checks.
Run complete CI before merging, retaining first failures and fixes. Browser,
container and actual LaTeX execution stays in hosted CI. Results are recorded below.

## Shared statistics fallback

Review found that printing an in-progress count-up could capture an intermediate
number. The shared renderer now separates its decorative animated value from
one stable authored value for assistive reading and print. No-JS markup retains
the complete number, and missing IntersectionObserver keeps the visual value
static. Existing motion tests still inspect the animated value's completion.

| Before | After | Why |
| --- | --- | --- |
| Print or assistive reading could encounter an in-progress statistic | A stable authored value accompanies the decorative animation and becomes visible in print | A statistic must retain its actual meaning in every output |

The reused copy control now appears only after hydration and when the clipboard
API exists. Without JavaScript/API access, the command remains ordinary selectable
text. Its reset timer is cancelled on unmount. Clipboard permission failures still
leave the button idle, never claiming a copy that did not happen.

## First local native finding

The first full native run stopped in `full_starter`: the schema vocabulary did
not yet recognize `additionalProperties: false` from strict embedded objects.
The native validator already rejects undeclared object keys. It now accepts
only the equivalent explicit `false` constraint on an object with a properties
map; permissive, schema-valued and invalid-context forms remain rejected.
A native regression checks both the accepted form and actual unknown-key
rejection. The original failure remains recorded; the corrected native suite passes.

The first focused CLI test also mixed `--count` with a later complete JSON
replacement. Counts deliberately apply to the original template/preset before
JSON replacements. The regression now checks count edits on `product-overview`
and scalar edits after complete JSON separately, preserving existing precedence.
The expected preset count was updated from seven to nine; these were test
assumption corrections, not changes to CLI option ordering.

Print review also found the shared gradient-heading treatment retained a
transparent text fill when backgrounds were omitted. Print now uses solid dark
text; the embedded feature fixture and browser test exercise that fallback.

A further print review found that embedded inverted bento tiles and dark-theme
cards retained their surface when heading text switched to black. Embedded
cards, CTA links and pricing flags now print without their coloured background,
with dark foregrounds. A dark-theme print assertion and capture cover the pairing.

## First hosted interaction finding

Focused run `44393615245` on `46acfaa` passed 142 browser cases, skipped 12
conditional motion cases and failed two forced-colour team-link clicks in
Chromium desktop/mobile. Its retained trace shows an x-coordinate of 111.5
for a link whose visible label starts at 112: the inline link exposes its
absolutely positioned, visually hidden name prefix as a tiny initial hit box.
The click strikes the parent list item or sticky header while scrolling.
Team links now establish one positioned inline-flex hit area with a 44px minimum
height. The accessible person prefix remains; the regression checks the centre
hit target before a normal, unforced click. WebKit had already passed.

| Before | After | Why |
| --- | --- | --- |
| An inline team link can expose a hidden prefix as a tiny hit box | One positioned link box with a 44px touch target | Keep pointer interaction aligned with the visible label and accessible name |

Visual inspection of the retained dark-theme print capture found another
pairing missed by the computed foreground/card assertions: the outer composition
was transparent over a dark page background. Printed compositions now paint
white, with transparent inner groups; the sticky site header is hidden in print.
The browser test checks both the outer white surface and hidden header, and
retains a full-page print capture for review.

The full run on `79f6102` passed all 526 browser cases without retries, with 140
conditional skips. Its WebKit CSS-print capture was taken before offscreen lazy
images had loaded (unlike the light/dark visual check, which visits and awaits
each image). The print fixture now uses that same explicit readiness check before
emulating print and checks image visibility afterward. This adjusts the capture
precondition; it does not change the production image loading policy or claim
that CSS-media emulation is a native print-dialog test.
The narrow-container capture uses the same image-readiness requirement after
scrolling its bento into view; WebKit could otherwise capture its lazy placeholder.

## Local verification

Implementation `b9865f2` passes 817 Rust tests (five hosted-only cases ignored),
527 engine tests, 22 native authoring cases, 20 release-policy guards, workspace
types, formatting, warnings-denied Clippy and native/engine authoring parity for
all 25 section types and nine editable presets. Strict deep checks pass for
16 showcase routes, 59 documentation routes and 33 previews, and 21 CodeSeal
routes. All 24 standalone schemas/templates and the previous seven presets are
unchanged; the section catalog remains at 25.

Runtime follow-ups: `46acfaa` pairs embedded print cards and dark foregrounds;
`2a7b07f` fixes team-link hit areas; `79f6102` prints compositions on white and
hides the sticky header. `5e52bb2` and `2f69966` only stabilize lazy-image capture
readiness. Final hosted acceptance passes on `2f69966`.

Final run `44397663238` encountered GitHub ARM runner capacity: portable macOS
job `135132002174` was cancelled after waiting without a runner or any executed
steps. GitHub's annotation says “The job was not acquired by Runner of type
hosted even after multiple attempts” and reports macOS ARM capacity constraints.
After the other 31 jobs passed, only that job was retried. Replacement job
`135157310225` passes; all 31 prior results are retained.


## Final hosted acceptance

[Full run `44397663238`](https://github.com/mylife-inc/Zellij/actions/runs/44397663238)
passes all 32 jobs on `2f69966d49ae571a1e1a1114c95717fa6d0b8a61`, attempt two.
The retry addressed runner capacity only; no test failure was retried on this
source. Showcase passes 526 cases without retries, with 140 conditional skips.
Engine passes 527 cases. Native export exercises all 25 templates and nine
presets, including all eight reused types in the two new presets, ordered wizard
output and structured edits. Cross-platform installers, managed channels,
containers, website/book templates and real Linux/ARM LaTeX and Next PDF checks
pass. Five locally ignored container-adapter cases execute successfully in CI.

Focused run `44395075273` passed 144 browser cases without retries (12 conditional
motion skips) and six preservation probes on the team-link correction. Later
full acceptance covers all these cases plus the final print/readiness changes.
The redundant final diagnostic probe was cancelled during browser setup only
after the final-source full showcase and preservation checks passed. Its workflow
replacement remains on a probe branch and is not part of this feature.

Documentation verification preserves all 46 original routes, 282 original heading
IDs, 82 compatibility mappings and 29 byte-identical images; all 13 added routes
are checked, no browser errors are captured, and six preservation probes pass.

Artifact `14470384029` retains 52 unique composer captures: the previous 40 and
12 new light/dark, narrow and print captures across desktop, mobile and WebKit.
All 12 new captures were visually reviewed. Thirty-six previous captures match
exactly. The other four were reviewed: one heading shifts two device pixels
with scroll position and matches exactly after alignment, and three differ only
in native video loading-indicator paint. The final WebKit narrow/print captures
include loaded media. Printed compositions have white surfaces, dark text,
stable authored statistics, readable commands and no sticky header or copy
buttons. These are CSS print-media checks after explicit image readiness, not
native print-dialog or pagination verification.

See [the machine-readable evidence](evidence/zellij-compose-reuse-2026-11-18.json)
for commits, failures, jobs and capture hashes. The final follow-up changes only
this record, the evidence file and `ZELLIJ_CONTEXT.md`; runtime, active guides,
dependencies, workflows and acceptance inputs remain identical to the accepted
source. No release or production deployment occurred. Next are rc.11 publication,
installer/public-documentation updates and reviewed rc.10 retirement; live-provider
verification stays deferred.
