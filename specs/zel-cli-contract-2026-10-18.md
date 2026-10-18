# zel CLI command contract

Status: **runtime policy, configured-site layout, working-command shortcuts, completions, images and registry publication implemented in source; additional capabilities remain proposed**.
The runtime prerequisite policy in §2.9 is implemented in source. The configured site layout and repository-root default below are implemented,
including `build.out` and `preview.port`. Shortcuts for existing handlers and
shell completions are implemented. `image.name` and registry-only `publish`
settings are implemented; shared selectors, provider delivery, CI fields and
other command refinements remain proposed.
See the [inventory](zel-cli-inventory-2026-10-17.md) for current implementation;
the [original specification](zellij-cli-spec.md) remains design history.

The full CLI covers authoring, inspection, import, validation, local development,
portable and hosted builds, packaging, publication, automation, and maintenance.
No family disappears because a shorter workflow is added.

## 1. Agreed layout and argument meanings

```text
software-repository/
  docs/
    specs/
    zellij/                 # Any name is allowed
      zel.yaml              # Identifies this site
      content/              # Markdown, hosted content where applicable, assets
      site/                 # Generated HTML; not an authored-content directory
  .github/workflows/        # Repository workflows, outside the site root
```

From the repository root, `zel build` uses `docs/zellij/zel.yaml`.
`zel build docs/manual` selects `docs/manual/zel.yaml`, reads its `content/`,
and writes its `site/`. A custom directory is supplied explicitly; do not scan
the whole repository for a differently named site. The folder name does not
identify the format. Markdown belongs under `content/`, not beside `zel.yaml`.

Command signatures below use distinct argument roles:

| Argument | Meaning |
| --- | --- |
| `[site]` | Parent directory of `zel.yaml`; omitted means the configured-site default. |
| `<source>` | External input for detection/import, possibly another tool's repository. |
| `[destination]` | A folder a creation command will write; it need not exist yet. |
| `<tag>` | Container image reference, not a directory. |
| `[file]` | An explicit output file; omission may mean stdout. |
| `<name>` / `<type>` | A content object or schema type, not a site location. |

Implemented resolution includes one convenience: when the current directory itself
contains `zel.yaml`, use that site. Otherwise use `./docs/zellij/zel.yaml`.
Explicit directory arguments win. The proposed `--site-root` / `--config` selectors
are not registered yet; their eventual conflicting selections must fail; configuration does not redirect an explicit site path. There
is no upward or recursive discovery. Inside `content/`, use an explicit `..`
site argument or change to the site root. A missing default gives the expected config path
and an example with an explicit custom site, not an unrelated fallback.

CLI paths are relative to the effective working directory (`--cwd` if given).
Paths inside configuration are relative to its directory. Navigation, image,
and Markdown references are relative to the content source as appropriate.
The default output is `<site>/site`; an explicit `--out` can override it.
Source/output overlap rules compare `content/` with output, not the site-root
container with its own generated output. Generated output retains ownership
checks, staging, and protection of authored files.

## 2. Alias rules

Aliases call the same handler with the same options, defaults, errors, and exit
status. Help and completion show both names. Examples for automation use
descriptive names; interactive examples may show either form. There is no
automatic abbreviation of arbitrary prefixes, and aliases are built into the
binary rather than dependent on a shell. Short existing names need no alias.

Tables describe the proposed contract. **I** means a current implementation
exists, possibly with different arguments/defaults; **S** means a registered
stub; **N** means no command at the reconciliation baseline. The working-command
shortcuts are now implemented: `init`, `nd`, `ns`, `np`, `ng`, `npr`, `ex`,
`sections`, `themes`, `formats`, `targets`, `ctx`, `cv`, `mig`, `c`, `s`, `v`,
`b`, `si`, `ni`, `ir`, `ip`, `idf`, `pub`, `netlify`, and `comp`. `ci run` is implemented with `ci publish`
as a compatible spelling. `completions` now supports Bash, Zsh, Fish, PowerShell,
and Elvish. Help and completion derive from the same argument schema.

The proposed **capabilities** in the tables are not all implemented: conversion
still writes raw hosted content and `view` remains portable-only. Static-image
packaging, Next packaging with a portable Markdown adapter, `idf --kind static|next`,
and registry/Netlify publication via `publish` / `pub` are implemented. `deploy netlify`
and `netlify` are implemented, with bare `deploy` using saved publication. `na`
and bare `ci [site]` remain unregistered. See the [current shortcut reference](../examples/zellij-docs/content/02-cli/shortcuts.mdx).

### Creating

| Command | Shortcut | Role and default | Baseline |
| --- | --- | --- | --- |
| `zel new portable [destination]` | `zel init [destination]` | Create `zel.yaml` and `content/` for portable Markdown; default `docs/zellij`. | I |
| `zel new docs [destination]` | `zel nd [destination]` | Create a hosted YAML/MDX site with the same outer layout; default `docs/zellij`. | I |
| `zel new app <destination>` | `zel na <destination>` | Create editable Next.js wrapper files; select existing content with `--site-root`. `--with-docs` creates a hosted site at `<destination>/docs/zellij`. | S |
| `zel new section <type>` | `zel ns <type>` | Print a section template, or write an explicit `--out`; stdout needs no site. | I |
| `zel new page <name>` | `zel np <name>` | Add hosted section files under selected `content/`; retain printed `pages.yaml` entry. | I |
| `zel new guide <name>` | `zel ng <name>` | Add a hosted guide under selected `content/`. | I |
| `zel new project <destination>` | `zel npr <destination>` | Create a software repository with `docs/zellij`, product directory, and workflow. Product language remains a hint for the user's own initializer. | I |

`new app` creates files; `image next` creates an image. Both use the same wrapper
template. Existing destination protection remains. Hosted page/guide/section
writers must explain a renderer mismatch rather than insert hosted YAML into
a portable Markdown site. `new project --docs-dir` selects the site-root path
relative to the new repository, defaulting to `docs/zellij` in the new contract.

### Inspecting

| Command | Shortcut | Role | Baseline |
| --- | --- | --- | --- |
| `zel explain <type>` | `zel ex <type>` | Inspect one content schema; no site required. | I |
| `zel list sections` | `zel sections` | List the compiled section catalog. | I |
| `zel list themes` | `zel themes` | List themes. | I |
| `zel list formats` | `zel formats` | List implemented import formats. | I |
| `zel list targets` | `zel targets` | List output capabilities with runtime requirements. | I |
| `zel context [file]` | `zel ctx [file]` | Write the content reference or JSON schemas; default stdout. | I |

### Importing

| Command | Shortcut | Role | Baseline |
| --- | --- | --- | --- |
| `zel detect <source>` | Same | Inspect another documentation tree without writing. | I |
| `zel convert <source> --out <destination>` | `zel cv ...` | Convert to an explicit new configured site; retain `--dry-run` and `--from`. | I |
| `zel migrate <source> --out <destination>` | `zel mig ...` | Convert and validate that site, without publication. | I |

An import source is always explicit. For the new configured-site output,
conversion writes `<destination>/zel.yaml` and `<destination>/content/` and
chooses the hosted renderer for existing converters. Keep the old raw-content
output behavior available explicitly during transition (section 7); never
silently add an extra `content/` layer to a legacy invocation.

### Checking and local work

| Command | Shortcut | Role | Baseline |
| --- | --- | --- | --- |
| `zel check [site]` | `zel c [site]` | Validate using the selected renderer; `--strict`, hosted `--deep`. | I |
| `zel fmt [site]` | Same | Format supported YAML; `--check` reports differences without changing files. | S |
| `zel watch [site]` | Same | Repeat validation after edits, without serving or publishing. | N |
| `zel dev [site]` | Same | Authoring server with change handling; select portable or hosted backend. | I: hosted only |
| `zel serve [site-or-output]` | `zel s [site-or-output]` | Serve existing output. A configured root resolves to its `site/`; an explicit export folder remains supported. No build. | I |
| `zel view [site]` | `zel v [site]` | Build, serve, open a browser; optional `--watch` and `--no-open`. Extend the current portable implementation to hosted static output. | I: portable only |
| `zel build [site]` | `zel b [site]` | Generate a folder of HTML with the configured renderer. | I |

`dev` is the editing loop. For portable content it can share the watch/preview
backend without opening a browser by default. Hosted development retains Next's
development backend. `view` previews generated output; `serve` previews output
already on disk. A failed watch rebuild retains the last good preview.

### Images

| Command | Shortcut | Role | Baseline |
| --- | --- | --- | --- |
| `zel image static [site]` | `zel si [site]` | Build HTML and package it with an HTTP server. | I |
| `zel image next [site]` | `zel ni [site]` | Build the Next.js wrapper and package its server. | I |
| `zel image run <tag>` | `zel ir <tag>` | Run that image locally; `--port` selects the host port. | I |
| `zel image push <tag>` | `zel ip <tag>` | Push that image; retain registry selection. | I |
| `zel image dockerfile [site] --kind <kind>` | `zel idf [site] --kind <kind>` | Print the recipe for explicitly selected `static` or `next`; retain the legacy no-kind Next form. | I |

Bare `zel image` shows its operations, without choosing an image kind. Each
short build command identifies its contents. Preserve `zel image build` and
`zel build --target image` as compatibility spellings for the Next image path;
do not change them to static images depending on the last command run.

Use distinct generated default tags for new static and Next paths so they do
not overwrite one another. Explicit tags win. Preserve the legacy Next tag
default for legacy invocations and explain the new defaults in migration notes.
`image run` retains its existing omitted-tag behavior for compatibility, with
an accurate explanation; canonical examples pass a tag. No implicit push or
deployment occurs during either image build command.

Static packaging uses the selected renderer's HTML output. The implementation
uses a temporary native export or a hosted export inside the image build, leaving
existing local output untouched. The final unprivileged NGINX image serves on
port 3000. Hosted deployment prefixes are preserved. New static tags use
`<normalized-site-name>-static:latest`; legacy Next defaults are unchanged.
Printed portable-static recipes take generated HTML as their context; hosted
static/Next recipes take hosted content. `image next` / `ni` adapts portable
Markdown into a private generated content tree, preserving document and navigation
semantics while using the hosted guide presentation. It defaults to a separate
`<normalized-site-name>-next:latest` tag. `idf --kind next` remains hosted-only;
`ni` owns portable adaptation. Next health checks honor the configured basePath.

Native portable HTML must retain direct-file viewing outside the image. Hosted Next static
exports keep their HTTP requirements. For plain Markdown to a Next image,
create a temporary hosted-content adapter and wrapper; preserve the originals
and verify navigation/assets/config mapping. Do not assume the current native
configuration can already be passed to Next unchanged.

### Delivery and CI

| Command | Shortcut | Role | Baseline |
| --- | --- | --- | --- |
| `zel deploy <provider> [site]` | `zel netlify [site]` for Netlify | Validate, build, and deploy to an explicitly chosen provider. | I: Netlify |
| `zel publish [site]` | `zel pub [site]` | Execute saved image build/push or Netlify deployment. | I |
| `zel ship <source> --out <destination>` | Same | Import into a new configured site, then execute delivery; preserve source. | S |
| `zel ci init github [--site-root <site>]` | `zel ci [site]` | Generate a GitHub workflow using saved settings. | I: saved settings and shorthand |
| `zel ci run` | Existing `zel ci publish` | Dispatch an existing workflow; retain `--workflow` and `--ref`. | I: `ci publish` |

`deploy` specifies the destination provider at the command line. `publish` uses
saved delivery settings; `--to` can explicitly override the configured provider.
Bare `zel deploy` is an exact synonym for configured `publish`, preserving
the previously proposed convenience. A single argument to `deploy` is a provider;
use `publish docs/manual` to select a site without spelling out the provider.
`ship` adds import first, retaining `--provider` for an explicit deployment
override. Its `--out` is the converted site root, not the generated HTML folder.

All three resolve one execution plan: validate, build once, deliver. Container
deployment may require build, registry push, then cloud deployment. A registry
publication can stop after push. Compositions call shared operations, not a
public `build` followed by a public `deploy` that builds the same thing again.
Stop at the failing stage and report completed stages; do not claim that a
successful build proves a migration preserved every source-tool feature.

Netlify is the first requested direct deployment integration. Keep Cloudflare,
Vercel, GitHub Pages, S3, Fly, and Railway in the broader inventory; implement
and label each separately rather than advertising seven working integrations.
Reject unsupported provider/artifact combinations instead of overriding the
chosen image or renderer silently.

`ci` generates; `ci run` executes. Reserved action names (`init`, `run`,
`publish`) are parsed as actions; a site folder with that name can be supplied
as `./run` or with `--site-root`. GitHub is the default generator; retain the
full form for future generators. Generate at the repository's workflow root,
with site paths relative to that repository. Default triggers are pull requests,
push to the configured branch, and manual dispatch. Tags/schedules are optional
settings. PRs validate/build; publication requires configured delivery and an
allowed non-PR trigger. Configuration, content, and the workflow itself must
be covered by change filters. Use a pinned compatible tool release.

The proposed generation default writes local configuration/workflow files.
Credential provisioning is a separate explicit `--provision` choice. Keep the
current `--no-provision` spelling accepted, and document the default change at
the version boundary; both short and full generation forms use the same policy.

### Maintenance and advanced use

| Command | Shortcut | Role | Baseline |
| --- | --- | --- | --- |
| `zel doctor` | Same | Diagnose required tools and selected site settings without exposing credentials. | I: container diagnostics |
| `zel pull` | Same | Download the compatible configured engine image. | N |
| `zel upgrade` | Same | Update the native CLI, preserving a working binary on failure. | N |
| `zel version` | Same | Report binary, schema, and engine versions; retain simple `--version`. | N |
| `zel completions <shell>` | `zel comp <shell>` | Completion for canonical names and aliases. | N |
| `zel eject [site] --out <destination>` | Same | Extract an editable wrapper from the selected engine distribution. | N |
| `zel features <selection> [--site-root <site>]` | Same | Configure optional renderer capabilities with explicit renderer-aware mapping. | N |

`new app` scaffolds a wrapper; `eject` extracts the one from a selected engine
distribution. `pull` updates an engine image; `upgrade` updates the executable.
Standalone native operations must remain usable without Docker or Node.

### 2.9 Container prerequisites — implemented 2026-10-18

The user authorized implementation after reference reconciliation and asked for
defined behavior when Docker is absent. The first increment establishes:

- Native portable build/check/serve/view and Dockerfile generation need no
  container runtime. Generation resolves `ZELLIJ_IMAGE` without detection.
- Container commands try Docker, Podman, then nerdctl. Only Docker receives
  discovered Docker socket overrides; Windows uses client connection settings.
- Nonempty Docker (`DOCKER_CONTEXT`, `DOCKER_HOST`), Podman
  (`CONTAINER_CONNECTION`, `CONTAINER_HOST`), or nerdctl (`CONTAINERD_ADDRESS`,
  `CONTAINERD_NAMESPACE`) endpoint variables select that runtime. No fallback
  to another runtime/socket is allowed in that case; conflicting runtime
  selections fail. Clients retain their own within-runtime precedence.
- Missing executable, unexecutable client, unsuccessful info and timeout are
  distinct failures with OS-appropriate guidance and exit code 4. There is no
  automatic install/start, renderer change, or remote execution fallback.
- Info probes get three seconds each within a shared twelve-second budget;
  version and image metadata probes get two seconds each. Runtime-dependent
  operations themselves retain their existing lifetimes. Probe processes are
  killed and reaped on timeout. Explicit endpoint values and failed client
  output are not echoed; doctor lists the variable name instead.
- Doctor remains a container preflight: no runtime means exit 4; a responding
  runtime means exit 0 even when version/image metadata is unavailable. It is
  not required before a portable build and does not validate site content.

The current command reference is [doctor.mdx](../examples/zellij-docs/content/02-cli/doctor.mdx).
Site layout, working-command shortcuts and completions are also implemented;
additional image/delivery commands still need their own implementation increments. This policy does not imply a new release
or publication of the local documentation.

## 3. Configuration: implemented subset and remaining proposal

Use `zel.yaml` as the site-root configuration, with a version marker separating
the new contract from existing flat portable configs. No second TOML defaults
file is needed for new sites. Minimal portable example (implemented in source):

```yaml
version: 1
renderer: portable
name: Product documentation
```

Hosted scaffolds use `renderer: hosted`; their authored `site.yaml`, section
definitions, and MDX stay inside `content/`. Existing hosted `site.yaml` remains
authoritative for hosted branding/theme until a deliberate migration exists.
Do not silently merge two competing sets of presentation settings. Existing
portable identity, navigation, branding, and Markdown options remain available
in the versioned portable configuration, with paths relative to `content/`.

Omitted settings mean source `content/`, output `site/`, preview port 3000, and
no configured remote delivery. `build`, `preview`, `image.name`, and the
registry and Netlify forms of `publish` are implemented. For example:

```yaml
image:
  name: ghcr.io/acme/manual:v1
publish:
  action: push
  image: static
```

`publish.image` accepts `static` or `next`. `image.name` also supplies the saved
default for new `si` / `ni`; explicit `--tag` wins. An explicit configured name
is shared by both image kinds; generated defaults stay distinct when absent.
Legacy image commands retain their old defaults. `publish` requires saved
intent and an explicit destination from config or `--tag`; it never publishes
a generated folder-derived name. Provider/site/credential fields are rejected
for registry publication. `pub --dry-run` resolves the plan without tools,
content validation, builds or remote requests; `--json` remains unsupported.

The provider and `ci` blocks below are implemented. While the development
package still reports 0.1.3, CI generation requires an explicit compatible
release pin (minimum 0.2.0, including candidates); generation does not establish
that the selected release has been published:

```yaml
build:
  out: site
preview:
  port: 3000
image:
  name: ghcr.io/acme/manual
publish:
  action: deploy
  provider: netlify
  site: acme-docs.netlify.app
ci:
  system: github
  branch: main
  zel-version: 0.2.0
  triggers: [pull_request, push, manual]
```

For registry-only publication, `publish.action: push` and `publish.image:
static` (or `next`) select the image path; `image.name` supplies its registry
reference. Registry-only delivery has no deployment provider. Container cloud
delivery uses `action: deploy` with an explicit image kind and a compatible
provider. The eventual schema must reject irrelevant/conflicting fields.

Settings precedence is explicit CLI options, then individually documented
environment overrides, then configuration, then defaults. Retain `ZELLIJ_IMAGE`;
do not invent an environment variable automatically for every configuration key.
Credentials remain in environment/credential stores and CI secrets. A site ID
or registry address is configuration, not a credential. An explicit deployment
command supplies intent; ordinary repeatable runs need no extra prompt.

## 4. Shared options and output

- `--site-root` selects a configured site for commands whose positional argument
  has another role. It does not replace `--site`, the existing provider site-ID
  option. Add a clearer `--site-id` synonym when implementing deployment.
- `--config <file>` explicitly selects a configuration file and its containing
  root; `--cwd <directory>` changes the base for CLI paths without changing Git
  state. These are proposed options, not current capabilities.
- `--out` is generated-output destination; `--port` is preview port. Do not
  overload single-letter options differently across command families.
- `--json` is a documented capability per command. Structured summaries belong
  on stdout, human progress on stderr; unsupported combinations fail before
  work. Long-running servers need an explicit event-stream contract before JSON
  support is advertised. Preserve raw `context --json` schema output.
- `--quiet`, `--verbose`, and `--no-color` are shared future output controls;
  neither quiet nor verbose changes validation or publication behavior.
- `--yes` only answers actual interactive setup questions; it does not grant
  overwrite authority or substitute for required settings. Existing overwrite
  policies remain command-specific and documented.

## 5. Representative proposed workflows

```sh
# New site in an existing software repository
zel init
zel v --watch
zel b
# Open docs/zellij/site/index.html directly for a portable site

# A custom site root
zel init docs/manual
zel v docs/manual --watch
zel c docs/manual --strict

# Two distinct images
zel si docs/manual --tag acme/manual-static:v1
zel ni docs/manual --tag acme/manual-next:v1
zel ir acme/manual-static:v1

# Direct deployment or saved delivery settings
zel netlify docs/manual
zel pub docs/manual

# Workflow generation and explicit execution
zel ci docs/manual
zel ci run

# Import an external tool's documentation into a separate site
zel mig ../old-manual --out docs/manual
```

These examples are design examples, not commands to copy into current CI.

## 6. Implementation sequence and acceptance

Runtime prerequisite handling (§2.9) is the first implemented increment following
the user's additional Docker requirement. The configured-site increment implements
steps 1–2 for current handlers; portable `dev`, hosted `view`, global selectors
and unimplemented handlers remain future extensions. Continue with the sequence:

1. Introduce the versioned config/site resolver, retain explicit legacy paths,
   and update portable/hosted scaffolds. Prove the user's default and custom
   layouts without inspecting unrelated `docs/specs` or generating inside input.
2. Wire all site-oriented handlers to that resolver. Keep global inspection,
   external import, destination creation, and image-tag arguments distinct.
3. Register aliases and help/completions from one catalog. Verify every alias
   has identical parsed intent to its canonical command and no naming collision.
4. Complete static-image packaging and plain-Markdown-to-Next adaptation;
   validate navigation and assets in both. Build/run/push remain separate.
5. Implement the shared delivery plan and Netlify adapter; compose publish/ship
   without repeated builds. Validate before uploading, and preserve import input.
6. Update CI generation to call the same supported operations; include build
   checks, trigger/config handling, and pinned release compatibility.
7. Finish formatter and the retained advanced/maintenance backlog in explicitly
   scoped milestones. Keep unimplemented features labeled in help and docs.
8. Publish matched docs/binaries only after command examples, cross-platform
   path handling, offline portable viewing, and aliases pass acceptance.

Required cases include conventional/custom roots, spaces/Unicode, Windows
paths, multiple sites, missing/malformed configuration, missing `content/`,
owned versus authored output, unsupported renderer options, legacy raw folders,
and manual CI dispatch. Assertions should check behavior rather than just help
text. No additional full test suite is needed merely to edit this proposal.

## 7. Compatibility and scope boundaries

- Preserve existing explicit raw-Markdown builds and flat portable configs.
  New versioned configs select the configured-site contract. A missing new
  `content/` must fail rather than fall back to scanning the site root.
- Preserve explicit hosted raw-content entry points during transition. Give
  users one migration guide; do not silently move or rewrite their files.
- Conversion needs an explicit output-layout choice for legacy invocations;
  propose `--layout site` for configured output and `--layout content` for
  existing raw-content output. The new `cv` alias has identical defaults to
  `convert`; it cannot secretly select a different layout. Keep the old default
  until a documented version boundary, then make site layout the new default.
- Keep `image build`, `build --target image`, `ci publish`, and old explicit
  directory flags working through compatibility paths. Conflicting legacy/new
  selectors produce a useful error instead of choosing silently.
- Provider and maintenance milestones remain in the catalog; this proposal
  does not assert that every historical feature is required in the first release.
- Only the site layout/default path is already agreed. Record later acceptance
  of alias names, workflow semantics, schema keys, or scope separately.

## 8. Review evidence

The draft accounts for all 39 forms in the inventory (32 registered plus seven
design-only names), including compatibility spellings. Its 29 shortcut entries
have distinct command spellings; this includes the existing `ci publish` name
for the proposed `ci run`. Alias parsing/execution still needs implementation
acceptance; a table check is not a runtime test.

Alongside this proposal, the local CI reference was reconciled against the
parser and actual generated validation-only, image, and Netlify workflows in
temporary folders. Generation used `--no-provision` for Netlify and performed
no dispatch, credential upload, image build, or deployment. The image run-tag
description and maintenance-command status were also corrected. Deep/strict
validation of the documentation passed for 47 routes and 23 embedded examples.
This evidence checks current documentation; it does not validate proposed syntax.


### Configured-site implementation — 2026-10-18

The shared resolver accepts `version: 1` with `renderer: portable|hosted`, fixed
`content/`, default `site/`, optional `build.out` and `preview.port`. Build/check,
serve/view, hosted dev, image build/dockerfile and hosted page/guide creation
use configured roots. New portable/docs/project starters use the agreed layout.
Portable presentation remains in root `zel.yaml`; hosted presentation stays in
`content/site.yaml`. Hosted configs reject duplicate presentation fields.

Bare site commands select cwd's config, then `docs/zellij/zel.yaml`; they fail
when neither exists. Explicit legacy folders and flat configs remain supported;
legacy `.` hosted discovery remains available explicitly. Serve accepts either
a configured site or an explicit output folder. View remains portable-only,
dev remains hosted-only. The running preview pins its output/port, watches only
content/config, and preserves output when the config disappears or changes
renderer/layout. Restoring the config recovers without restarting.

CI generation still targets raw hosted content, including `<site>/content` for
configured sites; it does not consume proposed image/delivery/CI defaults.
Conversion output is unchanged. Aliases and shared selectors are not registered.


### Registry-publication implementation — 2026-10-18

`image.name`, `publish: {action: push, image: static|next}` and `publish` / `pub`
are implemented. Shared core types validate configuration and resolve the
selected image/destination before execution. Local image commands and publication
use one build operation; publication preflights the runtime once, builds once,
then pushes the exact tag. A failed build cannot trigger a push. A failed push
reports the completed local build and a push-only retry command.

No implicit login, registry creation, cloud deployment, installation or daemon
startup occurs. Credentials remain with the runtime. Registry access is checked
by the actual push, not by runtime detection or dry-run. Exit 3 is retained for
unsuccessful image runtime operations, now with operation-specific diagnostics.
Original sources and previous output remain intact, and temporary contexts are
removed even on failure. `--to` remains explicitly rejected until provider
delivery exists. CI configuration and generation changes are the next stages.


## Stage 5b: native Netlify delivery (2026-10-18)

`deploy netlify` / `netlify`, saved `publish` / `pub`, and bare `deploy` share
one resolved delivery plan. Netlify accepts static HTML from either renderer.
Portable deployment uses native rendering and HTTPS; hosted deployment uses
one container export. No Node or provider CLI is required on the host. The
hosted export mounts files at its engine-resolved basePath; ordinary `build`
retains its existing output layout. Private exports preserve source and prior
local output and are cleaned up after success/failure.

Use `publish: {action: deploy, provider: netlify, site: acme-docs.netlify.app}`.
Destination precedence is `--site-id` (alias `--site`), `NETLIFY_SITE_ID`, then
`publish.site`. The site field is optional when supplied elsewhere. Credentials
come only from `NETLIFY_AUTH_TOKEN`; stored provider login is not read in this
implementation. Tokens/image kinds/production flags are invalid in this block.
Draft preview is the default; `--prod` explicitly requests live publication.
An explicit provider supplies intent, while `pub --to netlify` refuses to
replace saved image-push intent. `--tag` is rejected for Netlify. Other providers
are rejected. `--json` remains unsupported for delivery.

Dry runs resolve only: no credentials, tool probes, content validation, output
or remote requests. Execution preflights access to an existing site, validates
and builds once, sends a SHA1 file manifest, uploads requested content once per
digest, then polls to ready. API errors are redacted, redirects refused, request
and processing time bounded. An uncertain response never automatically retries
creation. A remote deployment may finish after a client timeout; report completed
stages and direct the user to its dashboard before retrying. No site creation,
remote build, functions, login, live provider acceptance, release or documentation
deployment is part of this implementation. Ship/import composition and saved CI
settings/workflow generation remain open.


### Implementation stage 5c — saved CI and workflow generation (2026-10-18)

Short `ci [site]` and full `ci init github` share settings, resolution and writing.
Generation locates the site's Git root, serializes workflow data, covers config,
content and workflow changes, and rejects unsafe destination links/paths. PRs
validate/build; saved publication uses native delivery with one build per run.
Publication gates restrict manual/scheduled runs to the configured branch;
explicit tag patterns enable tag publication. Netlify defaults to previews;
`ci.production` / `--prod` selects production. Registry secrets are step-scoped.
Local generation no longer provisions credentials; only `--provision` uploads a
Netlify token via gh stdin after the workflow exists. `--no-provision` remains
accepted. Legacy content/target/deploy options retain explicit override roles.

Implemented CI fields: system, branch, workflow, zel-version, triggers (push,
pull_request, manual), positive tags, numeric UTC schedules, production. The
version pin must support the 0.2.0 contract. Current public release 0.1.3 cannot
run these workflows; generation prepares a file without checking availability
or releasing anything. A compatible installed version will supply the default
pin; development builds still at 0.1.3 require an explicit pin. The project
starter prepares a workflow pinned to planned 0.2.0 and reports that prerequisite.
Real released-asset/provider acceptance remains separate from local generator,
protocol, renderer and CI tests. See ZELLIJ_CONTEXT.md for current evidence.
