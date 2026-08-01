# Zellij

A shared Next.js engine that turns a folder of plain text — YAML, Markdown and
assets — into a complete website: marketing pages and documentation, one theme,
one deployment.

The name is Moroccan. *Zellij* is mosaic tilework, where elaborate walls are
composed from a small fixed vocabulary of geometric tiles. The engine works the
same way: pages are composed from a fixed vocabulary of **sections**, styled by
**themes**.

```
Zellij/
├── specs/          The documents this was designed from
├── engine/         The engine — @shebka/zellij
└── examples/       Three sites built with it
```

## The engine

[`engine/packages/zellij`](engine/packages/zellij) is the whole product: 18
section types, 14 themes, a documentation renderer, ⌘K search, and a build that
refuses to ship broken content.

It is **not a code generator**. Nothing is scaffolded into a consuming
repository, so upgrading is a version bump rather than a merge. A wrapper app is
three files and under thirty lines, and none of it changes as the site grows.

Read [the package README](engine/packages/zellij/README.md) for the full API,
the section reference and the config schema.

## The examples

Zellij is developed against real content rather than fixtures. Each of these is
a genuine site, and each proves something different.

| | What it proves | Port |
| --- | --- | --- |
| [`examples/showcase`](examples/showcase) | That the output is worth showing people. Also the visual test bed — every section, theme and layout, with 240 end-to-end tests. | 4300 |
| [`examples/zellij-docs`](examples/zellij-docs) | That it documents itself. Zellij's own documentation, built with Zellij: two guide trees plus a marketing page set. | 4320 |
| [`examples/payos`](examples/payos) | That real content works. A product's documentation converted from MkDocs sources by a script that only translates admonition syntax. | 4310 |

Each is laid out the same way:

```
examples/<site>/
├── src/app/        The wrapper — layout.tsx and [[...slug]]/page.tsx
├── content/        What the wrapper consumes
├── scripts/        Asset generation, verification
└── next.config.js
```

Content sits beside `src/`, not inside it. That is deliberate: content is the
*input* to the wrapper, not part of its source, and the layout should say so.

## Working on it

```bash
npm install

npm run dev          # the showcase, at :4300
npm run dev:docs     # Zellij's documentation, at :4320
npm run dev:payos    # the PayOS site, at :4310

npm run check        # validate all three content roots, no build required
npm test             # engine unit tests
npm run test:e2e     # end-to-end, desktop and mobile
npm run typecheck    # every workspace, strict
```

One npm workspace spans `engine/` and `examples/`, so an engine edit is visible
in a running example immediately — Turbopack compiles the TypeScript source
directly, with no package build step in the loop.

## The specs

[`specs/zellij-engine-spec.md`](specs/zellij-engine-spec.md) is the
implementation specification. [`specs/zellij-content-model-design.md`](specs/zellij-content-model-design.md)
is the design for the content model — what a content root contains, how routes
come out of it, and why it is shaped that way. The second supersedes the first
wherever they disagree.

---

© 2026 Shebka LLC
