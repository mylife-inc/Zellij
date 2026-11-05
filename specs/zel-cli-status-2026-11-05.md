# zel CLI implementation status — 2026-11-05

All registered command families have implementation handlers. The full CLI
design is **not complete**. Provider, format and advanced-option coverage must
be assessed separately from the existence of a command. This is the current
summary; the [original inventory](zel-cli-inventory-2026-10-17.md) retains the
dated audit and implementation history.

| Area | Implemented command forms | Remaining scope or limits |
| --- | --- | --- |
| Creation | `new portable` / `init`, `new docs`, `new app`, `new section`, `new page`, `new guide`, `new project`, with documented aliases | Interactive authoring and proposed per-field wizard flags remain unimplemented. |
| Inspection | `explain`, `list sections/themes/formats/targets`, `context`, `detect`, `doctor`, `version` | Doctor principally diagnoses container tooling; it is not a universal provider/site diagnostic. |
| Validation and local work | `check`, `fmt`, `watch`, `dev`, `build`, `serve`, `view` | Hosted operations need their documented runtime/engine prerequisites. JSON is supported per command, not universally. |
| Images | `image static`, `next`, legacy `build`, `dockerfile`, `run`, `push`; shortcuts including `si` and `ni` | Linux amd64/arm64, including combined publication through publish/ship/CI. These commands require a container runtime. |
| Delivery | `deploy`, `netlify`, `publish`, `ship` | Netlify static deployment and image registry publication work. Cloudflare, Vercel, GitHub Pages, S3, Fly and Railway adapters remain unimplemented. |
| CI | `ci`, `ci init github`, `ci run` and legacy `ci publish` | GitHub Actions generation/dispatch work; GitLab/CircleCI adapters remain unimplemented. |
| Imports | `convert` / `cv`, `migrate` / `mig`; shared conversion in `ship` | MkDocs, Docusaurus and declarative VitePress. Sphinx and Jekyll/Hugo converters remain planned. Plain Markdown is directly supported by the portable renderer. `--assets link` is not implemented. |
| Maintenance | `pull`, `upgrade`, `eject`, `features`, `completions` | Supported operations have documented runtime, version and filesystem requirements; they do not imply automatic dependency installation. |

Dispatch now handles every command explicitly. Only already-normalized aliases
are unreachable; adding a new command variant without a handler is a compile
error. The former generic “not implemented” catch-all was not reachable through
current parsed commands and has been removed. Existing alias tests exercise the
normalization contract.

Import website-link work adds root-relative and same-origin HTTP(S) Markdown
URL mapping for supported source prefixes/routes, preserving queries, fragments,
Unicode and asset bytes. Docusaurus routing is read only from direct literal
configuration. Relative document URLs now use source page URL directories and
report ambiguous slash policies or conflicting file interpretations. Versioned,
localized, multi-instance and custom plugin routing, HTML/JSX attributes and
relative website assets still require review. No source JavaScript is executed
and no URL is fetched. See [the conversion reference](../examples/zellij-docs/content/02-cli/convert.mdx).

VitePress imports accept literal `.vitepress/config.{js,ts,mjs,mts}` modules
and the standard `defineConfig` wrapper without evaluation. They read `srcDir`,
nested/multiple sidebars, page metadata, `base`/`cleanUrls` links and referenced
source/public assets. Dynamic config, rewrites, locales, exclusion patterns and
custom output/cache paths are rejected before writes. Vue, home layout and
custom Markdown presentation remain manual migration tasks. VitePress is new in
source; published rc.1 does not include it.

The newest published native candidate remains **`v0.2.0-rc.1`**. Current source
is newer and has not been published as rc.2. The old rc.2 publication proposal
predates subsequent fixes and should not be used unchanged. Production Netlify
migration also remains separate, pending the reported hosting-comment hydration
issue. Passing tests do not establish zero bugs or completion of the whole design.

PR #74 merged as `74abf77a6592633288692a1af21fee9905f3ace6` after all 16
checks passed on `5846c16c698aaa1212df6a42a89afe83c54aa877` in
[41991356062](https://github.com/mylife-inc/Zellij/actions/runs/41991356062).
Both post-merge workflows passed: main `41995023512`, base image `41995023365`.
This document's website-link milestone must still pass its own final-head CI
before merge; the previous PR's results do not validate new changes.

The first website-link CI run found a Windows MkDocs path-normalization bug.
Source `74bacea` fixes nested-page inventory and derived navigation paths;
final-head Windows and full CI acceptance are pending.

**PR #75 closure:** all 16 final-head checks passed in
[42001094217](https://github.com/mylife-inc/Zellij/actions/runs/42001094217) on
`bb2527a5f50b3b9ecd870c5bb5e3a9f9c20962b1`. It merged as
`2c5ac7caae98af6bf4ec552c467d2614904ef1fe`, with the identical tested source
tree `185ce605b71138a58d700b8e7a6c9e9f1850f77c`. Base-image post-merge run
`42005279024` passed; main `42005279080` was still running at this refresh.
The new relative-link milestone requires its own final-head verification.

The PR #75 post-merge Windows preview step subsequently failed an immediate
watch-announcement assertion. Source `a1003bd` waits for the separately relayed
stdout message, retaining bounded readiness, early-exit checks and the assertion.
This harness fix joins the relative-link PR and requires new Windows CI evidence.

**PR #76 closure:** relative website links and the Windows watch-readiness fix
merged as `e8f525d` after all 16 checks passed on `903c66d` in run `42007978630`.
Post-merge base-image run `42011273390` passed; main `42011273809` has 15 passing
jobs with CLI acceptance still running. New VitePress source `38b9d41` passes
140 local Rust tests, Clippy/formatting, deep/strict docs and all three Node
import-rendering fixtures. Its own hosted/native CI and merge remain pending.
