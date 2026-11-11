# CLI completion gates — 2026-11-11

The CLI is **not yet fully verified for completion**. This page separates
implemented command families, remaining acceptance work, and format boundaries.
It supplements the [current status](zel-cli-status-2026-11-05.md), which retains
the feature history. Refresh CI and account state before relying on this snapshot.

The user requested continuing until completion. The working target is the
[agreed command contract](zel-cli-contract-2026-10-18.md), including conservative
declarative importers. The original design explicitly declines computed source
configuration rather than executing another project's code. A scope question
about full source-generator/plugin/theme parity is pending; that larger scope
has not been assumed or silently marked complete.

## Implemented surface and verification

The parser, exhaustive dispatch and current command reference have handlers for
every registered command family. Adding a parsed variant without dispatch fails
compilation. A handler's existence alone does not prove every real-provider case.

| Family | Current evidence | Outstanding gate or boundary |
| --- | --- | --- |
| Creation and authoring | Site/app/page/section wizard tests; full starter; all 23 section templates; configured layouts and aliases | Existing authored files are protected; page registration remains a documented printed entry. |
| Inspection and maintenance | Inspection JSON, doctor, version, completions, runtime discovery, native installer/self-upgrade, eject/features acceptance | JSON is available only on documented commands; dependencies are not installed automatically. |
| Build/check/fmt/watch/dev/serve/view | Portable native and hosted engine tests; direct-file browser acceptance; watch/reload and failure recovery | Hosted rendering requires the documented container/engine prerequisites. |
| Static and Next images | Both Linux architectures, combined manifests, registry publication, fresh HTTP/browser checks | Images target Linux; the CLI itself ships on all six native OS/architecture targets. |
| Imports/migrate/ship | Six format fixtures through the real CLI and engine; source containment, assets and configured delivery tests | Supported source subsets are explicit; automatic full format/plugin parity is not implemented. |
| Deployment | Netlify, Cloudflare, Vercel, GitHub Pages, S3, Fly and Railway adapters and protocol tests | Live Fly/S3 acceptance pending; Vercel deferred by the user. |
| CI generation | GitHub/GitLab/CircleCI generator tests and installed-CLI delivery runs | Actual GitLab/CircleCI runner acceptance pending; provisioning/dispatch remain GitHub-specific. |
| Distribution | Published rc.2: all 29 release jobs, six native packages and image variants passed | Current fixes/features are newer than rc.2 and need a separately approved release after final verification. |

Latest implementation: Sphinx inclusion patterns, source `47b3293`, in
[PR #123](https://github.com/mylife-inc/Zellij/pull/123). All 720 local Rust
tests passed (five hosted cases run in CI), including 64 Sphinx CLI cases.
Clippy, formatting, all six real-CLI/engine conversion checks, and strict/deep
docs checks passed. Cross-platform run `42998033906` passed all 16 jobs. PR #123
merged as `872db136bd7cf10e3722b9c05d180881a7e4d799`; its tree matches the tested
head (`f465fe23f4950675e734bfcae35ba6b499af8417`). Post-merge CI `43002975098`
passed all 16 jobs and Base image `43002974973` passed all 13 jobs, without reruns.

The Fly/S3 acceptance harnesses are in
[PR #124](https://github.com/mylife-inc/Zellij/pull/124), head `5230d48`, with all
70 local helper tests and all 16 jobs in run `43003339641` passing. It merged as
`af98abc356a09f964bf249b0830a62ded01613ed` with the tested tree unchanged.
Both manual workflows are registered and active on GitHub. Duplicate merge-push
builds were skipped for these test/workflow/docs-only changes; no runtime code
changed. This preparation does not establish live provider success.

Active documentation was reconciled in `e3a78b4`: 14 guide/reference pages now
use the configured layout, current commands and accurate release boundaries.
Native scaffold examples, strict/deep checks (55 routes / 23 examples), and
streaming MDX renders with all 89 existing heading IDs on edited pages pass.
[PR #125](https://github.com/mylife-inc/Zellij/pull/125) passed all 16 jobs in
run `43005542302`, including built-site browser preservation. It merged as
`6f8abd337afde39bed8f3fd38a29f9deb61a0a34`, with tree
`e11d852fc1e593859f350dd94d195925c25799e6` matching the tested head exactly.
Duplicate merge-push builds were skipped for this documentation-only revision.
Historical audits were not rewritten. The newest published candidate was
refreshed after the merge and remains `v0.2.0-rc.2`.

## Remaining work in execution order

1. Use the merged Fly disposable acceptance workflow with
   an eligible temporary trial organization, run static/Next live acceptance,
   inspect the evidence and confirm app cleanup. The lifecycle harness is
   locally tested; no live account success is claimed.
2. Run the merged S3 acceptance workflow in an
   explicitly authorized test account. Ten lifecycle tests pass locally; live
   portable/hosted first and replacement uploads remain unverified. Confirm
   account eligibility/cost constraints before provisioning, and verify cleanup.
3. Run the actual generated GitLab and CircleCI workflows in temporary projects
   on eligible accounts/runners, including event gates and native image/manifest
   artifact handling, then remove the test resources. Four build-only fixture
   directories are prepared; their generation is not live runner acceptance.
4. Reconcile any failures and repeat only the affected checks. Refresh the
   completion record, docs and current-source package/image acceptance.
5. Prepare the next release from the verified source. Obtain publication approval
   for that concrete release; approval for rc.2 does not publish another version.

Browser automation currently fails during initialization because a Codex writable
root traverses the `~/.codex` symlink. No provider signup occurred in the retry.
Account-dependent acceptance therefore needs working browser access or the
user's account-side setup. See [the external runbook](zel-external-acceptance-2026-11-10.md).

## Explicitly open or deferred scope

- **Vercel live acceptance:** user-deferred after trial signup required payment.
  Preserve that decision; do not resume account setup without direction.
- **Import fidelity:** VitePress rewrites/locales/exclusions and custom Vue;
  Jekyll posts/dated collections/globs/theme-specific navigation; Hugo menus,
  split config/modules/locales/cascades; Sphinx numbering/MyST/domains/extensions
  and broader reStructuredText remain outside current supported subsets.
  These limits are disclosed, not counted as implemented parity. The pending
  scope answer determines whether further parity belongs to this completion goal.
- **Full documentation hosting:** Netlify's inserted-comment hydration problem
  still blocks production migration. Small CLI deployment-fixture success does
  not close it. Preserve the old production site and domain.
- **Zero bugs:** passing regression and acceptance gates is evidence of tested
  behavior, not proof that no undiscovered defect exists.
