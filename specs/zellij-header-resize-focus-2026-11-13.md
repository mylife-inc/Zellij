# Header resize focus follow-up — 2026-11-13

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
checks (59 routes, 25 authored examples). Hosted acceptance is pending; the
initial candidate stress run passed 250 cases and failed two WebKit first-resize
cases before the header had initialized (`data-measured=false` in both traces).
The test now waits for measured navigation and asserts initial focus before
resizing; source code is unchanged. These failures are retained in the evidence.
