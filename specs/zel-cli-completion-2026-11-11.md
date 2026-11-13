# CLI completion gates — 2026-11-11

The **documented CLI command surface is implemented**. Live deployment-provider
and external-runner acceptance remains incomplete. The user explicitly moved
deployment verification to the end and authorized cutting a CLI release with
updated documentation on 2026-11-11. This page separates implementation,
release checks, deferred acceptance work, and format boundaries.
Refreshed 2026-11-13: **v0.2.0-rc.6 is published** from PR #155
(`668e8fa`), including composition/presets, the shared book model, optional
LuaLaTeX PDFs, configured preview/static delivery and book-classic 0.2.0.
All 23 source CI jobs passed. Source and mirror have 20 byte-identical assets;
the actual standalone rc.5-to-rc.6 upgrade passed. All eight public acceptance
jobs and all four public Homebrew install/upgrade/uninstall checks pass. See
[rc.6 release evidence](evidence/zellij-rc6-release-2026-11-13.json).
rc.5 retirement is complete after consumer replacement: two releases, 40 assets,
two remote tags and 26 obsolete image versions removed. Current images and
CodeSeal remain intact; fresh inventory and HTTP checks pass. See
[rc.6 retirement evidence](evidence/zellij-rc6-image-retirement-2026-11-13.json).
Earlier scoped retirement removed 213 obsolete image versions while preserving
current image graphs and CodeSeal; see
[retirement evidence](evidence/zellij-rc5-image-retirement-2026-11-12.json).
It supplements the [current status](zel-cli-status-2026-11-05.md), which retains
the feature history. Refresh CI and account state before relying on this snapshot.

Post-release source fix: PR #158 (`dd0f620`, tested at `bd0ae9b`) explicitly
selects/preflights Docker in generated container CI while preserving native
portable jobs. All 23 CI jobs and six consumer checks pass. The public rc.6
generator is unchanged; the source update awaits the next release. The desktop
header-resize failure is reproduced and resolved by PR #160 (`dece6c9`),
which also fixes drawer-link focus resets during header updates. No-retry stress
passes 252 targeted and 636 full-workload cases; all 23 CI jobs and all 13
development-image jobs pass. These header fixes also await the next candidate.
See [verification evidence](evidence/zellij-ci-runtime-selection-2026-11-13.json)
and [the focus investigation](zellij-header-resize-focus-2026-11-13.md).

`zel create` / `zel new site`, `zel templates [show]`, three bundled starters,
local/release catalogs, exact versions, checksums, offline cache and receipts
were first available in rc.5 and remain included in rc.6. See [the template contract](../templates/README.md).
PR #147 (`2373c5e`) adds `zel presets` / `zel list presets` and `zel ns --preset`
(`-p`) for four offline section compositions, plus a media-accordion section and
composition node. This expands the source catalog to 25 section types. All 20
final-head CI checks pass; the merged tree matches tested `d8c6b7c`. All 356
showcase cases pass without retries and six light/dark captures were reviewed.
All 13 development-image jobs pass, publishing all four variants on both Linux architectures.
These additions are included in rc.6. See
[the implementation record](zellij-media-composition-2026-11-12.md) and
[verification evidence](evidence/zellij-media-composition-2026-11-12.json).
PR #149 (`72a8445`) adds a versioned book manifest, complete reading order,
metadata/cover, shared publication JSON and content revision, language/direction,
and Next adaptation. All 20 final-head jobs pass, including 756 Rust and 490
engine tests, cross-platform book browser checks and a real Next-image book.
All 13 development-image jobs in `43328363366` pass on the merged source.
See [book evidence](evidence/zellij-book-publication-2026-11-12.json).
PR #151 (`c609251`) adds optional `build --pdf`, `export pdf` / `pdf` and native
`pdf --check`. Web and PDF share a revision and publish atomically; standalone
export preserves the website. All 23 final-head CI jobs pass, with 762 local
Rust tests and actual Docker amd64/arm64 plus rootless Podman PDF acceptance.
The single unchanged Engine retry followed a legacy-bookmark viewport timeout;
that first failure remains recorded. All 24 final page captures match the eight
visually reviewed pages. See [PDF evidence](evidence/zellij-book-pdf-2026-11-12.json).
PR #153 (`71b97e4`) adds saved PDF enablement, A5/A4/Letter and 10/11/12pt
settings, preview/watch, static delivery/images and compatible CI generation.
All 23 final-head jobs in `43376564097` pass without reruns; 767 local Rust
tests pass. Nine real PDFs and 66 captures pass across three Linux runtimes.
See [configuration/delivery evidence](evidence/zellij-pdf-configuration-2026-11-12.json).
PR #155 versions book-classic at 0.2.0 with the shared manuscript and optional
PDF. Twelve source PDFs and 87 page captures pass across three configured
Linux runtimes; the seven-page starter was visually reviewed. The print adapter
remains English/LTR; advanced publishing semantics and other expansion work
remain incomplete. Older milestones retain their dated evidence.

The user subsequently requested new branding, composable sections, site/book
templates, PDF publishing and installation channels. These extend the earlier
contract and remain a separate [expansion backlog](zellij-product-expansion-2026-11-11.md).
The previous completion statement does not claim these new features are done.
Independent company/product branding is now implemented by PR #131
(`b4cc9ba`), with all 16 CI jobs passing on `7d7febbe`; see
[branding evidence](evidence/zellij-company-branding-2026-11-11.json). The four
active guides were updated. This increment and the hydration fix below shipped
in rc.4. The first template catalog shipped in rc.5. Bounded composer v1,
media-accordion, the first four composition presets, the book publication model
and optional PDF/LaTeX pipeline ship in rc.6. Further composition primitives
and advanced book publishing remain planned.
Homebrew is live; Scoop, WinGet and other new package-manager
channels remain planned.
The user's follow-up specifies opt-in build-time PDFs with a first-class LaTeX
backend, short Homebrew naming and removal of superseded pre-1.0 distributions.
See the expansion design and [release retention policy](zellij-release-retention-2026-11-11.md).
The first PDF execution path is implemented; richer print formats remain open.
Scoped retirement followed rc.4 publication
and consumer migration; see the retention record for the exact removal scope.

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
| Creation and authoring | Versioned templates, guarded creation and public/offline catalog acceptance; site/app/page/section wizard tests; full starter; all 25 section types and four composition presets in rc.6; configured layouts and aliases | Existing authored files are protected; page registration remains a documented printed entry. |
| Inspection and maintenance | Inspection JSON, doctor, version, completions, runtime discovery, native installer/self-upgrade, eject/features acceptance | JSON is available only on documented commands; dependencies are not installed automatically. |
| Build/check/fmt/watch/dev/serve/view | Portable native and hosted engine tests; direct-file browser acceptance; watch/reload and failure recovery | Hosted rendering requires the documented container/engine prerequisites. |
| Book publication and PDF (rc.6) | Shared publication snapshot, direct LaTeX adapter, saved settings, atomic web/PDF builds and previews, static delivery/images, standalone export and native compatibility checks; three real PDF runtime jobs and 87 reviewed/matched captures | English/LTR classic template in A5/A4/Letter; opt-in external Linux runtime; Next-image PDFs, multilingual print, bibliography/index and technical/study templates remain open. |
| Static and Next images | Both Linux architectures, combined manifests, registry publication, fresh HTTP/browser checks | Images target Linux; the CLI itself ships on all six native OS/architecture targets. |
| Imports/migrate/ship | Six format fixtures through the real CLI and engine; source containment, assets and configured delivery tests | Supported source subsets are explicit; automatic full format/plugin parity is not implemented. |
| Deployment | Netlify, Cloudflare, Vercel, GitHub Pages, S3, Fly and Railway adapters and protocol tests | Live Fly/S3 acceptance pending; Vercel deferred by the user. |
| CI generation | GitHub/GitLab/CircleCI generator tests and installed-CLI delivery runs | Actual GitLab/CircleCI runner acceptance pending; provisioning/dispatch remain GitHub-specific. |
| Distribution | Published rc.6 assets; six native platforms and four engine variants on both Linux architectures; all eight public acceptance jobs and four public Homebrew jobs pass | Other package channels and live provider/external-runner acceptance remain deferred. |

Earlier importer milestone: Sphinx inclusion patterns, source `47b3293`, in
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

## Historical rc.5 template and distribution milestone

PR #138 merged as `a92c2e9`, with the exact tree accepted by all 20 jobs in
`43180055105`. Local preflight passed 741 Rust tests (five hosted-only cases run
in CI), 461 engine tests, warnings-denied Clippy, formatting, typechecks and
schema consistency. Hosted acceptance includes three templates at desktop/mobile
sizes, 317 showcase checks and preservation of 46 original documentation routes,
282 heading IDs and 29 image files, alongside ten added routes.

Release `43185514003` passed all 29 jobs. Both release repositories contain the
same 20 assets; public template checks verify source/mirror downloads, embedded
content and cached offline reuse. Homebrew `e0e8c31` passed four native jobs in
`43187271603`. PR #139 (`7430844`) migrated the six saved delivery workflows,
which all passed build-only checks; the Next job's first container-discovery
timeout recovered on one unchanged-head retry. No live provider was deployed.
Both acceptance images, all 13 development-image jobs and two fresh-runner HTTP
checks passed. rc.4's two release entries, 32 assets and two remote tags were then
retired while preserving all rc.5 and CodeSeal assets and source/audit history.

Review fixed ancestor path case collisions (`c370987`), missing product-card text
(`6810aee`) and a stale standalone image default (`3a9f807`). The catalog now
detects silently dropped YAML fields; general strict authoring diagnostics still
need that protection. At that milestone, book-classic was a reading website, with PDF/LaTeX and the
composer still planned; rc.6 implements their first versions. See [rc.5 evidence](evidence/zellij-rc5-release-2026-11-12.json).

## Historical rc.4 release and distribution milestone

PR #135 merged as `6d116b2`, with the exact tree accepted by all 20 CI jobs in
run `43158764791`. Local release preflight passed 726 Rust and 461 engine tests,
typechecks, schema drift, formatting and Clippy. Public release run `43163531316`
passed all 29 jobs. The Homebrew tap at `mylife-inc/homebrew-tap` passed all four
jobs in `43164789683`, including actual public downloads and revision upgrades.

PR #136 (`ea9d61b`) migrated all six saved delivery workflows to rc.4 and verified
their build paths without publishing Netlify drafts or other provider sites.
Static/Next image publication and fresh-runner HTTP checks passed. The four
mutable development image aliases were refreshed before retiring older versions.
Nine older release entries, 113 assets and nine remote tags were removed while
preserving rc.4, CodeSeal and source/audit history. See
[release evidence](evidence/zellij-rc4-release-2026-11-11.json) and
[retention status](zellij-release-retention-2026-11-11.md).

At that milestone, the next implementation was the versioned template catalog
and native creation flow, subsequently released in rc.5. The wider product expansion remains incomplete; these successful
checks do not establish the absence of undiscovered bugs.

## Historical rc.3 release milestone

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

Strict section-field diagnostics, recovered section headings and corrected native
starter feature text merged in PR #141 (`3008a16`), with all 20 final-head checks
passing. See [evidence](evidence/zellij-section-field-diagnostics-2026-11-12.json).
The historical rc.5 artifact remains unchanged. Bounded composer v1 merged in PR #143
(`ba5f0ad`), with stack/grid/split layouts and gradient reveal. All 20 final-head
checks, 335 showcase browser cases and 13 development-image jobs passed; six
light/dark desktop/phone captures were reviewed. See [composer evidence](evidence/zellij-compose-2026-11-12.json). Media-accordion and four composition presets subsequently merged in PR #147
(`2373c5e`), with all 20 CI and 13 development-image jobs passing; see
[media evidence](evidence/zellij-media-composition-2026-11-12.json).
The [book publication model](zellij-book-publication-2026-11-12.md) merged in
PR #149 (`72a8445`), with all 20 final-head jobs passing on the identical tree.
Optional PDF/LaTeX and configuration/preview/static delivery subsequently merged
in PRs #151 and #153. PR #155 versions the book starter around the shared
publication model and optional print settings, and publishes rc.6. Public
acceptance and Homebrew migration pass. PR #156 (`8eb117a`) merges docs and
consumer pins after all 23 CI and six delivery build jobs pass. Replacement
image checks and scoped rc.5 retirement are complete. Generated hosted/container CI still needs
explicit runtime selection: its Next consumer passed only after one unchanged-head
retry when Docker/Podman discovery timed out. Fix this reliability issue first. Advanced print semantics, technical/study editions and the remaining
installation channels follow. The first additional package channel is an owned
Scoop bucket, preceded by CLI protection against overwriting Scoop-managed
installations. Package availability alone does not establish safe upgrades. The [expansion design](zellij-product-expansion-2026-11-11.md)
tracks these separate feature milestones.

The original implementation/release/docs contract is complete. The user deferred
the following external verification phase until the end; account setup is not
required now.

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
  [hydration report](zellij-netlify-hydration-2026-11-02.md). The fix shipped in rc.4 and remains included in rc.5. Live verification
  still blocks production migration. Small CLI deployment-fixture success does
  not close it. Preserve the old production site and domain.
- **Zero bugs:** passing regression and acceptance gates is evidence of tested
  behavior, not proof that no undiscovered defect exists.
