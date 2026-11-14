# CLI rc.7 preparation — 2026-11-14

Baseline `474fc8f97edf28a3692cf3ee92a91559047e3d91`, after shared book glossary
and index acceptance. Public rc.6 remains unchanged during preparation.

## Scope

Version the six native Rust packages and standalone site recipe at
`0.2.0-rc.7`. Collect shared equations, statement/proof blocks, numeric citations,
glossary/index, responsive keyboard-focus fixes, Scoop distribution/diagnostics
and explicit generated-CI runtime selection in versioned release notes.
The three published template archives keep their existing versions and bytes.
No extra LaTeX packages or runtime-image dependencies are introduced here.

The release workflow already waited for Scoop but its `always()` publication
condition only required verify/binaries/docs success. A new test evaluated the
actual condition against dependency outcomes and reproduced publication being
allowed after `scoop: failure`. Publication now additionally requires Scoop
success; failed, cancelled and skipped required jobs prevent publication.
Independent image failures retain their documented separate-release behavior.
The initial failing test is retained in `/private/tmp/zellij-rc7-gate-before.log`.

## Gates

Local release preflight, full hosted source CI, source merge and reviewed tree
identity precede an immutable tag. Release workflow and public downloads then
verify six native platforms, four image variants on both Linux architectures,
template archives and actual installed-binary PDF output. Update the owned
Homebrew/Scoop channels and active documentation/consumer pins after publication;
verify actual old-to-new managed and standalone upgrades before scoped rc.6
retirement. Record exact artifacts, commits, jobs and failures as observed.

Local browser, GUI, container and actual LaTeX execution remain delegated to
hosted CI. Live providers and production-domain migration remain deferred.
Unexplained browser timing findings, multilingual print, BibTeX/custom styles,
richer book editions and remaining package channels retain their existing
status. Preparation is not publication, and passing tests do not prove zero bugs.

## Source acceptance

All 27 jobs in [CI `43642650594`](https://github.com/mylife-inc/Zellij/actions/runs/43642650594)
pass at source `81315063f9ef2f7550fddd85c245db0fc521bb28` without workflow
reruns. The six native packages, four Homebrew and four Scoop combinations,
offline browsers, prefixed Next, native images, engine and showcase checks pass.
All 24 PDFs/192 page captures match the previously reviewed glossary/index
source layouts byte-for-byte. Local helper checks pass 37 cases, including the
reproduced and fixed Scoop publication gate; formatting, Actionlint and committed
version checks pass. The [source evidence](evidence/zellij-rc7-preparation-2026-11-14.json)
retains all job IDs and capture hashes. The final record-only update changes
context and preparation evidence, with no runtime, workflow or test differences.
The normal local release preflight still precedes tagging after merge.

## Merge, local preflight and release start

PR #175 merges as `f3d55f6ab9f63e0e59daca6844ff8dbb01ddefe0`. Its tree
`076631bcab2e01e102336d399b0bcc3f9ac83b76` matches reviewed `5bbb60d`.
The normal `zrelease` helper passes 796 Rust tests (five hosted-only cases),
490 engine tests, typechecks, schema consistency, formatting and Clippy, then
creates immutable tag `v0.2.0-rc.7` at that merge.

The merge's `[skip ci]` message suppresses automatic push workflows. Automatic
approval review initially rejected a manual dispatch because it assumed a
release was already running; no dispatch command executed in that attempt.
Two exact-head API queries found zero runs. A guarded command checked again
before starting the first [Release run `43646140679`](https://github.com/mylife-inc/Zellij/actions/runs/43646140679).
At that checkpoint publication/public-download acceptance was pending. The
subsequent verified outcome is below; no rc.6 distribution has yet been removed.

## Published acceptance

**2026-11-14: v0.2.0-rc.7 is published** at immutable tag/merge
`f3d55f6`. All 33 jobs in release `43646140679` pass. Both release repositories
have 21 byte-identical assets; all three template archives retain their previous
bytes and versions. Six public native installs and both Linux image checks pass;
16 public PDFs/128 page captures match the reviewed content and pixels.
Shared equations, theorem/proof blocks, numeric citations, glossary/index,
responsive focus fixes, Scoop diagnostics and generated-CI runtime selection
are included. The actual standalone rc.6-to-rc.7 upgrade passes.

The owned Homebrew tap at `ebdcc19` and Scoop bucket at
`0d57926` publish the attached, verified rc.7 recipes. All four public
lifecycle checks and all four actual rc.6-to-rc.7 upgrade checks pass for
each channel. The upgrade checks verify both installed CLI versions and
portable builds; they use actual HTTPS release archives. See
[release evidence](evidence/zellij-rc7-release-2026-11-14.json).
Documentation/consumer migration and scoped rc.6 retirement are tracked
separately; production hosting and deferred live provider checks are unchanged.

All 13 development-image jobs in `43649847900` pass at released source
`f3d55f6`, publishing all four `main` variants on amd64/arm64 and building/serving
from the published default image. The documentation update at `fb2767c` has no
diff in engine, Docker, package-lock or development-image workflow inputs.

## Documentation and consumer acceptance

PR #176 passes all 27 jobs in `43648322918` at `fb2767c` without workflow
reruns. All six generated consumer build workflows pass against public rc.7;
provider publication is skipped. The 378 showcase cases pass without retries
(140 conditional skips), and preservation retains all 46 original routes,
13 additions, 282 heading IDs, 82 compatibility mappings and 29 original images.
Both owned package channels pass four public lifecycle and four actual
rc.6-to-rc.7 upgrade checks. The final records-only update touches context and
specs/evidence, with no runtime, active guide, fixture, test or workflow changes.
Replacement static/Next publication and scoped rc.6 retirement follow the merge.
