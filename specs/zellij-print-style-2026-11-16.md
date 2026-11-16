# Authored print styles — 2026-11-16

Baseline: `23c56d0`. Public rc.9 remains unchanged. Source workspace version
advances to rc.10; publication is a separate reviewed milestone.

## Contract

`pdf.style` adds optional accent, body/title font, chapter style, line spacing
and millimeter margins to all three editions. Only quoted RGB hex, enums and
bounded integers enter internal LaTeX templates. Serif/sans use the existing
Latin Modern families; math/mono remain unchanged. No dependency, custom font
file, raw preamble or executable author input is added. Chapter style overrides
numbered and unnumbered openings; the cover composition and web edition stay
selected by `book.edition`. Body-family changes also affect ordinary section
headings, while title-family changes affect cover and chapter titles.

Omitted and empty styles generate the same source bytes as the original edition.
Omitted style stays absent from serialized settings. Print settings remain in the
PDF receipt, separate from the manuscript revision. The existing snapshot check
and watcher include configuration, so a style-only edit rebuilds even when the
manuscript revision is unchanged. Existing output survives validation, typesetting
and concurrent-edit failures. All generated CI systems reject pre-rc.10 pins
before writes whenever style is explicit, including an empty or disabled style.

Geometry retains the existing paper choices and uses 15–40 mm physical margins;
omitted sides retain the established values. Baseline multipliers are selected
from fixed constants. Reference: [geometry](https://ctan.org/pkg/geometry) and
[LaTeX font selection](https://www.latex-project.org/help/documentation/fntguide.pdf).
No multilingual/RTL support or arbitrary installed-font support is claimed.

## Verification plan and history

Native tests cover all nine edition/chapter combinations, exact empty-style
source equality, native/Next print-source equality, unchanged web and manuscript
revision, receipts, invalid input/output preservation and all three CI generators.
Real LuaLaTeX runs only in hosted CI on amd64/arm64 Docker and rootless Podman.
Two styled books retain rendered page captures, extracted text and embedded-font
reports. Next publishes a third styled fixture and verifies download/prefix,
private-file exclusion and failed rebuild preservation. Watched style changes
must update the receipt/hash while preserving the content revision; the same
styled PDF is served from a minimal static image. Existing PDF source/page
baselines are compared against reviewed rc.9 evidence.

Initial local command corrections: workspace formatting required `--all`;
a preparation-script insertion marker differed after formatting and was corrected
before adding the new tests. Existing and new focused Rust checks then pass.
These were local preparation errors, not typesetter failures. Full local and
hosted verification, visual review and merge results will be recorded below.

No local browser, GUI, container or real LaTeX process is used. Public release,
package channels, template archive bytes, production hosting and live-provider
tests remain unchanged by this implementation milestone.

Initial hosted run `43946520960` catches the standalone Docker recipe still
selecting rc.9 after the native source version advances. The version-consistency
regression fails before the engine test stage; the recipe pin is corrected to
rc.10. The original failed/cancelled run is retained. Full local validation passes
806 Rust tests (five existing hosted-only cases ignored), formatting and
warnings-denied Clippy; engine build and strict/deep docs pass (59 routes).


## Verified result

Final source `c8893da` passes all 32 jobs in `43946832641`. Local checks pass
806 Rust tests, 103 helper tests, formatting and warnings-denied Clippy. Hosted
engine tests pass 491 cases; showcase passes 418 without retries (140 conditional
skips). Documentation retains 46 original routes, 282 heading IDs and all 29
original image bytes.

All 30 prior TeX sources and 312 page captures are unchanged. The 42 checked
PDFs include eight styled outputs and 101 new page captures; technical native
and Next output match across five runtime/architecture cases, and study matches
across three. Ten representative covers/contents/chapters/proofs/tables/prose/
footnote/index pages are visually reviewed; six are retained in evidence.
Settings survive watched rebuilds and actual static/Next image publication.
The LaTeX toolchain and font inventory are unchanged and absent from serving
images. The first local comparison mistakenly compared the saved enabled flag
between native `--pdf` and configured Next; it now verifies each expected flag
and compares the actual print settings, revision, text and page pixels.
See [the evidence](evidence/zellij-print-style-2026-11-16.json).

PR #194 carries the verified implementation. The final records update changes no runtime, active guide,
versioned template, test, workflow or image inputs from verified `c8893da`.
