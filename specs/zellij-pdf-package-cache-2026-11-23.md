# Versioned PDF package and cache — 23 November 2026

Status: maintainer qualification. Released rc.19 still uses its pinned PDF
container. This increment does not publish third-party runtime/source archives
or enable a hidden installed CLI option.

## Package

The optional archive contains the seven runner/recipe files, the existing compact
hash-addressed resources, and original notice files including the complete EC
distribution. Private Node and the much larger corresponding-source archive are
separate. `toolchain.py` verifies every member of that source archive, its overall
hash and manifest, and the current build recipes before copying runner/notice
bytes from it. The runtime manifest records the exact source archive identity.
Stale source recipes, resource corruption and an existing output fail before
publication. Archive ordering, metadata and gzip timestamps are fixed.

The qualification package is named `zel-pdf-qualification-vVERSION.tar.gz` with
a checksum sidecar. It remains explicitly experimental and records redistribution
review as incomplete. The historical `url.sty` wording and final component review
remain open; this increment does not rewrite terms or infer clearance.

## Cache

`pdf_package.rs` is compiled by the `pdf_cache` maintainer example only. It requires
an explicit endpoint and uses the shared cancellable download implementation.
Archive size is bounded at 32 MiB; expanded data at 128 MiB, individual files at
64 MiB, manifests at 1 MiB and the inventory at 2,048 files. Only regular file
entries and portable paths are accepted. Unknown manifest fields, incompatible
versions/Node major, unsafe paths, case aliases, links/junctions and missing,
unlisted or changed files fail. Every cache reuse checks the entire inventory.

Entries live at `$ZELLIJ_HOME/cache/pdf-qualification/VERSION`, isolated from future
released caches. Offline misses never download or create directories. Acquisition
owns a private `.download-*` stage and removes it on error/cancellation; existing
damaged caches are preserved and diagnosed. Concurrent successful acquisition is
accepted only after validating the winning entry. The explicit endpoint supplies
both archive and checksum; this is not independent mirror authentication or
protection against arbitrary concurrent hostile filesystem mutation.

The harness passes an absolute Node 24 executable and invokes the existing Rust
PDF supervisor using the scripts/resources from the verified cache. Production
dispatch will reuse the renderer cache's verified private Node. The current
example is not evidence that installed `zel pdf` already does so.

## Verification

Synthetic Python regressions cover deterministic packaging, source identity,
stale recipes, modified originals, resource inventory, output preservation and
invalid versions. An initial deterministic test catches mutation of the caller's
file inventory; the builder now copies that dictionary before adding its manifest.
Rust unit tests cover cache corruption, version/source/review state, size bounds,
unsafe names, missing/unlisted entries, links, duplicate archive members,
case aliases, gzip corruption and pre-cancelled transfer.

Real-package acceptance builds a repository-owned book with an empty system PATH
after cold acquisition, then again offline without another HTTP request. It
compares the PDFs and source/package identities, uses Unicode/spaced paths,
rejects six forms of cached damage (including Windows junctions), and checks
checksum/truncated-download failures and cancellation during three stalled
transfer phases. Previous outputs and cache staging cleanup are checked. The
cancellation timer belongs to the maintainer harness; real Ctrl-C delivery is
already covered by the shared transfer's six-platform renderer acceptance.

The first local cache comparison incorrectly expects identical PDF bytes across
separate invocations. Inspection identifies only LuaHBTeX's two timestamps and
document-ID pair; after masking exactly those fields, every byte matches. Extracted
layout text also matches. The acceptance comparison now requires those exact
metadata fields and compares every remaining byte. Success receipts are emitted
only after all thirteen cache scenarios pass, so a failed subtest cannot leave
a success-shaped receipt.

The workflow assembles the full package locally on six platforms; it uploads
receipts and synthetic PDFs only. Exact final-head results belong in the PR.
The existing nineteen-manuscript process/isolation/native-comparison matrix
continues alongside package-cache acceptance.

Local verification passes 38 Python source/packaging tests, eight Rust harness
tests, Clippy with warnings denied, formatting and all thirteen real-package
cache scenarios on Intel macOS. The qualification archive is 11,326,373 bytes
with 529 files (29,502,907 expanded bytes); its separate source archive is
429,781,852 bytes. These are qualification measurements, not published assets.

Pre-CI review also identifies a Windows packaging issue: Python orders Windows
`Path` objects without case sensitivity, whereas the canonical archive verifier
orders portable names with case sensitivity. Distribution packaging and source
manifests now sort relative POSIX strings explicitly. A regression covers the
mixed-case root/source/notice names; the six native builds qualify the same
archive on Windows for the first time. This adds a thirty-ninth Python test and
changes the source/runtime hashes from the initial local measurements above.
PDF source text is checked out with LF on every platform. An aggregate receipt
job requires all six package/source identities to match each other and the
independent source job, and all 78 cache scenarios / twelve compilations to pass.

Remaining: distribution review, production package/release arrangement, installed
CLI dispatch and private-Node reuse, offline propagation through every PDF call
path, diagnostics and user documentation, installed/public-package acceptance.
