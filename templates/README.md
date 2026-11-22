# Zellij template catalog

This is the authored source for the catalog bundled in `zel` and attached to
CLI releases. It is independent of `examples/`. The rc.18 catalog contains nine
starters. The five existing starters are:
`product-docs` (portable handbook), `product-studio` (hosted product landing page),
`book-classic` (portable book publication with optional LuaLaTeX PDF),
`book-technical` (mathematics and proofs) and `book-study` (passages and commentary).
The latter two start at template version 0.1.0 and require CLI/engine rc.8 or
later. They set `book.edition` for coordinated
web/print layouts and include citations, glossary and index examples. No artwork,
font or runtime dependency is added.
Product-docs remains at template version 0.1.0. Product-studio 0.1.2 requires
CLI rc.14 and documents standalone rich building, validation and live preview. Book-classic 0.2.0
requires CLI/engine 0.2.0-rc.6 and replaces the earlier reading-only starter in
the bundled catalog; previously published 0.1.0 archives remain immutable.

The catalog also includes three website starters introduced in rc.9 at template version 0.1.0:

| ID | Direction | Pages |
| --- | --- | --- |
| `product-launch` | Sculptural product imagery, Cupertino typography, restrained gradient reveals | Overview, Design with media accordion, two guide pages |
| `software-platform` | Dark software interface, connected workflow, practical product story | Product, Workflow, two guide pages |
| `creative-studio` | Editorial serif typography, geometric identity studies, generous spacing | Studio, Selected work, two guide pages |

These are original designs with local SVG art, not copies of Apple, Linear,
Stripe or another company's assets. They use the existing engine sections;
no React, external fonts, tracking, image services or new dependencies are added.
Their released 0.1.3 revisions require CLI rc.14 and retain the bento heading contrast fix.
Included in rc.18, they correct inline guide instructions for standalone
building, checking and previewing. Existing released 0.1.2 archive bytes are
unchanged. Their READMEs
cover rich `check`, `watch`, `view` and `view --watch` using the cached renderer.
Create them by ID, for example `zel create my-website -t product-launch`.
The rich builder downloads the matching renderer once. Local templates are also supported:

```sh
zel create my-website -t ./templates/product-launch --offline
```

All three have rich double-click output, optional HTTP viewing, light/dark theme
controls, an editable second landing page and a customization guide. They do not
configure PDF output. Creation works offline; builds reuse a renderer downloaded
once and need no separately installed Docker, Node.js, npm or Python. Explicit
HTTP static exports share that cache in rc.18; hosted `zel dev` still uses a
container. Real signup, purchase and contact services belong
to the author's product. The sample buttons point to functioning local pages.
Earlier published identities retain their original bytes; README updates use new versions.

Each catalog directory has `template.yaml` and `files/`. Files under `files/`
become the created site. Its layout must use `zel.yaml`, `content/` and generated
`site/`. Do not commit generated output or personal configuration here.

## Manifest version 1

A manifest declares `schema: 1`, a lowercase ID, an exact semantic `version`,
title/description, minimum CLI and engine versions, renderer (`portable` or
`hosted`), kind (`site` or `book`), and outputs (`web`, optionally `file` for
portable sites and `pdf` for print books). Books must explicitly declare
`print: unavailable` or `print: lualatex`. The latter requires a portable book,
`pdf` in outputs and minimum CLI rc.6 or later. Creation checks the expanded
manuscript against the native PDF adapter without running a toolchain. Enabled
PDF settings must be disclosed by the manifest; a false print declaration fails
before the destination is published. This capability does not enable PDF
automatically or promise successful typesetting of arbitrary subsequent edits.
The three print editions remain English/LTR; their optional Linux toolchain runs only
when PDF is requested.

`variables` maps names to default single-line text. `substitute` lists the exact
files eligible for expansion. Use `{{zel.title|yaml}}` for a complete YAML scalar
and `{{zel.author|markdown}}` for literal Markdown/MDX prose. Filters are mandatory;
unknown variables, duplicate overrides and unknown filters fail. Substitution
runs once and never evaluates code. Do not use these filters as HTML attributes,
URLs, JavaScript, shell commands or other languages.

`assets` lists local file paths and their license identifiers. Optional `preview`
names one of those assets. All referenced paths must exist beneath `files/`.
The starter illustration is original Shebka artwork under the current Zellij
product terms (`LicenseRef-Zellij`); this catalog does not change the product's
license or include third-party/Apple imagery.

## Authoring and verification

```sh
zel templates show ./templates/product-docs --json
zel create /tmp/my-template-test -t ./templates/product-docs --offline
zel check /tmp/my-template-test --strict
```

Use a new or empty destination, other than the current working directory. A
local template may contain only `template.yaml` and `files/`. Links, special files,
Git metadata, case collisions, Windows device names and escaping paths are
rejected. Limits: 512 regular files, 4 MiB per file, 32 MiB total expanded content,
32 directory levels for local templates, and 16 MiB per compressed archive.
Creation validates the site configuration and portable content before publishing.
Hosted creation validates YAML syntax and declared assets without launching an
engine; deep hosted schemas/routes are checked with `zel check --deep --strict`.

Build the CLI after editing a template. `scripts/check-site-templates.mjs`
compares bundled/local/archive content, checks deterministic packaging, exercises
an HTTP catalog and then reuses its cache with the server stopped. Hosted CI adds
real export and desktop/mobile browser checks and screenshots.

## Distribution

`scripts/package-templates.mjs <zel> <release-tag> <asset-directory>` requires the
matching CLI and source. It emits `templates-catalog.json` and deterministic
`template-<id>-<template-version>.tar.gz` archives. Each archive contains only
regular `template.yaml`/`files/…` entries. SHA-256 values are in the catalog;
archive URLs are relative filenames so identical assets work in both the source
release and the public `mylife-inc/releases` mirror. The native `SHA256SUMS`
also covers the separate renderer archives. Node and Python are packaging tools, not
requirements for template users.

Catalog JSON is `{schema: 1, release: "v…", templates: [{manifest, archive:
{url, sha256}}]}`. Entries use the manifest fields above, with the same kebab-case
keys. Catalogs are at most 1 MiB/128 entries and reject duplicate ID/version pairs.
All entries must be compatible with the current CLI/engine. IDs without a version
select the highest semantic version in the selected catalog; `id@version` selects
exactly. The archive manifest must equal the catalog's manifest.

The default is the embedded catalog, requiring no network or source repository.
Use `--catalog <HTTPS-URL>` for another release catalog. Downloads carry no
credentials; URLs must not contain credentials, queries or fragments. HTTPS
redirects are bounded; HTTP is allowed only for literal loopback testing.
`--offline` requires both the selected remote catalog and archive already in the
cache. Catalogs are keyed by URL and archives by SHA-256; every archive cache hit
is rechecked. `--cache-dir` overrides the OS cache directory.

A generated `.zel-template.json` records ID/version, CLI version, origin, source
checksum, variables and original generated-file checksums. Edit those files
normally. No template hooks run, and no automatic update rewrites a site or book.
Review custom hosted MDX as executable content before a later hosted build.
Release retirement can remove older catalogs; bundled/local/cached copies continue
to work. Never reuse an already published template version for different bytes.
`published.json` records previously published archive hashes. Packaging uses an
explicit gzip level and refuses different bytes for a known ID/version; record
new hashes only after public release verification, without rewriting old entries.

The website starters center the product home link and menu on the viewport.
Optional `brand.company` stays at the far left; search and theme controls stay
at the far right. The mobile header uses the complete navigation drawer.

## API platform in rc.10

`api-platform` uses the `kie` theme, with a wide introduction, pricing,
model cards, integration steps and a code example. Its four pages and original
SVG artwork use no remote assets or services. Its current 0.1.2 revision requires
CLI rc.14; the design uses features introduced in engine rc.10. The rc.18 catalog
includes nine starters. New template versions update build instructions without
rewriting older archive identities. The API, plans and product names are fictional examples.
