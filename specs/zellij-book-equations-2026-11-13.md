# Shared book equations — 2026-11-13

Source baseline: `d7a8416` (PR #166). Public rc.6 remains unchanged.
Implementation is under verification; do not call hosted acceptance complete
until the final run and source commit are recorded below.

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

Final local and hosted results remain to be recorded. Public documentation
marks this feature as source-only after rc.6. No release or provider deployment
is part of this increment. Per-row numbering, references embedded in larger
math, custom tags, theorem/lemma/proof semantics, bibliography/index/glossary,
multilingual print and richer book templates remain separate work.
