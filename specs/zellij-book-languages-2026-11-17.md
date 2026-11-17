# Multilingual books and print fonts — 2026-11-17

Baseline `ebd43e1`, source rc.10; public rc.9 remains unchanged.

## Contract

Extend print from English/LTR to French, German, Spanish and Arabic. Regional
tags select the primary language; Arabic requires explicit `direction: rtl`.
Unsupported languages or conflicting directions/scripts fail before output is
replaced; web-only books remain available. The language layer uses the pinned
LuaLaTeX image and fixed Babel names, with no authored TeX or system-font paths.
Reference: [Babel language loading](https://latex3.github.io/babel/guides/which-method-for-which-language.html)
and [Arabic](https://latex3.github.io/babel/guides/locale-arabic.html).

Arabic uses pinned Amiri 1.003 (OFL) in print and local compressed web fonts.
`pdf.style.body-font` and `title-font` additionally accept `amiri` and `libertinus`;
Libertinus Serif 7.051 is a fixed OFL font snapshot and is rejected for Arabic.
Default English print retains its old TeX and layouts. Other editions/print
styles remain composable. Code/math keep LTR order. UI controls and generated
semantic labels are still English; this is book typography, not translated UI.

Native and Next use one manuscript and print adapter. Configured multilingual
PDF CI requires rc.10. The PDF runtime stays optional, network-disabled and
isolated. Font files enter print input only when selected and never enter the
Next serving image as private TeX sources. Web fonts are pinned/local and load
only when their family is used. Arbitrary custom font files, other scripts and
regional typography variants are not claimed.

## Verification and history

Hosted probe `44069432572` inventories the pinned image: Latin Modern and
DejaVu are available; Amiri/Libertinus are not. Font snapshots therefore ship
with Zellij. First probe `44069360285` failed from shell grouping escaped only
once in JavaScript; `d1ee8f1` corrects it.

Kie.ai is a separate user-requested theme queued after this increment. Hosted
capture returned a Cloudflare 403/security page, which is not a design reference.
Ordinary HTTPS retrieval of public HTML/CSS succeeded locally without browser
execution; design measurement and themed acceptance follow separately.

Verification pending: native tests, real PDFs across Docker/Podman/amd64/arm64,
Arabic native/Next parity, embedded fonts/text extraction/visual pages, moved
file:// reading without network, responsive no-JS reading, default preservation
and full CI. No local browser, container or real LaTeX process is used.

Initial native typesetting run `44071291159` exits successfully across all three
runtimes, but visual review of Arabic pages catches unjoined letter forms when
explicit named Amiri bypasses Babel's automatic script selection. The fixed
named family explicitly selects Arabic script/language for RTL. The same review
catches reversed chapter alignment caused by substituting raggedleft; LuaTeX's
RTL paragraph already interprets raggedright relative to reading direction.
These results are not accepted visual baselines. Extraction checks now normalize
Arabic compatibility forms/bidi controls and require the authored Arabic strings.
