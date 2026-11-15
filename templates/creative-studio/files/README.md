# Atelier North website starter

An editorial studio website with expressive typography, original project studies and a considered approach.

This studio uses the Porcelain theme, expressive serif headings and original geometric artwork. The home page introduces a practice; Selected work expands two fictional project studies. Begin with `content/sections/home/hero.yaml`, then replace the Field and Archive stories with your own work.

`content/pages.yaml` composes the pages from `content/sections/`. Edit those
YAML files and the Markdown guide; no React is required. Images live in
`content/assets/`, and output goes in `site/`. Read the included guide for
company branding, palettes, navigation, motion and artwork replacement.

```sh
zel check --strict
zel build
zel view --watch
```

Requires CLI and engine rc.9 or later, plus a supported container runtime for
hosted builds. Serve the result over HTTP; this is not a direct-file or PDF
starter. Creation is offline. The template receipt records original checksums
without preventing edits. No external assets or extra dependencies are needed.

All businesses, product names, interfaces and projects here are fictional
demonstrations. This is an original design, not a copy of another brand's site.
Replace the sample identity and content before publishing.
