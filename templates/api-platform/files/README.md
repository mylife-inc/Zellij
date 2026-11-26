# API platform

Create with `zel create my-api -t api-platform` using CLI rc.24 or later.
Edit `content/site.yaml`, `content/pages.yaml` and `content/sections/`.
Start with the local guide, then run `zel build`.

All product claims and prices are examples. No API, payment or signup service
is provided. The Kie theme follows kie.ai's visual language, with original
sample content and art. Previously published templates are unchanged.

Requires CLI rc.24 or later. `zel build` downloads a verified renderer once,
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
