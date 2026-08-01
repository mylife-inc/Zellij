# @shebka/zellij

A shared Next.js engine that turns a folder of structured content (YAML + MDX + assets) into a complete product website — marketing pages and user-guide documentation, one site, one deployment.

The name comes from Moroccan *zellij* mosaic: intricate walls composed from a fixed vocabulary of geometric tiles. Zellij works the same way — polished sites composed from a fixed vocabulary of **sections** ("tiles"), styled by **themes** ("glazes").

**Zellij is not a code generator.** It is a versioned package a thin per-product wrapper imports. All rendering lives here; products upgrade by bumping a dependency version.

---

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
npm install @shebka/zellij next@16.2.x react@19.2.x react-dom@19.2.x
```

Node 20.11+ is required.

### 2. The wrapper app

This is the whole app — three files, under 30 lines.

`app/layout.tsx`

```tsx
import '@shebka/zellij/styles.css';
import { createZellijRootLayout } from '@shebka/zellij';

export default createZellijRootLayout();
```

`app/[[...slug]]/page.tsx`

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
    "predev": "zellij prepare",
    "dev": "next dev",
    "prebuild": "zellij prepare",
    "build": "next build"
  }
}
```

| Command | Does |
| --- | --- |
| `zellij prepare` | Both of the below — the usual choice |
| `zellij sync-assets` | Copies `<content>/assets/**` → `public/_zellij/assets/**` |
| `zellij search-index` | Writes `public/_zellij/search-index.json` |

### 4. Content

```
content/
├── marketing/
│   ├── landing.yaml        # required — the "/" page
│   └── features.yaml
├── guide/
│   ├── _sidebar.yaml
│   └── quickstart.mdx
└── assets/
```

`npm run dev`, and the site is live.

---

## Content model

| Path | Route |
| --- | --- |
| `marketing/landing.yaml` | `/` |
| `marketing/features.yaml` | `/features` |
| `marketing/solutions/startups.yaml` | `/solutions/startups` |
| `guide/index.mdx` | `/guide` |
| `guide/quickstart.mdx` | `/guide/quickstart` |
| `guide/config/database.mdx` | `/guide/config/database` |

Any page may override its route with `route: /custom-path`. Two pages resolving to the same route is a build error, as is a guide page colliding with a marketing page.

### Marketing page shape

```yaml
title: "Features — Acme"        # required; <title> and OG
description: "..."               # required; meta description
route: /features                 # optional; defaults to the filename
layout: stack                    # optional; see Page layouts
theme: midnight                  # optional; per-page theme override
motion: expressive               # optional; per-page motion override
navHighlight: features           # optional; which nav item shows as active
sections:                        # ordered list of tiles
  - type: hero
    headline: "..."
```

---

## Config reference

`zellij.config.yaml` lives at the wrapper app root.

```yaml
product:
  name: Acme                       # required
  tagline: "..."                   # optional
  logo: assets/logo.svg            # optional
  logoDark: assets/logo-dark.svg   # optional

content: ./content                 # path to the content dir, relative to the app
basePath: ""                       # mirror of next.config basePath — keep in sync
siteUrl: https://acme.example.com  # required for absolute OG/canonical URLs and sitemap
ogImage: assets/og.png             # optional social share image

theme: cupertino                   # see Themes
mode: system                       # light | dark | system
motion: subtle                     # none | subtle | expressive

overrides:                         # optional; tweak a theme without forking it
  accent: "#E86A33"
  accentForeground: "#ffffff"
  radius: lg                       # none | sm | md | lg | xl
  fontHeading: "Georgia, serif"
  fontBody: "system-ui, sans-serif"

nav:
  sticky: true                     # default true
  align: left                      # left | center
  hideOnScrollDown: false
  transparentOverHero: true        # applies to `layout: landing` pages
  scrollThreshold: 80
  themeToggle: true                # light/dark toggle
  themeSwitcher: true              # theme picker — see Themes
  search: true                     # enables the ⌘K modal
  links:
    - { label: Features, href: /features }

    - label: Solutions             # dropdown
      children:
        - { label: For startups, href: /solutions/startups }

    - label: Resources             # mega panel
      mega:
        groups:
          - heading: Learn
            items:
              - { label: Guide, href: /guide, description: "Install and operate.", icon: book }

    - { label: Security, href: /features#security }   # anchor into a section
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
      links: [{ label: Features, href: /features }]
  legal: "© 2026 Acme"
  social: { github: "..." }

analytics:
  provider: none                   # plausible | none
  domain: acme.example.com
```

A nav item declares **exactly one** of `href`, `children`, or `mega`.

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

Every section accepts `id` (anchor target) and `motion` (`none` | `subtle` | `expressive`, overriding the page).

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
```

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

Only the **active** theme's tokens are emitted into the page, so the engine can ship many themes at no cost to a consuming site. Fonts use `next/font` with `preload: false` — every theme's `@font-face` rules are present, but browsers download only the families in use. `cupertino` uses the system stack and downloads nothing.

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

Set `nav.search: true`. `zellij search-index` writes a static JSON index at build; the ⌘K modal fetches it and loads FlexSearch **on first open**, so a visitor who never searches downloads neither. Guide content only — marketing pages are excluded by design. Results are split per heading, so a hit lands on the relevant section.

---

## Deploying under a path prefix

To serve at `example.com/products/acme/docs`, set the prefix in **both** places:

```js
// next.config.js
export default { basePath: '/products/acme/docs', transpilePackages: ['@shebka/zellij'] };
```

```yaml
# zellij.config.yaml
basePath: "/products/acme/docs"
```

Next applies the prefix to `<Link>` and optimized `next/image` URLs; Zellij's `basePath` value covers everything else — the search index fetch, unoptimized images, and the sitemap.

---

## Build errors

Invalid content fails the build loudly with the file, the path inside it, and what was expected. Errors are aggregated so everything can be fixed in one pass.

| Failure | Behaviour |
| --- | --- |
| Malformed YAML/MDX | Error naming the file and parser message |
| Unknown section `type` | Error listing every valid type |
| Unknown lucide icon name | Error naming the icon and the export it looked for |
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
