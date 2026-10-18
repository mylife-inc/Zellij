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
