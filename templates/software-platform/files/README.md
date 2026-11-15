# Relay website starter

A focused software website with a product interface, connected workflows and a practical guide.

This starter pairs a compact software introduction with a large interface showcase and a connected workflow. Begin with `content/sections/home/hero.yaml`, replace `content/assets/workspace.svg` with a real product capture, and turn the sample Workflow page into your own product tour.

`content/pages.yaml` composes the pages from `content/sections/`. Edit those
YAML files and the Markdown guide; no React is required. Images live in
`content/assets/`, and output goes in `site/`. Read the included guide for
company branding, palettes, navigation, motion and artwork replacement.

```sh
zel check --strict
zel build
zel view --watch
```

Requires CLI and engine rc.8 or later, plus a supported container runtime for
hosted builds. Serve the result over HTTP; this is not a direct-file or PDF
starter. Creation is offline. The template receipt records original checksums
without preventing edits. No external assets or extra dependencies are needed.

All businesses, product names, interfaces and projects here are fictional
demonstrations. This is an original design, not a copy of another brand's site.
Replace the sample identity and content before publishing.
