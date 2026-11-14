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
