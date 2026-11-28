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

[`engine/packages/zellij`](engine/packages/zellij) is the whole product: 25
section types, 14 themes, a documentation renderer, ⌘K search, and a build that
refuses to ship broken content.

It is **not a code generator**. Nothing is scaffolded into a consuming
repository, so upgrading is a version bump rather than a merge. A wrapper app is
three files and under thirty lines, and none of it changes as the site grows.

Read [the package README](engine/packages/zellij/README.md) for the full API,
the section reference and the config schema.

## Portable Markdown documentation

The Rust `zel` binary also has a native Markdown workflow that needs no Docker,
Node or network connection. With the CLI candidate:

```bash
zel new portable docs --name "My documentation"
zel check docs --strict
zel build docs
zel view docs --watch
```

Open `docs/site/index.html` directly in a browser, or use `zel serve docs`
for HTTP preview. The generated folder can be moved or shared intact, including
its offline search and local fonts/assets. `--minimal` creates only a home page
and `zel.yaml`; an existing Markdown folder needs no scaffolding.

The candidate packages include these commands. Check `zel new --help` for options.
See the [portable contract](specs/zel-portable-docs.md) for supported Markdown,
configuration, platform acceptance and remaining rendering limits.

Portable math is opt-in with `markdown: {math: true}` in `zel.yaml`. Inline
`$...$` and display `$$...$$` equations render at build time with a bundled font,
including when opened directly with JavaScript disabled. Source builds require
Rust 1.96 or newer; downloaded binaries do not require a Rust installation.

YAML `mindmap` fences render to SVG during the native build, with a text outline
and no browser library. They work offline with JavaScript disabled. Enable
`markdown: {mermaid: true}` for Mermaid diagrams through a bundled offline browser
runtime. Diagram source stays readable without JavaScript; Mermaid grammar errors
are reported in the browser, while native checks validate input limits.

[Candidate 0.2.0-rc.25](https://github.com/mylife-inc/Zellij/releases/tag/v0.2.0-rc.25)
provides native x64 and ARM64 downloads for GNU/Linux, macOS and Windows, with
pinned shell/PowerShell installers and checksums. Homebrew installation is
`brew install mylife-inc/tap/zel`. Windows users can install through the owned
[Scoop bucket](https://github.com/mylife-inc/scoop-bucket):

```powershell
scoop bucket add mylife-inc https://github.com/mylife-inc/scoop-bucket
scoop install mylife-inc/zel
```

Use `scoop update; scoop update zel` for that installation; `zel upgrade` preserves
package-manager ownership. Rich `zel build` downloads and caches its renderer;
portable web builds run natively. Neither needs installed build tools. Explicit
image builds use the release's matching container images. `zel dev` downloads
its optional development dependencies once and needs no separately installed
Docker, Node.js, npm or Python; `zel dev --offline` reuses verified packages.
Rich HTTP `--target static` builds use the same downloaded renderer as `file`. See the
[native distribution contract](specs/zel-native-distribution.md) for tested OS
versions, commands and limits.

`zel create my-book -t book-classic` creates a complete manuscript with a cover,
preface, part, chapters and appendix. `zel pdf my-book` generates a PDF only;
`zel bp my-book` builds the website with a PDF download. Both use the verified
cached compiler/private Node by default, without installed Docker, Node, Python
or TeX. `--pdf-backend container` retains the separate container toolchain. `zel bs` selects an HTTP
static export, `zel vw` previews edits, and `zel lint` runs deep/strict validation.
See the detailed [book Quickstart](examples/zellij-docs/content/01-guide/quickstart/books.mdx)
and [website-template Quickstart](examples/zellij-docs/content/01-guide/quickstart/website-templates.mdx),
for step-by-step creation, editing, validation, previewing and sharing.
PDF is off by default. Choose `book-technical` for mathematics and proofs or
`book-study` for passages and commentary. All three editions coordinate web/print
layouts. Print supports English, French, German and Spanish LTR and Arabic RTL,
with pinned Amiri/Libertinus fonts. `pdf.style` customizes colors, fonts, chapter
openings, spacing and margins without writing LaTeX.

Configured PDF delivery also works in Next images: `zel ni` and saved
`zel pub` include a matching downloadable PDF and receipt. Keyboard activation,
JavaScript-disabled readers and URL prefixes work; TeX sources and the toolchain
stay outside the serving image.

Books also support shared labelled equations, theorem/lemma/proof blocks, numeric
bibliography/citations, glossary/index and cross-chapter references for portable/Next websites and PDF, with stable anchors
and native validation. See the
[equation contract](specs/zellij-book-equations-2026-11-13.md) and
[statement contract](specs/zellij-book-statements-2026-11-13.md), plus the
[bibliography contract](specs/zellij-book-bibliography-2026-11-14.md) and
[glossary/index contract](specs/zellij-book-terms-2026-11-14.md).

The website catalog includes `product-launch`, `software-platform`
and `creative-studio`. Each includes two landing pages, two guide pages and local
artwork. Run `zel create my-site -t product-launch`, edit `content/`, then
`zel build my-site`. It downloads and caches its rich renderer, then generates
a website that opens by double-clicking `site/index.html`. The menu centers on desktop,
company branding stays at the far left, and search/theme controls stay at the
far right. On phones, sections open their own menu view with Back and Close.

The `kie` theme and `api-platform` starter are also included: `zel create my-api -t api-platform`.
The catalog contains nine starters. Product-launch, software-platform and
creative-studio use version 0.1.4; product-studio and api-platform use 0.1.3.
These revisions require rc.24 and document cached Next development; previously published
archives remain unchanged.

The [candidate notes](specs/releases/v0.2.0-rc.25.md) describe the desktop navigation
focus correction and the CLI help update. Cached Next development, offline reuse
and the isolated PDF runtime workaround remain included.
The short workflows, cached PDF default and matching compiler sources remain available. Cached editable sources
for default `eject` and `new app`, rich HTTP/file builds, validation and preview
remain included.
Wrapper generation needs only `zel`; running an editable Next app needs Node/npm. WinGet ownership protection and the x64/ARM64 manifest bundle remain
included. Microsoft catalog acceptance remains separate from this candidate's
verification; use Scoop or the pinned installer for the verified rc.25 release.
Standalone rich validation and live preview remain included. Downloaded resources share one home: `ZELLIJ_HOME`,
default `~/Dev/config/zellij`. Standalone installers use its
`bin/`; renderer, remote-template and editable-source downloads use
`cache/engines/`, `cache/templates/` and `cache/sources/`. Optional development
and PDF packages use `cache/development/` and `cache/pdf/`. Homebrew and Scoop
keep their own executable locations.
The section composer supports video, layers, semantic content, product selectors, scroll stories
and eight reusable built-in sections. Nine editable presets are available with
`zel presets`; generate one with `zel ns -p product-overview --out content/sections/overview.yaml`. See the
[release evidence](specs/evidence/zellij-rc11-release-2026-11-18.json) and
[completion ledger](specs/zel-cli-completion-2026-11-11.md) for verification and
deferred provider/runner checks. This is a preview, not a stable 1.0 release.

Maintainers: see the [release procedure](specs/zel-release-procedure.md) for
candidate channels and pinned installation. The [documentation publishing guide](specs/zellij-docs-deployment.md)
explains automatic updates from `main`, manual runs and rollback.

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

Install `zel` using one of the methods above, then create a website:

```sh
zel create my-docs -t product-launch
zel b my-docs
zel dev my-docs --open
```

The build creates `my-docs/site/index.html`, which opens directly from disk.
The development command serves the standard Next.js wrapper and reloads edits.
Both acquire verified packages under `ZELLIJ_HOME` and support `--offline` after
the first downloads. No source checkout or separately installed Node.js, Docker,
Python or npm is needed.

For container delivery, `zel si my-docs` packages a static website and
`zel ni my-docs` packages the Next wrapper. These need a container builder.
[`docker/site.Dockerfile`](docker/site.Dockerfile) is also available for an
existing Docker pipeline. It validates content before building, then retains
only the compiled serving app in the final image.

## The command line

[`cli/`](cli) is `zel`: one native binary for scaffolding a content folder,
converting one from another documentation tool, validating it, building it and
shipping it. Candidate rc.25 builds rich double-click websites with a downloaded,
verified renderer cached separately from the small CLI: no Docker, Node.js, npm
or Python installation is required. `zel build` selects this for rich sites;
`zel build --offline` requires the matching cache to be present. Explicit
`--target static` shares that cache for HTTP output; image commands retain
their container workflow. Commands that describe Zellij's content
model read the schema compiled into the native binary.

Candidate rc.25 also uses the cached renderer for rich `zel check`,
`zel watch`, `zel view` and `zel view --watch`. These commands accept
`--offline` for both `file` and `static`. Static deployment builds and optional
import validation also reuse the cache.
See [the implementation contract](specs/zellij-cached-http-export-2026-11-21.md).
The [cached-toolchain review](specs/zellij-cached-toolchains-2026-11-21.md) describes
which remaining commands could remove external prerequisites in future releases.

```bash
zel explain bento-grid      # every field, with a worked example
zel new section hero        # a section file, fields filled in
zel context reference.md    # the whole model, for an AI agent
```

[`specs/zellij-cli-spec.md`](specs/zellij-cli-spec.md) is the full surface.
