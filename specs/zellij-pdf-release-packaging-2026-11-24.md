# Reviewed optional PDF packages — 24 November 2026

This increment prepares a real reviewed runtime/source pair and tests that exact
compiler archive through installed development CLI commands. No public compiler
archive or new CLI candidate is published by this work. The default remains
container; released rc.19 is unchanged.

## Component and source boundary

The owner-approved PDF-only GPL-2.0-only grant remains scoped to
`scripts/pdf-wasm/`. The original runtime, 65 frozen source packages and selected
linked components are bound in `distribution-review.json`. `review.py` compares
observed inputs to that explicit record; generating an inventory never grants
approval. Changes to compiler, notices, license, source selection or packaging
inputs invalidate the review.

The original component registry references a link inventory absent from its
source archive. The review preserves that registry and instead maps the 28
archives observed in Zellij's qualified relink to exactly one of 14 components.
Unknown or ambiguous archives fail. Original source notices are included, along
with complete EC and URL distributions. The review also adds the actual Spanish
MIT/X11 pattern/loader notices and English hyphen.tex notice; it preserves the
older Spanish LPPL documentation without treating it as the pattern-data terms.
Renderer resource bytes and selected compiler bytes are unchanged.

`distribution-review.md` describes the selected terms and source arrangement.
Recipients receive the matching source archive beside the runtime with equivalent
access, not an encrypted substitute. The pair and its original notices must stay
available while either release repository offers the runtime, including sealing
or repository migrations. This review is not an upstream endorsement.

## Modified-library relink evidence

The clean build in PR #244 already reproduced the original compiler. The separate
experiment recorded with PR #253 reuses qualified native code-generation tools
and rebuilds the entire Wasm target phase in an isolated, network-disabled
container. Original library inputs match the archived source except three explicit
API marker changes in kpathsea, Graphite2 and zziplib. A C export calls all three.

The target build exits zero without OOM. The modified JS/Wasm hashes are
`da1a9f323364880060401ca8ed829700c254aac07c91483263bd045765479c32` and
`252ba3840b36a288bc918abbc140028d41a12955347e90ef6cb9a7ccd7c02e3d`.
The import map remains identical, and all three passes return marker bitmask 7.
The 20,670-byte result has three pages with its contents and references resolved.
The first fixture omitted OpenType math and requested unavailable bitmap cmr10;
the original compiler reproduces that failure. Adding the intended unicode-math
setup resolves the fixture without changing the runtime or weakening checks.

The source package retains `relink-proof.patch`, `relink-link-inputs.txt`,
`relink-evidence.json` and `relink.mjs`. The portable verifier independently
repeats all three successful passes with the exact modified compiler. Modified
test binaries are not shipped or substituted for production pins. These records
distinguish the reused native phase from the rebuilt target phase.

## Packaging and acceptance

`release.py` assembles new outputs only, verifies the source correspondence and
exact review, binds the source name/size/hash in the runtime, and emits four
assets: runtime, matching source and both checksums. Existing or late-created
destinations are preserved; a missing, changed or mismatched asset prevents output.
The optional runtime excludes the large source archive and reuses the renderer's
private Node. Ordinary CLI use needs no new system tool installation.

Installed acceptance serves the unchanged reviewed compiler archive through a
loopback server with an empty system PATH. It covers compatibility checks, cache
misses, cold compilation, offline exports/builds, output preservation and watched
offline previews. The renderer shell remains a fixture; this does not claim a
public download or complete installed-release acceptance.

Six platform jobs build qualification and reviewed profiles, and the source job
independently rebuilds/replays the reviewed source archive. The parity job requires
identical archive and manifest identities across every platform and the independent
source build. Ordinary PRs upload only receipts and generated PDF/log evidence.
A release caller may retain the paired assets, but publication requires all eight
PDF jobs as well as the existing native, documentation and manager gates.

## Verification and remaining release work

Local verification passes 52 Python source/package regressions, workflow lint,
JavaScript syntax and documentation validation (59 routes, 33 content examples).
The actual reviewed archive passes all four installed-command groups on Intel
macOS with Node 24: cold acquisition/compilation, offline aliases/builds, corrupted
cache preservation and watched offline rebuilding. The packaged runtime sources,
format/font inputs and complete review inventory replay offline from the source
archive alone. A concurrent documentation edit during the first local package
attempt correctly triggers the stale-source guard and leaves no release output;
the final package is built after the source files are frozen.

At private acceptance version `0.2.0-rc.19`, the final runtime is 11,792,847 bytes,
SHA-256 `2577a32d3516443aa8b81301ba2b625f58df07d97f1bfd416386ba5f214073af`.
The separate source is 430,327,686 bytes,
SHA-256 `16431be391237c5190bf97990a87de7dd686a9d0285efc027d066d8c1658d2d3`.
These are private acceptance files, not replacements for the published rc.19.
Hosted cross-platform verification is tracked on the implementing pull request.

Public-download acceptance, the default-backend transition, a new CLI
candidate and matching released-user documentation remain. Earlier timing findings
are retained separately; no zero-bug claim follows from this package review.
