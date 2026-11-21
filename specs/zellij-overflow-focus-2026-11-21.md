# Desktop overflow focus — 21 November 2026

Fixing commit: `f80596b63ae7799c96c0181a1d0c4bcb6166e3e4`. Local regression acceptance passes; hosted review is
pending. Public CLI rc.16 is unchanged.

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

## Product scope review

All documented CLI command families have implementation handlers. The current
readiness record separates remaining investigations and delivery acceptance
from optional expansion, including broader import fidelity, additional citation
formats and unrestricted executable/animation composition. Older status pages
now point directly to that current record instead of presenting rc.7/rc.14
milestones as today's status.
