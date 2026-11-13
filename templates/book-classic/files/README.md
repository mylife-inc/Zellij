# Your Zellij book

Edit `content/` and `zel.yaml`. Generated output goes in `site/`.

```sh
zel check --strict       # Validate the manuscript without a runtime
zel build               # Generate site/; open site/index.html directly
zel view --watch         # Preview and rebuild while you write
zel pdf --check          # Check the classic print adapter without typesetting
```

The `.zel-template.json` receipt records the exact starter and original file
checksums. It does not prevent normal edits or automatically update your files.

Edit the title, author, cover and reading order under `book` in `zel.yaml`.
The title page and table of contents are generated from this manifest. Add
chapters under `content/`, then list them in the manifest; keep unpublished
drafts outside `content/`. This starter includes a preface, one part, two
chapters and an appendix. Replace the sample prose and original cover artwork.

PDF is optional and disabled by default. With a running Docker, Podman or
nerdctl Linux container runtime:

```sh
zel build --pdf          # Publish matching web/PDF output and a download link
zel pdf                 # Export only PDF and print sources to pdf/
```

Set `pdf.enabled: true` for ordinary builds, previews and static deliveries to
include PDF. Use `--no-pdf` for a web-only build or preview session. Paper accepts
`a5`, `a4` or `letter`; `font-size` accepts 10, 11 or 12 points. No toolchain is
installed automatically, and normal website images do not include LaTeX.

The classic print adapter supports English/LTR and its documented Markdown/math
subset. PNG/JPEG covers work in web and print. Other writing systems, custom
print designs, bibliography/index/glossary and numbered theorem semantics remain
future work; PDF compatibility checks do not promise successful typesetting
for every edit. `site/` and `pdf/` are generated outputs: edit `content/` instead.
