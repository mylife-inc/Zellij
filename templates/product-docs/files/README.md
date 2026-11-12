# Your Zellij site

Edit `content/` and `zel.yaml`. Generated output goes in `site/`.

```sh
zel check --strict
zel build
zel view --watch
```

The `.zel-template.json` receipt records the exact starter and original file
checksums. It does not prevent normal edits or automatically update your files.
