# Zellij template catalog

This is the authored source for the catalog bundled in `zel` and attached to
CLI releases. It is independent of `examples/`. Version 1 contains three starters:
`product-docs` (portable handbook), `product-studio` (hosted product landing page)
and `book-classic` (portable reading edition; no PDF yet).

Each catalog directory has `template.yaml` and `files/`. Files under `files/`
become the created site. Its layout must use `zel.yaml`, `content/` and generated
`site/`. Do not commit generated output or personal configuration here.

## Manifest version 1

A manifest declares `schema: 1`, a lowercase ID, an exact semantic `version`,
title/description, minimum CLI and engine versions, renderer (`portable` or
`hosted`), kind (`site` or `book`), and outputs (`web`, optionally `file` for
portable sites). Books must explicitly declare `print: unavailable` until a
print renderer is implemented. These starters are not complete book-publication
models, and neither browser print nor a PDF download is promised.

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
remains the native-package inventory. Node and Python are packaging tools, not
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
