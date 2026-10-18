# zel documentation workflow and portable HTML requirements

Status: portable build merged in PR #12 and native `serve`/`view` in PR #13.
`view --watch` merged in PR #14 with all-platform acceptance. Portable `zel.yaml`
configuration/navigation and excluded-asset checks merged in PRs #15–16.
Native validation and structured diagnostics merged in PR #17. The portable
starter is merged in PR #18; opt-in native math is merged in PR #23. Native YAML
mindmaps are merged in PR #24. Opt-in Mermaid is the current implementation milestone. The native renderer and server
are separate from the hosted Next router.

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

## Agreed configured-site layout — pending implementation

The user's CLI design decision on 2026-10-17 distinguishes the software
repository from a Zellij site within it. A configured site has this layout:

```text
my-product/
  src/
  docs/
    specs/
    zellij/                 # Conventional location; any folder name is valid
      zel.yaml              # Identifies the site root
      content/              # Markdown and local source assets
        index.md
        guides/
        assets/
      site/                 # Generated HTML and assets
        index.html
```

From the software repository root, the agreed default is:

```sh
zel build                   # Uses docs/zellij/zel.yaml
zel build docs/zellij       # Explicit equivalent
zel build docs/manual       # Uses docs/manual/zel.yaml instead
```

The positional path identifies the **parent of `zel.yaml`**. Source and output
defaults are `content/` and `site/`, resolved relative to that selected site root.
For example, `zel build docs/manual` reads `docs/manual/content/` and writes
`docs/manual/site/`. Recognition depends on `zel.yaml`, not the folder's name.
A nonstandard location is supplied explicitly rather than discovered by a
recursive scan of the repository's documentation. Other documentation, such as
the adjacent `docs/specs/`, is outside the selected website source.

Source/output separation applies to `content/` versus `site/`: the output can
be inside the configured site root without being inside the content source.
This replaces the proposed flat Markdown/config layout and `dist/` convention
for newly configured sites. It does not claim that discovery or these defaults
already work in the binary. Existing direct Markdown-folder builds and serving
an explicit output folder remain requirements; implementation must reconcile
their existing contracts with the new configured-site entry point.

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
  Explicit local references and configured logos cannot publish files from those
  excluded paths; published assets must live in the watched source tree.
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
- Raw HTML is literal text with a warning; `.mdx` is rejected. MathML rendering
  is opt-in through `markdown.math`; YAML `mindmap` fences render to SVG and a
  semantic outline. Opt-in Mermaid uses a bundled browser runtime with source fallback. Remote media remains remote. MkDocs plugins,
  full-text search language processing and incremental compilation are not
  implemented. Optional `zel.yaml` supplies identity, branding and navigation. `view --watch` performs full rebuilds and
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
| Markdown, titles, automatic/custom navigation, themes | Core plus `zel.yaml`, opt-in math/Mermaid and native YAML mindmaps implemented | Raw HTML policy, broader config |
| Assets, TOC, offline search | Implemented for local Markdown references | Richer search/navigation where needed |
| Strict diagnostics | Native `check`, strict warning gate and JSON reports implemented | Hosted JSON reports, richer source locations |
| Project scaffolding | Native `new portable` full/minimal starters; existing hosted scaffolds | Extend starter content with future rendering capabilities |
| Serve an existing export | Implemented | Preserve cross-platform routing/containment/lifecycle acceptance |
| View a source folder | Implemented, including optional watch/reload | Richer configuration |
| Development rebuild/reload | Portable `view --watch` implemented; hosted preparation watching implemented | Hosted automatic page reload, incremental compilation |
| Deploy/publish | Existing images and generated provider workflows | Finish chosen CLI providers and release scope |
| Native distribution | Six-target packaging/install acceptance and Windows installer implemented in source | Publish a new reviewed version; signing and broader OS coverage remain outside this matrix |

## Native YAML mindmaps — 2026-10-17

`mindmap` fences use the hosted engine's documented YAML hierarchy syntax and
render in Rust without new dependencies, Node, a browser, or network access.
They are built in and need no configuration flag. The shared check/build pass
emits finished inline SVG plus a native `details` disclosure containing the
equivalent nested-list outline. Source labels stay in the offline search index.
Each diagram has reserved, distinct title/description IDs. Only pages with
mindmaps link the stylesheet; sites without any omit it from the export.

The root must be one mapping entry. Keys are strings; mapping values define
children; sequences contain scalar labels or mappings. Strings, finite numbers
and booleans become leaves; a null mapping value means a leaf. Null sequence
items, directly nested sequences, duplicate keys, custom tags, empty labels and
multiple YAML documents are errors. Labels collapse whitespace and are escaped
in SVG, accessible names and outlines. YAML alias expansion is bounded during
deserialization. Limits per diagram: 64 KiB source, 512 nodes, 32 levels including
the root, 256 Unicode scalar values per label. No authored file is rewritten.

Layout uses conservative fixed-font width estimates, columns sized by their
widest label, disjoint leaf rows and parents centered over their children. This
is not a promise of the hosted renderer's exact geometry, branch palette or
arbitrary complex-script shaping. Wide maps scroll instead of shrinking labels.
Colors follow the active theme through CSS. Print styles fit SVG to page width.
The existing local enhancement handles Left/Right and Home/End on focused
diagram viewports. WebKit's native arrow scrolling fails even on a minimal
overflow div; with JavaScript disabled, the keyboard-operable outline exposes
the entire hierarchy. SVG and outline rendering itself requires no JavaScript.

Invalid diagrams fail with source-file `rendering` diagnostics and diagram
ordinal before publication. Watch mode retains the previous export/revision
and recovers after edits. Tests cover grammar/limits/escaping, real CLI JSON
without external tools, relocated nested `file://` output in Chromium/WebKit,
actual SVG/text bounds, outline hierarchy, theme/search/navigation, mobile
overflow (including expanded maximum-depth/label outlines in their own scroll
region), keyboard controls, JavaScript-disabled content and HTTP/watch error
preservation/recovery. Mermaid fences remain literal code; this milestone does
not add Mermaid execution or claim exhaustive screen-reader/browser acceptance.

## Portable Mermaid — 2026-10-17

`markdown: {mermaid: true}` enables Mermaid fences; the default remains literal
code. The Rust compiler emits escaped source disclosures, page-local runtime/CSS
links and local MathML font assets. It validates nonempty source, 64 KiB per fence
and 32 diagrams per page. Grammar/layout validation happens in the browser,
including with native `check --strict`; invalid syntax keeps source visible and
shows an inline error without stopping the other diagrams. A browser-enforced
500-edge limit cannot be increased by an author directive. Native limit failures
remain prepublication errors and preserve the old export.

A committed classic IIFE embeds Mermaid 11.17.2, KaTeX and all diagram chunks;
no ESM, fetch, CDN, Node or HTTP server is required by the generated artifact.
Diagrams require JavaScript, while their source remains readable with JavaScript
disabled or the script unavailable. Only pages with enabled Mermaid load its
approximately 3.5 MB runtime. Removing the last diagram removes the runtime;
shared math assets remain if native math needs them. Remote author resources
remain remote; diagram image references are not discovered/copied. Plugins,
remote icon packs and server-side Mermaid rendering are outside this contract.

Rendering is serialized and stale theme revisions discarded. Theme/mode changes
and system color-scheme changes redraw graphs after font readiness. Strict
security, text/edge bounds and modern MathML mode are protected from author
configuration. Repeated IDs emitted inside upstream mindmap SVG are removed
while preserving the first reference target. Native mindmap styles are scoped to
the native SVG so both formats can coexist. Source remains in offline search.

`scripts/prepare-portable-mermaid.mjs` regenerates/verifies the bundle, notices and
provenance manifest. CI verifies exact reproduction from locked dependencies.
The generator substitutes patched js-yaml 4.3.2 for the older upstream embedded
module and verifies its hash. Embedded parser dependencies have separate exact
version notices and source-map checks, including an assertion that the Lodash
4.17.23 subset excludes the functions affected by its known advisories. See
`cli/crates/zellij-docs/assets/mermaid/README.md` for evidence and upgrade policy;
a successful ordinary npm audit does not inspect upstream embedded dependencies.

Acceptance covers native config/limits/diagnostics and failed-build preservation;
real CLI checks without external tools; moved nested `file://` output in Chromium
and WebKit with network blocked; flowchart, sequence, state, pie, mindmap and
MathML labels; invalid grammar/edge-limit isolation; strict labels and protected
configuration; unique SVG IDs, rapid theme/mode changes, system appearance,
search/navigation, mobile layout, no-JavaScript and missing-runtime fallbacks;
and HTTP/watch opt-in changes, edits, failure preservation and recovery.

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
CLI name/theme overrides and output/port options are fixed for the lifetime of
the command. File-based name/theme/branding/navigation are reread on each build.
Native Linux/macOS/Windows CI uses the existing real-console lifecycle harness
plus Chromium/WebKit editing, error recovery, search/media, continuous HTTP,
shutdown, and moved direct-file acceptance in `scripts/preview-watch.mjs`.

## Portable configuration and navigation — 2026-10-17

Read only `zel.yaml` in the input Markdown root. Supported keys: optional `name`,
`theme`, `markdown.math`, `brand.logo` (`src`, `alt`, optional absolute HTTP(S) `href`), and `nav`.
All text is strictly typed; unknown/duplicate keys and invalid values fail before
staging/publication. A present config selects portable automatic build dispatch;
explicit target flags retain their meaning. CLI name/theme overrides win, then
file settings, then existing defaults. The whole config must still be valid.

Navigation is an ordered list of objects with exactly one of `page`, `children`,
or external HTTP(S) `href`, plus an optional `label` (required for groups/links).
Page values are exact source-relative Markdown filenames, including extension,
with literal Unicode/spaces and forward slashes. Missing/duplicate/traversing
paths and empty groups fail. Group nesting is bounded at 16. Omitted `nav` keeps
automatic ordering; `nav: []` removes navigation. All pages are still validated,
rendered and searched; unlisted pages have no previous/next links. Navigation
labels/order drive the sidebar and depth-first previous/next sequence, while
headings, search titles and folder breadcrumbs retain document metadata. A
synthetic home uses configured navigation when present.

Local logo images are copied through path/type/link checks and have encoded
relative output URLs. Logo and site title remain at the left of the sticky bar;
controls remain right-aligned. The logo can link to the company website while
the title always links home. Nested lists remain usable without JavaScript.
Watch reloads added/edited/removed configuration with existing preservation and
error recovery. Browser preferences persist only explicit choices, so default
config theme changes become visible; old automatically persisted defaults are
not treated as explicit choices. The revised preference format may reset an
older saved choice once. The portable config is distinct from hosted content
settings and the still-planned general `zel.toml` command defaults.

## Native validation contract — 2026-10-17

`zel check [dir]` shares build's target selection: a `zel.yaml` entry at the
requested root, then hosted discovery, otherwise portable Markdown. Explicit
`--target portable|static` overrides it; static means the hosted checker.
Portable checks run the same compiler as build entirely in memory, including
configuration, source discovery, Markdown parsing, navigation, reference/fragment
resolution, asset collection, rendering and output-path validation. Build alone
stages and publishes files. Checks never create output or temporary directories,
require no external tools or network, and ignore existing output ownership.

Raw HTML remains literal with a warning. `--strict` fails on warnings without
changing their severity. Local references are always checked; `--deep` is accepted
for shared commands but adds no native coverage. Document parse/read failures are
collected before dependent phases; once parsing succeeds, reference errors are
collected across pages. Configuration, discovery, rendering and output-path phases
stop at their first fatal error. No claim is made about checks that could not run.

Portable `--json` emits one report on stdout on both validation success/failure,
with no diagnostic stderr output. Schema version 1 has `target: "portable"`,
`root`, `strict`, `ok`, nullable `pages`, and `diagnostics`. Each diagnostic has
`severity`, `code`, nullable source-relative `file` (forward slashes), and `message`.
Codes are `source`, `configuration`, `markdown`, `reference`, `output-path`,
`rendering`, and `raw-html`. Root is canonical if resolved; otherwise requested.
Page count exists only on compilation success and includes a synthetic home.
No structured line/column fields are fabricated. Tolerate additive fields.

Exit codes are 0 on success, 3 for content/strict failure, and 1 for a returned
I/O failure; the first fatal error controls the code when errors are collected.
CLI usage failures and hosted JSON requests exit 2 before checking, using normal
stderr errors. Human native output sends summaries to stdout, diagnostics to
stderr. Hosted checks retain existing engine behavior; hosted JSON and quiet
output are not implemented. No `build --strict` is added by this milestone.

Checks do not test publication permissions, ownership, disk capacity, remote
URLs or browser behavior. Rust integration tests compare check/build failures,
snapshot input/existing exports, exercise strict/JSON/exit behavior without tools
on PATH, and run in the Linux/macOS/Windows portable CI matrix. Browser acceptance
continues to cover the shared compiler's actual file://, HTTP and watch artifacts.

## Portable starter contract — 2026-10-17

`zel new portable [dir]` creates a native Markdown project, defaulting to `docs`.
`--name` defaults to the resolved destination's leaf name, including `.`;
`--theme` defaults to `cupertino`. Both are validated before writing. Names must
be nonempty text without control characters. Metadata uses the shared YAML
string serializer; display names never become Markdown or shell syntax.

The full starter contains `zel.yaml`, `index.md`, `guide/getting-started.md` and
the guide's local SVG asset. `--minimal` contains just configuration and home.
Configuration supplies ordered navigation; removing `nav` restores automatic
discovery. There are no missing example links or enabled placeholder assets.
Creation requires no external tools, network, dependency installation or server.

The shared scaffold writer preflights all paths and refuses nonempty folders
and generated-file collisions. Existing dotfiles (including a fresh `.git`)
are retained. There is no overwrite flag for this command; create elsewhere
when the destination contains authored content. Hosted `new docs/page/guide/section`
retain their existing behavior and produce a different content format.

CLI integration tests create/check/build real starters with external tools
absent from PATH, cover every theme and unusual names, verify source preservation,
and exercise invalid options and destination conflicts. The portable OS matrix
runs those tests. `scripts/portable-starter.mjs`, called by the existing browser
gate, opens the actual full/minimal generated output after moving it to a path
with spaces and Unicode. Chromium/WebKit verify names, navigation, fragments,
local images and search with networking blocked, plus JavaScript-disabled content.

## Native math contract — 2026-10-17

`markdown: {math: true}` in `zel.yaml` enables pulldown-cmark's inline `$...$`
and display `$$...$$` syntax. It defaults to false, preserving dollar-sign prose.
Escaped dollars and code remain literal; unmatched delimiters remain ordinary
Markdown. Backslash-parenthesis/bracket delimiters are not enabled. Image alt
text retains plain LaTeX instead of embedded markup. Titles, TOC labels, slugs
and search derive from the equation source alongside ordinary heading text.

`math-core` is pinned to 0.9.0 and converts expressions to MathML during the
shared check/build compiler. No Node, TeX, subprocess, CDN or browser typesetter
is used. Malformed expressions, unsupported commands and unresolved equation
references fail with the existing `rendering` diagnostic, source file and an
expression ordinal. Previous exports survive failures. Each expression has
local definitions/numbering/references and a distinct reserved `_zel-math-N-`
ID namespace. Cross-expression references are not supported. Original LaTeX is
retained in an escaped MathML annotation; raw HTML policy is unchanged.

Only pages containing rendered math link its CSS; a site without equations
omits `_zellij/math`. The binary embeds the unmodified official STIX Two Math
2.13b171 WOFF2 and OFL license, plus the matching renderer CSS fixes/MIT license.
Pinned revisions and checksums live in `cli/crates/zellij-docs/assets/math/README.md`.
Display equations have keyboard-focusable horizontal scrolling on small screens.
Math follows inherited text color and uses its own font regardless of site theme.

The CLI and `zellij-docs` now require Rust 1.96 for source builds because of
the math dependency; the other crates retain their earlier minimum. CI checks
the complete workspace with 1.96 in addition to stable Rust quality gates.
Downloaded binaries need no Rust installation. No new binary release is implied.

Acceptance: library and actual-CLI JSON tests cover configuration, formulas,
heading links/search/alt text, source escaping, invalid/recursive commands,
local label IDs, failure preservation and removing unused assets. The portable
browser gate exercises relocated nested `file://` pages with networking blocked,
both with and without JavaScript: actual font decoding/checksum, fraction geometry,
matrices, search/navigation, theme color, mobile scrolling and HTML-like text.
Preview acceptance covers config edits, invalid-equation preservation and recovery
through the real HTTP watcher. CI runs these on Linux, macOS and Windows.
This is common LaTeX math support, not full TeX/MathJax parity, exhaustive font
layout or manual screen-reader acceptance. Mermaid remains a separate follow-up.
