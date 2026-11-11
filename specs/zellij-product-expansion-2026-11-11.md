# Product expansion: branding, composition, templates, books and installation

Date: 2026-11-11. Source baseline: `3ca63ba` on `main`; latest published CLI:
`v0.2.0-rc.3`. These are new requirements beyond the completed earlier CLI
contract. This document records the user's intentions and a proposed staged
design; proposed commands below are not yet implemented or published.

## Delivery order and release boundaries

| Order | Deliverable | Timing and acceptance |
| --- | --- | --- |
| 1 | Separate company and product branding | Implement now, in portable and hosted renderers. Include with the merged hydration fix in the next candidate, provisionally rc.4. Check mobile layout, direct-file relocation, destination/tab selection, dark variants and existing configurations. |
| 2 | Package-manager distribution | Prepare alongside the next candidate. Start with an owned Homebrew tap and Scoop bucket; submit WinGet after the intended release is available. Test install, upgrade and uninstall on supported native architectures. Stable channels must not silently install release candidates. |
| 3 | Template catalog and creation command | Next feature cycle. Establish a versioned format and create complete, editable sites with existing sections. First examples: product documentation, a product landing site, and a reading-oriented book site. |
| 4 | Composition engine and additional Apple-inspired patterns | Build a bounded composition format, reusable presentation/effect primitives and named presets. Extend the product template using these; retain all existing section types and authored content. |
| 5 | Complete book publishing and PDF | Extend the book template into a publication model, dedicated print renderer and version-matched downloadable PDF. Validate technical, prose and multilingual books before calling this milestone complete. |

Do not delay the next small CLI candidate until the entire expansion is done.
Do not promise dates or a final stable-version number until each gate passes.
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
home. This is implemented in source, newer than published rc.3:

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
  product-studio/
    template.yaml
    preview.webp
    files/
      zel.yaml
      content/
  book-classic/
  book-technical/
  book-study/
```

Generated site layout remains `my-docs/zel.yaml`, `my-docs/content/` and generated
`my-docs/site/`. The enclosing folder can have any name. A manifest declares
stable ID/version, title, preview, minimum engine/schema versions, renderer,
optional features, supported outputs, editable variables and asset licenses.

Proposed canonical creation command and short entry point:

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

Proposed author commands:

```sh
zel build my-book --pdf
zel export pdf my-book
zel pdf my-book
```

`pdf` aliases the export path. A build/CI job generates the PDF from the same
content revision as the website and adds a one-click download link. A static
website cannot invoke the author's local typesetter on a reader's machine;
server-side on-demand generation is a separate optional later capability.
This delivers a current downloadable book without requiring a server for
every reader. Browser print remains a convenience, not the publication-quality
export contract.

Prototype a maintained paged-media renderer rather than invent pagination.
[Vivliostyle CLI](https://docs.vivliostyle.org/publications/cli-en/en/cli/getting-started/)
is a candidate because it accepts HTML/Markdown/publication inputs and provides
PDF output with cover/contents tooling. Renderer choice remains open until
math, footnotes, long tables, images, cross-references, RTL and font embedding
are verified on representative books. Keep its large runtime optional and
separate from the Rust binary and normal lightweight images. Use an explicit
dependency check with actionable setup instructions; don't silently install
Docker or a browser. A dedicated PDF image may be an optional alternative.

## 5. Installation channels

Six native OS/architecture packages and shell/PowerShell installers already
exist. Package-manager registration and maintenance are new delivery work.

The name `brew install zellij` is already used by the unrelated terminal
multiplexer: [current Homebrew formula](https://formulae.brew.sh/formula/zellij).
Keep our executable `zel`, use clearly qualified package identities, and check
ownership/availability before registry publication. Proposed examples, not
currently available commands:

```sh
brew install mylife-inc/tap/zel
winget install --id MyLifeInc.Zel --exact
npm install -g @mylife-inc/zel
```

Homebrew supports [direct installation from an owned tap](https://docs.brew.sh/How-to-Create-and-Maintain-a-Tap).
Start with that and an owned Scoop bucket. WinGet requires manifest submission
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

The remaining rows are designed backlog, not completed features. The next
delivery task is the small candidate containing branding and the hydration fix,
with package-manager preparation alongside it. The next feature implementation
is the template manifest/catalog and native creation path, followed by the
composer/media-accordion and full book/PDF milestones. Local docs change with
each implementation; public docs must keep an explicit release boundary.
Live deployment-provider verification stays deferred as previously instructed.
