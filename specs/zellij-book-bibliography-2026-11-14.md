# Shared bibliography and citations — 2026-11-14

Source baseline: `6d2696a` on `main`. Public rc.6 and production hosting remain
unchanged. Implementation and source acceptance are complete at `d9e22cc`
in PR #171. All 27 final-source CI jobs pass; merge and development-image
publication are complete and recorded below.

## Contract and scope

Portable books opt into a local CSL-JSON catalog with
`book.bibliography: {source: references.json}`. The source is inside `content/`,
regular, contained and outside ignored/generated paths. The optional `title`
defaults to Bibliography; `include-uncited` defaults to false. The generated
`bibliography.html` follows appendices in navigation/search and print order,
with a PDF contents entry and no chapter number. Its route and reading ID are
reserved while enabled. A catalog with no citations emits an explicit empty
state unless uncited entries are requested.

`[@key]` and `[@key, p. 42; @other]` are linked numeric citation clusters. Numbers
follow first use in the manifest's reading order, with footnotes visited at the
first call site. Repeats keep their number; clusters keep their authored order.
Unreferenced footnotes are still validated/lowered after the main reading pass;
PDF retains its existing rejection of unreferenced footnote definitions.
Uncited entries, when enabled, follow in catalog order. Stable `_zel-bib-KEY`
anchors survive renumbering. References need no math feature or browser script.

The markers resemble a bounded subset of [Pandoc citation syntax](https://pandoc.org/MANUAL.html#extension-citations).
Only bracketed numeric citations and literal locators are implemented. Citations
are not headings or Markdown reference-link/image definitions. Existing link
labels and image alt text remain literal, as do escaped brackets, inline code,
fenced code and ordinary non-book Markdown. Each cluster is one line, at most
32 references, with at most 256 bytes per locator.

The catalog reads [CSL-JSON reference data](https://github.com/citation-style-language/schema/blob/master/schemas/input/csl-data.json),
not CSL style XML. Required fields are `id`, `type`, `title`. Supported optional
fields are `citation-key`, `author`, `issued`, `container-title`, `publisher`,
`publisher-place`, `volume`, `issue`, `page`, `DOI`, `URL`. Numeric IDs and numeric
volume/issue/page fields work. `citation-key` gives a short alias to exported IDs;
original IDs and aliases are both checked for duplicates. Citation keys use
1–128 ASCII letters/digits/colon/dot/underscore/hyphen and start with a letter,
digit or underscore. Name objects support literal organizations or family/given,
particles and suffix. Dates display year/range, literal/raw text or `n.d.`.
Unknown metadata is not displayed. No raw HTML, TeX or CSL formatting is executed.

The built-in numeric presentation is shared across adapters, without claims of
APA/Chicago compliance. DOI links take precedence over optional HTTP(S) URLs;
URLs cannot contain credentials. No lookup, fetch, Node, Pandoc or bibliography
service is required by the native renderer. Catalogs are bounded to 8 MiB and
10,000 entries. The raw catalog is not published unless explicitly linked as an
asset. Its bytes participate in publication revision identity, even for uncited
or undisplayed metadata. Existing books omit the new optional manifest and
publication fields and retain their earlier revision behavior.

## Model and rendering

`bibliography.rs` owns parsing, catalog validation, reading-order resolution,
link lowering and consistent bibliography text. Markdown's broken-reference
callback recognizes shortcut markers; an initial parse protects existing link
and image labels. Private citation URLs are validated and resolved before any
adapter sees output. Non-book URLs still pass the ordinary scheme checks.

The publication contains an optional bibliography record with title, source,
style, href and the numbered references; raw source bytes remain private. Each
reference exports its citation ID, optional distinct source ID, number, title,
formatted text, anchor, href and optional source URL/label. Web HTML and the inert
Next tree render the same ordered entries. LaTeX uses the same resolved text and
hyperlink destinations after `backmatter`; it does not run an independent
citation counter or a BibTeX pass. The existing optional LuaLaTeX toolchain and
image size are unchanged; the Rust adapter reuses the workspace's existing URL
parser dependency.

The [authoring guide](../examples/zellij-docs/content/02-cli/books.mdx) documents
the full supported contract. BibTeX import, arbitrary CSL styles, author-date/
note styles, locator localization and author-in-text syntax remain separate
work. Index/glossary, multilingual print, Next-image PDFs and richer book editions
also remain open. This increment does not claim zero bugs or a complete 1.0.

## Verification and retained findings

Integration coverage exercises registry/numbering parity in native/Next/PDF,
stable links, relocation, search, repeated/multiple citations, footnotes,
uncited/empty catalogs, numeric IDs/aliases, literal metadata, revision changes,
malformed inputs and preservation of the previous export. Initial local tests
passed six cases after correcting a fixture that accidentally removed equations
referenced elsewhere; the missing-reference validation itself was correct.

Review also found two integration edges before acceptance: a new shortcut
citation could consume the label of an existing outer Markdown link, and the
private-scheme bypass initially applied to non-book links. Existing link/image
ranges are now protected; the bypass is restricted to validated book citations.
Regressions cover literal nested link labels and rejection of the private scheme
outside books. Original and alias reference IDs are both checked for duplicates.

Hosted fixtures extend the equation/statement book without altering the earlier
PDF inputs. CI exercises relocated `file://` books in Chromium/WebKit on all
three OS families, JavaScript on/off, two widths, prefixed Next images and real
PDF on Docker amd64/arm64 plus rootless Podman. It retains bibliography browser
captures, PDF receipts, extracted text/fonts and every page capture. Final source
results and exact commits/job IDs are recorded below.

The separate WebKit overflow and docs hydration timing findings remain open.
Live provider acceptance and production-domain migration stay deferred.

Local verification passes 789 workspace Rust tests (five hosted-only cases
ignored locally), 490 engine tests, workspace typechecks, engine packaging,
warnings-denied workspace Clippy, formatting and Actionlint. All 79 native docs
tests pass; the final seven bibliography integration cases and final docs Clippy
pass after the edge-case corrections. Strict/deep validation passes all 59 docs
routes/25 examples and the actual six-page Rust-emitted Next manuscript.
The isolated unprotected-parser probe reproduces the nested-link ambiguity:
only the private citation target survives and the original external URL is lost.
The protected parser regression retains the outer link and literal marker.
No local browser, GUI, container or actual LaTeX execution was used.

## Final source acceptance

All 27 jobs in [CI `43578754363`](https://github.com/mylife-inc/Zellij/actions/runs/43578754363)
pass at exact source `d9e22ccd5d0b23494e66ffa1165d968ced825fc9` without workflow
reruns.

- Portable books pass relocated `file://` acceptance in Chromium/WebKit on
  Linux/macOS/Windows, with JavaScript on/off and two viewport widths. Prefixed
  Next passes the same references, bibliography links and responsive checks.
  All 26 new browser captures are retained; desktop/mobile Next plus native
  Linux/macOS/Windows samples are visually reviewed. Numeric labels appear once,
  entries wrap readably, and keyboard focus and previous navigation remain visible.
- Docker amd64/arm64 and rootless Podman each render seven books: 21 PDFs and
  156 page captures. All 129 previous captures are byte-identical. The nine-page
  bibliography manuscript has identical rendered PNGs across all three runtimes;
  pages 2, 3, 6, 7 and 9 were inspected directly, and pages 1, 4, 5 and 8 match
  previously reviewed statement pages. Contents entries, footnote citations,
  locators, bibliography hanging paragraphs and links are readable without
  observed clipping or overlap. PDF receipts, text/font checks and qpdf checks
  pass. This is rendered-layout equality, not PDF binary equality.
- Existing engine coverage passes 490 tests. Showcase passes 378 cases with
  140 conditional skips and no retries. Documentation preservation covers
  46 original and 13 added routes, 282 original heading IDs, 82 compatibility
  mappings and 29 byte-identical images; all six browser regressions pass.
  Six native packages, four Homebrew targets and four Scoop combinations pass.

The [machine-readable evidence](evidence/zellij-book-bibliography-2026-11-14.json)
retains job IDs, local logs' hashes, PDF receipts, capture hashes, visual review
and the initial parser/fixture findings. A documentation-only commit follows
CI: context, ledgers, this contract, evidence, and a two-line public-guide wording
clarification that URL-query rejection in PDF applies to internal links.
That guide passes strict/deep validation separately (59 routes, 25 examples);
runtime, test and workflow inputs are identical to the tested source.

Index/glossary is the next implementation priority. BibTeX/custom CSL, broader
print support, richer book editions, remaining installation channels and the
separate unresolved browser timings stay open. No new public CLI candidate or
production deployment is performed by this increment.

## Merge and development-image publication

PR #171 merges as `e58133bceee500849376cdd17d969069a0399388`. Its tree
`43368f80d05accd4838f61927fc379ee0189012f` matches reviewed head `11b3c55`.
Only the six documentation/evidence paths described above differ from tested
source `d9e22cc`; runtime, test and workflow inputs are unchanged.

All 13 jobs in [development-image run `43581129613`](https://github.com/mylife-inc/Zellij/actions/runs/43581129613)
pass at that merge without reruns. Eight native builds feed four manifests
(`main`, `main-maths`, `main-diagrams`, `main-all`) on Linux amd64/arm64. The
published default image builds and serves a site. The JSON evidence retains
all job IDs and the observed manifest digests/architectures.

This final record-only follow-up changes context, this contract and evidence;
it does not repeat product CI or image publication. Public rc.6 and production
hosting remain unchanged. Index/glossary remains the next implementation task.
