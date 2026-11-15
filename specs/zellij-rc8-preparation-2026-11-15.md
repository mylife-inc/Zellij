# CLI rc.8 preparation — 2026-11-15

Baseline `ec463edfda3059314cd68dea0fdec3a61bc098f7`, after technical/study
editions and Next-image PDF delivery. Public rc.7 remains unchanged during
preparation. The native workspace and standalone image recipe already select
rc.8; this increment does not change the renderer, templates or toolchain.

## Scope

Collect the two book starters and Next-image PDF delivery in versioned release
notes. Extend the existing two-architecture PDF-image acceptance workflow with
an optional published-tag mode. The normal source mode keeps building the CLI
and engine from the checkout. Public mode checks out the exact released source,
installs the checksum-verified public CLI, pulls its exact engine tag anonymously,
and executes the tagged PDF/browser driver. It never rebuilds a replacement CLI
or engine. A receipt retains the native archive hash and resolved image digests.

Public verification rejects mismatched binary versions, wrong image architecture
and missing immutable image identities before running PDF tests. Driver failures
remain failures, with downloaded identities retained and owned temporary installs
cleaned. Auth tokens and checksum-bypass settings are removed from child commands.
The existing release checks still verify every bundled/published/cached template
and compare the source and shared release artifacts. Older releases without the
Next PDF driver explicitly skip this new capability while retaining their
existing native/template/image checks.

## Gates and release boundaries

Local helper checks and full hosted source CI precede merge. Reviewed tree
identity and normal release preflight precede an immutable rc.8 tag. All native
packages, Homebrew/Scoop lifecycle checks, exact image variants and public-download
acceptance must pass before treating the candidate as ready. The release now
includes two extra public Next PDF jobs and two new template archives.

After publication, update the owned package channels, active documentation and
consumer pins; verify actual rc.7-to-rc.8 upgrades before scoped rc.7 retirement.
Do not reuse published tags or rewrite known template archive hashes. Preserve
CodeSeal and unrelated artifacts. Record results and failures as observed.
Local browsers, containers and real LaTeX remain delegated to hosted runners.
Live providers and production-domain migration stay deferred. Remaining print
and installation work, and unexplained browser timings, retain their status.
Preparation is not publication and passing checks do not prove zero bugs.

## Verification

Local verification passes all 100 helper tests, including the 42 release-focused
cases. Actionlint, JavaScript syntax, matching native/image version and diff
formatting checks pass. Deterministic packaging produces five template archives;
the three existing published archives retain their bytes and versions.
Regression tests exercise pinned public installation, token removal, binary/image
identity checks, failure propagation/cleanup and workflow separation of source
versus public mode. An initial test assumed every acceptance job contained steps;
it now checks the reusable workflow's concrete job too, retaining version-input
and immutable-checkout assertions for all three execution paths. Its first
failure log is retained in `/private/tmp/zellij-rc8-first-helper-tests.log`.
All 29 source CI jobs in `43694035458` pass at `0afb807`, including both
Next PDF architectures. The 34 PDFs retain all 312 reviewed page layouts and
30 TeX sources. Showcase passes 378 cases without retries (140 conditional
skips); preservation retains 46 original and 13 added routes, 282 original
heading IDs, 82 mappings and 29 byte-identical images without browser errors.
The actual rc.7 compatibility dispatch `43695181909` passes both architectures,
explicitly skipping the unavailable Next PDF capability. This does not claim
that rc.7 supports Next PDFs. See [verification evidence](evidence/zellij-rc8-preparation-2026-11-15.json).

Normal release preflight, publication, public-download checks, channel updates,
consumer migration and scoped retirement remain pending. The final records-only
update changes no implementation, workflow or runtime inputs from tested `0afb807`.


## Publication follow-through

The immutable tag is `d70e10f`. All 35 release jobs, public PDF comparisons,
managed-channel lifecycles and actual rc.7-to-rc.8 upgrades pass. PR #183 merges
updated guides/consumer pins after 29 CI jobs and six consumer checks pass.
Scoped rc.7 retirement and post-cleanup inventory/HTTP checks are complete.
See [release evidence](evidence/zellij-rc8-release-2026-11-15.json) and
[retirement evidence](evidence/zellij-rc8-image-retirement-2026-11-15.json).
The earlier pending statuses above describe the preparation checkpoint.
