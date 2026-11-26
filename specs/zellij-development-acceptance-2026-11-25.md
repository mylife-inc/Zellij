# Development commit boundary and Windows file locks — 25 November 2026

Current status, 26 November: fixes `288913c` and CI allowance `a3933b6`
ship in rc.24 at immutable `2c3b7da`. All 37 source CI, nine source PDF and
55 release jobs pass. All six tagged native packages and public installations
pass, including tagged Windows/Linux Chromium and the real Windows lock fixture.
Independent public Mac and installed default-home checks pass. Rc.22/rc.23
failures and tags remain retained. See [the rc.24 record](zellij-rc24-release-2026-11-26.md).

## Observed release failure

Rc.23 source `5425125` passed all 37 CI and nine PDF jobs and merged as
`120cea9`, with the identical tree. Tagged release `45774039883` then failed
Linux x64 native job `139755977143`, at the invalid-edit assertion in
`scripts/check-cached-development.mjs`. The search index still matched the last
valid edit, but the reload marker differed from the one the test had saved.
See [the retained evidence](evidence/zellij-rc23-release-2026-11-25.json).

The watcher writes search and assets before publishing their final reload
marker. The old acceptance fixture waited for those outputs and captured the
marker immediately. It could therefore capture the previous marker, submit an
invalid edit, and then see the pending valid edit's marker commit before the
invalid edit was rejected. That is an incorrect test boundary. An unchanged
retry would not correct the fixture, so no retry was requested.

## Controlled reproduction and correction

Two watcher tests hold the actual marker writer after search output becomes
visible. The old equality assertion fails reliably when an invalid edit is
submitted during this barrier. Waiting for the completed valid edit's marker
before submitting invalid content preserves both the valid index and marker.
All twelve watcher tests pass with these regressions included.

The native acceptance fixture now saves edits atomically, staging outside the
watched content. This also avoids an in-place truncate being interpreted as an
unrelated valid empty document. It submits text and asset edits as separate
generations, waits for each generation's changed reload marker, and then checks
the corresponding output. Only after those commits does it save invalid MDX.
The existing invalid-index and unchanged-marker assertions remain. Browser
acceptance still exercises edits to multiple files, automatic reloads, images,
search and invalid-edit recovery.

The watcher loop and published marker format do not change. This correction
does not resolve the product's historical browser timing findings. The earlier
Windows PDF mitigation also remains unchanged and retains its reviewed hashes.

## Separate Windows publication failure

Windows x64 release job `139755977358` reached browser image reload acceptance
and timed out expecting image width 180 while the browser retained width 160.
The captured preparation error was `EPERM` renaming the staged file over
`public/_zellij/dev-revision.json`. Requests were active, but the log does not
identify which handle held the file. A browser-server reader or another process
is a possible cause, not an established attribution.

The shared prepared-file writer previously attempted the replacement once.
It now retries only Windows `EPERM`, `EACCES` and `EBUSY`: six additional attempts
with delays of 10, 20, 40, 80, 160 and 320 milliseconds. These are scheduled
delays, not a hard wall-clock deadline. It keeps the old destination intact and
never unlinks it to force replacement. Persistent errors retain the old file,
remove the staged file and propagate the error. Other platforms and error codes
still fail immediately. This does not promise recovery from indefinitely held
locks or a filesystem-wide transaction across every generated output.

Seven tests exercise transient and persistent sharing errors, old-file retention,
temporary cleanup and unrelated/platform-specific errors. All nineteen focused
writer/watcher tests and engine typechecking pass locally. The hosted Windows
fixture acquires an actual deny-delete `FileStream` handle, proves the old
one-shot rename fails, releases the handle during the retry window and verifies
replacement. It then keeps the lock through the retry budget and checks output
preservation and cleanup. This real Windows check runs before the cached browser
acceptance in both source CI and release qualification; it is not yet a local
Windows pass.

Rc.24 is a fresh candidate because rc.23's immutable tag and versioned images
already exist. The five new website template archives remain unpublished and
receive rc.24 minima before their first publication. Previously published
template identities are preserved.

## Remaining qualification

At `288913c`, all six hosted native package jobs pass, including the corrected
fixture, Windows/Linux Chromium and the real Windows lock test. All nine PDF
jobs also pass in run `45778612077`. Tagged public-download gates remain required
before publishing. Documentation and installation-channel promotion remain
gated on that release result.

## Browser job setup allowance — 26 November

Source run `45778612146` initially finished with 35 successful jobs, two cancelled
browser jobs and one conditional skip. Jobs `139768292330` (rich-file export)
and `139768292492` (showcase) reached their 20-minute job limit after Ubuntu
dependency installation took 1,117 and 1,034 seconds respectively. The package
logs show slow Ubuntu mirror transfers; browser CDN downloads completed quickly.
The actual tests were cancelled after 24 and 125 seconds, with no reported
assertion failure. The earlier passing rc.23 source run installed the same
browser dependencies in 38 and 34 seconds.

Only those two jobs were retried on the unchanged commit. Rich-file job
`139782381209` passed after a 343-second installation. Showcase job
`139782380694` then reached the same job limit: setup took 1,102 seconds and
the browser run was cancelled at case 155 of 721 without a reported assertion
failure. Attempt two finished with 36 successful jobs, one cancellation and
one conditional skip. Its retained log SHA-256 is
`862d727cc26e621cf78bd20bb89fa12386fc2ed0e97a44a929356998baa71274`.
The earlier complete showcase run needed roughly nine additional minutes for building,
721 browser cases, deployment-prefix checks and development acceptance.

The two affected overall job limits are raised from 20 to 30 minutes to allow
both dependency setup and the complete acceptance workload. Browser assertions,
individual test deadlines and retry settings are unchanged. This is a CI setup
allowance, not a resolution of the retained historical browser timing findings.
The revised workflow still needs qualification before merge and publication.
