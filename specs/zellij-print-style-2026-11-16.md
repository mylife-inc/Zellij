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
