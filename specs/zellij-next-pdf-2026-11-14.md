# PDF-enabled Next images — 2026-11-14

Baseline `daa865b`; source rc.8 remains unreleased. Published rc.7 supports PDF
static delivery. This increment completes the configured-PDF path for Next
images without adding a new command or installing a runtime automatically.

## Contract

`pdf.enabled: true` on a portable book is honored by `zel ni` / `image next`,
saved `pub`, and shared Next-image delivery, including generated CI. Existing
English/LTR print boundaries remain. An ordinary web-only Next build keeps its
previous path. Live provider checks remain deferred.

The native compiler resolves web documents and print inputs from the same
publication. The existing optional isolated LuaLaTeX pipeline runs before image
building. Snapshot checking rejects manuscript/configuration edits made during
typesetting. Failure or cancellation prevents packaging/publication and leaves
existing exports intact; no authored files are rewritten.

The prepared Next content uses a private temporary print directory. Before
publishing it, the adapter verifies the PDF signature and a regular JSON receipt
whose revision and SHA-256 match the publication and bytes. Only `book.pdf` and
`pdf.json` move to `assets/pdf`; the temporary TeX, logs, assets and intermediates
are removed. Image layers never receive that working directory or toolchain.

Every reading page exposes an ordinary downloadable anchor to
`/_zellij/assets/pdf/book.pdf`, automatically prefixed by the existing asset
renderer. Receipt URL is `/_zellij/assets/pdf/pdf.json`. The engine's inert bridge
accepts only literal `download: true`, not arbitrary handlers or filenames.
Keyboard activation and no-script reading remain supported.

GitHub, GitLab and CircleCI generation require CLI/engine rc.8 or later for
PDF-enabled Next images, including legacy image targets and Fly/Railway image
plans. Older pins fail before writing, even with force. Static PDF settings keep
the existing rc.6 minimum. Generation does not assert release availability.

## Verification

PR #180 merges as `5ed3aeb`, with the same tree as verified head `6804917`.
All 29 jobs in [CI `43688478658`](https://github.com/mylife-inc/Zellij/actions/runs/43688478658)
pass without workflow reruns. Local validation passes 802 Rust tests (five
existing hosted-only cases ignored), 490 engine tests and 95 helper tests;
workspace types, engine build, warnings-denied Clippy, formatting, workflow lint,
JavaScript syntax and strict/deep docs (59 routes, 25 examples) also pass.

Regression checks cover shared publication/TeX identity, receipt/signature/hash
validation, private-file exclusion, prefixed download rendering, build-once
publication, cancellation/stale-input failures, preservation of source and
previous exports, dry runs, and rejection of old CI pins before file writes.

Both native Linux image jobs publish to a disposable loopback registry, pull
and serve study/root and technical/prefixed books. Actual keyboard downloads
with JavaScript on/off match the recorded PDF hashes and publication revisions.
All four PDFs pass structural and text checks; 48 page captures match the
corresponding native layouts exactly. Sixteen browser viewport captures cover
both architectures, desktop/mobile and scripting modes; representative study
and technical layouts are visually reviewed. The final images run nonroot and
exclude LaTeX, Pandoc, npm, print sources, logs and private intermediates.
A print validation failure preserves the local/registry image identity and
previous export. PDF samples are 59,352 and 63,248 bytes; recorded Docker Size
values are storage-driver dependent, not compressed download sizes or a direct
cross-architecture comparison.

All three ordinary PDF jobs also pass: 30 PDFs/264 page captures across amd64
Docker, arm64 Docker and amd64 rootless Podman. The 30 TeX sources are byte-identical
and 264 page captures pixel-identical to the previous editions milestone.
Those comparison downloads come from the first run's successful PDF jobs; the
final source changes only Next-test server cleanup, and all three PDF jobs pass
again. Exact artifact provenance is retained in the
[machine-readable evidence](evidence/zellij-next-pdf-2026-11-14.json).

Showcase passes 378 cases without retries (140 conditional skips). Documentation
preservation retains 46 original routes, 13 added routes, 282 original heading
IDs, 82 compatibility mappings and 29 byte-identical images, with no captured
browser errors. Native Linux/macOS/Windows packages and Homebrew/Scoop lifecycle
checks pass. No local browser, container daemon or LaTeX is run.

### Retained initial acceptance failure

First run `43688080406` completed the study PDF/download checks on both
architectures, then attempted a fresh registry pull while its own test server
still held the image tag. Docker correctly refused to remove that tag. Commit
`6804917` stops/removes only the task-owned server before the second pull.
No runtime, renderer, print or template inputs changed; the final run passes.

### Development images and records

All 13 jobs in [development-image run `43691586062`](https://github.com/mylife-inc/Zellij/actions/runs/43691586062)
pass at merge `5ed3aeb`: eight native builds, four combined manifests and the
published default image's site build/HTTP check. Only `main` variants are
refreshed. The first startup probe reset; the next succeeded in the existing
bounded readiness loop. Exact manifest digests and startup results are in the evidence.
The records-only follow-up changes context/specification/evidence files;
runtime, active guides, versioned templates, tests and workflows are unchanged.
Public rc.7 remains the available candidate; source rc.8 is unreleased.

The earlier editions, classic outputs and public template archive versions stay
unchanged. Remaining print design customization, multilingual/RTL print,
additional citation/index capabilities and installation channels remain open.
No zero-bug or 1.0-complete claim is made.
