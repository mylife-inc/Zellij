# Distribution retention review — 26 November 2026

Outcome: the subsequent [scoped retirement](zellij-rc16-rc18-retirement-2026-11-26.md)
is complete. The following dated review preceded that execution.

Status at review: read-only, with a verified correction to the image inspector.
No release, tag, image version or source archive has been deleted. Rc.24 remains
the published and installed candidate. Existing user authorization covers a
reviewed pre-1.0 cleanup; this record does not request another permission grant.

## Confirmed verification gap and correction

The rc.19 publication configurations and generated workflows selected rc.19,
but `scripts/fixtures/release-delivery/cases.json` still selected rc.18 for
inspection. The generator comparison passed despite that discrepancy. The logs
of run `45011278526`, jobs `137173935593` and `137173935762`, confirm that its
successful fresh-pull checks actually exercised rc.18. Those results remain
valid evidence for rc.18, not rc.19; earlier records are annotated accordingly.

Fix `45eb4fa` generates the inspection target from the same configuration and
checks it against the generated publication step. The metadata also records the
configuration's SHA-256. The dependency-free inspector verifies that hash before
pulling and prints its selected reference. A configuration edit without
regeneration now fails early. The normal CI generator check rejects divergent
metadata or publication targets; `--write` refreshes both workflows and metadata.

The original comparison passed with the stale rc.18 metadata. The corrected
comparison rejects it; after regeneration all six workflows compare exactly.
Six HTTP/inspection tests pass, covering stale metadata, changed configuration,
wrong generated publication targets and the existing HTTP/content checks.

Fresh run [45921906069](https://github.com/mylife-inc/Zellij/actions/runs/45921906069)
at `45eb4fa` passes both corrected image jobs, including actual pulls, nonroot
users, health checks, pages, assets and source-file isolation. The provider job
is skipped. The verified rc.19 digests match the inventory:

- Static: `sha256:36eeb7d29d375b9d71ab27f25dd7d2181f274ab1c2ee2e9fbcd2c2f99b314220`.
- Next: `sha256:fbc1d54d9d73acd93a3acfb1b335e3c48647e33496d2dd61e3034f2e91d0c1c7`.

## First cleanup group

Inventory [45921020469](https://github.com/mylife-inc/Zellij/actions/runs/45921020469)
at `dad1fca` succeeds with no inventory errors and no deletion job. It records
11 Zellij releases in each owned release repository, 4,914 base-image versions,
22 static acceptance-image versions and 21 Next acceptance-image versions.

The [exact review](evidence/zellij-retention-review-2026-11-26.json) identifies:

| Scope | Candidate removals |
| --- | --- |
| Native releases | rc.16, rc.17 and rc.18 in both owned repositories: six releases, six release tags and 240 assets |
| Published asset bytes | 2,044,866,250 bytes across the two repositories, approximately 1.90 GiB; this is asset size, not a measured registry storage reduction |
| Base images | 54 exact version identities reachable only from those candidate tags |
| Acceptance images | Nine static and nine Next version identities |
| Total image versions | 72, excluding every dependency of a retained tag |

The review preserves 226 manifest nodes reachable from retained tags and all
4,659 unreferenced build-history versions. It verifies fetched manifest digests,
walks child/subject references, protects shared dependencies and unknown tags,
and records exact target IDs, digests and tags. Remaining-image fingerprints
allow a later executor to verify that excluded identities are unchanged.
Release target identities, asset inventories, tag objects and mutable-release
status were rechecked independently. Unrelated releases in the shared repository
are recorded for preservation.

## Retention reasons

| Version or scope | Disposition |
| --- | --- |
| rc.24 | Current qualified distribution and owned package channels |
| rc.21 | Actual previous-version channel-upgrade input; retain its matching PDF source/runtime package too |
| rc.20 | Published PDF runtime and corresponding-source pair; no partial source/package deletion in this group |
| rc.19 | Active generated delivery, provider and image fixtures; corrected image inspection now proves the intended images |
| rc.15 | Current Microsoft catalog submission still references these exact public archives; catalog approval remains separate |
| rc.14 | Real WinGet upgrade fixture in `scripts/check-winget.mjs` and the repository-owned script-compatibility fixture |
| rc.11 and rc.13 | Previously recorded compatibility retention remains in force for this group. Public guides still contain matching-rc.11 instructions. No current executable rc.13 pin was found in the tracked-source scan; that alone does not settle the earlier compatibility constraint. |
| rc.22 and rc.23 | Keep immutable failure tags and image identities; neither published a native CLI release |
| Main, unknown tags, shared manifests, unreferenced history | Preserved; this is not a general registry sweep |
| Template identities and unrelated products | Preserve the immutable template ledger and all unrelated release identities |

The source scan distinguished executable download/image pins from historical
version prose and parser fixtures. A version mention alone does not prove an
active download dependency. Existing evidence and source history are retained.

## Reproduction and execution boundary

`node scripts/review-release-retention.mjs inventory.json review.json` makes
only registry reads and writes a local schema-2 review. It has no apply mode,
DELETE operation or token with write permissions. The inventory workflow runs
this review and its guards; the frozen historical rc.12 executor cannot consume
it. The source-only inspector fix requires no new CLI release.

Local validation passes 28 JavaScript tests (inspection/HTTP, existing image
safeguards and new review safeguards), 13 native-retirement guard cases,
Actionlint and all six generated-workflow comparisons. Tests exercise shared
children, unknown tags, duplicate identities, missing retained data, exact
version matching and stable remaining-identity fingerprints.

Next: implement and test execution guards for this exact group, refresh the
inventory and repository heads, recheck replacement/consumer gates, commit the
execution plan, then apply the already-authorized cleanup. Afterwards compare
all retained identities and fresh-pull the retained images. Never feed this
review into the spent rc.12 executor or treat a read-only review as a completed
retirement. No provider account or deployment verification is resumed here.
