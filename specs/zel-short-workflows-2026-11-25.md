# Short everyday workflows and use-case Quickstart

Status: implemented in `98a2cff` for rc.21 source; published rc.20 keeps its previous defaults.
This is a follow-up to the dated CLI contract, not a rewrite of its audit history.

## Product decision

The user requested `zel pdf` for the long cached PDF recipe, an audit of other
long commands, and detailed Quickstart use cases for books and website templates.
The existing `pdf` alias already means `export pdf` and writes `pdf/book.pdf`.
Preserve that meaning: it does not replace or silently regenerate a website.

After rc.20's successful six-platform public-download qualification, rc.21
selects the verified cached PDF backend by default. `zel pdf` therefore needs
no backend flag or separately installed Docker, Node.js, Python or LaTeX.
The explicit flag overrides `ZEL_PDF_BACKEND`, which overrides the default.
`--pdf-backend container` retains the isolated container compiler.
PDF remains opt-in; ordinary builds still honor `pdf.enabled` and `--no-pdf`.

## Command audit and resulting surface

| Command family | Short everyday form | Decision |
| --- | --- | --- |
| PDF-only export | `pdf` | Existing alias; cached default removes the long backend option |
| Website and PDF | `bp` | New preset for `build --pdf` |
| Rich HTTP export | `bs` | New preset for `build --target static` |
| Rich double-click / portable reading output | `b` | Existing default output and alias already short |
| Build, serve, open and watch | `vw` | New preset for `view --watch` |
| Thorough validation | `lint` | New preset for `check --deep --strict` |
| Check / serve / one-time view | `c`, `s`, `v` | Existing aliases; meanings unchanged |
| Template creation | `create -t ID` | Existing alias and short option; no new synonym |
| Portable / rich starter | `init`, `nd` | Existing aliases and folder convention |
| App / page / guide / project / section | `na`, `np`, `ng`, `npr`, `ns` | Existing aliases |
| Static / Next image | `si`, `ni` | Existing artifact-specific aliases; builder still required |
| Image run / push / recipe | `ir`, `ip`, `idf` | Existing aliases; explicit safety options remain |
| Provider delivery | `netlify`, `cf`, `vc`, `ghp`, `s3`, `fly`, `rw` | Existing aliases; production intent remains explicit |
| Saved publication | `pub` | Save supported destination/settings instead of repeating flags |
| CI generation / dispatch | `ci`, `ci run` | Already short; no provisioning hidden inside an alias |
| Conversion / migration | `cv`, `mig` | Existing aliases; preserve layout/output safety |
| Discovery / AI context | `sections`, `presets`, `themes`, `formats`, `targets`, `ex`, `ctx` | Existing aliases, now all in the reference table |
| Completion | `comp` | Existing alias for five supported shell formats |
| Other operations | `dev`, `fmt`, `watch`, `ship`, `pull`, `doctor`, `version`, `upgrade`, `eject`, `detect`, `templates` | Single useful names; avoid extra abbreviations without a recurring long recipe |

Inside the site root, omit the source argument. From a repository root the
`docs/zellij/zel.yaml` convention also permits bare commands. A differently
named folder remains fully supported with an explicit path or global selector.
Saved PDF/image/publication/CI settings remove options where the configuration
already defines them; aliases do not invent new implicit deployment authority.

## Implementation and compatibility

Build, view and check share Clap argument types between ordinary commands and
presets. Normalization runs before global site selection and dispatch. The same
handler owns validation, atomic publication, watch/cancellation and exit status.
`bp --no-pdf` and `bs --target file/portable/image` fail during argument parsing.
Repeating the preset's own implied option is accepted. Full help lists ordinary
aliases and every preset without overwriting earlier help. All five completion
formats use the same command tree.

Generated GitHub/GitLab/CircleCI PDF jobs pinned to rc.21 or later explicitly
select the cached backend and omit the PDF-only Docker preflight. Earlier pins
retain their previous behavior. Image jobs still require their builder. The
standalone Docker recipe and workspace version move together to rc.21.

Container PDF acceptance explicitly selects `container`, including static and
Next image tests and multilingual print. Real cached acceptance removes the
backend environment setting, exercises plain `pdf` and offline `bp`, verifies
complete matching sources, and shares the real private renderer with a rich
website build while the child PATH contains no system tools.

Book-classic becomes 0.3.0; technical and study become 0.2.0. Their README
instructions require CLI rc.21, while their manuscript and engine format minima
stay unchanged. Previously published template identities and hashes are not
rewritten. The existing scoped PDF licensing/distribution requirements remain.

## Documentation

Quickstart has separate book/PDF and designed-website walkthroughs, plus the
existing creation, import, new-repository and existing-repository labs. The new
pages explain template choice, source trees, editing, reading order/sidebar
updates, live previews, output locations, offline operation and delivery.
Rc.20-compatible command forms remain explicit until rc.21 is published.
Optional container setup follows the main use cases; native deployment no
longer claims to need a future command or an installed provider CLI.
Existing public routes/headings remain; preservation checks include both new
routes. Shortcut, book, template, build, doctor and CI reference pages explain
the changed default and the distinction between PDF-only and combined output.

## Verification

Local workspace testing passes **856 tests**, with five existing hosted-renderer
tests intentionally ignored in the local run. Clippy passes with warnings denied.
Parser coverage includes every registered alias/preset, global selectors,
conflicts, help and five completion formats. Generated workflows pass all three
systems with old/new PDF pins; the six checked-in acceptance workflows match the
generator and retain their published execution pins.

All nine templates pass deterministic archive generation, local/bundled/remote
parity, exact versions, offline creation and strict validation. The real cached
compiler/private renderer passes `pdf` and offline `bp` with an empty child PATH,
complete corresponding-source verification and a subsequent rich website build.
See [the local package receipt](evidence/zellij-rc21-short-pdf-local-2026-11-25.json).
It qualifies local bytes, not a public release download.

Deep/strict documentation validation and a production export pass. Browser
preservation covers 46 original plus 15 added routes, 282 original heading IDs,
82 compatibility mappings and 29 byte-identical images with no captured
hydration errors. Six failure-detection regressions and 20 release-policy checks
also pass. Hosted CI supplies separate exact-commit cross-platform evidence.
A passing candidate is not a claim of zero possible bugs.

PR #257's first hosted run found a remaining `book-classic@0.2.0` pin in the
container PDF acceptance script after this change moved the bundled starter to
0.3.0. The Docker ARM and Podman jobs failed at template creation after their
earlier technical/study typesetting checks. The script now selects 0.3.0; the
historical archive remains unchanged. Hosted verification is repeated on the
corrected head rather than treating those failures as compiler regressions.
