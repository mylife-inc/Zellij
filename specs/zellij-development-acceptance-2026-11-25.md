# Development commit boundary and Windows file locks — 25 November 2026

Status: deterministic regressions pass; rc.24 corrections under
qualification. Rc.21 remains installed and published. Rc.22 and rc.23 tags
are retained; neither failed candidate was promoted to an installation channel.

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

The corrected native fixture must pass against the actual cached runtime,
including Chromium; all six hosted targets and tagged public-download gates
remain required before publishing. Documentation publication and installation
channel promotion remain gated on that release result.
