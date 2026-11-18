# Composer scroll stories — 2026-11-18

Baseline `590fe5e`, source rc.11, public rc.10. This continues the approved
composer roadmap. Controlled section reuse and rc.11 publication remain later.

## Contract

`story` is an ordered narrative of 2–8 scenes, each with a subordinate heading,
inline Markdown, an image or direct-file video segment, and an optional CTA.
Scene headings stay in normal source order. Wide expressive-motion layouts pin
the copy and media for a bounded scroll interval; the next scene enters through
normal page flow. Video seeks silently over `start..end` seconds and reverses
with scrolling. No autoplay, wheel/touch interception, global playback state,
scene overlays, audio synchronization or arbitrary animation timeline.

A story needs 760px viewport and container widths, fitting copy/media, Resize-
and IntersectionObserver, and no reduced-motion/forced-colors/print override.
Otherwise all scenes remain ordinary readable content with native video controls.
A local reading button, keyboard scrolling or content focus disables effects.
Media errors and out-of-duration segments also retain the reading layout.
Seeks wait for metadata, coalesce while seeking and apply the latest target.
Video metadata loads only as an enhanced scene approaches the viewport. Every
poster, asset, track, transcript, body and CTA retains schema reference metadata.

Stories cannot nest inside layer layouts or scroll stages. Scene arrays are a
finite vocabulary; no arbitrary child sections. Engine and native authoring share
bounds and cross-field rules. The `scroll-story` preset expands to editable YAML,
bringing the catalogue to seven without changing the previous six presets.

## UI review

| Before | After | Why |
| --- | --- | --- |
| One scroll stage has one transform range | Ordered scenes with bounded pinning intervals | Tell a sequence without authoring a custom controller |
| A video plays only in elapsed time | A silent authored segment follows scroll progress | Explain a transformation at the reader's pace |
| A motion preference is the main opt-out | Local reading control and keyboard/content-focus fallback | Keep the story accessible without changing system settings |

The motion is explanatory, opt-in through `motion: expressive`, has no easing
or post-scroll interpolation, and never delays a keyboard action. The design
uses [W3C's interaction-motion guidance](https://www.w3.org/WAI/WCAG22/Understanding/animation-from-interactions.html)
and the [HTML media seeking contract](https://html.spec.whatwg.org/multipage/media.html#seeking).
These references guide behavior, not a claim of comprehensive accessibility certification.

## Verification

Local schema/reference/render, native editing/rejection-before-write and actual
engine/native parity. Hosted Chromium/WebKit exercise real video forwards and
backwards, delayed metadata, failed/short files, keyboard/focus, independent
stories, container/text sizing, print/no-JS/forced-colors/reduced-motion fallbacks.
Review light/dark and scrub captures. Keep documentation preservation and complete
CI as merge gates. No local browser/GUI/container/LaTeX runs, production deployment
or release mutation. Verification and merge are pending.

## Early local findings

The first native run rejected the newly generated `exclusiveMinimum` schema
keyword before scaffolding any full starter. The video end field now uses the
existing inclusive numeric schema, with the shared `start < end` cross-field
rule enforcing the strict bound in both engine and native code. This keeps the
native schema allowlist explicit. Deep checking also caught an unresolved
showcase transcript route; the clip now links to its actual authored scene
and includes its complete silent-motion description.

## First hosted run

Source `88af13f` passes 520 engine tests, 815 Rust tests (five hosted-only cases
ignored), 21 native authoring cases, 20 release-policy guards, strict checks,
workspace types, formatting, Clippy and actual CLI/engine parity for all 25
section types and seven presets. Documentation preservation also passes.

Full run `44363366370` passes 496 showcase cases and fails nine new assertions
(three cases across three browser projects). Focused run `44363433317` passes
96 cases and fails the same nine without retries. The track assertion omitted
the asset version query string; the delayed/failing-network route matchers also
omitted it, so they did not intercept requests. Correct those URL matches and
wait for actual seek completion before inspecting the decoded frame. The full
run was cancelled after retaining evidence; no runtime correction was needed
for these test failures. Hosted acceptance is pending a corrected run.

Visual review also found that the pinned scene's inset omitted the announcement
bar. The story now measures the full sticky header and observes size changes,
with a browser assertion that the copy clears the header. A WebKit dark-mode
capture showed faint inherited body text; new computed-colour assertions check
that concern before accepting the captures.

| Before | After | Why |
| --- | --- | --- |
| Pin inset used only the navigation-height token | Inset includes the actual sticky header and its announcement bar | Keep every scene line below visible chrome |
