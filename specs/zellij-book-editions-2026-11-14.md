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
and all three published template hashes remain unchanged. Hosted native/browser/Next
and actual PDF acceptance are pending.
New acceptance captures light/dark desktop/mobile, forced colors and no-script
relocated output on Chromium/WebKit across Linux/macOS/Windows; Next exercises
both editions with a path prefix for technical. Real PDF checks run both new
starters on amd64/arm64 Docker and amd64 rootless Podman. Existing classic
PDF captures must remain unchanged. No local browser/container/LaTeX runs.

## Remaining scope

Custom typography/design-token authoring, multilingual/RTL print, alternate
citation styles/BibTeX, deeper index hierarchies and additional installation
channels remain open. Live providers and production-domain migration remain
deferred. No 1.0 or zero-bug claim is made.

Explicit `book.edition` settings require a CI pin of rc.8 or later, including
explicit `classic` (older binaries reject the new field). `zel ci` rejects an
older `--zel-version` before writing any workflow. Generation does not publish
the candidate or check whether its release assets are available yet.
