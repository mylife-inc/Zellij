# zel CLI command contract

Status: **configured sites, shortcuts, completions, static/Next images, registry
and Netlify delivery, CI generation, import/ship, editable wrappers, YAML formatting
and validation watching, portable dev, hosted view, version/pull/upgrade, eject
and explicit portable/hosted feature selection are implemented in source**.
Working-directory, shared site selectors and explicit config-file selection are also implemented. Other
providers and remaining advanced flags remain proposed. Stage records below
and the local CLI reference describe actual support. Candidate 0.2.0-rc.1 is
published with six native packages and verified public artifacts; stable 0.1.3
predates these additions. Stable promotion and production docs remain separate work.
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
`--config` selects a named YAML configuration and its parent root, and conflicts
with other site selectors; configuration does not redirect an explicit site path. There
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

The provider and `ci` blocks below are implemented. Candidate 0.2.0-rc.1 defaults
to its own published version when no pin is supplied. Explicit pins require
0.2.0 or later, including candidates; generation does not establish that an
arbitrary selected version has been published:

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
  zel-version: 0.2.0-rc.1
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
  directory or Git state. `--config <file>` selects an existing YAML
  configuration file and its containing root. See stage 8b for behavior.
- `--out` is generated-output destination; `--port` is preview port. Do not
  overload single-letter options differently across command families.
- `--json` is a documented capability per command. Structured summaries belong
  on stdout, human progress on stderr; unsupported combinations fail before
  work. Long-running servers need an explicit event-stream contract before JSON
  support is advertised. Preserve raw `context --json` schema output.
- `--quiet` (`-q`), `--verbose`, and `--no-color` are shared output controls
  implemented in stage 8c. Neither quiet nor verbose changes validation or
  publication behavior.
- `--yes` only answers actual interactive setup questions; it does not grant
  overwrite authority or substitute for required settings. Existing overwrite
  policies remain command-specific and documented.

## 5. Representative workflows

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

These examples work with published candidate 0.2.0-rc.1. Generated CI uses its
own version when no explicit pin is saved; stable 0.1.3 predates these capabilities.

## 6. Implementation sequence and acceptance

The sequence below records the implementation plan. Runtime prerequisites,
configured sites, shortcuts, image packaging, Netlify delivery, CI generation,
formatting and maintenance commands are now implemented in source. Portable
`dev`, hosted `view`, `--cwd` and shared `--site-root` are also implemented.
Output controls are implemented too. Remaining work includes additional
providers, broader conversion fidelity, refreshed candidate acceptance and
matched stable release/documentation publication. Advanced build overrides and
combined image manifests have since been implemented; see the dated evidence. Dated stage records
below give each milestone's actual scope and limitations.

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

### Stage 8b — explicit configuration files (2026-10-18)

Global --config selects an existing YAML file and its parent site root. It works
before/after commands and through shortcuts. Repeated flags or combinations with
--site-root, positional site arguments, authoring --dir, CI --content, --with-docs
or explicit image-run tags fail before effects. Unsupported command families
reject it using the shared site-selector rules. --cwd resolves its relative path.

Versioned configuration retains content/ and default site/ layout; legacy flat
portable YAML retains its flat content and sibling output. Saved build.out is
relative to the selected config's parent. The file must be regular, and missing,
invalid, linked or replaced configs never fall back to zel.yaml. The shared
SiteInput retains directory-versus-explicit-file identity across renderers,
preview reloads, validation, images, delivery and authoring. No config copying,
renaming, environment override or merging with a default file is involved.

Formatting/features update the selected file; named configs do not replace the
editable wrapper's zellij.config.yaml format. Portable check JSON identifies the
selected filename. Watch tracks that file even when hidden, preserves previous
output on errors and resumes after restoration. Configured sites ignore other
root configs; legacy flat watching retains its existing whole-content scope.

CI serializes the repository-relative config path as data and passes --config to
validation, build and delivery. Its site-folder filter covers the file. Generated
workflows require a compatible pinned release; generation still does not verify
that release's availability. Other output controls/providers and matched release
and documentation publication remain unfinished.


### Stage 8c — shared output controls (2026-10-19)

All command families and shortcuts accept --quiet/-q, --verbose, --no-color
before/after the command. Quiet and verbose conflict across subcommands before
side effects. No -v option is introduced; v remains the view command alias.
A nonempty NO_COLOR also disables styling, overriding force-color environment
settings. Parser help/errors honor this before dispatch, including selector
normalization errors. Completions expose the same registered options.

Routine progress/success messages honor quiet across scaffolding, native/hosted
builds, previews and their worker, formatting, images, CI, extraction and delivery.
Warnings/errors, conversion limitations and requested data remain visible. Data
includes inspection/doctor/version/upgrade reports, context, section YAML,
Dockerfiles, completions, dry-run plans, formatting findings, page-registration
snippets, JSON, and the complete hosted check report (mixed diagnostics/summary).
Quiet never skips execution, validation, browser launch, watch, cleanup or publish.

Verbose diagnostics use stderr: timing, selected site input, runtime discovery
and child starts. Raw command arguments/environment are never dumped. Ordinary
text stream placement is retained for compatibility; JSON stdout remains clean.
Portable check JSON stays one report without stderr unless verbose was requested.
There is still no structured event stream for long-running commands.

No-color filters human output and streamed child logs, and sends color preferences
to clients and run containers. Generated data is not rewritten. Quiet suppresses
successful subprocess logs, replaying the final 64 KiB per stream on stderr on
failure (with truncation notice). Quiet/no-color child logs spool to temporary
files with streaming reads for no-color; this avoids pipe deadlocks, does not wait
for descendant-held pipe handles, and bounds failure replay memory. Temp logs
consume disk until command completion and are removed afterward. External logs
remain unstructured; successful tool warnings cannot be separated from progress.
Hosted validation therefore preserves its full captured report in quiet mode.

Shared immutable presentation settings are initialized once by the CLI before
workers start; library callers retain the previous default without initialization.
Output flags do not affect safety rules or environment selection. Additional
providers, remaining advanced/converter scope and release/doc publication remain.


## Stage 9a: single-platform container images (2026-10-19)

Implemented `--platform linux/amd64|linux/arm64` on image static/next/build,
si/ni, build/b --target image, saved publish/pub/deploy, ship --image, and CI
image generation. Optional version-1 `image.platform` supplies the default;
explicit CLI wins, omission retains runtime defaults. `image.name` is optional
for local builds, but publication still requires an explicit destination.
Legacy image build/tag behavior remains unchanged. No platform flag applies
to run/push: those operate on an already built image. Invalid values/lists and
irrelevant HTML/Netlify flags fail before build/write/publication.

Image builds target all stages. Docker and nerdctl retain plain progress;
Podman omits the unsupported Docker progress option. nerdctl push uses
--all-platforms so a foreign single-platform image is not host-filtered.
Build errors retain diagnostics and explain base-image/emulation prerequisites.
No emulator/daemon starts locally. CI freezes the selection in both jobs and
selects native ubuntu-24.04-arm runners for ARM64 to avoid known emulated Node
failures; AMD64/default workflows use ubuntu-latest. Generated
workflows remain reviewable local artifacts with existing event/credential gates.

Combined multi-architecture manifests are deliberately not implemented: Docker
image-store capability, Podman manifest build/push behavior, and nerdctl index
publication need a separate tested contract. Other architectures/Windows
container images are outside the supported Linux distribution targets. This
is independent of native zel releases for Linux/macOS/Windows x64/ARM64.

Verification covers aliases, every renderer/image kind, saved defaults/overrides,
delivery ordering and no-push-on-failure, all runtime argv, dry plans, retained
ship configuration, and CI generation. CI adds a real ARM64 portable-image
build/push/pull/HTTP round trip plus an AMD64 override. Full cross-architecture
hosted/Next emulated builds are not claimed by that small acceptance fixture.


## Stage 9b: per-build rendering features (2026-11-02)

Implemented --features maths|diagrams|maths,diagrams|all|none for build/b
(all three targets), image static/next/build (including si/ni), and hosted
Dockerfile generation. Values are typed complete selections, not arbitrary
Dockerfile/shell strings; duplicate/unknown features, empty lists and all/none
combinations fail during parsing. Mindmaps remain independent.

Portable compilation and adaptation apply overrides after validating the
selected configuration, without writing it. Hosted exports and image recipes
require feature-selection contract version 1 before selecting the disposable
wrapper's feature manifest and running npm install. Older distributions fail
with guidance without replacing an existing export. Explicit
none is a real operation even on an enabled base. A portable Next build matches
its inferred requirements explicitly, including none when no optional features
are needed. Omission otherwise preserves existing defaults. Authored files and
failed-export preservation are unchanged; local images never publish implicitly.
Portable static Dockerfiles consume already-built HTML and reject the override.

Preview/dev, delivery/ship and generated CI do not accept this invocation-only
flag in this increment. Persistent portable settings and hosted image variants
remain their defaults. Use build/image then an explicit push when a temporary
image override is wanted. Generic --build-arg, site-url/no-check, combined image
manifests, additional providers/conversion work and compatible release/docs
publication remain on the backlog.

Regression coverage includes named configs, source immutability, enable/disable,
invalid values, output preservation, adapted document trees, recipe/install
ordering and no-runtime Dockerfile generation. Container CI exercises hosted
exports with all -> none on a shared cache, static images overriding an enabled
base, and Next images enabling features from the default base, with browser
checks for rendered math and Mermaid.


## Stage 9c: rendering features in preview sessions (2026-11-02)

view/v and dev accept the same typed --features selection as build. It applies
to the initial build and every rebuild for the lifetime of the session, including
portable configuration edits. No authored configuration, content or wrapper files
are modified. A later invocation without the flag resumes saved portable settings
or the hosted engine image's default. Restart to change the session selection.
serve/s continues serving existing output and does not accept rendering flags.
Delivery/ship and generated CI still use their saved settings or engine variants.

Portable dev uses the native watch/preview path; hosted dev selects the disposable
wrapper's features before npm install and Next startup. Hosted view applies the
override to every staged export, retaining prefix pinning, error recovery and
cancellation protections. Both paths require the existing engine feature-selection
contract before honoring an explicit selection. Content validation still runs.
Hosted dev distinguishes invalid content from dependency/startup failures.

Hosted exports and development sessions namespace their dependency volumes by
the canonical explicit feature selection as well as the base image identity.
Equivalent all and maths,diagrams values share a cache; none and omission do not.
This prevents one selection from uninstalling dependencies used by a concurrent
development session with another selection. Existing no-override cache behavior
is retained; the CLI does not delete old caches or start a local runtime.

Regression coverage includes malformed flags, absent runtimes, installation/startup
ordering and diagnostics, cache isolation, native browser rendering through config
edits and failed-build recovery, and hosted preview orchestration on all three
OS families. Actual container CI checks enabled math/Mermaid during hosted preview
and development edits, then disables both for a subsequent preview.


## Stage 9d: rendering features in delivery and CI (2026-11-02)

publish/pub, deploy/netlify and ship accept the shared typed --features
selection. It is complete, invocation-only, displayed in dry plans, and passed
to the single portable/hosted static or image build before publication. Saved
configuration, authored content and previous local exports remain unchanged.
Malformed values fail during argument parsing before writes or tool/provider
access. Omission preserves defaults; explicit none disables both optional
renderers. Existing validation, compatibility guards, runtime checks and
no-publication-after-build-failure behavior remain in force.

Ship saves its existing destination/platform settings but does not save the
feature override. Its success/failure messages explain how to repeat the
selection on a later publish. No feature flag applies to manual image push,
which publishes an already-built artifact.

ci and ci init github accept --features for generation. Both verification and
publication builds receive the canonical explicit selection, including build-only,
Netlify, static/Next image and legacy image workflows, plus named configuration
and platform combinations. Check and prebuilt image-push commands do not receive
it. Selection is frozen in the generated YAML until regenerated; regenerating
without it restores saved/default behavior. ci run/publish dispatch does not
accept rendering options. No new settings keys or implicit credential provisioning
are introduced. Existing compatible-release requirements still apply.

Verification covers generated commands executed against the native portable
renderer, all workflow delivery forms, malformed inputs/output preservation,
portable upload bytes, each renderer/image kind, ship retention and build/push
ordering. Hosted CI exercises selected features in Netlify exports, portable
Next image publication and imported static/Next images. Remaining work includes
advanced build options, combined image manifests, additional providers,
conversion fidelity and matched native release/documentation publication.


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


### Image arguments — first CI correction, 2026-11-04

[PR #67](https://github.com/mylife-inc/Zellij/pull/67) tracks implementation
`86b6d2d` and context `bcb0401`. Run `41880580238` failed the new Windows native
fixture during the fake Docker info probe, before building. Rust on Windows
searches the executable and system directories before inherited PATH; setting
only the fixture PATH does not isolate an installed system Docker. Correction
`45f44bf` copies zel beside its fake clients and verifies each selected probe
writes the fixture marker. Production runtime discovery is unchanged. Corrected
native acceptance, JavaScript syntax and diff checks pass locally; final Windows
and real-container acceptance remain pending on the replacement head.

The earlier main CI `41826282286` now passed on attempt 2: Windows ARM packaging
job `126744911367` succeeded without source changes after the isolated PowerShell
CLR crash. This is retry recovery, not a diagnosed product fix. No release was
published and no production site changed.


### Combined image publication and smaller runtimes — 2026-11-04

The user authorized the next CLI milestone and requested smaller images.
Implementation `a29a792121ce0cb3fa6ba027e243173d6e0a40e7` adds `--platform all`
(and either ordering of the explicit AMD64/ARM64 comma list) to registry
publication and ship. `image.platform: all` saves the default. Single-platform
local commands remain compatible; combined selection on a local-only image
command fails before runtime discovery with publication/override guidance.
Netlify still rejects explicit image platform flags.

Both renderers and static/Next image kinds share the same recipe/argument paths.
Docker uses the caller's selected Buildx builder and direct registry output;
Podman uses an invocation-owned lowercase manifest, pushes all architectures,
and cleans the manifest after success/failure; nerdctl uses its image exporter.
No builder, emulator or daemon is installed/started. Combined publication does
not promise a local image; failure guidance acknowledges possible remote layers
and recommends inspecting the registry before retrying. Ship retains its
converted site and saved architecture choice after delivery failure.

Generated combined CI verifies both architectures without credentials on PRs.
Publication events use native AMD64/ARM64 runners, unique per-run/attempt staging
tags, and a dependent final job joining immutable digests only after both jobs
succeed. Staging tags remain for registry retention policies. Existing event
restrictions, permissions, explicit configuration selectors and build overrides
are retained. Minimum CLI pin is 0.2.0-rc.2 (still unpublished); single-platform
workflows, including all six release acceptance files, are unchanged.

Static images use upstream NGINX Alpine slim without optional dynamic modules.
Next images trim npm, Yarn and Node headers in a separate stage and copy that
filesystem into a fresh runtime, so removed tools do not remain in inherited
layers. Node's matching Alpine libraries, certificates, licenses and shell are
kept, alongside traced app dependencies, public assets and runtime content.
New CI jobs build/run both image kinds on both native architectures, replay the
generated publication/manifest commands against loopback registries, and compare
uncompressed image size against the old runtime bases with identical site data.
The existing container job also tests direct combined Buildx static publication,
both architecture pulls/HTTP serving, and the existing Next browser suite.

Local verification: 329 Rust tests passed, one real-container test ignored;
warnings-denied Clippy, formatting, isolated native runtime acceptance, existing
image-argument acceptance, all six generated release-workflow comparisons, and
12 combined generated workflows through actionlint passed. Docs deep/strict
checks passed all 55 routes and 23 examples. An initial parallel Rust run hit
three known local fake-shell info-probe timeouts; the full serial run passed.
No local Docker or browser was used. Actual image sizes and final CI acceptance
remain pending for this branch; do not claim measured savings or a merge yet.
Remaining CLI areas include additional providers and broader import fidelity;
release publication and Netlify production migration remain separate decisions.

Prior milestone evidence: PR #67 merged as `557d8b6` after all 13 checks passed
in run `41882538968` attempt 2 on `ba37a8a`; merge/head trees match. The first
attempt's existing Docker info timeout recovered on retry without a code change.
Main base-image run `41889745285` passed; main CI `41889745157` was still running
at this entry. The Netlify hosting-comment hydration issue remains external.


### Combined publication — first CI fixture correction, 2026-11-04

[PR #68](https://github.com/mylife-inc/Zellij/pull/68) tracks this milestone.
Compatibility correction `c215266` preserves Node's original container entrypoint
in the trimmed runtime; all 25 engine tests passed afterward. Run `41893692905`
on `ec35452` built/pushed/pulled static and Next images on both native Linux
architectures. Static serving and size comparison passed. The new Next acceptance
fixture then requested `/index.html`, but the portable-to-hosted adapter maps the
home page to `/` (confirmed in source and the generated route table). The server
started successfully; the fixture's incorrect route caused both failures.
The fixture now requests the renderer's actual home and reports HTTP status on
failure. Full container acceptance, final size figures and merge remain pending.


### Combined publication — native acceptance and size accounting, 2026-11-04

Run `41894634123` on `eec6ca1` passed both native image jobs and the dependent
combined-manifest job (`126803607170`, `126803607619`, `126805836087`). Both static
and Next images build, push, pull, serve both documents, deny Markdown-source
access and pass health checks on AMD64 and ARM64. The generated final command
joins the exact platform digests. Node/npm/Yarn/header checks also pass.

The first size report used Docker's Size field and called it uncompressed;
that label is not portable between classic and containerd image stores. The
runner-local comparison showed reductions, but raw AMD64 and ARM64 counts must
not be compared directly. Acceptance now also pushes the identical-payload
baseline to the disposable loopback registry and sums compressed layer sizes
from its platform manifest. This supplies consistent download-size comparisons
and keeps Docker-reported sizes explicitly labeled. Final compressed figures
and full final-head CI remain pending. No live external registry was used.

Prior main CI `41889745157` is now successful. Superseded PR run `41893692905`
required force-cancellation after its macOS preview job did not stop following a
normal cancellation request; this unblocked the corrected run, not a product fix.


### Combined publication — final runtime footprint pass, 2026-11-04

Read-only inspection of the tested AMD64 image archive confirmed that most Next
runtime space is Node itself and traced app dependencies, with negligible source
maps. It also exposed retained Alpine package-index caches. The final recipes
remove those caches. Static packaging additionally removes the unused alternate
NGINX debug executable and copies the prepared filesystem into a fresh stage so
upstream deleted/cache layers do not inflate downloads. NGINX's entrypoint,
non-root user and SIGQUIT shutdown signal are explicitly retained. Node's
entrypoint remains intact. Required runtime libraries and traced app dependencies
are not pruned. Native CI verifies cache absence, NGINX startup metadata and the
existing serving/health behavior, and measures compressed registry layers against
the former runtime bases. No local container or browser was started.


### Import heading anchors and completed image milestone — 2026-11-04

PR #68 merged as `e7b0d7987787ee4e0856bb3d3edb280eaec0c9ae` after all
16 checks passed in run `41897666269` on `f5c0a5c`; merge and tested trees match.
Both native image jobs and the generated combined-manifest job passed, as did
direct combined Buildx publication, native package/install checks on six targets,
portable acceptance on three OS families, Next browser acceptance and the full
CLI/container suite. Base image post-merge run `41902611084` passed; main CI
`41902611421` had 14 of 16 successful jobs when this record was written.
The final compressed-layer measurements are in
`specs/evidence/zellij-image-publication-2026-11-04.json`: static images decreased
from 27.1–27.5 MB to 6.6–7.0 MB (74.7–75.6%), and Next images from about 84.3 MB
to 75.8–75.9 MB (10%). These compare identical fixture payloads and exclude small
manifests/configuration and attestations; they are not universal site sizes.

Source `d5b6fac` addresses another portion of audit R5: imports now preserve
explicit document-level ATX/setext heading IDs in `{#id}` notation and Docusaurus
MDX/HTML comment forms. The converter recognizes headings with Markdown parser
offsets, preserves inline formatting/rewritten links, and emits literal MDX
`h1`–`h6` elements. A leading H1 becomes a literal anchor below the layout's
single title. Title extraction ignores code examples and removes ID syntax.
Duplicate/unsupported IDs and nested identified headings fail before output
writes; the error identifies the source document. Classes and other heading
attributes are not carried over. Automatic source/plugin slugs and implicit
leading-H1 anchors still need review; this does not close all of R5.

The engine's shared heading reader now indexes literal JSX heading IDs for TOC
and search without executing expressions or assuming spread/dynamic/component
IDs. Existing deep validation checks the actual literal IDs, including collisions
with generated anchors. No new runtime dependency was added.

A new regression exposed the old fence scanner treating shorter or different
fences inside examples as closing fences. Shared marker/width tracking fixes
that in both Material demotion and MDX escaping. Balanced markup spanning a
replaced heading still uses the complete document as its recognition context.

Local validation: 56 converter unit tests and 29 CLI conversion/import/write
safety tests passed; 54 engine heading/TOC/search/deep-check tests passed.
Warnings-denied workspace Clippy, formatting, engine typechecking/packaging,
and the docs deep/strict check (55 routes, 23 examples) passed. The real native
CLI-to-engine fixture passed for MkDocs and Docusaurus: explicit title IDs,
formatted and linked headings, Unicode IDs, reference URLs, unique rendered IDs,
search/TOC targets, strict links and asset bytes. Its existing hosted CI step
also checks full static exports, one page title and rendered TOC links.
No local container or browser ran. Final-head CI and merge are pending for
this import milestone; source functionality remains unpublished rc.2.

Other providers, broader source-format fidelity and release/production migration
remain separate work. Netlify's hosting-comment hydration issue remains external;
the old production site/domain is retained. No new release was published.


### Imported metadata and Docusaurus document IDs — 2026-11-04

PR #69 merged as `b1ff5081013a89b8c98ed95715d8bc114c8a1fe5` after all
16 checks passed in run `41905598125` on `fb02026`; the merge and tested trees
match (`fda6beec21e54a0813a023a2ee256b155277e91c`). Its post-merge base-image
run `41910675985` passed; main CI `41910675972` was still running when this
record was written. Both post-merge runs for PR #68 (`41902611421` and
`41902611084`) finished successfully.

Source `9e296cc` addresses another portion of audit R5. Both importers now use
an authored YAML title and description as active page metadata, retaining the
original metadata in comments. Title precedence is front matter, document H1,
sidebar label, then source path without extension. Sidebar labels are emitted
independently using the engine's existing page/label entries. The shared inert
YAML reader handles BOMs, CRLF and `...` delimiters, reports malformed metadata
with its source path, and validates the types of supported fields. Retained
comments also handle Unicode YAML line separators so their text cannot become
active output fields. No runtime dependency or engine schema was added.

Docusaurus imports inventory the actual `.md`/`.mdx` sources and resolve custom
IDs using the default numeric-prefix rules and `parse_number_prefixes: false`.
A custom ID replaces the basename only. `sidebar_label` overrides a sidebar
item label. Legacy filename IDs remain accepted when unambiguous; conflicts
between canonical IDs and legacy names, duplicate IDs, missing sidebar targets
and ambiguous `.md`/`.mdx` stems fail before writes. Routes and relative links
still derive from physical source paths; custom `slug` values, plugin paths and
custom JavaScript prefix parsers are not applied. Plain `sidebars.json` is now
parsed directly. The inventory rejects symlinks/junctions before reads and
normalizes native path separators for portable lookup.

Draft/unlisted documents are excluded and reported, including explicit sidebar
entries and folder-derived navigation. An explicit empty sidebar stays empty.
Unsupported autogenerated branches leave omitted documents in the report;
they no longer cause all documents to be promoted when no supported entries
remain. Empty navigation and groups emit YAML arrays rather than null values.
Dynamic/missing sidebars retain the documented alphabetical fallback, with
exclusions applied. This remains a partial fidelity improvement, not full
Docusaurus execution or closure of R5.

Local verification: 60 converter unit tests and 37 CLI metadata/reference/
import/write-safety tests passed. Warnings-denied workspace Clippy, formatting
and docs deep/strict checks passed (55 routes, 23 examples). Both real CLI-to-
engine fixtures passed for metadata, distinct sidebar labels, custom IDs,
physical routes, rendered MDX, heading/search/TOC targets and asset bytes. The
metadata CLI suite is now in Linux/macOS/Windows CI; the existing full-export
fixture additionally checks page metadata and sidebar labels in generated HTML.
No local container or browser ran. Final-head hosted CI and merge are pending.

This remains unpublished rc.2 functionality. Release preparation must be
refreshed before any publication, which still needs explicit approval following
the earlier automatic approval rejection. Netlify's hosting-comment issue,
production cutover, other deployment/CI providers and broader import fidelity
remain separate work. The existing production site/domain remains intact.


Follow-up review fix `0fb7e14`: plain JSON sidebar strings are always parsed as
inert data, so labels containing ellipses, backticks or `require()` do not
trigger JavaScript fallback and accidentally include omitted files. All nine
metadata CLI fixtures pass locally after this correction (38 CLI tests across
the milestone's four suites, plus 60 converter tests). PR #70 hosts the final
native/full-export acceptance; CI on the corrected head is pending.


Native CI follow-up: run `41913945831` on `1bc4b1f` passed eight metadata
fixtures on Windows but failed while creating the ninth fixture's junction,
before invoking the importer. `d3db17c` builds that path with native Windows
separators, matching the existing passing junction fixtures, logs setup errors,
and removes the junction before cleanup. All nine fixtures pass locally after
the correction; final-head Windows and full acceptance must still pass.
