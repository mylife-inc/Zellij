# Your Zellij book

Edit `content/` and `zel.yaml`. Generated output goes in `site/`.

```sh
zel lint       # Validate the manuscript without a runtime
zel build               # Generate site/; open site/index.html directly
zel vw         # Preview and rebuild while you write
zel pdf --check          # Check the classic print adapter without typesetting
```

The `.zel-template.json` receipt records the exact starter and original file
checksums. It does not prevent normal edits or automatically update your files.

Edit the title, author, cover and reading order under `book` in `zel.yaml`.
The title page and table of contents are generated from this manifest. Add
chapters under `content/`, then list them in the manifest; keep unpublished
drafts outside `content/`. This starter includes a preface, one part, two
chapters and an appendix. Replace the sample prose and original cover artwork.

PDF is optional and disabled by default. With zel 0.2.0-rc.21 or later:

```sh
zel bp          # Publish matching web/PDF output and a download link
zel pdf                 # Export only PDF and print sources to pdf/
```

Set `pdf.enabled: true` for ordinary builds, previews and static deliveries to
include PDF. Use `--no-pdf` for a web-only build or preview session. Paper accepts
`a5`, `a4` or `letter`; `font-size` accepts 10, 11 or 12 points. The verified compiler and private runtime download on first use into
`$ZELLIJ_HOME` (default: `~/Dev/config/zellij`). Later builds reuse the cache;
`zel bp --offline` forbids downloads. Docker, Node.js, Python and a system LaTeX
installation are not required. `--pdf-backend container` selects the optional
container compiler explicitly; it needs a running Linux container runtime.

The print adapter supports the documented Markdown/math subset, bibliography,
index, glossary, numbered equations and theorem blocks. Configure language,
direction and print design in `zel.yaml`; see https://zellij.shebka.net/cli/books.
PNG/JPEG covers work in web and print. Compatibility checks validate authoring;
run a full PDF build and inspect the result after changing the manuscript. `site/` and `pdf/` are generated outputs: edit `content/` instead.
