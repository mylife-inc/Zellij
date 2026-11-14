# Rapid header focus restoration — 2026-11-13

Status: implementation prepared; hosted regression verification is pending.
This follows the [retained investigation](zellij-header-resize-focus-2026-11-13.md).
Public rc.6 remains unchanged.

## Reproduction and cause

The original Chromium assertion from source CI `43531633137` is reproduced
without retries in diagnostic `43537359369`, head `ba3c1fd`: 112 passed and
eight failed, all at the final hamburger-focus assertion. The diagnostic adds
observations of native focus/media events and React restoration state. All eight
failures show the same sequence: a focused hamburger is hidden at desktop width
and loses focus to `body`; the viewport returns to mobile before the media
handler runs. The remembered hamburger is visible again, so checking its current
rectangles incorrectly concludes that focus was not lost. The handler discards
restoration and leaves keyboard focus on the document body.

Representative case `raw-107-107.json`: hidden blur at 1050.3 ms (width 1280,
zero rectangles), then the media handler at 1073 ms (width 390, one rectangle)
sets `restore=false`. All eight event pairs and a complete representative
sequence are retained in [the evidence](evidence/zellij-rapid-focus-restoration-2026-11-13.json).

This is distinct from the earlier WebKit test precondition: browsers may coalesce
intermediate widths. The permanent rapid-resize test now observes the first
desktop closure before applying its remaining burst. The diagnostic retains
the original unacknowledged Chromium sequence as separate stress coverage.
[CSSOM event reporting](https://www.w3.org/TR/cssom-view/#evaluate-media-queries-and-report-changes)
and [HTML rendering order](https://html.spec.whatwg.org/multipage/webappapis.html#update-the-rendering)
explain why a handler cannot assume each transient viewport was reported.

A second baseline run, `43538255973` at `87c6049`, holds animation callbacks
through two real breakpoint changes. Navigation restoration fails in all four
Chromium and all four WebKit repetitions (eight failures); the 16 content/pointer
preservation cases pass. This gives a deterministic regression independent of
the probability of the original fast-resize race.

## Implementation and regression boundaries

The header records hidden focus loss when blur occurs, rather than inferring it
from the later state of the old element. One pending restoration intent survives
intervening breakpoint changes and drawer removal. A revision schedules its
post-commit animation frame; the frame uses the current breakpoint. It does not
clear a newer target by comparing stale mobile/desktop strings.

Any new focus destination cancels the pending move, as does a pointer press
outside navigation. A visible focused navigation control is preserved. New
browser regressions hold animation callbacks after the drawer opens while real
DOM commits and media-query events continue. They cover restoration after two
resizes, deliberate focus in page content or on the brand, and a pointer click
on non-focusable content. They release the queued frames and wait for subsequent
frames before asserting focus, so an immediate assertion cannot hide a later
focus theft. The test harness changes neither the app's event handlers nor the
browser's actual viewport/media-query behavior.

Local checks pass 490 engine tests, engine/showcase typechecks, engine build and
strict/deep guide validation (59 routes, 25 examples). All browser and container
acceptance remains on disposable hosted runners. Record final source and stress
results before calling the issue resolved or merging the implementation.

The first fixed probe (`43538892591`, `c650cd9`) passes all 24 navigation,
content-focus and brand-focus cases. Its eight pointer cases fail because the
test clicked the screen-reader-only h1 at x=-0.5, outside the viewport, in both
browsers. This did not exercise pointer cancellation. All eight click coordinates
and trace hashes are retained in the evidence. The corrected test clicks a
visible non-focusable paragraph and asserts receipt of its real pointer event
before releasing callbacks. No further engine change was required by this
finding; hosted validation of the corrected test remains pending.

The next probe (`43539410410`, `19d2dad`) passes 32 delayed cases and all 120
original rapid Chromium sequences without retries. Header stress passes 334 and
fails two WebKit cases; the full-workload stage therefore does not run. The
rapid case reveals a second precondition error: a role locator omits the drawer
while desktop CSS hides it, before React removes it. Permanent removal assertions
now use the drawer-root DOM locator, including the initial desktop acknowledgment.
The other case retains More after widening to 1600. Its trace records no painted
frame after that resize, but the cause is not established. Event/measurement
observations on the diagnostic branch will distinguish delivery from measurement;
do not count this failed stress run as acceptance.
