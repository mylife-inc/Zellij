# Technical and study book editions

Date: 2026-11-14. Baseline `af31e7f`; source rc.8 is unreleased. Published rc.7
and all its templates remain untouched. This implements the next book-template
milestone from the product expansion, without adding a toolchain dependency.

## Contract

`book.edition` accepts `classic` (default), `technical`, `study`. It belongs to
the shared publication manifest. Omitted and explicit classic serialize identically,
retaining the original revision and rendered source. Other editions participate
in the revision so PDF download metadata describes the same selected edition.
Unknown names and wrong types fail before publishing over an existing export.

Native articles and inert adapted Next documents carry scoped edition classes.
Both renderers use `engine/packages/zellij/src/styles/book-editions.css`; native
books emit it only for nonclassic editions. Neither document adapter nor authored
markup gains executable properties. Reader themes, dark mode, offline navigation,
keyboard access and forced colors remain supported. No scroll animation is needed
for long-form reading. Existing classic HTML/tree/TeX remains unchanged.

Technical pairs blue rules and Outfit headings with Source Serif reading text.
Study pairs warm accents, Cormorant headings and ruled quotations with Source Serif.
Print uses coordinated cover/chapter treatment with existing Latin Modern fonts,
not identical web fonts or pagination. Editions extend the classic preamble using
internal constants; authored text is still escaped. Receipts identify the actual
`classic-v1`, `technical-v1` or `study-v1` print design.

The two new bundled/local/archive templates begin at 0.1.0 and require CLI/engine
rc.8. They have typographic covers, preface, part, two chapters, appendix,
bibliography, glossary and index. Technical demonstrates a correct finite-sum
proof, invariant and code. Study provides original passages, commentary, questions
and footnotes. Reference entries are clearly marked illustrative. No downloaded
artwork or added fonts are required. PDFs remain off by default and English/LTR.
The three previously published template archives retain their exact bytes/versions.

## Verification

Local targeted Rust checks pass for edition validation, classic equivalence,
revision changes, portable HTML and inert adapted trees, template content,
PDF source generation and failed-output preservation. Full local validation passes 799 Rust tests (five hosted-only cases ignored),
490 engine tests, 95 helper tests, all workspace typechecks, engine build,
formatting, warnings-denied Clippy and strict/deep documentation validation.
All five bundled/local/archive starters match, deterministic packaging passes,
and all three published template hashes remain unchanged. The actual published rc.7
binary and rc.8 source produce 89 byte-identical files for the classic starter.
Released rc.7 rejects each new template with an upgrade message before creating
the destination.
New acceptance captures light/dark desktop/mobile, forced colors and no-script
relocated output on Chromium/WebKit across Linux/macOS/Windows; Next exercises
both editions with a path prefix for technical. Real PDF checks run both new
starters on amd64/arm64 Docker and amd64 rootless Podman. Existing classic
PDF captures remain unchanged. No local browser/container/LaTeX runs.

Final source `02d0c7b` passes all 27 jobs in workflow `43683371365`, including
six native packages, four Homebrew and four Scoop combinations. Showcase passes
378 cases without retries, with 140 conditional skips. All 250 new edition web
captures are retained; representative native Linux/macOS/Windows and Next layouts
are visually reviewed. Thirty PDFs/264 page captures pass across the three
runtimes: all 24 previous TeX sources and 192 previous page images are unchanged.
Each new starter has 12 pages; its rendered pages match across runtimes and all
fonts are embedded with Unicode mappings. Cover, contents, proof/math, table/code,
passage/commentary and index samples were inspected.

The first run `43682531662` passed all PDF jobs but exposed a Next test assumption:
`innerText` reflected the edition CSS uppercase styling, making `Chapter 1` become
`CHAPTER 1`. Fix `02d0c7b` asserts the semantic DOM text with Playwright instead.
It changes no renderer, template or PDF input. The downloaded PDF comparison
uses that first run; final-source PDF jobs pass again. The cancelled first run
and this correction remain in the [evidence](evidence/zellij-book-editions-2026-11-14.json).

## Remaining scope

Next implementation priority: deliver configured PDFs in Next images, which
currently reject PDF-enabled books. Keep typesetting at build time, reuse the
publication revision/receipt, expose a download link, and keep the optional LaTeX
toolchain out of the final serving image.

Custom typography/design-token authoring, multilingual/RTL print, alternate
citation styles/BibTeX, deeper index hierarchies and additional installation
channels remain open. Live providers and production-domain migration remain
deferred. No 1.0 or zero-bug claim is made.

Explicit `book.edition` settings require a CI pin of rc.8 or later, including
explicit `classic` (older binaries reject the new field). `zel ci` rejects an
older `--zel-version` before writing any workflow. Generation does not publish
the candidate or check whether its release assets are available yet.

## Merge and development images

PR #178 merges as `b57d71d`, with the same tree as verified `02d0c7b`.
Development-image workflow `43685610738` is dispatched at that merge after
confirming no automatic run was created by the skip-CI merge. All 13 jobs pass,
publishing four `main` variants on amd64/arm64 and building/serving a site from
the published default image. The first HTTP startup probe reset; the next
succeeded within the existing bounded loop. Manifest digests and probe results
are retained in the evidence. Public rc.7 assets, tags, package recipes and hosting
remain unchanged. The follow-up evidence update changes no runtime, active
guide, versioned template, test or workflow inputs.
