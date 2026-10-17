# @shebka/zellij

A shared Next.js engine that turns a folder of structured content (YAML + MDX + assets) into a complete product website — marketing pages and user-guide documentation, one site, one deployment.

The name comes from Moroccan *zellij* mosaic: intricate walls composed from a fixed vocabulary of geometric tiles. Zellij works the same way — polished sites composed from a fixed vocabulary of **sections** ("tiles"), styled by **themes** ("glazes").

**Zellij is not a code generator.** It is a versioned package a thin per-product wrapper imports. All rendering lives here; products upgrade by bumping a dependency version.

---

## Three sites in this repo

Zellij is developed against real content, not fixtures.

| App | What it is | Port |
| --- | --- | --- |
| `examples/showcase` | The visual test bed and design showpiece — every section, theme and layout | 4300 |
| `examples/zellij-docs` | **Zellij's own documentation, built with Zellij** — two guide trees plus a marketing page set | 4320 |
| `examples/payos` | A real product's documentation, converted from MkDocs sources | 4310 |

```bash
npm run dev        # the demo
npm run dev:docs   # these docs
npm run check      # validate all three content roots
```

## Contents

- [Quick start](#quick-start)
- [Content model](#content-model)
- [Config reference](#config-reference)
- [Page layouts](#page-layouts)
- [Section reference](#section-reference)
- [Guide docs & MDX components](#guide-docs--mdx-components)
- [Themes](#themes)
- [Motion](#motion)
- [Search](#search)
- [Deploying under a path prefix](#deploying-under-a-path-prefix)
- [Build errors](#build-errors)
- [Implementation notes](#implementation-notes)

---

## Quick start

### 1. Install

```bash
npm install @shebka/zellij next@~16.4.2 react@19.2.x react-dom@19.2.x
```

Use Node 20.19+ on the 20.x line, or Node 22.12+. Development uses Node 24 (`.nvmrc`).

The `zellij init` scaffold copies the engine manifest's dependency `overrides`
into the app. Keep them when editing a generated app: npm only applies
overrides from the root manifest. For a manually assembled wrapper, copy that
map from `engine/packages/zellij/package.json` into the app's `package.json`
before installing. It currently keeps the optional math/diagram dependencies
on patched KaTeX and XML parser versions.

### 2. The wrapper app

This is the whole app — three files, under 30 lines.

`src/app/layout.tsx`

```tsx
import '@shebka/zellij/styles.css';
import { createZellijRootLayout } from '@shebka/zellij';

export default createZellijRootLayout();
```

`src/app/[[...slug]]/page.tsx`

```tsx
import { createZellijPages } from '@shebka/zellij';

const zellij = createZellijPages();

export const generateStaticParams = zellij.generateStaticParams;
export const generateMetadata = zellij.generateMetadata;
export default zellij.Page;
```

> `Page` is returned by name rather than as `default`. Destructuring a key called `default` does not create a module default export, which the App Router requires.

`next.config.js`

```js
/** @type {import('next').NextConfig} */
export default {
  transpilePackages: ['@shebka/zellij'],
};
```

Optionally, `app/sitemap.ts` and `app/robots.ts`:

```ts
import { createZellijSitemap } from '@shebka/zellij';
export default createZellijSitemap();
```

```ts
import { createZellijRobots } from '@shebka/zellij';
export default createZellijRobots();
```

### 3. Build steps

Zellij reads content at build time and needs two prebuild steps: assets are copied
into `public/` so `next/image` can optimize them, and the guide search index is
generated.

```json
{
  "scripts": {
    "dev": "zellij dev",
    "prebuild": "zellij prepare",
    "build": "next build"
  }
}
```

| Command | Does |
| --- | --- |
| `zellij prepare` | `sync-assets` and `search-index` together — the usual choice |
| `zellij dev -- --port 3000` | Prepares content, watches edits, and runs the installed Next dev server |
| `zellij prepare --watch` | Watches preparation in a separate process when you own the server command |
| `zellij sync-assets` | Copies `<content>/assets/**` → `public/_zellij/assets/**` |
| `zellij search-index` | Writes `public/_zellij/search-index.json` |
| `zellij check [dir] [--deep] [--strict]` | Fast content validation; optionally parse MDX and check static references, or fail on warnings |

The preparation commands accept `--content <directory>` (absolute or relative
to the wrapper root) and `--config <legacy-config>`. Set `--content` to the same
folder passed as `contentDir` to the rendering factories; for example,
`zellij prepare --content ../../docs/external`. Without it, they use `./content`
or the legacy configuration's content path. Preparation reads fresh content,
uses canonical guide routes and heading IDs, and removes stale generated
assets even when the source assets folder has been removed. `prepare` alone is
one-shot. `dev` and `prepare --watch` poll for edits, additions and deletions,
including on container mounts, and update assets and search after a short debounce.
Invalid edits are reported; watching resumes when the content is corrected.
Reload the browser to see page/image changes; reopening search fetches its new
index in development. There is no automatic browser reload for content edits.
Restart dev after changing the deployment prefix or wrapper configuration.
Pass Next options after `--`, for example
`zellij dev --content ../../docs/external -- --port 4000 --hostname 127.0.0.1`.

`check --deep` checks static prose/section links, heading anchors, MDX syntax,
section previews and schema-declared local media. `--strict` fails on warnings,
including computed MDX references that cannot be verified without execution.
Neither mode evaluates authored JavaScript or checks remote URLs. Literal menu
`url:` destinations remain unchecked. Use the installed wrapper's optional math
parser for math-enabled content. Keep a production build and browser checks;
static analysis cannot verify arbitrary component execution or wrapper-owned
public resources. The Rust equivalent is `zel check <dir> --deep --strict`.

### 4. Content

The content root is a folder of any name that describes itself entirely. Every
file in it is optional — a folder holding nothing but one guide folder builds a
complete site.

```
content/
├── site.yaml               # identity, brand, theme, footer
├── menu.yaml               # the top bar's items
├── pages.yaml              # pages, each an ordered list of section names
├── sections/               # one file per section: .yaml, .md or .mdx
│   ├── hero-main.yaml
│   └── why-us.mdx
├── assets/
└── 01-guide/               # a guide folder: any name, holds _sidebar.yaml
    ├── _sidebar.yaml
    └── quickstart.mdx
```

`npm run dev`, and the site is live. `zellij check` validates the folder without
building anything.

---

## Content model

| Source | Route |
| --- | --- |
| the home destination | `/` |
| `pages.yaml` → `- name: features` | `/features` |
| `pages.yaml` → `- name: solutions/startups` | `/solutions/startups` |
| `01-guide/index.mdx` | `/guide` — the ordering prefix is stripped |
| `01-guide/quickstart.mdx` | `/guide/quickstart` |
| `01-guide/config/database.mdx` | `/guide/config/database` |

Any page may override its route with `route: /custom-path`. Two destinations
resolving to the same route is a build error, as is a page and a guide folder
sharing a name.

**What "/" shows:** `home:` in `site.yaml` if set, else a page that declares
`route: /`, else the first menu item that resolves to a page, else the first
page or guide folder. A guide folder at `/` redirects there rather than moving
its pages.

### Pages

```yaml
# pages.yaml
- name: features
  title: "Features — Acme"       # optional; <title> and OG. Defaults from the name
  description: "..."             # optional; meta description
  route: /features               # optional; defaults to /<name>
  layout: stack                  # optional; see Page layouts
  theme: midnight                # optional; per-page theme override
  motion: expressive             # optional; per-page motion override
  navHighlight: features         # optional; which nav item shows as active
  sections:
    - hero-main                                    # → sections/hero-main.yaml
    - { section: features, background: tinted }    # with a presentation override
```

### Sections

One file per section, under `sections/`. Its name is its path below that
directory with the extension dropped, so `sections/home/intro.yaml` is
`home/intro`.

A `.yaml` file declares a `type` plus that type's fields. A `.md`/`.mdx` file is
a prose section (`markdown-prose`) whose frontmatter may set presentation or a
different `type`.

```yaml
# sections/hero-main.yaml
type: hero
headline: "Ship faster"
cta:
  - { label: "Get started", href: /guide/quickstart, style: primary }
```

Because sections are files, a page can use the same one twice, and two pages can
share one. **A reference may override presentation, never content** —
`background` (`default`, `tinted`, `inverted`, `accent`, or a `gradient:` of two
or three stops), `id`, `motion`, `padding`. Anything else stays in the section
file, which is the one place its content lives. A section used twice on a page
needs a distinct `id` on the second, so its anchor stays unique.

### The menu

`menu.yaml` declares the bar's items and nothing else — search, the theme picker
and the colour toggle are built in and configured under `nav:` in `site.yaml`.

```yaml
- { name: Features, page: features }              # a page or guide folder by name
- { name: Security, url: /features#security }     # any literal href
- name: Solutions                                 # a dropdown
  items:
    - { name: For startups, page: solutions/startups }
- name: Resources                                 # a child with its own items
  items:                                          # renders as a full-width panel
    - name: Learn
      items:
        - { name: Guide, page: guide, description: "Install and operate.", icon: book }
        - { name: Blog,  url: "https://blog.example.com" }
```

Each item declares exactly one of `page:`, `url:` or `items:`. A `page:` that
names nothing fails the build; a `url:` is never validated. An external `url:`
gets an outbound icon and opens in a new tab (`newTab: false` to override).

**With no `menu.yaml`**, one is derived: every entry in `pages.yaml` in order,
then any guide folder not already named — the union, so nothing on the site
loses its way in. Past `menuOverflowAfter` items (default 7) the remainder
collapses into one trailing drawer, so a book with thirty chapters still gets a
usable bar with no configuration.

---

## Config reference

`site.yaml` lives **inside** the content root, so pointing a build at a folder —
including one just cloned from a repo — is enough on its own. Where the folder
sits and where the app is mounted belong to the wrapper and are passed to
`createZellijPages({ contentDir, basePath })`.

```yaml
name: Acme                         # required; defaults to the folder's own name
tagline: "..."                     # optional

brand:
  logo:
    src: assets/logo.svg
    dark: assets/logo-dark.svg     # optional
    alt: Acme
    href: https://shebka.com       # optional; omit and the mark links to "/"
  title: Acme                      # optional; shown beside a mark that leads elsewhere

theme: cupertino                   # see Themes
mode: system                       # light | dark | system
motion: subtle                     # none | subtle | expressive

home: home                         # optional; default is the first menu item
menuOverflowAfter: 7               # optional; derived menus collapse past this

basePath: ""                       # withZellij reads this for Next; rebuild after changes
siteUrl: https://acme.example.com  # required for absolute OG/canonical URLs and sitemap
ogImage: assets/og.png             # optional social share image

overrides:                         # optional; tweak a theme without forking it
  accent: "#E86A33"
  accentForeground: "#ffffff"
  radius: lg                       # none | sm | md | lg | xl
  fontHeading: "Georgia, serif"
  fontBody: "system-ui, sans-serif"

nav:                               # how the bar behaves; menu.yaml says what is in it
  sticky: true                     # default true
  align: left                      # left | center
  hideOnScrollDown: false
  transparentOverHero: true        # applies to `layout: landing` pages
  scrollThreshold: 80
  themeToggle: true                # light/dark toggle
  themeSwitcher: true              # theme picker — see Themes
  search: true                     # enables the ⌘K modal
  cta:
    - { label: "Get started", href: /guide/quickstart, style: primary }
  social:
    github: "https://github.com/acme/acme"
  announcement:                    # optional bar above the nav
    text: "Acme 2.0 is live —"
    link: { label: "read the notes", href: /changelog }
    dismissible: true

footer:
  columns:
    - heading: Product
      links:
        - { label: Features, page: features }    # `page:` or `href:`
  legal: "© 2026 Acme"
  social: { github: "..." }

analytics:
  provider: none                   # plausible | none
  domain: acme.example.com
```

### The brand block, and sites that share an owner

`brand.logo.href` is what lets a family of sites belong to one owner. Point it at
the hub and the mark leaves for the hub, while `brand.title` becomes the way home
within this site — the pattern Apple and AWS both use. Omit it and the logo links
to `/`, which is the single-site case; `brand.title` is then unnecessary.

This needs no session state and no query parameters, and it works for a visitor
who arrived from a search result rather than from the hub.

Unknown top-level keys produce a warning rather than an error, so a newer config can be read by an older engine.

---

## Page layouts

A closed set, deliberately — arbitrary per-product layouts would reintroduce the visual drift the engine exists to prevent.

| `layout` | Use for |
| --- | --- |
| `stack` (default) | The standard full-width marketing page |
| `stack-narrow` | Text-led pages — About, Security, Changelog |
| `stack-sidebar` | Long feature pages; adds a sticky "On this page" rail built from section `id`s |
| `landing` | `stack` plus a nav that starts transparent over the hero and solidifies on scroll |

Any section may declare `id: pricing`, making `/features#pricing` a valid nav or CTA destination. Anchor scrolling accounts for the sticky bar automatically.

---

## Section reference

Every section accepts the presentation fields a page reference may also
override: `id` (anchor target), `motion` (`none` | `subtle` | `expressive`),
`background`, `padding`, and `titleGradient`.

`background` is `default`, `tinted`, `inverted`, `accent`, or a gradient:
`background: { gradient: [accent, "#101018"] }`. `titleGradient` takes the same
stops and paints the section's leading heading with them. Stops that name a
palette token (`accent`, `surface`, `inverted-bg`, …) follow the active theme;
literal colours do not.

### hero

```yaml
- type: hero
  badge: "Acme 2.0 is here"          # optional
  headline: "Required headline"
  subhead: "Optional."
  cta:                               # up to 2
    - { label: "Get started", href: /guide, style: primary }
    - { label: "See features", href: /features, style: ghost }
  visual: assets/hero.png
  visualStyle: floating              # floating | framed | full-bleed
```

### logo-strip

```yaml
- type: logo-strip
  heading: "Trusted by teams at"
  logos: [assets/logos/a.svg, assets/logos/b.svg]
```

### feature-grid

```yaml
- type: feature-grid
  heading: "Everything you need"
  subhead: "Optional."
  columns: 3                         # 2 | 3 | 4
  items:
    - icon: shield-check             # lucide name, kebab-case; unknown names fail the build
      title: "Compliance built in"
      body: "Supports **inline markdown**."
      href: /features                # optional, makes the card a link
```

### zigzag

```yaml
- type: zigzag
  heading: "Optional"
  rows:
    - title: "See every process"
      body: "Inline markdown allowed."
      image: assets/shot.png
      align: image-right             # image-right | image-left
      cta: { label: "Learn more", href: /features }
```

### showcase

```yaml
- type: showcase
  eyebrow: "The platform"
  headline: "Required"
  body: "Optional."
  visual: assets/showcase.png
  background: tinted                 # default | tinted | inverted
  cta: { label: "Read the guide", href: /guide, style: secondary }
```

Under `motion: expressive` this becomes a sticky-scroll panel: the visual pins while the copy scrolls past.

### stats-band

```yaml
- type: stats-band
  heading: "Optional"
  stats:                             # 3–5
    - { value: "40%", label: "Less manual review", prefix: "", suffix: "" }
```

Numeric values count up under `motion: expressive`.

### testimonial

```yaml
- type: testimonial
  heading: "Optional"
  quotes:                            # 1–5; a single quote renders without carousel chrome
    - quote: "We replaced four tools."
      author: "Dana Whitfield"
      role: "VP Operations"
      avatar: assets/avatars/dana.png
```

### pricing-table

```yaml
- type: pricing-table
  heading: "Plans"
  subhead: "Optional."
  tiers:                             # 1–4; a single "Contact us" tier renders centred
    - name: Team
      price: "$24"
      period: "per user / month"
      description: "Optional."
      features: ["Unlimited workflows"]
      highlighted: true              # at most one
      cta: { label: "Start trial", href: /guide, style: primary }
```

### faq

```yaml
- type: faq
  heading: "Common questions"
  items:
    - question: "Can we self-host?"
      answer: "Yes — inline markdown allowed."
```

Built on `<details>`, so answers are keyboard-operable and present for crawlers without JavaScript.

### cta-banner

```yaml
- type: cta-banner
  headline: "Ready to ship?"
  body: "Optional."
  background: tinted                 # default | tinted | inverted
  cta:                               # a single CTA or a list of up to 2
    - { label: "Get started", href: /guide, style: primary }
```

### markdown-prose

```yaml
- type: markdown-prose
  heading: "Optional"
  body: |
    Free-form prose inside marketing styling. Uses the same MDX pipeline as the
    guide, so `<Callout>` and friends work here too.
```

### video-embed

```yaml
- type: video-embed
  heading: "Optional"
  src: assets/demo.mp4               # or a YouTube/Vimeo URL
  poster: assets/poster.png
  caption: "Optional."
```

Nothing loads until play is pressed — no iframe, no `<video>` — so third-party media stays off the critical path.

### media-panel

A rounded panel filled edge to edge with one image or video, copy laid over it.

```yaml
- type: media-panel
  heading: "Take a closer look."     # optional, above the panel
  title: "A guided tour"             # copy inside it
  body: "Twelve minutes, no signup."
  image: assets/tour.png             # exactly one of image | video
  # video: assets/tour.mp4           # muted, looping, still under reduced motion
  # poster: assets/tour-poster.png
  alt: ""
  align: left                        # left | center | right
  tone: dark                         # light | dark | auto — keys the copy colour
  ratio: ultrawide                   # ultrawide | wide | square | portrait
  scrim: 0.45                        # 0–1, darkens the media behind the copy
  cta: [{ label: "Watch", href: /guide }]
  # href: /tour                      # makes the whole panel a link (drops cta)
```

The panel grows past its ratio when the copy needs more room, and drops to a
portrait frame with full-width copy on a phone.

### card-carousel

A horizontally scrolling row of cards, snapped, with the next one peeking in.

```yaml
- type: card-carousel
  heading: "Why teams pick Acme"
  link: { label: "Browse the guide", href: /guide }   # trailing link on the heading row
  cardWidth: medium                                    # narrow | medium | wide
  items:
    - eyebrow: Workflows
      title: "Route work the way your team already works."
      body: "Branches, parallel steps and conditions."
      image: assets/card-workflows.png
      href: /features#workflows
      background: tinted                               # per card
```

Scrolling is the browser's own, so a trackpad, a touch drag and keyboard
scrolling all work; the arrows are a convenience and hide when nothing overflows.

### bento-grid

An asymmetric tile grid — `feature-grid` weights every item equally, this one
does not.

```yaml
- type: bento-grid
  heading: "One platform, not five tools"
  items:
    - title: "Automations that read like sentences"
      body: "When a request is approved, open the fulfilment task."
      image: assets/bento-automation.png
      span: large                    # normal | wide | tall | large
      href: /features#workflows
    - title: "Roles and rules"
      icon: shield
      span: normal
      background: tinted
```

Four columns on desktop, two on a tablet, one on a phone.

### timeline

A dated sequence: history, milestones, a roadmap, the chapters of a book.

```yaml
- type: timeline
  heading: "The story so far"
  items:
    - date: 2019                     # free text; a bare year is fine
      title: "A spreadsheet that would not die"
      body: "Acme started as an internal tool."
      icon: sprout
      image: assets/first.png
```

### team-grid

People: a board, a faculty, a team, a list of contributors.

```yaml
- type: team-grid
  heading: "Who builds it"
  columns: 3                         # 2 | 3 | 4
  people:
    - name: Ada Okonjo
      role: Engineering
      bio: "Built the workflow engine twice."
      photo: assets/team/ada.png
      links: [{ label: Writing, href: /about }]
```

Each link is announced with the person's name, so a page of "Website" links
stays usable when tabbed through.

### gallery

A grid of images, each openable full size.

```yaml
- type: gallery
  heading: "Around the place"
  columns: 3                         # 2 | 3 | 4
  lightbox: true                     # false for a plain grid
  items:
    - image: assets/gallery/office.png
      alt: The Lisbon office
      caption: Lisbon, 2024
```

The viewer is a native `<dialog>`, so Escape, the focus trap and the inert
backdrop come from the platform; arrow keys page between images.

> Write list items in block style rather than `{ a: 1, b: 2 }`. An unquoted
> comma ends an entry inside a flow mapping, so `caption: Lisbon, 2024` quietly
> becomes `caption: Lisbon`.

---

## Guide docs & MDX components

`guide/_sidebar.yaml` defines nav order, which also drives previous/next links
and breadcrumbs. Groups nest to any depth, and every group is collapsible:

```yaml
- label: Getting Started
  items:
    - quickstart                     # → guide/quickstart.mdx
    - installation
- label: Configuration
  items:
    - config/overview
    - label: Data                    # a group inside a group
      items:
        - config/database
```

Groups start expanded. A reader's collapsed groups are remembered, but the group
containing the current page is always opened — otherwise following a link into a
collapsed branch would land on a page invisible in the navigation.

A sidebar entry with no matching file is a build **error**. A file missing from the sidebar is a **warning** — the page still builds. A group whose entries all fail to resolve is omitted rather than rendered empty.

These components are available in every guide file without imports:

```mdx
<Callout type="info|tip|warning|danger" title="Optional">Body.</Callout>

<Steps>
  <Step title="Install">Numbering is derived from order.</Step>
</Steps>

<Tabs>
  <Tab label="CLI">…</Tab>
</Tabs>

<CodeGroup>
```bash title="docker"
docker run acme
```
```yaml title="kubernetes"
kind: Deployment
```
</CodeGroup>

<CardGrid>
  <Card title="Database" href="/guide/config/database" icon="database">Optional body.</Card>
</CardGrid>

<Screenshot src="assets/shot.png" caption="Optional." />

<SectionPreview example="hero" />        <!-- source and rendered output, in tabs -->

<Theorem kind="theorem" number="1" title="Cauchy–Schwarz">…</Theorem>
<Proof>…</Proof>
```

### Mathematics, diagrams and mindmaps

| Written as | Rendered | Costs |
| --- | --- | --- |
| `$x$` and `$$x$$` | MathJax SVG, at build time | ~50 MB to install, nothing shipped |
| ```` ```mermaid ```` | Mermaid, in the browser | ~100 MB to install, ~10 MB shipped |
| ```` ```mindmap ```` | SVG, at build time | nothing — built in |

Maths and Mermaid are **optional dependencies**. A site that does not ask for
them never installs them, and the engine renders without them rather than
failing over a package nobody wanted — `$…$` stays as written, and a mermaid
fence renders as its source. Turn them on with `zel build --features
maths,diagrams`, or by choosing the container image that has them.

A mindmap is YAML, and is laid out during the build rather than in the browser:

````mdx
```mindmap
Zellij:
  Content: [site.yaml, menu.yaml]
  Output: Static HTML
```
````

A mapping is a node and its children, a list is siblings, a bare string is a
leaf. Because the layout is arithmetic rather than rendering, no library reaches
the browser and the colours are custom properties — so the drawing follows a
theme change the way text does, instead of needing to be drawn again.

Code blocks are highlighted at build time by Shiki, with a copy button, filename labels via ```` ```ts title="next.config.js" ````, and line highlighting via ```` ```ts {2,4-6} ````. Both light and dark themes are emitted and selected by CSS variable, so the theme toggle needs no re-highlighting and **no highlighter ships to the browser**.

---

## Themes

Set `theme:` in config. Fourteen are built in:

| Theme | Personality |
| --- | --- |
| `midnight` | Dark, electric accent, tight geometry |
| `paper` | Light, warm, editorial serif headings |
| `terracotta` | Warm Moroccan palette, generous curves |
| `mono` | Austere black/white, brutalist hard-offset shadows |
| `aurora` | Dark, soft gradient glows, glassy surfaces |
| `slate` | Cool corporate blue-grey, enterprise-safe |
| `forest` | Deep greens, natural and calm |
| `sunburst` | Light, high-energy, playful radius |
| `carbon` | Near-black developer tool, monospace headings |
| `porcelain` | Ultra-minimal, hairline borders, quiet luxury |
| `cupertino` | Very large tight headlines in the system (SF) stack, centred copy, pill buttons, deep vertical silence |
| `studio` | Editorial creative studio — warm paper, monochrome type, a luminous violet accent |
| `iris` | Bright enterprise SaaS — near-white neutrals, roomy rounded surfaces, a violet brand |
| `meridian` | Warm cream over deep navy ink with an electric blue — calm, human, trustworthy |

Like `cupertino`, the last three carry neutral names: they capture a design
language taken from a spec, not a brand.

Themes differ structurally, not just by palette: font pairing, radius scale, shadow character, display scale, letter-spacing, and motion personality all vary. `mono` moves in 220 ms; `aurora` in 760 ms.

Only the **active** theme's tokens are emitted into the page. Theme fonts are pinned local WOFF2 assets, so development and production builds do not contact Google Fonts. Every theme's `@font-face` rules are present with their Unicode subsets, and the stylesheet references a bundled license notice file. Fonts are not preloaded: browsers download only the families and glyph subsets in use. `cupertino` prefers the system stack on Apple platforms and falls back to Inter elsewhere. See [font provenance and update instructions](src/themes/fonts/README.md).

To adjust without forking a theme, use `overrides` (see [Config reference](#config-reference)).

### Letting visitors choose

Set `nav.themeSwitcher: true` to add a picker to the bar. Selecting a theme
restyles the whole site immediately — colours, typography, geometry and motion —
and the choice is remembered.

It is **opt-in** because it changes what ships. Normally only the active theme's
tokens are emitted; a runtime switcher needs every theme's tokens in the page,
which costs roughly 19KB of inline CSS (about 4KB over the wire). Sites that
pick a theme at build time pay none of that.

Switching is a single attribute write on `<html>` — no re-render, no request —
because every theme's rules are already present, scoped by name. The same
scoping paints each swatch in the picker with the theme it represents, so the
preview cannot drift from the real thing.

---

## Motion

`motion: none | subtle | expressive` in config, overridable per page and per section.

| Preset | Behaviour |
| --- | --- |
| `none` | No animation |
| `subtle` (default) | Entrance reveals with sibling stagger |
| `expressive` | Adds sticky-scroll showcase, hero parallax, and count-up stats |

**`prefers-reduced-motion` collapses any preset to `none`**, regardless of configuration. This is enforced in one place (`useResolvedMotion`) so it cannot be forgotten. All motion is transform/opacity only — never layout-affecting — so it cannot contribute to CLS. Sticky pinning also stands down on short and touch-sized viewports.

Guide pages use no scroll animation; motion is a marketing-layout concern.

---

## Search

Set `nav.search: true`. `zellij search-index` writes a static JSON index at build; the ⌘K modal fetches it and loads FlexSearch **on first open**, so a visitor who never searches downloads neither. Guide content only — sections pages are excluded by design. Results are split per heading, so a hit lands on the relevant section.

---

## Deploying under a path prefix

Set the prefix once in `content/site.yaml`:

```yaml
basePath: "/products/acme/docs"
```

The generated `withZellij()` Next config reads that value and supplies the same
build-time prefix to rendering. For custom content, use
`withZellij({}, { contentDir: '../content' })` and pass the same content root to
preparation and runtime factories. `appRoot` and legacy `configFile` are also
accepted in the second argument.

Alternatively omit the content setting and use
`withZellij({ basePath: '/products/acme/docs' })`. Matching duplicates work;
conflicting explicit values fail instead of producing inconsistent URLs.
`ZELLIJ_BASE_PATH` is reserved for the resolved build-time value. Rebuild after
changing the prefix. Links, Next resources, image sources, search, social images
and sitemap URLs use the same prefix.

HTTP deployment and direct-file viewing are separate capabilities. Portable
`file://` output is planned; the current Next static export does not satisfy it.

---

## Build errors

Invalid content fails the build loudly with the file, the path inside it, and what was expected. Errors are aggregated so everything can be fixed in one pass.

| Failure | Behaviour |
| --- | --- |
| Malformed YAML/MDX | Error naming the file and parser message |
| Unknown section `type` | Error listing every valid type |
| Unknown lucide icon name | Error naming the icon and the export it looked for |
| Section reference with no file | Error naming the page and listing the sections that do exist |
| `page:` naming no page or guide folder | Error listing the available names |
| A page and a guide folder sharing a name | Error naming both |
| Nav or footer link to a nonexistent page | Error naming the label and href |
| Anchor link to an undeclared section `id` | Error listing available anchors on that page |
| Sidebar entry with no matching file | Error |
| Guide page missing from the sidebar | Warning; page still builds |
| Duplicate or colliding routes | Error |
| Unknown top-level config key | Warning |

---

## Implementation notes

Choices worth knowing when working on the engine.

- **MDX** is compiled with `next-mdx-remote/rsc`, chosen over `@next/mdx` because content lives outside the app directory and is resolved by a catch-all route.
- **Sections are server components.** Only interactive leaves are client components — carousel, video, drawer, tabs, search, and the motion wrappers. Icons resolve server-side, keeping lucide out of the client bundle.
- **Heading slugs** come from `github-slugger` — the library `rehype-slug` itself uses — rather than a reimplementation. Parity has to be exact or table-of-contents links break, and the rules are not obvious (`"SSO & SCIM"` becomes `sso--scim`, not `sso-scim`).
- **The published build is unbundled** (`bundle: false`). This is an RSC library where `"use client"` marks the boundary per module; bundling merges modules into chunks and either loses the directive or forces it onto a chunk that also holds server-only code. `npm run build` verifies every client module keeps its directive and that server-only modules do not gain one.
- **Content loading sits behind a `ContentSource` interface** so remote or database-backed sources can be added without touching the render layer.
- **The section registry** is the extensibility seam: adding a tile means registering a schema and a component, never editing dispatch logic.
