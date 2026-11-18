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
container and actual LaTeX execution stays in hosted CI. Verification is pending.

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
rejection. The original failure remains recorded; final verification is pending.

The first focused CLI test also mixed `--count` with a later complete JSON
replacement. Counts deliberately apply to the original template/preset before
JSON replacements. The regression now checks count edits on `product-overview`
and scalar edits after complete JSON separately, preserving existing precedence.
The expected preset count was updated from seven to nine; these were test
assumption corrections, not changes to CLI option ordering.

Print review also found the shared gradient-heading treatment retained a
transparent text fill when backgrounds were omitted. Print now uses solid dark
text; the embedded feature fixture and browser test exercise that fallback.
