# API platform

Create with `zel create my-api -t api-platform` using CLI rc.12 or later.
Edit `content/site.yaml`, `content/pages.yaml` and `content/sections/`.
Start with the local guide, then run `zel build`.

All product claims and prices are examples. No API, payment or signup service
is provided. The Kie theme follows kie.ai's visual language, with original
sample content and art. Previously published templates are unchanged.

Requires CLI rc.12 or later. `zel build` downloads a verified renderer once,
then generates the rich website without separately installed Docker, Node.js,
npm or Python. Double-click `site/index.html`; keep the complete `site/` folder
together. Later builds can use `zel build --offline` with the cached renderer.
`zel serve` optionally serves the existing output over HTTP.

Optional live development (`zel view --watch`), explicit `--target static` HTTP
exports, image commands and standalone `zel check` still use a supported
container runtime and matching engine. Normal `zel build` validates content
automatically. This template does not configure PDF output.
