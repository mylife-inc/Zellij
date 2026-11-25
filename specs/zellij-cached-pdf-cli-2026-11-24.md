# Cached PDF command integration — 24 November 2026

Status: development integration, not a published compiler package. Released
rc.19 and the current default still use the container backend. The
[scoped adapter licensing decision](zellij-pdf-distribution-decision-2026-11-24.md)
is approved and applied; final distribution review remains pending.

## Command behavior

The global `--pdf-backend cached|container` selector (environment default
`ZEL_PDF_BACKEND`) controls the shared typesetter. `container` remains the default.
The cached path serves `pdf`, `export pdf`, PDF-enabled portable builds and
previews, and the existing shared image/deployment preparation paths. A web-only
build does not acquire the compiler. Rich YAML/MDX sections do not gain a new
print model through this change; the existing portable-book contract remains.

`pdf --offline`, `build --offline` and `view --offline` prohibit tool downloads.
PDF-enabled offline builds require the cached backend and both verified caches.
They do not silently fall back to a container or a system executable. Portable
web-only builds/previews also accept `--offline`. `pdf --check` still validates
the manuscript natively, without downloads, cache writes or typesetting.

The optional PDF package uses `$ZELLIJ_HOME/cache/pdf/<CLI-version>`; the existing
renderer package provides private Node 24 from `cache/engines/<version-platform>`.
The default home is `~/Dev/config/zellij`. The complete renderer inventory is
verified before using its Node binary. PDF resources contain no second Node copy.
`ZEL_PDF_BASE_URL` is the maintainer download override, with the same HTTPS or
loopback-HTTP constraints as package acquisition. The default URL is the matching
version in `mylife-inc/releases`; no PDF asset is published there by this change.

The released-package contract is format 2, kind `zellij-pdf-toolchain`, exact CLI
version, completed review marker, exact source archive identity and preserved
distribution notices. It cannot accept a format-1 qualification package. The
maintainer harness retains its separate `pdf-qualification` cache and manifest.
Runtime validation checks identity and complete file hashes; the licensing review
itself must be established by the release process, not inferred from a boolean.

Generated print inputs go to a disposable sibling staging directory. The Rust
owner launches the verified process adapter and checks its bounded protocol and
artifacts. The existing strict typesetting checks and transactional publication
remain shared with the container path. Failure, cancellation or a stale watched
revision retains the previous complete output. The receipt identifies the Wasm
backend, exact toolchain manifest, matching source archive, engine/resource and
format/font identities, private Node version, publication revision and PDF hash.

For a portable site with PDF enabled, `doctor --pdf-backend cached` inspects both
caches without downloading, executing or probing a container. It reports
`download_required` for an absent cache and rejects a damaged one. Container
diagnostics and web-only behavior remain unchanged.

## Verification

Local Rust checks cover profile separation, required distribution records,
offline behavior, read-only diagnostics, explicit backend overrides and previous
output preservation. Existing container tests remain in place.

`scripts/pdf-wasm/installed.test.mjs` exercises the actual compiled CLI, real
Wasm typesetter and real private Node, using repository-owned manuscripts with
an empty system PATH. Its four scenario groups cover:

1. Native compatibility checks, read-only diagnostics and an offline cold miss.
2. Cold download of both archives, backend/source receipt and cache diagnostics.
3. Offline `export pdf` and `build --pdf`, matching book/PDF revisions, plus damaged
   runtime rejection preserving the last successful output.
4. An offline watched preview serving a PDF, then regenerating a matching revision
   after an authored Markdown edit.

The loopback server deliberately supplies a **synthetic release-profile fixture**.
Its simulated review entry does not authorize compiler distribution; the actual
approved adapter license and original third-party notices are retained. The
fixture archive is never published or uploaded. Only the verification receipt leaves the runner. Six-platform CI
runs the same installed-command checks alongside qualification/source/package
parity; the parity job requires all six receipts. This is implementation
acceptance, not verification of a public release download.

Initial local Mac acceptance passes all four scenario groups. Targeted Rust tests
pass 191 tests, with five existing integration-dependent unit tests skipped;
hosted CI runs their configured acceptance separately. Final branch CI results
and any fixes are recorded below when available.

Warnings-denied workspace Clippy and formatting pass after changing the test-only
immutable cancellation token from a `const` to a `static`; compiling this module
in the CLI exposes Clippy's interior-mutable-constant lint. The six package guard
tests also pass through both the CLI and qualification harness. All 44 Python
source/package tests pass.

Source-policy review also corrects an earlier combined "rebuild and relink"
completion label. PR #244 proves the clean compiler build but explicitly does
not qualify modified-library relinking. The policy now retains that distinction
and lists the relink exercise as outstanding; the actual hashes are unchanged.

The initial exact-head PDF run `45530308441` at `5711954` rejects stale source
inventories before typesetting: the policy clarification also changes the
generated inventory's review fields and policy hash. Regeneration from the
verified original inputs changes only `openItems`, `resolvedItems` and
`policySha256` in the three source locks. Package/resource bytes, upstream hashes
and verification guards remain unchanged. The initial failure is retained.

Normal CI `45530964794` at `013dc78` passes all 37 jobs with one conditional skip;
Showcase reports 552 passed, 169 conditional skips and no flaky results. Docs
validation `45530964705` passes. PDF run `45530964691` passes source correspondence
and five complete platform jobs, but the Intel Mac cache harness stops a cold
acquisition/compile at its 45-second acceptance deadline before the installed
CLI tests. The failure captured no phase data and does not establish a compiler
defect or prove a scheduling cause. Local phase instrumentation then measures
about 25 seconds cold and 20 seconds offline, both producing equivalent PDFs.

The harness now retains acquisition/verification/pass timings and uses the same
120-second compilation allowance as installed-command acceptance. Read-only
checks retain their 45-second bound; compiler limits (120 seconds per pass,
600 seconds overall), negative deadline tests and cancellation checks are
unchanged. The complete CI job gets 45 minutes for cold Rust compilation, full
source packaging and the expanded suite. The original timeout remains recorded;
later success is not proof of its underlying host timing cause.

The owner subsequently approved the PDF-only GPL-2.0-only scope. The exact GNU
license text, separate scope/copyright file and original MIT attribution are now
retained by both qualification archives and checked by the source-package tests.
The runtime fixture simulates only distribution review, preserving the actual
adapter license. No compiler package is published by this approval.

## Subsequent packaging work

PR #253 merged as `f8578e5` with the tree of `9498a75`. Final normal CI
`45535195352` passes 37 jobs with one conditional skip; final PDF CI `45535195306`
passes all eight jobs without retries. Post-merge CI, base images and docs pass
in `45538460837`, `45538460744` and `45538460646`. The timeout above remains
unexplained; later success does not establish its cause.

The [packaging increment](zellij-pdf-release-packaging-2026-11-24.md) completes the
modified-library relink evidence and component review, introduces the paired
release packager and replaces the simulated compiler fixture with the actual
reviewed package. The earlier fixture and policy descriptions above are retained
as the implementation history.

## Remaining release work at the integration milestone

With the scoped licensing decision approved, finish the exact component/source review
and implement paired runtime/source release packaging. Then qualify real public
downloads, switch the default only when those packages are available, and ship a
new candidate with matching documentation. No release flag, source grant or
compiler publication is implied by this integration.
