# PDF configuration, preview and static delivery — 2026-11-12

Base: `3b42b03`. This increment extends the first PDF pipeline from PR #151.
Merged in PR #153 as `71b97e4a40ca8954df71cbf81d621620169f8c6f`, with the same tree as tested head
`ecfbfba8e6390680f481052cbcc3fc3a08d7379d`. It is not in published rc.5.

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
wording; its runtime-not-probed assertions remain unchanged. Final results and hosted evidence follow below.

The first focused preview test passed failure retention, stale-build rejection,
recovery, configuration toggles, HTTP PDF retrieval and cancellation during a
held fake-runtime invocation. Real typesetting remains hosted-only: no local
Docker/Podman/nerdctl, browser or GUI was used. New hosted coverage exercises
configured default A5 plus A4/12pt and Letter/10pt and captures their pages.

Existing Homebrew Node could not start because of a missing simdjson library;
checks use the bundled Node 24 runtime without modifying host packages. A guessed
built CLI path and unsupported `check --help` were corrected to the package's
`bin/zellij.mjs check <content> --strict --deep` entry point.

## Final verification and closure

Run `43376564097` passed all 23 jobs on the final source without reruns.
Local verification passed 767 Rust tests with zero failures and five existing
hosted-only cases that run separately in CI, formatting, warnings-denied Clippy,
script syntax/actionlint and strict/deep documentation (59 routes / 25 examples).
Hosted results include 490 engine tests, 356 showcase cases with 124 conditional
skips, Linux/macOS/Windows portable preview/cancellation and book browser checks,
six native packages and all four Homebrew targets. Documentation preservation
retains 46 original routes, 282 heading IDs and 29 byte-identical images.

Real PDF acceptance passed Docker Linux amd64/arm64 and rootless Podman Linux
amd64. Nine PDFs have matching receipt/content revisions, checksums, correct
paper dimensions and embedded/subset/Unicode fonts. A5/11pt has eight pages and
13 fonts; A4/12pt has seven pages and 12 fonts; Letter/10pt has seven pages and
14 fonts. All 66 final page captures match 22 reviewed images. A4 and Letter
pages were individually reviewed; A5 matches the prior reviewed layout. No
clipping or overlap was seen in these sample books. This is not a general
typographic or PDF/UA certification.

All three runtimes passed actual HTTP PDF previews, watched revision changes and
graceful cancellation. Both Docker architectures additionally built and served
configured PDF static images as nonroot `nginx`, with matching web/PDF receipts
and no LaTeX, Node or Cargo in the runtime. Uncompressed image sizes were
7,241,042 bytes on amd64 and 14,040,817 bytes on arm64. The separate pinned
optional LaTeX image was not enlarged or added to normal runtime images.

The superseded run `43376193640` failed CLI/Linux/macOS on the same stale doctor
wording assertion; `ecfbfba` fixes it without removing runtime-probe assertions.
All three PDF jobs had already passed in that attempt. Its failed/cancelled
history is retained in [the evidence](evidence/zellij-pdf-configuration-2026-11-12.json).
No local runtime/browser/GUI, live provider test, release or production docs
deployment ran. Node engine inputs are unchanged, so no development engine
publication was needed.

Next, publish a new version of the book starter that uses this publication model
and optional PDF settings, then prepare rc.6 with updated installation/docs and
normal release acceptance. Previously published template bytes remain immutable.

## Remaining boundaries

Native release/public documentation publication, Next-image PDFs, advanced print
semantics/templates, multilingual/RTL print and deferred live-provider acceptance
remain separate work. No zero-bug or full book-publishing completion claim is made.
