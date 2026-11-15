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

Local tests exercise publication/TeX identity, PDF and receipt validation,
private-file exclusion, prefixed anchor rendering, build-once publication,
failed or stale-input rejection, source/output preservation, dry-run behavior
and old CI pin rejection. Full results and hosted acceptance remain pending.
New hosted acceptance builds, publishes to a disposable loopback registry,
pulls and runs study/root and technical/prefixed Next books on native Linux
amd64 and arm64. It checks actual downloads with JavaScript on/off at desktop
and mobile widths, PDF structure/text/hash/revision, absent private files and
LaTeX executables, and preservation of the prior image/export after a print
validation error. PDFs, page images, browser screenshots and size reports are
retained. No local browser, container daemon or LaTeX is run.

The earlier editions, classic outputs and public template archive versions stay
unchanged. Remaining print design customization, multilingual/RTL print,
additional citation/index capabilities and installation channels remain open.
No zero-bug or 1.0-complete claim is made.
