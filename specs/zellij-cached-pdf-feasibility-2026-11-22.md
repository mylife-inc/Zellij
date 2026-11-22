# Cached PDF feasibility — 22 November 2026

Status: implementation groundwork and maintainer experiment, not a released
CLI backend. Published **v0.2.0-rc.19** is unchanged. The existing web/source
cache migrations remain released; the PDF migration is still in progress.

## Confirmed defect in the existing container path

On Intel macOS with Colima, an output under the shared repository was visible
but generated TeX inputs under the default `/var/folders` temporary directory
were not. Released `zel pdf` failed with `I can't find file book.tex`. Choosing
a shared `TMPDIR` made the same manuscript succeed. A separate attempt with an
output under an unshared system path failed with an output permission error;
that output-share prerequisite is distinct from the input-staging defect.

The fix stages **only generated print inputs** beside the prepared PDF output,
using the same share. It does not mount the authored project or relax the
container's network, identity, resource or read-only settings. Inputs are removed
after success/failure. The new regression fails on the previous implementation;
all ten PDF integration tests pass after the fix. Real Colima builds of the
classic and Arabic technical fixtures pass without overriding `TMPDIR`.
The fix is commit `0619869`; hosted acceptance is recorded below after CI.

## WebAssembly path

A native downloaded TeX binary would need a suitable isolation mechanism on each
OS. The feasibility runner instead executes the same LuaHBTeX generation in
WebAssembly with an in-memory filesystem and a fixed resource inventory. It
would reuse the renderer's private Node. Ordinary web downloads would not grow;
the typesetter and its data would be optional.

The candidate engine is LuaHBTeX 1.24.0 / TeX Live 2026, matching the generation
in the existing pinned Pandoc image. The upstream
[WasmTex engine release](https://github.com/corca-ai/wasmtex/releases/tag/engine-2026-f5d94438a7c75a23)
has a matching format and corresponding-source declaration. The probe uses its
core JS/Wasm, not its upstream Node adapter: that adapter uses synchronous
external HTTP lookups and does not provide the desired host-access contract.

The reproducible probe has 200 SHA-256-pinned files, 31,989,582 downloaded bytes
before archive compression; the engine Wasm itself is 8,077,323 bytes. This is
an observed feasibility inventory, not a final package size or license approval.
The exact source/data pins, bounded runner, tests and usage are in
[`scripts/pdf-wasm`](../scripts/pdf-wasm/README.md). No engine binaries are tracked
or uploaded as CI artifacts.

## Local observations

- The original eight-page classic fixture has identical extracted text/layout
  and page dimensions against the container baseline; a second run uses only
  cached files.
- The seven-page Arabic technical fixture has identical rasterized pages at
  1300 pixels on the long edge. Raw extracted text differs in Arabic Unicode
  presentation forms; NFKC-normalized text including whitespace/layout is
  identical. This distinction is retained rather than claiming byte identity.
- The hardened offline runner has ten passing regressions: corrupted resources,
  input links, guest host-access restrictions, virtual-file size limits,
  console/heap limits, endless-guest termination/recovery and typesetting/output
  failure handling.
  Initial tests assumed the socket module was available and allowed an absolute
  temporary write; those assumptions were false. The corrected tests explicitly
  verify unavailable socket modules and exercise file growth through an allowed
  relative virtual file. No failed test is counted as a pass.
- A console stress test exposed swallowed stream exceptions in the initial
  probe: memory stayed bounded, but libc translated the exception to EIO and the
  guest continued until timeout. Budget violations now notify the supervisor
  immediately before throwing. The bounded regression passes in about one
  second with a reduced 32 KiB test limit; normal operation retains 8 MiB.
- Follow-up review found MEMFS first writes bypass its expansion helpers.
  A new 320 MiB many-file regression fails on the initial runner (it accepts
  the write) and passes after `43b8c12`, which counts initial writes as well.
  The same fix counts terminal bytes before buffering a line, with a separate
  regression for output without a newline. These are prototype corrections,
  not defects in a shipped PDF backend.
- All **16 local cases / 48 passes** finish successfully: the three book editions,
  custom print styles, equations, statements, bibliography, glossary/index,
  Arabic editions/default-font selection and French/German/Spanish. The broad
  suite initially found five missing optical-size fonts; they were added to the
  pinned inventory before the complete successful run. Peak observed guest heap
  is 496,369,664 bytes; this is not total process-memory measurement.
  See the [evidence](evidence/zellij-cached-pdf-feasibility-2026-11-22.json).
  Initial six-platform workflow `45081234139` at `2228b58` passes all 16 cases
  per platform (96 cases / 288 typesetting passes), plus its then-eight
  regressions on each platform. Linux x64 additionally passes qpdf structure
  checks, normalized text/layout, page sizes/counts and **145 identical page
  rasters**. The hardened follow-up pins Node 24.22.3 on all six targets to match
  the private runtime; the initial Intel Mac runner used its cached 24.19.0.
  Current follow-up checks and merge state are on
  [PR #242](https://github.com/mylife-inc/Zellij/pull/242).
- Initial workflow `45080968117` was rejected before any jobs ran: job-level
  `env` cannot reference `runner.temp`. Actionlint reproduces that expression
  error. The bundle now uses `github.workspace`, and actionlint passes. This
  failed setup run provides no platform acceptance.
- A separate, original 100-chapter stress manuscript produces 302 pages in
  one pass (11,538 ms, 441,319,424 guest-heap bytes). This is a single-pass size
  probe only, not full publication acceptance or a total-memory benchmark.
- The upstream 154,968,364-byte corresponding-source archive matches SHA-256
  `e6a1f9b8ad84c7127368bb521217af381407fcc0d5c5518114345d55b470d637`.
  Its embedded manifest matches the exact core JS, Wasm and format bytes used
  here. It contains source/build records; this check does not replace a clean
  rebuild or the separate TeX-resource/font licensing review.

## Remaining before release

Complete resource coverage and component/font licensing, verify/provide exact
corresponding source, qualify isolation and resource budgets, package a versioned
optional artifact, connect Rust cache/PDF dispatch and cancellation, and run all
installed/offline/public-download acceptance. Update doctor/help/generated CI and
public prerequisites only when the backend is actually supported. Container
fallback behavior must be explicit. This record does not declare the PDF
migration, CLI completion or zero bugs.

## Unrelated checkpoint refresh

The final rc.19 receipt merge `2a3ac72` has successful main CI `45011754304` and
base-image run `45011754299`. The earlier historical browser timeout remains
unexplained; these later passes do not resolve it. No catalog/provider verification,
release retirement or production-documentation changes belong to this probe.
