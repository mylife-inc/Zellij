# Offline LuaHBTeX feasibility probe

This is a maintainer experiment for the optional cached PDF toolchain. It is
**not connected to `zel pdf`**, not a release artifact, and not a claim that the
production isolation contract is complete. Released rc.19 still uses its pinned
container for PDF. Ordinary rich web builds already use the cached renderer.

The probe downloads 200 hash-pinned resources (31,989,582 bytes before packaging)
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
German and Spanish. It writes PDFs, typesetter logs and experimental receipts
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

The eight tests exercise hash corruption, input links, host-file/environment/shell
denial, unavailable Lua socket modules, oversized virtual writes, console and
guest-heap limits, endless-guest
termination/recovery, nonzero TeX status and output preservation. Socket modules
are unavailable in this format; this test does not claim to have made a socket
connection and then observed network filtering.

The console regression lowers the budget to 32 KiB for a quick real-engine
test; the normal cap remains 8 MiB. Budget violations notify the supervisor
before throwing, because C stdio can catch the stream error and keep running.
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
