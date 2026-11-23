# Compact PDF metadata and resource archive — 23 November 2026

Status: maintainer qualification. Installed/public rc.19 still uses its container
PDF backend. This change publishes no runtime resource archive and changes no
installed prerequisites.

## Verified baseline

PR #245 merged as `e7c1d36`, identical in tree to tested `aff7d36`. All 37 normal
and seven PDF jobs pass (`45139281706`, `45139281588`). Main CI `45142894032`
and base image `45142894139` also pass. The six platforms agree on the format
and source-manifest hashes; 114 book cases / 342 passes and 269 native-matched
Linux pages pass. Earlier Windows CRLF receipt disagreement remains recorded in
the [format source record](zellij-pdf-format-sources-2026-11-22.md).

## Original inputs instead of an opaque database

The old metadata consist of a 3,406,791-byte font-name database and 5,609,407-byte
Type 1 map. A separate native-generator experiment first showed that a small
database could preserve the observed book layout, but depended on a full native
TeX image. The implementation here instead runs the original `luaotfload-tool`
with the already verified LuaHBTeX Wasm compiler.

`font-inputs.json` selects 78 inputs: 34 original Lua scripts, five Unicode data
files and exactly the 39 original OTFs in the current runtime inventory.
`font-sources.lock.json` binds them to seven packages and 16 frozen archives.
The preparer verifies archive and file bytes, collects 11 notices, refuses stale
or incomplete inventories and preserves existing outputs. The selected inputs
total 8,520,553 bytes; this maintainer source collection is not the user download.

The generator has only the verified JS/Wasm compiler and these original files.
It reads no old format or generated metadata. Fonts are installed into the
virtual filesystem in stable order; unknown resources cannot be downloaded.
The same time, filesystem, memory and process/network restrictions apply.
Two fresh builds must use precisely all selected inputs without an unresolved
resource request and produce identical normalized metadata.

The generated date labels depend on the guest timezone even with a fixed epoch.
Normalization therefore replaces precisely the two date labels and 39 cache
timestamps after checking their counts. All font basenames must match the
selected inventory exactly. No font bytes, names, metrics or rendering settings
are modified. The database is **36,179 bytes**, SHA-256
`b3461a143b5c86c01a244182d820e58bf3c44301414bb024db3e6cc543677616`.
The replacement OpenType-only map is 30 bytes, SHA-256
`6fd8afa2d0d1fc0415b5d1d9fa3b43edffcb3e3489b86d8a0011d436b6d46fe0`.

## Experimental resource archive

`compact.py` verifies each selected input before staging a resource directory
and deterministic tar/gzip archive. It includes the compiler, independently
rebuilt format and compact resource set. A manifest records each name, size and
hash; blobs use flat hash filenames. Changed manifests fail before any supplied
blob name is resolved. Corrupt/truncated blobs fail before executing the engine.
Outside format/metadata replacements cannot alter a compact package.

Two local archives are byte-identical: **10,845,108 bytes**, containing 286 files
and 27,375,102 uncompressed bytes including the manifest. The archive hash is
`d13d2192e2d0216f3ee9c7a971cc19b465fea69cc7846b0a25282075c75c36f3`.
Cross-platform compression libraries may produce different gzip streams; the
manifest and uncompressed resource hashes are the portable identity.

This archive deliberately remains experimental. It does not contain the final
required notices/source arrangement, a private Node runtime or an installed CLI
entry point. It is not uploaded by CI or offered for user installation. The
existing full source and notice checks remain in place; a generated replacement
does not resolve the separate EC distribution, historical `url.sty` wording or
engine/component distribution questions by itself.

## Verification and remaining integration

Local checks include 24 Python tests, 19 Node tests including actual engine
execution, two identical metadata generations and two identical resource archives.
The package loader supplies exactly the same compiler/format/resource bytes as
the independently prepared inputs, without the original generated files.
All 19 repository-owned books compile with the compact metadata (57 passes).
All 269 pages also match the previous native-verified Linux output in normalized
text, geometry and complete raster streams. Current hosted results are recorded
in the implementation pull request; prior-font results are not substituted for
the new qualification.

CI builds the new database independently on all six native platforms, verifies
source/metadata receipt agreement, assembles the compact archive and runs the
entire manuscript matrix from its resource directory. Linux compares PDF
structure, text, geometry and every page raster against the native renderer.
Only receipts and Zellij-produced books/logs are retained as hosted artifacts.

Next work is final component/font/source packaging, production isolation review,
Rust cache/download/dispatch and cancellation integration, diagnostics/generated
CI/user guides, and installed/public acceptance. Explicit custom container
selection must keep its existing semantics. No release or provider verification
is included in this increment; historical browser timing findings stay open.
