# zel CLI inventory and published-documentation comparison

Reviewed 2026-10-17 (America/Los_Angeles; fetches began
2026-10-18 06:44 UTC). Source baseline:
`e8ec68159468344d1f7bcdb79ad27300d2498e1a`.

This is a documentation/source audit, not an implementation milestone. The
user's recent build, image, CI, and folder-layout decisions are part of the
whole CLI design. They do not reduce its scope to those commands or supersede
unrelated authoring, inspection, conversion, validation, or maintenance work.

## Sources and method

- [Published CLI overview](https://zellij.shebka.net/cli), all its linked CLI
  reference pages, and a direct request to `/cli/view`.
- Local [CLI documentation](../examples/zellij-docs/content/02-cli/index.mdx)
  and [original CLI specification](zellij-cli-spec.md).
- Rust [declarations and dispatch](../cli/crates/zellij-cli/src/main.rs),
  [discovery](../cli/crates/zellij-core/src/project.rs),
  [portable configuration](../cli/crates/zellij-docs/src/config.rs), and
  [workflow generation](../cli/crates/zellij-scaffold/src/pipeline.rs).
- Read-only help checks against the existing local debug binary, cross-checked
  with source. Seven specification-only commands were also checked with
  `--help`; all were rejected with exit code 2.

HTTP retrieval returned 200 for these 20 pages: the overview, `new`, `explain`,
`list`, `context`, `detect`, `convert`, `migrate`, `check`, `fmt`, `dev`, `serve`,
`build`, `deploy`, `image`, `ci`, `publish`, `ship`, `doctor`, and `config`.
`/cli/view` returned 404. The public browser-fetch tool could not retrieve the
site; direct HTTPS retrieval succeeded. Temporary responses, extracted main
text, metadata/hashes, and help output are in
`/private/tmp/zellij-cli-docs-audit/`, not durable project files.

No deployment revision was identified. This audit did not re-run builds,
deployments, releases, or the full test suite. Source status below does not
mean the latest downloadable binary contains the same capabilities.

## Complete registered command inventory

There are **19 top-level command families** and **32 leaf command forms** in the
current parser, excluding generated help and version flags. Five forms dispatch
to the unimplemented fallback; the other 27 have implementation handlers.
A handler does not imply every documented option or provider works.

| Command form | Current source status | Contract or limitation to retain in the redesign |
| --- | --- | --- |
| `new portable` | Native | Markdown starter; current flat layout needs migration to the agreed site layout. |
| `new docs` | Native | Hosted YAML/MDX scaffold, including sections and optional guide. |
| `new app` | Stub | Wrapper application scaffold remains work. |
| `new section` | Native | Schema-based hosted section creation; explicit file output or stdout. |
| `new page` | Native | Creates hosted section files and prints a `pages.yaml` entry for the user to add. |
| `new guide` | Native | Creates a hosted guide with sidebar and index. |
| `new project` | Native | Software repository, documentation, workflow; existing layout differs from the new convention. |
| `explain` | Native | Section schema inspection, including JSON. |
| `list sections` | Native | Compiled section inventory; currently 23 types. |
| `list themes` | Native | Shared 14-theme catalog. |
| `list formats` | Native | Registered converters: MkDocs and limited Docusaurus. |
| `list targets` | Native | Current build targets: portable, static, image. |
| `context` | Native | Content reference or raw schemas for humans and AI tools. |
| `detect` | Native | Inspects external documentation format; argument is an import source. |
| `convert` | Native | External tree to separate hosted content; `--out`, `--from`, `--dry-run`. |
| `migrate` | Implemented composition | Detect, convert, validate; explicit source and output. |
| `check` | Native or container | Portable validation; hosted schema/deep checks; strict mode; JSON only for portable. |
| `fmt` | Stub | YAML formatting and check-only mode. |
| `dev` | Container | Hosted development server and preparation updates. |
| `serve` | Native | Existing generated folder over HTTP; current default is `dist`. |
| `view` | Native | Portable build, HTTP preview, browser launch, optional watch/reload. |
| `build` | Native or container | Native portable HTML, hosted Next static export, or Next runtime image. |
| `image build` | Container | Hosted Next.js runtime image; no separate static-server image builder. |
| `image run` | Container | Runs a tag; omitted tag is derived from current content, not remembered from the last custom-tag build. |
| `image push` | Container | Pushes an image, optionally retagging for a registry. |
| `image dockerfile` | Implemented | Prints the generated Next.js image Dockerfile. |
| `ci init github` | Implemented | Workflow generation: validation, image publication, or Netlify upload. GitLab/Circle are not implemented. |
| `ci publish` | Implemented | Dispatches an existing GitHub workflow; needs `gh` for actual dispatch. |
| `deploy` | Stub | Provider-based build/upload; seven provider names are not seven integrations. |
| `publish` | Stub | Configured validation/build/deployment workflow; relationship to `deploy` needs consistent documentation. |
| `ship` | Stub | Conversion through publication; provider/registry semantics differ between older examples. |
| `doctor` | Implemented | Container environment diagnostics; not general resolved-site/provider diagnostics. |

## Additional documented design scope

These commands are not registered today. Keep them in the design inventory;
an old specification entry does not establish implementation or a release
commitment.

| Command | Documented purpose | Evidence |
| --- | --- | --- |
| `watch` | Repeated validation without a preview server | Original CLI spec; unrecognized by current parser. |
| `eject` | Extract a wrapper application | Original CLI spec; unrecognized. |
| `features` | Configure optional engine capabilities | Original CLI spec; unrecognized. |
| `pull` | Fetch/update the engine image | Published/local doctor reference and original spec; unrecognized. |
| `upgrade` | Update the native binary | Published/local doctor reference and original spec; unrecognized. |
| `completions` | Generate shell completion | Published/local doctor reference and original spec; unrecognized. |
| `version` | Report binary/engine/schema versions | Published/local doctor reference and original spec; unrecognized. `--version` works. |

Configuration and shared options are also scope. Historical plans include
`--quiet`, `--verbose`, `--no-color`, `--yes`, `--config`, and `--cwd`; none is
registered. `--json` is globally registered but supported only for context,
list, explain, detect, and portable check. Its appearance in generated help
does not establish JSON support for other commands.

## Published versus local discrepancies

The deployed and local documentation are **not in sync**. Local pages describe
newer functionality, but some still repeat obsolete promises.

| Area | Published documentation | Local source / documentation finding |
| --- | --- | --- |
| Preview | [`serve`](https://zellij.shebka.net/cli/serve) is planned; `view` is absent and returns 404. | Both work natively in source; portable watch is also implemented. |
| Portable builds | [`build`](https://zellij.shebka.net/cli/build) describes only hosted static and Next image outputs. | Portable HTML, direct-file navigation/search, local fonts, math, mindmaps, and opt-in Mermaid are implemented and documented locally. |
| Creation | [`new`](https://zellij.shebka.net/cli/new) omits portable creation and labels `new project` planned in its inventory. | Both have handlers. `new app` remains a stub. |
| JSON | [`list`](https://zellij.shebka.net/cli/list) and [`explain`](https://zellij.shebka.net/cli/explain) describe JSON as ignored. | JSON is implemented; local references describe the correction. |
| Validation | [`check`](https://zellij.shebka.net/cli/check) describes container-only checking, planned strict mode, and ignored JSON. | Native portable checking, strict mode, hosted deep checks, and portable JSON exist. |
| Configuration | The [overview](https://zellij.shebka.net/cli) promises upward TOML discovery, while [config](https://zellij.shebka.net/cli/config) says configuration is planned. | Portable `zel.yaml` exists. TOML defaults/upward discovery do not. The new site-root convention is agreed but unimplemented. |
| CI options | [`ci`](https://zellij.shebka.net/cli/ci) lists `--registry`, `--on`, and `--out`. | Local CI docs repeat these; the parser accepts none. Actual options also include `--site`, `--force`, and `--no-provision`, absent from that table. |
| CI execution | The same page shows `zel deploy netlify`, describes push-only publication, and omits `ci publish`. | Local CI docs repeat the gaps. Actual generation uses Netlify's uploader, supports manual dispatch, and gates publication on a non-PR event. `ci publish` exists. |
| Static images | The [build page](https://zellij.shebka.net/cli/build) relegates static-server container packaging to a manual operation. | New user requirements explicitly include this workflow. That older exclusion must be revised. |
| Maintenance | [`doctor`](https://zellij.shebka.net/cli/doctor) says four planned commands return an unimplemented message. | Local docs repeat this; they actually fail as unrecognized commands (exit 2). |
| Image tag defaults | [`image`](https://zellij.shebka.net/cli/image) describes an omitted run tag as the last image built. | Local docs repeat this; `image_run` derives the default tag anew and does not remember a custom build tag. |
| Inventory counts | Published section examples still refer to 19 types. | Current inventory is 23; some older local prose/spec examples retain 19. |

Platform instructions differ too: the deployed overview directs Windows users
to WSL; the local reference describes native Windows packaging in current source
and warns that releases may lag. This review did not query fresh release assets
and does not establish availability of a new binary release.

## Consequences for the CLI design

1. Maintain one full catalog: canonical commands, aliases, input meanings,
   defaults, renderer/runtime needs, outputs, implementation status, examples.
   Keep all existing authoring, inspection, import, publication, and maintenance
   workflows visible while simplifying build/image/CI commands.
2. Apply the agreed `zel.yaml` + `content/` + `site/` convention consistently to
   site-oriented scaffolding, validation, formatting, previews, builds, images,
   deployment, and workflows. Repository-root default: `docs/zellij/`; custom
   site roots: explicit paths.
3. Preserve distinct argument roles. Import commands read external sources;
   scaffolding creates destinations; `context` can write a file; image run/push
   take tags. Do not blindly interpret these as configured site roots. Preserve
   explicit serving of an existing output directory.
4. Preserve portable Markdown and the full hosted YAML/MDX/section engine.
   A shared `zel.yaml` marker must not force hosted sites through the portable
   renderer. Current dispatch treats its presence as portable, so the common
   configuration/renderer contract needs attention.
5. Distinguish static-server and Next.js images, retaining run/push/Dockerfile
   operations. `image static` / `image next` and `si` / `ni` remain proposed
   spellings, not implemented aliases.
6. Generate workflows from supported commands and a compatible pinned release.
   Watch config and content changes. Write workflows at the software repository's
   `.github/workflows`, even when the Zellij site is nested deeper.
7. Reconcile local references before publishing them. Separate current source,
   released behavior, and future design; deploying the current local docs alone
   would republish some incorrect examples.

Use this inventory to revise the full CLI contract and alias map. The discussion
has not authorized publishing the documentation or made every historical
proposal a required v1 feature.

## Follow-up — 2026-10-18

The [full contract draft](zel-cli-contract-2026-10-18.md) now maps this inventory
to proposed canonical commands, aliases, argument/default rules, compatibility,
and implementation milestones. Only the user-agreed layout is settled.

Local reference edits correct CI generation options/execution/provisioning,
document `ci publish`, and fix the maintenance-command status and image run-tag
description. The discrepancy table above records what was found on 2026-10-17;
it is not silently rewritten to imply the deployed documentation was updated.
No new public deployment or release was made. Local checks passed: generation
of three workflow modes without provisioning/dispatch and deep/strict docs
validation (47 routes, 23 embedded examples). Edits remain uncommitted at this
design milestone.

## Implementation follow-up — 2026-10-18

The user subsequently authorized implementation, including behavior when Docker
is absent. The runtime prerequisite increment includes these reference edits
and the full contract draft. It adds bounded runtime discovery, platform-specific
errors, explicit endpoint isolation and native Dockerfile generation; the local
doctor/image references have been revised accordingly. The historical inventory
and discrepancy table above remain the comparison at `e8ec681`, not a claim that
the website was redeployed. The configured-site layout and aliases remain next
implementation work. See [context](../ZELLIJ_CONTEXT.md) for verification/commit
evidence as this increment completes.


### Later implementation — configured sites (2026-10-18)

The inventory tables above remain the dated audit baseline. The subsequent
configured-site increment implements versioned `zel.yaml`, content/site layout,
repository/cwd defaults, build.out/preview.port and portable/hosted/project
starters. Current site-oriented handlers share that resolver; explicit legacy
inputs remain supported. Bare `serve` now selects a configured site's output
instead of assuming `dist`. See the updated CLI contract and local references.
Aliases, additional image modes, deployment and configured CI remain pending.

## Implementation delta — 2026-10-18 shortcuts and completion

Source `271493f46d3b97ceed12623524505374e3e12190` implements the
working-command shortcuts listed in the [current reference](../examples/zellij-docs/content/02-cli/shortcuts.mdx).
`ci run` now dispatches the same workflow as retained `ci publish`.
`completions <shell>` / `comp <shell>` is registered and implemented for Bash,
Zsh, Fish, PowerShell and Elvish; the earlier unrecognized-command inventory
above is the historical baseline. Help and generated completion share the
parser schema, including both spellings. This does not add `si`, `ni`, `na`,
`pub`, `netlify`, or bare `ci [site]`, nor change converter output format.


## Implementation delta — 2026-10-18 static images

Source `5ed8fb49126ce3919abedbe532f6ec7ba63506e4` adds the implemented
`image static` leaf and `si` shortcut, for both renderers. Dockerfile generation
now accepts `--kind static|next`, defaulting to the previous Next recipe. Existing
Next build paths are unchanged. `image next` / `ni` remains unregistered until
its Markdown adapter preserves navigation, all pages, links, assets and settings.
See the CLI image reference and durable context for context-folder semantics,
runtime requirements, tag defaults and verification evidence.


### Implementation delta — 2026-10-18, Next images

`image next` / `ni` is implemented for hosted content and portable Markdown.
Portable content is adapted to validated inert document trees in a temporary
folder, preserving all pages, navigation, links/assets, metadata, branding and
Markdown features. New default tags end in `-next:latest`; legacy image commands
remain hosted-only with their original defaults. Next health checks now honor
basePath. There are 24 top-level shortcuts. See the durable context for the
source/CI record and the updated image guide for exact behavior and limitations.
This does not implement shared delivery configuration or other remaining stubs.


### Registry-publication delta — 2026-10-18

The historical table above remains the audit baseline. `publish` now has an
implementation for registry-only delivery, with the `pub` alias (25 top-level
shortcuts). Versioned configs accept `image.name` and
`publish: {action: push, image: static|next}`. New image commands honor the saved
name; legacy commands retain their tag rules. `publish --dry-run` resolves a
plan without tools or side effects. The common build operation runs once, and
push runs only after success. Direct provider deployment/Netlify, ship, saved CI
settings and expanded workflow generation remain incomplete. See the current
command reference and durable context for source and verification evidence.


## Implementation update — 2026-10-18, Netlify

The historical inventory above is retained. `deploy netlify` / `netlify`,
`publish` / `pub` with saved Netlify settings, and bare `deploy` now execute the
same native static delivery operation. Portable delivery requires no external
runtime; hosted content exports once in its container, preserving basePath.
Draft previews are default; `--prod` requests live publication. Credentials are
read from NETLIFY_AUTH_TOKEN only. No provider CLI, stored login discovery,
site creation or remote build is used. The local reference and current contract
record actual options, failure stages and validation limits. Other providers,
ship, saved CI generation and the remaining CLI families are still incomplete.


### Later implementation: saved CI and short generation — 2026-10-18

The historical inventory above is retained. `ci [site]` now shares GitHub
workflow generation with `ci init github --site-root`, consuming typed saved
CI and delivery settings. PRs check/build, eligible non-PR events build/deliver
once, and native Netlify replaces the Node uploader. Generation is local-only
unless --provision is explicit; old --content/--target/--deploy/--no-provision
options and ci publish dispatch remain accepted. A pinned compatible 0.2.0+
release is required; current public 0.1.3 is not compatible. See the current
contract and durable context for options, validation and release limitations.


## 2026-10-18 configured import and delivery update

Configured import (`convert`/`migrate --layout site`) and `ship` are implemented
in source; historical stub rows above describe the initial inventory. Legacy
content layout remains the default for convert/migrate. Ship requires a new
configured destination and either explicit Netlify delivery or static/Next image
publication, preserves the source, retains conversion reports and calls shared
delivery once. See the contract's stage 5d and CLI reference for flags and limits.
New app, fmt/watch, portable dev, hosted view, other providers and advanced
maintenance remain pending. The public 0.1.3 release predates these commands.


## 2026-10-18 editable-wrapper update

The historical new-app stub is replaced in current source by `new app` / `na`.
It writes an editable Next.js wrapper using the engine initializer's shared
snapshot, references existing hosted content or creates docs/zellij with
--with-docs, and performs no dependency installation or server startup. A built
local engine is required until npm distribution exists. Matching engine and
wrapper-config versions, destination containment and no-overwrite protections
are enforced. The local guide/contract record exact options and limits.
The remaining backlog includes fmt/watch, portable dev, hosted view, other
providers, maintenance commands and compatible release publication.


## 2026-10-18 formatting and validation-watch update

The historical fmt stub is implemented in source and watch is now registered.
Fmt natively formats selected site YAML with comment/value preservation and a
nonwriting --check mode. Watch performs initial and debounced repeated validation,
recovering after invalid edits with a pinned renderer/layout; portable is native
and hosted uses the container engine. Both honor configured-site discovery and
explicit legacy roots. See contract stage 6b and the local fmt/watch references.
Remaining work includes portable dev, hosted view, other providers, advanced
maintenance and compatible release/public documentation publication.


## 2026-10-18 development and preview update

Portable dev is implemented through native watched preview, with no browser
launch by default. View/v now supports hosted static content as well as portable
Markdown, including watching, engine-resolved basePath and last-good export
preservation. Hosted dev remains Next.js development; its port must be nonzero.
See contract stage 6c and the local dev/view references for options and limits.
Remaining work includes other providers, advanced maintenance, additional dev
flags, compatible release packages and public documentation publication.

### Maintenance implementation follow-up — 2026-10-18

The historic inventory above remains the audit baseline. Current source now
registers version (including JSON), pull and upgrade. Their complete argument,
verification, platform and failure contracts are documented in the corresponding
local CLI pages and contract stage 7a. Completions include all three commands.
Public zellij-v0.1.3 still predates these additions. Eject/features, further
providers and advanced options remain pending.


### Eject and features implementation — 2026-10-18

The native CLI now registers `eject [site] --out <destination>` (or --with-docs)
and `features <selection> [--site-root <path>]`. Eject copies the selected engine
distribution's /app and /opt/zellij using a stopped runtime container. It stages,
validates, cleans up and publishes only into new/empty output. Vendored engine
sources use file:vendor/zellij; tsconfig excludes those source/test copies from
the wrapper's own typecheck. External hosted content is referenced; image starter
content moves to docs/zellij/content with zel.yaml. The historical 'no npm'
promise was wrong: the distribution includes sources, not installed dependencies.
Users run npm install afterward. Runtime discovery/prerequisites remain shared.
Cancellation/copy failures attempt cleanup; cleanup failure reports the container
name and does not publish. No local Docker daemon was started.

Features maps maths/diagrams to portable markdown.math/markdown.mermaid switches
or to a compatible editable wrapper's direct feature packages. Portable edits
preserve YAML comments and unrelated spelling; alias/anchor targets are refused.
Wrapper changes retain unrelated manifest data and lockfiles, with npm install
left explicit. Hosted content without a wrapper is refused with useful guidance.
The feature dependency mapping is shared with the generated engine template.

A pre-existing engine limitation was confirmed: npm optionalDependencies may be
installed even when a wrapper has no direct feature requirements. Hosted `none`
therefore removes direct requirements, not rendering capability or all installed
packages. Strict exclusion/explicit hosted capability switches remain follow-up
work. The current documentation and historical spec corrections state this;
portable toggles do control rendering. No zero-bug/completion claim is made.

Local checks: 277 Rust tests, warnings-denied Clippy, Rust formatting,
actionlint for the changed workflow, and deep/strict documentation validation
(55 routes / 53 guide pages) passed. Native runtime-protocol acceptance exercises
selected sources, digest overrides, movable engine references, feature changes,
nonempty output, failed copy, incompatible engine, links and real Ctrl-C cleanup.
CI adds the same extraction acceptance on Windows/macOS/Linux and a real-container
eject + npm install + validation + production build + typecheck/browser check.
Final CI evidence and merge status must be refreshed before relying on this work.
Remaining work includes other providers/advanced options, the hosted dependency
limitation, compatible native releases and documentation publication.


### Hosted feature selection follow-up — 2026-10-18

This supersedes the hosted limitation documented in the eject/features milestone.
Editable wrappers now save package.json zellij.features explicitly; native and
Node scaffolding/setters share the same feature map. Fresh wrappers default to
none, while legacy wrappers without metadata retain installed-package detection.
The engine uses optional peers rather than optionalDependencies, so fresh none
wrappers omit math/Mermaid libraries. Selected missing packages fail the Next
configuration step with npm install guidance. After a rebuild/dev restart, none
disables rendering even when packages remain installed for another consumer.
Native mindmaps remain enabled. Engine-scoped loaders and aliases support both
Turbopack and Webpack without intercepting unrelated app imports of those libraries.
The native compatibility check refuses engines without featureSelectionVersion: 1
before changing a wrapper. No npm/lockfile mutation or package deletion is implicit.

Fresh installation also exposed a hidden dependency: the engine's source types
needed d3-hierarchy/d3-shape declarations previously supplied through Mermaid.
Those declarations are now explicit engine dependencies, keeping mindmap consumers
independent of optional diagrams. Docker distributions carry the shared selection
helpers and typed loaders. CI adds standalone fresh-none, enabled-all and retained-
packages-none browser/build/typecheck acceptance.

Local checks: 364 engine tests, 277 Rust tests, warnings-denied Clippy, Rust
formatting, workspace typechecks, engine packaging, generated-template consistency,
extraction/cancellation acceptance, workflow lint and strict/deep documentation
validation passed. Final browser acceptance and commit/CI evidence follow below.
Other providers, advanced CLI options, compatible native releases and published
documentation remain separate unfinished work.


Final local hosted-feature acceptance passed: a standalone wrapper installed
without Mermaid/remark-math/rehype-mathjax and built/typechecked/rendered with
Turbopack; enabling all installed them and rendered real MathJax/Mermaid output;
switching to none with packages deliberately retained disabled both under Webpack.
Native mindmap SVG and browser-error checks passed in every state. Each npm state
is checked in a fresh Node process, avoiding stale negative package lookups.
No locked dependency versions changed. Source commit and hosted CI evidence will
be recorded after PR validation; no new native release or manual deployment occurred.
