# Product expansion: branding, composition, templates, books and installation

Date: 2026-11-11. Source baseline: `3ca63ba` on `main`; latest published CLI:
`v0.2.0-rc.3`. These are new requirements beyond the completed earlier CLI
contract. This document records the user's intentions and a proposed staged
design; proposed commands remain planned unless explicitly marked implemented below.

## Delivery order and release boundaries

| Order | Deliverable | Timing and acceptance |
| --- | --- | --- |
| 1 | Separate company and product branding | Implemented and released in rc.4, including portable/hosted rendering, mobile layout, independent destinations/tab selection, dark variants and compatibility. |
| 2 | Package-manager distribution | Homebrew is live and verified with rc.5. Add the owned Scoop bucket next; submit WinGet after the intended release is available. Test install, upgrade and uninstall on supported native architectures. Stable channels must not silently install release candidates. |
| 3 | Template catalog and creation command | Released in rc.5: versioned bundled/local/release templates, guarded creation, offline cache and receipts. Product documentation, hosted product landing and classic reading-book starters are verified. Technical/study editions remain planned. |
| 4 | Composition engine and additional Apple-inspired patterns | Build a bounded composition format, reusable presentation/effect primitives and named presets. Extend the product template using these; retain all existing section types and authored content. |
| 5 | Complete book publishing and PDF | Extend the book template into a publication model, dedicated print renderer and version-matched downloadable PDF. Validate technical, prose and multilingual books before calling this milestone complete. |

Do not delay the next small CLI candidate until the entire expansion is done.
The user reserves `1.0.0` for the completed agreed feature set with no known
unresolved bugs and passing release/acceptance gates. Continue incrementing
pre-1.0 versions for delivered additions; do not promise a completion date.
The user's follow-up also requests removal of superseded pre-1.0 distributions
after each successful replacement. See [the retention and migration policy](zellij-release-retention-2026-11-11.md).
Update local reference docs and examples with each implemented increment. Public
documentation should describe the corresponding released version, clearly
marking newer source-only features. The existing production-content preservation
and live Netlify migration gates remain in force. No new provider accounts,
production cutover, package publication or release is performed by this design.

## 1. Two independent brand marks

At the baseline, navigation had one `brand.logo` and optional `brand.title`.
The logo could leave for the owner website, consuming the product icon. PR #131
preserves that legacy configuration and adds an optional independent
`brand.company` before it, so a product logo with no explicit `href` still goes
home. This is implemented and included in published rc.4:

```yaml
brand:
  company:
    src: assets/company.svg
    alt: Company website
    href: https://company.example
    newTab: true
  logo:
    src: assets/product.svg
    alt: Product documentation home
```

`newTab: false` is the default. Hosted company marks may also have a `dark`
asset, independently of the product image. Portable fields are `src`, `alt`,
`href`, `newTab`; local source assets must remain valid after export relocation.
Company links require HTTP(S), a nonempty accessible label and safe new-tab
attributes. A company mark can coexist with a product-name home link when the
product has no image. Search/theme/menu controls must retain their usable space.
Presentation lives in `zel.yaml` for portable content and `content/site.yaml`
for hosted content, following the current contract.

## 2. Existing sections and Apple comparison

The complete current catalog is: `hero`, `hero-split`, `logo-strip`,
`feature-grid`, `zigzag`, `showcase`, `stats-band`, `testimonial`,
`pricing-table`, `faq`, `cta-banner`, `markdown-prose`, `video-embed`,
`media-panel`, `card-carousel`, `bento-grid`, `timeline`, `steps-panel`,
`code-cards`, `matrix`, `team-grid`, `gallery`, `promo-grid`.

Source inspected: `engine/packages/zellij/src/sections/components/index.ts`,
`sections/schemas.ts`, `sections/shared.tsx`, `motion/scroll-stage.tsx` and
the existing motion/browser tests. The engine already has gradient titles,
viewport reveals, parallax, sticky media with a scale/opacity ramp, horizontal
cards, image lightboxes, responsive grids and reduced-motion fallbacks.

Current reference pages reviewed on 2026-11-11:
[Apple home](https://www.apple.com/), [iPhone](https://www.apple.com/iphone/),
[iPhone product detail](https://www.apple.com/iphone-18-pro/) and
[MacBook Pro](https://www.apple.com/macbook-pro/).
The local browser initially failed to initialize because the configured writable
root traverses the Codex-home symlink. A separate hosted Chromium audit then
loaded all four public pages successfully and captured 17 selected desktop/mobile
views. A second run verified the iPhone media accordion switching its open item
and companion image, and a feature card opening a rich-content dialog; three
additional screenshots were inspected. See [the research record](evidence/zellij-apple-section-audit-2026-11-11.json)
for exact runs, source commits, URLs, image hashes and retained local references.
This establishes those rendered states and interactions, not exhaustive
accessibility or precise scroll-animation timing.

| Pattern | Existing capability | Useful next work |
| --- | --- | --- |
| Large product heroes and promotional tiles | `hero`, `hero-split`, `promo-grid`, `bento-grid` | Better ready-made compositions and original sample assets; no duplicate section type merely for spacing changes. |
| Horizontal highlight cards | `card-carousel`, `gallery` | A feature-detail preset with accessible rich-content dialogs, beyond the existing image-only lightbox. |
| Text accordion with a synchronized large illustration | `faq` and static split/media sections separately | A `media-accordion` preset: one selected disclosure controls the text and companion image, with keyboard operation and a sequential reading fallback. This is the first concrete additional section candidate. |
| Product/color or specification choices | Static image/text sections and a static `matrix` | A media-selector primitive: named choices update image, caption and associated details, with keyboard and touch controls. |
| A visual that stays while a story advances | `showcase`, `media-panel`, `ScrollStage` | Multi-step scroll story with synchronized media/text states, plus an ordinary sequential reading fallback. |
| Large gradient typography | `titleGradient`, marked gradient text and `Reveal` | A reusable solid-to-gradient reveal tied to visibility/progress, as requested by the user. |
| Product comparison | `matrix`, `pricing-table`, `stats-band` | Selectable comparison data where it improves an actual template; keep a useful static/print table. |

The multi-step scroll story and solid-to-gradient heading are proposed Zellij
treatments, not claims of verified animation on the inspected Apple pages.
Use original content,
illustrations and appropriately licensed assets. A template can reproduce a
design pattern, but its quality also depends on typography, imagery and copy;
it cannot guarantee pixel-identical results for arbitrary replacement content.

## 3. A composable section, with presets

Introduce `type: compose` as an opt-in section. Keep the 23 concise section
types for common tasks. The composer is a typed tree, not an unrestricted HTML,
CSS or JavaScript execution language. Named presets are ordinary validated
compositions, so a beginner can choose a preset while an advanced author edits
its structure. Existing sections need not be rewritten to introduce it.

Separate four concerns:

- Structure: stack, grid, split, layers and bounded sticky stages; responsive
  breakpoints and reading order are explicit.
- Content: heading, rich text/Markdown, image, video, button/link, icon, stat,
  divider and a supported existing section. Book-specific semantics belong to
  the publication model rather than a pile of decorative text nodes.
- Appearance: theme tokens, type scale/weight, spacing, alignment, surface,
  radius and bounded gradient treatments. Shared tokens serve web and print.
- Behavior: reveal, gradient reveal, small transform ramps and scroll stages.
  Start/end progress, target and fallback are declared; no authored executable
  callbacks or arbitrary scripts are needed.

Illustrative syntax to validate during the composer prototype (not available
in the current release):

```yaml
type: compose
version: 1
layout: { kind: stack, align: center, gap: large }
children:
  - type: heading
    level: 2
    text: A beautiful idea, revealed.
    typography: { scale: display, weight: bold }
    effects:
      - type: gradient-reveal
        trigger: viewport
        range: [0.15, 0.65]
        from: foreground
        to: ["#2575fc", "#9c27b0"]
  - type: text
    markdown: Readable content remains the foundation.
  - type: image
    src: assets/feature.webp
    alt: An illustration of the feature
```

The schema must bound tree depth/node count, validate layout and effect
combinations, preserve source locations in diagnostics, and support the existing
`check`, `explain`, `context` and section scaffolding paths. Prefer one declarative
representation; an MDX/tag shorthand can compile to it later, without becoming
a second incompatible format.

Prototype acceptance: recreate an existing hero, a split media section, a
feature grid and the requested gradient heading using the same primitives.
Then create `media-accordion`, `scroll-story`, `feature-explorer` and
`media-selector` presets where the prototype demonstrates an actual gap.
Verify hydration, keyboard
operation, mobile layouts, reduced motion, forced colors, missing/disabled JS,
and print fallbacks. Text must never disappear permanently awaiting an effect.
Animations cannot be reproduced in PDF; print resolves to an authored stable
state. Publish renderer capabilities explicitly: today's portable Markdown
renderer does not already implement the hosted 23-section catalog.

This can cover a broad family of sections. Literally every imaginable layout
and interaction would require extending the primitive/component registry; do
not market the initial bounded format as unlimited.

## 4. Versioned templates and book publishing

Store authored templates in the main repository, independently of `examples/`:

```text
templates/
  catalog.yaml
  product-docs/
  product-studio/
    template.yaml
    files/
      zel.yaml
      content/
  book-classic/
  # planned: book-technical/, book-study/
```

Generated site layout remains `my-docs/zel.yaml`, `my-docs/content/` and generated
`my-docs/site/`. The enclosing folder can have any name. A manifest declares
stable ID/version, title, preview, minimum engine/schema versions, renderer,
optional features, supported outputs, editable variables and asset licenses.
Book templates additionally declare their supported print renderer and provide
coordinated web/print styles; include this contract in the template milestone
even though full PDF execution arrives later. A template must not advertise
working PDF output before its renderer and publication checks are implemented.

The canonical creation command and short entry point shipped in rc.5
(`book-technical` remains planned):

```sh
zel new site my-docs --template product-studio
zel create my-docs -t product-studio
zel create my-book -t book-technical
zel templates
zel templates show book-classic
```

`create` delegates to the same implementation as `new site`; existing `new
docs`, `new portable`, `new app`, `init` and their aliases remain supported.
Template selection determines the renderer; it must not silently promise a
hosted interactive design in portable mode. Keep an explicit capability list
for direct-file viewing, hosted web and PDF.

Publish a catalog/archive per release with immutable revisions and checksums.
Author templates in this source repository, but distribute their catalog and
archives through the existing public `mylife-inc/releases` mirror. Default
creation must keep working if the development repository is later sealed or
retired; it must not depend on downloading a branch of this source repository.
Resolve `id@version`, record the source revision in a lock/receipt file, and
support cached/offline and local-directory templates. Extract into a new/empty
destination atomically, protecting existing authored files. Validate archive
paths, compatibility, referenced assets and variable substitutions before
committing output. Template installation does not run arbitrary repository
hooks. Editing generated files stays ordinary Markdown/YAML editing; later
template updates must show a diff rather than overwrite a manuscript.

Initial templates: plain product documentation; an editorial product site using
existing Cupertino-style sections; classic prose/novel; technical/math; and a
study edition suited to quotations, commentary and religious/nonfiction works.
Ship the first small catalog before promising every variant. Add original
interactive product presets as the composer is proven.

### A book is a publication, not merely a long page

Define a book manifest with title/author/language/direction, cover, ordered
parts/chapters, front matter, appendices, bibliography and output settings.
Plan stable cross-references, footnotes, figures/tables and math/equation labels.
Include chapter navigation, search, reading width/type scale and a calm reading
mode for the web. Test Unicode and RTL text with suitable bundled/licensed fonts
before claiming multilingual book support.

Print needs trim size/margins, chapter opening rules, page numbers, running
headers, a paginated table of contents, footnote placement, widows/orphans,
image/table breaking rules and linked PDF bookmarks. Share typography, color,
ornaments and content with the web theme while adapting layout for pages.
Screen motion/video become stable artwork or a labelled link in print.

PDF is **opt-in**. A normal build produces the website without invoking or
requiring any PDF toolchain. The author can request PDF for one build or enable
it explicitly in project configuration. Proposed author commands:

```sh
zel build my-book --pdf
zel export pdf my-book
zel pdf my-book
```

`pdf` aliases the export path. An explicitly enabled build/CI job generates the
PDF from the same content revision as the website and adds a one-click download
link. A static
website cannot invoke the author's local typesetter on a reader's machine;
server-side on-demand generation is a separate optional later capability.
This delivers a current downloadable book without requiring a server for
every reader. Browser print remains a convenience, not the publication-quality
export contract.

If PDF is disabled, omit its download link. If requested PDF generation fails,
fail the requested build and retain the previous complete output rather than
publish a website linked to an absent, incomplete or stale book. Site and PDF
must identify the same content revision. The standalone export command may
generate a PDF without rebuilding or publishing the website.

### LaTeX quality with template-controlled design

The user explicitly requires a first-class LaTeX route for mathematical books,
with a table of contents, index and the richer features of traditional book
typesetting. The earlier open renderer choice is narrowed: prototype a LaTeX
backend first for technical books; HTML paged media remains an optional later
alternative for templates that benefit from it. These are design commitments,
not features available in rc.3 or current source.

Use one semantic book model for web and print: parts, chapters, headings,
paragraphs, equations, theorem/lemma/proof environments, figures, tables,
footnotes, citations, references, index entries and glossary terms. Preserve
stable identifiers and resolve references before output. The web renderer
uses that model with its theme; a print adapter generates LaTeX with the
selected template. Do not attempt to translate arbitrary website CSS into TeX.

The proposed implementation to prototype is a structured document adapter
(potentially using [Pandoc](https://pandoc.org/MANUAL.html#creating-a-pdf))
followed by generated `.tex`, **LuaLaTeX**, and an explicit build driver such
as [latexmk](https://ctan.org/pkg/latexmk). Pandoc can emit LaTeX and supports
LaTeX PDF engines and templates; it does not automatically supply the complete
Zellij book semantics or every index/cross-reference requirement. Verify those
in the adapter and driver. Keep conversion out of rendered Next.js HTML and
never assume arbitrary MDX or section effects have a lossless TeX translation.

The driver must complete the necessary passes for chapter/page references,
table of contents, indexes, bibliographies and glossaries, invoking supported
index/bibliography tools where needed. Missing labels, undefined citations,
missing glyphs and significant layout overflow must produce actionable source
diagnostics, not a success exit with a visibly broken book. Expose the generated
`.tex` and build log for inspection and expert customization.

Templates carry the visual complexity: cover/spine-ready artwork where
supported, distinct part/chapter openings, section-title treatments, running
headers/folios, theorem/example boxes, side notes, drop caps or ornaments,
spacing and illustration placement. Provide simple choices and shared design
tokens for text/heading/math fonts, colors, trim size and chapter style. A
technical template can pair restrained mathematics/body text with expressive
chapter typography; a novel can use a different visual hierarchy. Do not
require authors to maintain a custom LaTeX preamble just to change these.

Evaluate [fontspec](https://ctan.org/pkg/fontspec) for text fonts and
[unicode-math](https://ctan.org/pkg/unicode-math) for compatible OpenType math
fonts. Bundle or pin appropriately licensed fonts for reproducible output;
test glyph coverage, math compatibility, font embedding and Arabic/RTL layouts
rather than assuming a chosen font or engine makes every language work.

Normal authors write Markdown and LaTeX-style math plus semantic book markup.
Support a documented shared math/macro subset for web and print. Advanced raw
TeX can be an explicit print-only extension with an authored web equivalent
or a clear unsupported-content diagnostic; arbitrary packages/macros cannot
silently disappear from the website. Custom local TeX templates are an expert
extension. Downloaded templates must not execute arbitrary install hooks, and
typesetting must not enable shell escape or fetch unpinned dependencies as a
side effect of rendering content. TeX execution needs its own defined process
and filesystem boundary; disabling shell escape alone is not a sandbox.

Keep the TeX/PDF runtime optional and separate from the small Rust binary and
normal images. When PDF is requested, check dependencies before replacing any
output and explain how to install the supported toolchain or select a dedicated
PDF container. Do not silently install Docker, a browser or a TeX distribution.
Pin template/toolchain versions and support offline builds after explicit setup.

Acceptance must include a substantial math book (numbered equations, proofs,
cross-references, index and bibliography), a prose book, and a multilingual/RTL
book. Check semantic correctness and text extraction as well as visual samples:
cover, contents, chapter openings, dense equations, footnotes, tables and index.
Confirm predictable page breaks, working bookmarks/links, complete embedded
fonts, failed-build output preservation, opt-in behavior and the website's
download link. A beautiful short demo alone does not establish book support.

## 5. Installation channels

Six native OS/architecture packages and shell/PowerShell installers exist.
Homebrew is now published; other package-manager channels remain delivery work.

The name `brew install zellij` is already used by the unrelated terminal
multiplexer: [current Homebrew formula](https://formulae.brew.sh/formula/zellij).
Keep our executable `zel`, use clearly qualified package identities, and check
ownership/availability before registry publication. Homebrew is available now:

```sh
brew install mylife-inc/tap/zel
```

The user prefers **`brew install zel`**. This is the target short name, subject
to checking formula-name collisions at publication time. On a fresh machine,
no-setup installation by that name would require acceptance into Homebrew's
main catalog. Its current [core requirements](https://docs.brew.sh/Acceptable-Formulae)
include a stable release, an eligible open-source license and available source;
do not promise this route while planning a potentially sealed source repository.

Homebrew supports [direct installation from an owned tap](https://docs.brew.sh/How-to-Create-and-Maintain-a-Tap)
using **three** components: `owner/tap/formula`. The user has now settled the
namespace: **Shebka** is the company behind Zellij, formerly named MyLife, and
the user owns and will continue using the GitHub account **`mylife-inc`**.
The agreed one-command form is **`brew install mylife-inc/tap/zel`**. Use Shebka
for company-facing branding; retain the established GitHub/distribution
namespace. No account rename or `shebka` tap owner is required. The public tap
is published at `mylife-inc/homebrew-tap`, selecting the rc.5 preview.

The [current tap rules](https://docs.brew.sh/Taps) allow the short command after
one-time tap setup and formula-specific trust. The qualified command above
is the tested installation path; the equivalent explicit setup is:

```sh
brew tap mylife-inc/tap
brew trust --formula mylife-inc/tap/zel
brew install zel
```

For first-time users the fully qualified install is still fewer steps. Verify
the supported Homebrew versions and their trust behavior in actual installation
tests; do not invent a two-component alias or wrap/replace the user's `brew`.
Start with the owned tap and Scoop bucket. WinGet requires manifest submission
and validation in the [community repository](https://learn.microsoft.com/en-us/windows/package-manager/package/repository);
publication timing is not entirely under our control. A scoped npm launcher is
useful for JS users but must download the correct verified native artifact and
document its network/offline behavior. Evaluate Debian/RPM, Nix and Chocolatey
subsequently; no claim of universal registry availability is made. Cargo registry
publication uploads source packages, so defer it until the user explicitly
settles how public source distribution fits the intended later sealing of this
repository; see [Cargo publication](https://doc.rust-lang.org/cargo/reference/publishing.html).
Keep binary package recipes pointed at the existing public release mirror as
well, so installation survives a future source-repository transition.

Generate manifests from one immutable release inventory/checksum source, then
exercise install/upgrade/uninstall for each supported platform. Preserve stable
versus preview channels, shell completions, executable naming and package-manager
ownership during `zel upgrade`. Registry credentials are requested only at the
point they are needed; keep them out of content/templates and chat. A current
signed/checksummed release plus an install-method table is the public source of
truth. Do not mark a channel done merely because a manifest was generated.
Apply the user's pre-1.0 retention policy to channels we control; evaluate
registry version-removal constraints before publishing previews to external
catalogs. A channel that cannot support the requested cleanup may wait for 1.0.

## Current work and next handoff

The company slot merged in [PR #131](https://github.com/mylife-inc/Zellij/pull/131)
as `b4cc9baf0133cf477ea9d128f035a0554c70cad2`. All 16 jobs in
[CI run 43088663509](https://github.com/mylife-inc/Zellij/actions/runs/43088663509)
passed on `7d7febbef4af6254dc30a31df339ea00d65ec2b0`, with the merged tree
unchanged. The increment includes portable-to-hosted conversion, generated CLI
schema, four active documentation pages and regression tests. See
[branding evidence](evidence/zellij-company-branding-2026-11-11.json). Duplicate
merge-push workflows were skipped; no new release was cut or package channel
published. The preceding in-progress snapshot remains in the PR history.

The later rc.4 milestone completed candidate publication, Homebrew and consumer
migration, then retired nine superseded GitHub releases and their tags.
Image retirement is prepared but awaits a package-access check. See
[release evidence](evidence/zellij-rc4-release-2026-11-11.json) and
[retention status](zellij-release-retention-2026-11-11.md). All 20 final-head CI
jobs, 29 release jobs and four public Homebrew acceptance jobs passed. Other
package channels remain planned. At that milestone the next feature was the
template manifest/catalog and native creation path (now released below), followed by the
composer/media-accordion and full book/PDF milestones. Local docs change with
each implementation; public docs must keep an explicit release boundary.
Live deployment-provider verification stays deferred as previously instructed.


## Template milestone released — rc.5, 2026-11-12

The native template path merged in PR #138 (`a92c2e9`) and is released in rc.5:
`zel create` / `zel new site`, `zel templates [show]`, three bundled starters,
local directories, immutable release archives and checksummed remote catalogs,
exact version selection, offline cache and creation receipts. The source is
`templates/` with the format documented in `templates/README.md`. No dependency
installation or template hooks run during creation. Existing commands keep their
semantics. Creation refuses nonempty destinations and the current working
directory; run from its parent to populate an existing empty folder.

The first catalog is deliberately small: `product-docs`, `product-studio` and
`book-classic` at template version `0.1.0`. The product starter composes existing
Cupertino sections; the book starter is a portable reading edition with no print
renderer. Technical/study editions, richer reading controls, the composer/new
sections and full PDF/LaTeX publishing remain planned. Catalogs require compatible
CLI/engine versions; they do not execute source-framework import logic.

All 20 final-head CI jobs and 29 release/public-artifact jobs passed. Six hosted
previews were reviewed at desktop/mobile widths, including every authored product
card body. Source and public mirror publish identical catalogs/archives; installed
binaries verify bundled/downloaded/offline parity. Homebrew passed all four native
checks. See [release evidence](evidence/zellij-rc5-release-2026-11-12.json).

Review exposed a general authoring gap: unknown engine section fields can be
silently stripped. Official-template acceptance now detects this, but arbitrary
user sections still need strict-check diagnostics before the composer expands.
Local guides and public installer docs describe rc.5; the existing production
documentation site and domain have not been redeployed.


## Section diagnostics and recovered headings — 2026-11-12 source

The post-rc.5 source increment reports discarded section keys instead of hiding
misspellings. Ordinary checks/builds warn; strict checks fail. It covers referenced
sections (YAML and Markdown front matter), examples, legacy inline sections and
page-reference overrides, including nested arrays/objects. Values are omitted
from warnings. Retained schema-extension fields, defaults and scalar coercions
remain accepted; intentionally removed object keys in custom transforms warn.

This exposed thirteen pre-existing omissions in the CodeSeal example. Eleven
section eyebrows and an FAQ introduction are now connected to the existing
shared header; a comma-bearing hero label is quoted correctly. Feature-grid,
steps-panel and FAQ schemas accept eyebrow; FAQ accepts inline-Markdown subhead,
including deep link validation. Native authoring metadata and active docs match.
Two schema-to-render regressions reproduce the omissions before the fixes; five
check regressions reproduce silent acceptance on the unmodified rc.5 baseline.
Local verification passes 471 engine tests, 12 native authoring tests, all 23
section-authoring cases, three site templates and deep/strict checks of four
sites. Final hosted acceptance and merge remain pending. Published rc.5 is
unchanged; no new release or provider/production-docs deployment is implied.


The first hosted run (`43192044415`, PR #141) exposed four more ignored fields in
the native starter shared by ordinary and full creation. Correct its feature-grid
`title` to `heading` and three item `description` fields to `body`, preserving all
text. The existing full-starter strict gate reproduced the failure; SSR and
hosted export acceptance now assert the actual heading and card copy. Focused
verification passes 35 scaffold tests, four full-starter tests, nine site-authoring
tests and all 40 native/ten Node starter and ten guide-title rendering fixtures.
The failure is retained as evidence; a fresh final-head CI run is required.


Final acceptance: PR #141 merged as `3008a164774460cf6a0c2f449ba793d1dab3e8c4` with the exact tree tested at
`43295fcfb2135ba114be9de17a3a2b8fe417bd4b`. All 20 jobs in `43193011736` passed,
including native packages, schema consistency, real CLI/container checks,
full-starter export assertions, template and browser acceptance. Documentation
preservation retains 46 original routes, 282 heading IDs and 29 unchanged image
files, alongside ten added routes, with no captured browser errors. The earlier
pending snapshots above are superseded by this result; the initial failure is
retained in [the evidence](evidence/zellij-section-field-diagnostics-2026-11-12.json).
Published rc.5 and production documentation/domain are unchanged. The next
implementation milestone is the composer/media-accordion, followed by book/PDF.

## Composer integration notes — 2026-11-12 review

Before implementing `type: compose`, extend or deliberately bound the shared
schema traversal. Current engine context/starter generation and reference
collection walk inline objects, arrays and unions; they do not resolve JSON
Schema `$ref`/`$defs`. The Rust schema/authoring walkers have the same boundary.
A recursive composition schema must not silently omit nested asset/link checks,
produce incomplete context, or create invalid starters. Choose an explicit
reference-resolution contract with cycle/depth bounds (or a measured finite
expansion), and verify the same authored tree through both engines before
publishing a catalog entry. This is a prerequisite, not an implemented feature.

Relevant integration points are `engine/packages/zellij/src/cli/context.ts`,
`cli/section-templates.ts`, `content/reference-schema.ts` (the latter two under
that same engine source root), `cli/crates/zellij-core/src/schema.rs`, and
`cli/crates/zellij-scaffold/src/authoring.rs`.

Heading ownership also needs an explicit decision: `SectionsPageView` currently
asks whether a section *type* always provides an H1. A composer may contain an H1
or only subordinate headings, so inspect the validated instance or expose an
equivalent instance-aware contract. Preserve the fallback page heading and
meaningful page-rail labels. The current search index intentionally indexes
guides only; adding composition does not implicitly change that product rule.


## Bounded composer v1 — 2026-11-12 implementation checkpoint

The reference-resolution prerequisite above is settled for the first slice by
finite schema expansion: one group branch per level, three nested groups, and
14,701 bytes of compact exported JSON. There are no `$ref`/`$defs`; the existing
engine and Rust traversals consume the same nested definitions. Total node and
primary-heading limits live in exported authoring rules. Native `explain` names
every top-level node/layout alternative, generated context expands array unions,
and complete JSON edits plus scalar refinements author nested structures.

Implemented: stack/grid/split; heading, inline-Markdown text, image/caption,
button, stat, divider and group; surface/padding/radius, theme-aware typography,
entrance reveal and heading gradient reveal. The latter blends between 15% and
65% visibility, with readable solid text beneath it. Shared section gradients
supply heading fallbacks. Layouts collapse below 760px without changing DOM
order. Instance-aware registry heading ownership avoids a duplicate fallback H1.

This is not the entire proposed universal section. Layers, sticky stages, block
Markdown, video/icon/embedded-section nodes, authored effect ranges, named
composition presets and interactive media-accordion/selector/story/explorer
patterns remain follow-ups. Existing adjacent sections remain available.
No book/PDF or portable section renderer is implied.

The showcase recreates a hero, split story and feature grid with the same type,
original copy and an original SVG. Hosted browser tests cover responsive layout,
images/links, gradient visibility, reduced motion, forced colours, print, missing
observers and disabled JavaScript; screenshots are retained for visual review.
Local checks pass 481 engine tests, workspace types, 14 native authoring tests,
all 24 authored section types through the real engine and full-starter generation.
The Rust workspace passes 743 tests with five existing ignored cases; Clippy passes.
Published rc.5 remains unchanged; CI, visual review and merge are pending.
