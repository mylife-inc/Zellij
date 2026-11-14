# Shared glossary and index — 2026-11-14

Baseline `0a29442` on main. Implementation is in progress; local and hosted
acceptance, merge and development-image publication remain pending. Public rc.6
and production hosting remain unchanged.

## Contract

Optional `book.glossary` and `book.index` each have a contained local JSON
`source` and optional plain `title`. They generate `glossary.html` and
`book-index.html` after bibliography/appendices; the home remains `index.html`.
Corresponding routes and reading IDs are reserved only while enabled. Catalogs
are regular files outside hidden/generated paths, at most 8 MiB and 10,000 terms.
The source bytes affect revision identity without being copied into public assets
unless explicitly linked. Absent fields preserve existing book revisions.

Each strict catalog entry has `id`, `term`, optional `sort` and `see-also`.
Glossary definitions are required plain text; index entries may instead have a
`parent` naming a different top-level term. One index subentry level is supported.
All entries are included, with an explicit unused state. Sorting uses lowercase
Unicode text then exact ID, with children immediately after their parent. This
is deterministic ordering, not locale-aware collation. Catalog IDs have separate
namespaces. Text, ID, occurrence and see-also bounds and examples are in the
[authoring guide](../examples/zellij-docs/content/02-cli/books.mdx).

Authors mark `[visible text](glossary:id)` or `[visible text](index:id)` in prose
or footnotes. Plain text/inline-code labels are allowed; headings, image labels,
empty/rich labels and URL queries/fragments are rejected. Code stays literal.
Ordinary non-book links still reject these unsupported schemes. Undefined IDs,
unknown fields, duplicates, cycles and collisions fail before export replacement.

One registry in `terms.rs` resolves all references and occurrences before output.
Each collection appears in optional publication-schema-v1 `terms`. Entries expose
term/definition/sort/parent/see-also data, stable anchor/href and occurrences;
occurrences expose reading-entry ID, source, display label, anchor and href.
Raw catalogs remain private. `_zel-use-` reference-link IDs are reserved before
lowering. Adapters consume normal resolved links with generated occurrence IDs;
no pseudo scheme is emitted or treated as an unvalidated ordinary URL.

Term targets use `_zel-KIND-ID`. Occurrence targets use `_zel-use-KIND-ID-N`,
scoped to the source page, counting the term's markers in source order. Different
term insertions preserve these targets; inserting the same term may renumber
later targets. Backlinks follow manifest reading order, visiting footnotes at
their first call site. Unreferenced footnote definitions remain lowered for web;
PDF retains the existing rejection of unused definitions.

Native HTML and Next use semantic lists, readable definitions, one-level index
indentation and ordinary keyboard links. Web entries expose every occurrence.
PDF uses the same anchors and labels, with linked page references and duplicate
folios removed per entry. Existing hyperref/refcount support and the existing
three LuaLaTeX passes supply page values; no new runtime package or executable
is added. See the [hyperref reference](https://tug.ctan.org/macros/latex/contrib/hyperref/doc/hyperref-doc.html)
for explicit targets and starred page references. Pulldown's link IDs describe
reference labels, so the adapters explicitly render validated generated IDs;
see [the parser API](https://github.com/pulldown-cmark/pulldown-cmark/blob/v0.13.5/pulldown-cmark/src/lib.rs).

## Acceptance and retained findings

Seven integration cases cover three-output publication identity, stable anchors,
sorting/subentries, footnote reading order, source revision changes, undefined
links, unsafe catalog paths, cycles, markup/data escaping, non-book behavior,
math-free use and failed-export preservation. During review, a naive home-route
prefix replacement would have collapsed `index.html.md`; the adapter now only
rewrites an exact `index.html#` home target. A regression covers the dotted page.

Hosted fixtures extend the bibliography manuscript without changing existing
PDF inputs. Acceptance covers relocated offline Chromium/WebKit on all three
OS families, JavaScript on/off, two widths, prefixed Next and actual PDF under
Docker amd64/arm64 and rootless Podman. The fixtures retain PDF receipts/text/fonts,
all page captures and glossary/index screenshots for visual review. Exact
results will be recorded after observation.

Deeper index hierarchies, page-range compression, automatic indexing, rich
Markdown definitions and localized collation remain extensions. BibTeX/custom
CSL, multilingual/RTL print, Next-image PDFs, richer book editions and remaining
installation channels remain open. The earlier separate WebKit overflow and
docs hydration timings remain unexplained. Live providers and production-domain
migration remain deferred. This increment does not establish zero bugs or 1.0.

Local verification passes 796 workspace Rust tests (five hosted-only cases
ignored locally), 490 engine tests, workspace typechecks, engine packaging,
warnings-denied workspace Clippy, formatting, Actionlint and JavaScript syntax
checks. All 86 native docs tests pass after the final renderer edits. Strict/deep
validation passes 59 documentation routes and 25 examples, plus the actual
Rust-emitted eight-page Next manuscript. Fresh dependency installation reports
no vulnerabilities. No local browser, GUI, container or actual LaTeX was run.
Hosted source acceptance remains required before merge.
