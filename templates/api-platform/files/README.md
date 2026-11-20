# API platform

Create with `zel create my-api -t api-platform` using CLI rc.14 or later.
Edit `content/site.yaml`, `content/pages.yaml` and `content/sections/`.
Start with the local guide, then run `zel build`.

All product claims and prices are examples. No API, payment or signup service
is provided. The Kie theme follows kie.ai's visual language, with original
sample content and art. Previously published templates are unchanged.

Requires CLI rc.14 or later. `zel build` downloads a verified renderer once,
then generates the rich website without separately installed Docker, Node.js,
npm or Python. Double-click `site/index.html`; keep the complete `site/` folder
together. Later builds can use `zel build --offline` with the cached renderer.
`zel serve` optionally serves the existing output over HTTP.

Validate without building with `zel check --deep --strict`, or keep validating
edits with `zel watch --deep --strict`. Use `zel view --watch` to rebuild and
reload a local browser preview. These commands use the same cached renderer
and need no separately installed Docker, Node.js, npm or Python. Add `--offline`
after the first renderer download to require the verified cache.

Explicit `--target static` HTTP exports, `zel dev` and image commands retain
their container prerequisites. This template does not configure PDF output.
