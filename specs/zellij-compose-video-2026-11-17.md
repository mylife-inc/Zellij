# Composer video and container responsiveness — 2026-11-17

The user asked whether the long-requested universal section was complete and
could express every possible section. `compose` is a released bounded first
version, not the whole proposed system. Its existing guide describes the tree,
four presets, limits, authoring commands and gradient reveal. The guide now also
has an explicit capability/status matrix. Solid-to-gradient headings already
work in rc.10. Layers, sticky scroll stages, independent media selectors,
feature explorers, block Markdown, icons and embedding arbitrary section types
remain unfinished. Arbitrary scripts/CSS are outside the declarative contract.

## Source rc.11 increment

- Add `type: video` as a composer leaf at every supported group depth. Require
  a label and source; support a poster, inline caption, transcript link and up
  to eight WebVTT caption/subtitle tracks. Existing tree limits are unchanged.
- Share media rendering with `video-embed`; its visible caption stays plain
  text for compatibility. Both gain tracks and transcripts. Asset annotations
  cover sources, posters and tracks; href annotations cover transcripts. The
  generated native schema/context and authoring tests exercise the same fields.
- Video remains user initiated. Fix the existing inert no-JavaScript play
  button with a native self-hosted player using `preload="none"`. Provider
  videos keep a direct source link and load no iframe until requested. After
  activation, focus moves from the disappearing button to the player.
- Print hides player controls and retains the source/transcript links and
  visible caption. Caption text is not a substitute for synchronized speech
  captions. Provider subtitles are managed by the provider; local tracks apply
  to self-hosted media. Cross-origin media/tracks need anonymous CORS support.
- Provider detection now parses an exact hostname and path rather than matching
  `youtube.com`/`vimeo.com` anywhere in a URL. Query parameter order and YouTube
  short/watch/embed URLs are supported. Embeds use the no-cookie YouTube host.
- Nested grids and splits use their own 760px containing width instead of the
  browser width. Groups supply a container around their inner layout. Narrow
  cards stay readable on desktop; source order is unchanged. Unsupported
  container queries and print retain a single column.

No dependency is added. Composer format remains version 1; existing documents
are valid. These additive nodes/fields require a matching rc.11 CLI/engine.
Published rc.10 artifacts, existing template archives and production hosting
are untouched. Docker's default source version follows Cargo's rc.11 version.

## Original fixture

`compose-motion.mp4` is an original 4.5 KB, two-second silent H.264 clip generated
for this task with FFmpeg: 640×360 at 12 fps, warm paper with three rectangles
(copper/slate/gold), fading in over 0.8 seconds. It contains no third-party
footage. Both showcase and documentation include the same bytes and a local
English WebVTT track. The showcase supplies a readable animation transcript.

## Initial verification (superseded runs retained)

Local: engine types/build and 504 unit tests pass. The first run caught a test
assertion using lowercase `srclang` against React's server markup `srcLang`;
the assertion now uses React's emitted attribute spelling. Browser HTML remains
case-insensitive. The full native workspace passes 811 tests (five existing hosted-only cases
ignored), followed by all 18 authoring integration tests on the final source.
Warnings-denied Clippy, workspace types, all 25 real-engine authoring schemas,
20 release-policy guards and strict showcase/docs checks pass. Hosted schema
parity and engine/documentation checks pass. Showcase run `44179876080` passes
427 browser cases with 140 conditional skips and no retries. All 59 documentation
routes render without captured errors, retaining 46 original routes, 282 heading
IDs, 82 compatibility mappings and 29 unchanged images. Broader hosted acceptance
is still in progress.

Hosted coverage adds real local-video decoding/playback, lazy loading, caption
loading, focus after activation, no-JavaScript native controls, print links,
reduced-motion behavior and nested container sizing on desktop, Android-sized
Chromium and iPhone-sized WebKit. Existing composer screenshots cover the
additional section in light/dark themes. No local browser or container is run.

## Remaining capability priorities

The current catalog has 25 section types. Composing their visible copy is not
equivalent to implementing their semantics or behavior. For example, `stat`
is not `stats-band`'s count-up animation; a grid of images is not `gallery`'s
keyboard-operable lightbox; and visually arranged text is not `matrix`'s table.
Do not declare the composer universal by approximating those with decorative
text. Keep the dedicated components available until their behavior can be reused.

1. Layered layout and bounded child placement: text/CTA over media, clear local
   contrast settings, useful mobile/print linearization, and no occluded focus
   targets. The existing `media-panel` is the dedicated alternative today.
2. Reusable semantic/interactive primitives: block prose, icons, tables, code,
   accordions, carousels, selectors and explorers. Prefer sharing existing
   accessible components to duplicating their state machines in the composer.
   Reusing a section must account for heading ownership and the finite schema
   size; unrestricted recursive section embedding is not an accepted shortcut.
3. Bounded sticky/scroll stages and authored effect ranges. Every authored
   effect must define a stable no-JS, reduced-motion and print state. Preserve
   the current reversible solid-to-gradient heading as the simple default.
4. More editable presets built from proven primitives, with visual examples
   and authoring recipes. Preset breadth cannot substitute for missing behavior.

These are priorities, not newly implemented capabilities or a promise to accept
arbitrary HTML/CSS/JavaScript. The reference guide is the public-facing status
matrix; this record explains why the remaining work is more than styling.

The separate development-only picomatch advisory remains open under its dated
exception. No zero-bug, unlimited-composition or production-publication claim
is made. The next composer slice should define layered layouts/placement before
adding further scroll behavior; it must preserve a meaningful mobile/print
reading order and avoid covering interactive content.


## Catalog coverage audit

This maps the 25 registered section types against the final source composer.
A similar visual arrangement does not establish semantic or behavioral parity.
The evidence is the finite `compose-schema.ts` node union and the corresponding
renderers in `engine/packages/zellij/src/sections/components/`. Keep this as a
remaining-work inventory, not an instruction to remove dedicated sections.

| Existing section(s) | What composing can provide today | Missing parity |
| --- | --- | --- |
| `compose` | Its own validated tree, styled groups and presets | Remaining structure/content/behavior below |
| `media-accordion` | The same shared synchronized-media component | Already reusable as a child; its surrounding heading is authored separately |
| `video-embed` | The same player, sources, captions and transcript in source rc.11 | Already reusable as a child; not in published rc.10 |
| `hero`, `hero-split` | Heading, copy, image, actions, gradient reveal and split layout | Parallax, marked gradient runs, badges/icons, terminal and semantic highlight lists |
| `feature-grid`, `team-grid`, `logo-strip` | Grids of images, headings, copy and actions | List semantics, icons, whole-card links and the dedicated image/identity treatments |
| `cta-banner` | Headline, body and buttons | Copy-command control |
| `bento-grid` | Equal-column grids and styled groups | Per-tile cell spans, icons and whole-tile links |
| `media-panel`, `promo-grid` | Media beside or above copy | Overlaid copy, media scrims, authored placement, background-video behavior and per-tile dimensions |
| `showcase` | Static copy, image and action | Sticky scroll stage and visual transform ramp |
| `stats-band` | Static value/label pairs | Count-up behavior and numeric formatting contract |
| `code-cards` | Card headings and explanatory copy | Real selectable code blocks and filename treatment |
| `faq` | Static question/answer copy | Native disclosure semantics and controls |
| `gallery` | Images and captions in a grid | Full-size lightbox, dialog navigation and focus lifecycle |
| `markdown-prose` | Inline Markdown in text nodes | Block prose and the shared MDX component pipeline |
| `matrix` | Decorative rows of text | Real table structure, header associations and responsive cell labeling |
| `pricing-table` | Static headings, prices and buttons | Semantic feature lists and dedicated tier/featured treatments |
| `steps-panel`, `timeline` | Sequences of headings, text and images | Ordered-list semantics, automatic markers, icons, diagrams/terminals and sticky companion media |
| `testimonial` | Quote text and attribution as ordinary nodes | Blockquote/citation structure and testimonial carousel behavior |
| `card-carousel` | A static card grid | Horizontal track, navigation controls and whole-card links |
| `zigzag` | Repeated split groups | Dedicated alternating visual placement independent of source order and directional reveals |

The next layout increment should address layers and placement. Subsequent
content/interaction work should reuse the existing semantic components, with
separate acceptance for keyboard, resizing, no-JS, reduced-motion and print.
No coverage percentage is assigned: a static lookalike is not a completed
replacement for an interactive or semantic section.

## Visual review clarification

The six desktop/Android-sized/iPhone-sized light/dark captures were reviewed.
One downscaled preview appeared to omit part of an SVG. Diagnostic-only branch
`7dff129` changes only the manual probe workflow and a capture test; product
runtime is identical to `b1d31ff`. Run `44181878309` passes three repetitions
without retries and captures the image both in place and in close-up. The
original dark screenshot and complete repeated screenshot are byte-identical
(SHA-256 `98f0ae5c3bdbf3c0f545b2abfed6848da149909722097d25f6b6cf4eb1ad1b56`).
Their upper-half pixels also match exactly. This disproves missing artwork in
the captured page; no product or capture-harness fix is warranted. Keep the
initial observation and comparison in the [evidence](evidence/zellij-compose-video-2026-11-17.json),
rather than turning an image-preview concern into a claimed product bug.

## Semantic review history

| Before | After | Why |
| --- | --- | --- |
| Source links followed a figure caption | Commit `3de61c8` groups the visible caption and source/transcript links in one final `figcaption` | HTML requires `figcaption` to be the first or last figure child; visible content and ordering stay the same. |

The updated server-rendering regression passes with all 16 focused video tests.
Run `44184326067` repeated the full pipeline on that runtime. It superseded
`44179876080` after 31 successful jobs; its unfinished CLI job was cancelled by
GitHub's concurrency rule. Do not report the earlier run as 32/32 complete.

The follow-up `e8a8eaf` places the no-script-only stylesheet in the document
head and leaves only the native fallback media inside the body's `noscript`.
The [HTML noscript content model](https://html.spec.whatwg.org/multipage/scripting.html#the-noscript-element)
and [style placement rules](https://html.spec.whatwg.org/multipage/semantics.html#the-style-element)
support this separation. All 22 focused video/root-layout tests pass. Final-head
hosted acceptance must also recheck the existing head-annotation hydration probes.

## Hosting regression caught before merge

The head stylesheet attempt fails two existing acceptance cases in run
`44186274088` (Engine job `134403685670`): added head whitespace and hosting
annotation/CRLF both produce React hydration error 418. Ordinary documents and
four negative-control cases still behave as expected. The failed run and its
artifact `14374047934` remain recorded; it was cancelled after diagnosis.

The replacement uses a server-hidden play button that becomes available after
hydration, inside a frame that reserves its aspect ratio. No-JS self-hosted
video retains native controls; providers have an ordinary poster link to the
original source URL. The player needs no head element or inline fallback style.
Root layout source and its tests are restored exactly to the previously verified
version. The final figure caption remains the last child. All 505 engine tests
and engine types pass locally. A focused hosted probe runs the complete composer
browser tests and both document-preservation suites before the full run.


## Corrected runtime acceptance

Final runtime `04c92b546d6c7fb004bfefefb45854845c4cfafc` uses progressive
enhancement without modifying the root document. Diagnostic commit `a4bcce7`
changes only the manual workflow relative to that runtime. Its hosted run
`44187882059` passes 505 engine tests, workspace types, all 27 composer browser
cases without retries, the 59-route documentation-preservation check and all
six hydration-preservation cases, including both failures from the rejected
head-stylesheet attempt. Diagnostic artifact `14374054272` retains the captures.

All six final desktop/Android-sized/iPhone-sized light/dark screenshots have been
reviewed: two match earlier reviewed captures byte-for-byte; the four changed
captures retain complete illustrations, readable video alternatives and correct
mobile stacking. Full acceptance run `44189074537` is in progress on the exact
final runtime. The Engine job already passes 505 tests and documentation
preservation. The diagnostic workflow is not part of the pull request.


## Final verified result

[PR #202](https://github.com/mylife-inc/Zellij/pull/202) contains implementation
`aaab66a`, inline-playback correction `b1d31ff`, caption correction `3de61c8`
and final progressive-enhancement correction `04c92b5`. All 32 jobs in
[run 44189074537](https://github.com/mylife-inc/Zellij/actions/runs/44189074537)
pass on the first attempt at the exact final runtime. The earlier checkpoints
above are preserved as history and are superseded by this result.

The full showcase suite passes 427 cases without retries (140 conditional
skips). Engine checks pass 505 cases and preserve all 46 original routes, 282
heading IDs, 82 compatibility mappings and 29 unchanged images; 13 additional
routes also render, with no captured browser errors. The CLI export corpus
renders all 25 sections and four presets. All native/managed installation,
container, template, book/PDF and portable checks pass. The final full-run
artifact `14376286554` contains six composer screenshots, each byte-identical
to a reviewed initial or corrected-probe capture.

The final follow-up changes only this record, its evidence JSON and project
context. Runtime, active guides, dependencies, workflows and acceptance inputs
remain identical to verified `04c92b546d6c7fb004bfefefb45854845c4cfafc`.
Published rc.10 and production hosting remain unchanged. The next composer
priority remains layers/placement, followed by shared semantic/interactive
primitives and bounded scroll behavior; this is not universal completeness.
