# PDF configuration, preview and static delivery — 2026-11-12

Base: `3b42b03`. This increment extends the first PDF pipeline from PR #151.
Implementation/verification is in progress; it is not in published rc.5.

## Contract

A versioned portable book can declare top-level `pdf: {enabled: true, paper: a5,
font-size: 11}`. Defaults are false/A5/11pt; paper also accepts A4 and Letter,
and font size accepts integer 10/11/12. Unknown fields, wrong types, null settings,
missing book manifests, hosted and unversioned use fail. The classic template's
fonts/margins remain controlled by its renderer. Settings enter the PDF receipt,
separate from the shared manuscript revision.

Build, view and portable dev accept mutually exclusive `--pdf`/`--no-pdf`.
Flags override configuration for the invocation/session; otherwise each build
uses the saved choice. Explicit standalone `pdf` always exports, irrespective of
`enabled`. Web-only generation and `pdf --check` remain native. Doctor checks
runtime/user mapping only when the book explicitly enables PDF, and never pulls
or starts its image.

The shared CLI preparation path stages web/PDF together, verifies that authored
inputs have not changed during preparation, and leaves publication to the caller.
Preview uses the existing HTTP/revision lock, serial debounce, cancellation and
error recovery. Configuration toggles remove/recreate the PDF and download link
together. Edits during typesetting discard stale work; cancellation cleans up its
container and retains the previous export.

Static provider delivery and static-image packaging use this same path; PDF
failure stops upload/packaging. The LaTeX image remains a separate optional build
tool and is not copied into a static image. Next-image PDF delivery fails explicitly;
web-only Next adaptation remains available with PDF disabled. No live provider
verification is performed in this increment.

Generated CI inherits configuration. Any explicit PDF settings require a pin of
0.2.0-rc.6 or later (even when disabled, because rc.5 cannot parse the field).
This is a compatibility floor, not publication evidence. GitLab enables Docker
and shared temporary build paths for PDF; GitHub/Circle use existing native Linux
machine runners. PDF-enabled Next-image jobs fail generation before writes.

## Verification history

Local checks caught and corrected intermediate mechanical parse/argument errors.
A regression exposed serde's null-to-default mapping; `pdf: null` now fails at
site loading. The first static-image fixture put its new PDF handler in a shared
shell stub, shadowing two existing hosted-test handlers; PDF behavior is now
injected only in the new case. Those failed attempts are retained in the local
verification logs. The broader suite then found repeated Ctrl-C handler installation
in sequential provider tests. Cancellation now installs one process handler with
weak owners for active operations, retaining interruption after a PDF build
finishes and execution continues into upload/packaging. A subprocess regression
confirms Ctrl-C during subsequent image packaging exits 130. A later full pass
found an exact human-message assertion needing the new "portable web-only"
wording; its runtime-not-probed assertions remain unchanged. Final results and hosted evidence will be appended.

The first focused preview test passed failure retention, stale-build rejection,
recovery, configuration toggles, HTTP PDF retrieval and cancellation during a
held fake-runtime invocation. Real typesetting remains hosted-only: no local
Docker/Podman/nerdctl, browser or GUI was used. New hosted coverage exercises
configured default A5 plus A4/12pt and Letter/10pt and captures their pages.

Existing Homebrew Node could not start because of a missing simdjson library;
checks use the bundled Node 24 runtime without modifying host packages. A guessed
built CLI path and unsupported `check --help` were corrected to the package's
`bin/zellij.mjs check <content> --strict --deep` entry point.

## Remaining boundaries

Native release/public documentation publication, Next-image PDFs, advanced print
semantics/templates, multilingual/RTL print and deferred live-provider acceptance
remain separate work. No zero-bug or full book-publishing completion claim is made.
