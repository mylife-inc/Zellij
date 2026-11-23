# Cached PDF format from source — 22 November 2026

Status: maintainer qualification. Installed/public `zel` remains rc.19; its PDF
backend still uses a container. No release asset or installed prerequisite changes.

## Baseline receipt

PR #244 merged as `0e2db69`, with an identical tree to tested `8a24f45`.
Its 37 regular CI jobs and seven source/probe jobs pass (`45120456002` and
`45120455991`). Main CI `45124076067` and base image `45124075839` also pass.
The separate clean build matches the selected JS adapter and Wasm compiler
byte-for-byte; the exact source/toolchain hashes and bounded format experiment
are retained in [PR #244](https://github.com/mylife-inc/Zellij/pull/244).
The source verifier's `rebuildVerified: false` remains intentional: that program
checks source correspondence and does not itself run a compiler.

## Repeatable INITEX build

The previous two-book experiment used source inputs but produced varying format
bytes. Setting only TeX's `SOURCE_DATE_EPOCH` did not fix the difference. Inspection
of the verified Lua source (`libs/lua53/lua53-src/src/lstate.c`) shows its hash seed
also uses C `time(NULL)`. The format worker now fixes its guest `Date` clock and
TeX date to 2026-08-26 UTC. Normal PDF workers retain their existing clock behavior.
The worker supervisor's timeout uses real time; a deliberately endless INITEX
regression verifies termination with the guest clock frozen.

The format is built from 75 unmodified original files (5,314,191 source bytes),
across 15 packages and 34 checksum-pinned archives. Archive downloads remain a
maintainer prerequisite; offline source preparation and offline generation are
separate operations. The generator reads only the selected compiler JS/Wasm and
prepared source files. No precompiled format or generated font metadata is present
in the minimal-engine regression directory.

Two independent builds must match each other, the selected input-usage list and
the reviewed output: **6,744,585 bytes**, SHA-256
`d94612e32609313f591f7e39d7cae4a94e9cd2a284781db279befcec14a5adfd`.
The output is `zellij-lualatex.fmt`; the upstream runtime lock is unchanged.
This is a new independently generated format, not a claim to reproduce the old
upstream format's bytes.

The source preparer reuses the original-archive checks, validates the format lock
against pinned metadata, rejects modified/missing source bytes and notices, and
publishes only a complete staged output. Existing outputs and corrupt source
caches are preserved. Eighteen Python source tests cover these paths. Three new
real-engine tests cover independent format generation, checksum/output failure
and the frozen-clock deadline alongside the ten existing runtime guard tests.

## Supported print-size coverage

Extending the probe to the other supported font sizes exposed missing `bk10.clo`
and `bk12.clo`, followed by missing optical-size Latin Modern font files. The
10-point and 12-point books both failed before adding those resources. The locked
inventory now includes the two original class-option files and all Roman/Sans/Mono
optical variants needed by those named families (15 additional original OTFs).
The existing math font stays present. Initializing legacy math sizes also needed
additional Computer Modern metrics; the full 75-file metrics set adds only
87,752 bytes beyond the five already present. All added files come from the same
verified LaTeX, Latin Modern and Computer Modern packages; there is no new source
package or notice policy.
The source verifier now matches 280 package-backed files across the same 58
packages. The prototype bundle is 287 files / 33,610,991 uncompressed bytes.

Two additional manuscript cases cover 10-point/A4 and 12-point/Letter alongside
the existing 11-point/A5 coverage. These use the normal native/PDF comparison
checks; no warning or missing-resource assertion is weakened. Published rc.19
uses the complete container distribution and is not changed by this prototype
resource correction.

## Source and notice boundary

The 21 collected records include original headers as well as license notices.
`null.tex` has only an explanatory comment; it must not be reported as a license
text. Its frozen package metadata and [CTAN package record](https://ctan.org/pkg/knuth-lib)
identify the [Knuth terms](https://ctan.org/license/knuth), and the source remains
unchanged. Final notice/redistribution review is separate from input verification.

The two original generated font-metadata resources total 9,016,198 uncompressed
bytes. They are not used to generate this format, but book compilation still uses
them; their source closure or a fully qualified replacement remains next work.
This increment does not change those bytes or claim their review is complete.

## Acceptance and remaining work

The six-platform workflow independently rebuilds the pinned format, runs the
boundary tests and uses the new format for all 19 repository-owned manuscripts.
Linux's optional native comparison checks PDF structure, text, page geometry and
every page raster. Current hosted results belong to the implementation PR; the
successful prior-format results above are not substituted for this qualification.
Only JSON receipts and generated Zellij books/logs are uploaded, not format/source
or compiler binaries.

Generated font metadata, final component/font/source distribution, production
isolation qualification, archive/cache integration and public-package acceptance
remain open. The historical browser timing findings remain unresolved; no browser
runtime or assertion change is included here.
