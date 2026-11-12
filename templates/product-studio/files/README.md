# Your Zellij site

Edit `content/` and `zel.yaml`. Generated output goes in `site/`.

```sh
zel check --strict
zel build
zel view --watch
```

The `.zel-template.json` receipt records the exact starter and original file
checksums. It does not prevent normal edits or automatically update your files.

This hosted template uses the existing hero, bento-grid and feature-grid
sections. Building/previewing it needs a supported container runtime and the
matching engine. It is not a direct-file or PDF template.
