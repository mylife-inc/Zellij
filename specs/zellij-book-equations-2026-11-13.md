# Shared book equations — 2026-11-13

Source baseline: `d7a8416` (PR #166). Public rc.6 remains unchanged.
PR #167 merges this increment as `fe0f82c`, with the same tree as reviewed
head `168c903`. Final-source acceptance passes at `09fd2cc` in CI `43557146014`;
all 13 development-image jobs pass in `43560554366` at the merge.

## Authoring contract

In a portable book with `markdown.math: true`, a display expression containing
one top-level `\label{eq:energy}` becomes a numbered equation. Only labelled
expressions consume numbers. Each chapter/appendix restarts at 1 and prefixes
its manifest number (`2.1`, `A.1`). Front matter uses `F1.1`, `F2.1`, etc.; an
authored title page uses `T.1`. One `aligned`/`gathered` expression has one label
outside that environment and one number. Numbering follows the book reading
order and authored expression order, not sorted filenames.

Standalone inline `$\eqref{eq:energy}$` and `$\ref{eq:energy}$` become ordinary
links with parenthesized/plain numbers. Forward, backward, cross-chapter and
footnote references resolve before rendering. Custom-text Markdown links can
target the reserved `_zel-eq-eq:energy` fragment. Labels remain stable through
renumbering; renaming a label or file changes its target. Labels are unique,
case-sensitive ASCII identifiers of at most 128 bytes, starting with a letter
and then using letters, digits, colon, dot, underscore or hyphen.

Headings/links cannot contain equation references or labelled math. Reference
commands must occupy their complete inline expression. Explicit tags, number
suppression and separate numbering environments are rejected for books with
guidance to use the shared syntax. Non-book expression-local math retains its
existing behavior. Arbitrary LaTeX execution is not introduced.

## Shared model and adapters

The native parser resolves a single book-wide registry after reading-order
resolution and before link validation or output creation. The publication's
schema version 1 gains an optional `equations` array, omitted when empty.
Each equation includes its label in `id`, number, reading-entry ID, source,
anchor and encoded root-relative href. Internal source indexes are not exported.
The existing manifest/manuscript/asset revision also changes when labels,
equations or reading order change.

References lower to existing link events for native HTML, the data-only Next
adapter and PDF. Formulae become native MathML or hosted MathJax, while number
labels and anchors come from the registry. A fixed-width number stays visible
beside independently scrollable math. The Next bridge adds only inert spans
and accessible labels. The LaTeX adapter emits matching explicit tags and
hyperlinks instead of asking a second counter to determine their values.

Validation reports source filenames for invalid labels/references, rejects
collisions and missing targets, and retains the existing atomic publication
contract. Semantic scanning respects comments, escaped control symbols and
command boundaries. Existing math parsing still validates formula syntax.

## Verification plan and retained findings

Native regressions compare publication JSON across inspection/export/Next,
generated HTML and LaTeX, forward/backward references, multi-line formulae,
front matter/title/appendices, reordering, stable anchors and failure preservation.
Engine tests exercise inert-node validation, rendered math and accessible links.
The real CLI fixture is `scripts/fixtures/equations`.

Hosted browser acceptance builds and relocates the fixture, opens it via
`file://` in Chromium/WebKit with JavaScript both enabled and disabled, checks
keyboard links, mobile layout and screenshots. Next image acceptance renders
the same manuscript at `/books/equations`. Actual LuaLaTeX acceptance checks
extracted numbers, PDF destinations, matching revisions and page captures on
Docker amd64/arm64 and rootless Podman.

An initial concurrent local workspace run timed out probing the fake Podman
client in an existing runtime-discovery test. The fake client does not start a
container. A later unchanged runtime test passes. An intermediate equation test
incorrectly expected the duplicate to be reported in the earlier source; the
resolver correctly reports the later conflicting definition with the earlier
filename in its message. That assertion was corrected. The initial Next unit
test also assumed `next/link` applies a deployment base path in a plain React
unit render; actual prefix handling is checked by the hosted prefixed build.
No product behavior was changed to accommodate those test assumptions.

The first hosted run `43552849248` exposed a Windows-only fixture assumption:
the chapter-reorder test replaced LF text in a CRLF checkout and therefore did
not reorder anything. The fixture now explicitly exercises CRLF, normalizes
line endings before replacement, and asserts that the replacement changes the
input. All four equation integration tests pass locally after that correction.
The original Windows failure is retained; it did not require a product change.

The corrected-CRLF run is `43553446088`. Its 26 other jobs pass, but the
Next-image build
finds an adapter defect at `manual/p1.zeldoc.json`: a colon in an authored
fragment was treated as a URL scheme, leaving `../series.html#_zel-eq-eq:sum`
unconverted. Fix `4c399af` splits query/fragment before checking the path for a
scheme. Native regressions cover both fragment and query colons, and the hosted
browser helper follows the custom-text link explicitly. All 68 native docs
tests and warnings-denied crate Clippy pass after the fix. The Next-image gate
moves earlier in CI without removing any checks. The final helper's exact link
text is corrected in `6adfae7` before its hosted run.

The Next-image gate passes at `6adfae7` in `43556287071`, including the
custom-text link, deployed prefix and desktop/phone captures. Source review
then reproduces a second defect: deleting a label between `\alpha` and `x`
forms the invalid command `\alphax`. Fix `09fd2cc` substitutes a space; the
renumbering regression fails before and passes after the correction. All 68
native docs tests and crate Clippy pass afterward. The combined-source
acceptance below closes that verification requirement.

## Verification results

Implementation `2c77a39`, CRLF fixture correction `ed869ce`, URL fix `4c399af`
and TeX-boundary fix `09fd2cc` are tracked in PR #167. Combined-source CI
`43557146014` passes all 27 jobs at `09fd2cc` without workflow reruns. The
intermediate runs and their findings remain above and in the evidence.

- Local: 778 Rust tests pass (five existing hosted-only cases ignored locally),
  warnings-denied workspace Clippy, formatting, 490 engine tests, workspace
  typechecks, engine build, JavaScript syntax and Actionlint. Strict/deep docs
  validation passes 59 routes and 25 examples. The four equation integration
  tests also pass after the CRLF fixture correction.
- Portable browsers: Linux/macOS/Windows all pass relocated `file://` navigation
  in Chromium and WebKit, with JavaScript enabled and disabled. The fixture
  checks forward/backward links, keyboard self-links, front matter/appendices,
  two viewport widths and absence of page errors. All 24 new screenshots are
  retained; Linux desktop/phone, macOS WebKit phone and Windows Chromium desktop
  samples were reviewed. Equation numbers remain visible and text fits.
- Real PDF: Docker amd64/arm64 and rootless Podman each generate five books.
  All 15 receipt hashes verify. The 18 new equation-book page captures and
  extracted text agree across runtimes and match the six visually reviewed
  pages. All 87 existing page captures match the rc.6 review exactly.
- Existing product checks: 378 showcase cases pass, 140 conditional skips,
  zero flaky cases/retries. Full docs preserve 46 original routes, 13 added
  routes, 282 heading IDs, 82 mappings and 29 identical images, with no captured
  hydration errors. All six browser regressions pass. Six native packages,
  four Homebrew targets and four Scoop lifecycle combinations pass.

The [machine-readable evidence](evidence/zellij-book-equations-2026-11-13.json)
retains receipts, page/screenshot hashes and intermediate findings. The
final-source Next-image gate passes with the custom-link regression and both captures. All 26 equation
web captures match their initial artifacts; the six listed desktop/phone
samples were visually reviewed. Merge and publication are recorded below.

## Merge and development-image publication

The merge tree `227b8e4b0c2101ccdbc0185b0cd6324da0a893b6` matches reviewed
head `168c903`; only context/evidence files differ from tested source `09fd2cc`.
All 13 jobs in `43560554366` pass at merge `fe0f82c`. Eight native builds feed
four amd64/arm64 manifests (`main`, `main-maths`, `main-diagrams`, `main-all`);
the published default image builds and serves a site. Manifest digests and job
IDs are retained in the evidence. The final record-only follow-up changes no
runtime or test inputs and needs no repeated product CI or image publication.

## Release boundary and next work

Public documentation marks this feature as source-only after rc.6. No release
or provider deployment is part of this increment. Per-row numbering, references embedded in larger
math, custom tags, theorem/lemma/proof semantics, bibliography/index/glossary,
multilingual print and richer book templates remain separate work.
