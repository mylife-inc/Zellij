# Forma One website starter

A sculptural product launch with cinematic imagery, material details and a quiet editorial rhythm.

The launch page moves from a large product illustration to a gradient headline, material details and an everyday setting. The separate Design page adds an accessible media accordion and frequently asked questions. Begin with `content/sections/home/hero.yaml` and replace the fictional Forma One story.

`content/pages.yaml` composes the pages from `content/sections/`. Edit those
YAML files and the Markdown guide; no React is required. Images live in
`content/assets/`, and output goes in `site/`. Read the included guide for
company branding, palettes, navigation, motion and artwork replacement.

```sh
zel build
zel build --offline
```

Creation is offline. The template receipt records original checksums without
preventing edits. No external artwork needs to be downloaded.

All businesses, product names, interfaces and projects here are fictional
demonstrations. This is an original design, not a copy of another brand's site.
Replace the sample identity and content before publishing.

Requires CLI rc.22 or later. `zel build` downloads a verified renderer once,
then generates the rich website without separately installed Docker, Node.js,
npm or Python. Double-click `site/index.html`; keep the complete `site/` folder
together. Later builds can use `zel build --offline` with the cached renderer.
`zel serve` optionally serves the existing output over HTTP.

Validate without building with `zel check --deep --strict`, or keep validating
edits with `zel watch --deep --strict`. Use `zel view --watch` to rebuild and
reload a local browser preview. These commands use the same cached renderer
and need no separately installed Docker, Node.js, npm or Python. Add `--offline`
after the first renderer download to require the verified cache.

HTTP exports also use the renderer cache: `zel bs` creates an HTTP-served
edition, while `zel b` creates the rich double-click edition.

For Next.js development, run `zel dev --open`. It downloads a separate verified
development package, reuses private Node and wrapper sources under
`$ZELLIJ_HOME` (default `~/Dev/config/zellij`), and reloads valid edits. No installed
Docker, Node.js, npm or Python is needed. After the first successful run,
`zel dev --offline --open` uses those verified packages without downloading.
Ctrl-C stops the server and removes its temporary session; downloads remain.
`zel vw` is the lighter watched static preview.

Image commands and explicit `zel dev --dev-backend container` need a container
builder/runtime. This template does not configure PDF output.
