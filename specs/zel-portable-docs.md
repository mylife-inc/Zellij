# zel documentation workflow and portable HTML requirements

Status: requirement recorded 2026-10-17; native portable build implemented in
this milestone, with hosted cross-platform acceptance pending. `serve` and
`view` remain planned. This extends the Next-based static export with a separate
Rust renderer; the target does not depend on the Next router.

The user wants MkDocs-equivalent documentation workflows, without requiring the
same command syntax, including these three operations:

| Operation | Required behavior |
| --- | --- |
| Build a Markdown folder | Discover Markdown and local assets, generate a navigable static site, and produce an `index.html` that can be opened directly without an HTTP server. |
| Serve a generated site | Start a local HTTP server for an existing static output directory. This remains useful even though direct-file viewing also works. |
| View a Markdown folder | Build the site, start the local server, and open its actual URL in the default browser. Compose the same build/serve implementations. |

The build interface is implemented; the remaining proposed commands are `zel build <docs> --out <site>`, `zel serve <site>`, and
`zel view <docs>`. Default target selection preserves hosted builds when structured content is
discovered and otherwise uses portable Markdown. `--target portable` selects it
explicitly; `--target static` retains the existing HTTP-only export.

## Markdown input

A plain folder of `.md` files, including nested documents and relative assets,
must work without hand-authoring Zellij `_sidebar.yaml`, React or Next files.
Define deterministic ordering, title/home discovery and optional navigation
configuration. Preserve authored inputs. Translate relative Markdown links,
including fragments, to the actual generated HTML paths. Reuse conversion and
content-model code where their contracts match; do not require a MkDocs config.

## Direct-file acceptance

Automate opening `file:///.../index.html` in supported browsers with no server
running and network access blocked. Also open a nested page directly and move
the entire output folder to another path (including spaces and Unicode).

Verify:

- Home, nested pages, sidebar/menu, breadcrumbs, previous/next and heading anchors.
- Local images, styles, fonts, syntax highlighting and enabled math/diagrams.
- Search and its result navigation without HTTP fetches for an index or router data.
- Keyboard operation, theme controls and essential disclosure/navigation widgets.
- No dependence on absolute deployment-root paths, a CDN, browser security flags,
  a service worker, or an already-running local server.

Core documentation should remain readable with JavaScript disabled. External
links and explicitly remote media remain external; document their offline limits
rather than silently claiming the whole internet is included in the output.

The current engine's Next router, root-relative links/chunk URLs and fetched
search index assume HTTP. A normal Next static export is therefore not evidence
of this requirement. Evaluate a portable rendering target or an intentionally
separate browser runtime; a blanket string replacement of HTML URLs is insufficient.

## HTTP serve/view acceptance

Serve on loopback by default; support a selected port and report the actual URL.
Handle occupied ports, missing/invalid output, nested routes, MIME types and
clean Ctrl-C shutdown. Contain requests to the served directory, including
encoded traversal and symlink cases. Serve the portable artifact unchanged.
`view` opens a browser only after a successful build and a ready server; browser
launch failure must leave a usable URL. Provide a no-open option for automation.
Exercise macOS, Linux and native Windows, including paths with spaces/Unicode.

MkDocs' development server also rebuilds on edits. Keep build/serve/view and the
existing `zel dev` responsibilities explicit; watch/rebuild/live reload belongs
in the capability comparison, even if static `serve` itself does not watch.

## MkDocs comparison to maintain during CLI implementation

Track core capabilities: scaffold a project; Markdown/config/navigation/themes;
static builds and alternate output directories; strict checks and diagnostics;
development serving, watching and reload; assets/search/TOC; deploy/publish
workflows; command help and version reporting. Plugin-specific integrations are
separate capabilities, not an implied promise of compatibility with every plugin.
The user explicitly prioritized build, serve and view above; do not stop at
matching command names or accepting flags with no implementation.

MkDocs already documents direct-file viewing with `site_url: ""` and
`use_directory_urls: false`. Its ordinary search plugin must be disabled or
replaced, and themes may need local support files. Zellij should test its
portable experience, including search, instead of relying on those assumptions.

Primary references checked 2026-10-17:

- [MkDocs command reference](https://www.mkdocs.org/user-guide/cli/)
- [MkDocs local-file deployment](https://www.mkdocs.org/user-guide/deploying-your-docs/#local-files)
- [MkDocs URL configuration](https://www.mkdocs.org/user-guide/configuration/#use_directory_urls)
- Installed Next documentation: `node_modules/next/dist/docs/01-app/02-guides/static-exports.md`

## Implemented native build contract — 2026-10-17

`zellij-docs` uses pulldown-cmark and build-time syntect highlighting. All HTML,
relative links, search data, theme CSS, JavaScript enhancements and 75 pinned
font files are produced by the binary. No process, Node, Docker or network is
needed at runtime. Shared staged publication in `zellij-core::output` preserves
the existing export when validation or rendering fails.

- `.md`/`.markdown`, nested folders and referenced local assets are supported;
  hidden entries and node_modules are excluded, nested symlinks rejected.
- Index documents win over README for each directory's `index.html`. A missing
  root home is synthesized. Home sorts first, followed by numeric front-matter
  `order` and output path. Titles use front matter, first H1, then filename.
- Local document links and heading/footnote fragments are resolved and validated.
  Output uses actual `.html` files and percent-encoded relative URLs. Unsafe or
  conflicting portable filenames fail before publication.
- Navigation, breadcrumbs, TOC, previous/next, images, tables, task lists,
  footnotes, code colors and ordinary links do not require JavaScript. Search,
  themes and mobile menu controls enhance these pages through local scripts.
- Default output is the sibling `<input-name>-site`. Input and output must be
  disjoint. `--name` controls the header name; `--theme` selects from the shared
  14-theme catalog, defaulting to cupertino.
- Raw HTML is literal text with a warning; `.mdx` is rejected. No math rendering
  or Mermaid execution is enabled. Remote media remains remote. MkDocs plugins,
  custom navigation/config, full-text search language processing and incremental
  rebuild/live reload are not implemented for this target.

Acceptance lives in `zellij-docs/tests/portable.rs`, the CLI's
`tests/portable_build.rs`, and `scripts/check-portable.mjs`. The browser fixture
builds with external tools absent from PATH, moves the output to a Unicode/space
path, opens home and nested files offline, checks search/anchors/fonts/media/
keyboard/themes/mobile and JavaScript-disabled content, and verifies failed
rebuild preservation. CI runs native builds and Chromium/WebKit on Linux,
macOS and Windows. Successful runs establish that workflow, not native Windows
installation, release packaging or container integration.

## Remaining capability comparison

| Capability | Native portable status | Next work |
| --- | --- | --- |
| Plain-folder static build and alternate output | Implemented | Expand real-document fixtures and rendering compatibility |
| Markdown, titles, automatic navigation, themes | Core implemented | Explicit navigation/config, raw HTML policy, math/diagrams |
| Assets, TOC, offline search | Implemented for local Markdown references | Richer search/navigation where needed |
| Strict diagnostics | Broken references/anchors/metadata fail build | Separate check command, warnings gate and structured reports |
| Project scaffolding | Existing structured Zellij scaffolds | Decide portable starter/config workflow |
| Serve an existing export | Planned | Loopback HTTP, MIME/routing, request containment and lifecycle |
| View a source folder | Planned | Compose build/server, ready URL, browser launch and no-open |
| Development rebuild/reload | Existing hosted dev workflow only | Portable watch/rebuild and reload acceptance |
| Deploy/publish | Existing images and generated provider workflows | Finish chosen CLI providers and release scope |
| Native distribution | Portable OS CI added | Release/install smoke tests, Windows packaging/installer |
