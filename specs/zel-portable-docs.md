# zel documentation workflow and portable HTML requirements

Status: portable build merged in PR #12 and native `serve`/`view` in PR #13.
`view --watch` is implemented in the next milestone, with hosted acceptance
pending. The native renderer and server are separate from the hosted Next router.

The user wants MkDocs-equivalent documentation workflows, without requiring the
same command syntax, including these three operations:

| Operation | Required behavior |
| --- | --- |
| Build a Markdown folder | Discover Markdown and local assets, generate a navigable static site, and produce an `index.html` that can be opened directly without an HTTP server. |
| Serve a generated site | Start a local HTTP server for an existing static output directory. This remains useful even though direct-file viewing also works. |
| View a Markdown folder | Build the site, start the local server, and open its actual URL in the default browser. Compose the same build/serve implementations. |

The implemented interfaces are `zel build <docs> --out <site>`,
`zel serve <site>`, and `zel view <docs>`. Default build target selection keeps
structured Zellij sites on the hosted static path; `view` always builds portable
Markdown. `--target portable` selects that build target explicitly.

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
  compilation are not implemented. `view --watch` performs full rebuilds and
  reloads the HTTP preview after successful publication.

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
| Serve an existing export | Implemented | Preserve cross-platform routing/containment/lifecycle acceptance |
| View a source folder | Implemented, including optional watch/reload | Richer configuration |
| Development rebuild/reload | Portable `view --watch` implemented; hosted preparation watching implemented | Hosted automatic page reload, incremental compilation |
| Deploy/publish | Existing images and generated provider workflows | Finish chosen CLI providers and release scope |
| Native distribution | Portable OS CI added | Release/install smoke tests, Windows packaging/installer |

## Native serve/view contract — 2026-10-17

`zellij-docs::server` serves existing exports using Hyper/Tokio and MIME detection.
It binds only to 127.0.0.1, supports selected/ephemeral ports, prints the actual
URL, serves GET/HEAD, and returns 405 for other methods. File bytes remain
unchanged. Exact files, extensionless HTML routes, directory indexes and custom
404 pages are supported. Directory redirects retain queries. `--base-path`
mounts an export under an explicit prefix; it does not rewrite the artifact.

Decoded request paths must stay beneath the resolved root. Traversal, malformed
encodings, metadata paths, symlinks and Windows junctions are refused. No directory
listing or home-page fallback occurs. The root must contain a regular index.html;
unmarked exports are allowed. This is a local preview server, not a production
hosting, TLS or authentication implementation. Responses use no-store and nosniff.

`view` reserves its port before invoking the same portable build helper as
`build --target portable`, starts the server, and invokes a launcher against
its actual URL. `--no-open` skips launching; `--browser` accepts one executable
path and passes the URL as a single argument. It conflicts with `--no-open`.
Browser failure warns without taking down HTTP. The default uses the OS opener;
automated launch acceptance uses a real substitute executable that requests the
ready URL before recording success, avoiding GUI launches on test runners.

Ctrl-C stops with exit 0 and releases the listener. POSIX termination signals
are also handled. Up to 256 async connections share a 16-thread filesystem
pool; idle sockets and slow readers do not block the control loop. Requests have
a 10-second header-read timeout, and file bodies stream with backpressure.
Shutdown cancels in-flight responses. Source and generated files are not deleted.
Ordinary `view` and `serve` do not watch; `view --watch` is described below.

Real HTTP integration tests cover routes/MIME/HEAD/methods/prefixes/containment/
port conflicts and listener release. `scripts/check-preview.mjs` exercises the
actual binary without external tools on PATH, Chromium/WebKit under a prefix,
port reservation before writes, launch ordering and failures, no-open, failed
build preservation, junctions, and Ctrl-C with stalled clients. On Windows,
`scripts/fixtures/preview-console.rs` puts only the tested child in a private
console and sends a real Ctrl-C event; Node's forced process termination would
not test the handler. The default desktop browser selection itself remains an
OS-opener integration, not an automated GUI acceptance claim.


## Portable watch contract — 2026-10-17

`zel view <docs> --watch` adds full rebuilds and HTTP live reload. Ordinary
`view` still builds once; `serve` continues to serve an existing artifact.
The initial source snapshot precedes the initial build, so edits during startup
are not lost. The source root is resolved once. The watcher pauses 300 ms between
scans, hashes streamed bytes of visible regular files, and waits for two equal
snapshots before rebuilding. Hidden entries and node_modules are excluded.
Nested symlinks, Windows junctions, and special files are refused without
following them. Repeated unchanged failures are not rebuilt or logged endlessly.
Full scans/rebuilds can be expensive for very large trees; no incremental compiler
or configurable watch roots are promised.

`prepare` stages a complete portable export without publication. The watcher
publishes it under the live-preview write lock; HTTP holds the read side while
resolving/opening files and attaching a matching revision, not while streaming
responses to clients. This excludes the directory-swap gap from preview reads.
Separate HTTP requests can still span generations; this is not a whole-browser
snapshot transaction. Failed builds retain the previous artifact and revision.
Later edits, including fixes to missing documents/assets or a restored source
root, recover. Changes made during a build are picked up by subsequent scans.

Only watch HTTP responses receive the local reload script and revision endpoint.
The embedded revision prevents a publication between HTML load and first poll
from being missed. Revision changes reload the current URL; errors show a notice
without reloading the last good page. Deleted-page 404 previews include the same
client and recover after restoration. Reload requires JavaScript. All exports
remain free of preview scripts/endpoints and work offline when moved. The
loopback/Host/path/method restrictions also apply to preview endpoints.

Ctrl-C joins the watcher; a build still being staged is discarded after
cancellation and its temporary directory is cleaned. Exported files remain.
Options such as name/theme/output/port are fixed for the lifetime of the command.
Native Linux/macOS/Windows CI uses the existing real-console lifecycle harness
plus Chromium/WebKit editing, error recovery, search/media, continuous HTTP,
shutdown, and moved direct-file acceptance in `scripts/preview-watch.mjs`.
