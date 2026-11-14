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
