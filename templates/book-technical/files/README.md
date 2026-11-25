# Your technical book

Start with the preface and two chapters in `content/`. Edit `book.title`,
`book.authors`, subtitle and reading order in `zel.yaml`. Keep drafts outside
`content/`; every discovered Markdown file belongs in the reading order.

```sh
zel lint
zel build                # Open site/index.html directly; no server needed
zel vw         # Browser preview while writing
zel pdf --check          # Validate print compatibility without a toolchain
zel pdf                  # PDF only: pdf/book.pdf
zel bp          # Optional matching PDF and website download link
```

`book.edition: technical` selects coordinated web and print typography, chapter
openings and accent colors. Switch to `classic`, `technical` or `study` without
changing the manuscript. Web readers can choose their theme and light/dark mode.
The PDF uses the existing embedded Latin Modern family; web uses bundled fonts.
The formats share a design direction, not identical pagination or font files.

PDF is off by default. In zel 0.2.0-rc.21 or later the verified compiler and
private runtime download on first use into `$ZELLIJ_HOME` (default:
`~/Dev/config/zellij`). No Docker, Node.js, Python or system LaTeX is needed.
Use `zel bp --offline` after the first successful build to forbid downloads.
`--pdf-backend container` selects the optional container compiler explicitly. `zel pdf` exports
only the PDF, generated TeX and logs into `pdf/`. You may set `pdf.enabled: true`
for ordinary builds. Paper: `a5`, `a4` or `letter`; font size: 10, 11 or 12 points.
Print supports the documented Markdown/math subset. For languages, direction
and custom print designs, see https://zellij.shebka.net/cli/books.
Use local PNG/JPEG artwork if adding `book.cover`; raw TeX/HTML is not executed.

The sample bibliography entry is explicitly an illustrative, unpublished note;
replace it with real sources. Definitions and index entries live in the JSON
catalogs. Numbering, cross references and PDF page references are generated.
The passages are original sample writing, not quotations from an existing book.

Created files are yours to edit. `.zel-template.json` records the starter version
and original checksums; it does not update or overwrite your later edits.
Generated `site/` and `pdf/` are replaceable; author under `content/`.
