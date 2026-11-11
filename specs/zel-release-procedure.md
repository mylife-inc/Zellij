# Zellij release channels and candidate procedure

Implemented policy, November 2, 2026. This document does not announce a release.
Candidate 0.2.0-rc.3 is being prepared under the user's 2026-11-11 publication
instruction. Remaining live provider/external-runner checks are deferred until
after this release; normal regression and artifact checks still apply.
Candidate 0.2.0-rc.2 is published and its public package/image acceptance passed.
Publication status must be checked in GitHub; a version change in source is not
proof of an uploaded release.

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
| `main` branch | `main` | None | Unchanged |
| `v0.2.0-rc.1` example | `0.2.0-rc.1` only | Prerelease, never latest | Unchanged; docs still built |
| `v0.2.0` example | `0.2.0`, `0.2`, `latest` | Stable | Installer scripts updated; docs deploy only when enabled and credentials exist |

All four image variants follow this rule: no suffix, `-maths`, `-diagrams`,
and `-all`. Container versions omit the leading `v`. Release assets in this
repository use `v…`; the shared `mylife-inc/releases` repository uses `zellij-v…`.
Branch workflow dispatches only publish when run against `main`.

`scripts/release-policy.mjs` supplies version validation, image tags and pinned
release notes to CI and the local publishing helpers. Release notes download the
installer attached to that specific release and explicitly select its version
and asset directory. Candidates cannot replace the shared installer's `main`
files, enable production documentation deployment or become GitHub's latest
release. Both Unix and PowerShell instructions are generated from the same policy.

This policy governs future publications. It does not retag existing images:
older branch workflows already wrote `latest`. Use an exact matching image tag
for candidate acceptance. Source development can explicitly select
`ZELLIJ_IMAGE=ghcr.io/mylife-inc/zellij-base:main`; `main` remains mutable.

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
4. From the reviewed, clean commit, run `source scripts/app.sh`, followed by
   `zrelease v0.2.0-rc.3` (substitute the actual agreed candidate). The helper
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
in both release pages. The rc.3 notes are specs/releases/v0.2.0-rc.3.md;
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
`main` and provide its published `version` tag, for example:

```sh
gh workflow run release-acceptance.yml --ref main -f version=v0.2.0-rc.1 -f shared=true
```

The workflow checks out `refs/tags/<version>` separately. Version, engine metadata,
installer byte comparisons and the native acceptance suite use that tagged source;
only the download/acceptance driver comes from the selected workflow revision.
This performs public reads and temporary local installs on the runners. It does
not rebuild, republish or move a release tag. Record both the source tag and the
acceptance workflow revision when reporting this verification.

## Stable promotion

After candidate acceptance, commit the stable workspace version and run the
same review/CI process before creating its new stable tag. The release workflow
builds and publishes that version; it does not turn a candidate tag into stable.
Stable production docs require `DEPLOY_DOCS=true`, `NETLIFY_AUTH_TOKEN` and
`NETLIFY_SITE_ID`. A candidate never deploys production docs even when all three
are present. Coordinate docs publication with the corresponding native release
so the website does not advertise commands absent from the stable executable.

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
