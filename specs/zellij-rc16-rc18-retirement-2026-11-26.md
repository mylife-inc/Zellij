# Scoped rc.16–rc.18 retirement — 26 November 2026

## Reviewed scope and preparation

The [read-only review](zellij-retention-review-2026-11-26.md) merged in PR #265
as `e4518f2`, with the same tree as qualified `ff78154`. All 37 executable CI
jobs passed on their first attempt. The corrected rc.19 inspection passed both
actual images. The hosted inventory independently matched the reviewed targets
and retained identities. The showcase had 552 passes, 169 conditional skips and
no retries; this does not establish a root cause for historical timing findings.

The [execution plan](evidence/zellij-rc16-rc18-execution-plan-2026-11-26.json)
is a distinct schema-3 document bound to the committed review's SHA-256. It
selects exactly six rc.16–rc.18 release IDs and tags in the two owned repositories,
240 assets and 72 image versions. Eight exact-source replacement/consumer
qualification runs must still pass. All 18 retained releases, 28 other tag
references, 226 protected manifest nodes and 4,659 unreferenced historical
image versions remain protected. Every removed template archive has an identical
copy in a retained release. The reviewed asset total is 2,044,866,250 bytes,
not a measurement of physical registry storage reclaimed.

The new executor refuses the read-only review and the completed rc.12 plan.
It checks unchanged repository heads, full retained release inventories, target
asset/tag identities, all image references and retained-manifest graphs. Apply
requires the intended owner, package deletion scopes, a clean committed checkout,
the exact plan hash and a successful hosted guard-test run from that checkout.
It waits for active workflows to finish rather than racing publication.

A journal must be writable before mutation. Each DELETE uses an exact reviewed
release, tag or image-version identity, re-read immediately before removal.
Partial API failure and a lost deletion response are resumable against the same
plan and source. Changed identities or a missing protected artifact stop execution.
Post-verification requires every target to be absent and every excluded identity
to match. These checks do not claim an atomic transaction across GitHub services.

The inventory workflow's default `inventory` mode now collects raw state without
requiring obsolete versions to remain available. `review` additionally generates
the read-only review and preflights the committed execution plan; `verify-retained`
checks the exact expected post-cleanup state. The historical rc.12 workflow path
remains separate and cannot consume the new plan. Apply is local-owner-only.

Preparation verification: 14 new executor tests cover immutable releases, wrong
owners/scopes, source qualification, moved tags, changed assets, protected/shared
manifests, unreferenced history, journal failure, partial failure, lost responses
and idempotent resumption. Actions syntax is checked with Actionlint. Execution
and fresh-pull/download results will be recorded after their actual completion.
