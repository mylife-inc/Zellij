# Zellij Content Model — Design

**Status:** awaiting sign-off
**Supersedes:** the `marketing/` + `guide/` layout in `zellij-engine-spec.md` §4
**Owner:** Shebka LLC

---

## 1. Why this changes

The current layout hardcodes one use case. A content folder must contain
`marketing/` (required) and may contain `guide/`. Those names are the contract,
so every Zellij site is shaped like a software product's site whether or not it
is one.

Zellij's actual scope is wider: an engine that turns plain-text content into a
complete website. That site might belong to a company, an institution, a book,
or a conference. `marketing/` is meaningless for most of them.

This document replaces that layout with one built from four ideas:

1. The content folder is **self-describing** — it carries its own identity, so
   pointing a build at a folder (local or a cloned repo) is enough.
2. Sections are **named files**, reusable across pages, referenced by name.
3. Any folder containing `_sidebar.yaml` is a **guide folder** — a documentation
   tree, mkdocs-style — regardless of its name.
4. Every file is **optional**. A folder with nothing but `_sidebar.yaml` builds.

It also absorbs what were tracked separately as roadmap items 2A (conventions),
2B (sections as files) and 2C (menu definition). They are one design; splitting
them would mean migrating the existing sites three times.

## 2. Vocabulary

| Term | Meaning |
|---|---|
| **Content root** | The folder holding the whole site. Any name. |
| **Page** | Everything rendered below the menubar. Two kinds, below. |
| **Sections page** | A page declared in `pages.yaml` as an ordered list of sections. |
| **Guide folder** | Any folder containing `_sidebar.yaml`. Renders as a documentation tree with left nav, TOC, breadcrumbs and prev/next. Is itself a page. |
| **Section** | One file under `sections/`. A band of content: hero, feature grid, prose. |
| **Menu item** | One entry in the top bar. Targets a page, an external URL, or a drawer of children. |

"Page" covers both kinds deliberately: a menu item points at a page, and does
not care which kind it got.

## 3. Layout

```
my-book/                          ← content root; any name
├── site.yaml                     ← identity, brand, footer, analytics   (optional)
├── menu.yaml                     ← top bar items                        (optional)
├── pages.yaml                    ← sections pages                       (optional)
├── sections/
│   ├── hero-main.yaml
│   ├── why-us.mdx
│   └── home/intro.yaml           ← subfolders allowed
├── assets/
│   └── logo.svg
├── 01-first-chapter/
│   ├── _sidebar.yaml             ← makes this a guide folder
│   └── *.md, *.mdx
└── 02-second-chapter/
    └── _sidebar.yaml
```

Nothing here is required. The engine builds whatever it finds, and reports what
it could not use (§10).

## 4. `site.yaml`

Site identity. Moves here from `zellij.config.yaml` at the wrapper-app root so
the content root is portable — a prerequisite for the CLI (roadmap item 4)
building from a folder it just cloned.

```yaml
name: PayOS                       # required if the file exists
tagline: Payments infrastructure

brand:
  logo:
    src: assets/shebka.svg
    dark: assets/shebka-dark.svg  # optional
    alt: Shebka
    href: https://shebka.com      # default "/" — see §4.1
  title: PayOS                    # shown beside the logo; links to this site's "/"

theme: cupertino                  # the theme the site first loads with
mode: system                      # light | dark | system
motion: subtle                    # none | subtle | expressive
overrides: { accent: '#0b6' }     # per-site token tweaks

home: pricing                     # optional; default is the first menu item (§8)
menuOverflowAfter: 7              # optional; grouping threshold (§9)

siteUrl: https://payos.dev        # canonical origin, for OG and sitemap
ogImage: assets/og.png
analytics: { provider: plausible, domain: payos.dev }

footer:
  columns:
    - heading: Product
      links: [{ label: Pricing, page: pricing }]
  legal: © 2026 Shebka LLC
  social: { github: https://github.com/shebka }
```

Every key optional except `name`. An absent `site.yaml` means: name derived from
the content-root folder name, `theme: paper`, everything else at its default.

### 4.1 The brand block and returning to the hub

Where several sites belong to one owner, the org logo points at the hub and the
site's own name points at its own root:

- `brand.logo.href` omitted → logo links to `/`. The single-site case; `brand.title`
  is then unnecessary.
- `brand.logo.href` set to an absolute URL → logo leaves for the hub, and
  `brand.title` becomes the link home within this site.

This is the whole mechanism for getting back to the origin site after following
an outbound menu item. It needs no session state and works for visitors who
arrived from search rather than from the hub.

## 5. `menu.yaml`

Declares the items in the top bar, and only those. Search, the theme picker and
the colour-mode toggle are built in and are configured in `site.yaml`.

```yaml
- name: Products
  items:                            # drawer: opens on hover
    - name: PayOS
      url: https://payos.dev        # external
      description: Payments infrastructure
    - name: Pricing
      page: pricing                 # internal page reference
- name: Guide
  page: 01-getting-started          # a guide folder is referenced the same way
- name: Blog
  page: blog
```

Each item declares **exactly one** of:

| Key | Target |
|---|---|
| `page:` | A sections page name, or a guide folder path. Validated at build time. |
| `url:` | A literal href — external, or internal when it needs an anchor (`/features#security`). Not validated. An external one opens in a new tab (`newTab: false` to override) and renders an outbound icon. |
| `items:` | Child items — makes this a drawer. Children follow the same rules; one level of nesting. |

Optional per item: `description`, `icon`, `wide` (render the drawer as a
full-width panel rather than a compact dropdown).

## 6. `pages.yaml`

Sections pages: an ordered list of sections per page.

```yaml
- name: home
  title: PayOS — payments infrastructure
  description: Ship payments in a week.
  sections:
    - hero-main                          # by name, no overrides
    - section: features                  # with overrides
      background: { gradient: [indigo, violet] }
    - section: features                  # same content, second appearance
      id: enterprise
      background: tinted

- name: pricing
  title: Pricing
  description: Simple, per-seat.
  route: /plans                          # optional; default is /<name>
  layout: stack                          # stack | stack-narrow | stack-sidebar | landing
  sections: [pricing-main, faq-billing]
```

A section reference is either a bare name or a mapping with `section:` plus
overrides.

**Overrides may change presentation, never content.** Permitted: `background`
(`default`, `tinted`, `inverted`, `accent`, or a `gradient:` of two or three
stops), `id` (the anchor), `motion`, and `padding`. Not permitted: any field the
section file itself declares — headline, body, items, images. Content lives in
exactly one place, which is the point of sections being files.

Gradient stops name palette tokens (`accent`, `surface`, `inverted-bg`, …) so a
gradient follows the active theme, and a literal CSS colour is accepted for the
cases where it must not.

`id` matters when a section appears twice on one page: anchors must stay unique,
so the second occurrence needs its own.

## 7. Section files

One file per section, under `sections/`. The name is its path below `sections/`
minus the extension — `sections/home/intro.yaml` is `home/intro`.

**`.yaml`** — a structured section. `type` picks the renderer; the rest are that
type's fields:

```yaml
# sections/hero-main.yaml
type: hero
eyebrow: Now in general availability
headline: Payments that ship in a week
body: One API for cards, wallets and bank transfers.
actions:
  - { label: Start building, href: /01-getting-started, style: primary }
media: { src: assets/hero.png, alt: Dashboard }
```

**`.md` / `.mdx`** — a prose section. Defaults to `type: markdown-prose`, with
the body as content and optional frontmatter for presentation or a different
type:

```mdx
---
background: tinted
---

## Why teams choose PayOS

Three lines of code, one settlement account.
```

Registered types today: `hero`, `feature-grid`, `zigzag`, `showcase`,
`logo-strip`, `stats-band`, `testimonial`, `pricing-table`, `faq`, `cta-banner`,
`video-embed`, `markdown-prose`. Unknown types are a build error listing the
valid ones.

## 8. Guide folders and `_sidebar.yaml`

Any folder containing `_sidebar.yaml` is a guide folder. Its `.md`/`.mdx` files
become pages in a documentation tree, ordered by the sidebar.

Today's bare-list form is unchanged and stays valid:

```yaml
- label: Getting Started
  items: [quickstart, installation]
- label: Configuration
  items:
    - config/overview
    - label: Data
      items: [config/database]
```

An optional mapping form adds folder-level metadata, needed for ordering (§9)
and for a display label that differs from the folder name:

```yaml
title: Getting Started
order: 1
nav:
  - label: Introduction
    items: [quickstart, installation]
```

Both forms are accepted; which one is in use is decided by whether the document
is a sequence or a mapping.

Routes mount under the folder's own route: `01-getting-started/config/database.mdx`
→ `/getting-started/config/database` (prefix stripped, §9). `index.mdx` maps to
the folder root. A guide folder with no `index.mdx` redirects its root to the
first page in the sidebar.

## 9. Names, routes and ordering

**Numeric prefixes are stripped** from both the route and the display label.
`01-getting-started/` and `1.GettingStarted/` both give route `/getting-started`
and label "Getting started". Recognised forms: `NN-`, `NN.`, `NN_`.

Explicit values always win over derivation: `title:` in `_sidebar.yaml`, `name:`
in `menu.yaml`, `route:` in `pages.yaml`.

**Ordering ladder** — first rung that applies decides:

| # | Rule |
|---|---|
| 1 | Listed explicitly in `menu.yaml` or `pages.yaml` → that order |
| 2 | `order:` in the folder's `_sidebar.yaml` |
| 3 | Numeric folder prefix |
| 4 | Alphabetical by folder name |

Route collisions between a sections page and a guide folder are a build error
naming both sources.

## 10. Deriving the menubar

`menu.yaml` is the source of truth when present: the bar contains exactly what
it declares, and nothing is added.

When it is absent the menu is derived as the **union**, in this order:

1. Every entry in `pages.yaml`, in file order — which may itself name guide
   folders, giving exact control without a `menu.yaml`.
2. Every guide folder not already named above, ordered by §9.

The union matters: deriving from `pages.yaml` alone would leave guide folders
unreachable from the bar. Nothing on the site silently loses its way in.

If the result exceeds `menuOverflowAfter` (default 7), the first N are shown and
the remainder collapse into one trailing drawer, rendered as a full-width panel.
A book with thirty chapters therefore gets a usable bar with no configuration.

## 11. Validation

Three levels, applied consistently:

**Error — the build stops.** Unresolvable `page:` reference in `menu.yaml` or
the footer; unknown section `type`; a section reference naming a file that does
not exist; route collisions; malformed YAML; a section failing its own schema;
a sidebar entry naming a missing file.

**Warning — the build continues, the problem is printed.** A file in `sections/`
that no page references; a `.md`/`.mdx` file inside a guide folder absent from
its `_sidebar.yaml`; an unknown top-level key in `site.yaml` (forward
compatibility); an asset reference resolving outside `assets/`.

**Silently accepted.** Unknown keys inside a section file that its type does not
declare — sections are extended more often than the top-level schema, and a
newer content folder should still build on an older engine.

`zellij check [dir]` runs the full load and prints errors and warnings without
building, so content authors get a fast loop that does not require Next.js. It
runs from the package's bundled build rather than from source: Node's ESM
resolver requires file extensions on relative imports and the source does not
carry them. Published installs always have that build; inside this repo it needs
`npm run build -w @shebka/zellij` once.

## 12. Migration

The change is breaking. The old loader stays in place, and the new one is
selected by what the content root contains.

**Detection.** A root containing `marketing/` uses the legacy loader and prints
one deprecation warning. Otherwise the new loader runs.

**Compatibility retained for one release:** `marketing/*.yaml` pages with inline
sections; a `guide/` folder without `_sidebar.yaml` still treated as a guide;
`zellij.config.yaml` at the app root, read when no `site.yaml` is present.

**Both existing sites migrate as part of this work:**

- `examples/showcase` — `marketing/landing.yaml` splits into `sections/*.yaml` plus a
  `pages.yaml` entry; `content/guide/` keeps its `_sidebar.yaml` unchanged and
  gains a `menu.yaml` entry; `zellij.config.yaml` becomes `content/site.yaml`.
- `examples/payos` — same shape. Its converter script (`scripts/convert-docs.mjs`)
  emits the new layout directly.

The legacy path is removed in a later pass, once both sites have been on the new
loader through a full test run.

## 13. Deliberately out of scope

- Configurable folder names. `sections/`, `assets/` and `_sidebar.yaml` are
  fixed; the payoff for making them configurable is small and the cost to tooling
  and documentation is not.
- More than one content root per site.
- Content versioning and internationalisation.
- A cross-site "return to the page you left" protocol — §4.1 covers the case
  with no moving parts.
- Section-level partials or includes. Reuse is a page referencing the same
  section twice.

## 14. Status

Signed off and implemented. Notes from the build:

- `basePath` and the content-root location moved from `zellij.config.yaml` into
  `createZellijPages()` options, so a new site needs no file outside its content
  root. `site.yaml` still accepts `basePath` for sites that prefer it there.
- The overridable set is `background`, `id`, `motion`, `padding`. `align` was in
  the draft and was dropped rather than shipped as a silent no-op: no section
  reads it.
- The demo and PayOS both migrated. PayOS now carries two guide folders
  (`01-guide/`, `02-using-payos/`), which is what exercises multi-tree
  navigation, ordering prefixes and per-tree sidebars against real content.
