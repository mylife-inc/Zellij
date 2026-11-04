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


### Shared CLI path options — 2026-10-18

Source 58b7129ea18f4add46154fe8b123edc383626152 adds global --cwd and --site-root.
The former applies before command dispatch; the latter selects existing site
inputs through the same handlers as positional arguments and existing selectors.
CI --content is retained. Conflicting, repeated or inapplicable selections fail
with exit 2 before effects. Creation/import destinations stay separate. See the
current contract stage 8a and local configuration reference for full support.

All 284 Rust tests, Clippy with warnings denied, formatting and strict/deep docs
validation passed locally. Cross-platform CI and merge evidence must be refreshed
for this branch. Explicit --config, remaining advanced flags and other providers
remain proposed. These source capabilities are not in public native release 0.1.3.


### Explicit configuration files — 2026-10-18

Source 1653d84afb2b7ad065e0eb00c60ddd8d700c49db implements global --config for
existing-site workflows. The selected YAML and its parent replace normal config
discovery; versioned and legacy portable layouts retain their path semantics.
Conflicting/repeated selectors fail before effects, and missing files never
fall back. Watch, formatter/features, diagnostics, images/delivery and generated
CI retain the same selection. Wrapper zellij.config.yaml remains separate.

Local verification passed: 290 Rust tests, Clippy, formatting, workflow lint,
strict/deep docs and native preview/watch/hosted-fixture/wrapper acceptance.
Refresh final CI and merge status for this source. Remaining shared output
controls, additional providers and matched release/publication work are still
unfinished; the public 0.1.3 executable predates these capabilities.


### Output controls available in source — 2026-10-19

PR #43 merged as 8912aee8f7ed037fe8b491a5fdf5d916fe997637 after all 13 CI
jobs passed on ea6ada4e2b77b86ecf014cfa1f7291d50a2f06ed (39211231453).
Its main CI 39213225423 and Base image 39213225268 both passed; PR #42's
previously pending main CI 39209473176 passed as well.

Fixing commit f7d141441652cc951a0f75ae98d83a672b1181f9 implements shared --quiet/-q,
--verbose and --no-color with alias/help/completion support. Quiet retains data,
errors/warnings and requested reports; verbose adds stderr diagnostics without
raw arguments/env; color filtering covers parser errors and child output.
NO_COLOR is honored, quiet/verbose conflict before work, and validation/output/
publish behavior is unchanged. Hosted check keeps its complete mixed report.
Child output uses independent file handles and anonymous temporary spooling,
with 64 KiB per-stream failure tails and normal cancellation/cleanup ownership.
Spooling consumes temporary disk while the command runs. See contract stage 8c.

Local: 299 Rust tests plus final focused output regressions, warnings-denied
Clippy, formatting, workflow lint and strict/deep docs validation passed. Actual
quiet preview/HTTP/watch/recovery/Ctrl-C, validation watch and no-color hosted
preview with an isolated runtime/browser passed. CI adds cross-platform output
acceptance, real no-color preview and quiet/no-color ejection. Refresh final PR
checks before treating the source as merged/cross-platform accepted. Additional
providers, remaining advanced/converter scope and native release/docs publication
remain; no local Docker startup or manual live deployment occurred.


### Image platform selection — 2026-10-19

PR #44 merged as 2ada5fe0ad1e1227c70eb963746cca929fc72af5 after all 13 CI
jobs passed on 52cc3620cdfbdff5f8db4d443abe9a925619cbb4 (39216062489).
Its main CI 39218117100 and Base image 39218117079 both passed.

Fixing commit c9db8dfc1edbe7e702f7585e2680fbceb002435a implements single-platform Linux
image selection (linux/amd64 or linux/arm64) across image static/next/build,
si/ni, build --target image, saved publication/deploy, ship --image and CI.
Optional image.platform supplies the default; CLI overrides it. image.name is
now optional for local image builds, while publication still requires an
explicit destination. Plans show the platform, ship retains it in zel.yaml,
and generated CI freezes it in both jobs. A follow-up uses native ARM64 runners
for generated workflows, avoiding known QEMU/Node failures in release builds.
HTML/Netlify flags and unsupported/combined platform values fail before work.

Podman builds omit Docker's unsupported --progress option. nerdctl pushes use
--all-platforms to preserve a foreign single-platform local image. Build failure
keeps diagnostics, explains base/emulation requirements and never proceeds to
push. No local emulator/service is installed or started. See contract stage 9a.

Local: 304 Rust tests in 38 suites, final focused delivery tests, warnings-denied
Clippy, formatting, workflow and generated-workflow lint, and strict/deep docs
validation (55 routes, 53 guide pages, 23 examples) passed. A first focused run
had one 3-second fake-runtime probe timeout; the complete run with four test
threads passed. CI adds real ARM64 portable build/push/pull/HTTP acceptance and
an AMD64 override. Refresh final PR checks before claiming hosted acceptance.

Remaining: combined multi-platform manifests, further advanced options/providers,
conversion fidelity, matched native release and documentation publication.
Foreign-architecture hosted/Next execution is not covered by the small ARM64
portable acceptance fixture; dispatch for all renderer/kind pairs is covered.
No manual live provider/registry deployment or native release was performed.

Follow-up be57372ec7e32c4edad9bce538ac78ec9c914d5f selects native ARM64 generated-workflow runners.
All nine workflow tests, scaffold tests, generated workflow lint, Clippy,
formatting and strict/deep docs validation passed after this refinement.


### Temporary build/image features — 2026-11-02

PR #45 merged as 42bf4c906d1ca6be64940da8aaddfb9d63f1192c after all 13 CI
jobs passed on 5c75428ca495a0afcae722a678607cafb6e4c368 (39220991457).
Refreshed on November 2: main CI 39223399959 and Base image 39223399906 both
passed. The latest public native release remains zellij-v0.1.3 (August 13).

Fixing commit aa179631e7371d9c7185a07ec1929452e2262af5 adds --features to build/b,
image static/next/build, si/ni and hosted image dockerfile. Accepted complete
selections are maths, diagrams, maths,diagrams (either order), all and none.
The override does not edit authored configuration or content. Portable builds
apply it in memory; hosted exports and image recipes apply it to the disposable
wrapper before npm install. Explicit none overrides an enabled base image.
Portable Next adapters retain the selected rendering features and install the
matching dependencies, including an explicit none for an empty selection.

A shared typed parser rejects malformed/unknown/duplicate selections before
work. Hosted feature selection requires featureSelectionVersion 1 from the
engine manifest; older engines fail clearly instead of silently mishandling
none. Previous exports survive build/compatibility failures. Portable static
Dockerfiles take prebuilt HTML and direct users to build/si for feature changes.
Preview/dev, delivery/ship and generated CI retain saved configuration or engine
image variants; this temporary flag is not implemented for those commands.
See contract stage 9b and the updated build/image/features references.

Local verification: all 310 Rust tests in 38 suites passed. After the final
engine compatibility guard, all 22 engine tests and 9 static-image tests passed;
warnings-denied workspace Clippy, formatting, workflow lint, script syntax and
strict/deep documentation validation passed (55 routes, 53 guide pages,
23 examples). Hosted CI now exercises all/none folder exports and static images,
enabling features in a hosted Next image, and refusal of an incompatible engine.
Cross-platform and actual-container results are pending for this branch; refresh
the final PR head/checks before claiming hosted acceptance or merge completion.

Remaining: combined multi-platform manifests, other advanced options/providers,
conversion fidelity, matched native releases and documentation publication.
No local Docker daemon was started, native release published or manual live
provider/registry deployment performed for this milestone.


### PR #46 dependency and schema compatibility follow-up — 2026-11-02

[PR #46](https://github.com/mylife-inc/Zellij/pull/46) is open. Its first run,
41500959209 at 5dee8187c858c3fac3cfc7f509d41cd14f8a97ce, failed npm audit:
Zod 4.4.3 is affected by [GHSA-mrhx-c7xc-m3w5](https://github.com/advisories/GHSA-mrhx-c7xc-m3w5).
Fixing commit 1c9374665c64bff29bfd76054c9d28a75f5f9134 raises the engine dependency
and lockfile to Zod 4.6.7. npm audit --audit-level=low now reports zero known
vulnerabilities as checked on November 2; this is a dated audit result.

The update exposed primitive unions emitted as JSON Schema type arrays instead
of anyOf. Section template generation now handles those arrays, and Rust field
descriptions retain their alternatives. The generated schema was refreshed;
text context and generated YAML starters remain unchanged. Regressions exercise
actual stats-band values and timeline dates, alongside every section starter.

All 365 engine tests and 311 Rust tests (38 suites), workspace typechecks,
engine packaging, warnings-denied Clippy, formatting and strict/deep docs
validation passed locally. Final CI on the updated PR head remains required
before merge; neither the initial failing run nor local checks establish hosted
acceptance. Authentication works with XDG_CONFIG_HOME=/Users/hamid/Dev/config.


### Preview/development feature selection — 2026-11-02

PR #46 merged as eea28d6053622f9d692d6b56aa4150492d9f0f21 after all 13 CI
jobs passed on e5d619687e6460bb454a06a7a722715998bc9378 (41502232990).
Both post-merge runs now passed: CI 41506317359 and Base image 41506317357.
The latest public native release is still zellij-v0.1.3 (August 13), refreshed
on November 2; source capabilities below are not a claim about that executable.

Fixing commit 6ab08a3bcee51133a5fa176d3af2eb5bfb5bd554 adds --features to view/v
and dev for both portable and hosted sites. Typed complete selections match
build/image. Overrides survive every watched rebuild, portable configuration
edit and recovery, without changing authored files. Omitting the flag on a
later invocation restores saved/default selection. Serve still serves prebuilt
output and does not accept rendering options. See contract stage 9c.

Hosted dev and preview apply the override before dependency installation, using
the existing engine compatibility guard. Hosted export/dev dependency caches
now include the canonical explicit feature selection, preventing a build with
another selection from uninstalling a live dev session's packages. Equivalent
all and maths,diagrams share a cache; omission and none remain distinct. Old
caches are not deleted. Hosted dev now distinguishes content-validation errors
from dependency/startup failures instead of reporting all failures as bad content.

Local verification: 312 Rust tests in 38 suites, warnings-denied Clippy,
formatting, workflow lint, JavaScript syntax and strict/deep docs validation
passed (55 routes, 53 guide pages, 23 examples). Actual Chromium preview/watch
acceptance covers config edits, rendering, failure preservation, recovery,
source immutability, default restoration and Ctrl-C. Hosted orchestration with
an isolated runtime fixture also passed, including feature propagation on
rebuilds and cancellation. An initial full Rust run had one existing fake-runtime
probe timeout while other checks were active; the isolated full two-thread run
passed. Do not treat this as a runtime-probe implementation fix.

CI now checks actual hosted preview math/Mermaid rendering through rebuilds,
then explicit none in a subsequent preview; hosted development checks both in
a browser before and after edits. Final cross-platform/container CI and merge
are pending for this branch. No local Docker daemon was started or native release
published. Remaining work includes delivery/CI temporary controls, advanced
build options, combined image manifests, providers beyond Netlify, conversion
fidelity, and matched native release/documentation publication.


### PR #47 development-origin regression — 2026-11-02

First CI run 41511951845 at c98e226e565da6dcd1c9b5eec263fb090fff6b6a
passed 10 jobs, including real hosted preview rendering and five native package
targets. The CLI job failed the new hosted dev Mermaid browser check. Both
macOS ARM jobs were cancelled before any step ran because GitHub reported
hosted runner capacity constraints; these cancellations are not test results.

The browser check exposed an existing hydration failure: Next binds 0.0.0.0
inside the container, but the engine excluded loopback addresses from allowed
origins. Opening 127.0.0.1 blocked the HMR connection and left client components
unhydrated. This was reproduced locally with the real Turbopack dev server.
Fixing commit b23e1d417dd93f2d708f5a4aa86c28a3b14033bf allows the exact IPv4
and IPv6 loopback hosts (127.0.0.1 and [::1]). Direct wrapper browser acceptance
now uses the container's wildcard binding and checks real math/diagram output,
editing, search, recovery and origin enforcement. Requests from unrelated
hosts, including a hostname resembling a loopback IP, still return 403.

After the fix, real local Chromium/Turbopack acceptance and all 365 engine tests
passed. Final CI on the updated PR head, including the actual container and
both macOS ARM jobs, is still required before merge. No local Docker daemon
was started. PR: https://github.com/mylife-inc/Zellij/pull/47.


### Rendering features in delivery and CI — 2026-11-02

PR #47 merged as 42641950fdcfd75d43737a1f40ecbdf64a457a48 after all 13 jobs
passed on final head 27415a15a1b8159d3fab19fdd5e18c84051cdb26 in CI run
41517322451. This includes the actual hosted-development regression and both
macOS ARM jobs. Post-merge Base image run 41520378666 passed; CI 41520378824
had 12 of 13 jobs passed with CLI still running at this checkpoint. Refresh that
run before relying on a final post-merge result. Latest public native release
remains zellij-v0.1.3 (August 13), refreshed on November 2.

Fixing commit bf78d8346b5149668d03b7773fadc73acf2b9d26 extends the shared typed
--features selection to publish/pub, deploy/netlify, ship and generated CI.
Delivery forwards the invocation-only selection to one portable/hosted static
or image build. Dry plans display it. Source, saved configuration and previous
local exports remain unchanged. Failed builds do not publish. Ship retains its
existing destination/platform settings but not this override, and explains how
to repeat the selection on a later publish after success or failure.

CI generation freezes the canonical selection in verification and publication
build commands for every supported delivery form, including named configs,
platform selections and legacy image generation. Validation and prebuilt image
push commands do not receive rendering flags. Regenerating without the flag
removes the override. Dispatch does not accept it. Existing compatible-release
pins, event gates and explicit credential provisioning are unchanged.
See contract stage 9d and updated publish/deploy/ship/ci/features references.
The deployment reference's stale claim that CI ignores saved delivery settings
was also corrected.

Local verification: all 315 Rust tests in 38 suites, warnings-denied Clippy,
formatting, script syntax and strict/deep docs passed (55 routes, 53 guide pages,
23 examples). Ten generated workflows passed actionlint. Tests execute generated
portable commands, inspect selected HTML uploaded to a loopback API, cover all
renderer/image combinations, and retain ship/config/output failure guarantees.
Hosted CI now checks features in Netlify export/upload, portable Next image
publication, and imported static/Next images through browser rendering. Final
cross-platform/container CI and merge are pending for this branch.

No local container daemon was started, native release published, real provider
site deployed or external registry image manually published. Remaining work:
advanced build options, combined multi-platform images, additional providers,
conversion fidelity and matched native release/documentation publication.


### Release candidate isolation and runner capacity — 2026-11-02

PR #48 merged as b1d636934d2be06b92e01a02b150c11dcbb3dfe9 after all 13 jobs
passed on b365ad6e97c7c524a4dd396419c3d38c53e8265f (run 41523296482).
The earlier PR #47 post-merge CI 41520378824 also passed. PR #48 post-merge
Base image 41526351348 passed, but CI 41526351538 failed: 11 jobs passed, the
CLI container group exhausted hosted runner disk during npm install in the
Next ship image, and macOS ARM native packaging was cancelled before its first
step. Neither the disk failure nor the cancellation proves a product regression;
neither counts as successful acceptance. Logs identify ENOSPC, including failure
to write the runner's own logs. Commit 7c0397c298935c0ce43fac01948d5a96e6e7bd04
reclaims unused Docker builder cache between large acceptance groups while
retaining tagged engine images. Hosted verification remains required.

Fixing commit 0e5638f3b24ca057f26465f01bb1bdbc514d62d2 centralizes release policy
in scripts/release-policy.mjs and applies it to tagged CI and local helpers:

- Candidates publish exact image tags only, become GitHub prereleases, and
  cannot update stable/latest image aliases, deploy production docs or replace
  shared installer scripts. Branch builds only update main feature variants.
- Both repository release pages now use version-pinned attached Unix/PowerShell
  installers and correct image tags without the leading v. The previous
  candidate notes invoked a stable installer and referenced nonexistent v-prefixed
  image tags. Existing published aliases are not changed retroactively.
- The emergency local publisher now uses the same policy, including --prerelease
  and --latest=false. Both release helpers reject workspace version mismatches
  before tagging/building. Candidate preflight never enables production docs.
- The release workflow still allows native publication after an independent
  image failure. Complete candidate acceptance requires checking all jobs and
  final downloaded artifacts; a release page alone is not proof of completion.

Local verification: 12 release policy tests passed, including actual publishing
shells and manifest commands with fake Docker/GitHub/Git executables; no remote
publication is possible through those fixtures. Workflow lint, Bash/Zsh syntax,
workspace version check and strict/deep documentation validation also passed
(55 routes, 53 guide pages, 23 examples). Tests run in ordinary CI and tagged
release verification. Final hosted CI and merge are pending for this branch.

See specs/zel-release-procedure.md for candidate preparation and stable promotion.
No version bump or release tag was made, no native release was published, no
manual provider/registry deployment occurred, and no local Docker daemon was
started. The intended next candidate is 0.2.0-rc.1, after reviewed versioning and
full CI. Current native source and latest public release still say 0.1.3.


Release review follow-up: c7931f250214117cf58b2955db86399866b66946 changes
zimage-push's default from latest to local and zimage-all to local feature tags,
matching zimage. Explicit raw pushes remain available; release helpers apply
channel policy. Generated release notes now show matching ZELLIJ_IMAGE settings
for both Unix and PowerShell. Fourteen release tests pass locally, including
execution of the branch manifest command and development image helpers.

PR #49: https://github.com/mylife-inc/Zellij/pull/49. Initial run 41533203275
passed all six native package/installer targets, engine and schema checks before
the final follow-up was pushed; it does not certify the updated head. Refresh
PR #49's final head/checks before merging. Read-only setup checks confirmed that
RELEASES_GITHUB_TOKEN, NETLIFY_AUTH_TOKEN and NETLIFY_SITE_ID exist by name;
DEPLOY_DOCS is unset. Secret values were neither read nor printed. Presence does
not establish token validity or destination permissions. The development repo
has no published releases; the shared repo's newest Zellij release is still
zellij-v0.1.3. No user setup action is currently needed for this milestone.


### Native 0.2.0-rc.1 preparation — 2026-11-02

PR #49 merged as d324f9ef51a6c36d47304ef765adb6725aa12bff after all 13 jobs
passed on 136a206de28a24e1bdf7232f9d9332f80abd23a5 (41534811176). Its post-merge
Base image run 41538755022 passed, confirming main-only publication. Post-merge
CI 41538755131 failed the engine watcher test: 364 of 365 engine tests passed;
the assertion read nested/new.svg after observing an earlier logo edit and could
race the later asset preparation. This is an invalid test completion signal,
not evidence of permanent asset loss. Production preparation still serializes
updates and uses the existing short directory swap; it is not a whole-site
transaction. The test now forces separate edit batches and polls the requested
new asset. Refresh the complete post-merge run for other jobs' final results.

Preparation commit 03ec06f2c2bf93aee88c629449dd3a5bc2ce36f0 sets the Rust workspace
and its six package lock entries to 0.2.0-rc.1. Bare zel ci now defaults to its
own compatible version, and its regression checks this pin plus preservation on
invalid input. Release notes in specs/releases/v0.2.0-rc.1.md describe supported
commands, platforms and remaining limits and are included in both release pages.
The public stable release remains zellij-v0.1.3 until a new publication occurs.

The release workflow and local ztest/zcheck helpers also used npx vitest at the
repository root, but the lockfile installs Vitest inside the engine workspace.
That command failed locally with vitest not found. They now use the existing
npm test workspace command, matching ordinary CI.

Published release acceptance is a new reusable/manual-tag workflow. After
publication it downloads the exact archive, per-file checksum, SHA256SUMS and
both attached installers from the development repository and (when published)
mylife-inc/releases. It checks hashes and source installer bytes, compares the
two repositories, installs through their actual public URLs, and runs the native
archive/installer/upgrade/build/serve/view acceptance on six native targets.
Windows public installation uses both PowerShell 5.1 and PowerShell 7. Installer
line endings are fixed to LF so source comparisons agree on Windows. The four
published image variants are pulled and executed on native Linux AMD64 and ARM64;
architecture, engine contract and feature selection are checked. Acceptance jobs
have read-only repository permissions and receive no publishing/cloud secrets.
They can be rerun on a published tag without republishing assets.

Local verification: 365 engine tests, 315 Rust tests in 38 suites, 25 release
policy/download tests, workspace typechecks, engine packaging, warnings-denied
Clippy, Rust formatting, workflow lint and strict/deep docs passed (55 routes,
53 guide pages, 23 examples). Download tests use fake HTTP responses and image
tests use a fake Docker executable; no local Docker daemon was started. Locked
npm ci restored dependencies and reported zero advisories at this checkpoint.

Next: final-head PR CI and merge, then tag the reviewed candidate and monitor
all release and post-publication acceptance jobs. Do not call a release page
alone complete: native publication can still succeed while image jobs fail.
No tag, native release or production website deployment has occurred in this
preparation commit. Repository visibility and the old sealed repository are
unchanged; development remains in public mylife-inc/Zellij.


### Published 0.2.0-rc.1 and download recovery — 2026-11-02

PR #50 merged as 61881e805fdf34af12757e33ca4c1703c314825e after all 13 CI jobs
passed on 7b6838154cce0eaf69788c58a63abe0d1a2e5efa (41542526979). Annotated tag
v0.2.0-rc.1 points to that merge commit. The candidate is now public in both
repositories, each with six archives, six per-file checksums, SHA256SUMS and
both installers (15 assets):

- https://github.com/mylife-inc/Zellij/releases/tag/v0.2.0-rc.1
- https://github.com/mylife-inc/releases/releases/tag/zellij-v0.2.0-rc.1

Release run 41546241822 built all six native targets, published all four exact
engine image variants for Linux AMD64/ARM64, and built the documentation.
Production Netlify deployment and shared stable-installer updates were skipped.
Both release pages are prereleases; scoped stable Zellij remains zellij-v0.1.3.
Main Base image run 41546163136 also passed. Main CI 41546163311 had 11 passing
jobs with CLI and Windows portable acceptance still running at this checkpoint;
refresh it before describing its final result.

The original release run is NOT green: 27 jobs passed, while both Windows
public-acceptance jobs failed after completing the development-repository
installation and native suite. Their next fetch, from the shared repository,
failed with ECONNRESET/UND_ERR_SOCKET. A failed-jobs retry reproduced this.
This was an acceptance downloader failure; no checksum, installer or binary
assertion failed. Do not erase these failures or move the published tag.

Fix f9c6b45 adds at most three attempts for connection errors/timeouts. Each
attempt discards partial bytes and retains size, checksum and installer/source
checks. Missing/oversized assets and integrity failures are not retried. Fix
29029e16e2caf30b1b1dbb99e43f64b95d5ee76d permits an explicit published version in
manual acceptance runs. The harness checks out refs/tags/<version> separately,
so source comparisons and native tests still use the actual release commit.

Published release acceptance 41548199714 PASSED all eight jobs using harness
29029e1 against unchanged source tag v0.2.0-rc.1 at 61881e8. All six native
targets downloaded both repositories' assets, verified byte equality/checksums,
and exercised public installation. Windows ran PowerShell 5.1 and 7; its x64
log confirms a reset was recovered on attempt 2. Both native Linux architectures
pulled and executed all four published image variants. Use this successful
follow-up run together with the original publication run as candidate evidence:
https://github.com/mylife-inc/Zellij/actions/runs/41548199714

Local verification: 29 release/download tests, workflow lint and diff checks
passed. The actual published Intel macOS archive also passed temporary
installation from both repositories, replacement/refusal/upgrade tests, portable
build, moved-output preview, serve/view and Ctrl-C. No local Docker daemon was
started, and no global zel installation or PATH was changed. The download fix
changes acceptance infrastructure; candidate archives/installers/images were
not rebuilt or replaced. The fix branch still requires final PR CI and merge.

Next: finish the workflow-fix review, then exercise candidate-generated CI and
provider delivery against explicitly selected test destinations before stable
promotion and matched production documentation. Other tracked scope remains:
advanced build options, combined multi-platform site images, additional
providers and converter fidelity. Candidate acceptance does not mean the full
product has zero bugs or that every historical feature proposal is implemented.


Candidate documentation follow-up: local installation, quickstart, CLI and
contract pages now identify the published 0.2.0-rc.1 candidate, show pinned
attached Unix/PowerShell installers and the matching engine image, and use the
available candidate in copyable CI pins. Stale statements that source still
reports 0.1.3 or Windows packages await publication were corrected. Stable
0.1.3 remains explicitly distinguished from the candidate. The current-source
sections were refreshed while dated implementation history was preserved.
Strict/deep documentation validation passed: 55 routes, 53 guide pages and
23 examples. Both release pages also link the successful acceptance run and
retain the original publication failures. Production docs remain undeployed.


### Candidate acceptance closeout — 2026-11-02

[PR #51](https://github.com/mylife-inc/Zellij/pull/51) merged as
49d9a148f40df2b747d7ce6a0f46efc5a16c3954 after final CI
[41549182222](https://github.com/mylife-inc/Zellij/actions/runs/41549182222)
passed all 13 jobs on 9ab6232758f535cf517acff263523fc5ee96353b. This includes all
six native package/installer jobs, three portable/browser platforms, engine,
schema, showcase and full container delivery acceptance. The separate candidate
source main CI 41546163311 also finished with all 13 jobs passing, superseding
the pending result in the preparation record above.

Published-artifact acceptance remains 41548199714: eight passing jobs against
unchanged v0.2.0-rc.1 source 61881e8, using download harness 29029e1. Both release
pages now link that result and preserve the original Windows download failures.
An additional local smoke used the actual published Intel macOS executable to
generate portable `zel ci` and hosted `zel new project` workflows in temporary
repositories: both pinned 0.2.0-rc.1 and its matching engine and passed actionlint.
This checked generation; live execution of those newly generated workflows and
real provider delivery at designated test destinations remain the next gate.

The candidate is available for acceptance use. Stable 0.1.3, production docs,
and the published candidate binaries/installers/images retain their prior state.
No local Docker daemon was started. The next work is live candidate delivery
acceptance and an explicit stable-promotion decision, alongside the tracked
advanced build/provider/conversion backlog. Do not infer a zero-bug or complete
historical-feature claim from these passing checks.


### Generated workflow and delivery acceptance — 2026-11-02

Refreshed PR #51 post-merge status: CI 41552573220 and Base image 41552573125
both passed on 49d9a14. Candidate v0.2.0-rc.1 remains at 61881e8; no new release
or stable promotion is part of this source change.

Fixing commit 0a79f9504b233b39ad311eb112789e4299290a88 addresses a multi-site CI
queue collision. Every generated workflow previously had the display name
Documentation and used github.workflow in its concurrency group, so distinct
site workflows on the same ref shared a queue. GitHub allows only one pending
run per group by default, allowing a new site's run to replace another pending
site run. See https://docs.github.com/en/actions/concepts/workflows-and-actions/concurrency.
The generator now names workflows by their configured filename and derives a
bounded group key from SHA-256 of that filename plus github.ref. The hash
preserves case-sensitive file identity even though GitHub group names are
case-insensitive. sha2 was already locked for the CLI; no third-party version
changed. Tests cover independent sites, stable regeneration and case-only names.
This generator fix is newer than the published candidate and needs a later
native release. Existing workflows need deliberate regeneration/review.

Six committed acceptance workflows are generated without hand-editing from
scripts/fixtures/release-delivery. They use the corrected source generator,
but install and execute the public 0.2.0-rc.1 binary and exact engine image.
They cover native portable and hosted static builds, static/Next image delivery,
and portable/hosted Netlify delivery. PR events only validate/build. Publication
requires manual dispatch on main; Netlify never receives --prod. Dedicated image
destinations are ghcr.io/mylife-inc/zellij/acceptance-static:0.2.0-rc.1 and
acceptance-next:0.2.0-rc.1. A separate manual read-only inspection workflow pulls
those packages on fresh runners and verifies HTTP pages/assets, non-root users,
health checks, registry digests and source-file isolation.

The preparation helper regenerates workflows in a temporary Git repository,
compares their exact bytes, requires independent queues and checks PR/manual
triggers plus draft-only publication. Ordinary CI checks generated-workflow
drift. The shared HTTP checker supports either public draft renderer and rejects
missing pages/styles, altered assets and exposed sources. No local Docker daemon
was started. No actual registry or Netlify publication has occurred at this
preparation checkpoint. A user question about the Netlify test destination is
pending: existing documentation-site draft previews or a separate supplied site.
Do not manually dispatch Netlify delivery until that target choice is settled.

Local verification: 317 Rust tests in 38 suites, warnings-denied Clippy, Rust
formatting, all seven acceptance workflows plus CI through actionlint, exact
workflow regeneration, and three HTTP-checker tests passed. The HTTP checker
also passed against a real native portable build/temporary server. Hosted
fixture deep/strict validation passed (3 routes, 2 guide pages), and product
docs passed (55 routes, 53 guide pages, 23 examples). Full PR CI, actual generated
GitHub execution, registry publication/pulls and selected Netlify drafts remain
pending. Preserve their eventual run IDs and distinguish source-generator
acceptance from capabilities actually shipped in the candidate binary.


### Live delivery results and Netlify account discovery — 2026-11-02

PR #53 merged as bc3955bbd14a586a9d2da8375ff6f0f8717cb95b after all 19 executed
checks passed on acbdb94: full CI 41561583999 (13 jobs) and six actual generated
workflow checks (41561584047, 41561584073, 41561584038, 41561584027,
41561584019, 41561584037). The four PR publication jobs were correctly skipped.
Post-merge Base image run 41564319833 passed; CI 41564319708 was still running
at this checkpoint and must be refreshed before stating its result.

Manual generated workflows installed the public 0.2.0-rc.1 executable and
published dedicated test packages. Static run 41564355308 passed. Next run
41564361875 initially uploaded layers but GHCR denied the manifest; attempt 2
passed with no code or permission changes. The first failure remains evidence
of an undiagnosed registry failure, not a fixed product defect.

Fresh-pull inspection initially exposed an acceptance-checker mistake:
41565212699 passed static and could not find the not-yet-published Next tag;
41565578836 passed static but probed /index.html for Next, whose adapted home
route is /. Fix d2844ab9dcf28851e838aa9464cc06642c026d4b checks the HTTP root,
adds a regression fixture with no /index.html route, and retains startup status
and bounded container logs on failure. Four local HTTP tests passed (the new
case failed before the fix). Corrected run 41565958521 passed BOTH jobs against
the unchanged published images, checking AMD64, non-root users, health checks,
actual pages/assets and source-file isolation. Registry digests:

- ghcr.io/mylife-inc/zellij/acceptance-static@sha256:831a2eaec96ffa66d23133659fecdbb104128461f7b57f67e817b9c29a923740
- ghcr.io/mylife-inc/zellij/acceptance-next@sha256:fd0cf3db3087c70e0bcfd3d9a452231f7874bd964f252de569fe36864ee6fb50

The user authorized drafts on the existing Zellij Netlify site. The official
OpenAPI contract was checked: createSiteDeploy takes a deployFiles JSON body
with draft/async fields; the client sends draft: true without --prod.
https://github.com/netlify/open-api/blob/master/swagger.yml
Portable draft run 41564369100 failed GET-site preflight with HTTP 404 BEFORE
building or creating any deployment. The user's screenshot confirmed both
secret names exist, but not their current access. A temporary repository
variable selected the known public domain zellij.shebka.net; run 41565327817
also failed preflight with 404. That temporary variable was removed and the
existing secrets were never read or changed. Hosted draft dispatch was held.
No draft URL exists and live Netlify acceptance is NOT passed.

The user then reported separate GitHub accounts (hamidbm and mylife-inc), with
the existing Netlify deployment originating from hamidbm, and proposed a new
Netlify project associated with mylife-inc. This is a migration discussion,
NOT authorization to delete the live site or move its domain yet. Preserve
existing production while comparing every published route and its information
with current local documentation; prepare a separate staging destination before
any cutover. Netlify account/token access, not GitHub repository identity alone,
determines API access. User requires no loss of published information.

Public response fingerprints for /, /cli, /guide/quickstart were identical
before and after the failed preflights (130579, 124251, 159412 bytes). SHA-256:
8e1204979df3a294646e794f378d91e02f4186e289522890a26b161b22cb47f7;
8a895d38e705e5c6f5ab502996b3c316757d3a2683b913f0a935c28dc6d01247;
fdc8a5c4d7f67e5ced91a019c9693d165919fe4bbac0852df2f6a5042a83022d.
This proves those three bodies matched, not whole-site deployment identity.
No local Docker daemon was started. Candidate assets/tag, stable channel and
production docs were not promoted. The generator fix still needs a later native
release; the checker fix requires PR CI and merge. Full content parity and the
Netlify account migration remain separate unfinished work.


### Netlify staging and published-content preservation — 2026-11-02

PR #54 merged as 35a2261f8112f05a3365c64e8659712dd6e28f64 after all 13 CI jobs
passed in 41569977319. PR #53 main CI 41564319708 also passed. Both published
acceptance images remain verified by 41565958521; Netlify drafts have not passed.

The user reports separate hamidbm/mylife-inc identities and confirmed that the
intended destination is unavailable in Netlify's transfer dialog. Proceed with
a separate staging project under the intended account, while retaining the
existing production project and zellij.shebka.net domain. Deletion, transfer,
domain cutover and production replacement remain unapproved.

Source fix 8370ec4fc43ef2af6e775096df7010b624b608d9 preserves old documentation
bookmarks and corrects repository links. A fresh public inventory found 46
pages; all exist in the 55-route local site. Thirteen normalized page bodies are
identical and 33 contain reviewed corrections/expansion/reorganization. All 29
referenced content images match source bytes. The initial comparison identified
82 lost heading IDs; 74 static aliases and eight restored headings now retain
them. Local Chromium verified all 282 original heading IDs, 46 original and nine
new routes, 29 image hashes, fragment navigation, search and current GitHub links.
Strict/deep validation (55 routes, 53 guide pages, 23 examples), static webpack
build, workflow lint and diff checks passed. Normal Engine CI now runs this
preservation check. Full PR CI/merge for this source fix remain pending.

See specs/zellij-netlify-migration-2026-11-02.md and specs/evidence for the
comparison, legacy mapping, limitations and migration gates. Local ignored
artifacts: backups/zellij-production-2026-11-02.tar.gz (public crawl, not a full
Netlify backup) and backups/zellij-staging-2026-11-02.zip (validated 55-page
static upload). The report records their hashes. A full production deploy ZIP
from Netlify is still needed before cutover; the crawl cannot discover unlinked
files or provider settings. All generated build-only source changes were restored.

The user has the staging upload and instructions to create a separate project,
keep its temporary netlify.app address, and update the existing GitHub Netlify
secrets for it. Await the new URL and readiness before running draft acceptance
there. No credentials were read, no local Docker daemon started, and no existing
production content replaced. Native candidate/stable channels are unchanged;
the multi-site workflow fix still requires a later native candidate.


### Preservation acceptance and staging handoff — 2026-11-02

PR #55 merged as f1ca1ff04f049366a1175b4dfe2bd6e78a6c34a1 after all 13 jobs
passed in 41573372928 on final head 27947739e2bfdfa126f323aca847e970627c71bc:
https://github.com/mylife-inc/Zellij/actions/runs/41573372928
Source fix 8370ec4 preserves 82 legacy bookmarks and corrects repository links.
The Engine gate verified 46 original routes plus nine additions, 282 original
heading IDs, 29 byte-identical images, legacy navigation and search. All six
native package targets, three portable platforms, container delivery, schema
and showcase checks passed. This closes the pending PR result above; it does
not establish remote Netlify acceptance. PR #54 main CI 41572436263 also passed.
PR #55 post-merge CI 41576036593 and Base image 41576036573 remain running at
this closeout; their outcomes must be refreshed independently.

Read-only DNS inspection: zellij.shebka.net CNAME zellij-docs.netlify.app;
shebka.net nameservers ns1.hostmonster.com and ns2.hostmonster.com. No settings
changed. The validated staging ZIP is ready; await the new project's temporary
URL and confirmation that GitHub's Netlify secrets target it. The prepared
remote checker has only been syntax-checked. Both CLI draft deployments and
remote documentation preservation remain unverified pending that destination.

Retain the original live project/domain. A complete production deploy ZIP,
deploy ID and relevant settings remain prerequisites before a separately
approved cutover. The public crawl is not a complete provider backup. Candidate
0.2.0-rc.1 assets and stable 0.1.3 remain unchanged; the workflow queue fix still
needs a later native candidate. See the migration report for the concrete user
handoff. No local Docker daemon or production deployment was involved.


### Live Netlify draft acceptance and staging browser finding — 2026-11-02

Public candidate 0.2.0-rc.1 now passes actual portable and hosted Netlify draft
deployment from generated GitHub workflows on main 3114219. Portable run
41587034632 and hosted run 41587142130 succeeded against the user-selected
zel-stage project with corrected credentials. After the user made previews
public, both drafts passed HTTP pages/assets/source-isolation and browser
search/navigation checks. Draft URLs, exact evidence and verification limits
are in specs/evidence/zellij-netlify-acceptance-2026-11-02.json and the latest
audit/migration-report entries. The initial 401 preview-access failure is retained.

The full staged documentation separately exhibits React #418 when Netlify's
hosting comment is injected into its HTML. Local controls reproduce the issue
with that comment alone, not with the badge script alone. A user-side badge
setting diagnostic is pending; do not describe full documentation browser
acceptance as clean. Neither old production nor stable/candidate release assets
were replaced. A later native candidate still needs to include the workflow
queue fix. Successful main CI 41576279817 and Base image 41576279748 supersede
the cancelled post-PR55 runs 41576036593 and 41576036573.


### Staging badge diagnostic outcome — 2026-11-02

The user turned off the staging badge; the HUD script disappeared, but the
hosting annotation/newline and React #418 remain. Local isolation identifies
the added LF before viewport metadata as sufficient; a comment without it is
clean. See specs/zellij-netlify-hydration-2026-11-02.md for verified reproduction
and the prepared, unsent support request. The full-docs hosting issue remains
open; the earlier portable/hosted candidate draft acceptance remains valid
within its stated fixture coverage. No candidate assets or product code changed.


### Candidate 0.2.0-rc.2 preparation — 2026-11-02

The user authorized continuing with browser acceptance and the next candidate.
Workspace version and only the six local package versions in Cargo.lock are
0.2.0-rc.2; external dependencies are unchanged. Reviewed candidate notes list
the per-workflow queue fix, public-download recovery, preservation corrections,
and strengthened hydration acceptance. The hosting newline issue remains open
with Netlify and is explicitly not claimed fixed. rc.1 assets are immutable;
normal installation documentation continues to target the published candidate.

Local release/download tests: 29 passed. Native CLI unit/workflow tests: 26 unit
and 11 integration tests passed; one real-container test remains intentionally
ignored locally and is a CI gate. The first sandboxed unit run failed seven
local API fixtures because loopback binding was denied; the identical rerun
with loopback permission passed. Version preflight confirms v0.2.0-rc.2.
Final-head CI, review/merge and publication remain pending; no rc.2 tag exists
and stable/production channels have not changed.


### Browser acceptance complete; rc.2 prepared — 2026-11-02

PR #59 merged as 93dca084a018f7ff56e6259441beca4a82874c32. Source commits
68db620 and 97f9f9f capture both browser error channels and wait for mounted
controls. All 13 checks passed on final head 97f9f9f in run 41594855467. Engine
verified all 55 routes, 282 original headings, 82 compatibility mappings, 29
original image hashes and search with no captured browser errors after
hydration. All three negative controls passed: head newline/React #418,
uncaught exception, and a console-only error in an existing JavaScript asset.
This resolves the acceptance coverage gap, not Netlify's HTML transformation.
The first run 41594298255 was superseded/cancelled after its Engine negative
test failed on an overly specific reporting-channel expectation; retain that
failure separately from the passing final run.

PR #60 merged as 0fbf0a12c3cf7074c4620e2bb84cc5d8e5fd7469. All 13 checks
passed on final head 04653b0 in run 41595462484. The merge and checked head both
have tree 4353ca74faba1aad6f6d8e9a4444ad1aa95cf2d0. Version preparation is in
249d0b1, with final scope/verification notes in 04653b0. The reviewed release
notes are specs/releases/v0.2.0-rc.2.md. All six archive/installer platforms,
three portable platforms, engine, schema, real containers and showcase passed.

Automatic approval review rejected executing `zrelease v0.2.0-rc.2`: creation
and publication of a public tag/assets requires explicit user approval beyond
preparation/validation. The command did not execute. No tag, release assets,
versioned images, stable promotion or production deployment was created by this
attempt. Local non-publishing `zcheck` was started separately; its final result
is recorded in the follow-up below. The source remains ready for a specifically
authorized prerelease from 0fbf0a1. After approval, refresh checks and publish to
mylife-inc/Zellij and mylife-inc/releases with matching exact GHCR tags, then
verify the downloaded public artifacts and released delivery workflows.

The user reports filing the Netlify bug and emailing support@netlify.com; a
provider response remains pending. Preserve the old project/domain and obtain
the full original deploy backup before any separately approved cutover.


#### Final local validation — 2026-11-02

The first full local `zcheck` passed typechecking, 365 engine tests, schema
consistency, Rust formatting and warnings-denied Clippy, then failed four
`static_images` mock-client probes at the three-second deadline. Repeating that
suite yielded two timeouts; one affected test passed in isolation. A separate
experiment, without Zellij, launched 12 new exit-zero shell scripts concurrently:
five exceeded three seconds and the slowest took 4.841 seconds. Sequential
re-execution took 0.012–0.022 seconds. This establishes startup latency outside
the product; the OS-level cause was not diagnosed.

The entire `zcheck` was then rerun with `RUST_TEST_THREADS=1`: exit 0, all
365 engine tests and 317 Rust tests passed, with one intentionally ignored
real-container test covered by CI. All ten mock-image tests passed. Product deadlines and CI
parallelism are unchanged, and no local container daemon was started. Default
parallel local execution remains a documented environment limitation, not a
claimed product fix. Raw logs are retained under ignored
backups/zellij-rc2-preparation-2026-11-02/ with hashes in the evidence JSON.
Publication still requires the specific user approval described above.

At this record, post-merge Base image run 41598251612 passed. Main CI rerun
41598251651 has 11 successful jobs, with CLI and Windows portable checks still
in progress; it is not recorded as passed. Both final PR runs already passed
all 13 jobs. Refresh the main run before publication.


### Hosted development auto-reload — 2026-11-03

The user asked to continue implementing the remaining CLI capabilities. Hosted
`dev` and updated editable wrappers now publish an opaque development revision
only after successful search/asset preparation. Development layouts poll that
revision and reload the current URL; image-only edits are included. Invalid
edits do not announce a reload. The development runtime cache is keyed by the
revision so a reload inside the normal 250 ms memo window cannot reuse the old
revision. Production layouts omit the poller. Watcher shutdown removes its
signal; one-shot preparation removes a signal left by an interrupted process.

Focused regressions and all 369 engine tests pass, as do workspace typechecks,
the packaged engine build, installed-CLI watcher/child lifecycle acceptance,
actionlint, syntax/diff checks, and deep/strict validation of all 55 docs routes
and 23 examples. Browser acceptance now requires automatic content and image-only
reloads, no reload on invalid edits, recovery and removed navigation entries,
and no production polling. Final hosted CI and fixing commit remain pending.
No local container daemon was started and no release was published.

This extends the unpublished rc.2 candidate notes. The earlier proposed release
commit 0fbf0a1 predates this work; any later release proposal must use a reviewed
commit containing its final advertised scope. Publication approval is still
pending. Remaining CLI work includes hosted JSON validation, advanced options,
combined image manifests, additional deployment/CI providers and converters.


### Hosted JSON validation — 2026-11-03

Hosted `zel check --json` now consumes a versioned engine report. It preserves
real content/deep findings and warning severity, including strict warning-only
failures. Native success is exit 0, content/strict failure 3, unavailable runtime
or inaccessible mount 4, configuration conflict 2, and incompatible/failed engine
report 1. Normal JSON mode writes one stdout report and no stderr diagnostics
after site/checker selection; explicit verbose diagnostics still use stderr.
The Node check command emits the same validation schema but retains exit 1 for
failed validation. Hosted `pages` counts loaded routes, not compiled HTML files.

The protocol validates version, target, mounted root, requested options, status
and findings before accepting success. The structured host root preserves
quotes, backslashes and Unicode; diagnostic messages stay verbatim so authored
URLs cannot be mistaken for container paths. Older engine
images fail with image-selection/pull guidance rather than parsing terminal
text. Runtime/configuration failures also receive structured reports. Content
mounts remain read-only. Invalid CLI flags and discovery failures before the
checker is selected retain ordinary CLI errors.

Regressions cover engine findings, strict warnings, bootstrap failures, malformed
or inconsistent reports, native option forwarding, host path escaping and
missing runtimes. The installed-package/real-container acceptance script now
exercises JSON success, deep failure, strict warnings and recovery.

Implementation commits: `9a3d740`, `06e9d1e`, `9d89dd5`; review and hosted
acceptance are tracked in [PR #63](https://github.com/mylife-inc/Zellij/pull/63).
Local verification passed: 321 Rust tests (one real-container test intentionally
ignored locally), warnings-denied Clippy, formatting, 372 engine tests,
workspace typechecks, engine package build and installed JSON checks. All 55
documentation routes and 23 examples pass deep/strict validation; actionlint,
JavaScript syntax and diff checks pass. Rust fixtures used serial execution and
loopback access for mock HTTP services. Two obsolete tests expecting hosted
JSON rejection were updated to assert its structured runtime failures.

The dependent reload implementation is `3ddd2cd` plus image-cache correction
`8d3d4f1`, tracked in [PR #62](https://github.com/mylife-inc/Zellij/pull/62).
Both Chromium and WebKit passed automatic content/image reload, invalid-edit
preservation and recovery in job 126401231393 of run 41777437141. Existing
showcase browser coverage also passed 317 tests. Current full platform/container
results remain available in the PR checks; refresh them before relying on a
release source. Neither implementation publishes a release or deploys production.

Remaining CLI scope includes additional deployment and CI providers, combined
multi-platform image manifests/build arguments, broader import fidelity,
hosted dev host/browser options, and advanced build overrides. The Netlify
hosting-injected hydration failure remains a separate provider issue.


### Development host and browser options — 2026-11-03

Implementation `222ffda` adds `zel dev --host`, `--open` and
`--browser <executable>` for portable and hosted sites. Defaults remain portable
loopback (`127.0.0.1`) and hosted IPv4 wildcard (`0.0.0.0`). `localhost` and
literal IPv4/IPv6 addresses are accepted; wildcard browser URLs use loopback.
Portable preview validates request hosts against the connection's local address,
including dual-stack address normalization, while retaining path/source guards.

Hosted development owns a uniquely named container. Successful engine preparation
publishes its session and current `basePath` with the development revision. The
CLI confirms both container metadata and the matching HTTP response before
announcing/opening the URL, once per session. Failed startup, occupied ports and
unrelated listeners cannot cause a browser launch. Current engine metadata and
a reachable published port are required; after five minutes without confirmation,
manual-opening guidance is printed and development continues. Ctrl-C uses bounded
runtime stop/removal, reaps the foreground client and retains the dependency cache.
Quiet cancellation does not replay runtime logs as an unexpected failure.

Local verification: 322 Rust tests passed (one real-container test intentionally
ignored), warnings-denied Clippy, formatting, 374 engine tests, workspace
typechecks and package build. All 55 documentation routes and 23 examples pass
deep/strict checks. Native fake-runtime/browser acceptance passed IPv4/IPv6,
readiness delay, prefix handling, one-time launch, wrong-session rejection,
startup/port/browser failures, quiet mode and cancellation during startup/serving.
No local Docker or GUI browser was used. The new native acceptance runs on all
three CI operating systems; real-container/browser coverage now checks explicit
loopback publishing, prefixed browser opening and container/port cleanup. CI
results must be checked before merging; this entry records local evidence only.

Prior milestones are now merged: PR #62 as `e6a83bbc7fff5ed383d146a04fa2b5c25d83bb29`
with all 13 checks passing in run `41780233137` on `caf5e3c`; PR #63 as
`a3a1571a2a479b8d8dd679a30b31a23b1c3afe21` with all 13 passing in `41780276435`
on `09d1b65`. Each merge tree matches its tested head. Main `a3a1571` CI
`41782982430` and Base image `41782982399` also passed. Earlier failed runs remain
part of the dated history above.

Remaining CLI scope: advanced build overrides, image build arguments and combined
multi-platform manifests, additional deployment/CI providers, and broader import
coverage/fidelity. The Netlify hosting-injected hydration issue remains external.
No release publication, production deployment or domain change is included.


### Development options — acceptance follow-up, 2026-11-04

PR #64's core implementation is `222ffda`. Follow-up `d0eae61` releases the
session's temporary identity before the inaccessible-mount diagnostic exits;
native regressions verify cleanup for invalid content and inaccessible mounts.
Run `41795843334` on `d0eae61` passed 12 of 13 jobs, including all three native
development-option suites and six native package/install jobs. Its CLI job
failed before starting a container: the new browser fixture had made the app
directory nonempty before scaffold initialization. This was an acceptance setup
error, not evidence that container startup or cleanup had passed.

Harness correction `3b95629` initializes the app first, waits for shutdown output
to drain with a bounded deadline, and runs real development acceptance before
long image-publication checks. The scaffold/launcher setup, JavaScript syntax,
actionlint and diff checks pass locally without Docker or a GUI browser. Final
container results are tracked in [PR #64](https://github.com/mylife-inc/Zellij/pull/64).
Earlier run `41794505393` was canceled after the startup cleanup correction;
it did not establish final acceptance. No release or production deployment was made.


### Development options — final acceptance, 2026-11-04

PR #64 merged as `706f5430772a13ba9428aeef0d60f3237fd2dd4f` after all
13 checks passed in run `41798421655` on `bae81ce2cdc63616ed20c8b8c39b71dd9653b49b`.
The tested head and merge have identical trees. Refreshed main CI `41801848062`
and Base image `41801847816` both passed. This supersedes the pending acceptance
status above while retaining its earlier failure/correction history.

### Hosted build overrides — local implementation, 2026-11-04

The current branch adds invocation-only `--base-path`, `--site-url` and
`--no-check` to hosted `build`/`b`, static/Next image commands (`si`/`ni`), legacy
Next image builds and hosted Dockerfile generation (`idf`). Portable Markdown
adapted into a Next image accepts them; pure portable HTML/static images reject
them before runtime probing. Deploy/publication/CI generation do not yet accept
these three flags. Existing delivery callers retain default behavior.

An explicit prefix overrides the authored/wrapper prefix; `/` clears it. The
public site URL is an HTTP(S) origin only, with the path supplied separately.
Both reach rendering, SEO metadata, image layout and health checks. Next images
retain settings at runtime, including unknown-route content loads. Authored
configuration is validated before applying overrides and is never rewritten.
`--no-check` removes only the separate preflight; preparation, schema, rendering
and output ownership validation remain active. Failed exports preserve the last
successful site.

A typed Rust settings value crosses the CLI/engine boundary as versioned,
hex-encoded JSON in `ZELLIJ_BUILD_OVERRIDES`. User strings never become shell or
Dockerfile instructions. Engine capability `buildOverridesVersion: 1` is checked
before installation; older images fail with upgrade guidance. The shared Node
bootstrap validates/normalizes the protocol and supplies both Next configuration
and content sources. The production runtime cache keys include these settings.

Implementation: `62ec4f1`. Local verification passed: 323 Rust tests (one real
container test intentionally ignored locally), warnings-denied Clippy, formatting,
415 engine tests, workspace typechecks, package build and isolated installed
checker acceptance. All 55 documentation routes and 23 examples pass deep/strict
validation. Native fake-runtime acceptance covers aliases, generated recipes,
root reset, preflight selection, invalid arguments, portable rejection and source/
output preservation. JavaScript syntax, actionlint and diff checks pass. Rust
fixtures ran serially with loopback access for their existing mock services.
No local Docker or GUI browser was used. CI adds the native fixture on all three
operating systems plus real export/static-image/Next-image URL and health checks;
final platform/container results are pending.
Remaining CLI scope includes image build arguments and combined multi-platform
manifests, propagation of these options to delivery/CI where appropriate,
additional deployment/CI providers and broader import fidelity. Netlify's
hosting-injected hydration problem remains external. No release publication,
production deployment or domain change is included.


### Hosted build overrides — first CI follow-up, 2026-11-04

[PR #65](https://github.com/mylife-inc/Zellij/pull/65) tracks implementation
`62ec4f1` and context/probe regression `23d471e`. In run `41814544611`, the real
hosted override export checks passed, including canonical/sitemap/robots URLs,
root reset, invalid-content preservation and old-engine rejection. Engine,
showcase and all six native package/install jobs passed. The Windows portable
job failed earlier in the existing development-options fixture with an HTTP
`ECONNRESET`, before reaching the new override fixture.

Correction `fc3397f`: the fake development server used a single socket read, which could respond and
close before a complete request arrived. The correction explicitly sets accepted
sockets to blocking mode (Windows can inherit listener mode), consumes bounded
complete headers, adds a deliberately fragmented request regression, and labels
HTTP failures. It adds no blanket retry and does not change production server
behavior. The superseding CI run must verify Windows and full image acceptance;
this partial run is not claimed as complete acceptance.

The corrected native development suite, including fragmented headers, passes
locally with fake browser executables and loopback sockets. JavaScript syntax,
actionlint and diff checks pass. The new build-override fixture now runs before
longer native preview fixtures in each platform job.


### Hosted build overrides — final acceptance, 2026-11-04

PR #65 merged as `58623ff683e7597471607c59825966dec44d158e` after all 13
checks passed in run `41815732030` on `a92b80c59bee232773267c20dc07ea98015e42d0`.
The merge and tested head have identical trees. This includes Windows acceptance
after fixture correction `fc3397f`, real hosted exports, both image kinds and
browser/health checks. Earlier failed/canceled evidence remains above.
Main Base image run `41819057643` passed; main CI `41819057870` was still running
when refreshed for this next milestone. Native rc.1 remains the latest published
release; no rc.2 publication or production/domain change was performed.

### Build overrides in delivery and generated CI — 2026-11-04

Implementation `c20d48b` extends `--base-path`, `--site-url` and `--no-check` to
`netlify`, `deploy`, `publish`/`pub`, `ship`, and GitHub workflow generation.
The same typed options reach the single hosted export or image build before
upload/push. Pure portable static delivery rejects them before runtime or remote
work; portable-to-Next publication accepts them. Dry runs show normalized values,
including `/` for an explicitly cleared prefix, and the skipped content preflight.
Netlify credentials/site access still precede the build. Failed builds never
create a deployment or push. Temporary exports, authored files and prior local
output retain their existing protections.

`ship` keeps its converted site and saved delivery destination after failure,
but never saves invocation-only build settings. Both success and failure guidance
say to repeat the applicable overrides. Bare saved `deploy` uses discovery or
`--site-root`; its first positional remains the explicit provider selector.

Generated workflows serialize URL settings as environment values and pass quoted
arguments only to build/delivery commands. GitHub expression syntax in values is
rejected. `/` retains its root-reset meaning. `--no-check` removes the separate
workflow deep/strict check and skips the build's preflight; preparation/schema/
rendering, credential/runtime checks and publication event gates remain active.
Without it, the workflow validates authored content as before. The generator
requires version 0.2.0-rc.2 or later when these flags are used; it still does not
publish or verify release availability. Omission on regeneration restores default
checks/settings. Existing generated release-acceptance workflows are byte-identical.

Local validation passed: 325 Rust tests (one real-container test intentionally
ignored locally), warnings-denied Clippy, formatting, all 12 workflow tests and
11 parser tests after the command enum allocation correction. Native fake-runtime
coverage verifies publication aliases, no push on failed builds, imported-site
retention, dry plans, root reset, portable rejection and unchanged source. A Bash
replay verifies shell-looking prefix values remain literal; workflow failures
preserve the previous file. Mock Netlify tests retain authentication preflight.
Generated workflows pass actionlint with and without preflight; all 55 docs
routes and 23 examples pass deep/strict validation. Script syntax/diff checks pass.
No local Docker or GUI browser was used.

CI now exercises prefixed imported static and Next images through push/pull,
browser navigation, canonical/sitemap metadata and health checks. Hosted Netlify
acceptance uses an overridden deployment prefix/origin and verifies invalid
content under `--no-check` cannot create a deployment. Final platform/container
results must be refreshed from the pull request before merging.

Remaining CLI work includes image build arguments, combined multi-platform image
publication, additional deployment/CI providers and broader import fidelity.
The Netlify hosting-injected hydration issue remains external. Candidate release
publication and production migration remain separate, pending decisions.


### Delivery overrides — CI harness correction, 2026-11-04

[PR #66](https://github.com/mylife-inc/Zellij/pull/66) tracks `c20d48b` and
context record `d634507`. Run `41821800581` failed the macOS workflow test because
its system Bash cannot execute the existing `${REPOSITORY,,}` expression in the
Ubuntu-only legacy image job. Correction `0c1802a` runs full workflow-shell replay
on Linux, matching the generated runner. All operating systems still check the
workflow data, option placement, release compatibility, rejection/preservation,
and native delivery forwarding. No product behavior or generated workflow changed.
The focused regression, formatting and diff checks pass locally after correction.
Final CI acceptance is pending on the replacement head; the first run is not
claimed as complete acceptance.


### Delivery overrides — final acceptance and main follow-up, 2026-11-04

PR #66 merged as `b7a076af0a72302f9c44f66e295ae9b208983ed8` after all 13
checks passed in run `41822262323` on `c3b94705fdb096ce5f699e9c30172b414aa937c4`.
The tested head and merge have identical trees. The changed Netlify export/upload
and static/Next ship image tests passed, as did Linux/macOS/Windows acceptance.
The earlier macOS harness failure remains recorded above.

Main Base image run `41826282353` passed. Main CI `41826282286` passed 12 jobs;
Windows ARM packaging compiled Rust successfully, then PowerShell crashed with
`Internal CLR error (0x80131506)` while creating the ZIP. Only that failed job
was rerun; attempt 2 was queued when this entry was written. This is not claimed
as a source fix or a fully successful main run. Native rc.1 remains the latest
published release; no rc.2 tag/release or production/domain change occurred.

### Image build arguments — 2026-11-04

Implementation `86b6d2d` adds repeatable explicit `--build-arg NAME=VALUE` to
static/Next image commands and aliases, legacy image builds, printed Dockerfiles,
image publication, image `ship`, and generated GitHub image workflows. Values
are typed argv data. Names are ASCII identifiers; values allow Unicode, spaces,
additional equals signs and literal shell syntax, but no control characters.
Empty values are explicit and repeated names use the last value. Environment
lookup is not implicit. Zellij-owned settings, historical FEATURES and automatic
platform arguments are reserved for their dedicated flags.

Recipes declare names only, before builder commands. Hosted final runtime stages
do not redeclare arguments or receive added ENV settings. Portable static images
apply them only during container packaging, after native Markdown compilation.
Proxy/BUILDKIT arguments are forwarded without extra declarations. Printed
recipes require the values again when built. Build arguments are not secrets:
builders can retain them in metadata, and build scripts can bake them into files.
Authored content and saved configuration are unchanged. Dry plans omit values;
ship success/failure guidance explains that arguments must be repeated.

Static-folder builds, Netlify delivery and Netlify ship reject image arguments
before output, credentials or remote requests. Image build failures do not push.
Generated workflows pass numbered environment assignments as quoted arguments
only to build/publication commands, never checks or prebuilt pushes. GitHub
expressions are rejected before writing; older CLI pins are rejected (minimum
0.2.0-rc.2, still unpublished). Regeneration without arguments removes them.

Local verification: 328 Rust tests passed, one real-container test intentionally
ignored locally; warnings-denied Clippy and formatting passed. Native fake clients
verify Docker/Podman/nerdctl argv, aliases, portable/hosted recipes, publication,
ship retention, omission/reset, dry plans, early rejection and unchanged source.
Existing override acceptance and all six generated release workflow comparisons
pass. Generated static/Next/legacy image workflows pass actionlint with combined
build arguments, features, URL overrides and ARM64 selection. All 55 docs routes
and 23 examples pass deep/strict checks; script syntax and diff checks pass.

CI adds the native fixture on all three operating systems. Real static/Next ship
acceptance uses a custom builder script that consumes an exact literal argument
and emits a public JSON file, then verifies the pulled artifact over HTTP and
checks that argument names are absent from runtime ENV. No local Docker or GUI
browser was used. Final CI acceptance must be refreshed before merging.
Remaining work includes combined multi-platform image publication, additional
providers and broader import fidelity. Netlify's hosting-injected hydration issue
and the separate release/production-migration decisions remain pending.
