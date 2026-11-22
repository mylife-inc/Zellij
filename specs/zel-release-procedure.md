# Zellij release channels and candidate procedure

**2026-11-21: rc.18 publication.** Immutable source `faaee97` ships cached
static validation/watch, HTTP build/preview and native provider static-build
stages. Source CI `44987846589` passes all 37 required jobs; both repositories
contain 40 matching assets. Independent public Mac, installed upgrade and
empty-PATH default-home/offline checks pass. Seven guide/consumer checks pass.
All 40 release jobs pass in `44989656538`; all eight actual rc.17-to-rc.18
Homebrew/Scoop lifecycle cases pass. Production receipts are recorded in the
[rc.18 evidence](evidence/zellij-rc18-release-2026-11-21.json). Editable wrapper,
optional typesetting and Next development packages remain future work. No old
distributions are retired; retain compatibility and actual-upgrade fixtures.

**2026-11-21: rc.17 publication completed.** Immutable source `0bd1e8b` passes
37 source and 40 release jobs; both repositories contain 40 matching assets.
The public renderer includes `f80596b`'s menu focus correction. Public Mac
standalone/direct-file checks, local verified upgrade and all eight actual
rc.16-to-rc.17 Homebrew/Scoop lifecycle cases pass. The nine template identities
are unchanged. Main images pass 13 jobs; seven targeted guide/consumer checks
pass. Production `44951019765` and live rc.17 page checks pass; both current
fixture images publish and pass fresh-pull inspection `44951268820`. No old
distributions are retired. See the
[release evidence](evidence/zellij-rc17-release-2026-11-21.json). Proposed cached
static validation, HTTP export, wrapper and optional typesetting/development
packages remain future work; do not describe them as rc.17 behavior.


**2026-11-21: rc.16 publication completed.** Immutable source `e3fe086` passes
37 required source CI and 40 required release jobs. Both repositories publish
40 matching assets, including the three corrected 0.1.3 website-starter guides.
All nine public template identities and archive reproductions pass. All eight
real rc.15-to-rc.16 Homebrew/Scoop upgrades, public WinGet x64/ARM64 manifests,
physical Mac standalone/browser acceptance and production docs publication pass.
No releases are retired: preserve rc.11/rc.13 compatibility pins, the rc.14
WinGet fixture and rc.15 Microsoft submission/channel upgrade dependencies.
See [the dated evidence](evidence/zellij-rc16-release-2026-11-21.json) and
[current readiness](zellij-readiness-2026-11-21.md). Earlier entries below remain
historical records, not the current release status.

**Historical rc.12 distribution status — 2026-11-19.** Immutable `v0.2.0-rc.12` at `73979de`
passes all 38 release jobs in `44637677186`, including final public-artifact
checks. Both repositories have 39 matching asset digests. Homebrew and Scoop
public lifecycle and actual rc.11-to-rc.12 upgrades pass on all eight managed
cases. The standalone local Mac installation is updated and verified with an
empty PATH, first renderer download and byte-identical offline rebuild.
Public-guide PR #212 merges as `de409a6` after all 35 general CI jobs and six
build-only consumers pass. Automatic production docs run `44645212486` verifies
the promoted artifact; an independent live build-guide check matches it. See
[rc.12 evidence](evidence/zellij-rc12-release-2026-11-19.json).
Prior-candidate retirement remains pending; do not delete rc.11 or its images
without the replacement-image gates and a reviewed scoped plan.

**Documentation publication — 2026-11-19:** Production documentation now has its
own `docs.yml` workflow, independent of release tags. See the
[documentation publishing guide](zellij-docs-deployment.md). Earlier dated
release records below retain their original publication policy and outcomes.

**Current rc.11 distribution status — 2026-11-18.** The candidate, Homebrew,
Scoop and verified guides are published. Documentation PR #208 merges as
`79de857`. All source/release/public-artifact, managed lifecycle/upgrade and
consumer checks pass. PR #209 completes reviewed rc.10 retirement: two releases,
54 assets, two tags and 26 image versions removed; all 39 current manifests,
3,771 excluded build-history versions and both CodeSeal releases retained.
Independent inventory `44433078378`, fresh-pull HTTP `44433082092` and fresh anonymous
public downloads pass. See [retirement evidence](evidence/zellij-rc11-image-retirement-2026-11-18.json).
Production hosting and live providers remain deferred; additional installation
channels remain separate work. No stable 1.0 or zero-bug claim.

**Earlier 2026-11-18 checkpoint: rc.11 publication and verification.** Immutable tag
`1f8232b` passes all 37 jobs in `44423006473`, including 12 final-public-artifact
checks. Both repositories have 27 byte-identical assets and retain all nine
current template archive hashes. All 224 website captures and 46 PDFs/420 page
captures match reviewed source content and pixels. All managed lifecycle and
actual upgrade cases, the standalone upgrade, all 32 documentation CI jobs and
six build-only consumers pass. PR #208 is ready to merge; replacement acceptance
images and scoped rc.10 retirement follow.
See [the evidence](evidence/zellij-rc11-release-2026-11-18.json).

**2026-11-18: rc.11 preparation.** The source implementation is verified at
`0d25d84`; publication remains pending. GitHub withdrew the development-only
picomatch advisory on November 18. Full and production audits now report zero
findings; the temporary exception is removed. The dated finding below remains
historical. Continue to audit immediately before every release tag, even after
green CI. See [the rc.11 plan](zellij-rc11-preparation-2026-11-18.md).

**2026-11-17: v0.2.0-rc.10 is published and publicly verified.** Immutable tag
`9319729` passes all 37 release jobs in `44159670077`; expanded public acceptance
`44163817890` passes all 12 jobs, including the tagged language-PDF driver.
Both release repositories have 27 matching assets and nine current template
archives. Forty-six public PDFs/420 page captures and 224 website captures
match reviewed source output. All eight managed lifecycle checks and eight
actual rc.9-to-rc.10 upgrades pass; one Homebrew macOS ARM job required a retry
after GitHub failed to allocate a runner. Documentation merge and retirement
status are tracked in [the rc.10 record](evidence/zellij-rc10-release-2026-11-17.json).
Documentation PR #200 is merged as `d5ed0bf`. Scoped rc.9 retirement is complete
after all 17 gates: two releases, 52 assets, two tags and 26 image versions removed.
Fresh inventory `44175159235` and both HTTP checks in `44175166206` pass;
38 current manifest nodes, 3,743 excluded build-history versions and CodeSeal
remain intact. [PR #201](https://github.com/mylife-inc/Zellij/pull/201) records
[the exact cleanup evidence](evidence/zellij-rc10-image-retirement-2026-11-17.json).

The full repository audit has an open, unpatched development-only picomatch
advisory. Production and fresh all-features wrapper audits have no findings.
The [exact disposition](zellij-picomatch-disposition-2026-11-17.md) expires
December 1; it is not a fix or a clean full audit. Run the audit immediately
before future release tagging, even when an earlier source CI run was green.

**2026-11-16: v0.2.0-rc.9 is publicly verified.** The immutable source tag is
`f03f803`; release run `43825155269` passes all 37 jobs. Both repositories have
26 matching assets, including eight template archives. Source CI passes 32 jobs,
and normal preflight passes 802 Rust and 491 engine tests. Public acceptance
verifies six native installations, all image variants on both Linux architectures,
24 PDFs/224 page captures and 170 website/navigation captures. Managed upgrades,
documentation migration and retirement status are tracked in
[the rc.9 record](evidence/zellij-rc9-release-2026-11-15.json).
Earlier release checkpoints below remain historical; refresh GitHub state before
using their versions or run IDs.


**2026-11-14: v0.2.0-rc.7 is published** at immutable tag/merge
`f3d55f6`. All 33 jobs in release `43646140679` pass. Both release repositories
have 21 byte-identical assets; all three template archives retain their previous
bytes and versions. Six public native installs and both Linux image checks pass;
16 public PDFs/128 page captures match the reviewed content and pixels.
Shared equations, theorem/proof blocks, numeric citations, glossary/index,
responsive focus fixes, Scoop diagnostics and generated-CI runtime selection
are included. The actual standalone rc.6-to-rc.7 upgrade passes.

The owned Homebrew tap at `ebdcc19` and Scoop bucket at
`0d57926` publish the attached, verified rc.7 recipes. All four public
lifecycle checks and all four actual rc.6-to-rc.7 upgrade checks pass for
each channel. The upgrade checks verify both installed CLI versions and
portable builds; they use actual HTTPS release archives. See
[release evidence](evidence/zellij-rc7-release-2026-11-14.json).
Documentation/consumer migration and scoped rc.6 retirement are complete.
Production hosting and deferred live provider checks are unchanged.

PR #176 merges as `eae65fb`, matching reviewed tree `e3f7014`. Replacement
static/Next publications `43651607322`/`43651609913` and both fresh-pull HTTP
checks in `43651756802` pass. Scoped rc.6 retirement is complete after 16 exact
replacement gates: two releases, 40 assets, two remote tags and 26 obsolete
image versions removed. All 38 current manifest nodes, 3,517 excluded build-history
versions and both CodeSeal releases remain intact. Fresh inventory `43652529198`
and both post-cleanup HTTP checks in `43652531820` pass; only rc.7 remains in
the two Zellij release channels. See [retirement evidence](evidence/zellij-rc7-image-retirement-2026-11-14.json).

## Earlier release and implementation milestones

Implemented policy, November 2, 2026; publication status refreshed November 13.
Candidate **0.2.0-rc.6 is published and publicly verified**. Both repositories
have 20 identical assets. All eight jobs in public acceptance `43496517451`
pass, using harness `3d26a96` against immutable source/tag `668e8fa`. The original
release run's Linux ARM64 PDF failure remains recorded: it selected unconfigured
rootless Podman; the new harness selects/preflights the hosted Docker endpoint.
All four Homebrew jobs in `43497080249` pass for tap commit `1555b7d`.
See [the release evidence](evidence/zellij-rc6-release-2026-11-13.json) and
[preparation history](zellij-rc6-preparation-2026-11-13.md).
Remaining live provider/external-runner checks and the production-domain
migration remain deferred. Replacement consumer/image checks and scoped rc.5
retirement are complete; only rc.6 remains in the two Zellij release channels.
See [retirement evidence](evidence/zellij-rc6-image-retirement-2026-11-13.json).
Publication status must be checked in GitHub; a version change in source is not
proof of an uploaded release.

## Historical rc.4 publication

PR #135 merged as `6d116b2ca6df8eb8d078f03978d2a49598b58de1` with the same tree
as the 20-job final CI head `6a2601c` (run `43158764791`). The required local
preflight passed 726 Rust and 461 engine tests, typechecks, schema drift,
formatting and Clippy. Tag `v0.2.0-rc.4` points at that merge and is immutable.
Release run `43163531316` passed all 29 jobs without reruns; both repositories
have the same 16 assets, including the generated `zel.rb` formula.

The public [Homebrew tap](https://github.com/mylife-inc/homebrew-tap) at `5abad25`
passed all four actual-public-install acceptance jobs in `43164789683`.
`brew install mylife-inc/tap/zel` selected the rc.4 preview at that milestone. Homebrew
owns subsequent upgrades; the installed CLI directs users to
`brew upgrade mylife-inc/tap/zel` before making a network request.

PR #136 (`ea9d61b`) retargeted six saved delivery workflows. All six build checks
passed; provider publish jobs were deliberately skipped. Replacement acceptance
images and development aliases were published and checked before cleanup.
The shared repository's installer files/docs were explicitly migrated in
`d83dd7d` as part of the authorized retirement. Their bytes match tagged rc.4
source. This was a reviewed maintenance operation; automatic candidate
publication still does not replace shared installer files.

Nine older release records/assets/tags have been retired. Stable lookup now
reports no stable Zellij release and suggests preview or exact-version selection;
it never substitutes a candidate or CodeSeal. `zel upgrade --preview --check`
resolves rc.4 without modifying the executable. See
[release evidence](evidence/zellij-rc4-release-2026-11-11.json) and the
[retention record](zellij-release-retention-2026-11-11.md). Old release links below
are historical; their original artifacts need not remain downloadable.
Production documentation was built but not deployed. Provider verification and
the production-domain migration remain deferred.

## Historical rc.3 publication

[PR #127](https://github.com/mylife-inc/Zellij/pull/127) merged as
`58be9b1ab992424576572ee8e2ba736f134f727c` with the exact source tree tested by
all 16 jobs in CI run `43013070798`. The release helper's local preflight passed
720 Rust and 453 engine tests, typechecks, schema drift, formatting and Clippy.
It created immutable tag `v0.2.0-rc.3` at that merge.

Duplicate merge-push CI was skipped for the reviewed version/docs/fixture change.
The Release workflow was then explicitly dispatched against the tag. Run
[`43018433622`](https://github.com/mylife-inc/Zellij/actions/runs/43018433622)
passed all 29 jobs without reruns. All six public package/install targets and
all four engine variants on both Linux architectures passed. The development
and shared release repositories have matching assets; candidate isolation
skipped stable-installer updates and production docs deployment.

See [the rc.3 release evidence](evidence/zellij-rc3-release-2026-11-11.json) for
asset digests, immutable image manifests and exact verification boundaries.
Live provider/external-runner acceptance remains deferred. Frozen rc.2 delivery
fixtures can be retargeted when that phase resumes; they were not dispatched
for rc.3.

## Previous rc.2 publication

The user approved publication from exact commit
`54e636faf2162c36e50a91507f1f35be63a660d0`, the reviewed merge of PR #104.
[Release run 42768544795](https://github.com/mylife-inc/Zellij/actions/runs/42768544795)
passed all 29 jobs, including six native public-download/install checks and all
four engine variants on native Linux AMD64/ARM64. Both release repositories
contain matching archives; their installers match the tagged source. See the
[release record](evidence/zellij-rc2-release-2026-11-09.json) for public checksums
and immutable image digests. Production documentation and stable installer
updates were skipped under the candidate policy.

The [2026-11-09 preparation record](evidence/zellij-rc2-preparation-2026-11-09.json)
retains the pre-publication evidence. It superseded the old proposed target
`0fbf0a12c3cf7074c4620e2bb84cc5d8e5fd7469`; the [2026-11-02 record](evidence/zellij-rc2-preparation-2026-11-02.json)
remains historical, including its earlier approval rejection. The user later
explicitly authorized the refreshed candidate. Never move the published tag to
a later commit or replace its assets to include further implementation work.

The six delivery fixtures are retargeted to rc.2 and regenerated with the CLI.
Their PR verification and manual-main delivery runs need separate evidence;
old rc.1 delivery outcomes do not establish rc.2 acceptance. Netlify tests use
the existing approved staging site and draft previews. Production/domain
migration and live acceptance on other providers remain separate tasks.

## Channels

| Build | Image tags, before feature suffix | GitHub release | Production docs and shared installer scripts |
| --- | --- | --- | --- |
| `main` branch | `main` | None | Relevant docs changes deploy after documentation checks; installer scripts unchanged |
| `v0.2.0-rc.1` example | `0.2.0-rc.1` only | Prerelease, never latest | Unchanged; docs still built |
| `v0.2.0` example | `0.2.0`, `0.2`, `latest` | Stable | Installer scripts updated; docs built but publication remains independent |

All four image variants follow this rule: no suffix, `-maths`, `-diagrams`,
and `-all`. Container versions omit the leading `v`. Release assets in this
repository use `v…`; the shared `mylife-inc/releases` repository uses `zellij-v…`.
Branch workflow dispatches only publish when run against `main`.

`scripts/release-policy.mjs` supplies version validation, image tags and pinned
release notes to CI and the local publishing helpers. Release notes download the
installer attached to that specific release and explicitly select its version
and asset directory. Automatic candidate publication cannot replace the shared installer's `main`
files, enable production documentation deployment or become GitHub's latest
release. Both Unix and PowerShell instructions are generated from the same policy.

This policy governs future publications. It does not retag existing images:
older branch workflows wrote `latest`, subsequently retired during rc.4 cleanup. Use an exact matching image tag
for candidate acceptance. Source development can explicitly select
`ZELLIJ_IMAGE=ghcr.io/mylife-inc/zellij-base:main`; `main` remains mutable.

## Managed Windows distribution

The owned [Scoop bucket](https://github.com/mylife-inc/scoop-bucket) publishes
rc.7 for native Windows x64/ARM64. All four public lifecycle jobs in `43647962901` and all four actual
rc.6-to-rc.7 upgrades in `43648717666` pass.
The release pipeline requires all four Scoop checks to succeed before native
publication and attaches `zel.json` generated from verified `SHA256SUMS`.

For each candidate, copy its verified manifest to the bucket, update its README
and CI version, and pin the reviewed acceptance tooling. Run public installation,
architecture/shim/ownership checks, checksum-failure preservation, a fixture
package update and uninstall. Before retiring the previous CLI, dispatch the
bucket workflow with `upgrade_from` set to its still-published tag; this installs
and upgrades the real old/new HTTPS archives in a disposable bucket. The empty
default keeps later lifecycle checks independent of retired downloads.

For Homebrew, the owned tap workflow similarly accepts `upgrade_from_commit`,
the full previous tap commit SHA. It installs that formula, restores the current
recipe, upgrades and checks both real CLI versions and portable builds.
The standalone public executable also runs `upgrade --preview --check` and
`upgrade --preview` in an isolated directory; its final bytes must match the
verified current archive. Fixture-only package revisions do not establish a
real CLI migration. See [Scoop history](zel-scoop-distribution-2026-11-13.md).

## Prepare a candidate

1. Merge the reviewed changes with all CI jobs passing on the final commit,
   including six native archive/installer targets and real container/browser
   acceptance. A runner cancellation is not a passing test.
2. Set the Rust workspace version in `cli/Cargo.toml`, refresh `cli/Cargo.lock`,
   and review version-bearing documentation and release notes. Run the normal CI
   gates on this version change. Do not use an already published tag again.
3. Check repository access, GHCR permissions and the optional
   `RELEASES_GITHUB_TOKEN` secret by name/existence only. That token enables
   publication to `mylife-inc/releases`; without it, the workflow only creates
   the development repository release and emits a warning. No extra token is
   needed for ordinary package/installer tests.
4. From the reviewed, clean repository root, run
   `node scripts/check-dependency-audit.mjs` and require success. Registry
   advisories can change after source CI; any bounded development disposition
   must still be unexpired and applicable. Then run `source scripts/app.sh`, followed by
   `zrelease v0.2.0-rc.10` (substitute the actual agreed **new** candidate). The helper
   checks the committed native version before creating or pushing a tag. A
   candidate does not prompt to enable production docs. Monitor `zrelease-watch`.
5. Check every release job and all six attached packages. The native publication
   job intentionally permits an independent image failure; a native release page
   alone does not establish a complete release. Retry/fix the failed image
   publication before calling the candidate ready.
6. Install from the published candidate's own pinned instructions in clean
   environments. Verify the reported version and portable build, direct
   `file://` navigation/search/assets, serve/view/dev, then container commands
   using the matching exact image tag. Exercise generated CI and deployment
   against explicitly selected test destinations. CI artifact acceptance before
   upload and acceptance of the final downloaded release are separate checks.

Candidate notes live in specs/releases/<version>.md and are included automatically
in both release pages. The rc.10 notes are specs/releases/v0.2.0-rc.10.md;
the earlier rc.2 publication evidence is linked above. The Published release acceptance workflow runs after
publication and can be dispatched again against the same tag. It downloads both
repositories' public assets, checks their bytes and checksums, uses their attached
installers, and exercises all six native targets. It also pulls and executes all
four engine variants on native Linux AMD64 and ARM64. No registry/cloud write
credentials are passed to these acceptance jobs. Native Windows
and macOS/Linux acceptance remains on GitHub's native runners; there is no
requirement to start Docker on the maintainer's laptop.

Public-asset downloads allow at most three attempts for transient connection
errors and timeouts. Each attempt starts with empty buffers and a fresh timeout;
partial responses are discarded. Missing files, oversized responses, checksum
failures and installer/source mismatches fail without retrying. The original
0.2.0-rc.1 tag predates this download-recovery improvement; rerunning that tag's
workflow still uses its original downloader. To verify an existing release with
the latest acceptance code, select the Published release acceptance workflow on
`main` and provide its published `version` tag, for example (use a still-published version):

```sh
gh workflow run release-acceptance.yml --ref main -f version=v0.2.0-rc.10 -f shared=true
```

The workflow checks out `refs/tags/<version>` separately. Version, engine metadata,
installer byte comparisons and the native acceptance suite use that tagged source;
only the download/acceptance driver comes from the selected workflow revision.
On native Linux, print-capable catalogs additionally run the tagged PDF
acceptance driver with the installed public binary, retaining PDF captures and
receipts. When present, the tagged language-PDF driver also runs with the same
downloaded binary. Those hosted checks explicitly select and preflight Docker's default
context, matching the dedicated Docker PDF jobs. Preinstalled rootless runtimes
are not assumed to have delegated resource controllers; dedicated Podman jobs
provision their own rootless session. Older reading-only releases skip this check. Template packaging checks
known archive hashes in `templates/published.json`; record new hashes only after
public verification, and never rewrite a previously published identity.
This performs public reads and temporary local installs on the runners. It does
not rebuild, republish or move a release tag. Record both the source tag and the
acceptance workflow revision when reporting this verification.

Public acceptance also runs the released Next-image PDF and website drivers
when that tagged source supplies them. The shared public-engine helper installs
checksum-verified public CLI bytes, pulls the exact image anonymously, verifies
its architecture/digest and retains an installation receipt. It never rebuilds
the CLI or engine. Both drivers run on native Linux amd64/arm64.

Website acceptance covers the four website starters and a separate authored
company/nested-menu fixture: light/dark, desktop/phone, centered header,
no-JavaScript content, Back/Close focus, reduced motion and external new tabs.
Compare public captures against reviewed source captures on the same architecture.
Older tags explicitly skip a driver they do not contain; that skip does not imply
support for its feature. The remaining native, template and image checks remain
required. After active guides are verified and merged, point release-note links
to their exact documentation commit without changing assets or release tags.

## Stable promotion

After candidate acceptance, commit the stable workspace version and run the
same review/CI process before creating its new stable tag. The release workflow
builds and publishes that version; it does not turn a candidate tag into stable.
Stable and candidate tags build documentation but do not publish it. The
independent `docs.yml` workflow publishes from `main` using the dedicated
`NETLIFY_DOCS_PRODUCTION_TOKEN` secret. The old `DEPLOY_DOCS` variable is no longer
used. Label forthcoming functionality clearly so documentation updates do not
claim commands are available in an older published executable.

`scripts/publish.sh image` and `release` enforce the same candidate policy.
`zrelease-local` delegates publication to that helper, but remains an emergency
two-target fallback; it is not the six-target acceptance/release path.

Development helpers `zimage` and `zimage-push` default to `local`; `zimage-all`
builds `local`, `local-maths`, `local-diagrams` and `local-all`. The raw push helper
still accepts an explicit tag. Use the release helpers for release publication,
which apply the stable/candidate policy automatically.

## Verification

Run `node --test scripts/release-policy.test.mjs scripts/download-release.test.mjs` with Node 24, Bash and jq
after `npm ci`.
The suite executes publishing scripts and manifest commands with fake Docker,
GitHub and Git commands; it cannot upload assets or publish images. It covers
stable/candidate tag sets, both repositories' pinned notes, workflow gates,
malformed versions, digest validation, local prerelease flags and rejection of
workspace version mismatches. CI runs this suite on pull requests and releases.
