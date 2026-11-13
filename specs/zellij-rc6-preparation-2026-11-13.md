# Book starter and rc.6 preparation — 2026-11-13

Preparation base: `22335b742fce426105eae81f7b4104d7145e2f6f`, initially on
`codex/book-starter-rc6`. rc.5 remained published during preparation. The dated
closure entries below record subsequent rc.6 publication and retirement.

## Implementation

Book-classic 0.2.0 uses the shared versioned book manifest: generated title page
and contents, original PNG cover, preface, part, two chapters and appendix. The
same content builds native web output and optional English/LTR classic PDF.
PDF is disabled by default; the starter contains explicit A5/11pt settings.
The title and author are safely substituted into manifest text. Product
templates remain at 0.1.0 and previously published archives are immutable.

The catalog adds `pdf` output and `print: lualatex`. These require a portable
book and minimum CLI rc.6; legacy `print: unavailable` remains accepted.
Creation checks expanded PDF compatibility natively before publishing a
destination. It never installs or starts a runtime. Unsupported declarations,
missing book manifests and invalid print content fail without replacing files.

The Rust workspace/lockfile and standalone Docker recipe select rc.6. The
versioned release notes collect the composer, media accordion, presets,
diagnostic fixes, book model, optional PDF and configured preview/delivery work
merged since rc.5. Current guides disclose the pending release explicitly.

Hosted PDF acceptance now creates the actual bundled starter and captures its
whole PDF on Docker amd64/arm64 and rootless Podman. Template browser acceptance
checks cover/authors and front-matter/chapter/appendix navigation. Public-release
acceptance runs the tagged PDF driver against the downloaded binary on both
Linux architectures; older reading-only releases remain compatible with the
acceptance workflow. Only hosted runners execute containers/browsers.

## Verification history

The first filtered pass passed 13 template unit cases and one matching CLI case.
The full template CLI pass found one new assertion using the wrong missing-runtime
exit code (2 rather than the existing documented dependency code 4). The test
was corrected; no product behavior changed. One multi-file patch rejected an
outdated README context without modifying files and was reapplied with the
correct context. Comparing packages with rc.5 then exposed a Python gzip-default
change: unchanged product files produced different compressed bytes. Explicit
level 9 restores byte-identical product archives; `templates/published.json`
now guards known archive identities. The old public book-classic 0.1.0 catalog
was downloaded, created and built successfully with the new CLI.

The local workspace passed 770 Rust tests (five existing hosted-only cases),
formatting and warnings-denied Clippy. Strict/deep docs checks passed 59 routes
and 25 examples. Release helper checks exposed a positional assertion assuming
the acceptance driver was the last workflow step; it now finds the driver by
name, retaining its immutable-tag working-directory check after artifact upload
was added. All 33 release helper checks now pass, as do syntax/actionlint and
final deterministic local/bundled/catalog/offline template acceptance. The
existing product archives match rc.5 byte-for-byte under explicit gzip level 9.
The first hosted run `43482576387` produced the starter PDF, but all three
PDF jobs failed its exact title-text assertion. The new title wraps at the
paper width; acceptance now normalizes layout whitespace and separately checks
the exact metadata title. Starter artifacts are saved before this text check
so any further failure remains inspectable. The next run `43483044136` passed
all three PDF jobs; 12 PDFs and 87 page captures have been checked, including
visual review of every starter page. Its Windows starter test failed because
replacing the child PATH alone did not guarantee the expected no-runtime
diagnostic. Like the existing PDF fixture, the starter fixture now sets
conflicting runtime selectors to stop discovery before any runtime probe, and
asserts that specific dependency failure and previous-output preservation.
Final hosted acceptance is pending.

## Release gates and remaining scope

Finish local/native checks, review hosted browser/PDF artifacts, merge the exact
passing source, run the normal release preflight, and publish a new immutable
rc.6 tag. Then verify public assets, update Homebrew/current installation docs,
migrate release consumers and retire rc.5 only after replacement gates pass.
No live provider verification or production-domain migration is part of this
increment. Multilingual/RTL print, bibliography/index/glossary, numbered
equation/theorem semantics, richer templates and additional package-manager
channels remain open.

## Merged source and publication gate

PR #155 merged as `668e8faa47a7d4a435c71f63a51b903838a12f48`, with the same tree
as tested `70ca56442322b108310845d2755449b67b7df831`. All 23 jobs in `43484785719`
pass. The first Apple Silicon package attempt was cancelled before a runner was
assigned; GitHub explicitly reported macOS ARM64 capacity constraints. Its
unchanged-head retry and all four dependent Homebrew checks pass. The first
retry request was rejected because the workflow was still running, so the
specific job was retried after all other checks completed. No product fix was
needed for this infrastructure event.

All 12 final-head PDFs have valid checksums, shared content revisions and embedded
fonts. Their 87 page captures and all eight template browser captures match the
visually reviewed baseline. Hosted acceptance passed 490 engine tests, 356
showcase cases (124 conditional skips), all six native packages, original docs
route/heading/image preservation and portable Linux/macOS/Windows checks.

The release helper passed 770 Rust tests (five existing hosted-only cases), 490
engine tests, typechecks, schema drift, formatting and warnings-denied Clippy.
It pushed immutable tag `v0.2.0-rc.6` at the merge. Release workflow `43492620884`
was explicitly dispatched because the reviewed merge skipped duplicate push CI.
Its publication/public-download acceptance is pending; a tag alone is not a
verified public release.


## Public release closure — 2026-11-13

rc.6 is published from the unchanged merge/tag. Both release repositories have
20 byte-identical assets, checked against API digests, SHA256SUMS and the template
catalog. Product template archives remain byte-identical to rc.5; the newly
published book-classic 0.2.0 hash is now recorded in the immutable archive ledger.
The real public standalone rc.5-to-rc.6 upgrade passed in an isolated directory.

Release packaging needed one controlled retry after the Apple Silicon job waited
unassigned; all completed build jobs had passed. Publication then succeeded.
The original release run finished with 28 successful jobs and one PDF acceptance
failure: Linux ARM64 selected preinstalled rootless Podman, which lacked the CPU
cgroup controller. The Docker-discovery fallback cause was not recorded. No
release asset or tag was changed. Harness `3d26a96` selects/preflights the hosted
Docker endpoint explicitly, as the dedicated Docker PDF jobs already do.
All eight public jobs in `43496517451` now pass against the published rc.6 tag.
All eight public PDFs have matching extracted text, valid receipts/revisions and
embedded fonts. Their 58 page captures are pixel-identical to the reviewed source
layouts; binary PDF timestamps and PNG encodings differ across runs/platforms.

Homebrew commit `1555b7d` contains the exact attached formula and passes all four
public native install/upgrade/uninstall jobs in `43497080249`. Shared installer
docs are updated at `7cb742c`; installer files remain byte-identical to the tag.
The source documentation and six saved delivery workflows are retargeted in the
publication follow-up. Its CI, consumer images and scoped retirement remain
separate gates. No old distributions or CodeSeal assets were removed here.
See [release evidence](evidence/zellij-rc6-release-2026-11-13.json).


## Documentation, consumers and retirement closed

PR #156 (`8eb117a`) merges the exact tree tested at `f767260`, with all 23 CI
jobs and all six generated delivery build checks passing. The Next consumer
needed one unchanged-head retry after runtime-discovery timeouts; this remains
an explicit reliability follow-up. Browser preservation passes all 46 original
and 13 added routes, 282 original headings, 82 compatibility mappings and 29
byte-identical images without captured browser errors.

Both replacement image publications, all 13 development-image jobs and fresh
HTTP checks pass. The scoped cleanup removed two rc.5 releases, 40 assets, two
remote tags and 26 obsolete image versions. rc.6, CodeSeal, all current image
nodes and excluded history remain intact. Post-cleanup inventory and HTTP checks
pass. See [retirement evidence](evidence/zellij-rc6-image-retirement-2026-11-13.json)
and [implementation follow-ups](zellij-post-rc6-followups-2026-11-13.md).
