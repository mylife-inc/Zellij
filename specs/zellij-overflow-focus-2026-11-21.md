# Desktop overflow focus — 21 November 2026

Fixing commit: `f80596b63ae7799c96c0181a1d0c4bcb6166e3e4`, PR #229. Local
regression acceptance, all 37 required source CI jobs (`44890416888`),
documentation validation and the final full browser run without retries pass.
One source CI job is intentionally skipped. Public CLI rc.16 is unchanged.

## Reproduction and correction

After a desktop resize moves links into or out of More, the header commits the
new layout and queues keyboard-focus restoration. If the user focuses content
or the brand, or clicks non-focusable page content before that frame runs, the
old callback pulls focus back into navigation. The callback captured its target
and never checked whether the user's newer choice cancelled it. The breakpoint
restoration path already respected those choices; desktop overflow did not.

The corrected callback reads the current restoration intent. Focus changes and
pointer presses outside navigation cancel it. A visible focused element is
preserved, and automatic restoration uses `preventScroll`.

The regression delays application animation callbacks while native rendering
and ResizeObserver delivery continue. One frame batch commits the new menu;
the test then makes a deliberate focus/pointer choice and releases later work.
Every layout and input precondition is checked before the final focus assertion.
On the original engine, all twelve deliberate-choice cases fail in Chromium and
WebKit; eight ordinary-restoration/reciprocal-resize controls pass. With the fix,
the full header/mobile suite passes 90 cases with 46 conditional skips, no
failures and zero retries. All 540 engine tests, both typechecks and engine/
showcase builds pass. See [evidence](evidence/zellij-overflow-focus-2026-11-21.json).

## Historical timing findings remain distinct

The original overflow job log still shows More surviving the 1600-pixel resize
past its five-second assertion. Its trace is not available from the current
run-artifacts endpoint; the earlier recorded observations remain evidence.
The hydration failure's original stack identifies the legacy-heading alias
navigation loop, but provides no failing URL or causal browser diagnostic.
Current readiness errors and failure captures already collect that information.
Neither log establishes a cause. This new focus correction does not establish
that either historical timeout is fixed; both remain open in the
[browser investigation](zellij-browser-timing-investigation-2026-11-13.md).

## Story-theme assertion race found during hosted acceptance

Source CI's first showcase run passes 550 cases, skips 167 and retries one
mobile story-theme assertion successfully. The original failure is retained.
Its trace shows the dark foreground token (`#f5f5f7`) already applied while
computed inherited text still reports the previous `rgb(29, 29, 31)`. The test
captures that old heading value once, then waits five seconds for text that has
already become `rgb(245, 245, 247)` to match it. The later frame shows both
heading and paragraph correctly light on dark; the frozen expected value is
stale, not evidence of a persistent contrast failure.

Test correction `e4d9d53` waits for the showcase's requested Cupertino foreground
and compares each related color pair in one browser evaluation. It changes no
product CSS, assertion timeout or retry count. Eighteen local repetitions pass
across desktop Chromium, mobile Chromium and mobile WebKit without retries.
The full Linux run without retries (`44891618278`) passes the corrected story
case but fails an independent menu assertion: 550 passed, 167 skipped, one
failed. That first failure remains recorded below; it is not counted as a clean
acceptance run.

## Menu anchor test selected the pre-hydration layout

The failed trace shows `data-measured="false"` after navigation. The test's
immediate More count returns zero, then measurement commits `data-measured="true"`
and moves Specs into More. Since the test skipped opening More, it waits for a
link that is inside the closed panel. Its click never occurs, so this does not
explain the older documentation hydration timeout after route navigation.

Test correction `e9e486b` waits for measured navigation and exercises both known
layouts: Specs inside More at 901px and directly visible at 1600px. Both must
navigate to the same anchored route. Twelve local repetitions pass without
retries; showcase types pass. No product code, timeout or retry setting changes.

Full Linux browser acceptance `44892687678` passes **552 cases**, with 169
conditional skips, zero failures/flaky cases and **zero retries**. Its branch
differs from source `e9e486b` only by the scoped verification workflow. The
first failing runs remain recorded above. Native/runtime checks are reused from
the unchanged implementation in source CI `44890416888`. Documentation validation
`44890416851` passes; publication is intentionally skipped for a pull request.

## Product scope review

All documented CLI command families have implementation handlers. The current
readiness record separates remaining investigations and delivery acceptance
from optional expansion, including broader import fidelity, additional citation
formats and unrestricted executable/animation composition. Older status pages
now point directly to that current record instead of presenting rc.7/rc.14
milestones as today's status.

## Merge and publication

PR #229 merges as `4d1654954e027dc6905775f90a5ab60672c0fa07`, identical in
content to reviewed `ab7fb7a`. Automatic production run `44893589525` verifies
deploy `6b0170369ba63b773b8d74fa`. The live home and menu-guide pages hydrate
without captured browser errors; the guide contains the new focus explanation.
Main base-image publication `44893589526` passes all 13 jobs, including building
and serving the showcase from the published image.

Duplicate main CI `44893589549` was explicitly cancelled after confirming tree
identity, not counted as a pass. Public CLI rc.16 and all release archives remain
unchanged; a later renderer/CLI candidate must carry this correction.
