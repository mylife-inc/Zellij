# Mobile navigation with section views

Date: 2026-11-15. Baseline: `259567e` on main; source rc.9, public rc.8.
The user approved the proposed mobile interaction before rc.9 publication.

## Behavior

The hamburger opens the main section names. A group opens a dedicated view
with a heading and its links; existing mega-menu headings, descriptions and
icons stay in that view rather than adding unnecessary levels. Back returns
to the root menu with its previous scroll position and focused section. Close
and Escape exit the entire menu. Direct links navigate immediately and close;
external/new-tab metadata is preserved. Each opening starts at the main menu.

The existing menu model supplies both desktop and mobile navigation. Phones
(up to 600px) use a full-width panel; tablets up to the existing 900px desktop
breakpoint use a 30rem side panel and retain backdrop dismissal. Back/Close
controls remain outside the independently scrolling menu, with safe-area
padding and 44px targets. Root CTA/GitHub links remain in a bounded scrolling
footer. Light/dark themes use existing tokens; no dependency/font is added.
Animations honor reduced motion and `motion: none`.

One portal dialog replaces its active content; no layered dialogs or duplicate
hidden focus targets are added. Focus moves to the section heading on entry,
to its originating button on Back, and follows the existing modal cleanup on
close. The mounted open component owns view state so closing/reopening resets
it, including across responsive breakpoint changes. Header scroll updates do
not restart the trap or reset the active view.

The old mobile leaf/root renderers omitted `target`/`rel` metadata. This change
also restores the already-configured new-tab behavior in both menu levels.

## Verification

Existing drawer and header focus/resize tests stay active. New hosted browser
cases cover root/section/Back/Close, direct and nested navigation, real external
popup behavior through an intercepted destination, root scroll restoration in
a short viewport, document scroll on Escape, resize reset, light/dark layouts
at 320/390/768px, reduced motion, motion none, section keyboard trapping and
tablet backdrop dismissal. Chromium and WebKit run the new suite in phone and
desktop keyboard contexts. Screenshots capture root and section views in both
palettes. Actual CLI website exports also open/close their direct-link mobile
menus and retain screenshots.

Native portable books retain their independent sidebar navigation. Public rc.8,
production hosting and deferred live-provider checks are unchanged. Source CI and visual review are complete below; merge and development-image
evidence will follow.

### First hosted pass and corrections

Run `43774698117` at `18739b3` produced 103 successful template captures, but
its showcase suite found eight failures (two checks across four projects) and
one existing Safari overflow fixture retry. The reduced-motion wildcard lost
`animation-name` specificity to the Back selector. The scroll trace showed a
22px movement when the test focused the sticky-header trigger and another 22px
when modal cleanup restored it. Commit `eef7755` adds `preventScroll` to modal
restoration and test setup, asserts scroll at every stage, and gives the reduced
motion override sufficient specificity. The overflow fixture now waits for fonts
and the final item measurement before keyboard interaction. The superseded run
was cancelled; its failures are preserved in the evidence rather than waived.

Local engine checks pass again (491 tests and type checking). Run
`43775728142` then passed all 36 new navigation cases. It exposed a Linux watch
fixture that kept Safari polling after intentionally stopping its server; the
trace shows a refused status fetch, reported by WebKit as a page error despite
the caught fetch. `0e2f688` closes the browser session first while retaining the
real Ctrl-C and output-preservation checks. `91d3571` also waits for hydrated
navigation before an existing mobile-Safari search-shortcut case, which retried
in that run. Full acceptance is now tracked by `43776685463`; superseded runs
remain in the evidence.

Run `43776685463` again passed the new navigation cases, but the search fixture
used a desktop measurement attribute that intentionally stays false on phones.
That produced two fixture failures (416 passed, 140 conditional skips).
`a99cc94` replaces it with the established bounded shortcut-opening pattern from
`search.spec.ts`. The current full acceptance run is `43777347096`.

## Accepted source and visuals

All 30 jobs in `43777347096` pass at `a99cc94`. The showcase reports 418 passed,
140 conditional skips and no retries; all 36 new navigation cases pass. Both
browser engines also pass prefixed deployment and development-reload acceptance.
Local verification passes 491 engine tests, type checking/build, strict deep docs
(59 routes), eight-template packaging/parity, actionlint and script syntax checks.

The final template job retains 103 captures, byte-identical to the initial
reviewed set. All four final root/section captures also match their reviewed
versions. Ten reviewed menu previews are retained in
`specs/evidence/mobile-navigation-2026-11-15/`; machine-readable evidence records
artifact IDs, log/image hashes, fixing commits and earlier unsuccessful runs.
The final records update changes no runtime, guide, template, test or workflow
inputs from the verified head. Public rc.8 stays unchanged; rc.9 is unreleased.
