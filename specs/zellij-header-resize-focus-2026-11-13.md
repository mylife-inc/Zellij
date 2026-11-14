# Header resize focus follow-up — 2026-11-13

Status: **resolved by PR #160 (`dece6c9`)**, with the same tree as tested
head `17be3af`. The dated discovery and unsuccessful experiments below remain
part of the record. A new rapid-resize assertion failed once in run `43531633137`
and remains open; see the final section below.

CI run 43508089027 at bd0ae9b687805e5cfebf936cd72cae90dc9f58e0, job
132170883473, recorded one flaky desktop case in
`examples/showcase/e2e/header-layout.spec.ts:92`. Opening the mobile drawer at
390 px then resizing to 1280 px removed the dialog and released body scroll
lock, but the first desktop menu item remained inactive through the five-second
focus assertion. Automatic retry #1 passed. Results: 355 passed, one flaky,
124 conditional skips. No workflow retry was requested.

The CI runtime-selection change touches neither this test nor the header.
Retained local trace:
`/private/tmp/zellij-ci-runtime-browser/header-layout-measured-ove-d09c3-ock-when-resized-to-desktop-desktop/trace.zip`.
SHA256: `f2236975900541884795538ff99dc105f22c306f60df71d9b1ef6607931a1d64`.
Hosted artifact: `playwright-failure-details` (seven-day retention).

Inspection points: `chrome/nav-bar-client.tsx` snapshots `document.activeElement`
inside the media-query change handler before closing the drawer and schedules
focus in a requestAnimationFrame. `styles/chrome.css` hides the drawer above
900 px. `chrome/hooks.ts` restores the old trigger in focus-trap cleanup; the
overflow observer can also schedule focus relocation. The existing trace does
not establish which ordering caused the failure. These are investigation leads,
not a proven root cause or a completed fix.

Next: reproduce on hosted Chromium with retries disabled, capture active-element
and layout transitions around the breakpoint, then fix and exercise drawer/menu
focus, rapid reciprocal resizes and preservation of focus outside navigation.
Do not weaken the focus assertion or mark the issue resolved because a rerun
passes. All browser acceptance must remain hosted; local trace inspection is
permitted. Address before Scoop packaging, then resume the expansion backlog.

## Reproduction and implementation

The initial isolated diagnostic ran 80 repetitions with no failures (run
43516946486). Repeating the complete desktop workload four times with retries
disabled reproduced the original failure in run 43517554663: 579 passed, one
failed, 64 conditional skips. The captured event order establishes the cause:
CSS hid the focused drawer panel and Chromium moved focus to `body` at 657.3 ms;
the media-query event arrived at 688.9 ms with the drawer still mounted. The
handler checked only the current active element, so it skipped restoration.
Focus-trap cleanup then tried the now-hidden hamburger. No desktop control
received focus.

A second regression reproduces an independent defect in all eight repetitions
(run 43517974674, 56 passed/eight failed): after focusing a drawer link, scrolling
past the header's transparent-to-solid threshold resets focus to the dialog.
The inline `onClose` prop changes on each parent render; because the focus trap
uses it as an effect dependency, a header state update cleans up and restarts
the trap. A stable close callback preserves the user's place.

The implementation retains navigation focus lost specifically when CSS hides
a control, includes an open drawer when deciding whether to restore focus, and
schedules restoration after panel closure commits. It checks the current
breakpoint and respects focus moved into document content before the scheduled
restoration. New regression coverage exercises reciprocal and rapid resizes,
content focus, and scroll updates while the drawer is open. Desktop WebKit now
runs the header-layout suite alongside Chromium in ordinary CI.

The temporary focus-event instrumentation and diagnostic workflow remain on the
probe branch only. The product change retains the behavioral assertions and
browser coverage. See the accompanying evidence JSON for exact tested heads,
run IDs and captured event order. All browser acceptance ran on hosted runners.

Source implementation first committed at `302cf86`; regression readiness
corrected at `6078d82`. Local verification passes 490 engine tests, engine and
showcase typechecks, engine build, workflow lint and strict/deep documentation
checks (59 routes, 25 authored examples). The initial candidate stress run
passed 250 cases and failed two WebKit first-resize cases before the header
had initialized (`data-measured=false` in both traces).
The test now waits for measured navigation and asserts initial focus before
resizing; source code is unchanged. These failures are retained in the evidence.

## Final verification

Hosted diagnostic run `43520294777` at `6078d82` passes all 252 targeted cases
(12 repetitions in Chromium/WebKit), then 636 cases in four repetitions of the
full desktop workload (64 conditional skips). Both use four workers and zero
retries. Its engine source, permanent tests and browser configuration exactly
match final PR head `17be3af`. All 23 jobs in CI `43520478220` pass on that head,
without workflow reruns; showcase reports 370 passed, 132 conditional skips
and no flaky cases. Engine tests, docs preservation, six native packages, four
Homebrew targets, both native image architectures, combined manifests, three PDF
runtime jobs and Linux/macOS/Windows portable acceptance all pass.

PR #160 merged as `dece6c9cde379eb6edf4548d8bde5d64dbc3ef1a` with the tested
tree unchanged. All 13 development-image jobs in `43524812741` pass at that merge, publishing all four `main`
variants on Linux amd64/arm64 and building/serving from the published image.
The menu guide and durable context are updated. Public rc.6 remains unchanged;
these source fixes will be included in the next candidate. This resolves the
two reproduced focus defects, not a claim that every product bug is eliminated.
See [structured evidence](evidence/zellij-header-focus-2026-11-13.json).


## New rapid-resize finding during Scoop acceptance

Run `43531633137`, job `132250636619`, at Scoop source `7788140` records a
first-attempt failure in the desktop rapid-drawer-resize case at
`examples/showcase/e2e/header-layout.spec.ts:162`. The final mobile hamburger
remained inactive after widths 1280 → 390 → 1280 → 390. The drawer was closed
and body scroll lock released. Its automatic retry passed; totals are 369
passed, one flaky and 132 conditional skips. **This finding remains open.**
The earlier fixes and their no-retry verification above remain part of the
history; they do not establish that this newly observed case is resolved.

Scoop PR #162 changes no engine or browser-test code. Retained trace:
`/private/tmp/zellij-scoop-focus-failure/header-layout-measured-ove-9e6df-during-rapid-drawer-resizes-desktop/trace.zip`,
SHA-256 `e20f3e9416396ca0631b89ec4a5817a4754c80152b597e9477d042a8eb028741`.
The hosted `playwright-failure-details` artifact retains the original evidence.

Next: reproduce with retries disabled and instrument pending restoration,
focus events, drawer removal and media-query changes. A possible race is that
another breakpoint arrives after the drawer's focused node is disconnected but
before pending focus restoration runs; the current handler can then clear the
request. This is an inspection lead, not an established cause. Preserve the
behavioral assertion and check focus deliberately moved into content. Prioritize
this finding before cutting the next candidate; browser acceptance stays hosted.


A hosted no-retry diagnostic at `c0a2bed` (run `43533726998`) passes 239 targeted
cases and fails one WebKit rapid-resize case at the drawer-closure assertion.
Its captured events show only final-width 390 resize events after the drawer
opens, with no desktop media-query change. This exposes a test precondition:
WebKit may coalesce intermediate widths, so the test cannot assume it observed
a desktop transition. The diagnostic at `9f24261` first waits for closure at the
initial desktop width, then keeps the remaining rapid burst and final focus
assertion. This does **not** establish the cause or resolution of the original
Chromium focus failure. See [the retained events and outcome](evidence/zellij-rapid-focus-discovery-2026-11-13.json).
The original event JSON's `testInfo.status` was captured before finalization;
the Playwright failure, not that premature field, determines the result.


The revised diagnostic run `43534470738` at `9f24261` passes 240 targeted
Chromium/WebKit cases and 632 cases from four full-desktop repetitions (64
conditional skips), with retries disabled. The diagnostic changes only the test
precondition and instrumentation; engine code is unchanged, and neither the
probe workflow nor its test changes are merged with Scoop. These passes confirm
that the clarified scenario is viable, not that the original Chromium failure
has been explained. Continue investigating its pending restoration/event ordering
before claiming that finding resolved.
