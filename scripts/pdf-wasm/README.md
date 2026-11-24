# Offline LuaHBTeX feasibility probe

This is a maintainer experiment for the optional cached PDF toolchain. It is
**not connected to `zel pdf`**, not a release artifact, and not a claim that the
production isolation contract is complete. Released rc.19 still uses its pinned
container for PDF. Ordinary rich web builds already use the cached renderer.

The probe prepares 287 hash-pinned resources from the upstream engine release,
an immutable TeX mirror and independently pinned original archives. It then compiles
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
python scripts/pdf-wasm/originals.py SOURCE_CACHE test-results/pdf-originals
node scripts/pdf-wasm/download.mjs "$ZEL_PDF_PROBE_BUNDLE" test-results/pdf-originals
node --test scripts/pdf-wasm/probe.test.mjs
node scripts/pdf-wasm/check.mjs
```

The maintainer downloader retries only transient connection failures, deadlines
and selected temporary HTTP statuses, at most three attempts with one-/two-second
backoff. Each attempt starts with an empty buffer. Permanent HTTP/TLS failures,
oversized responses and checksum mismatches fail immediately; previous output is
preserved. Nine loopback/failure regressions run before downloads in CI. This
does not add network access to the compiler or change installed CLI downloads.

`check.mjs` uses only repository-owned manuscripts: all three book editions,
equations, statements, bibliography, glossary/index, Arabic RTL, and French,
German and Spanish, URL/query/path layout, plus a 100-chapter book across two parts. It writes PDFs,
typesetter logs and experimental receipts
under `test-results/pdf-wasm`. It refuses to overwrite outputs. Remove or move
only those generated test outputs before intentionally repeating the check.
The default matrix now runs each manuscript in a separate Node process with
three disposable worker passes. Receipts identify that execution mode. Explicit
legacy format/metadata substitutions retain the in-process comparison path;
those runs do not qualify the process boundary.

The link-layout case includes long plain, bold and code-styled URL labels. The
print adapter permits wrapping at URL punctuation while preserving hyperlink
destinations and descriptive labels. This fixes an overflow reproduced with
both the earlier URL 3.4 resource and the selected unchanged original 3.3;
strict overflow validation remains enabled.

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

## Versioned package and cache qualification

`toolchain.py` assembles the runner scripts, compact resources and original
notices into a single versioned optional archive. It verifies the complete
corresponding-source archive and refuses stale build recipes before binding its
size, hash and source-manifest hash into the runtime manifest. The source archive
and private Node are separate; neither is duplicated inside the runtime download.
The package remains explicitly experimental, with redistribution review pending.

```sh
python scripts/pdf-wasm/distribution.py SOURCE_CACHE test-results/pdf-toolchain-distribution
python scripts/pdf-wasm/toolchain.py test-results/pdf-compact/package test-results/pdf-toolchain-distribution test-results/pdf-toolchain --version 0.2.0-rc.19
cargo test --manifest-path cli/Cargo.toml --locked -p zellij-cli --example pdf_cache
cargo build --manifest-path cli/Cargo.toml --locked -p zellij-cli --example pdf_cache
node --test scripts/pdf-wasm/cache.test.mjs
```

The Rust `pdf_cache` example downloads from an explicit maintainer endpoint;
there is no public PDF package URL or installed CLI command. It uses the shared
cancellable transfer, bounds archive/extraction sizes, rejects unsafe paths,
links/junctions, case aliases, unlisted/missing files and hash/version mismatches,
and verifies every cached file before reuse. Failed acquisition removes only its
own staging directory. A damaged existing cache is reported and preserved.

Qualification entries live under `$ZELLIJ_HOME/cache/pdf-qualification/VERSION`
(default `~/Dev/config/zellij/cache/pdf-qualification/VERSION`), separate from
future released toolchains. Offline misses make no request and create no cache;
valid offline entries run the packaged scripts through the Rust supervisor.
The harness passes an absolute Node 24 executable; production CLI integration
must reuse the renderer's verified private Node. A checksum supplied by a chosen
endpoint does not independently authenticate that endpoint, and these checks do
not prevent arbitrary concurrent hostile filesystem replacement.

All six runners build the package locally and exercise cold/offline compilation
with an empty system PATH, Unicode/spaced paths, corruption and link/junction
rejection, and cancellation during stalled transfers. Only receipts are uploaded.
An aggregate job compares all six runtime/source identities with the independently
built source archive. Source text uses LF on every checkout, and archive members
are sorted as portable strings rather than platform-specific Path objects.
The installed `zel pdf`, build integration and published prerequisites remain
unchanged until distribution review, dispatch and public-package acceptance finish.

## Process supervision contract

`process.mjs` supplies a runner entry point for the future Rust cache integration
and a maintainer acceptance adapter, `compileProcess`. Installed `zel` does not
invoke it yet. After preparing the compact package, run:

```sh
ZEL_PDF_COMPACT_PACKAGE="$PWD/test-results/pdf-compact/package" node --test scripts/pdf-wasm/process.test.mjs
```

The adapter starts the absolute Node executable in a private staging directory,
with no shell, system PATH, user-home configuration, credentials or Node startup
hooks. It explicitly disables `NODE_V8_COVERAGE`: Node otherwise propagates that
setting even into a supplied environment allowlist. A regression sets a preload
hook and coverage destination and checks neither is used. Each compiler worker
also has an empty environment and no inherited execution flags.

The internal invocation is `--run BUNDLE INPUT NEW_OUTPUT TOTAL_MS PASS_MS`.
Launch Node with `--max-old-space-size=128`, matching the worker's old-generation
budget; Node's process-level V8 flag overrides the worker option.
The caller must keep stdin open without sending data. EOF, unexpected input or
a pipe error aborts the worker; SIGINT/SIGTERM do the same where the OS delivers
those signals cooperatively. Stdout contains bounded JSON-line events (`ready`,
three ordered `pass` events, `complete`), never guest console output. The total
deadline is at most 600 seconds, and each pass at most 120 seconds. A caller must
also enforce its own deadline and kill/reap fallback. The acceptance adapter
allows one second for cooperative cancellation before forcing termination, and
checks process exit separately from completion events.

The caller owns the outer staging directory. It waits for the child to close
before removing that directory, checks the PDF/receipt, and publishes only after
checking cancellation and destination ownership again. Forced child termination
therefore leaves no published partial PDF. Forced termination of the caller
itself can leave its private staging directory, which a later CLI recovery policy
must handle; the child observes the closed parent pipe and stops. This is not a
claim of atomic ownership against arbitrary concurrent filesystem mutation.

Twelve regressions cover success without system tools or startup hooks;
cancellation at each pass and after completion; invalid/deadline inputs; endless
input and recovery; abrupt child death; unresponsive-child termination; parent
EOF; corrupt resources and invalid TeX; existing/late destinations; and observer
failure. The forced-suspension regression uses Unix SIGSTOP and is explicitly
skipped on Windows; the other eleven run everywhere. All six manuscript jobs
use the separate-process path and retain the native Linux PDF comparison.

Process/worker heap flags and existing guest budgets do not establish a hard
total RSS limit. This does not turn `node:vm` or Node's permission model into a
security sandbox. Final isolation review, Rust ownership/download/cache
integration, distribution review and installed-package acceptance remain before
the cached PDF backend can be released.

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
compiler match the pinned bytes exactly. The original mirror-generated metadata remain the legacy comparison inputs;
the compact replacement below has its own complete selected input inventory. `source-policy.json` records remaining
review items and records resolved findings. The selected original URL 3.3 has
unchanged source/documentation and an unversioned LPPL notice; it replaces the
later 3.4 resource rather than interpreting that release's historical wording.
The additional ConTeXt notice is collected
from the exact frozen documentation archive, not inferred from a package label.
The policy now also records the completed engine/format rebuild and font-source
work with its evidence; those historical findings are not silently discarded.

For an intentional runtime/resource update, download the newly reviewed metadata,
update `source-policy.json`, and generate a **new** proposed lock for review:

```sh
python3 scripts/pdf-wasm/sources.py inventory /tmp/zellij-pdf-source-cache /tmp/proposed-pdf-sources.lock.json
```

Never copy a new inventory over the reviewed lock merely to make a failed
correspondence check pass. Inspect changed source, notices, resource coverage and
engine receipts first. This verifier does not package or publish a runtime.

## Package original notices and matching sources

The separate `distribution.py` maintainer tool creates private qualification
archives. It does not publish them, alter licenses, add a CLI dependency or mark
the release review complete:

```sh
python3 scripts/pdf-wasm/sources.py fetch /tmp/zellij-pdf-source-cache
python3 scripts/pdf-wasm/format_sources.py fetch /tmp/zellij-pdf-source-cache
python3 scripts/pdf-wasm/font_sources.py fetch /tmp/zellij-pdf-source-cache
python3 scripts/pdf-wasm/distribution.py /tmp/zellij-pdf-source-cache test-results/pdf-source-distribution
```

The packager regenerates and checks all three input inventories, combines their
65 original packages without losing notices, and rejects conflicting package,
archive or resource records. It verifies 351 distinct original resource files.
The notices package retains 115 original notice records and every one of the
121 files listed in EC's original `00files.txt`, with original names and bytes.
Missing, duplicate, nonregular or incorrectly sized EC files fail qualification.
It also retains all four original URL 3.3 component files, with source/archive
receipts. The runtime loader receives the unchanged original `url.sty` bytes.

The notices archive is under **1 MB**. The separate matching-source archive is
about **430 MB**, including the original compressed source packages, engine
corresponding source, provenance metadata, exact build programs/recipes and
notices. It is not part of an ordinary rendering download. The prospective
runtime package must incorporate the notices and provide the actual matching
source download when it is released; the resource-only archive does neither.

Both archives use fixed ordering, timestamps and ownership. The packager reads
each completed archive back to verify all members and hashes. It stages output
before publication and preserves pre-existing outputs and corrupt inputs.
`SOURCE-MANIFEST.json` inventories the source package; `NOTICES.json` maps the
original notices and records review items. CI replays source verification and
format/font preparation using only the packaged scripts and upstream files.
Only `verification.json` is uploaded, never either archive or the copied sources.

The earlier 3.4 notice issue remains documented in the historical audit records.
Its code is absent from the new runtime/source package selection. The package
preparation does not change any original terms or complete the final component
redistribution review. The archive's own READMEs explain its contents, offline
verification and original compiler rebuild instructions.

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
then uses it for all 20 manuscripts; Linux compares the books with the native
renderer. Only JSON receipts and Zellij-produced PDFs/logs are uploaded, never the
format binary, input sources or compiler. This does not ship a cached PDF backend.

For an intentional format-input update, create and review a new inventory using
`python3 scripts/pdf-wasm/format_sources.py inventory CACHE NEW_LOCK_FILE`.
Review changed input bytes, terms and output before updating the expected hash;
a failed reproducibility check must not be resolved by merely repinning it.


## Build compact font metadata and the experimental resource archive

The font database now has an independent source recipe: 78 verified original
files (34 Lua scripts, five Unicode data files and 39 OpenType fonts), seven
packages and 16 frozen run/doc/source archives. The preparer also retains 11
original source notices. Generation uses the same verified Wasm engine, a fresh
virtual filesystem, no precompiled format, no old font database and no network
resolver. Docker, a native TeX installation and host fonts are not involved.
Python/curl below are maintainer preparation/packaging tools, not installed CLI
prerequisites.

```sh
python3 scripts/pdf-wasm/font_sources.py fetch /tmp/zellij-pdf-source-cache
python3 scripts/pdf-wasm/font_sources.py prepare /tmp/zellij-pdf-source-cache test-results/pdf-font-inputs
export ZEL_PDF_FONT_INPUTS="$PWD/test-results/pdf-font-inputs"
node scripts/pdf-wasm/fonts.mjs "$ZEL_PDF_PROBE_BUNDLE" "$ZEL_PDF_FONT_INPUTS" test-results/pdf-font-metadata
node --test scripts/pdf-wasm/fonts.test.mjs
python3 scripts/pdf-wasm/compact.py "$ZEL_PDF_PROBE_BUNDLE" test-results/pdf-format test-results/pdf-font-metadata test-results/pdf-compact
export ZEL_PDF_COMPACT_PACKAGE="$PWD/test-results/pdf-compact/package"
export ZEL_PDF_FONT_METADATA="$PWD/test-results/pdf-font-metadata"
node --test scripts/pdf-wasm/compact.test.mjs
```

These steps follow the format build above. Existing outputs are preserved.
The font generator runs twice, requires exactly the selected fonts and input
usage, normalizes only the two date labels and 39 cache timestamps, and checks
the resulting 36,179 bytes against the reviewed SHA-256. Font bytes and names
remain unchanged. The 30-byte map declares OpenType use; it contains no bitmap
or Type 1 font mappings. The two original metadata files total 9,016,198 bytes.

The resource archive uses fixed names/order, modes and timestamps. Its manifest
binds every compiler/format/resource blob to its size and SHA-256. The loader
matches the complete manifest against the reviewed inventory before resolving
any blob, then checks every used byte. It reads neither the old precompiled
format nor either upstream metadata file. The local archive is 10,845,108 bytes;
compression-library versions can affect archive bytes, while the uncompressed
manifest and blob identities must agree. This is **not a release package**: final
notices, source-distribution arrangements and the installed runtime/CLI entry
point remain separate work. No private Node runtime is included here.

To test the assembled resource directory, start with a new `test-results/pdf-wasm`
output (move an existing generated result if it needs preservation):

```sh
unset ZEL_PDF_REBUILT_FORMAT ZEL_PDF_FONT_METADATA
ZEL_PDF_PROBE_BUNDLE="$ZEL_PDF_COMPACT_PACKAGE" node scripts/pdf-wasm/check.mjs
python3 -m unittest discover -s scripts/pdf-wasm -p 'test_*.py' -v
```

Outside replacements are rejected for a compact package. CI independently
prepares and generates metadata on every platform, assembles the resource
archive, and typesets all 20 books from that compact inventory. Linux compares
all resulting pages with the native backend. Four metadata regressions cover
independent builds, input/output integrity, incomplete/oversized databases and
termination; two package-loader regressions cover byte equivalence, manifest
changes and corrupt/truncated blobs. The combined Python suite has 24 cases.
Only JSON receipts and Zellij-generated PDFs/logs are uploaded; neither the
experimental archive nor its compiler/data/source files are published.

For an intentional source-selection change, propose a new inventory with
`python3 scripts/pdf-wasm/font_sources.py inventory CACHE NEW_LOCK_FILE`, review
original source/notices and font coverage, and then qualify the new output.
Do not repin a hash merely to hide a failed reproducibility or rendering check.

## Qualify the Rust supervisor

The installed `zel pdf` still uses the container backend. The next cache path
uses `zellij_engine::pdf_runner`, with the shared `ProtocolChild` owner. It
launches one private Node process directly, retains the empty stdin liveness
pipe, and checks bounded protocol/artifacts before publishing a new output.
Quiet/no-color settings cannot replace or expose its protocol captures.

From the repository root, after preparing `test-results/pdf-compact/package`:

```sh
cargo build --locked --manifest-path cli/Cargo.toml -p zellij-engine --example pdf_process
node --test scripts/pdf-wasm/rust-process.test.mjs
ZEL_PDF_PROBE_BUNDLE="$PWD/test-results/pdf-compact/package" ZEL_PDF_RUST_SUPERVISOR=1 node scripts/pdf-wasm/check.mjs
```

The existing manuscript exporter and `zel` debug binary must also be built for
`check.mjs`, as described above. Windows uses the equivalent environment
variables and the automatically selected `.exe` harness. The workflow runs the
same Rust owner on all six native targets, retaining native Linux comparisons.
Only the maintainer harness reads `ZEL_PDF_TEST_*`; these are not CLI options or
installed backend overrides. Its source is in
`cli/crates/zellij-engine/examples/pdf_process.rs` in the Zellij repository.

The Rust caller must verify the executable, runner and resources before calling
this API, preserve authored-output/revision checks, and own its working stage.
A killed caller may leave that stage even though stdin EOF stops the compiler.
Capture budgets are checked while polling, not OS disk quotas; the heap limits
are not a hard total RSS cap. No matching-source offer or redistribution
permission is created by these tests. See
[the Rust supervision record](../../specs/zellij-pdf-rust-supervision-2026-11-23.md).
