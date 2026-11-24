# Original URL component for cached PDFs — 24 November 2026

Status: original-source selection and compatibility qualification. Installed
rc.19 still uses container PDFs; this increment does not publish a compiler or
declare the complete component/release review finished.

## Decision and provenance

Earlier qualification used URL 3.4, whose original source and documentation name
LPPL version 2 or later. The [LaTeX project's published license history](https://www.latex-project.org/lppl/)
lists 1.3c as its current version. The earlier ambiguity is retained in the audit
history; this change does not rewrite that notice or infer permission for 3.4.

Select the independently distributed, unchanged URL 3.3 original instead. The
[TeX Live 2012 final runtime archive](https://ftp.math.utah.edu/pub/tex/historic/systems/texlive/2012/tlnet-final/archive/url.tar.xz)
and [documentation archive](https://ftp.math.utah.edu/pub/tex/historic/systems/texlive/2012/tlnet-final/archive/url.doc.tar.xz)
preserve the original source and documentation with an unversioned LPPL notice.
The source/manual identify version 3.3 dated 12 April 2006; TeX Live records
revision 16864. Its older catalogue-version field still says 3.2, so the source
and manual, not that stale label, identify the selected version.

| Input | SHA-256 |
| --- | --- |
| Runtime archive, 8,404 bytes | `8121206b39935cc52f7efcdeab3a3af73fc322521ec231770c3d23cbfbc65dc2` |
| Documentation archive, 240,792 bytes | `ce0fbde77e1f81cb0d3bf6902bb7b5c4980bd68ed22c8161a8d084fa90c943a4` |
| Unchanged `url.sty` | `a89cb82fae2a1cf2a273fb30d8a96228b80e79a53b5326fd3582a3f21e8ba767` |

All four original component files are preserved: `url.sty`, `url.tex`, `url.pdf`
and `miscdoc.sty`. Runtime notices include the original complete file layout, and
the separate source archive includes both complete pinned upstream archives. The
already collected original LPPL text is retained. This follows the complete-work
preservation model in the [published LPPL](https://www.latex-project.org/lppl/lppl-1-3c/);
the original notice is not assigned a newly invented version number.

## Implementation

`source-policy.json` now supports explicitly selected original packages alongside
the frozen mirror inventory. Each selected runtime resource must match the exact
original package's file pin, source path and archive provenance. Mixing an
original selection with a mirror URL, unknown IDs, duplicate/conflicting entries
or changed bytes fails. Format and font selections remain unchanged; their
source receipts record the updated policy.

The maintainer `originals.py` verifies the pinned archives and stages unchanged
hash-named bytes. `download.mjs` accepts those prepared originals and verifies
them again without an HTTP fallback. This Python step is maintainer tooling;
the installed runtime still needs no system Python. Its receipt is retained in
CI. Full original files are copied into notices and matching source packages,
with checks for missing/duplicate members. No old URL 3.4 runtime/source archive
is selected by the new package assembly.

## Compatibility and verification

Code comparison identifies three behavioral differences from 3.4: query-separator
line breaks, the invalid-math-mode error helper, and the internal math ligature
helper. The original 3.3 is kept unchanged. Zellij generates a controlled print
preamble and escaped hyperlinks; it does not expose this package as a general
replacement for arbitrary user-authored TeX installations.

Dedicated real-engine regressions exercise query strings, underscores, percent
escapes, email, Windows paths, robust URL definitions in headings, descriptive
hyperlinks and font-style selection. An invalid math-mode URL still fails rather
than producing a successful partial PDF. A new repository-owned link-layout
manuscript expands the full matrix to twenty books and checks real authored
links, encoded paths and anchors against the existing container output on Linux.

The first local matrix failed on a long visible URL with `Overfull \\hbox`.
An isolated three-pass reproduction reports the same 3.62152pt overflow using
both URL 3.4 and the selected original 3.3; this is an existing print-adapter
defect, not a component-selection regression. The adapter now adds discretionary
breaks at punctuation in URL-shaped external-link labels, including bold and
inline code labels and bibliography link text. It leaves escaped destinations,
descriptive prose labels and internal-link labels unchanged. A Rust regression
checks those boundaries; both original components compile the repaired isolated
case without overflow. No strict validation threshold is weakened.

Synthetic source tests cover exact original correspondence, changed/unknown
pins, complete distribution preservation, duplicate/missing originals and owned
output preservation. Downloader tests require prepared originals, reject links
and corruption, and prove no fallback request occurs. All package/cache and
process regressions continue on six platforms, including the archive/source
parity check introduced in PR #251. Final-head receipts and native comparison
results belong in the implementation PR.

Remaining: final engine/component distribution review and release-source
arrangement, production package/schema transition, installed `zel pdf` and
build/view/watch dispatch with the verified private Node, offline propagation,
diagnostics/user docs and installed/public-package acceptance.
