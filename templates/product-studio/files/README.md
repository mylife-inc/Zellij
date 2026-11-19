# Your Zellij site

Edit `content/` and `zel.yaml`. Generated output goes in `site/`.

```sh
zel build
zel build --offline
```

The `.zel-template.json` receipt records the exact starter and original file
checksums. It does not prevent normal edits or automatically update your files.

This rich template uses hero, bento-grid and feature-grid sections.

Requires CLI rc.12 or later. `zel build` downloads a verified renderer once,
then generates the rich website without separately installed Docker, Node.js,
npm or Python. Double-click `site/index.html`; keep the complete `site/` folder
together. Later builds can use `zel build --offline` with the cached renderer.
`zel serve` optionally serves the existing output over HTTP.

Optional live development (`zel view --watch`), explicit `--target static` HTTP
exports, image commands and standalone `zel check` still use a supported
container runtime and matching engine. Normal `zel build` validates content
automatically. This template does not configure PDF output.
