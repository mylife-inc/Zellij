# Book starter and rc.6 preparation — 2026-11-13

Base: `22335b742fce426105eae81f7b4104d7145e2f6f`. Current work is on
`codex/book-starter-rc6`. Published rc.5 remains unchanged; version changes in
source are not publication evidence.

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
so any further failure remains inspectable. Final hosted acceptance is pending.

## Release gates and remaining scope

Finish local/native checks, review hosted browser/PDF artifacts, merge the exact
passing source, run the normal release preflight, and publish a new immutable
rc.6 tag. Then verify public assets, update Homebrew/current installation docs,
migrate release consumers and retire rc.5 only after replacement gates pass.
No live provider verification or production-domain migration is part of this
increment. Multilingual/RTL print, bibliography/index/glossary, numbered
equation/theorem semantics, richer templates and additional package-manager
channels remain open.
