# Book publication model — 2026-11-12

This is the first book publishing increment, following the released rc.5
`book-classic` starter. It implements a complete web reading model shared by
portable validation/build/preview and the existing Next-image adapter. It does
not implement PDF/LaTeX execution or claim the expansion backlog is complete.

## Contract

The optional portable `zel.yaml` `book` block is versioned independently of the
site. It owns order in place of `nav`: automatic title page, front matter,
chapters optionally grouped into one level of parts, and appendices. Sources
must cover every discovered Markdown document exactly once except the automatic
home. Chapter numbers are continuous across parts; appendices use spreadsheet-
style letter sequences. IDs are stable publication identifiers, not URL aliases.
The guide at `examples/zellij-docs/content/02-cli/books.mdx` is the author-facing
field, limit and migration reference.

`zellij-core::book` owns the typed manifest and structural validation.
`zellij-docs::book` resolves it against the already parsed document inventory,
producing one reading sequence for the sidebar, contents, previous/next links,
inspection and future print adapters. The compiler captures original Markdown
when reading each source; hashing never rereads it after rendering.

Portable `_zellij/book.json` and optional `zel check --json` `book` objects are
identical. Their schema version is 1. The revision is SHA-256 over a domain
marker, canonical serialized manifest, each ordered source path and original
Markdown, and sorted referenced asset paths/bytes. Every field is length
prefixed. This is content identity, not a cache key for rendered appearance,
authenticity proof, or reproducibility claim across engine versions.

The generated home renders metadata and an optional contained local cover.
An authored home keeps its own body. Source filenames remain the routing rule;
custom navigation labels do not rewrite authored headings. Every validation or
rendering failure precedes the existing atomic publication step.

Next adaptation emits inert HAST, preserving order and chapter markers. Its
public `/_zellij/assets/book.json` wraps the portable publication record with
an explicit ID-to-hosted-route map. Hosted `site.yaml` and root HTML now accept
language/direction; their defaults remain English/LTR. This uses the current
source engine; released rc.5 does not contain the new model.

The language syntax is deliberately bounded rather than claiming complete IANA
validation. Language and direction declare content semantics; they do not
translate UI strings or supply missing glyphs. Portable inline edges use logical
CSS. No complete Arabic/Hebrew typesetting or font-coverage claim is made.
[W3C language guidance](https://www.w3.org/International/questions/qa-html-language-declarations)
explains the root language declaration independently of writing direction.

## Acceptance

- Structural tests: strict values/fields, version, unique IDs and sources,
  nonempty parts, safe source paths and reserved title-page ID.
- Compiler tests: complete order, front matter/part/appendix numbering through
  AA, author-owned home, escaped metadata, language/direction, cover copying,
  hosted adaptation, publication JSON and revision stability/change sensitivity.
- Failure tests: missing/unlisted/repeated documents, invalid cover/config and
  mixed `nav`/`book` preserve the previous export.
- Actual CLI test: default `docs/zellij` discovery, check without writes and build
  without external tools, shared JSON identity and failed-rebuild preservation.
- Hosted CI: Chromium/WebKit file viewing after relocation on Linux/macOS/Windows,
  search/keyboard navigation, mobile/desktop layout, RTL direction, no-script
  reading, covers and captures. The existing Next-image acceptance also builds
  and serves the same complete book and checks its routes/sidecar/search.

PR #149 merged as `72a84456e7a3192a64c4003020ee68598d893442`, with tree
`cc1843e6b3ad12799c269bf693fa91a7bb339351` identical to tested head
`2b5cfecd9a10e8fac9448089142e0a0f55f72b98`. All 20 jobs in
[`43322268842`](https://github.com/mylife-inc/Zellij/actions/runs/43322268842)
passed: 756 Rust tests (five existing ignored cases run separately), 490 engine
tests, 356 showcase cases (124 existing conditional skips), all six native
packages and all four Homebrew targets. Book CLI and relocated-file browser
acceptance passed on Linux/macOS/Windows; the real Next-image book checks passed.
All 12 Linux desktop/mobile Chromium/WebKit title/chapter/RTL captures and two
Windows captures were visually reviewed. No clipping or overlap was observed;
Windows WebKit serif headings appeared lighter than Linux. Typography is not
claimed to be pixel-identical across platforms.

All 13 development-engine jobs in
[`43328363366`](https://github.com/mylife-inc/Zellij/actions/runs/43328363366)
passed on merge `72a8445`. Four `main` variants were published for Linux
amd64/arm64, and the published default image built and served a site. Native
rc.5 release assets and production documentation/domain remain unchanged.

Local focused book, layout and watcher tests, workspace Clippy with warnings
denied, formatting, engine build, typechecks and strict/deep documentation checks
passed. Two broad local engine runs hit four existing MDX/math timeouts under
heavy host compilation load; the broad Rust run stopped at an existing fake-
runtime timeout. The complete hosted runs passed. The host Homebrew Rust toolchain
could not load its LLVM/z3 dependency; matched rustup tools passed without
changing host packages. Failed attempts remain recorded in
[the evidence](evidence/zellij-book-publication-2026-11-12.json).

First hosted run `43321606840` at `c14c4a9` found two fixture defects: Windows
CRLF left an intended removed-chapter case unchanged, and the existing engine
watch fixture's in-place YAML save briefly published a valid empty configuration
before the intended invalid text. The chapter mutation now ignores line endings
and asserts it changed its input. Watch fixtures stage outside the watched tree
and atomically rename the complete intended edit. These are test-harness fixes;
validation expectations and production watcher behavior remain unchanged.

## Next increments

1. Opt-in build-time PDF with an explicit backend contract. Reuse this model for
   one ordered publication, title page/parts/chapters, print styles and output
   ownership. Generate a downloadable artifact only when requested and verified.
2. LaTeX backend integration: declared prerequisites, bounded process execution,
   actionable missing-tool errors, math, TOC, bibliography/index and cross-reference
   semantics. A data model is not equivalent to TeX typesetting parity.
3. Versioned classic/technical/study templates with coordinated web/print design,
   then native release and documentation publication. Keep rc.5 template archives
   immutable; opt-in migration replaces their existing `nav` block.

[Pandoc's PDF documentation](https://pandoc.org/MANUAL.html#creating-a-pdf)
is a backend reference, not a selected dependency or a claim of lossless
conversion. Provider verification and production domain migration remain deferred.
