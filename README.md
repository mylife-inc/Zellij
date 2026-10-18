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
├── cli/            `zel`, the command line (Rust)
├── docker/         The base image, and the one a site is built from
└── examples/       Three sites built with it
```

## The engine

[`engine/packages/zellij`](engine/packages/zellij) is the whole product: 19
section types, 14 themes, a documentation renderer, ⌘K search, and a build that
refuses to ship broken content.

It is **not a code generator**. Nothing is scaffolded into a consuming
repository, so upgrading is a version bump rather than a merge. A wrapper app is
three files and under thirty lines, and none of it changes as the site grows.

Read [the package README](engine/packages/zellij/README.md) for the full API,
the section reference and the config schema.

## Portable Markdown documentation

The Rust `zel` binary also has a native Markdown workflow that needs no Docker,
Node or network connection. With a current source build:

```bash
zel new portable docs --name "My documentation"
zel check docs --strict
zel build docs
zel view docs --watch
```

Open `docs-site/index.html` directly in a browser, or use `zel serve docs-site`
for HTTP preview. The generated folder can be moved or shared intact, including
its offline search and local fonts/assets. `--minimal` creates only a home page
and `zel.yaml`; an existing Markdown folder needs no scaffolding.

Native release packages may lag source. Check `zel new --help` for availability.
See the [portable contract](specs/zel-portable-docs.md) for supported Markdown,
configuration, platform acceptance and remaining rendering limits.

Current-source packaging covers x64 and ARM64 on GNU/Linux, macOS and Windows,
with installation tests against actual release archives. Windows uses the
PowerShell installer; Unix uses the shell installer. See the
[native distribution contract](specs/zel-native-distribution.md) for tested OS
versions, commands and limits. These changes do not replace the older published
`v0.1.3` binaries or create a new release automatically.

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

## Using it without this repository

The engine is private; the image is not. A site is built from a public base
image that has the engine already installed, so nobody needs access to this
source to publish documentation with it.

```bash
docker build -f site.Dockerfile --build-arg CONTENT=docs -t my-docs .
```

[`docker/site.Dockerfile`](docker/site.Dockerfile) is the half that gets copied
into another repository. It validates the content folder before building, so a
broken link fails the build rather than shipping a dead page, and its runtime
stage carries only the compiled site — the engine never reaches what gets
deployed.

## The command line

[`cli/`](cli) is `zel`: one native binary for scaffolding a content folder,
converting one from another documentation tool, validating it, building it and
shipping it. It needs no Node — the commands that build use the container, and
the ones that describe Zellij's own content model read a schema the engine
generates and the binary compiles in.

```bash
zel explain bento-grid      # every field, with a worked example
zel new section hero        # a section file, fields filled in
zel context reference.md    # the whole model, for an AI agent
```

[`specs/zellij-cli-spec.md`](specs/zellij-cli-spec.md) is the full surface.
