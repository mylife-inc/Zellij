# Cached editable sources — 22 November 2026

Status: implemented for candidate rc.19; local and hosted acceptance are in progress.
The published rc.18 assets remain unchanged.

Default `eject` and `new app` now share a version-matched source archive under
`$ZELLIJ_HOME/cache/sources/<version>/`. No installed runtime, package manager or
container is needed to generate the wrapper. The platform-independent package
contains the canonical editable wrapper, engine source/package files, built CLI
helpers, fonts and notices; it includes no Node runtime or installed dependencies.
Initial local packaging is approximately 8.3 MB compressed. Actual release sizes
and public-download acceptance must be recorded after packaging finishes.

Each output receives its own `vendor/zellij` copy. `new app --engine` retains an
explicit local dependency; `eject --image` and nonempty `ZELLIJ_IMAGE` retain the
selected stopped-image extraction path. `--offline` uses only a verified existing
source cache and is rejected with explicit image extraction. Running an editable
Next app still needs its documented Node/npm development tools. PDF, Next dev
and image creation are not migrated by this change.

Archive and file hashes, CLI version, file inventory, portable paths, links,
size limits and source/output separation are checked. Download workers own only
network state, allowing Ctrl-C during a stalled request without leaving a late
writer. Extraction and wrapper preparation are staged. Native `new app` keeps
its existing noncolliding dotfile/Git-metadata behavior, using the common
no-clobber file publisher when that directory already exists. This is not a
filesystem-wide transaction against arbitrary concurrent modifications.

The interactive wizard defaults its engine answer to `cached`; an explicit
local path is still accepted. All answers and source/destination preflight
finish before a download. Cancellation while answering writes/downloads nothing.

The previously recorded CI preamble wording is corrected in both GitHub and
external generators. Generated jobs and their version-conditioned prerequisites
are unchanged; existing acceptance fixture pins remain rc.18 until rc.19 is public.

Initial findings: an existing wizard test relied on the old implicit local-engine
answer; explicit-local regression cases now select that path and keep their
assertions. Cache-default behavior has separate real-package acceptance. An
intermediate Rust compile found a Camino/std Path type mismatch during verified
copying; the conversion is corrected. Initial native package checks pass cold
retrieval, shared offline reuse, independent copies, preserved Git metadata,
corruption/unlisted-file refusal, failed/truncated downloads and cancellation.
The actual npm install, strict/deep check, typecheck, production build and
HTTP base-path/search checks pass outside the repository on Intel Mac. The
source cache stays unchanged. Four cache unit tests and all 12 explicit-local
app/wizard regressions pass. The first Clippy run found four style lints and a
helper below the test module; those are corrected. Normal preflight, six-platform
and public release gates remain to be completed. Historical browser timing findings remain open.


First hosted attempt, `45003586391` at `ed5e768`, exposed an obsolete integration
expectation: `features.rs` still expected default ejection to require Docker.
The local development helper exports `ZELLIJ_IMAGE`, which had masked that
expectation in the first local preflight. The fixture now removes the inherited
image selection, uses a temporary resource home, checks default offline cache
failure without writes, and explicitly selects an image for runtime diagnostics.
The first attempt remains a failure; the corrected Rust suite is rerun without
the development shell's image override before the next hosted attempt.
