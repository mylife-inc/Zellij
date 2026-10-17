# zel documentation workflow and portable HTML requirements

Status: user requirement recorded 2026-10-17; not yet implemented. This extends
the CLI scope beyond the current Next-based static export. Command spellings
below are proposed interfaces, not claims about available commands.

The user wants MkDocs-equivalent documentation workflows, without requiring the
same command syntax, including these three operations:

| Operation | Required behavior |
| --- | --- |
| Build a Markdown folder | Discover Markdown and local assets, generate a navigable static site, and produce an `index.html` that can be opened directly without an HTTP server. |
| Serve a generated site | Start a local HTTP server for an existing static output directory. This remains useful even though direct-file viewing also works. |
| View a Markdown folder | Build the site, start the local server, and open its actual URL in the default browser. Compose the same build/serve implementations. |

Possible commands are `zel build <docs> --out <site>`, `zel serve <site>`, and
`zel view <docs>`. Final output-mode naming and defaults must keep existing
hosted website builds compatible. The portable build is a required first-class
capability; it must not be represented as already supported by today's export.

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
