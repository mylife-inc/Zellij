# Offline LuaHBTeX feasibility probe

This is a maintainer experiment for the optional cached PDF toolchain. It is
**not connected to `zel pdf`**, not a release artifact, and not a claim that the
production isolation contract is complete. Released rc.19 still uses its pinned
container for PDF. Ordinary rich web builds already use the cached renderer.

The probe downloads 287 hash-pinned resources (33,610,991 bytes before packaging)
from the upstream engine release and an immutable TeX mirror. It then compiles
generated Zellij print inputs without a network resolver, container, system TeX,
curl, Python or npm packages. The maintainer invokes Node 24 explicitly here; a
production integration would reuse the private Node already in the renderer
cache rather than ask users to install Node or download a second copy.

## Run

From the repository root, choose new directories for each output:

```sh
cargo build --manifest-path cli/Cargo.toml --locked -p zellij-cli
cargo build --manifest-path cli/Cargo.toml --locked -p zellij-docs --example pdf_probe_inputs
export ZEL_PDF_PROBE_BUNDLE="$PWD/test-results/pdf-wasm-bundle"
node scripts/pdf-wasm/download.mjs "$ZEL_PDF_PROBE_BUNDLE"
node --test scripts/pdf-wasm/probe.test.mjs
node scripts/pdf-wasm/check.mjs
```

`check.mjs` uses only repository-owned manuscripts: all three book editions,
equations, statements, bibliography, glossary/index, Arabic RTL, and French,
German and Spanish, plus a 100-chapter book across two parts. It writes PDFs,
typesetter logs and experimental receipts
under `test-results/pdf-wasm`. It refuses to overwrite outputs. Remove or move
only those generated test outputs before intentionally repeating the check.

For one generated input folder:

```sh
cli/target/debug/examples/pdf_probe_inputs scripts/fixtures/pdf-book test-results/pdf-input
node scripts/pdf-wasm/probe.mjs "$ZEL_PDF_PROBE_BUNDLE" test-results/pdf-input test-results/pdf-probe-output
```

On Windows, append `.exe` to the Rust executable and set the environment variable
using the current shell. The six-platform workflow runs the same scripts.

## Boundary

The locked engine is LuaHBTeX 1.24.0 / TeX Live 2026, compiled by
[WasmTex](https://github.com/corca-ai/wasmtex/tree/1334ff2f6df172fbbb2dfee11dfafebc8b921817).
Only its verified core JS/Wasm, matching format and pinned data files are used.
The upstream Node worker, runtime HTTP resolver and SDK are not executed.
The JS adapter's resource naming/configuration follows the upstream controller;
see [its attribution](NOTICE).

Each pass gets a fresh disposable worker and virtual filesystem. The Wasm guest
has no host filesystem mapping, environment or network bridge. Shell/socket/DNS
imports are denied explicitly. Virtual file growth, Wasm heap, JS heap, console
logs and pass duration have separate bounds. Unknown TeX resources are treated
as absent, never downloaded at compile time. Actual engine exit status matters;
an existing partial PDF is not accepted as success. Three passes and final
typesetting/PDF validation must finish before publishing a new proof output.

`node:vm` is used to keep the trusted Emscripten glue in browser mode; it is **not
the security boundary for arbitrary JavaScript**. The design depends on the
locked Wasm engine and its audited import functions. No arbitrary JS is accepted.
Further resource/isolation qualification, crash handling, process-level
cancellation integration and independent review remain necessary for production.

The ten tests exercise hash corruption, input links, host-file/environment/shell
denial, unavailable Lua socket modules, oversized and initial virtual writes, console and
guest-heap limits, endless-guest
termination/recovery, nonzero TeX status and output preservation. Socket modules
are unavailable in this format; this test does not claim to have made a socket
connection and then observed network filtering.

The console regressions lower the budget to 32 KiB for quick real-engine
test; the normal cap remains 8 MiB. Budget violations notify the supervisor
before throwing, because C stdio can catch the stream error and keep running.
Both initial copies and later growth count toward storage limits. Console bytes
are counted before buffering a line, including a line with no newline.
Set `ZEL_PDF_PROBE_BASELINE=1` when Docker and Poppler/qpdf are available to compare
every case with the current container backend. That optional maintainer mode
checks PDF structure, page sizes/counts, NFKC-normalized extracted text/layout
and every page rasterized at 1300 pixels. It is enabled on the Linux x64 probe
runner; other runners test offline generation and isolation without containers.

## Before a released toolchain

- Complete component/font/package notices and license provenance. The engine
  manifest identifies GPL-2.0-only plus linked-component terms. The compiled
  format and TeX files have separate terms. `lock.json` records the exact
  corresponding-source URL/hash, but that source and the complete notices must
  be verified and provided with any Zellij binary redistribution. A maintainer
  download is not a completed redistribution review.
- Establish that the resource inventory covers every supported print setting,
  not just the manuscripts observed so far. Complete image, font, links and
  cross-reference inspection and native/Wasm comparisons.
- Qualify the resource limits, filesystem/process/network isolation and
  cancellation on all supported platforms. Preserve strict failure semantics.
- Add optional version-matched archive packaging, verified extraction, cache
  integrity/recovery and offline behavior under `ZELLIJ_HOME`; connect shared PDF
  dispatch, diagnostics, generated CI and documentation.
- Run installed/public-package acceptance before changing the documented
  prerequisites or cutting the candidate that includes this backend.

The probe workflow uploads Zellij-produced PDFs/logs only. It does not upload or
redistribute the experimental engine/data bundle.

## Source correspondence and notices

Before preparing any distributable toolchain, maintainers can reproduce the
source checks with Python 3.13, its standard library and the standard `curl`
HTTPS client (maintainer/CI tools only):

```sh
python3 -m unittest discover -s scripts/pdf-wasm -p test_sources.py -v
python3 scripts/pdf-wasm/sources.py fetch /tmp/zellij-pdf-source-cache
python3 scripts/pdf-wasm/sources.py verify /tmp/zellij-pdf-source-cache test-results/pdf-source-verification
```

This Python program is **maintainer tooling only**; the proposed installed PDF
backend still uses the private Node runtime and WebAssembly. No Python program,
compiler or package installer runs inside `zel`.

The fetch is deliberately separate from verification. Verification works offline,
regenerates the checked-in inventory from pinned metadata, verifies full archive
checksums and compares all 280 package-backed runtime files with the original
frozen TeX Live packages. It reads explicit regular tar members without extracting
paths or following links. Missing, modified or duplicated sources/notices fail the
check. An existing verification output or corrupt cached input is preserved.

`sources.lock.json` pins all 152 run/documentation/source archives for the 58
runtime packages, one supporting ConTeXt documentation archive, the engine's
corresponding-source archive and the three metadata inputs. This large maintainer
cache is **not** the proposed end-user download. The verifier checks that the
source release manifest matches the exact engine JS, Wasm and format bytes,
checks the build receipt, preserves both engine and format source revisions, and
collects package and engine notices into `COLLECTED-NOTICES.txt`. CI uploads only
the verification JSON, not third-party binaries, source archives or notices.

This is evidence of source correspondence, not a release approval or a clean
rebuild claim. A separate clean engine rebuild is recorded in
[PR #244](https://github.com/mylife-inc/Zellij/pull/244): the adapter and Wasm
compiler match the pinned bytes exactly. The two generated font-metadata files
still require source-input closure. `source-policy.json` records remaining
review items, including EC's full-distribution requirement and the historical
license-version wording in `url.sty`. The additional ConTeXt notice is collected
from the exact frozen documentation archive, not inferred from a package label.

For an intentional runtime/resource update, download the newly reviewed metadata,
update `source-policy.json`, and generate a **new** proposed lock for review:

```sh
python3 scripts/pdf-wasm/sources.py inventory /tmp/zellij-pdf-source-cache /tmp/proposed-pdf-sources.lock.json
```

Never copy a new inventory over the reviewed lock merely to make a failed
correspondence check pass. Inspect changed source, notices, resource coverage and
engine receipts first. This verifier does not package or publish a runtime.

## Rebuild the format from original sources

`format-inputs.json` selects 75 original files. `format-sources.lock.json` binds
them to 15 packages and 34 frozen run/doc/source archives. The source preparer
checks original bytes and collects 21 source headers/notices, including the
unmodified `null.tex` header. That header contains no license text; the retained
Knuth catalogue reference is not described as an embedded license notice.

```sh
python3 -m unittest discover -s scripts/pdf-wasm -p 'test_*sources.py' -v
python3 scripts/pdf-wasm/format_sources.py fetch /tmp/zellij-pdf-source-cache
python3 scripts/pdf-wasm/format_sources.py prepare /tmp/zellij-pdf-source-cache test-results/pdf-format-inputs
export ZEL_PDF_FORMAT_INPUTS="$PWD/test-results/pdf-format-inputs"
node scripts/pdf-wasm/format.mjs "$ZEL_PDF_PROBE_BUNDLE" "$ZEL_PDF_FORMAT_INPUTS" test-results/pdf-format
node --test scripts/pdf-wasm/format.test.mjs
export ZEL_PDF_REBUILT_FORMAT="$PWD/test-results/pdf-format/zellij-lualatex.fmt"
node scripts/pdf-wasm/check.mjs
```

Use new output directories; the tools preserve an existing output. Generation
uses only the verified JS/Wasm compiler and original files prepared from frozen
archives. It reads neither the old precompiled format nor the two mirror-generated
font-metadata files. It runs INITEX twice with a fixed UTC guest clock and source
date, checks both outputs against each other and the reviewed output hash, and
requires the exact observed input inventory. The supervisor deadline continues
using real time. Format construction is maintainer work, not an end-user step.

The compiled format is 6,744,585 bytes before outer archive compression. Its bytes
are intentionally pinned independently of the upstream precompiled format. The
six-platform workflow builds it independently, checks failure/recovery boundaries,
then uses it for all 19 manuscripts; Linux compares the books with the native
renderer. Only JSON receipts and Zellij-produced PDFs/logs are uploaded, never the
format binary, input sources or compiler. This does not ship a cached PDF backend.

For an intentional format-input update, create and review a new inventory using
`python3 scripts/pdf-wasm/format_sources.py inventory CACHE NEW_LOCK_FILE`.
Review changed input bytes, terms and output before updating the expected hash;
a failed reproducibility check must not be resolved by merely repinning it.
