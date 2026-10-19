# zel CLI command contract

Status: **configured sites, shortcuts, completions, static/Next images, registry
and Netlify delivery, CI generation, import/ship, editable wrappers, YAML formatting
and validation watching, portable dev, hosted view, version/pull/upgrade, eject
and explicit portable/hosted feature selection are implemented in source**.
Working-directory and shared site selectors are also implemented. Other
providers and remaining advanced flags remain proposed. Stage records below
and the local CLI reference describe actual support; public 0.1.3 predates these
additions and compatible release publication is still separate work.
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
Explicit directory arguments or `--site-root` select a site; combining them is
an error, including identical paths. `--cwd` changes the base for CLI paths.
`--config` remains proposed; configuration does not redirect an explicit site path. There
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
shortcuts are now implemented: `init`, `nd`, `na`, `ns`, `np`, `ng`, `npr`, `ex`,
`sections`, `themes`, `formats`, `targets`, `ctx`, `cv`, `mig`, `c`, `s`, `v`,
`b`, `si`, `ni`, `ir`, `ip`, `idf`, `pub`, `netlify`, and `comp`. `ci run` is implemented with `ci publish`
as a compatible spelling. `completions` now supports Bash, Zsh, Fish, PowerShell,
and Elvish. Help and completion derive from the same argument schema.

The tables retain their reconciliation baseline; later stage records supersede
their S/N and partial-support labels. `view` now supports portable and hosted
static output, and `dev` supports both renderers. `na` scaffolds editable hosted wrappers. Conversion supports explicit
configured-site output while keeping raw hosted content as its default. Static-image
packaging, Next packaging with a portable Markdown adapter, `idf --kind static|next`,
and registry/Netlify publication via `publish` / `pub` are implemented. `deploy netlify`
and `netlify` are implemented, with bare `deploy` using saved publication.
Configured `ship` and bare `ci [site]` are implemented. See the [current shortcut reference](../examples/zellij-docs/content/02-cli/shortcuts.mdx).

### Creating

| Command | Shortcut | Role and default | Baseline |
| --- | --- | --- | --- |
| `zel new portable [destination]` | `zel init [destination]` | Create `zel.yaml` and `content/` for portable Markdown; default `docs/zellij`. | I |
| `zel new docs [destination]` | `zel nd [destination]` | Create a hosted YAML/MDX site with the same outer layout; default `docs/zellij`. | I |
| `zel new app [destination]` | `zel na [destination]` | Create editable Next.js wrapper files; select existing content with `--site-root`. `--with-docs` creates a hosted site at `<destination>/docs/zellij`. | S |
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
chooses the hosted renderer for existing converters. Select this with `--layout
site`; the default remains `--layout content` during transition (section 7). Never
silently add an extra `content/` layer to a legacy invocation.

### Checking and local work

| Command | Shortcut | Role | Baseline |
| --- | --- | --- | --- |
| `zel check [site]` | `zel c [site]` | Validate using the selected renderer; `--strict`, hosted `--deep`. | I |
| `zel fmt [site]` | Same | Format supported YAML; `--check` reports differences without changing files. | S |
| `zel watch [site]` | Same | Repeat validation after edits, without serving or publishing. | I |
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
| `zel ship <source> --out <destination>` | Same | Import into a new configured site, then execute delivery; preserve source. | I |
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
| `zel pull` | Same | Download the selected engine image; `--image` overrides `ZELLIJ_IMAGE`. | I |
| `zel upgrade [version]` | Same | Verify and update the native CLI; `--check` inspects, `--force` reinstalls. | I |
| `zel version` | Same | Report binary, embedded schema/engine and selected image; retain `--version`; JSON supported. | I |
| `zel completions <shell>` | `zel comp <shell>` | Completion for canonical names and aliases. | I |
| `zel eject [site] --out <destination>` | Same | Extract an editable wrapper from the selected engine distribution. | I |
| `zel features <selection> [--site-root <site>]` | Same | Configure optional renderer capabilities with explicit renderer-aware mapping. | I |

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

- `--site-root` selects the existing site/content input for supported commands;
  it conflicts with another selector. It does not replace `--site`/`--site-id`,
  the implemented provider site-ID option. See stage 8a for supported commands.
- `--cwd <directory>` changes the base for CLI paths without changing shell
  directory or Git state. `--config <file>` remains a proposal for explicit
  configuration-file selection and its containing root.
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
  use `--layout site` for configured output and `--layout content` for
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


### Stage 5d — configured import and ship (2026-10-18)

`convert`/`cv` and `migrate`/`mig` share `--layout content|site`, `--from`,
`--dry-run`, and required source/`--out`. Content remains the compatibility
default. Site writes version: 1, renderer: hosted, with emitted files/assets
under content/. Migrate validates after writing; dry run skips writes/validation.
Equal-confidence format matches now require --from instead of registry-order
selection. Converter path containment includes Windows junction rejection.

Ship always writes a configured site. Select `--provider netlify` with an
existing `--site-id`/`--site` or NETLIFY_SITE_ID, or `--image static|next` with an
explicit `--tag`. Provider/image intents cannot be combined. Netlify defaults to
a draft; --prod applies only to that invocation. The nonsecret destination is
saved in zel.yaml for later publish; credentials and production intent are not.
Delivery selection and the complete import are prepared before writes. Dry run
reports conversion warnings/files/destination without tools or network.

After import, ship calls the shared resolved-plan executor; it does not invoke
public build/deploy commands. A failed build stops publication. Later failures
leave converted files for review and retry via check/publish (or image push to
retry only a failed push). It does not promise a persistent local site/ export.
Both existing converters emit hosted content; runtime/HTTP requirements apply.
This does not add portable import or imply lossless source-tool migration.
Cross-platform tests and actual container/registry/browser acceptance accompany
the source change; compatible release publication remains separate work.


### Stage 6a — editable Next.js wrappers (2026-10-18)

`new app [destination]` / `na` is implemented in source. Destination defaults to
`.` and must be empty apart from noncolliding dotfiles. There is no force option.
Existing hosted content is selected with --site-root or normal current-config /
docs/zellij discovery, resolved to an absolute path and referenced without copying.
--with-docs conflicts with --site-root and creates hosted documentation at
<destination>/docs/zellij with a landing page and guide. Portable and legacy
marketing content are rejected with guidance; image next retains its portable
adapter. Wrapper creation does not run external tools, install packages, or
create/launch a server.

The engine is not published to npm. --engine selects a built local package;
the sole implicit location is ./engine/packages/zellij in the working directory.
Preflight checks the package identity/version, wrapper-config capability and
required CLI bundles. npm's file dependency uses the actual path, with Windows
canonical prefixes normalized, not percent-encoded URL syntax. The generated
.npmrc copies that package during npm install. Node/npm are prerequisites for
running the app, and dependency installation creates the lockfile.

--name supplies starter display text and a normalized npm name. --port selects
1–65535, otherwise the selected preview.port or 3000; a saved port 0 requires an
explicit override. --features accepts comma-separated maths, diagrams or all.
The committed native template is generated from the engine initializer, shared
with image next, and checked for drift in CI.

Generated zellij.config.yaml has version: 1 and content only. All runtime,
preparation, Next configuration and check paths honor that selector. Presentation
remains in content/site.yaml; native site settings remain in zel.yaml. Relative
content is used for the starter, absolute for external sources. Moving engine or
external content requires updating the corresponding path. No npm publication,
portable wrapper adapter, package-manager choice, auto-install, or cloud upload
is implied. The public 0.1.3 release predates this command.


### Stage 6b — YAML formatting and validation watching (2026-10-18)

`fmt [site]` and `watch [site]` use normal current-config / docs/zellij selection,
with explicit custom/legacy roots. Both are already short; no additional alias.
The historical S/N baseline rows above are superseded by this implementation.
Neither command supports --json.

Fmt is native and tool-independent. Configured hosted sites select zel.yaml and
content/**/*.yaml or .yml; portable sites select only zel.yaml, leaving YAML
attachments alone. Markdown, hidden paths, node_modules, hosted assets and
configured/marked build output are excluded. Links/junctions are rejected.
Syntax-tree formatting retains comments, anchors and multiline scalars; known
section fields are ordered type/required/optional only where their context is
safe to move. Short scalar collections are compacted, quotes simplified where
the scalar type stays unchanged, and long flow collections can retain their
braces/brackets. Significant multiline whitespace and parsed values are checked
and preserved. Unsupported or value-changing transformations fail before writes.

All files are prepared before any replacement, staged on the same volume, with
closed staging handles, original permissions and a concurrent-change preflight.
Replacement is atomic per file, not a transaction across files on storage failure.
--check writes nothing: clean=0, changes=1, invalid input/path/transformation=2;
I/O=1. Formatting does not replace content validation.

Watch runs validation initially, polls byte snapshots, debounces saves and checks
serially. --deep and --strict share check semantics. Portable needs no tools;
hosted detects a runtime at startup and uses the real read-only engine check.
Unavailable runtime is exit4. Initial invalid configuration fails; invalid content
remains watched. Config deletion, malformed edits, changed renderer and linked
paths are reported without widening the root or changing renderer. Restoring
valid inputs recovers. Ctrl-C waits for the active synchronous check then exits0;
one-shot check remains the CI pass/fail operation. No output, server, browser or
publication is created. Compatible release and public docs publication remain open.


### Stage 6c — portable development and hosted preview (2026-10-18)

Portable `dev [site]` now uses the native `view --watch --no-open` workflow,
including configured-site discovery, output, port 0, browser reload and persistent
file://-capable export. No runtime probe or browser launch occurs. Hosted `dev`
retains Next.js development and requires a nonzero port. `--open`, `--host` and
`--features` are still planned; use view for explicit browser/export options.

`view` / `v` builds and serves both renderers. Hosted content needs a usable
container runtime, while the HTTP server and launcher remain native. Its staged
flat export carries the engine's resolved basePath for server routing; files stay
in ordinary build layout. The reserved loopback port, stage/index/prefix checks,
input snapshot comparison and cancellation check all precede publication.
`--name` and `--theme` remain portable-only. Hosted output requires HTTP.

`--watch` uses one serial worker with debounce, error notices and recovery.
Hosted source changes during a build discard that export and trigger a fresh
build. Configured layout, renderer, output, port and hosted basePath stay pinned;
restore invalid inputs or restart for a new layout/prefix. Initial failures exit
without browser launch. Ctrl-C stops serving, waits for any active build to finish
cleanup and discards its stage. It does not immediately terminate a container
build. Reload code is response-only, and retained portable files remain offline.

### Stage 7a — maintenance commands

`version` is local and supports JSON: CLI version/target, configuration version,
embedded engine version, schema fingerprint and configured image. `--version`
remains compact. Container contents are not inferred from tag names.

`pull [--image reference]` uses the explicit override, ZELLIJ_IMAGE, then the
existing default image. It accepts tags or SHA-256 digests, uses the common
runtime preflight, preserves explicit endpoint selection, and propagates pull
failure as exit 3 (no runtime: 4). It does not start/install Docker or log in.

`upgrade [version]` uses the stable shared Zellij release channel; `--check`
reports without changing files, `--force` permits same-version reinstalls but
never downgrades. Exact versions permit prereleases and skip API discovery.
HTTPS mirrors via --release-base-url/ZEL_RELEASE_BASE_URL require a version;
loopback HTTP is allowed for tests. Checksums, exact archive contents, binary
CPU/OS and executable-reported version must pass before replacement. Native
platform archives and replacement are tested on Linux/macOS/Windows x64/ARM64.

An OS file lock serializes upgrades and survives process crashes without staying
locked. Failed verification/download/cancellation preserves the installed binary.
Unix replacement uses a same-filesystem rename. Windows retains the previous
binary and attempts rollback if the final rename fails. Sudden power/storage
failure is not an atomic transaction guarantee on Windows. The documented
request deadlines bound stalled downloads. No external installer/tool/runtime
is required. JSON is rejected for pull/upgrade before side effects.

The current source version remains 0.1.3 and is ahead of the public 0.1.3 archive;
a compatible native release is still required before these additions can be
installed or upgraded from the public channel.


### Stage 7b — editable distribution extraction and features (2026-10-18)

`eject [site] --out <new-or-empty-directory>` references existing hosted content;
normal site discovery applies. `--with-docs` instead keeps the distribution's
starter at docs/zellij/content with a versioned hosted zel.yaml. `--image`, then
ZELLIJ_IMAGE, then the default selects the distribution. The command creates a
stopped container, copies /app and /opt/zellij, removes the container and validates
the staged layout before publication. It never starts that container or runs its
application. The engine becomes vendor/zellij with a relative file: dependency.
The selected image's wrapper sources, scripts and feature dependencies survive.
Version/configuration compatibility follows new app's embedded template contract.
External content is referenced, not copied. Source/output overlap, nonempty
output, links/special files, caches and incompatible distributions are refused.
No npm installation occurs; the user installs dependencies afterward. This
supersedes the historical 'engine installed, no npm' wording. Creation may pull
a missing image according to the runtime's usual policy. Copy/create timeouts and
Ctrl-C trigger snapshot cleanup; cleanup failures report the stopped container
name and leave the destination unpublished.

`features <maths|diagrams|maths,diagrams|all|none> [--site-root <path>]` is native.
All/none are standalone complete selections. Current-directory wrapper discovery
precedes normal site discovery; explicit --site-root selects a site or wrapper.
Portable sites update markdown.math and markdown.mermaid with comments and other
settings retained; anchored target settings are refused before writing. A wrapper
updates direct package.json dependencies and required engine overrides using the
same embedded feature map as new app. Its engine must be a compatible built local
file dependency. Other manifest fields and lockfiles are retained; npm install is
a separate user action. Hosted content without a wrapper fails with guidance.
Hosted `none` removes direct requirements, not installed/transitive packages.
The engine currently declares optionalDependencies that npm can still install;
this command does not claim strict package exclusion or a hosted rendering switch.
That pre-existing dependency-model limitation remains product follow-up work.
Both commands reject --json before effects. Eject requires a responding existing
Docker/Podman/nerdctl endpoint (exit 4 otherwise); features needs neither Docker
nor Node. Invalid arguments/layouts exit 2 and container-operation failures exit 3.


### Stage 7c — hosted rendering selection and optional peers (2026-10-18)

This supersedes stage 7b's hosted feature limitation. Editable wrappers now store
an explicit package.json `zellij.features` array ([] / maths / diagrams / both).
Native and Node initializers/setters share the engine's feature map. New wrappers
choose none unless requested; legacy wrappers without metadata retain automatic
installed-package detection. The native engine compatibility check includes
featureSelectionVersion: 1, preventing a successful write to an older engine that
would ignore the selection.

The engine's feature libraries are optional peers, with development dependencies
for its own build/tests. They are direct dependencies only in wrappers selecting
them. A fresh none wrapper does not install them. Next configuration rejects
explicitly selected but missing packages; it injects rendering flags and disabled
module aliases for Turbopack and Webpack. None disables math/Mermaid rendering even
with packages still on disk; built-in mindmaps remain enabled. Rebuild/restart is
required after selection changes. npm install is explicit and is needed when
enabling absent packages. No lockfile/node_modules deletion is performed, and
other consumers may retain the packages. Engine/CLI release publication remains
separate work.


### Stage 8a — working directory and shared site selection (2026-10-18)

Global --cwd selects an existing working directory before handlers or subprocesses
start. Its relative path is resolved from the launch directory; all other CLI
paths then resolve from it. Config-relative output and content-relative links keep
their semantics. Invalid cwd fails with exit 2; --help/--version do not access it.

Global --site-root is normalized after shortcuts into the existing handler's site
argument. Supported commands: build/check/fmt/watch/dev/view/serve, static/Next/
legacy image builds and Dockerfile generation, publish/deploy, CI generation,
eject, new page/guide/app and features. Omitted-tag image run supports it for the
legacy hosted-site tag; explicit tags conflict. Unsupported inspection, creation,
import, maintenance, image push and workflow-dispatch uses fail before side effects.
Positional sites, page/guide --dir, CI --content and --with-docs conflict, even
when paths are identical. New app/eject destinations keep their distinct meaning.
CI --content remains supported. Repeated global path flags are rejected even
across subcommand boundaries. Both selectors work before/after commands and
through aliases, with help and five-shell completion generated from the same schema.
Explicit --config selection, output controls and other providers remain work.
