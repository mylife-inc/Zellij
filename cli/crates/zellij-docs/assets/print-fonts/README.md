# Pinned print fonts

Amiri 1.003 is from aliftype/amiri commit
`480bb746e99ea700bb0d6b4dbf96302d58192103`. Libertinus Serif 7.051 is extracted
unchanged from the official release archive. `manifest.json` records each
upstream URL, byte count and SHA-256; each family retains its OFL notice.
Only selected fonts enter the isolated PDF input. No host-font discovery,
network request or font installation happens when a book is built.

Amiri web files in `engine/packages/zellij/src/themes/fonts` are lossless WOFF2
compression of these four TTFs using fonttools 4.61.1 and brotli 1.2.0, with
`TTFont(..., recalcTimestamp=False)` and `flavor = "woff2"`. No glyph subsetting,
renaming or font-design modification is performed. The web manifest hashes the
compressed files; the existing offline font verifier retains their license.

`node --test scripts/check-print-fonts.test.mjs` validates the print snapshot.
Changing this inventory requires real typesetting, embedded-font and visual
acceptance; do not refresh it during an ordinary build.
