# Pre-1.0 release retention and installation policy

Date: 2026-11-11. This records the user's explicit follow-up to the
[product expansion design](zellij-product-expansion-2026-11-11.md).
Refreshed 2026-11-13 with rc.6 publication, native retirement and the reviewed
26-version image retirement. Earlier records and failed attempts remain historical.

## User decision

- Prefer the short installed executable and Homebrew formula name `zel`.
- Increment the version as new functionality is delivered. Never overwrite or
  reuse a previously published version, even after its artifacts are removed.
- During pre-1.0 development, retain the newest verified usable distribution
  and remove superseded Zellij releases/artifacts from channels we control.
  No existing users need a long archive of preview versions.
- Reserve `1.0.0` for the completed agreed feature set, no known unresolved
  bugs and passing acceptance/release gates. At that milestone, retire earlier
  Zellij distributions after migrating their consumers. Passing checks cannot
  establish that undiscovered bugs are impossible; do not make that guarantee.
- Preserve source commits, dated audits and verification evidence. Release
  retirement is not authorization to rewrite development history or erase
  unrelated products from a shared repository.

This supersedes any earlier preference to retain all public candidates. Existing
dated release records remain valid historical evidence; add retirement records
rather than silently rewrite them. The user has authorized this cleanup policy;
do not repeatedly request approval for a reviewed cleanup within its scope.

## Current rc.6 execution status — 2026-11-13

rc.6 is public in both repositories with 20 byte-identical assets, all eight
public acceptance jobs passing, and four public Homebrew checks passing.
PR #156 (`8eb117a`) merges active docs and consumer pins on the exact tree tested
at `f767260`: all 23 CI and six delivery build checks pass. The Next consumer
needed one unchanged-head retry after runtime discovery timed out before its
build; explicit runtime selection in generated container CI remains open.
No live providers were deployed. The original public PDF runtime failure and
its acceptance-harness correction remain in the release evidence.

Static and Next replacements passed `43503689558` and `43503694530`; both fresh
pull/HTTP checks in `43503982549` pass. All 13 refreshed development-image jobs
in `43498926083` pass on released source `668e8fa`; PR #156 changes none of its
runtime inputs. The shared installation guide is at `7cb742c`, and Homebrew is
at `1555b7d`. The actual standalone rc.5-to-rc.6 upgrade passed.

**Native retirement is complete:** exact source release `442694651` and mirror
release `442694716`, their 40 assets and two rc.5 remote tags were removed after
13 replacement gates and a read-only preflight passed. Seven focused guard tests
include refusing changed assets/tags, moved branches, failed gates and a draft
replacement. All rc.6 and CodeSeal release assets/tag identities remain intact.
See [native retirement evidence](evidence/zellij-rc6-native-retirement-2026-11-13.json).

**Image retirement is complete:** hosted inventory `43504106766` passes its
13 safeguard tests and builds a new exact plan at `8caf3c1`. It selects 26
versions: 20 base, three static and three Next. All 46 retained manifest nodes
and 3,321 excluded unreferenced build-history versions are protected. The plan
hash is `9d6d1b9e4241cb574d2524d56bc53979cef7d77fc10916e3ea07fe8960328cc8`.
The owner fallback executed exactly this [rc.6 plan](evidence/zellij-rc6-image-retirement-plan-2026-11-13.json)
at `a59b4fe`, recording 26 deletions with no failed or partial attempt. Fresh
hosted inventory `43505203415` confirms zero retired tags/targets and exact
preservation of every excluded version identity, digest and tag set. Both
fresh-pull HTTP checks in `43505207912` pass. Only rc.6 remains in the two
Zellij release channels. The native upgrade check selects rc.6 without changes.
See [execution and post-verification evidence](evidence/zellij-rc6-image-retirement-2026-11-13.json).
Do not reuse this completed plan after mutable tags move; broader untagged
history and live provider/production-domain work remain outside this cleanup.

## Historical rc.5 execution status — 2026-11-12

rc.5 is published with 20 identical assets in each release repository, all 29
release jobs and four public Homebrew jobs passing. PR #139 (`7430844`) updates
all six saved delivery pins and active guides. All six installed-CLI build-only
checks pass; one Next-image runtime-discovery timeout required an unchanged-head
retry. Replacement static/Next images, all 13 development-image jobs and both
fresh-runner HTTP checks pass. Public mirror docs are at `0ddf790`; both shared
installers remain byte-identical to rc.5 source.

The exact reviewed native plan removed only source release `442548169` and
mirror release `442548207` (rc.4), their 32 assets and their two remote release
tags. rc.5 source `442694651` and mirror `442694716` retain all 40 asset identities
and digests; the two CodeSeal releases retain all 12. Preview upgrade lookup
selects rc.5 after retirement. See
[execution evidence](evidence/zellij-rc5-native-retirement-2026-11-12.json) and
[release gates](evidence/zellij-rc5-release-2026-11-12.json). No source history was
rewritten; historical rc.4 URLs below now refer to retired artifacts.

**Scoped image cleanup is complete.** The owner's refreshed login removed
exactly 213 versions under the reviewed plan: 195 base, nine static and nine Next.
All 46 current manifest dependencies remain intact. Fresh hosted inventory
`43281128886` confirms zero retired tags/targets and preserves every excluded
build-history version. Both fresh-pull HTTP jobs in `43281136218` pass. All
40 rc.5 release assets and 12 CodeSeal assets retain their original identities
and checksums. The execution record below retains the failed workflow attempt
and the safely resumed owner attempt. See [execution evidence](evidence/zellij-rc5-image-retirement-2026-11-12.json).
The old 187-version rc.4 manifest is stale: rc.5 added protected tags and the
development aliases moved. The guard correctly refuses new/moved references.
Do not run the old plan unchanged. Its excluded untagged build-history inventory
is not a deletion list.

## Historical rc.4 execution status

The replacement is public in both repositories, with 16 matching assets and all
29 release jobs passing. The owned Homebrew tap is live and passed all four
native install/completions/ownership/build/revision-upgrade/uninstall jobs.
PR #136 (`ea9d61b`) migrated all six saved delivery workflows; their build checks
passed without provider deployment. Static/Next acceptance images and refreshed
`main` engine aliases passed publication and fresh-runner HTTP checks.

The shared install scripts and docs migrated at `mylife-inc/releases` commit
`d83dd7d`. Both installer files match tagged rc.4 bytes. Stable lookup still means
stable; after retirement it reports no stable release and directs users to
preview/exact-version selection. The actual `zel upgrade --preview --check`
resolves rc.4 without changing the binary. See
[release evidence](evidence/zellij-rc4-release-2026-11-11.json).

**Native retirement is complete:** nine older Zellij release records, their
113 assets and nine remote release tags were removed. Only rc.4 remains in the
two Zellij release channels. Both CodeSeal releases and rc.4 asset identities
were checked before and after and remain unchanged. The exact old inventories
and execution journal are in
[native retirement evidence](evidence/zellij-rc4-native-retirement-2026-11-11.json).
Source history and dated audit evidence remain; old release URLs in those
records are historical. Never reuse a deleted version or move the rc.4 tag.

**Image retirement is prepared but incomplete.** The
[reviewed manifest](evidence/zellij-rc4-image-retirement-plan-2026-11-11.json)
selects 187 exact versions: 175 in `zellij-base`, six in `acceptance-static` and
six in `acceptance-next`. It protects all rc.4/main tags and their 46 manifest
dependencies, including shared children. Eight guard tests cover changed tags,
recreated identities, missing retained graphs, unknown references, orphaned
build history and safe resumption. The workflow requires the exact plan SHA256,
checks replacement acceptance and current identities, and journals each deletion.
It is specific to this reviewed rc.4 plan, not a general automatic retention job.

Hosted attempts `43168090301` and `43168570526` passed the read/graph safeguards,
but GitHub rejected the first DELETE with HTTP 404 (`Package not found.`).
Both journals have zero deletion events. The endpoint/API version was checked
against [GitHub's package API](https://docs.github.com/en/rest/packages/packages#delete-package-version-for-a-user).
A package-level Admin role is required for workflow deletion, as described in
[GitHub's deletion permissions](https://docs.github.com/en/packages/learn-github-packages/deleting-and-restoring-a-package).
The user was asked to inspect/grant that role to `mylife-inc/Zellij` for the three
packages. Do not infer success from the existing write/publish capability.
See [image retirement evidence](evidence/zellij-rc4-image-retirement-2026-11-11.json).
At that time the plan could be retried only with unchanged aliases. The rc.5
update above now requires a new reviewed plan before any retry, because aliases
and protected tags have changed. Verify current images again after cleanup.

The plan deliberately does not sweep 3,152 untagged versions outside the selected
old-release graphs. These are separate build-history cleanup work, not current
published release tags. Audit their reachability/retention before deleting them.
Live deployment-provider verification and production docs/domain migration were
not performed in this milestone.

## Historical inventory and dependencies before rc.4

Refreshed through GitHub on 2026-11-11 at source `df32e0e`:

| Location | Zellij releases observed | Important boundary |
| --- | --- | --- |
| `mylife-inc/Zellij` | `v0.2.0-rc.1`, `v0.2.0-rc.2`, `v0.2.0-rc.3` | rc.3 is the newest published candidate. |
| `mylife-inc/releases` | `zellij-v0.1.0`, `zellij-v0.1.2`, `zellij-v0.1.3`, and `zellij-v0.2.0-rc.1` through `rc.3` | Also contains CodeSeal releases; never apply repository-wide deletion. |

The six frozen delivery workflows still select `v0.2.0-rc.2`:
`acceptance-hosted.yml`, `acceptance-portable.yml`, `acceptance-next.yml`,
`acceptance-static.yml`, `acceptance-netlify-hosted.yml` and
`acceptance-netlify-portable.yml`. Their recorded live checks belong to that
version. The local shell/PowerShell installer and native upgrade logic distinguish
stable versions from candidates; the shared repository still supplies `0.1.3`
as the newest Zellij stable release. The release workflow intentionally updates
shared installer files only for stable releases.

Deleting these versions immediately would leave active pins or default stable
installation without a valid target. This dependency is addressed during the
next replacement release, not by silently retaining old versions indefinitely.
No releases, tags, packages or images were deleted in this documentation update.
Refresh inventories and remote installer bytes before any future removal.

## Replacement and retirement sequence

1. Select a new, strictly increasing version and complete its feature/docs
   changes. Keep release artifact bytes and checksums immutable for that version.
2. Publish to the owned source/public-release destinations. Verify all supported
   native packages, checksums, image variants and installation/upgrade paths.
   If publication or acceptance fails, keep the previous usable version.
3. Update current docs, template catalogs, package-manager recipes, delivery
   workflows and release consumers to the replacement. Preserve old evidence
   with its original version rather than imply a retargeted workflow has already
   passed live acceptance. Remaining live-provider work is still user-deferred.
4. Migrate the installer/channel contract before removing the last stable
   artifact: expose the development candidate explicitly as a preview and make
   the documented installation path select it. Do not silently label an rc as
   stable or resolve a shared repository's global `latest` to another product.
   Verify the remote installer files as well as the copies in this checkout.
5. Produce a concrete retirement manifest listing exact repository/release IDs,
   tags, asset inventories and exclusively owned image/package versions. Check
   active consumers and running jobs for old references; distinguish historical
   audit links and test literals from actual download/runtime dependencies.
6. Remove superseded Zellij release entries/assets and their release tags where
   appropriate. Remove corresponding registry/image versions only when they
   are not needed by retained manifests, architecture variants or aliases. Match
   both project ownership and exact version identity. Never delete a shared
   repository, all its releases, or unscoped package versions.
7. Recheck current installs and references after retirement, and record what
   was removed, what remains and why. Cleanup must be idempotent and resumable;
   a partial failure must not erase the last accepted replacement or report
   complete removal inaccurately.

The intended steady state before 1.0 is the latest accepted development release,
not an archive of older candidates. A temporary overlap is allowed only while
the replacement, its installation channels and dependent pins are being made
usable. A future 1.0 release follows the same publish/verify/migrate/retire order.
Decide post-1.0 retention separately when there are actual users.

Removal covers distributions and references under our control. It cannot revoke
third-party downloads, caches or forks. Some package catalogs restrict removal
of previously published versions; check their policy before publishing previews
there, and delay that channel until 1.0 if it conflicts with the requested
retention model. Do not promise erasure from the entire Internet.

## Homebrew naming

The desired formula name is `zel`. The fully qualified tap form has three
components: `owner/tap/formula`. The user subsequently confirmed that **Shebka**
is the company's current name, **MyLife** was its former name, and the existing
GitHub account **`mylife-inc`** is owned by the user. The agreed installation
command is **`brew install mylife-inc/tap/zel`**. Retain that technical namespace
while using Shebka as the company identity; no GitHub account migration is
needed. This settles the earlier namespace question. The tap is now live at
[mylife-inc/homebrew-tap](https://github.com/mylife-inc/homebrew-tap), selecting
the explicitly documented rc.5 preview.

On a fresh machine, an owned tap gives a one-command installation such as
`brew install mylife-inc/tap/zel`. After tap setup and the formula-specific
trust required by current Homebrew, the short `brew install zel` is possible
provided no higher-priority formula conflicts. See
[tap naming and short-name installation](https://docs.brew.sh/Taps) and
[direct installation](https://docs.brew.sh/How-to-Create-and-Maintain-a-Tap).

No-setup `brew install zel` through the main catalog depends on upstream
acceptance. Current [Homebrew core requirements](https://docs.brew.sh/Acceptable-Formulae)
include a stable release, eligible open-source licensing and source availability;
the user's potential later sealing of the source must be considered before
promising this distribution path. An owned tap is the initial delivery route.
The published formula respects Homebrew-managed upgrades and uses verified
versioned artifacts from the public release mirror.

## Remaining work

- Separately review untagged build-history retention before any wider cleanup.
- Add Scoop and other selected package-manager channels with native acceptance;
  respect external registries' version-removal limits before publishing previews.
- For the next release, repeat publish/verify/migrate/retire with a new concrete
  inventory. Do not reuse the rc.4 plan against changed manifests or tags.

This plan does not reopen live provider/account verification or production-site
migration, which remain deferred under the earlier user instruction.

## Refreshed image plan preparation — 2026-11-12

The user confirmed package-level Admin access. Composer PR #143 merged, and all
13 development-image jobs in `43274667686` passed before refreshing inventory.
The retirement helper now retains rc.5/main variants and includes the retired
rc.4 graphs; nine guard tests pass, including refusal of the stale rc.4 plan.
Replacement gates use the verified rc.5 release, acceptance images, HTTP check
and current development engine. All six saved delivery workflows still use rc.5.

Read-only inventory `43275231189` found 3,457 base-image versions and 31 acceptance
versions. Local manifest traversal timed out without producing a deletion plan.
Inventory mode now also generates a read-only plan in the hosted workflow;
execution still requires a committed exact plan and its matching SHA256. The old
187-version manifest is historical. No image has been deleted in this retry yet.

Hosted planning `43276113201` passed on `beed177`. The
[rc.5 plan](evidence/zellij-rc5-image-retirement-plan-2026-11-12.json) selects
213 exact versions: 195 base, nine static and nine Next. All 187 earlier targets
retain their identities; the additional 26 are rc.4 roots and exclusive children.
It protects 46 current manifest dependencies and excludes 3,229 unreferenced
build-history versions. Retained tags are the four rc.5 engine variants, four
current main variants, and rc.5 static/Next acceptance images. Exact SHA256:
`26a5c1c3d7fb002950bea73779d01bf8d024e5ce677afa9ee38a6f818e043a57`.
Nine guard tests, workflow lint and validation against the downloaded inventory
pass. Execution and post-cleanup HTTP verification remain pending at this checkpoint.

## Owner-login fallback preparation after the Admin grant — 2026-11-12

This preparation checkpoint precedes the completed execution recorded below.

PR #144 merged the refreshed plan and gates as `4383ae7`. Hosted execution
`43276711406` passed nine guard tests and replacement/inventory/graph preflight,
then GitHub returned `Package not found` (HTTP 404) for the first DELETE,
`zellij-base` version `1547715033`. Its journal contains zero deletion events.
The confirmed Admin grant did not resolve the workflow-token failure; the
underlying GitHub cause is not established. The original failed receipts remain.

The helper now exposes `apply-local` for the authenticated `mylife-inc` owner.
It requires package read and delete scopes and refuses use inside GitHub Actions.
The API host is fixed to `github.com`. It inspects identity and scope names without
reading or printing the credential. The hosted `apply` path retains its repository,
branch and workflow-token restrictions. Both modes share the exact plan hash,
replacement acceptance, full inventory, retained graph, per-version identity,
journal and post-deletion checks. The journal is initialized before mutations.
All 13 local guard tests pass. No owner-authenticated apply has run yet.

The existing local login is `mylife-inc` with `gist`, `read:org` and `repo` scopes.
The user was asked to authorize the missing package scopes with:

```sh
gh auth refresh --hostname github.com --scopes read:packages,delete:packages
```

This follows [GitHub's package permission requirements](https://docs.github.com/en/packages/learn-github-packages/about-permissions-for-github-packages).
Do not copy the owner's credential into repository secrets or chat. After the
refresh, verify the identity/scopes, merged source, unchanged plan hash and
absence of concurrent publishers. Run the exact reviewed plan with `apply-local`,
then obtain a fresh hosted inventory and current-image HTTP checks. Record the
result before claiming completion; a prepared fallback is not successful cleanup.

## Completed owner execution and verification — 2026-11-12

PR #145 merged as `637f2ee` with the exact tree of tested head `27e62c7`.
All 13 guard tests passed locally and in hosted read-only run `43278582061`.
Its fresh plan matched every reviewed target, retained reference and protected
manifest. The user then refreshed the existing `mylife-inc` login with package
read/delete scopes. There were no concurrent publication workflows.

The first owner attempt deleted 67 base-image versions and stopped when a
per-version GET returned HTTP 401. The login and required scopes remained valid;
the same GET immediately succeeded. Its cause is unproven. The second attempt
used unchanged source and the same plan checksum, revalidated every acceptance
and inventory/graph gate, recorded the 67 already-absent identities and deleted
the remaining 146 versions. The two journals together contain exactly the 213
planned deletions, each once, with no extra identity. The final journal is complete.

Hosted inventory `43281128886` confirms the exact difference: base versions
3,457 → 3,262, static 16 → 7 and Next 15 → 6. The new plan has zero retired tags
and zero remaining targets. All 46 retained manifest dependencies and all 3,229
excluded build-history versions match the original plan. Both retained acceptance
images passed fresh pulls, health checks and HTTP acceptance in `43281136218`;
the provider job was skipped. Release asset comparisons preserve all 40 rc.5
assets and all 12 CodeSeal assets. No native release or Homebrew update was needed.

The [consolidated evidence](evidence/zellij-rc5-image-retirement-2026-11-12.json)
links both owner execution journals and records workflow IDs, artifact hashes,
retained image digests, inventory counts and the earlier failed receipts.
Future releases require a newly reviewed plan; do not reuse this completed plan
after mutable tags change. Provider acceptance and production docs migration
remain deferred.
