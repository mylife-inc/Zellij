# Your technical book

Start with the preface and two chapters in `content/`. Edit `book.title`,
`book.authors`, subtitle and reading order in `zel.yaml`. Keep drafts outside
`content/`; every discovered Markdown file belongs in the reading order.

```sh
zel check --strict
zel build                # Open site/index.html directly; no server needed
zel view --watch         # Browser preview while writing
zel pdf --check          # Validate print compatibility without a toolchain
zel build --pdf          # Optional matching PDF and website download link
```

`book.edition: technical` selects coordinated web and print typography, chapter
openings and accent colors. Switch to `classic`, `technical` or `study` without
changing the manuscript. Web readers can choose their theme and light/dark mode.
The PDF uses the existing embedded Latin Modern family; web uses bundled fonts.
The formats share a design direction, not identical pagination or font files.

PDF is off by default. Requesting PDF needs a running Docker, Podman or nerdctl
Linux container runtime. Nothing is installed automatically. `zel pdf` exports
only the PDF, generated TeX and logs into `pdf/`. You may set `pdf.enabled: true`
for ordinary builds. Paper: `a5`, `a4` or `letter`; font size: 10, 11 or 12 points.
Print currently supports English/LTR and the documented Markdown/math subset.
Use local PNG/JPEG artwork if adding `book.cover`; raw TeX/HTML is not executed.

The sample bibliography entry is explicitly an illustrative, unpublished note;
replace it with real sources. Definitions and index entries live in the JSON
catalogs. Numbering, cross references and PDF page references are generated.
The passages are original sample writing, not quotations from an existing book.

Created files are yours to edit. `.zel-template.json` records the starter version
and original checksums; it does not update or overwrite your later edits.
Generated `site/` and `pdf/` are replaceable; author under `content/`.
