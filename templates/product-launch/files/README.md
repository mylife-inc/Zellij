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

Requires CLI rc.12 or later. `zel build` downloads a verified renderer once,
then generates the rich website without separately installed Docker, Node.js,
npm or Python. Double-click `site/index.html`; keep the complete `site/` folder
together. Later builds can use `zel build --offline` with the cached renderer.
`zel serve` optionally serves the existing output over HTTP.

Optional live development (`zel view --watch`), explicit `--target static` HTTP
exports, image commands and standalone `zel check` still use a supported
container runtime and matching engine. Normal `zel build` validates content
automatically. This template does not configure PDF output.
