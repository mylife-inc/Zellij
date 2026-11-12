# Pre-1.0 release retention and installation policy

Date: 2026-11-11. This records the user's explicit follow-up to the
[product expansion design](zellij-product-expansion-2026-11-11.md).
It is a release policy and implementation backlog, not a claim that pruning
automation or new package-manager channels already exist.

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

## Observed inventory and dependencies

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
needed. This settles the earlier namespace question; the tap is not yet live.

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
The future formula must respect Homebrew-managed upgrades and use verified
versioned artifacts from the public release mirror.

## Remaining implementation

- Prepare the next candidate containing the already merged branding/hydration
  changes and update the current installation/delivery references.
- Implement and test explicit preview discovery/installation as needed to retire
  the old stable distribution without silently changing stable semantics.
- Create the owned tap/bucket and validate their install/upgrade/uninstall paths.
- Implement exact-scope retirement inventory and cleanup, including safe repeated
  execution, preservation of the replacement and protection of CodeSeal assets.
- Perform the first retirement only after the replacement and dependent references
  pass verification. Record exact before/after evidence with that release.

This plan does not reopen live provider/account verification or production-site
migration, which remain deferred under the earlier user instruction.
