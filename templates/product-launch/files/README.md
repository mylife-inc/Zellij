# Forma One website starter

A sculptural product launch with cinematic imagery, material details and a quiet editorial rhythm.

The launch page moves from a large product illustration to a gradient headline, material details and an everyday setting. The separate Design page adds an accessible media accordion and frequently asked questions. Begin with `content/sections/home/hero.yaml` and replace the fictional Forma One story.

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
