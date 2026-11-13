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
