# Bundled theme fonts

These are unmodified WOFF2 files from Google Fonts, captured on 2026-10-17.
`manifest.json` records every source URL, SHA-256, byte count, the request user
agent, the pinned Google Fonts license revision and the fallback-metrics source.
The twelve families retain the previous `next/font/google` requests' normal
styles, weight ranges, Unicode subsets and `font-display: swap`. There is no
font preload. Browsers fetch only the families and character subsets in use.

`upstream/` preserves the original CSS responses. `fonts.css` changes their
family names to avoid collisions and rewrites URLs to `files/`, then adds the
same metric-adjusted fallbacks and public CSS variables. Next bundles these
relative assets under its normal static asset path, including the site's
basePath. Neither a site build nor development needs Google Fonts access.
Installing npm dependencies can still require network access.

Every family is distributed under its included SIL Open Font License. Keep the
notices in `licenses/` and the generated `/*! ... */` comments: the latter carry
the notices into compiled sites. The font binaries themselves are unchanged.

To intentionally update a family:

1. Fetch its manifest CSS URL with the recorded user agent; review changes in
   weights, axes and Unicode coverage. Preserve the complete CSS response.
2. Download every referenced WOFF2 file (URLs need not end in `.woff2`). Check
   the WOFF2 signature, name them with the family key and first 16 SHA-256
   characters, and update their URLs, sizes and full hashes in the manifest.
   Deduplicate repeated URLs across weights; remove obsolete files.
3. Retain the corresponding upstream OFL notice at a pinned Google Fonts
   revision. Update its manifest hash and provenance. Review fallback metrics
   if the font's metrics changed; the initial snapshot uses Next.js 16.4.2's
   `calculateSizeAdjustValues`, without requiring that helper at build time.
4. Run `node engine/packages/zellij/scripts/prepare-fonts.mjs --write`, then
   build the engine. The normal build verifies hashes and generated CSS and
   copies the font directory beside the compiled ESM/CJS modules.
5. Run the production and development browser gates, including the production
   build with network disabled. Check typography, language coverage, lazy font
   loading, URL prefixes and preserved license notices before committing.

Do not refresh fonts during normal builds. The committed bytes are the source
of truth; a changing upstream response must be reviewed as an explicit update.
