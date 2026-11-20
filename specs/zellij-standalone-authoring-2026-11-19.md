# Standalone rich authoring

Status: implementation on `codex/standalone-authoring`, preparing **rc.14**.
The published rc.13 already builds rich double-click websites without external
tools. This change extends its cached renderer to validation and preview.
No rc.14 release or deployment is implied by this document.

## Command contract

For rich YAML/MDX sites, `check`, `watch`, `view` and `view --watch` default to
`file`. The first invocation downloads the matching renderer into
`$ZELLIJ_HOME/cache/engines` (default `~/Dev/config/zellij/cache/engines`).
`--offline` requires that exact verified cache. Only `zel` must be installed;
the private runtime belongs to the renderer and is never installed globally.
Portable commands remain native. Explicit `--target static` retains the
existing container checker/Next export. `--offline` rejects other targets.

Validation reuses the shared schema/deep checker without evaluating authored
MDX, copying assets or creating an export. JSON mode emits one versioned report,
including first-download and runtime failures, without progress contamination.
It identifies the public target as `file`, reports native source paths, and
preserves the checker's actual warnings, errors, strict policy and exit codes.
The private request/report directory is removed on completion. A successful
content check does not guarantee a successful render or publication.

`watch` validates once, then after debounced source edits. Invalid content stays
watched; missing prerequisites fail at startup. A native preview reserves its
port before building, stages each rich export, and publishes only a successful
build whose input snapshot is still current. Invalid edits keep the last site
available. Successful publication reloads open browsers. The reload client is
injected into HTTP responses only: exported HTML remains usable through file://.
The initial basePath, output, port and renderer stay fixed until restart.
Ctrl-C cancels/reaps a running standalone renderer before cleaning its stage;
it does not publish an interrupted build. Cache download/verification is
synchronous and can finish before cancellation is observed.

## Scope retained elsewhere

Hosted `dev`, explicit static exports, image packaging, deployment exports and
PDF typesetting retain their existing prerequisites. `convert --validate` and
`migrate` retain the container checker for hosted imports. Generated delivery CI
selects static or portable validation explicitly to match its deployment engine.
This change does not migrate private client data or exercise external providers.

## Implementation and verification

- `offline/standalone.ts` adds a nonexecuting check operation with a report
  callback into the shared checker, plus a temporary basePath build receipt.
- `rich_build.rs` owns cache selection, cancellable private processes, bounded
  reports and prepared-output publication. `check_report.rs` verifies the
  shared report's root, flags, schema, findings and exit status before exposing it.
- `preview/rich.rs` shares the native HTTP server and reload/watch machinery.
  The existing explicit container preview remains available.
- `check-standalone-authoring.mjs` exercises real check, strict/deep diagnostics,
  uncached offline failure, dynamic MDX nonexecution, validation recovery,
  preview/browser opening, successful and failed rebuilds, prefix changes and
  Ctrl-C with no external tools on the CLI's PATH.
- Native package/public-release acceptance runs that driver on six platforms.
  The source Linux browser job additionally tests Chromium and WebKit reloads.

Local tests and final hosted results must be recorded before claiming release
readiness. Retain failures and their corrections as verification history.

## Local acceptance checkpoint

The implementation passes 825 Rust tests (five existing hosted-only ignores),
540 engine tests, workspace typechecks, Clippy/format, schema drift, workflow
lint and strict/deep validation of all 59 documentation routes. Both full and
production npm audits report zero advisories at this checkpoint.

The actual Mac Intel CLI/package test passes with an empty PATH: first-download
JSON checking, checksum rejection, cached offline builds, all 25 section types
through the 28-route full starter, traversal rejection, cache tampering and
failed-output retention. The new authoring driver passes check/watch/view,
nonexecuting dynamic MDX checks, strict warnings, invalid configuration recovery,
invalid-edit preservation, basePath pinning and Ctrl-C. Chromium and WebKit both
pass HTTP live-reload and recovery with no captured page errors.

Initial harness runs exposed an incorrect fixture configuration (`port` belongs
under `preview`) and a missing local Playwright Chromium binary. The fixture is
corrected; macOS uses installed Chrome as in the existing file-browser test.
The final complete driver passes. These were harness failures, not hidden passes.

The five rich starters receive 0.1.2 identities/minimum CLI rc.14 because their
README guidance changes. Their authored website content/artwork is unchanged;
existing published 0.1.1 archives are not rewritten. Native packaging and full
hosted CI remain required before merging/releasing.

Review reproduced a legacy preview defect: `zel view .` lost the literal-dot
discovery rule when Site normalized its input, treating the repository root as
content and rejecting its sibling output as overlap. Fix `e33c0f670e64fcb8ec11af1355cb39ca427b4059`
pins the actually discovered content folder for preview and validation watch.
It also honors explicit static preview dispatch for unconfigured inputs. Native
regressions pass; the real authoring driver now additionally checks legacy-dot
preview/watch and ignores sibling output/unrelated repository edits. Final
expanded package acceptance is in progress.
