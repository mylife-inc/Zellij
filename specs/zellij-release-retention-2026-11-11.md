# Pre-1.0 release retention and installation policy

Date: 2026-11-11. This records the user's explicit follow-up to the
[product expansion design](zellij-product-expansion-2026-11-11.md).
The policy is followed by the actual rc.4 execution record. Completed native
retirement and Homebrew publication are distinct from pending image cleanup.

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

## Current rc.4 execution status

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
After access is corrected, rerun the reviewed workflow on current main with
`mode=retire-images` and the recorded plan hash; it rechecks live references and
refuses drift. Replan if aliases moved. Verify current images again afterward.

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
the explicitly documented rc.4 preview.

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

- Complete the reviewed 187-version image retirement after package Admin access
  is confirmed, then verify retained public images and record the result.
- Separately review untagged build-history retention before any wider cleanup.
- Add Scoop and other selected package-manager channels with native acceptance;
  respect external registries' version-removal limits before publishing previews.
- For the next release, repeat publish/verify/migrate/retire with a new concrete
  inventory. Do not reuse the rc.4 plan against changed manifests or tags.

This plan does not reopen live provider/account verification or production-site
migration, which remain deferred under the earlier user instruction.
