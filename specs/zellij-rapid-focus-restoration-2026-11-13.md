# Rapid header focus restoration — 2026-11-13

Status: reproduced rapid-focus defect resolved by PR #164 (`cc88bc0`).
Separate timing findings remain open below.
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
acceptance remains on disposable hosted runners. These were the initial local
checks; final source and hosted results follow below.

The first fixed probe (`43538892591`, `c650cd9`) passes all 24 navigation,
content-focus and brand-focus cases. Its eight pointer cases fail because the
test clicked the screen-reader-only h1 at x=-0.5, outside the viewport, in both
browsers. This did not exercise pointer cancellation. All eight click coordinates
and trace hashes are retained in the evidence. The corrected test clicks a
visible non-focusable paragraph and asserts receipt of its real pointer event
before releasing callbacks. No further engine change was required by this
finding; hosted validation was still pending at that checkpoint.

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

## Final fixture and verification

The instrumented diagnostic `43540562985` (`46ea8c4`) passes 32 delayed cases,
120 original sequences, 336 header cases and 668 full-workload cases (64 skips),
with zero retries. Removing observers in `43541465663` (`7bb512f`) passes 31
delayed cases but exposes one WebKit fixture precondition timeout: only one of
two native media-query events arrives while all application animation callbacks
are held. It does not reach the focus assertion. The final fixture explicitly
keeps native rendering active using the captured native animation function,
while continuing to hold application callbacks. This models delayed application
restoration without suppressing the browser's rendering loop. No additional
engine behavior, assertion timeout or retry is introduced.

Control `43542045252` (`6d68b6d`) uses that exact final fixture with the unfixed
engine from `4ec0ec9`: all eight navigation-restoration cases fail at the focus
assertion, and the other 24 cases pass. The fixture therefore still detects the
real defect. Final clean stress `43541961094` (`1000876`) passes 32 delayed,
120 original rapid, 336 header and 668 full-workload cases: **1,156 passes,
64 conditional skips, zero retries**, using four workers. The engine matches
source `36c7882` exactly; the verification follow-up copies the four-line native
rendering driver into the permanent test. Diagnostic observers, raw tracing and
probe workflows remain on their separate branches.

All 27 effective source CI jobs in `43540700668` pass at `36c7882`. Showcase
reports 378 passes, 140 conditional skips and no flaky cases. The first attempt
passes 26 jobs and fails Engine's hosting-annotation/CRLF preservation case at a
15-second hydration-readiness wait. An unchanged-source rerun of only Engine
passes. The original failure is retained, not attributed to a source correction.
Engine tests (490), docs preservation, six native packages, four Homebrew targets,
four Scoop lifecycle jobs, portable Linux/macOS/Windows checks, both image
architectures, combined manifests and the three PDF runtime jobs pass.

PR #164 merged as `cc88bc010a196a1a121eece958fd6cab136c29eb`. Its tree equals the
tested source tree `456deb56d19aebf5ef33192de124582aa3865700`. All 13 development-
image jobs in `43543832220` pass at the merge, publishing all four `main` variants
on Linux amd64/arm64 and building/serving from the published image. The guide
and durable context describe the completed
behavior; public rc.6 binaries, tags and assets remain unchanged.

## Remaining timing findings

The isolated WebKit overflow timeout in `43539410410` and hydration-readiness
timeout in the first attempt of `43540700668` have no established cause. Later
passes are useful evidence, not fixes for those failures. The verification
follow-up adds the exact page URL to future readiness errors while preserving
the original cause, timeout and browser-error checks. Investigate these findings
before the next candidate; no zero-bug claim is made by this checkpoint.

Subsequent [bounded investigation](zellij-browser-timing-investigation-2026-11-13.md)
passes 120 overflow repetitions and 40 full-site annotation checks without
retries. Neither finding reproduces; both remain unexplained. The follow-up adds
bounded failure-only diagnostics instead of changing runtime behavior based on
an unestablished cause. See that record for capture acceptance and retained
probe failures. This advances the investigation without erasing the two findings.


The verification follow-up changes no product runtime. It carries the exact
native-frame fixture from the passing stress run, the readiness URL diagnostic,
the precise menu wording and these records. Its bounded local checks are
strict/deep guide validation (59 routes, 25 examples), checker syntax, evidence
integrity and whitespace checks; the actual updated browser fixture is verified
by the baseline/fixed hosted pair above. Repeating unrelated platform builds for
this documentation/test-only follow-up is unnecessary.
