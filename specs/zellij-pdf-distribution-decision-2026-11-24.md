# Optional PDF distribution decision — 24 November 2026

Status: concrete licensing proposal; owner approval pending. This file grants no
license and does not permit publication of the experimental compiler package.

## Verified boundary

Root `package.json` and `cli/Cargo.toml` declare Zellij `UNLICENSED`. The PDF
compiler's preserved `release/LICENSE-MANIFEST.json` identifies the selected
LuaHBTeX JS/Wasm artifacts as GPL-2.0-only with MIT and linked-component terms.
Its corresponding-source archive includes the exact engine-generation source,
TeX Live and Emscripten sources, port archives, build receipts and rebuild/relink
instructions. The separate clean Linux amd64 build recorded in
[PR #244](https://github.com/mylife-inc/Zellij/pull/244) reproduces the selected
JS and Wasm bytes exactly. The source verifier's `rebuildVerified: false` means
that verifier does not execute a compiler; it does not negate the separate
rebuild. A modified-library relink test remains unqualified. The upstream
manifest retains its own development-status wording; Zellij does not edit it or
claim upstream promotion.

The Zellij JavaScript adapter directly loads the compiler and supplies its Wasm
imports, virtual filesystem and worker/process boundary. The Rust CLI launches
that separate process and exchanges generated files and bounded protocol events.
The existing `scripts/pdf-wasm/NOTICE` grants upstream MIT permissions only; it
is not an explicit license for all newly written Zellij adapter code.

## Proposed owner decision

License **Zellij-authored files under `scripts/pdf-wasm/` only** under
GPL-2.0-only, with copyright attributed to Shebka. This includes the isolated
JavaScript runner, maintainer build/verification tools, tests and accompanying
original documentation/metadata. Retain all existing upstream notices and terms;
this proposal does not relicense third-party material. Add an explicit scoped
license and copyright notice to that directory and preserve it in the runtime
and corresponding-source packages.

Root project terms, Rust crates, website engine, content/templates and other
files remain unchanged. The public source required for this adapter and its
third-party compiler must remain accessible alongside the corresponding runtime
releases even if the remainder of Zellij later moves to a private or sealed
repository. Existing copies cannot have their granted GPL permissions withdrawn.
No licensing change is applied until the owner approves this specific scope.

## Distribution arrangement after approval

- Publish the exact matching source archive beside each optional runtime on the
  same GitHub release, with hashes and a versioned source link in the runtime's
  notices/manifest. Do not depend only on a third-party mirror or an encrypted
  source copy. Preserve required notices and source availability during release
  retirement/migration; do not delete source while its runtime is distributed.
- Retain original component notices, the complete EC and URL distributions, and
  the complete frozen package archives used by the runtime/format/font recipes.
- Retain LGPL component source and the preserved relinking instructions. Do not
  impose restrictions on debugging modifications to those components. Complete
  the modified-library relink qualification before promoting the runtime.
- Keep the larger source archive separate from the normal runtime download. It
  is not an end-user build dependency.
- Bind the review to exact resource/source recipes and explicit release checks.
  A changed input requires renewed review; a boolean alone is not evidence.

The proposed arrangement follows the source-access requirements in
[GPLv2 §3](https://www.gnu.org/licenses/old-licenses/gpl-2.0.html), the source/relink
provisions in [LGPLv2.1 §6](https://www.gnu.org/licenses/old-licenses/lgpl-2.1.html),
and preservation/distribution conditions in the
[LPPL](https://www.latex-project.org/lppl/lppl-1-3c/). It records an engineering
release plan, not a claim that an upstream project approved Zellij's release.

## Qualification already completed

PR #252 merged as `191a882`, with the tree of tested `03d0d00`. Normal CI
`45494370791` passes 37 jobs and one conditional skip; PDF CI `45494370827` passes
all eight jobs without retries. Six platforms pass 120 manuscripts / 360 passes,
272 native-matched page rasters, 78 cache scenarios and 12 cold/offline compiles.
All runtime/source archives match. Main CI `45498740733`, base image
`45498740771` and production docs `45498740731` also pass. Public rc.19 remains
unchanged and still uses container PDFs.
