# Cached editable sources — 22 November 2026

Status: source acceptance completed for candidate rc.19 at `3d3b426` (PR #239).
The candidate is published; postpublication and channel acceptance are in progress. Immutable rc.18 assets
remain unchanged.

Default `eject` and `new app` now share a version-matched source archive under
`$ZELLIJ_HOME/cache/sources/<version>/`. No installed runtime, package manager or
container is needed to generate the wrapper. The platform-independent package
contains the canonical editable wrapper, engine source/package files, built CLI
helpers, fonts and notices; it includes no Node runtime or installed dependencies.
The tagged source artifact is 8,183,144 compressed bytes and contains 1,439
hashed files plus its manifest. Archive and every file digest were independently
verified. Both public repositories now serve source bytes identical to that tagged artifact;
all 42 asset names, sizes and SHA-256 digests match between the repositories.

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


The second attempt (`45003811871`, `d1ace51`) passed the installed-wrapper checks
on both Linux and both Mac architectures, then Windows x64 caught a real source
path defect. The dotfile-preserving publication branch passed Windows path
separators from `WalkDir` to the shared portable-path validator. That branch now
normalizes its generated relative paths to `/`, with a nested-file/Git-metadata
publication and no-clobber regression. The original Windows failure is retained;
all platform gates must pass on the correction before publication.


## Corrected source acceptance

Source `bedf63e` passes all 37 required jobs in
[45004886385](https://github.com/mylife-inc/Zellij/actions/runs/45004886385);
merge `3d3b426` has an identical tree. The browser suite passes 552 cases, with
169 conditional skips and no reported failure or retry. All six native targets
pass empty-PATH generation, offline reuse, independent engine copies, retained
Git metadata, corruption/extra-file rejection, interrupted/failed requests and
cancellation, followed by real npm install, strict/deep validation, typecheck,
production build, prefixed HTTP/404 and search acceptance outside the repository.

The Windows path correction is `bedf63e`; the obsolete integration expectation
is corrected in `d1ace51`. Five cache unit tests now cover the additional
nested-file/Git-metadata publication regression. Normal local preflight passed
836 Rust tests with five existing ignores, 546 engine tests, formatting, Clippy,
types and schema checks before the additional path regression; its focused unit,
Clippy and actual-package checks also pass. Fresh pre-tag dependency audits
report no findings.

The duplicate main run `45007103274` later reproduced the historical
documentation-readiness timeout at `/cli/dev#what-it-is-for`. Its diagnostic
records no browser errors but also times out during state capture. This remains
unexplained, and no retry is counted as a correction. See the
[timing investigation](zellij-browser-timing-investigation-2026-11-13.md).
The recurrence does not erase the failed attempt or turn unrelated passing
acceptance into a timing-fix claim.

The [release evidence](evidence/zellij-rc19-release-2026-11-22.json) records publication,
public downloads, managed installation channels and final guide gates separately.
