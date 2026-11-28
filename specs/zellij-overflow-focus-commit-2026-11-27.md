# Overflow focus and menu commits — 27 November 2026

Fixing source: `f3ac599`. A queued callback could consume a newer overflow
focus request before React committed the corresponding menu. The correction
binds each request to its intended visible-item count. An older effect leaves
a request for a different layout pending; the matching effect restores focus
after that layout commits. Newer focus and pointer choices still cancel
restoration, as established by `f80596b`.

This is a separate defect from the historical missing-frame resize stall and
documentation hydration timeout. Neither historical timing finding is closed.

## Natural reproduction

The [structured evidence](evidence/zellij-overflow-focus-commit-2026-11-27.json)
records a reduced comparison using Linux, Playwright 1.62.1, WebKit 2336,
two workers and zero retries. Each case performs five narrow/open/Escape/wide
cycles. The variants are plain DOM with and without visual effects, a reduced
React header using the real measurement/focus hooks, and the full showcase
with and without passive callback instrumentation. The monitor observes
existing callbacks; it does not create a frame loop.

The first probe, `46083601538`, reports 52 passing and eight failing cases.
Its workflow incorrectly appeared successful because a `tee` pipeline masked
the test exit status. The investigation workflow subsequently specifies Bash
with pipefail; production CI was unaffected. Five failures belonged to a
simplified-fixture focus shortcut and were corrected in that fixture. These
are not eight established product bugs or a passing qualification run.

The revised probe, [46084440939](https://github.com/mylife-inc/Zellij/actions/runs/46084440939),
reports 71 passing and nine failing cases. Both DOM variants pass all 16 cases.
The reduced React variant loses focus six times, the instrumented showcase
twice and the uninstrumented showcase once. All nine failures have the correct
wide menu but no focused navigation control. Rendering continues. No historical
More-retained/frame-stall assertion reproduces.

In the instrumented showcase's sequence 5, a measurement and an older focus
callback run at 4,063 ms. The new menu commits at 4,067 ms, removing the focused
More button. No replacement focus callback is scheduled, and subsequent frames
continue with the document body focused. The old callback consumed the new
request while the old menu was still present. The uninstrumented occurrence
confirms that focus loss is not confined to the callback monitor.

The [instrumented trace](evidence/browser-overflow-focus-2026-11-27/showcase-observed-sequence-5-trace.zip)
and [uninstrumented trace](evidence/browser-overflow-focus-2026-11-27/showcase-control-sequence-8-trace.zip),
with their state captures, remain available after hosted artifacts expire.
Artifact/member hashes were checked. Requests are confined to the repository
showcase's loopback server and contain no authorization or cookie headers.

## Controlled regression and fix

The regression queues real application callbacks while native rendering and
ResizeObserver delivery continue. It runs a new measurement followed by an older
focus callback in one task, before React commits the new menu. It checks each
layout/focus precondition and the final focused navigation control. Both resize
directions are covered. This intentionally controls callback order; it does not
simulate, reproduce or explain a native rendering stall.

An initial harness run, `46085057718`, reaches the intended focus failure in all
four narrow cases; four wide cases stop earlier because no second measurement
is queued. The final harness changes the container width by one pixel to queue
a real observer measurement consistently before the critical callback pair.

| Source / run | Chromium and WebKit, two directions, two repetitions each |
| --- | --- |
| Unchanged engine, `3a8902e` / [46085478688](https://github.com/mylife-inc/Zellij/actions/runs/46085478688) | All eight fail at the final focus assertion; every setup assertion passes |
| Corrected engine, `f4f7640` / [46085554070](https://github.com/mylife-inc/Zellij/actions/runs/46085554070) | All eight pass, zero retries or skips |

The qualified engine source is identical to `f3ac599`. The merged regression
differs from the qualified test only by a blank line. Local engine types,
focused browser-test types and all 556 engine tests in 50 files pass. Normal
source CI additionally covers the prior user-choice, mobile-menu, file-export
and documentation behavior. Experimental fixture builders, monitoring code and
the investigation workflow stay on their separate branch.

## Delivery boundary

The source fix and menu-guide update do not change CLI syntax or configuration.
Rc.24's immutable renderer remains unchanged; a future candidate must package
this correction. Ordinary documentation publication uses the corrected source
after merge. Provider verification and WinGet are outside this investigation.
