# Zellij content model — reference for an AI agent

Zellij turns a folder of structured content into a complete website: marketing
pages built from typed sections, and documentation built from MDX. You write
YAML and MDX; there is no code to write and no template to fill in.

Your job, given a codebase or an existing set of documentation, is to produce a
**content folder** that Zellij can build. Nothing else — no React, no Next.js
app, no configuration beyond the files described here.

## Rules that matter more than the details

1. **Never invent a field or a section type.** Anything absent from this
   document will fail the build. The valid section types are listed below and
   that list is complete.
2. **A section file holds its own text.** Pages reference sections by name and
   may override presentation, never content.
3. **Every internal link must resolve.** Links are validated at build time and a
   broken one fails the build.
4. **Validate before you finish** by running `zellij check <folder>`. It names
   the file behind any problem. Treat a clean run as the definition of done.

## The folder

    content/
    ├── site.yaml          identity, theme, nav and footer          (optional)
    ├── menu.yaml          the top navigation                       (optional)
    ├── pages.yaml         marketing pages, each a list of sections (optional)
    ├── sections/          one file per section, referenced by path
    ├── examples/          sections shown in docs as source + preview (optional)
    ├── assets/            images and video, referenced as assets/…
    └── <guide folders>/   any folder holding a _sidebar.yaml

Every file is optional. A folder containing nothing but one guide folder builds
a complete site.

### Naming and routes

- A guide folder mounts at its own name. A leading number is an ordering hint
  and is stripped: `01-guide/` serves at `/guide`.
- `index.mdx` is the folder's own page; `quickstart.mdx` becomes
  `/guide/quickstart`.
- A section is referenced by its path below `sections/` without the
  extension: `sections/home/hero.yaml` is `home/hero`.

## `site.yaml`

| field | type | |
| --- | --- | --- |
| `name` | string | **required** |
| `tagline` | string |  |
| `brand` | object { logo, title } | default `{}` |
| `theme` | `midnight` \| `paper` \| `terracotta` \| `mono` \| `aurora` \| `slate` \| `forest` \| `sunburst` \| `carbon` \| `porcelain` \| `cupertino` \| `studio` \| `iris` \| `meridian` | default `"paper"` |
| `mode` | `light` \| `dark` \| `system` | default `"system"` |
| `motion` | `none` \| `subtle` \| `expressive` | default `"subtle"` |
| `overrides` | object { accent, accentForeground, radius, fontHeading, fontBody } | default `{}` |
| `home` | string |  |
| `menuOverflowAfter` | integer | default `7` |
| `basePath` | string | default `""` |
| `siteUrl` | string |  |
| `ogImage` | string |  |
| `nav` | object { sticky, align, hideOnScrollDown, transparentOverHero, scrollThreshold, themeToggle, themeSwitcher, search, cta, social, announcement } | default `{}` |
| `footer` | object { columns, legal, social } | default `{}` |
| `analytics` | object { provider, domain } | default `{}` |

**`brand`**

| field | type | |
| --- | --- | --- |
| `logo` | object { src, dark, alt, href, height } |  |
| `title` | string |  |

**`brand.logo`**

| field | type | |
| --- | --- | --- |
| `src` | string | **required** |
| `dark` | string |  |
| `alt` | string |  |
| `href` | string |  |
| `height` | string |  |

**`overrides`**

| field | type | |
| --- | --- | --- |
| `accent` | string |  |
| `accentForeground` | string |  |
| `radius` | `none` \| `sm` \| `md` \| `lg` \| `xl` |  |
| `fontHeading` | string |  |
| `fontBody` | string |  |

**`nav`**

| field | type | |
| --- | --- | --- |
| `sticky` | boolean | default `true` |
| `align` | `left` \| `center` | default `"left"` |
| `hideOnScrollDown` | boolean | default `false` |
| `transparentOverHero` | boolean | default `true` |
| `scrollThreshold` | integer | default `80` |
| `themeToggle` | boolean | default `false` |
| `themeSwitcher` | boolean | default `false` |
| `search` | boolean | default `false` |
| `cta` | list of object { label, href, style } | default `[]` |
| `social` | object { github, x, linkedin, youtube, discord } |  |
| `announcement` | object { text, link, dismissible } |  |

**`nav.cta`**

| field | type | |
| --- | --- | --- |
| `label` | string | **required** |
| `href` | string | **required** |
| `style` | `primary` \| `secondary` \| `ghost` | default `"primary"` |

**`nav.social`**

| field | type | |
| --- | --- | --- |
| `github` | string |  |
| `x` | string |  |
| `linkedin` | string |  |
| `youtube` | string |  |
| `discord` | string |  |

**`nav.announcement`**

| field | type | |
| --- | --- | --- |
| `text` | string | **required** |
| `link` | object { label, href } |  |
| `dismissible` | boolean | default `true` |

**`footer`**

| field | type | |
| --- | --- | --- |
| `columns` | list of object { heading, links } | default `[]` |
| `legal` | string |  |
| `social` | object { github, x, linkedin, youtube, discord } |  |

**`footer.columns`**

| field | type | |
| --- | --- | --- |
| `heading` | string | **required** |
| `links` | list of object { label, href, page } (min 1) | **required** |

**`footer.social`**

| field | type | |
| --- | --- | --- |
| `github` | string |  |
| `x` | string |  |
| `linkedin` | string |  |
| `youtube` | string |  |
| `discord` | string |  |

**`analytics`**

| field | type | |
| --- | --- | --- |
| `provider` | `plausible` \| `none` | default `"none"` |
| `domain` | string |  |


## `menu.yaml`

A list of items. Each names exactly one target — `page`, `url` or `href` —
or carries `items` to become a dropdown. A child with its own `items` makes
the parent a full-width panel.

| field | type | |
| --- | --- | --- |
| `name` | string | **required** |
| `page` | string |  |
| `url` | string |  |
| `description` | string |  |
| `icon` | string |  |
| `newTab` | boolean |  |
| `wide` | boolean |  |
| `items` | list of object { name, items } \| object { name, page, url, description, icon, newTab } (min 1) |  |

**`items`**

| field | type | |
| --- | --- | --- |
| `name` | string | **required** |
| `items` | list of object { name, page, url, description, icon, newTab } (min 1) | **required** |

**`items.items`**

| field | type | |
| --- | --- | --- |
| `name` | string | **required** |
| `page` | string |  |
| `url` | string |  |
| `description` | string |  |
| `icon` | string |  |
| `newTab` | boolean |  |


## `pages.yaml`

A list of pages. `sections` names files below `sections/`, in the order they
should appear. An entry may be a plain name, or a mapping of `section:` plus
presentation overrides.

| field | type | |
| --- | --- | --- |
| `name` | string | **required** |
| `title` | string |  |
| `description` | string |  |
| `route` | string |  |
| `layout` | `stack` \| `stack-narrow` \| `stack-sidebar` \| `landing` | default `"stack"` |
| `theme` | `midnight` \| `paper` \| `terracotta` \| `mono` \| `aurora` \| `slate` \| `forest` \| `sunburst` \| `carbon` \| `porcelain` \| `cupertino` \| `studio` \| `iris` \| `meridian` |  |
| `motion` | `none` \| `subtle` \| `expressive` |  |
| `navHighlight` | string |  |
| `sections` | list of string \| object { id, background, motion, padding, titleGradient, section } | default `[]` |

**`sections`**

| field | type | |
| --- | --- | --- |
| `id` | string |  |
| `background` | `default` \| `tinted` \| `inverted` \| `accent` \| object { gradient } |  |
| `motion` | `none` \| `subtle` \| `expressive` |  |
| `padding` | `none` \| `tight` \| `normal` \| `loose` |  |
| `titleGradient` | list of string (min 2, max 3) |  |
| `section` | string | **required** |

**`sections.background`**

| field | type | |
| --- | --- | --- |
| `gradient` | list of string (min 2, max 3) | **required** |


## `_sidebar.yaml`

Either a bare list of entries, or a mapping with the fields below. An entry
is a page name, or `{ label, items }` to group — groups nest to any depth.

| field | type | |
| --- | --- | --- |
| `title` | string |  |
| `order` | number |  |
| `route` | string |  |
| `description` | string |  |
| `nav` | any | default `[]` |


## Guide pages

A guide folder is any folder containing `_sidebar.yaml`. Pages are `.mdx`
files with frontmatter:

```mdx
---
title: Quickstart
description: Get running in ten minutes.
---

## A heading

Prose, with the components below available without importing them.
```

### Components available in MDX

`<Callout type="info|tip|warning|danger" title="…">`, `<Steps>` / `<Step>`,
`<Tabs>` / `<Tab label="…">`, `<CodeGroup>`, `<Card>` / `<CardGrid>`,
`<Screenshot src="assets/…" alt="…">`, `<SectionPreview example="…">`,
`<Theorem kind="…" number="…" title="…">` and `<Proof>`.

Fenced code blocks take `title="file.ts"` and `{2,5-6}` line highlighting.
LaTeX goes between `$…$` inline or `$$…$$` display. A `mermaid` fence is
drawn as a diagram.

### Writing style for generated documentation

Write what the code does and why, not what its identifiers are called. A page
that restates the function signature adds nothing a reader could not get from
the file. Prefer a short page that answers one question to a long page that
lists everything.


## Section types

There are 23. Every section also accepts the shared fields `id` (URL anchor), `background`, `padding`, `motion` and `titleGradient`.

### `bento-grid`

| field | type | |
| --- | --- | --- |
| `heading` | string |  |
| `subhead` | string |  |
| `items` | list of object { title, body, icon, image, href, span, background } (min 2) | **required** |

**`items`**

| field | type | |
| --- | --- | --- |
| `title` | string | **required** |
| `body` | string |  |
| `icon` | string |  |
| `image` | string |  |
| `href` | string |  |
| `span` | `normal` \| `wide` \| `tall` \| `large` | default `"normal"` |
| `background` | `default` \| `tinted` \| `inverted` \| `accent` |  |


### `card-carousel`

| field | type | |
| --- | --- | --- |
| `heading` | string |  |
| `subhead` | string |  |
| `link` | object { label, href } |  |
| `cardWidth` | `narrow` \| `medium` \| `wide` | default `"medium"` |
| `items` | list of object { eyebrow, title, body, image, href, background } (min 2) | **required** |

**`link`**

| field | type | |
| --- | --- | --- |
| `label` | string | **required** |
| `href` | string | **required** |

**`items`**

| field | type | |
| --- | --- | --- |
| `eyebrow` | string |  |
| `title` | string | **required** |
| `body` | string |  |
| `image` | string |  |
| `href` | string |  |
| `background` | `default` \| `tinted` \| `inverted` \| `accent` |  |


### `code-cards`

| field | type | |
| --- | --- | --- |
| `heading` | string |  |
| `subhead` | string |  |
| `eyebrow` | string |  |
| `cards` | list of object { eyebrow, title, body, featured, code, href } (min 2, max 4) | **required** |
| `footnote` | string |  |

**`cards`**

| field | type | |
| --- | --- | --- |
| `eyebrow` | string |  |
| `title` | string | **required** |
| `body` | string |  |
| `featured` | boolean | default `false` |
| `code` | object { title, body } | **required** |
| `href` | string |  |

**`cards.code`**

| field | type | |
| --- | --- | --- |
| `title` | string |  |
| `body` | string | **required** |


### `cta-banner`

| field | type | |
| --- | --- | --- |
| `headline` | string | **required** |
| `body` | string |  |
| `command` | string |  |
| `cta` | object { label, href, style } \| list of object { label, href, style } (max 2) |  |

**`cta`**

| field | type | |
| --- | --- | --- |
| `label` | string | **required** |
| `href` | string | **required** |
| `style` | `primary` \| `secondary` \| `ghost` | default `"primary"` |


### `faq`

| field | type | |
| --- | --- | --- |
| `heading` | string |  |
| `items` | list of object { question, answer } (min 1) | **required** |

**`items`**

| field | type | |
| --- | --- | --- |
| `question` | string | **required** |
| `answer` | string | **required** |


### `feature-grid`

| field | type | |
| --- | --- | --- |
| `heading` | string |  |
| `subhead` | string |  |
| `columns` | `2` \| `3` \| `4` | default `3` |
| `items` | list of object { icon, title, body, href } (min 1) | **required** |

**`items`**

| field | type | |
| --- | --- | --- |
| `icon` | string |  |
| `title` | string | **required** |
| `body` | string |  |
| `href` | string |  |


### `gallery`

| field | type | |
| --- | --- | --- |
| `heading` | string |  |
| `subhead` | string |  |
| `columns` | `2` \| `3` \| `4` | default `3` |
| `lightbox` | boolean | default `true` |
| `items` | list of object { image, alt, caption } (min 2) | **required** |

**`items`**

| field | type | |
| --- | --- | --- |
| `image` | string | **required** |
| `alt` | string | default `""` |
| `caption` | string |  |


### `hero`

| field | type | |
| --- | --- | --- |
| `headline` | string | **required** |
| `subhead` | string |  |
| `badge` | string |  |
| `cta` | list of object { label, href, style } (max 2) | default `[]` |
| `visual` | string |  |
| `visualStyle` | `floating` \| `framed` \| `full-bleed` | default `"floating"` |

**`cta`**

| field | type | |
| --- | --- | --- |
| `label` | string | **required** |
| `href` | string | **required** |
| `style` | `primary` \| `secondary` \| `ghost` | default `"primary"` |


### `hero-split`

| field | type | |
| --- | --- | --- |
| `headline` | string | **required** |
| `subhead` | string |  |
| `badge` | string |  |
| `badgeIcon` | string |  |
| `cta` | list of object { label, href, style } (max 2) | default `[]` |
| `highlights` | list of object { icon, label } (max 4) | default `[]` |
| `media` | `left` \| `right` | default `"right"` |
| `visual` | string |  |
| `terminal` | object { title, body } |  |

**`cta`**

| field | type | |
| --- | --- | --- |
| `label` | string | **required** |
| `href` | string | **required** |
| `style` | `primary` \| `secondary` \| `ghost` | default `"primary"` |

**`highlights`**

| field | type | |
| --- | --- | --- |
| `icon` | string |  |
| `label` | string | **required** |

**`terminal`**

| field | type | |
| --- | --- | --- |
| `title` | string | default `"terminal"` |
| `body` | string | **required** |


### `logo-strip`

| field | type | |
| --- | --- | --- |
| `heading` | string |  |
| `logos` | list of string (min 1) | **required** |


### `markdown-prose`

| field | type | |
| --- | --- | --- |
| `heading` | string |  |
| `body` | string | **required** |


### `matrix`

| field | type | |
| --- | --- | --- |
| `heading` | string |  |
| `subhead` | string |  |
| `eyebrow` | string |  |
| `rowAxis` | string |  |
| `columnAxis` | string |  |
| `columns` | list of object { label } (min 2, max 5) | **required** |
| `rows` | list of object { label, description, cells } (min 2, max 6) | **required** |
| `footnote` | string |  |

**`columns`**

| field | type | |
| --- | --- | --- |
| `label` | string | **required** |

**`rows`**

| field | type | |
| --- | --- | --- |
| `label` | string | **required** |
| `description` | string |  |
| `cells` | list of string (min 2, max 5) | **required** |


### `media-panel`

| field | type | |
| --- | --- | --- |
| `heading` | string |  |
| `eyebrow` | string |  |
| `title` | string |  |
| `body` | string |  |
| `image` | string |  |
| `video` | string |  |
| `poster` | string |  |
| `alt` | string | default `""` |
| `align` | `left` \| `center` \| `right` | default `"left"` |
| `tone` | `light` \| `dark` \| `auto` | default `"auto"` |
| `ratio` | `ultrawide` \| `wide` \| `square` \| `portrait` | default `"wide"` |
| `scrim` | number | default `0` |
| `cta` | list of object { label, href, style } (max 2) | default `[]` |
| `href` | string |  |

**`cta`**

| field | type | |
| --- | --- | --- |
| `label` | string | **required** |
| `href` | string | **required** |
| `style` | `primary` \| `secondary` \| `ghost` | default `"primary"` |


### `pricing-table`

| field | type | |
| --- | --- | --- |
| `heading` | string |  |
| `subhead` | string |  |
| `tiers` | list of object { name, price, period, description, features, cta, highlighted } (min 1, max 4) | **required** |

**`tiers`**

| field | type | |
| --- | --- | --- |
| `name` | string | **required** |
| `price` | string | **required** |
| `period` | string |  |
| `description` | string |  |
| `features` | list of string | default `[]` |
| `cta` | object { label, href, style } |  |
| `highlighted` | boolean | default `false` |

**`tiers.cta`**

| field | type | |
| --- | --- | --- |
| `label` | string | **required** |
| `href` | string | **required** |
| `style` | `primary` \| `secondary` \| `ghost` | default `"primary"` |


### `promo-grid`

| field | type | |
| --- | --- | --- |
| `heading` | string |  |
| `gap` | `none` \| `tight` \| `normal` | default `"normal"` |
| `items` | list of object { eyebrow, title, body, image, video, alt, width, align, focus, tone, scrim, height, links, href } (min 1) | **required** |

**`items`**

| field | type | |
| --- | --- | --- |
| `eyebrow` | string |  |
| `title` | string | **required** |
| `body` | string |  |
| `image` | string |  |
| `video` | string |  |
| `alt` | string | default `""` |
| `width` | `full` \| `half` | default `"full"` |
| `align` | `top` \| `center` \| `bottom` | default `"top"` |
| `focus` | `top` \| `center` \| `bottom` | default `"center"` |
| `tone` | `light` \| `dark` \| `auto` | default `"auto"` |
| `scrim` | number | default `0` |
| `height` | `short` \| `tall` \| `full` | default `"tall"` |
| `links` | list of object { label, href } (max 2) | default `[]` |
| `href` | string |  |

**`items.links`**

| field | type | |
| --- | --- | --- |
| `label` | string | **required** |
| `href` | string | **required** |


### `showcase`

| field | type | |
| --- | --- | --- |
| `eyebrow` | string |  |
| `headline` | string | **required** |
| `body` | string |  |
| `visual` | string |  |
| `cta` | object { label, href, style } |  |

**`cta`**

| field | type | |
| --- | --- | --- |
| `label` | string | **required** |
| `href` | string | **required** |
| `style` | `primary` \| `secondary` \| `ghost` | default `"primary"` |


### `stats-band`

| field | type | |
| --- | --- | --- |
| `heading` | string |  |
| `stats` | list of object { value, label, prefix, suffix } (min 3, max 5) | **required** |

**`stats`**

| field | type | |
| --- | --- | --- |
| `value` | string \| number | **required** |
| `label` | string | **required** |
| `prefix` | string |  |
| `suffix` | string |  |


### `steps-panel`

| field | type | |
| --- | --- | --- |
| `heading` | string |  |
| `subhead` | string |  |
| `steps` | list of object { title, body } (min 2, max 12) | **required** |
| `media` | `left` \| `right` | default `"right"` |
| `visual` | string |  |
| `terminal` | object { title, body } |  |
| `diagram` | string |  |
| `caption` | string |  |

**`steps`**

| field | type | |
| --- | --- | --- |
| `title` | string | **required** |
| `body` | string |  |

**`terminal`**

| field | type | |
| --- | --- | --- |
| `title` | string | default `"terminal"` |
| `body` | string | **required** |


### `team-grid`

| field | type | |
| --- | --- | --- |
| `heading` | string |  |
| `subhead` | string |  |
| `columns` | `2` \| `3` \| `4` | default `3` |
| `people` | list of object { name, role, bio, photo, links } (min 1) | **required** |

**`people`**

| field | type | |
| --- | --- | --- |
| `name` | string | **required** |
| `role` | string |  |
| `bio` | string |  |
| `photo` | string |  |
| `links` | list of object { label, href } | default `[]` |

**`people.links`**

| field | type | |
| --- | --- | --- |
| `label` | string | **required** |
| `href` | string | **required** |


### `testimonial`

| field | type | |
| --- | --- | --- |
| `heading` | string |  |
| `quotes` | list of object { quote, author, role, avatar } (min 1, max 5) | **required** |

**`quotes`**

| field | type | |
| --- | --- | --- |
| `quote` | string | **required** |
| `author` | string | **required** |
| `role` | string |  |
| `avatar` | string |  |


### `timeline`

| field | type | |
| --- | --- | --- |
| `heading` | string |  |
| `subhead` | string |  |
| `items` | list of object { date, title, body, image, icon } (min 2) | **required** |

**`items`**

| field | type | |
| --- | --- | --- |
| `date` | string \| number | **required** |
| `title` | string | **required** |
| `body` | string |  |
| `image` | string |  |
| `icon` | string |  |


### `video-embed`

| field | type | |
| --- | --- | --- |
| `heading` | string |  |
| `src` | string | **required** |
| `poster` | string |  |
| `caption` | string |  |


### `zigzag`

| field | type | |
| --- | --- | --- |
| `heading` | string |  |
| `rows` | list of object { title, body, image, align, cta } (min 1) | **required** |

**`rows`**

| field | type | |
| --- | --- | --- |
| `title` | string | **required** |
| `body` | string |  |
| `image` | string |  |
| `align` | `image-right` \| `image-left` | default `"image-right"` |
| `cta` | object { label, href, style } |  |

**`rows.cta`**

| field | type | |
| --- | --- | --- |
| `label` | string | **required** |
| `href` | string | **required** |
| `style` | `primary` \| `secondary` \| `ghost` | default `"primary"` |


## A minimal, complete example

```yaml
# content/site.yaml
name: Acme
theme: cupertino
home: home
```

```yaml
# content/pages.yaml
- name: home
  title: Acme
  description: One sentence about the product.
  layout: landing
  sections: [home/hero]
```

```yaml
# content/sections/home/hero.yaml
type: hero
headline: Ship documentation with the product
subhead: A folder of content, and the site builds itself.
cta:
  - { label: Read the guide, href: /guide, style: primary }
```

```yaml
# content/01-guide/_sidebar.yaml
title: Guide
nav:
  - index
  - quickstart
```

That is a complete site: a landing page and a guide.

## When you are done

Run `zellij check <folder>`. Fix what it names. A clean run means the folder
builds.
