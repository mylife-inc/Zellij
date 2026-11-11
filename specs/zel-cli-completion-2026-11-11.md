# CLI completion gates — 2026-11-11

The **documented CLI command surface is implemented**. Live deployment-provider
and external-runner acceptance remains incomplete. The user explicitly moved
deployment verification to the end and authorized cutting a CLI release with
updated documentation on 2026-11-11. This page separates implementation,
release checks, deferred acceptance work, and format boundaries.
**v0.2.0-rc.3 is published**, including all post-rc.2 implementation changes.
All 16 final-head CI checks and all 29 release jobs passed. See the
[release evidence](evidence/zellij-rc3-release-2026-11-11.json).
It supplements the [current status](zel-cli-status-2026-11-05.md), which retains
the feature history. Refresh CI and account state before relying on this snapshot.

The user subsequently requested new branding, composable sections, site/book
templates, PDF publishing and installation channels. These extend the earlier
contract and remain a separate [expansion backlog](zellij-product-expansion-2026-11-11.md).
The previous completion statement does not claim these new features are done.
Independent company/product branding is now implemented by PR #131
(`b4cc9ba`), with all 16 CI jobs passing on `7d7febbe`; see
[branding evidence](evidence/zellij-company-branding-2026-11-11.json). The four
active guides were updated. This increment and the hydration fix below are newer
than rc.3 and await a later release. The template catalog, composer/new sections,
book/PDF publishing and package-manager registration remain planned.
The user's follow-up specifies opt-in build-time PDFs with a first-class LaTeX
backend, short Homebrew naming and removal of superseded pre-1.0 distributions.
See the expansion design and [release retention policy](zellij-release-retention-2026-11-11.md).
These decisions are recorded; PDF execution and retirement automation are not
implemented by this documentation update.

The user requested continuing until completion. The working target is the
[agreed command contract](zel-cli-contract-2026-10-18.md), including conservative
declarative importers. The original design explicitly declines computed source
configuration rather than executing another project's code. Full arbitrary
source-generator/plugin/theme parity is outside this release's documented
contract; unsupported cases remain disclosed rather than marked complete.

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
| Distribution | Published rc.3: all 29 release jobs; six native platforms and four engine variants on both Linux architectures passed | Live provider/external-runner acceptance remains deferred. |

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
refreshed after PR #125's merge was `v0.2.0-rc.2`; the rc.3 release below supersedes it.

## Release milestone complete

[PR #127](https://github.com/mylife-inc/Zellij/pull/127) merged as
`58be9b1ab992424576572ee8e2ba736f134f727c`, with tree
`52c2bdce26f4b3dbcc50b715be46d27a8b5ce56b` identical to tested head
`2b48747c15e813b26a98293a4474dbf8243bf281`. Run `43013070798` passed all 16
checks. The first run exposed a hardcoded rc.2 reference in the external-CI
replay fixture; `2b48747` makes it read the actual binary version. That original
failed/cancelled run remains recorded in preparation evidence.

The release helper passed 720 Rust tests, 453 engine tests, typechecks, schema
drift, formatting and warnings-denied Clippy before tagging. Release run
[`43018433622`](https://github.com/mylife-inc/Zellij/actions/runs/43018433622)
passed all 29 jobs on its first attempt. Both public release repositories have
matching sets of 15 assets. All six public native installs and all four engine
variants on native Linux AMD64/ARM64 passed. Stable installer updates and
production documentation deployment were skipped. The updated docs built.

No remaining live provider or external-runner tests were run for this release.
The six frozen delivery workflows retain their previous verified rc.2 pins;
retarget them when live delivery acceptance resumes.

## Remaining work in execution order

The implementation/release/docs milestone is complete. The user deferred the
following verification phase until the end; account setup is not required now.

1. Use the merged Fly workflow with
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

Browser automation currently fails during initialization because a Codex writable
root traverses the `~/.codex` symlink. No provider signup occurred in the retry.
Account-dependent acceptance therefore needs working browser access or the
user's account-side setup when that deferred phase starts. No account setup is
needed for the current release. See [the external runbook](zel-external-acceptance-2026-11-10.md).

## Explicitly open or deferred scope

- **Vercel live acceptance:** user-deferred after trial signup required payment.
  Preserve that decision; do not resume account setup without direction.
- **Import fidelity:** VitePress rewrites/locales/exclusions and custom Vue;
  Jekyll posts/dated collections/globs/theme-specific navigation; Hugo menus,
  split config/modules/locales/cascades; Sphinx numbering/MyST/domains/extensions
  and broader reStructuredText remain outside current supported subsets.
  These limits are disclosed, not counted as implemented parity. Broader
  source-generator parity would require a separate scope decision.
- **Full documentation hosting:** Netlify's inserted-comment hydration problem
  has a source fix in PR #129 (`360cb40`), with all 16 CI jobs passing and all
  55 routes tolerating the controlled mutation. See the
  [hydration report](zellij-netlify-hydration-2026-11-02.md). The fix is newer
  than rc.3 and has not been published as a new artifact. Live verification
  still blocks production migration. Small CLI deployment-fixture success does
  not close it. Preserve the old production site and domain.
- **Zero bugs:** passing regression and acceptance gates is evidence of tested
  behavior, not proof that no undiscovered defect exists.
