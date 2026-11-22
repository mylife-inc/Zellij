# Long-book PDF contents — 22 November 2026

Status: fixed in source by `6923a14`; not included in published rc.19.
This fixes the shared print template, not the cached PDF integration.

## Reproduction

A repository-owned 100-chapter manuscript reaches chapter number 100 in the
printed contents. The book class allocates only 1.5 em to that number. With
the default font, the number overflows by 2.46376 pt and strict PDF publication
fails. The installed rc.19 CLI reproduces the failure on a compact 110-page
book containing two parts. Its temporary inputs were placed in a shared location
to distinguish this failure from the separate Colima input-mount defect.

The initial 302-page stress result in the cached-PDF feasibility record ran only
one pass, before the contents were populated. A later three-pass run stayed at
441,319,424 bytes of guest heap but correctly rejected the same chapter-number
overflow. The native container rejects it too. This is a shared layout defect,
not evidence of a WebAssembly resource limit or a successful publication.

## Change

For 100 or more numbered chapters, the print adapter measures each chapter
number plus spacing in the configured body font after font initialization.
It widens the book class's contents-number column to the largest measurement,
retaining a consistent title alignment. Part headings and front/back matter do
not inflate the chapter count. Books with fewer than 100 chapters keep the
existing template. A failed template patch is an error; strict overflow checks
remain enabled. The existing pinned resource set already contains etoolbox,
which supplies the template patch; no new downloaded resources are introduced.

## Verification

- The installed rc.19 CLI fails the 100-chapter regression; the source fix
  generates all 110 pages with both native LuaLaTeX and the offline prototype.
- Extracted text retains the numbered contents entry, chapter-100 opening and
  final authored paragraph. The last contents page was visually inspected for
  aligned titles and separation between the number and title.
- Native builds also pass with Sans, Libertinus and Amiri body fonts.
- The seven shared print-generation tests, ten CLI PDF integration tests,
  formatting, warnings-denied Clippy and workflow lint pass locally.
- `scripts/check-long-book-pdf.mjs` adds real native typesetting, PDF structure,
  text/page assertions and retained review images to Docker x64/arm64 and
  rootless Podman CI. The six-platform offline probe adds the same manuscript;
  Linux additionally compares every page with native output. Read the current
  pull request checks for hosted acceptance; local checks are not a substitute.

Optional cached PDF packaging, licensing/source completeness, production
isolation qualification and Rust dispatch remain separate unfinished work.
