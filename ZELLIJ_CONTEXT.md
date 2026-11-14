# Zellij — durable project context

Last refreshed: **2026-11-14**. Initial review baseline: local `main` at
`6356b05deaddf5421b24d7c8a1e30f868d4923c0`.

PR #171 (`e58133b`, matching reviewed tree `11b3c55`) merges numeric bibliography
and citations from local CSL-JSON, with one registry for portable HTML, adapted
Next and PDF. All 27 jobs in `43578754363` pass at `d9e22cc`; local verification
passes 789 Rust and 490 engine tests. Twenty-one PDFs/156 page captures pass, with all 129 prior layouts
unchanged. All 26 new browser captures are retained; desktop/mobile Next and
native Linux/macOS/Windows samples are visually reviewed. The documented format
supports numeric citations, locators, footnotes, DOI/URL links and uncited entries;
BibTeX and custom citation styles remain open. No PDF package or image-size
change is required. See [the contract](specs/zellij-book-bibliography-2026-11-14.md)
and [evidence](specs/evidence/zellij-book-bibliography-2026-11-14.json).
The final documentation/evidence update changes no runtime, test or workflow
inputs; its public-guide URL wording passes strict/deep validation separately.
All 13 development-image jobs in `43581129613` pass at the merge, publishing all
four `main` variants on amd64/arm64 and building/serving a site from the published
default image. The final record-only follow-up changes no runtime, test or
workflow inputs. Public rc.6 is unchanged.
**Next priority:** index/glossary, followed by richer book editions and the
remaining print and installation work. The separate unresolved browser timings
and deferred provider verification retain their existing status.

PR #169 (`cd79114`, matching reviewed tree `5ab8acf`) merges shared book
theorem/lemma/proof semantics (and proposition, corollary, definition, example
and remark), with one statement counter per reading entry and a shared label namespace with equations. Native HTML, adapted
Next and PDF use the same resolved numbers, targets and references. All 27 jobs
in `43567230812` pass at `5dfaf59`; local verification passes 782 Rust and
490 engine tests. Eighteen PDFs/129 page captures pass; all 105 prior layouts
are unchanged. Authored home content now starts after the PDF contents and
resets the inherited running header. The record retains both native-fragment
test-readiness failures and the bounded original/revised comparison (6/30
failures versus 30/30 passes), without claiming a product focus-script fix.
See [the contract and verification record](specs/zellij-book-statements-2026-11-13.md)
and [evidence](specs/evidence/zellij-book-statements-2026-11-13.json).
All 13 development-image jobs in `43569715226` pass at the merge, publishing
all four `main` variants on amd64/arm64 and building/serving from the published
default image. The final record-only follow-up changes no runtime or test inputs.
Public rc.6 remains unchanged.

PR #164 (`cc88bc0`) resolves the reproduced rapid-resize focus race, with the
same tree as tested source `36c7882`. Hidden blur is remembered when it happens;
a pending restoration survives later breakpoints and respects subsequent focus
or pointer choices. Final hosted stress `43541961094` passes 1,156 cases without
retries (64 conditional skips). Its revised delayed-frame fixture still fails
all eight navigation cases against the old engine. All 27 source CI jobs in
`43540700668` pass after one unchanged Engine rerun; the first hydration-readiness
timeout is retained. The fixture follow-up keeps native rendering active while
application callbacks wait and adds the failed page URL to future readiness
errors. All 13 development-image jobs in `43543832220` pass at the merge,
publishing all four `main` variants on Linux amd64/arm64 and building/serving
from the published image. See
[the resolution and evidence](specs/zellij-rapid-focus-restoration-2026-11-13.md).

Scoop is published in the owned [`mylife-inc/scoop-bucket`](https://github.com/mylife-inc/scoop-bucket)
at `ca62345`, installing immutable rc.6 on native Windows x64/ARM64. All four
public lifecycle jobs in `43532235784` pass under Windows PowerShell 5.1 and
PowerShell 7, including actual HTTPS installation. PR #162 (`fd5f07a`, matching
the tested tree at `7788140`) merges a checksum-pinned manifest generator, exact `scoop update zel` guidance, native
lifecycle/release gates and a future `zel.json` release artifact. Its four source
Scoop jobs pass at `7788140`; all 27 jobs in `43531633137` pass without workflow
reruns. The unrelated browser retry is recorded below and in the evidence.
rc.6 keeps its generic ownership message and all 20 assets unchanged. The new
message ships in the next candidate. See [the Scoop contract](specs/zel-scoop-distribution-2026-11-13.md)
and [evidence](specs/evidence/zellij-scoop-distribution-2026-11-13.json).

PR #167 (`fe0f82c`, matching reviewed tree `168c903`) merges shared book
equations verified at `09fd2cc`. One registry in
`cli/crates/zellij-docs/src/equations.rs` supplies chapter/appendix numbers,
stable anchors and standalone `\eqref`/`\ref` links to native HTML, adapted
Next and PDF. All 27 final-source jobs in `43557146014` pass, including offline
Chromium/WebKit on Linux/macOS/Windows, prefixed Next images, six native packages,
four Homebrew and four Scoop combinations. Fifteen PDFs/105 page captures pass;
all 87 prior layouts remain unchanged. Local validation passes 778 Rust and
490 engine tests. The record retains the Windows CRLF fixture correction, the
fixed colon-in-fragment/query adapter defect and the fixed TeX token-boundary
defect. See [the contract and verification](specs/zellij-book-equations-2026-11-13.md)
and [evidence](specs/evidence/zellij-book-equations-2026-11-13.json).
All 13 development-image jobs in `43560554366` pass at the merge, publishing
all four `main` variants for amd64/arm64 and building/serving a site from the
published image. The final evidence follow-up changes only context and records.
At that equation milestone, bibliography/citations were next; PR #171 above
implements numeric citations. Index/glossary, richer book templates and remaining
installation channels follow.
These equation additions await the next candidate; public rc.6 is unchanged.
The bounded investigation of the separate WebKit overflow and docs
hydration timings passes 120 overflow repetitions and 40 full documentation
annotation checks without retries in `43547198588`. Neither failure reproduced;
both remain unexplained. Source `f70e942` adds bounded failure-only browser
state capture and CI artifacts. Hosted `43549176065` passes both deliberate
capture checks, 378 ordinary showcase cases (140 skips, zero retries), full
documentation preservation and all six browser regressions. See
[the investigation](specs/zellij-browser-timing-investigation-2026-11-13.md).
Do not infer a timing fix from later passes or block unrelated feature work on
indefinite repetition. Retain these findings when assessing the next candidate,
alongside the [resolved rapid-focus defect](specs/zellij-rapid-focus-restoration-2026-11-13.md).
Live provider verification and production-domain migration stay deferred.
Public rc.6 is unchanged.

PR #160 (`dece6c9`) fixes two reproduced header-focus defects; its tree
matches tested head `17be3af`. Focus lost when breakpoint CSS hides navigation
is restored after panel closure commits, and header updates preserve the focused
drawer link through a stable close callback. New tests cover reciprocal/rapid
resizes, content focus and drawer scrolling; desktop WebKit now runs header
coverage in regular CI. Local 490-test/typecheck/build/doc checks pass. Hosted
no-retry stress passes 252 targeted and 636 full-workload cases (64 conditional
skips). All 23 CI jobs in `43520478220` pass without workflow reruns, including
370 showcase cases (132 conditional skips, no flaky cases), six native packages
and four Homebrew targets. All 13 development-image jobs in `43524812741`
pass on the merged source. The original failures and the corrected test-readiness
gap remain in [the focus record](specs/zellij-header-resize-focus-2026-11-13.md)
and [evidence](specs/evidence/zellij-header-focus-2026-11-13.json). Public rc.6 is
unchanged at that milestone. The Scoop publication and new rapid-resize finding
recorded above are subsequent work; the original fixes and evidence remain valid.

PR #158 (`dd0f620`) fixes generated container CI selection after the recorded
rc.6 discovery failures; its tree matches tested head `bd0ae9b`. A shared requirement
covers hosted rendering, enabled PDF, static/Next images and multi-architecture
publication. GitHub/CircleCI select the default Docker context; GitLab preserves
its TLS service endpoint. All container jobs preflight Docker with a bounded
60-second check and five-second termination grace before build/login; portable
HTML and disabled PDF need no runtime. Local checks pass 37 scaffold and 31 CLI
workflow tests, formatting, warnings-denied workspace Clippy, six generated
workflow comparisons, Actionlint and strict/deep documentation validation.
All 23 jobs in CI `43508089027` and six generated consumer workflows pass at
that head, without workflow reruns. Hosted verification includes 774 Rust tests
(five hosted-only cases run separately), 490 engine tests, both native image
architectures and combined manifests, three PDF runtime jobs, six native
packages and four Homebrew checks. Source-generated consumer workflows execute
public rc.6; the released binary/tag are unchanged. See
[verification evidence](specs/evidence/zellij-ci-runtime-selection-2026-11-13.json).

The same generator CI recorded 355 showcase passes, one flaky header-resize
focus assertion and 124 conditional skips. PR #160 subsequently reproduces and
resolves that finding, as recorded above; the original failure remains preserved.

PR #155 (`668e8fa`) merges book-classic 0.2.0 and rc.6 preparation, with the
same tree as tested head `70ca564`. All 23 final-head CI jobs in `43484785719`
pass. GitHub cancelled the first unassigned Apple Silicon runner for capacity;
its unchanged-head retry and all four Homebrew checks pass. Local release
preflight passed 770 Rust tests, 490 engine tests, typechecks, schema drift,
formatting and Clippy. Twelve PDFs and all 87 page captures pass, with all eight
mobile/desktop template captures matching visual review. The immutable rc.6 tag
points at this merge. Release workflow `43492620884` published rc.6 to both
repositories with 20 byte-identical assets. Public Linux ARM64 PDF acceptance
selected unconfigured rootless Podman and failed its CPU resource controller;
`3d26a96` explicitly selects/preflights hosted Docker. Separate public acceptance
`43496517451` passes all eight jobs against the unchanged release. All eight
public PDFs have matching text and embedded fonts; all 58 page captures are
pixel-identical to the reviewed source layouts. Homebrew `1555b7d` passes all
four public install/upgrade/uninstall jobs in `43497080249`; mirror installation
docs are at `7cb742c`. The actual standalone rc.5-to-rc.6 upgrade passed.
See [rc.6 evidence](specs/evidence/zellij-rc6-release-2026-11-13.json).
PR #156 (`8eb117a`) merges the current guides and consumer pins, with the exact
tree tested at `f767260`. All 23 CI jobs and six delivery build checks pass.
The Next consumer required one unchanged-head retry after runtime probes timed
out before building. This prompted the runtime-selection fix in PR #158 above;
the original probe delays remain unexplained. All 13 development-image jobs in
`43498926083` pass at the released source; the docs follow-up changes no runtime
inputs. Replacement static/Next publications and fresh-pull HTTP checks pass.
rc.5 retirement is complete: two releases, 40 assets, two remote tags and 26
obsolete image versions removed. All 46 current manifest nodes, 3,321 excluded
build-history versions and CodeSeal releases are intact. Fresh hosted inventory
`43505203415` and both HTTP checks in `43505207912` pass. See
[retirement evidence](specs/evidence/zellij-rc6-image-retirement-2026-11-13.json).
Do not reuse completed plans after tags move. See the
[runtime-selection resolution and next priorities](specs/zellij-post-rc6-followups-2026-11-13.md).
See [the preparation record](specs/zellij-rc6-preparation-2026-11-13.md).

PR #153 (`71b97e4`) merges saved PDF settings, PDF-aware preview/watch and
static delivery/images, with the same tree as tested head `ecfbfba`. All 23 jobs
in `43376564097` pass without reruns: 767 local Rust tests, 490 engine tests,
356 showcase cases (124 conditional skips), six native packages and four
Homebrew targets. Real Docker amd64/arm64 and rootless Podman checks produced
nine valid PDFs; all 66 captures match the reviewed A5/A4/Letter layouts.
Configured PDF static images are about 7.2 MB amd64 / 14.0 MB arm64 uncompressed,
without LaTeX/Node/Rust. See [the checkpoint](specs/zellij-pdf-configuration-2026-11-12.md)
and [evidence](specs/evidence/zellij-pdf-configuration-2026-11-12.json).
At this configuration milestone rc.5 remained published. PR #155 subsequently
versioned the starter and published rc.6; see the current status above.

PR #149 (`72a8445`) merges the versioned book publication model, with the same
tree as tested head `2b5cfec`. Its portable manifest drives ordered front matter,
parts, chapters and appendices, metadata, covers, a content revision and shared
JSON inspection; the same model feeds the Next adapter. All 20 final-head jobs
in `43322268842` passed: 756 Rust tests (five existing ignored cases run separately),
490 engine tests, 356 showcase cases (124 conditional skips), six native packages
and four Homebrew targets. Actual book CLI and relocated-file Chromium/WebKit
checks pass on Linux/macOS/Windows; the Next-image book checks pass. Twelve Linux
and two Windows captures were visually reviewed. See
[the book checkpoint](specs/zellij-book-publication-2026-11-12.md),
[evidence](specs/evidence/zellij-book-publication-2026-11-12.json) and
[authoring guide](examples/zellij-docs/content/02-cli/books.mdx). PDF/LaTeX execution
was still pending at this book-model milestone; its first pipeline closure follows below. At that milestone rc.5 and the production domain were unchanged; these book
fields subsequently shipped in rc.6.
All 13 development-engine jobs in `43328363366` passed, publishing the four
`main` variants for Linux amd64/arm64 and building/serving a site from the image.

PR #151 (`c609251`) merges the first optional PDF pipeline, with the same tree
as tested head `de92de2`. `zel build --pdf` publishes web/PDF/download links
atomically; `zel export pdf` / `zel pdf` exports separately, and `pdf --check`
checks compatibility natively. `cli/crates/zellij-docs/src/pdf.rs` owns the direct
Markdown-event adapter; `cli/crates/zellij-cli/src/pdf.rs` owns isolated execution.
The optional digest-pinned LuaLaTeX image does not enlarge normal website images.

All 23 final-head jobs in `43358560394` passed, including actual Docker PDF
execution on Linux amd64/arm64 and rootless Podman on Linux amd64. All 24 final
captures match eight visually reviewed pages; sample PDFs are 51,193 bytes with 13
embedded fonts. Local verification passes 762 Rust tests (five existing hosted
cases run in CI), Clippy and strict/deep docs checks. All 490 engine and 356
showcase tests, six native packages and four Homebrew targets pass. The Engine
job needed one unchanged retry after a legacy-bookmark viewport timeout; its
cause is not established, and the original failure is retained. Rootless mapping,
heading placement and an unrelated watcher-fixture race were fixed before merge.
See [the PDF checkpoint](specs/zellij-book-pdf-2026-11-12.md),
[evidence](specs/evidence/zellij-book-pdf-2026-11-12.json) and
[guide](examples/zellij-docs/content/02-cli/books.mdx).

At the first PDF milestone, print was English/LTR A5 and saved configuration
and PDF-aware preview/delivery were still open. PR #153 closes those two gaps
and adds A4/Letter and 10/11/12pt settings. Multilingual print,
bibliography/index/glossary, numbered equation/theorem semantics and richer book
templates remain open.
At that PDF milestone rc.5 and the production domain were unchanged. No development
engine refresh was needed: Node runtime inputs are unchanged (one test fixture
changed); the optional PDF toolchain is separate.

For the current completion gates, use [the concise completion ledger](specs/zel-cli-completion-2026-11-11.md).

Company identity confirmed by the user on 2026-11-11: **Shebka** makes Zellij.
The company was originally called MyLife and was renamed because of domain-name
availability. The user owns the existing GitHub account **`mylife-inc`** and
explicitly accepts keeping that namespace for Zellij and Homebrew. The live
installation command is **`brew install mylife-inc/tap/zel`**, selecting the
current rc.6 preview. Use Shebka for company-facing branding and retain `mylife-inc`
for existing repository/distribution identifiers. A GitHub account rename or
move to a `shebka` namespace is not needed for this plan.

Historical rc.5 distribution baseline: **v0.2.0-rc.5 was published** from PR #138 merge
`a92c2e9f9d82abf5b75ea31bd825117cf86f9ec7`, with the same tree as tested head
`3a9f80771b48ffae3f57e8fa3d8be870848db889`. All **20 final-head CI jobs** in
`43180055105` and **29 release jobs** in `43185514003` passed. Source and public
mirror each contain 20 identical assets, including the versioned template catalog
and three archives. Homebrew commit `e0e8c31` passed all four native acceptance
jobs in `43187271603`; public mirror installation docs are at `0ddf790`.
See [rc.5 evidence](specs/evidence/zellij-rc5-release-2026-11-12.json).

Consumer migration PR #139 (`7430844`) passed all six installed-CLI build checks
without provider deployment. The Next image job passed on its sole unchanged-head
retry after container discovery timed out; the original failure is recorded.
Both replacement acceptance images, all 13 development-engine jobs and fresh-pull
HTTP checks passed. Exactly two rc.4 release entries, 32 assets and two remote
tags were retired; rc.5 and both CodeSeal releases retain their original asset
identities/checksums. Preview upgrade lookup then selected rc.5. See
[native retirement evidence](specs/evidence/zellij-rc5-native-retirement-2026-11-12.json).

Historical rc.4 distribution baseline (superseded by rc.5): **v0.2.0-rc.4 was published** from PR #135 merge
`6d116b2ca6df8eb8d078f03978d2a49598b58de1`, with the same tree as tested head
`6a2601c`. All **20 final-head CI jobs** and **29 release jobs** passed. The
release includes company branding, the hydration fix, `zel upgrade --preview`,
Homebrew ownership protection and default engine images matching the CLI version.
The public [Homebrew tap](https://github.com/mylife-inc/homebrew-tap) is live at
`5abad25`, with all four native Mac/Linux install, completion, ownership,
portable-build, revision-upgrade and uninstall jobs passing. The source and
public mirror have 16 identical release assets. See
[rc.4 evidence](specs/evidence/zellij-rc4-release-2026-11-11.json).

Delivery pins migrated in PR #136 (`ea9d61b`): all six installed-CLI build checks
passed without deploying to providers. Replacement static/Next acceptance images,
all four refreshed `main` engine variants and fresh-runner image HTTP checks pass.
Shared installer files/docs now match tagged rc.4 source (`mylife-inc/releases`
commit `d83dd7d`). Stable lookup remains explicit: with no stable release retained,
it reports that none exists and directs users to preview/exact-version selection.

Nine superseded GitHub releases, 113 assets and nine remote release tags were
retired only after these gates passed. rc.4 and both CodeSeal releases are intact;
source and audit history remain. The separate
[retention record](specs/zellij-release-retention-2026-11-11.md) tracks exact image
cleanup and its bounds. **Scoped image retirement is complete.** Exactly 213
obsolete versions were removed: 195 base, nine static and nine Next. Current
rc.5/main tags and all 46 manifest dependencies remain intact; the 3,229
unreferenced build-history versions are unchanged and remain outside this scope.
Fresh hosted inventory `43281128886` has zero retired tags or remaining targets;
both fresh-pull HTTP checks in `43281136218` pass. All 40 rc.5 assets and 12
CodeSeal assets retain their original identities and checksums.

The two historical rc.4 workflow attempts and refreshed rc.5 attempt
`43276711406` stopped with HTTP 404 before any deletion, including after the
user-confirmed package Admin grant. PR #145 (`637f2ee`) adds the explicit local
owner fallback with the same plan checksum, identity and dependency gates;
13 guard tests pass locally and hosted. The user refreshed package scopes.
Owner attempt 1 removed 67 versions before an isolated GET returned HTTP 401.
The unchanged login and exact GET then succeeded; attempt 2 rechecked every gate,
skipped those 67 absent identities and removed the remaining 146. Both journals
are retained, and their union matches the 213-version plan exactly once.
See [image retirement evidence](specs/evidence/zellij-rc5-image-retirement-2026-11-12.json).
The historical 187-version rc.4 plan is stale and must not be executed.
Do not reuse any retired version/tag.

The template milestone is merged and released in **rc.5**: `zel create` /
`zel new site` and `zel templates [show]`. `templates/` owns the bundled versioned
catalog, separate from `examples/`. The CLI module
`cli/crates/zellij-cli/src/templates/` loads bundled/local/checksummed catalogs,
validates and stages creation, records `.zel-template.json`, and supports an
offline cache. `templates/README.md` is the format/limits contract. Initial
starters: product-docs, product-studio and book-classic, all version 0.1.0.
Release archives/catalogs are identical across source and public mirror; public
installed-binary acceptance covers both and offline cache reuse. Six hosted
mobile/desktop previews were visually reviewed. The book publication model is
included in rc.6 (PR #149 above), followed by the first optional PDF/LaTeX
pipeline (PR #151) and book-classic 0.2.0 (PR #155). Technical/study editions and richer print semantics remain open. Released template archives remain immutable.

Review fixes: `c370987` rejects ancestor directory capitalization collisions;
`6810aee` corrects six card bodies lost through wrong YAML keys and guards official
templates against stripped fields; `3a9f807` aligns the standalone Docker recipe
with the CLI version. Historical rc.5 silently strips unknown section
fields. PR #141 (`3008a16`), included in rc.6, addresses that
in the engine: discarded fields warn in referenced YAML/Markdown sections,
examples, legacy inline sections and page overrides; strict checks fail. Field
paths include nested arrays, while diagnostic values stay private. Normal builds
continue, and defaults/coercions/retained extension fields remain accepted.

The stronger check exposed 13 omissions in the CodeSeal example. Source now
renders `eyebrow` for feature-grid/steps-panel/FAQ and an inline-Markdown FAQ
`subhead`, and quotes a comma-bearing hero label. Shared CLI schema and guides
are updated. Local checks pass: 471 engine tests, 12 native authoring cases,
23 schema authoring cases, all three site templates and all four authored sites.
The first hosted run (`43192044415`) also exposed four ignored fields
in the older native starter: its feature heading used `title`, and three card
bodies used `description`. The generator now emits `heading`/`body`. All 35
scaffold tests, four full-starter tests and nine site-authoring tests pass; real
engine/SSR acceptance verifies 40 native starter variants, ten Node starters,
ten guide titles and the full/wizard catalog. Hosted export checks now assert
the recovered feature text too. All 20 final-head CI jobs in `43193011736` passed
on `43295fc`; the merged tree is identical. Fixing commits are `1bb81e6` and
`43295fc`. See [diagnostics evidence](specs/evidence/zellij-section-field-diagnostics-2026-11-12.json).
The Apple Silicon packaging job first failed to acquire a hosted runner; the
unchanged-source retry and all four Homebrew checks passed. Development engine
run `43199867543` passed all 13 jobs on merge `3008a16`, publishing the four
`main` feature variants for Linux amd64/arm64 and building a site from the image.
These changes are not in published rc.5.
PR #143 (`ba5f0ad`) merges the first bounded composer (`type: compose`,
version 1), with the same tree as tested head `6b10098`.
A section has stack/grid/split layouts, heading/text/image/button/stat/divider
nodes and groups; three nested groups and 128 nodes are enforced by shared
engine/native constraints. Finite schema expansion is 14,701 compact JSON bytes,
with no recursive references or additional dependency. Plain `zel explain`
now lists node/layout alternatives, and generated context expands array unions.
Instance-aware H1 ownership preserves the fallback title when no composed H1
exists. Gradient reveal keeps solid text beneath its decorative paint and
provides no-JS, reduced-motion, forced-colour and print fallbacks.
Local verification: 482 engine tests, workspace types, 14 native authoring tests,
all 24 native-to-engine authoring cases and all full-starter variants pass. The
Rust workspace passes 744 tests (five ignored); warnings-denied Clippy passes.
Showcase adds an original hero/split/grid composition and SVG; docs add a source-
after-rc.5 guide and preview. All 20 final-head CI checks in `43265736975` pass;
the two Apple Silicon jobs needed an unchanged-head retry after runner-capacity
cancellation. The 335 showcase checks passed without browser retries, and six
light/dark desktop/phone captures were visually reviewed. All 13 development
engine jobs in `43274667686` pass. See [composer evidence](specs/evidence/zellij-compose-2026-11-12.json).
Layers, embedded sections/video, arbitrary effect ranges and interactive presets
are outside this first composer slice. Published rc.5 is unchanged.
PR #147 (`2373c5e`) merges a shared media-accordion section/composition node
and four offline named section presets. `zel presets` / `zel list presets`
discovers them; `zel ns -p <name>` expands editable YAML and can infer the type.
No runtime preset lookup or new dependency is introduced. The concise catalog
now has 25 types, with bounded accordion items and sequential no-JS/print output.
See [the implementation checkpoint](specs/zellij-media-composition-2026-11-12.md).
All 20 final-head checks in `43308742913` pass on `d8c6b7c`; the merged tree
matches exactly. All 356 showcase cases pass without retries (124 existing
conditional skips), and six desktop/phone light/dark captures were reviewed.
Local verification passes 489 engine tests, 747 Rust unit/integration tests
(five ignored), matched-toolchain doctests and 17 native authoring cases.
`0dac1d3` fixes printing of hidden panels and retains one caption across layouts;
`d8c6b7c` gives body/caption links a persistent underline. Earlier failures and
local toolchain retries remain recorded. All 13 development-image jobs in
`43313219720` pass; four `main` variants are published for Linux amd64/arm64. See [the evidence](specs/evidence/zellij-media-composition-2026-11-12.json).
Published rc.5 is unchanged. Book/PDF milestones and remaining composition
primitives follow this increment.

After the media/preset milestone: book/PDF publishing and remaining composition
primitives, plus technical/study book editions. Scoop,
WinGet and other package channels are not yet published. Live provider verification
and production documentation/domain migration remain deferred. Local guides,
release pages, shared installer docs and the tap are updated; this does not claim
the existing production documentation site was redeployed.

The documented CLI command families are implemented. On 2026-11-11 the user
authorized a release containing the post-rc.2 fixes and moved live deployment
provider verification to the end. **v0.2.0-rc.3 was published** from `58be9b1`,
with all 16 final-head CI checks and all 29 release/public-artifact jobs passing.
See [the release evidence](specs/evidence/zellij-rc3-release-2026-11-11.json).
Live provider/external-runner checks remain deferred follow-up work. The ledger
above separates these from documented importer limits. Newer dated
entries below supersede the earlier pending snapshots without erasing them.

Hosting-injected head whitespace is fixed in source by PR #129 (`360cb40`),
after all 16 CI jobs passed on `938a8bd`. All 55 documentation routes tolerate
the reproduced LF/comment/CRLF mutations; Chromium/WebKit preference checks pass.
See [the hydration report](specs/zellij-netlify-hydration-2026-11-02.md) and
[verification evidence](specs/evidence/zellij-hosted-head-hydration-2026-11-11.json).
Live Netlify verification and production migration remain deferred. This change
was newer than rc.3 at its original merge and is now included in rc.4. The tested
implementation merge skipped duplicate push workflows.

New requirements (2026-11-11) extend the earlier completion target: independent
company/product icons, an Apple-informed section audit and composable section
format, a versioned template catalog including books/PDF, and package-manager
distribution. See [the phased design and backlog](specs/zellij-product-expansion-2026-11-11.md).
The company slot is implemented in PR #131 (`b4cc9ba`), after all 16 CI
jobs passed on `7d7febbe` with the merged tree unchanged. It supports portable
and hosted sites, independent destinations/tab selection, hosted dark variants
and portable-to-hosted conversion. See [branding evidence](specs/evidence/zellij-company-branding-2026-11-11.json).
The Apple comparison now includes hosted visual and interaction checks; see
[research evidence](specs/evidence/zellij-apple-section-audit-2026-11-11.json).
Composition, full book/PDF publishing and additional package channels remain
planned. Company branding and the hydration
fix shipped in rc.4. No release or package-manager publication occurred in the
earlier branding-only increment. Templates must remain publicly downloadable if
the source repository is later sealed, using the existing public release mirror.

The user's follow-up on 2026-11-11 settles three expansion decisions:
prefer the short Homebrew formula name `zel`; increment pre-1.0 releases and
remove superseded Zellij distributions after the replacement and its consumers
are verified; and generate book PDFs only when requested, with a first-class
LaTeX route for mathematics, indexes and publication layout. Beautiful book
templates must encapsulate the LaTeX styling so authors normally edit content
and simple theme settings. See the updated expansion design and
[pre-1.0 retention policy](specs/zellij-release-retention-2026-11-11.md).
At that decision-only update, PDF/export remained planned. That update removed no
releases: its six delivery workflows still pinned rc.2 and stable installation
depended on older artifacts. The rc.4 milestone above migrated those consumers
and executed scoped retirement; retain the original decision history.

Implementation history: combined image publication and smaller runtimes merged
in PR #68 as `e7b0d79`, after all 16 PR checks passed. Static image compressed
layers shrank about 75%; Next images about 10% with the same fixture content.
Import heading anchors and literal code fences merged in PR #69 as `b1ff508`,
after all 16 PR checks passed. Imported page metadata and Docusaurus document
IDs merged in PR #70 as `5001718`, with all PR and post-merge checks passing.
Docusaurus autogenerated sidebars merged in PR #71 as `eb90705`, after all
16 PR checks passed; both post-merge workflows also passed. Generated category
pages and manual category links merged in PR #72 as `ab768ea`, after all 16
PR checks passed. Both post-merge workflows passed; main CI recovered from a
Docker info timeout on a targeted CLI-job retry. PR #73 merged as `a4b575d`
after all 16 final-head checks passed; both post-merge workflows passed. It adds
explicit Docusaurus slugs, fixes encoded Next page lookup/active navigation,
and allows selected runtimes the existing twelve-second discovery budget.
PR #74 merged import asset controls and optional `convert --validate` as
`74abf77`; all 16 PR checks and both post-merge workflows passed. PR #75 merged
website URL mapping and exhaustive CLI dispatch as `2c5ac7c`, after all 16
final-head checks passed, including the Windows correction in `74bacea`.
Its base-image post-merge workflow passed; main CI later exposed a Windows
watch-readiness assertion race, corrected in `a1003bd`. PR #76 merged relative
website document URLs and that harness fix as `e8f525d`, after all 16 final-head
checks passed; both post-merge workflows passed. VitePress imports merged in
PR #77 as `0729235`, after all 16 final-head checks passed; both post-merge
workflows passed too. Jekyll imports merged in PR #78 as `6190e51`, and Hugo imports merged in
PR #79 as `7a03f50`, each after all 16 final-head checks passed. Both post-merge
workflows passed for each PR. Sphinx imports merged in PR #80 as `c067b5a`,
after all 16 checks passed on `fb622d1` in run `42109434915`. Its base-image
post-merge run passed; main run `42113725983` exposed a Windows watched-folder
rename EPERM in the preview fixture. This milestone bounds retries for that
operation while retaining real disappearance/recovery assertions.
Cloudflare Pages static deployment merged in PR #81 as `9cde253`, and Vercel
static deployment merged in PR #82 as `c8b485c`, each after all 16 final-head
checks passed. Both post-merge workflows passed for both PRs. GitHub Pages
publication merged in PR #83 as `e25d8d9`, after all 16 final-head checks passed;
both post-merge workflows passed. S3 static website uploads merged in PR #84 as
`9e3dd79`, after all 16 final-head checks passed on `d141aa8`. Both post-merge workflows passed: main `42155106031`, base image
`42155106146`. Fly merged in PR #85 as `30a0272` after all 16 final-head checks passed;
both post-merge workflows passed. Railway merged in PR #86 as `b24b3dc` after
all 16 final-head checks passed; both post-merge workflows passed. GitLab and
CircleCI generation merged in PR #87 as `b590cb2`, after all 16 final-head
checks passed; both post-merge workflows passed. Section authoring merged in
PR #88 as `5078b51` after all 16 final-head checks passed. Its post-merge base
image passed; main CI exposed a Windows ARM upgrade sharing-lock failure.
PR #89 merged `new docs --full` and Windows upgrade lock recovery as `479eac4`
after all 16 final-head checks passed. Its post-merge base-image workflow passed;
the Ubuntu portable job timed out downloading browser system dependencies and
the targeted unchanged-head retry passed, so both post-merge workflows are green.
PR #90 merged site/configuration doctor diagnostics and JSON reports as
`e673306`, after all 16 final-head checks passed. Both post-merge workflows
passed. PR #91 merged rectangular Sphinx tables and fixed imported download
navigation as `6b3b4b4`, after all 16 final-head checks passed. Its base-image
workflow passed; its post-merge showcase dependency-download timeout recovered
on a targeted retry of the unchanged commit, leaving all 16 main checks green.
PR #92 merged Sphinx list/CSV table directives and directive boundary fixes as
`0c4d79e`, after all 16 final-head checks passed. Both post-merge workflows
passed. PR #93 merged nested Sphinx lists as `2ed992d`, after all 16
final-head checks passed; both post-merge workflows passed. PR #94 merged
alphabetic/Roman numbering and restored prose list markers as `3f14b99`, after
all 16 final-head checks passed; both post-merge workflows passed. PR #95 merged
document-local Sphinx text/image substitutions and inline image layout as
`4dcb12c`, after all 16 final-head checks passed. Its base-image artifact upload
recovered on a targeted retry; both post-merge workflows passed. Native site
wizards merged in PR #96 as `0849faf`, after all 16 final-head checks passed.
Both post-merge workflows passed. Native page wizards merged in PR #97 as
`c10c1e1`, after all 16 final-head checks passed; both post-merge workflows passed.
Native app wizards merged in PR #98 as `2d7e4a5`, after all 16 final-head
checks passed; both post-merge workflows passed. Complete JSON section values
and optional-field removal merged in PR #99 as `6c01f4a`, after all 16 final-head
checks passed; both post-merge workflows passed. Local Sphinx `literalinclude`
excerpts merged in PR #100 as `9fb29c8`, after all 16 final-head checks passed;
both post-merge workflows passed. PR #101 fixed MDX code leaking into search
and duplicated guide titles/shifted anchors with CRLF line endings, merging as
`cac41b7` after all 16 final-head checks passed; both post-merge workflows passed.
PR #102 added local-file Sphinx CSV tables and fixed multiline CSV header
indentation, merging as `245eacd` after all 16 final-head checks passed; both
post-merge workflows passed. Public asset URL linking merged in PR #103 as
`fa0b206`, after all 16 final-head checks passed; both post-merge workflows
passed. Candidate rc.2 is published from PR #104 merge `54e636f`; all 29 release
jobs passed, including public package and image acceptance. PR #105 merged
rc.2 delivery pins and active documentation as `5677577`, after all 16 main
PR checks and six generated-workflow verifications passed. All six manual
rc.2 delivery runs, both Netlify draft HTTP checks and both fresh-pull site-image
checks passed. Both post-merge workflows passed too. Jekyll TOML configuration
imports merged in PR #107 as `5563902` after all 16 final-head checks
passed; both post-merge workflows also passed. Collection metadata sorting and
stale order-list handling merged in PR #109 as `47ceb61` after all
16 final-head checks passed. These capabilities are newer than published
rc.2. See the latest entry and
delivery evidence for exact source, image digests, draft URLs and verification status.
Live Cloudflare/GitHub Pages portable and hosted acceptance and Railway
static/Next acceptance passed on 2026-11-10. Four provider defects were fixed in
PR #111, newer than rc.2; see the latest entry and live-provider evidence.
Both post-merge workflows passed; all temporary acceptance resources are removed.
Local parsed Sphinx includes merged in PR #114 as `ee866af` after all 16
final-head checks passed. The exact final source passed 702 local Rust tests; both post-merge workflows
subsequently passed (16 CI jobs and 13 Base image jobs). A disposable Vercel
acceptance workflow and the empty-upload acknowledgement fix merged in PR #116
as `d1d5b70`, after 704 local Rust tests and all 16 final-head CI jobs passed.
Both post-merge workflows passed (16 CI and 13 Base image jobs). The Vercel
token worked, but the supplied team is Hobby with no trial. The user deferred
Vercel live verification because trial signup required a payment method. The
temporary Actions secret and remote lookup branch were removed; no Vercel test
project was created. Published rc.2 lacks the fix. Live Fly and AWS account
acceptance remains pending; Vercel is explicitly deferred, not verified.
See the latest dated entry for the current source and verification limits.
Sphinx wildcard/reversed navigation merged in PR #119 as `206acc4`, after
709 local Rust tests and all 16 final-head CI checks passed. Its post-merge Base
image passed; main CI hit transient npm downloads and a WebKit navigation error.
All 16 main CI jobs passed after a targeted unchanged-source retry; the original
failures and recovery remain in the evidence. PR #121 adds source exclusion
globs and a preview navigation lifecycle fix, merged as `48f95c9` after all
16 final-head checks passed. Both automatic post-merge workflows passed
(16 CI jobs and 13 Base image jobs), without retries. See the latest dated entry
and evidence.
Every registered command family has a handler, but the full design
remains incomplete; see [the current CLI status](specs/zel-cli-status-2026-11-05.md)
for format/provider/advanced-option limits. The old
rc.2 proposal at `0fbf0a1` is superseded by the 2026-11-09 preparation; do not
tag that old commit. The user explicitly approved the refreshed candidate,
which is now published from `54e636f`. See the latest dated entries and the
release/image measurement evidence.

Latest content-preservation work: PR #55 merged as `f1ca1ff` after all 13 CI jobs passed in
`41573372928` on final head `2794773`. Source fix `8370ec4` preserves 82 legacy
bookmarks before a proposed Netlify migration. All 46 production routes, 282
original heading IDs and 29 referenced images are retained in the 55-route
staging site. Both published test images passed corrected fresh-pull acceptance;
PR #54 main CI `41572436263` also passed. See
[the migration report](specs/zellij-netlify-migration-2026-11-02.md).
The user created https://zel-stage.netlify.app and made its main site and
previews public. Updated GitHub secrets now work: candidate portable/hosted
draft deployments, HTTP acceptance and browser search passed. Full-docs routes,
images, bookmarks and search are retained, but React hydration errors remain:
the newline preceding Netlify's hosting comment reproduces the problem locally.
The user disabled the badge; its script disappeared, but the comment/newline
and React #418 remain. The user reports filing a Netlify bug and emailing
support@netlify.com; a response is pending. The support report and verified local
reproduction are in [the hydration diagnosis](specs/zellij-netlify-hydration-2026-11-02.md).
Keep the old live site/domain intact. See the latest dated entry and
specs/evidence/zellij-netlify-acceptance-2026-11-02.json.

Historical initial proposal (2026-11-02): candidate **0.2.0-rc.2 was prepared and merged** in PR #60 as `0fbf0a1` after
all 13 checks passed in `41595462484` on `04653b0`; their source trees are
identical. Browser acceptance PR #59 merged as `93dca08` after all 13 checks
passed in `41594855467` on `97f9f9f`. At that time, publication awaited explicit user
approval: automatic approval review rejected the attempted release command
before execution. No rc.2 tag or release existed then. The old proposal targeted
`0fbf0a12c3cf7074c4620e2bb84cc5d8e5fd7469`; see the latest dated entry and
specs/evidence/zellij-rc2-preparation-2026-11-02.json.
Final local `zcheck` passed with serial Rust fixtures: 365 engine and 317
Rust tests passed; the default parallel run hit independently reproduced local
shell-startup delays. See the final validation entry for this limitation.

Previous release: native **0.2.0-rc.1 was published** at tag commit `61881e8` (PR #50).
All eight public-artifact acceptance jobs passed in run `41548199714` using
corrected harness `29029e1`: six native platforms and both Linux image
architectures. The original release run has two Windows downloader failures;
the follow-up checks the unchanged published assets. Download recovery,
validation of existing tags and candidate installation/CLI documentation merged
in [PR #51](https://github.com/mylife-inc/Zellij/pull/51) as `49d9a14` after all
13 checks passed on `9ab6232`. Stable release remains `zellij-v0.1.3`; production
docs were not deployed. See the final dated entry and specs/zel-release-procedure.md.
Temporary build/image
selection merged in PR #46, including the Zod/schema compatibility fix.
Image platform selection
merged in PR #45. Shared output controls merged in PR #44.
Explicit --config merged in PR #43; shared --cwd and
--site-root merged in PR #42.
Hosted rendering selection and optional-peer installs merged in PR #41.
Eject/features merged in PR #40; maintenance
commands merged in PR #39.
Portable development and hosted static preview merged in PR #38.
Formatting and validation watch merged in PR #37.
Editable Next.js wrappers merged in PR #36, configured import/ship in PR #35,
CI generation in PR #34 and Netlify delivery in PR #33. These capabilities are
available in candidate `zellij-v0.2.0-rc.1`; the stable native release is still `zellij-v0.1.3`.

The user requested graphify on 2026-11-08 and selected **cli/** after the full
855-file corpus exceeded the skill's threshold. The local, git-ignored knowledge
map is at `graphify-out/graph.json`, interactive view `graphify-out/graph.html`,
and audit `graphify-out/GRAPH_REPORT.md`; `.graphify_root` records the CLI scope.
Use `graphify query` from the repository root. It does not map engine/examples/specs
outside `cli/`. Its integrity report flags dangling endpoints and collapsed
relationships; inferred edges and minified Mermaid symbol matches require source
verification. It is navigation assistance, not evidence of completeness or bugs.
The graph report records unrepresented inputs and unavailable agent token counts.
Its recorded snapshot is `6d0b5c3`; public asset URL linking is newer and must
be inspected in current source until the next graph refresh.

**Graphify scope decision — 2026-11-08:** the user excludes the repository-root
`examples/**` folder from the Graphify knowledge corpus. Apply this exclusion to
discovery, extraction and context enrichment, including future repository-wide
maps. The existing CLI-only graph already excludes that folder.

This is the starting point for future Zellij conversations in this checkout.
The companion [audit](specs/zellij-audit-2026-10-16.md) contains reproducible
defects, remaining work, evidence, and verification limits. Recheck dated facts
before acting. The initial review created documentation only; subsequent authorized work is
recorded below.

### Implementation update — 2026-10-16

The user subsequently authorized implementation and the repository setup.
`mylife-inc/Zellij` is now **public**, with **main** as its default branch.
Before the visibility change, Gitleaks 8.30.1 scanned all 134 reachable commits
and reported no detected leaks. No historical rewrite was performed.

The first milestone merged in [PR #1](https://github.com/mylife-inc/Zellij/pull/1)
as `6950589`. Commit `1a2e26b` adds shared output-path/collision
checks, conversion source containment, protected page/guide creation, staged
static exports, and streamed image Dockerfiles. `deda7a0` restores the browser
job's engine-build prerequisite; `bd6b6c6` bounds file-descriptor use during
large conversions and tests the CLI against an image built from its checkout.
See the audit's implementation record for the resolved finding IDs.

Verification established: **99 Rust tests**, warnings-denied Clippy, formatting,
**227 engine tests**, workspace typechecking, and the engine/showcase production
builds pass. Hosted browser run `39000366953` passed **294 tests**, with 114
existing conditional skips across desktop/mobile Chromium and mobile WebKit.
The CI workflow additionally exercises initial static export, failed-build
preservation, and successful replacement using a real container. Consult the
PR's latest checks and merge status before relying on a particular revision.

Compatibility change: conversion requires a new/empty destination outside its
source tree. Static export requires a new/empty directory outside content, or
an export previously created with the `.zellij-output` ownership marker.
Existing unmarked exports need a new/empty output on their first build with the
new CLI; never mark an authored directory as generated output.

A clean `npm ci` restored the missing local packages. `npm audit` also reported
19 advisories (3 critical, 6 high, 6 moderate, 4 low). Next.js and Vitest upgrades
need a separate compatibility-tested change; no automatic audit fix was applied.
This was the dependency baseline before the second milestone below.

### Dependency and template update — 2026-10-16

The user authorized continuing. Commit `17d5a23`, tracked in
[PR #2](https://github.com/mylife-inc/Zellij/pull/2), removes the 19 reported
npm advisories and fixes the invalid section scaffolds (audit R4). The locked
tree and a freshly generated standalone app with all features both report
**zero npm advisories** at this date; this is not a claim that all bugs are gone.

- Next.js is `16.4.2`; Vitest resolves to `4.1.12`, Vite `7.3.7`, Mermaid
  `11.17.2`, KaTeX `0.18.10`, esbuild `0.28.2`, and XML DOM `0.9.12`.
- Scoped dependency overrides keep optional math/diagram chains patched.
  The engine manifest carries the production overrides; `zellij init` and
  feature changes copy them to the app root, where npm actually enforces them.
  Manual wrappers must carry that map too. The repository additionally
  overrides tsup's esbuild. Tests keep the shared constraints in agreement.
- `src/content/frontmatter.ts` replaces gray-matter, using the existing YAML
  package for guides, prose, and search. Metadata must be a YAML mapping with
  a closing delimiter; executable language tags are rejected. BOM, Windows
  newlines, nested values, and multiline scalars are tested.
- `src/cli/section-templates.ts` generates YAML from schema shapes and bounds,
  chooses valid media alternatives, and validates the serialized result with
  the complete Zod schemas. `context --json` embeds `sectionTemplates`; Rust
  copies those strings instead of guessing nested types and list lengths.
  Templates contain TODO text and explicit replaceable media paths. The
  command writes only YAML; authors supply their actual assets.
- CI audits dependencies, validates and statically exports all 23 templates
  emitted by the actual Rust binary, and tests formulas/Mermaid math labels
  in browsers. Browser failures retain traces and the job has a time limit.

Local verification: **260 engine tests**, **99 Rust tests**, formatting,
warnings-denied Clippy, workspace typechecks, engine packaging, all four
example content checks, and all 23 Rust-generated template checks pass.
The showcase production build passes with webpack. Local Turbopack hits the
execution sandbox's helper-process port restriction; the normal Turbopack
build and container/browser acceptance are hosted CI gates. Consult PR #2's
latest checks for their results. No new native CLI release is issued here.

Hosted run `39004471983` at `d8c9f5d` passed all four jobs: engine, schema sync,
CLI/container acceptance, and browsers. The normal Turbopack build and all 23
template static exports passed; **297 browser tests passed**, with **114
conditional skips and no retries**. A minimal (`--features none`) fresh wrapper
also audited clean. Its lockfile still includes Mermaid, remark-math and
rehype-mathjax through the engine's optional dependencies; audit follow-up S11
tracks that feature-selection/package-size defect separately.

CI reliability history: after PR #1, run `39001111736` initially stalled on
repeated mobile-WebKit navigation timeouts. Its fresh-runner retry passed all
294 tests (114 conditional skips). This is not a diagnosed/fixed browser bug.
The separate automated security-review plugin failed its PR #2 commit review
with HTTP 400; that review must not be represented as passed.

### Conversion update — 2026-10-16

The next authorized milestone is [PR #3](https://github.com/mylife-inc/Zellij/pull/3).
Source commit `4196e6b` addresses R5's ordinary Markdown document links and
referenced assets for both MkDocs and Docusaurus. A shared parser-based pass
rewrites links to canonical engine routes, expands reference-style links,
preserves query/fragment text and code examples, and copies binary assets into
`assets/imported/docs/` or `assets/imported/static/`. Docusaurus's default
`static/` paths are supported; unlisted documents remain unpublished.

Assets participate in the same complete output preflight/staging as text.
Escaping references, symlinks, case-only file conflicts, hidden document names,
and duplicate public routes are rejected before writing. Missing or ambiguous
references and HTML/JSX attributes are reported. Markdown images under the
public asset prefix now receive the site's basePath in the MDX component map.

R5 still has explicit migration limits: source-specific/custom heading IDs,
fragments targeting a removed leading H1, HTML/JSX attributes and imports,
custom static folders and source route overrides require manual work. Fragment
text is retained, not universally validated or translated. Do not describe
this milestone as full-fidelity migration of arbitrary MkDocs/Docusaurus sites.

Local verification: **109 Rust tests**, **261 engine tests**, warnings-denied
Clippy, formatting, all workspace typechecks, engine packaging and the updated
documentation content check pass. `scripts/check-conversion.mjs` runs the real
CLI and engine to check routes, an actual heading anchor, rendered image URLs
under `/products/docs`, and copied binary bytes. Hosted CI adds static exports
of both imports and checks actual HTML links, fragments and asset files. Read
PR #3's latest checks before claiming hosted acceptance; no native CLI release
is part of this milestone.

Integration review exposed a separate prefix-configuration gap: generated
Next config does not mirror `site.yaml`'s basePath. The CI fixture configures
both explicitly using `scripts/fixtures/conversion.Dockerfile`. The existing
showcase `verify-basepath.mjs` also replaces strings no longer present in its
configuration, so it must not be trusted as current prefix acceptance (S12).

PR #2 merged as `3408f37`; main run `39005197060` passed all checks, and base
image run `39005197029` published all four feature variants for AMD64/ARM64.

### Scaffolding and discovery update — 2026-10-16

Commits `fe4240c` and `ee68282`, tracked in
[PR #4](https://github.com/mylife-inc/Zellij/pull/4), resolve the reproduced
content-authoring findings **R6–R8**.
Rust content scaffolds and converters share a JSON-backed YAML string scalar
serializer, including explicit escapes for YAML's extra Unicode line breaks and
characters its parsers reject literally.
The Node initializer uses the same representation. Names, copyright text,
page identifiers, section references, guide titles and front matter preserve
punctuation, implicit YAML values and multiline text. This covers generated
content metadata; it does not validate arbitrary npm package names or workflow
inputs as package identifiers, GitHub expressions or shell commands.

Plain/minimal starters omit the guide CTA unless `--guide` creates its target.
All CLI commands that resolve a content root now use `Project::discover`:
`./content`, current root, `./docs`, then `./documentation`. Explicit paths
retain their existing exact-directory behavior. Marker files, a sections
directory, and guide-only roots are recognized; hidden/reserved guide folders
and markers of the wrong filesystem type are excluded. Discovery never walks
upwards. This selects a root; the engine still owns semantic validation.

Local verification: **117 Rust tests**, **262 engine tests**, warnings-denied
Clippy, formatting, workspace typechecks, engine packaging, documentation
content validation, and both conversion fixtures pass. The new
`scripts/check-scaffolds.mjs` checks exact metadata and rendered Hero links
for **40 Rust starters, 10 Node content starters and 10 guide titles**. Hosted
CI additionally exercises four guide-only discovery locations and a menu-only
root through the real container engine. Hosted results are pending at this
record; consult the PR before claiming acceptance. No native release is made.

The separate security-review plugin failed to read its model response during
the source commit, although the commit itself succeeded. That review did not
complete and must not be described as passed.

Prior milestone completion: PR #3 merged as `dd99c2c`. Main CI `39007109533`
and image run `39007109576` passed, publishing all four variants for Linux
AMD64/ARM64. Browser acceptance had 296 initial passes plus one successful
retry, with 114 conditional skips. The retry was a mobile-WebKit navigation
error at `/guide/quickstart`; PR CI had a different FAQ interaction retry.
These flakes remain undiagnosed, not repaired by the conversion changes.

### Header layout and overflow update — 2026-10-17

Source commit `da1c222` implements the requested viewport-wide header: branding
at the left inset and search/theme/actions at the right. The existing
`brand.logo.href` remains the company-site link; `brand.title` can retain a
separate site-home link. No new branding schema or company assets are required
by the engine change. Search continues to open its existing dialog.

Desktop navigation measures the remaining space and inert copies of every
label, including active font weight and icons. Trailing items move into More
and return when space grows; resizing, loaded fonts and theme changes trigger
measurement. Existing `menuOverflowAfter` remains the configured count limit.
The overflow model preserves destinations, group headings, descriptions,
icons and external/new-tab flags. The mobile drawer receives the full resolved
menu, independently of the width-selected desktop prefix.

Keyboard focus follows items that move into/out of overflow; breakpoint changes
close stale panels and release drawer scroll lock. Tall More panels scroll,
and compact More panels stay within the viewport at either edge. Mobile
controls have 44px targets at the default root font size. The header was checked
at widths from 320 to 1600px, including larger menu text and a wide brand.
This is not a full WCAG audit or a claim of manual screen-reader acceptance.

Local evidence: **268 engine tests**, workspace typechecks, engine packaging,
and documentation validation pass. The focused desktop/mobile Chromium
navigation run passed **34 tests** with **26 device-specific skips**. The six
new desktop header cases also pass (the final compact-panel check was rerun
separately after correcting its fixture width). Final engine typechecking and
six focused unit tests pass after the positioning adjustment. Production builds,
the full browser suite and mobile WebKit remain hosted CI gates; consult the
PR containing `da1c222` for their outcomes. No native CLI release is made.

Prior milestone completion: PR #4 merged as `4be61dc`. PR CI `39010124427`
passed with 297 browser passes, 114 conditional skips and no retries. Main CI
`39010596721` passed with 296 initial browser passes, one successful retry and
114 skips. The retry was the previously observed mobile-WebKit FAQ interaction
(answer hidden after clicking the summary); its cause remains undiagnosed.
Image run `39010596751` published all four variants for Linux AMD64/ARM64 and
passed the published-image build/serve smoke test.

### Canonical search and preparation update — 2026-10-17

Source commit `29654d3` resolves **R10–R12** and the interrupted search load
**S2**. `createContentSource` in `src/config/runtime.ts` now supplies the shared
folder/legacy selection and content-path contract for rendering and preparation.
`writeSearchIndex` reads a fresh `loadSite` result, including canonical guide
routes, frontmatter/sidebar overrides, unlisted pages and title-stripped bodies.
Preparation bypasses the global rendering memo; S8 remains a separate issue.

`src/mdx/headings.ts` parses Markdown/MDX without executing authored JavaScript.
Search and the TOC use the renderer's `rehype-slug` semantics, including formatted,
setext, quoted, indented, repeated and mathematical headings. All heading levels
participate in duplicate IDs; only h2/h3 split search passages, so deeper
subsections retain their text. Search stores unprefixed routes for Next's router.
The existing regex-based prose cleanup is not a complete MDX text interpreter.
Optional math loading is shared with rendering and stays external in CLI bundles;
bundling MathJax broke its runtime paths and was caught using Zellij's own docs.

`zellij prepare`, `sync-assets` and `search-index` accept `--content <dir>` and
`--config <legacy-file>`, including inline values with spaces/equals signs.
Helpers still accept an app-root string or now a `ZellijRuntimeOptions` object.
Wrappers must pass the same explicit content root to preparation and rendering;
preparation does not infer TypeScript configuration. Missing roots and malformed
assets directories fail before clearing previous prepared files. Physical-path
overlap checks protect source assets, including public-directory symlinks.
Rerunning asset sync removes stale output even if the source assets folder has
been deleted. Automatic dev watching is still absent: **S9 is only partly fixed**.

Synthetic guide roots and configured home redirects now claim route ownership;
only a guide and its own root page may intentionally share that route. The
search dialog aborts cancelled requests, retries on reopen after cancellation
or failure, and caches completed data for the current deployment prefix.

Local verification: **285 engine tests**, all workspace typechecks, packaging,
all four example preparation/content checks, and the packaged CLI integration
fixture pass. The fixture covers canonical routes, actual optional math loading,
custom paths, legacy config, invalid flags and asset removal. Four focused
Chromium desktop/mobile recovery cases pass without retries. An initial broad
run against the local dev server timed out in the existing `networkidle` check;
production lazy-loading and WebKit acceptance remain hosted CI gates. The new
recovery test's early-hydration click was corrected to use the existing retrying
shortcut pattern. CI additionally exports a real container site and checks that
each indexed route and anchor exists. Consult this commit's PR for hosted results.

R13 documentation now states the fast check's actual limits. Complete prose-link
and asset validation remain open, as do basePath synchronization/acceptance S12,
remaining UI findings and native CLI/platform completion. No native binary
release is made in this milestone.

Prior milestone completion: [PR #5](https://github.com/mylife-inc/Zellij/pull/5)
merged as `7fb5178`. PR CI `39019128511` and main CI `39019556088` passed all four
jobs, including **305 browser passes, 124 conditional skips and no retries**,
268 engine tests and 117 Rust tests. Image run `39019556101` published all four
feature variants for Linux AMD64/ARM64 and passed published-image build/serve
smoke acceptance. Previously recorded FAQ/WebKit flakes remain undiagnosed.

### Deployment prefix and runtime isolation update — 2026-10-17

Source commit `8463720` resolves **S6, S8 and S12**. The shared native
`engine/packages/zellij/deployment-config.cjs` resolves content roots and the
deployment prefix for both `withZellij()` and the runtime. It lives at package
root so source, compiled modules and native Next configuration resolve the same
file; package and container distributions include it.

`site.yaml` supplies Next's basePath automatically. A wrapper may supply it when
content omits it; explicit values must agree, including an explicitly empty
prefix. Values are normalized and malformed prefixes rejected. Next embeds the
effective value as reserved `ZELLIJ_BASE_PATH`; runtime configuration must agree
with that build-time choice. Changing the prefix requires rebuilding. Custom
content paths must match across preparation, runtime factories and the second
argument of `withZellij(config, contentOptions)`.

Runtime memoization is keyed by absolute application/configuration/content
paths, runtime prefix and compiled prefix. Production retains each site's
result; development retains the 250 ms TTL. Failed loads evict their entry so
correcting a missing/invalid file can recover. Social metadata images now receive
the same prefix as canonical URLs and other assets.

The previous in-place showcase prefix script is replaced by an isolated,
production-built wrapper fixture. Only its content declares the prefix; its
custom content directory exercises preparation and rendering together. CI tests
redirects, nested links, ordinary/optimized images, Next resources, metadata,
sitemap, robots and search in Chromium and WebKit. Conversion exports now use
the default CI image without a special Next-config override.

Local evidence: **303 engine tests**, all workspace typechecks, engine packaging,
packaged-CLI integration and all four example preparation/checks pass. The
production prefix fixture passes Chromium with webpack and a local font-response
fixture. One earlier normal-font build succeeded; another failed in Next's Google
Fonts loader (new audit **S13**). Hosted acceptance uses normal fonts, Turbopack
and WebKit and remains a gate; consult the PR containing this commit. The local
font fixture is not shipped and does not resolve S13. No native binary release
is included.

Prior milestone completion: [PR #6](https://github.com/mylife-inc/Zellij/pull/6)
merged as `d115a9a`. PR CI `39022447464` passed all four jobs, with **285 engine
tests, 117 Rust tests, 311 browser passes, 124 conditional skips and no retries**.
Main CI `39022871779` passed with 310 initial browser passes plus one successful
retry of the existing mobile-WebKit FAQ interaction (`sections.spec.ts:191`).
Its cause remains undiagnosed. Image run `39022871680` published all four feature
variants for AMD64/ARM64 and passed published-image build/serve acceptance.

### Deep and strict validation update — 2026-10-17

Source commit `86e5e2d` addresses the static-validation portion of **R13**.
Both `zel check <dir>` and `zellij check <dir>` accept `--deep` and `--strict`.
The default remains the fast content-model check. Deep mode parses MDX without
executing authored JavaScript and checks static prose/section links, heading,
HTML and section anchors, local media and section previews. Strict mode fails
on existing or new warnings. Build/dev/migrate retain fast checks unless the
caller runs an explicit deep/strict gate first.

Reference semantics are Zod metadata in `src/content/reference-schema.ts`, used
by configuration/section schemas and exported in the generated Rust schema.
Do not add a new link/media/inline-prose field without its reference annotation.
`src/cli/deep-check.ts` uses canonical Site routes, redirect targets and the
renderer's heading/GFM anchor pass. Inline prose shares tokenization with
`InlineMarkdown`. Section source paths survive loading for useful diagnostics.
MDX positions refer to the loaded body, after front matter/title removal.

The checker preserves the first Markdown reference definition, follows
home/guide redirects before checking fragments, and validates example references
in their actual rendering page. It detects recursive previews and duplicate IDs.
Asset paths must resolve to files inside the physical content/assets boundary;
raw Markdown/HTML image URLs use `/_zellij/assets/...`, while Screenshot and
schema-declared media use content asset paths. Fast navigation validation now
ignores URL query strings when finding a route.

Limits remain explicit: no component/JavaScript execution or remote HTTP checks;
literal menu `url:` remains unchecked. Dynamic MDX, custom components, component
ID props, srcSet and contextless relative links warn. Wrapper-owned public files
outside the content asset prefix and SVG-internal fragments are not verified.
Optional math parsing depends on the installed environment: use the installed
wrapper for math-enabled deep checking; the bare container checker installs no
optional dependencies. Full build/browser acceptance is still necessary.

The checker exposed four broken showcase anchor links in `pro/tour.yaml` and
`pro/highlights.yaml`; they now target existing highlights/specifications content.
Local evidence: **317 engine tests**, **117 Rust tests**, workspace typechecks,
formatting, warnings-denied Clippy, engine packaging, isolated packaged-checker
and preparation integration, and all four local examples under `--deep --strict`
pass. CI adds deep/strict validation for the three fully tracked examples (PayOS
needs its generated docs), plus the real Rust/container command and static export.
Consult this source commit's PR for hosted results. No native binary release is
included; the new Rust flags require the current CLI source and matching image.

Source review also found **S14**: video-embed omits basePath for local video and
poster URLs. It remains a separate rendering fix, with prefixed browser acceptance
needed. S13 font-build reproducibility and the prior browser flakes remain open.

Prior milestone completion: [PR #7](https://github.com/mylife-inc/Zellij/pull/7)
merged as `ba30f9b`. PR CI `39025935776` and main CI `39026348416` passed all four
jobs, with **303 engine tests, 117 Rust tests, 311 browser passes, 124 conditional
skips and no retries**. Separate Chromium/WebKit prefix acceptance passed with
normal fonts and Turbopack. Image run `39026348423` published all four variants
for AMD64/ARM64 and passed the published-image build/serve smoke test.

### Development preparation update — 2026-10-17

Source commit `8674908` completes the remaining preparation-watching part of
**S9**. The Node helper now provides `zellij dev [--content <dir>] [--config <file>]
-- [Next dev arguments]` and standalone `zellij prepare --watch`. Generated
wrappers and all four examples use the supervised dev command. The base image
uses that same generator, so Rust `zel dev` gets watching through its existing
`npm run dev` invocation; no new Rust flag or native binary release is needed.
Existing wrappers must update their dev script or run the standalone watcher.

`src/cli/watch-preparation.ts` polls filesystem metadata every 500 ms, requires
two stable snapshots, and serializes preparation. This covers bind mounts,
custom/legacy content locations, root removal/recreation and symlinked roots.
Generated output, Next output, node_modules and Git internals are excluded.
Separate content/asset snapshots avoid copying images on prose-only edits and
avoid rebuilding search for asset-only changes. Failed preparations do not
acknowledge pending asset changes. Repeated identical errors are suppressed;
corrected content recovers without restarting. Startup must prepare successfully.

Search/manifest JSON is replaced through a sibling temporary file. Asset trees
are copied and hashed before a short rename swap; failed copies preserve the
old tree. This is not a transaction across the whole site, and an asset rename
swap can briefly leave its directory absent. The dev supervisor owns Next's
process, forwards signals, propagates its exit status, and closes the watcher.
Search clears its loaded data on close and bypasses HTTP caching in development;
production keeps the existing cache behavior.

Local evidence: **324 engine tests** (seven watcher regressions), workspace
typechecks, engine packaging, isolated installed-CLI process/exit acceptance,
existing packaged preparation, and deep/strict docs validation pass. The real
webpack dev server passes Chromium checks for updated search on reopen, rendered
headings/images after reload, added/deleted pages, removed/restored assets,
invalid-MDX recovery and released server port. CI adds the installed-package
fixture, actual Rust/container dev acceptance and Chromium/WebKit dev acceptance
with normal Turbopack. Consult the PR containing this commit for hosted results.

Content edits still require browser reload for page/image changes. Automatic
browser reload is future work; do not equate this watcher with full MkDocs live
reload. Restart after deployment-prefix, wrapper-config or feature changes.
Nested symlinks are not followed by the polling traversal. Portable Markdown
build/serve/view remain unimplemented, as recorded in the portable contract.

Browser verification exposed **S15**: MDX h2/h3/h4 links replace their accessible
names with “Link to this section.” This remains an accessibility follow-up;
S13, S14 and the existing mobile-WebKit flakes also remain open.

PR #9's first normal-Turbopack dev gate exposed S13 again: IBM Plex Sans failed
before browser acceptance with `next/font/google queries have exactly one entry`.
This matches upstream Next issue #99114, whose fix #100158 is merged but was not
in npm's stable 16.4.2 on 2026-10-17. See the audit for links and evidence limits.
A retry does not resolve the build's Google Fonts dependency. Treat a stable
font remedy as the next reliability priority before expanding the CLI surface.

Prior milestone completion: [PR #8](https://github.com/mylife-inc/Zellij/pull/8)
merged as `d4615ca`. PR CI `39036985726` and main CI `39037525173` passed all four
jobs (317 engine tests, 117 Rust tests and the real deep/strict container/export
fixture). Both browser runs had 310 initial passes plus one successful retry of
the known mobile-WebKit FAQ interaction, with 124 conditional skips. Separate
Chromium/WebKit prefix acceptance passed. Image run `39037525149` published all
four feature variants for AMD64/ARM64 and passed published-image build/serve
acceptance. No new native CLI release was made.

### Reproducible theme fonts — 2026-10-17

Source commit `7963587` replaces the build-time Google Fonts loader with a
pinned local snapshot, addressing **S13**. The twelve existing families retain
normal styles, weight ranges, Unicode subsets, swap behavior, CSS variable
names and metric-adjusted fallbacks. There are **75 unmodified WOFF2 files**
(1,539,836 bytes total). Fonts are not preloaded; a browser requests only the
families/subsets used by its page. The generated stylesheet uses namespaced
family names and relative asset URLs that Next processes with the site prefix.

`src/themes/fonts/manifest.json` records URLs, SHA-256, sizes, user agent, pinned
license provenance and fallback metrics. Original CSS and OFL notices are kept.
`prepare-fonts.mjs` verifies the snapshot entirely offline and copies it beside
the compiled modules. Git attributes preserve hashed upstream bytes on Windows.
The stylesheet references a bundled file containing every full license notice. Updates are explicit, reviewed
snapshot changes; normal builds never refresh fonts. Dependency installation
can still require network access. This does not implement portable `file://`
documentation or the requested Rust build/serve/view commands.

Local evidence: **324 engine tests**, workspace typechecks, engine build and
strict/deep documentation checks pass. The npm archive contains all font files
and notices for both source and compiled modules. Real webpack/Chromium
production-prefix and development-watcher acceptance pass, including all font
faces decoding, lazy loading, local prefixed URLs and deployed notices. CI adds
a fresh Turbopack site build with Docker `RUN --network=none` after dependency
installation, plus Chromium/WebKit production and development font checks.
The first hosted run `39053315323` passed the offline Turbopack build,
engine/schema/CLI jobs and 310 production browser tests plus one successful
retry of the known mobile-WebKit FAQ interaction (124 conditional skips).
Its production font gate failed because Turbopack stripped license comments.
Follow-up `21479d3` emits the combined notice as a real CSS URL asset and checks
its deployed response byte-for-byte. Local webpack/Chromium dev acceptance
passed this follow-up. [PR #10](https://github.com/mylife-inc/Zellij/pull/10)
tracks final hosted verification. Run `39054096571` passed all 311 production
browser tests without retries, then exposed a test URL-resolution error:
Turbopack emitted the notice relative to its stylesheet (`../media/...`), while
the test fetched it relative to the page. The trace confirmed the emitted CSS
reference; the check now resolves it against the owning stylesheet and still
requires the complete notice bytes and local deployment prefix. The corrected
license asset also passes local webpack production acceptance. No native CLI
release is made.

Run `39054561760` passed all engine/schema/CLI checks, offline build, and both
Chromium/WebKit production font/prefix checks. Development passed Chromium but
WebKit requested Next's own `/__nextjs_font/geist-latin.woff2` debug-overlay font,
which the test mistakenly treated as a Zellij asset. The installed Next overlay
middleware explicitly serves four local Geist font paths outside basePath.
Development acceptance now recognizes only those exact local framework paths;
all Zellij and remote font requests remain checked, and production has no
exception. That run's main browser suite had 310 initial passes plus one known
FAQ retry and 124 conditional skips. Consult PR #10 for final acceptance.

Prior milestone completion: [PR #9](https://github.com/mylife-inc/Zellij/pull/9)
merged as `bd01bdb`. Main CI `39052261993` passed all four jobs, including **311
browser tests**, 124 conditional skips and **no retries**. Chromium/WebKit
production-prefix and dev-watcher acceptance passed. Image run `39052261989`
passed eight platform/feature builds, four multi-platform publications and the
default-image build/serve smoke test. This successful run does not diagnose the
previous FAQ/WebKit flakes. PR #9's final PR run had one successful FAQ retry.

PR #10 completion: merged as `5a9e12c` after exact-head run `39055211438`
passed all four jobs: **324 engine tests**, **117 Rust tests**, schema sync,
typechecks, warnings-denied Clippy, offline Turbopack build, and **311 browser
passes / 124 conditional skips / no retries**. Main CI `39055788092` passed;
its browser suite had 310 initial passes and one successful known mobile-WebKit
FAQ retry. Both runs passed Chromium/WebKit production and development font,
license and prefix gates. Image run `39055788338` passed eight feature/platform
builds, four AMD64/ARM64 manifest publications and the default-image build/serve
smoke test. This closes S13's hosted acceptance; it does not diagnose the FAQ
flake or constitute a native CLI release.

### Truthful CLI inspection output — 2026-10-17

Source commit `4d659b4` resolves the reproduced **R9** behaviors. Rust `zel
context --json` emits the exact embedded engine JSON; supplying a filename
writes that format to the file with confirmation on stderr. `list themes`
lists all 14 names, labels and descriptions, generated from the engine's config
names and renderer tokens. The catalog is included in schema-sync CI.

JSON is implemented for every `list`, `explain` and `detect` as well. Explain
returns the selected full section schema. Lists return named arrays with stable
identifiers; detect returns the supplied source path and ordered matches with
numeric confidence and strength. Missing/non-directory sources fail instead
of masquerading as no match. Equal-confidence detection and converter listings
are deterministic across inventory registration order. See the CLI spec's
output contract and command docs for exact fields.

Other commands reject `--json` with exit 2 **before execution**. Unimplemented
`--verbose` / `-v` is removed from the parser/help and rejected, rather than
silently ignored. Errors remain human messages on stderr; JSON validation,
conversion/build reports, verbose tracing and config loading remain future work.
The existing Markdown context and human inspection formats are retained.

Local acceptance: **125 Rust tests**, warnings-denied Clippy and formatting,
**325 engine tests**, all workspace typechecks, engine package build and deep/
strict docs validation pass. Eight new binary integration tests run with no
external tools on PATH and cover exact embedded output, file/stdout/stderr
behavior, all section schemas, theme/config agreement, detection, error codes,
and refusal before command actions. A first engine run exposed a missing test
fixture name (corrected); a subsequent default-concurrency run hit two existing
math-rendering timeouts during a full Rust rebuild. The unchanged suite passed
with two local workers and unchanged timeouts. Hosted full acceptance remains
the next gate; no native binary release is part of this milestone.

Hosted completion: [PR #11](https://github.com/mylife-inc/Zellij/pull/11)
merged as `0a89131` after final source/help head `c411317` passed CI
`39059091256`. Main CI `39059796288` also passed all four jobs: 125 Rust tests,
325 engine tests, typechecks/schema sync, zero npm advisories, and **311 browser
passes with 124 conditional skips and no retries**. Container/scaffold/conversion,
font/license, production-prefix and development-watcher acceptance passed.
Image run `39059796306` passed all 13 jobs, publishing all four feature variants
for Linux AMD64/ARM64 and passing the default-image smoke test. No native binary
release was issued.

### Native portable Markdown build — 2026-10-17

Source commit `8f72022` adds the native `zellij-docs` crate and
`zel build --target portable`. A plain Markdown folder selects it automatically;
discovered structured Zellij content retains the hosted static default. The
renderer parses Markdown with pulldown-cmark, highlights code with syntect, and
writes full HTML pages with ordinary relative links. Embedded local scripts
provide search, theme selection and mobile navigation without fetch/router data.
The 14 themes are generated from engine tokens; the same pinned 75-font snapshot
and license notices are embedded by Cargo. Building needs no Node, Docker or
network. Source checkout builds do require the engine font snapshot in this repo.

Output defaults to a sibling `<input-name>-site`; `--out`, `--name` and `--theme`
configure publication. Index/README home selection, optional front-matter title/
description/order, directory links, heading/footnote fragments, referenced
assets, synthetic home, breadcrumbs and previous/next are implemented. The
shared staged publisher moved from `zellij-engine` to `zellij-core::output`
without changing the ownership/failed-build-preservation contract. Source and
output are disjoint; nested symlinks, missing references, duplicate outputs and
Windows-incompatible names fail before replacing output. URL paths normalize
Windows separators at the filesystem boundary.

Current limits: raw HTML is displayed literally with a warning; MDX is rejected.
Math rendering, Mermaid execution, custom navigation/config, portable watch/reload,
`serve` and `view` remain unfinished. External media is not downloaded. The
[portable contract](specs/zel-portable-docs.md) and CLI build docs carry the exact
behavior and capability comparison. This is not complete MkDocs compatibility.

Local acceptance: **137 Rust tests**, formatting, warnings-denied Clippy,
generated-theme parity, deep/strict docs checks and Chromium file acceptance
pass. The browser fixture builds without tools on PATH, preserves an old export
on failure, moves output to a space/Unicode path, opens home and deep links with
networking blocked, and checks search/anchors, local fonts/media, keyboard,
themes, mobile layout and JavaScript-disabled content. Review caught and fixed
Escape clearing the search field instead of closing its dialog. CI now includes
native portable build/CLI/browser jobs on Linux, macOS and Windows, each with
Chromium and WebKit. Hosted results are pending at this record; these jobs do
not establish full native Windows container support or release/install coverage.

Browser follow-up: the first hosted run passed Chromium but WebKit rejected
its initial file navigation with offline emulation enabled. A minimal local
file containing only an H1 reproduced that failure; HTTP-blocked file loading
without emulation passed. WebKit acceptance therefore blocks HTTP requests
explicitly and asserts that every observed request uses file/data URLs; Chromium
also retains offline emulation. Both JavaScript and no-JavaScript contexts use
these guards. No file-security flags or HTTP fallback are used. Both engines
now pass locally. WebKit additionally exposed missing focus restoration after
pointer-opened search; the runtime now records and restores the opening control.

Portable-build completion: [PR #12](https://github.com/mylife-inc/Zellij/pull/12)
merged as `680779f` after final head `99c0f54` passed all seven jobs in
`39063380795`. Native Linux/macOS/Windows portable checks passed in Chromium
and WebKit. PR browser acceptance had 310 initial passes plus one successful
retry of the existing mobile-WebKit FAQ test, with 124 conditional skips.
Main CI `39064011730` passed all seven jobs, including 138 Linux Rust tests,
325 engine tests and **311 browser passes, 124 skips, no retries**. Image run
`39064011643` passed all 13 jobs and published four AMD64/ARM64 variants. The
local debug binary was rebuilt from main; no native release was issued.

### Native serve and view — 2026-10-17

Source commit `e33cd92` implements `zel serve` and `zel view`.
`zellij-docs::server` uses tiny_http to serve existing static exports on IPv4
loopback. It supports selected and ephemeral ports, MIME types, GET/HEAD,
extensionless HTML, directory indexes/redirects, query preservation, custom
404 responses and an explicit `--base-path` mount. It serves bytes unchanged,
disables caching for preview and requires a regular root index.html. Traversal,
malformed encodings, metadata paths, nested symlinks and Windows junctions are
rejected; foreign Host headers are refused. There is no directory listing,
home-page fallback, or automatic prefix rewriting.

`zellij-cli/src/preview.rs` orchestrates the workflow. `view` reserves its port
before building and calls the same portable helper as `build --target portable`.
It builds plain Markdown only, prints the actual URL, and launches the system
opener in a separate thread. `--no-open` suppresses launch; `--browser` selects
one executable receiving the URL as one argument, without a shell. Missing or
failing launchers warn while HTTP keeps running. Source files remain unchanged;
build failures preserve owned output, and port conflicts fail before rebuilding.

Ctrl-C (plus POSIX termination signals) stops normally with exit 0 and releases
the listener. Response writers are bounded and do not block the control loop;
shutdown can interrupt in-flight responses and retains all exported files.
Neither serve nor view watches/rebuilds/reloads on edits. Portable raw HTML/MDX,
math/Mermaid, richer navigation/config, and native release/installer work remain
open. This preview server is not a public-hosting/TLS/authentication service.

Local evidence: **143 Rust tests**, warnings-denied Clippy, formatting and
updated deep/strict docs validation pass. Six new real-HTTP tests cover routing,
MIME/HEAD/methods, port and prefix behavior, path/symlink containment and shutdown.
`scripts/check-preview.mjs` checks the actual CLI without external tools on PATH,
both browsers under a URL prefix, launch readiness/errors/no-open, source and
failed-build preservation, and Ctrl-C with slow/incomplete requests. A compiled
browser substitute requests the ready URL before recording launch; automated
acceptance does not claim to exercise each machine's actual default GUI browser.
Windows CI uses `scripts/fixtures/preview-console.rs` to create a private console
for the tested child and send a real Ctrl-C event, rather than Node's force-kill.
Cross-platform hosted acceptance is pending at this record.

Native-preview completion: [PR #13](https://github.com/mylife-inc/Zellij/pull/13)
merged as `bc95346` after head `4b4da4c` passed all seven jobs in
`39066025208`. Main CI `39066685507` also passed all seven jobs: 144 Linux
Rust tests, 325 engine tests, 311 website browser passes, 124 conditional skips,
and no retries. Both Chromium and WebKit passed native direct-file and HTTP
preview on Linux/macOS/Windows, including launcher failure recovery and real
Ctrl-C. Image run `39066685512` passed all 13 jobs and published all four
AMD64/ARM64 feature variants. No native binary release was issued.

### Portable watch and live reload — 2026-10-17

Source commit `2c44098` implements `zel view --watch`. Ordinary `view` remains
one-time build/serve/open; `serve` remains unchanged-file preview. The watcher
captures the resolved source before the initial build, polls streamed file-byte
hashes, waits for two matching scans, and serializes full rebuilds. Hidden entries
and node_modules are ignored. Same-size/preserved-timestamp edits are detected.
Nested links, Windows junctions, special files and replacement root links are
refused. Missing roots and invalid documents/assets recover on subsequent edits.
Large trees require full scans/builds; no incremental compiler is implemented.

`zellij-docs::prepare` returns a staged `PreparedBuild`; ordinary `build` publishes
it immediately. Watch mode publishes under the shared `LiveReload` write lock,
while HTTP holds a read lock through file resolution/opening and revision
attachment. Responses do not hold it while streaming to clients. This excludes
the publication rename gap from preview reads, without making multiple HTTP
requests a single generation transaction. Failed builds retain the last export
and revision and report their error in the terminal and a browser notice.

Only watch HTTP responses receive `_zellij/preview.js` and revision polling.
The version embedded in each HTML response detects updates even before its
first poll. Success reloads the current URL, preserving query/fragment; deleted
page 404 previews can recover on restoration. Files on disk receive no preview
client or endpoint and remain movable/offline. Ctrl-C joins the worker and
cleans staged work before reporting completion; exported files remain. Name,
theme, output and port changes require restarting the command.

Local evidence: **147 Rust tests**, formatting, warnings-denied Clippy, strict/
deep docs validation, and actual CLI acceptance in Chromium and WebKit pass.
Four added Rust regressions cover snapshots (including preserved timestamps and
links), staged publication, HTTP revisions/endpoints/HEAD and unchanged output.
The browser/process fixture additionally exercises the first-poll race, atomic
saves, added/deleted pages, changed/missing assets, search refresh, error notice
and recovery, temporary source loss, symlinks/junctions, ignored trees, continuous
HTTP availability during publication, Ctrl-C, and moved offline exports. CI
uses the same native Linux/macOS/Windows matrix and real Windows console helper.
Hosted acceptance is pending at this record. No native release is issued.


Hosted follow-up: run `39068473117` exposed a race-fixture deadlock on Linux
and macOS. The test held the first preview-status response but awaited the
page's full load before allowing that response to continue. It now waits for
DOMContentLoaded plus the actual intercepted request before editing, then
releases the response only after publication. Reload assertions and timeouts
are unchanged; production code is unchanged. Final hosted status belongs to
[PR #14](https://github.com/mylife-inc/Zellij/pull/14).


Run `39068818910` passed the corrected race on Chromium, but Linux WebKit also
held DOMContentLoaded while a network request was intercepted. The final fixture
therefore delays only the first fetch initiation in the page. It waits for the
client to attempt that poll, changes the source, observes actual publication,
and then permits the real HTTP response. A session marker removes the delay on
reloaded documents. Production code, reload assertions and timeouts are unchanged.


Run `39069154907` passed Linux/macOS native acceptance, all broader checks,
and Windows Chromium watch acceptance (including real Ctrl-C). Windows WebKit
still timed out awaiting DOMContentLoaded in the deliberately paused first-poll
scenario. The fixture now waits for navigation commit, then explicitly requires
the rendered heading and attempted poll before editing. Failed watch runs retain
Playwright traces plus browser readiness and CLI output for diagnosis. Full
post-reload navigation, search, image and offline assertions remain in place.


The trace from `39069833700` confirmed successful publication and browser reload:
the page contained the updated heading and the new revision. It exposed an early
search shortcut sent before deferred page enhancements initialized. The fixture
now waits for the controls that site.js reveals before pressing the shortcut;
static heading visibility alone is not JavaScript readiness. Production code is
unchanged, and the first-poll race still must complete before these search checks.


HTTP transport correction: traces from `39070119126` showed requests for
site.js/search.js receiving no response, so the browser could not start its
first poll. The earlier lifecycle/interception explanations above were
provisional and did not explain the underlying failure. A standalone local
64-connection burst reproduced missing responses in ordinary `serve`; closing
answered keep-alive sockets immediately released the stuck requests. Inspection
of tiny_http's task queue found that queued connections can outnumber idle
workers, leaving connections unprocessed while those workers wait on persistent
clients. This affects PR #13's server as well as watch mode.

Source commit `6f14500` replaces the transport with Hyper HTTP/1 and Tokio:
idle sockets are async tasks, file
lookups use a bounded 16-thread blocking pool, and file bodies stream with
backpressure. A maximum of 256 connections, a 10-second header-read timeout and
cancellation on shutdown bound the preview service. Existing routing, host/path
checks, byte-preserving serving, HEAD lengths and watch publication locks remain.
The new regression queues 64 persistent requests plus idle preconnections,
requires all responses without closing other clients, then reuses the sockets
and shuts down with them open (eight bursts). Its initial 64-client version
failed against tiny_http after five seconds; the replacement passed the server
suite. Full local validation now passes: **148 Rust tests**, warnings-denied
Clippy, formatting, deep/strict docs checks, and Chromium/WebKit actual CLI
HTTP/watch plus moved direct-file acceptance. Cross-platform hosted CI remains
pending at this entry; see PR #14 for the final exact revision and results.

Watch/reload completion: [PR #14](https://github.com/mylife-inc/Zellij/pull/14)
merged as `142965f` from final head `4f20092`. PR CI `39071150585` and merged-main
CI `39071818383` each passed all seven jobs. Native Linux/macOS/Windows passed
both Chromium and WebKit offline/HTTP/watch acceptance, including real Windows
Ctrl-C. The CLI job passed 149 Linux Rust tests; engine tests remained 325, and
website browser acceptance passed 311 tests with 124 conditional skips and no
retries. Image run `39071818346` passed all 13 jobs and published all four
AMD64/ARM64 variants. Native release packages were not issued.

### Portable configuration, navigation and branding — 2026-10-17

Source commit `4ba149f` adds optional `<Markdown-root>/zel.yaml` for site name,
initial theme, local company logo/link and nested navigation. It has no parent
search. A present file selects portable automatic build dispatch before hosted
marker discovery; explicit targets still win. CLI name/theme flags override
file values, then folder-name/cupertino defaults apply. The entire file must
remain valid even when some settings are overridden.

Navigation entries contain exactly one of `page`, `children`, or HTTP(S) `href`.
Source-relative page filenames are exact and retain literal spaces/Unicode.
Groups/links require labels; duplicate or missing pages, empty groups, traversal,
unknown/duplicate keys and incorrectly typed values fail. Nesting is bounded at
16 groups. Configured order and aliases drive sidebar and previous/next links;
unlisted pages are still rendered, validated and searched, but have no paging.
Omitting nav keeps automatic ordering; an empty list intentionally hides links.
A synthetic home lists authored navigation. Folder breadcrumbs stay unchanged.

Logo paths are checked for containment, image type and symlinks/junctions, copied
locally, and linked with encoded relative URLs. The company logo can open its
HTTP(S) website while the adjacent title links to documentation home. Mobile
controls occupy a separate row to preserve title/logo space. Navigation and
branding remain usable without JavaScript and after moving a direct-file export.

Watch mode rereads configuration on creation, edit and deletion. Invalid settings
preserve the last export and show the existing preview error notice; recovery
rebuilds/reloads. Browser preferences now save only explicit visitor choices,
so new project defaults take effect until overridden. The new versioned storage
format may reset an older saved choice once. General `zel.toml` defaults remain
planned; this portable format is distinct from hosted `site.yaml`.

Local verification: 154 Rust tests, warnings-denied Clippy, formatting, deep/strict
documentation validation and a fresh CLI build passed. Chromium/WebKit passed
moved file://, HTTP and watch acceptance, including config addition/removal,
error recovery, theme precedence, logo loading, nested navigation, alias paging,
unlisted search, mobile layout and no-JavaScript use. Visual inspection caught
mobile brand compression; the fixed layout and geometry assertions pass in both
browsers. The existing three-platform CI matrix runs these added checks. Hosted
acceptance is pending at this dated entry; consult the PR containing `4ba149f`
for its final revision/results. No new native release packages are issued.

Configuration completion: [PR #15](https://github.com/mylife-inc/Zellij/pull/15)
merged as `7b7222f` from head `92f2d10`. PR CI `39075396282` and main CI
`39076012865` each passed all seven jobs, including Chromium/WebKit portable,
HTTP and configuration/watch acceptance on native Linux/macOS/Windows. PR CI
passed 155 Linux Rust tests and 311 website browser tests (124 conditional skips,
no retries). Image run `39076012860` passed all 13 jobs and published all four
AMD64/ARM64 variants. The local binary was rebuilt; no native release issued.

### Portable publication paths — 2026-10-17

Source commit `90befda` fixes a publication/watch inconsistency found during
final review. At `7b7222f`, a configured `node_modules/logo.svg` was successfully
copied into an export, but watch snapshots exclude that directory and would not
notice edits. Explicit Markdown asset links also bypassed hidden/dependency
exclusions. Discovery, watch snapshots, logo configuration and referenced assets
now share the same excluded-name rule. Hidden paths and `node_modules` cannot be
published through those references. Authors must move published assets into a
normal source folder such as `assets/`.

Local verification: 18 configuration/portable/watch tests, warnings-denied Clippy,
formatting and deep/strict docs validation passed. Regression fixtures contain
real excluded files and check image/download references, logo configuration,
source preservation and preservation of the previous export. Hosted acceptance
is pending at this dated entry; consult the PR containing `90befda` for results.

Publication-path completion: [PR #16](https://github.com/mylife-inc/Zellij/pull/16)
merged as `a6a9a7d` from head `e188a13`. PR CI `39076533168` and main CI
`39077026320` each passed all seven jobs; image run `39077026408` passed all
13 jobs and published all four AMD64/ARM64 variants. Linux ran 156 Rust tests.
Native Linux/macOS/Windows Chromium/WebKit file, HTTP and watch checks passed.
The website suites each had 310 initial passes, 124 conditional skips and one
successful mobile-Safari retry: the PR's testimonial-carousel slide did not
become active within 5 seconds; main's FAQ answer remained hidden after its
first click. These are recorded flakes, not diagnosed or fixed by this work.
No native package release was issued.

### Native portable validation — 2026-10-17

Source commit `7457daf` adds `zel check [dir] --target portable|static`, native
strict warnings and portable JSON reports. Automatic selection shares build's
helper: a root `zel.yaml` entry wins, then hosted discovery, otherwise portable.
The existing hosted engine checker remains available; explicit static is useful
for unmarked hosted folders. Hosted JSON is rejected before container execution.

`zellij-docs::check` and build now share a complete in-memory compiler. Check
performs configuration/discovery/Markdown/navigation/reference/render/output-path
validation without staging or writing files. Build then stages/publishes as
before. Virtual file/directory output collisions are caught before filesystem
writes, so checks can detect them too. Existing output ownership is irrelevant
to a check; it does not promise output permissions, disk capacity or publication.

Errors are collected per document during read/parse, then across references
when parsing succeeds. Fatal config/discovery/render/output errors stop their
phase. Raw HTML stays a warning; strict makes it fail without changing severity.
`--deep` is accepted but redundant for portable content, whose local references
are always checked. MDX and external URL availability remain outside this target.

Portable JSON emits one stdout report on validation success/failure with no
stderr diagnostics. Schema version 1 includes target/root/strict/ok, nullable
HTML page count and diagnostics with severity/code/nullable source-relative file/
message. No guessed line/column fields. Codes: source, configuration, markdown,
reference, output-path, rendering, raw-html. Exit 0 success, 3 content/strict,
1 returned I/O failure; first fatal error controls status. Usage errors use
stderr and exit 2. See the portable contract and `zel check` documentation.

Local verification: **164 Rust tests**, warnings-denied Clippy, formatting,
updated deep/strict docs checks, and fresh binary build passed. Chromium/WebKit
passed moved file:// navigation/search/branding/no-JS, HTTP preview and full
watch/configuration/recovery/Ctrl-C acceptance with the refactored compiler.
Nine new library/CLI tests verify check/build parity, multi-file errors,
structured output/exit behavior, target selection, strict warnings, collision
failures, and byte-preservation of source and existing output. The CLI tests
remove external tools from PATH. All-platform CI includes native checker tests.
Hosted acceptance is pending at this dated entry; consult the PR containing
`7457daf` for its final head/results. No native package release is issued.

Native validation completion: [PR #17](https://github.com/mylife-inc/Zellij/pull/17)
merged as `c391dc3` from head `df82cdf`. PR CI `39078577656` and main CI
`39079171966` each passed all seven jobs, including native Linux/macOS/Windows
portable validation and Chromium/WebKit acceptance. Linux ran 165 Rust tests;
the engine ran 325. PR website acceptance had 310 initial passes, 124 skips and
one successful mobile-Safari FAQ retry; main passed 311 tests with 124 skips and
no retries. Image run `39079171941` passed all 13 jobs and published all four
AMD64/ARM64 variants. Local main/binary were updated; no native release issued.

### Hosted reveal and FAQ interaction — 2026-10-17

Source commit `9acd934` fixes the audit's **S4** entrance-content visibility
failure and a reproduced FAQ scrolling race. The changes affect the hosted
engine; native portable rendering is unchanged.

The browser investigation distinguishes established defects from the earlier CI symptom. At `c391dc3`, 30 instrumented ordinary mobile-WebKit FAQ trials and 60 original FAQ/keyboard/carousel repetitions all passed locally. The earlier CI first-click FAQ/carousel failures therefore have not been reproduced exactly or conclusively attributed to one cause.

A related lost-click race is reproducible with a 200 ms pointer-down/up interval: 4 of 20 mobile-WebKit FAQ trials failed to close the answer. Event traces show the summary moving hundreds of pixels during a press and the click being retargeted to a surrounding section/div. A comparison changing only document scroll-behavior from smooth to auto passed 20/20. The product now uses immediate document scrolling to prevent default focus/scroll-into-view movement continuing through a press; entrance effects and horizontal carousel rules remain. The existing FAQ test now uses a real 200 ms press without action retries.

The audit's S4 is independently confirmed: the new actual JavaScript-disabled browser regression fails against the baseline because entrance wrappers have opacity zero. Reveal now emits visible `pending` server markup, installs its observer before arming offscreen entrances, and never hides content already in view when hydration arrives. The six new cross-project browser cases verify visible wrappers with JavaScript disabled and native FAQ operation, plus opening a FAQ while Next scripts are held and preserving that state when scripts are released.

CI now retains first-failure traces rather than tracing only a retry, and uploads failure traces/error context even when a later retry succeeds. This enables evidence-based diagnosis if the historical first-click flake recurs. Passing current acceptance is not proof that every prior flaky failure had the same cause.

Local verification: **325 engine tests**, all workspace typechecks, engine
packaging, a normal Turbopack production build, and deep/strict documentation
validation passed. Full production browser acceptance passed **317 tests**,
with **124 conditional skips and no retries**, across desktop/mobile Chromium
and mobile WebKit. The corrected product passed **20/20** instrumented 200 ms
press trials with its actual styles (no injected scrolling override). The six
new no-JS/delayed-hydration cases also passed directly. No-JS coverage asserts
computed opacity/transform, not only Playwright's visibility check, which allows
opacity zero. See [Playwright's definition](https://playwright.dev/docs/actionability#visible).

Baseline failure traces and experiment logs were captured under
`/private/tmp/zellij-faq-*` and `zellij-reveal-*` on this machine. The important
reproduction and limits are recorded here because temporary evidence may expire.
Hosted acceptance is pending at this dated entry; consult the PR containing
`9acd934` for its final revision/results. Native installers, S5/S14/S15 and the
remaining portable capabilities are still open; the original sporadic CI
first-click failures must not be marked conclusively diagnosed by this evidence.

### Native portable starter — 2026-10-17

Source commit `6af6a48` adds `zel new portable [dir]`, defaulting to `docs`.
The full starter supplies `zel.yaml`, a home page, a nested getting-started
page and its local SVG. `--minimal` emits just config and home. `--name`
defaults to the resolved destination's name, including `.`, and `--theme`
defaults to cupertino. The 14-theme catalog and nonempty/control-free names
are validated before writing. Display names use quoted metadata, never raw
Markdown or shell interpolation. Instructions use commands from inside the folder.

This is a native Markdown scaffold, distinct from hosted `new docs/page/guide/section`.
It needs no Node, Docker or network. Shared scaffold preflight refuses nonempty
folders and generated-file collisions; existing dotfiles and fresh Git metadata
are preserved. No overwrite option is offered. Both starters pass native strict
checking and build without warnings. Their configuration carries working ordered
navigation; authors may remove `nav` for automatic discovery. MDX, raw HTML
execution, math rendering and Mermaid remain unsupported by the native target.

Local verification: **169 Rust tests**, formatting and warnings-denied Clippy
passed. The full native offline browser fixture passed Chromium and WebKit,
including actual full/minimal starter output moved to paths with spaces/Unicode,
literal unusual display names, fragments, local images, search and JavaScript-disabled
navigation. CLI tests run without external programs on PATH, cover every theme,
invalid arguments, collision/source preservation, current-directory naming and
dotfiles. Deep/strict validation of the updated product documentation passed.
The cross-platform CI matrix now runs these starter tests and browser acceptance.
Hosted acceptance is pending at this dated entry; consult the PR containing
`6af6a48` for the exact checked revision and final results. No native binary
release or installer is published by this milestone.

Prior milestone completion: [PR #18](https://github.com/mylife-inc/Zellij/pull/18)
merged as `ec0394a`. PR CI `39080731998` and main CI `39081222627` passed all seven
jobs, each with **317 browser passes, 124 conditional skips and no retries**.
Main native acceptance passed Linux/macOS/Windows. Image run `39081222598` passed
all 13 jobs, publishing four variants for linux/amd64 and linux/arm64 plus the
published-image smoke test. This does not conclusively attribute every historical
first-click flake to the reproduced scrolling race. Live repository/release facts
were refreshed: public/default-main; latest native release remains `zellij-v0.1.3`.

### Native packages and installers — 2026-10-17

[PR #20](https://github.com/mylife-inc/Zellij/pull/20), source `94b2d8d` and
compatibility correction `489ad35`, adds one release packaging path and a reusable CI matrix for
x64/ARM64 on GNU/Linux, macOS and native Windows. Each runner builds the locked
release profile, packages the executable and checksum, installs that actual
archive through a loopback HTTP mirror, and exercises the installed binary.
Windows ZIPs use a static MSVC runtime; `install.ps1` supports Windows PowerShell
5.1 and PowerShell 7. Ordinary CI uploads artifacts without publishing a release.
Tagged releases call the same workflow after requiring a tag matching the
committed Rust workspace version. Both installers accompany release assets.

The Unix installer now validates the candidate's executable version before
replacement and stages in a unique destination file. Previously it replaced the
installation before checking execution and used an unsafe fixed `zel.new` name.
Both installers test checksums, malformed archives, version mismatches, upgrades,
existing destination conflicts and preservation of prior/unrelated files.
Neither edits PATH. Installed portable starter/check/build/serve/view acceptance
runs without external tools on PATH and verifies real Ctrl-C, including Windows.
See [the distribution contract](specs/zel-native-distribution.md) for exact OS
baselines, artifact conventions, installer options and limitations.

Local verification: the actual x86_64 macOS release archive passed all installer
and installed-command tests. Combined checksum generation, shellcheck, actionlint,
JavaScript/shell syntax and deep/strict product-documentation checks passed.
The first hosted run built both Windows binaries but found that PowerShell 5.1
could not resolve `Get-FileHash`. `489ad35` uses built-in .NET SHA-256 instead,
passes an explicit .NET null for replacement backup paths, and strengthens mirror
URL/token-isolation regression checks. Linux x64/ARM64 and Apple Silicon package
acceptance passed that initial run. Subsequent run `39084948928` passed all six
package/install targets, including both Windows shells on both architectures.
Its separate macOS watch test exposed an image-property evaluation losing its
execution context during an expected reload. The test now uses reload-aware
locator property assertions without weakening the required image widths.
Consult PR #20 for the final exact revision and complete hosted results. The separate
portable browser matrix remains the evidence for real offline browser behavior.
This milestone does not tag or publish binaries, sign/notarize packages, establish
all historical OS versions, or validate every Windows container integration.
The existing public `zellij-v0.1.3` assets must not be overwritten.

Prior milestone completion: [PR #19](https://github.com/mylife-inc/Zellij/pull/19)
merged as `4735144`. PR/main CI `39082442519` / `39082937391` passed all seven jobs,
with **317 browser passes, 124 conditional skips and no retries**. Image run
`39082937383` passed all 13 jobs, publishing four variants for both Linux
architectures and passing the published-image smoke test. The repository remains
public/default-main; the latest native release remains `zellij-v0.1.3`.

### Theme preferences, prefixed video and guide accessibility — 2026-10-17

Source `f35cc92` fixes audit S5/S14/S15 in the hosted engine. The root layout
restores mode and theme independently, according to the enabled controls. It
accepts only known modes/themes, preserves configured defaults for disabled
controls, and tolerates unavailable storage. A shared ordinary module owns the
fixed initialization script; configuration reaches it through escaped data
attributes. The mode button also follows changes to the system color preference
when computing its accessible action label.

Local video sources and posters now receive `site.config.basePath`; external
video/embed URLs remain intact. Guide h2/h3/h4 permalinks derive their accessible
names from the visible, potentially formatted title. The decorative hash is
hidden from assistive technology and appears on hover or keyboard focus.

Local verification: **329 engine tests**, workspace typechecks, engine packaging
and deep/strict documentation validation passed. The production prefix gate
passed Chromium and WebKit, including four independent theme-control settings
before hydration and after navigation/reload, invalid/blocked storage, system
preference changes, formatted heading names and keyboard permalinks, and actual
decoding of a prefixed local video/poster. The new video unit regression failed
on the old local-media renderer and passes with the fix. This does not establish
manual screen-reader acceptance or exhaustive accessibility/SEO/performance.
The first hosted run passed the new production checks but found an old watcher
assertion comparing raw heading text, which now includes the decorative hash.
That check now verifies visibility and the exact accessible name; the corrected
development watcher acceptance passed locally in Chromium and WebKit. Consult
[PR #21](https://github.com/mylife-inc/Zellij/pull/21) for its final checked head
and hosted results.

Prior milestone completion: [PR #20](https://github.com/mylife-inc/Zellij/pull/20)
merged final head `434f5eb` as `937331c`. PR CI `39085300006` and main CI
`39085863182` passed all 13 jobs, including all six native package/install targets
and both PowerShell versions on Windows. Each showcase run passed **317 tests,
124 conditional skips and no retries**. Image run `39085863219` passed all 13
jobs, publishing four variants for both Linux architectures and verifying the
published-image build/serve smoke test. No native binary release was published;
the latest public release remains `zellij-v0.1.3`.

### Page themes and Plausible analytics — 2026-10-17

Source `2823164` implements the two accepted-but-inactive settings in audit S3.
A sections page's `theme` now emits server-rendered tokens scoped to its main
content. Shared chrome and guides keep the site theme. The page follows global
light/dark/system mode and site token overrides; an explicit page theme wins
over the visitor's site-theme choice within that page and never changes storage.
Its anchor offsets retain the real header height. Theme-specific presentation
uses CSS `@scope` boundaries so a parent theme's rules cannot leak into an override.

Production hosted sites with `analytics.provider: plausible` lazily initialize
the official `@plausible-analytics/tracker` package, pinned to 0.4.6, once per
document. Its native history integration counts initial/path/history pageviews;
fragment navigation is not a new pageview. Default `none` and dev builds do not
initialize it. The provider's localhost/file and localStorage opt-out exclusions
remain intact. All requests use the default Plausible event endpoint. No user
site was switched to analytics, and tests intercept all events rather than send
them to Plausible. Custom endpoints/providers and native portable analytics are
not part of this contract.

Plausible now requires a nonempty domain. The schema represents its conditional
requirement as a union; the generated Rust JSON reference and Markdown context
include both choices. The context renderer now expands object-union branches
instead of documenting only the first. Documentation clarifies page-theme
precedence and analytics behavior, removes the unsupported Umami claim and fixes
the invalid footer-social example.

Local evidence: **338 engine tests**, all workspace typechecks, final engine
packaging, deep/strict product-documentation validation and **10 Rust core tests**
passed. Four regressions failed against the old source (three missing/empty-domain
cases and absent server-rendered page themes). Production Chromium/WebKit checks
passed real page styles/fonts, independent picker settings, token overrides,
mode changes, client navigation/reload, unthemed/guide destinations and disabled
JavaScript. The real Plausible library passed intercepted request-count/domain
checks, client navigation/history, disabled/ignored/local visits and blocked
requests. Both dev browser engines confirmed analytics remains uninitialized.
Initial new-fixture failures were an invalid four-CTA hero and reloading before
navigation completed; they were corrected without weakening product assertions.
No live Plausible dashboard receipt, exhaustive legacy-browser support, or broad
accessibility/SEO/performance acceptance is established. Consult the PR containing
`2823164` for its final revision and hosted results.

Prior milestone completion: [PR #21](https://github.com/mylife-inc/Zellij/pull/21)
merged head `1cb9d9a` as `8f10224`. PR/main CI `39088215253` / `39088773090` passed
all 13 jobs; both showcase runs had **317 passes, 124 conditional skips and no
retries**, plus production-prefix and development-watcher acceptance in Chromium
and WebKit. Image run `39088773074` passed all 13 jobs, published all four variants
for both Linux architectures and passed the published-image build/serve smoke test.
Repository facts were refreshed: public/default-main; latest native release is
still `zellij-v0.1.3`. No new native binary version is published by this milestone.

### Portable mathematics — 2026-10-17

Source `80174ac` adds opt-in native math through `markdown: {math: true}` in
portable `zel.yaml`. The default remains false. Pulldown-cmark recognizes inline
and display equations; pinned `math-core` 0.9.0 renders MathML during the shared
check/build compiler. No browser typesetter, Node, TeX installation or download
is required. Titles, TOC/slugs, search and image alt text retain equation source.
The font and styling load only on math pages, and sites without equations omit
the math assets entirely. The embedded official STIX Two Math 2.13b171 font and
licenses have pinned provenance/checksums in `cli/crates/zellij-docs/assets/math/`.

Invalid syntax, unsupported commands and unresolved equation references fail
with a source-file `rendering` diagnostic and expression ordinal; failed builds
preserve the last export. Each expression has local definitions, numbering and
references, with separate reserved ID namespaces. Cross-expression references,
full TeX/hosted MathJax parity and Mermaid execution remain unsupported. Raw HTML
policy is unchanged; generated math escapes text/annotations. Display math can
scroll horizontally by keyboard on small screens. `view --watch` rereads the
setting and recovers from invalid formulas using the existing publication path.

The CLI and `zellij-docs` now declare Rust 1.96 for source builds because of the
renderer dependency; helper crates retain 1.85. Three existing nested conditionals
were simplified to satisfy the Clippy checks enabled by the newer minimum.
CI now checks the whole workspace with Rust 1.96 as well as stable quality gates.
Downloaded binaries do not need a Rust installation. No binary version is released.

Local evidence: **176 Rust tests**, formatting and warnings-denied Clippy pass.
Deep/strict product docs validate 47 routes and 23 section examples. Offline
Chromium/WebKit acceptance passes relocated nested files, actual font decoding
and checksum, fraction geometry/matrices, search/navigation, inherited theme color,
HTML-like equation text, mobile scrolling and JavaScript-disabled rendering.
Real HTTP/watch acceptance passes enabling/disabling math, invalid-formula
preservation/recovery and clean shutdown. Four initial feature tests failed on
the old source. The new browser fixture needed to allow existing theme URL
parameters and wait for WebKit's resized layout; assertions were retained.
Consult the PR containing `80174ac` for final hosted/platform/minimum-Rust results.
This is not exhaustive math layout, older-browser or manual screen-reader acceptance.

Prior milestone completion: [PR #22](https://github.com/mylife-inc/Zellij/pull/22)
merged head `ac2bd54` as `0a52fdd`. PR/main CI `39090482991` / `39091005544` passed
all 13 jobs. Both showcase runs had 317 passes and 124 conditional skips, with
separate production page-theme/analytics/prefix and dev-watcher acceptance in
Chromium/WebKit. Image run `39091005435` passed all 13 jobs, publishing four
variants for Linux amd64/arm64 and passing published-image build/serve acceptance.
Repository facts were refreshed: public/default-main, latest native release
still `zellij-v0.1.3`. No user action is needed for this milestone.

### Portable YAML mindmaps — 2026-10-17

Source `a140800` adds built-in `mindmap` fences to native portable Markdown.
The compiler emits finished SVG and a keyboard-operable nested-list outline;
no new dependency, Node, browser renderer or network is needed. It uses the
hosted engine's documented single-root YAML hierarchy syntax, with strict
string keys and bounded source, labels, nodes, depth and alias expansion.
Invalid diagrams produce source-file/diagram-number `rendering` diagnostics
before publication. Raw labels are escaped; authored Markdown is unchanged.

Only diagram pages link the small stylesheet, and removing the last diagram
removes its asset. Theme colors update through CSS; wide diagrams scroll at
fixed readable label sizes. Layout uses conservative UTF-8 width estimates,
not hosted geometry or complex-script shaping. The existing local script adds
Left/Right and Home/End handling because a minimal WebKit overflow region also
failed native arrow scrolling. With JavaScript disabled, SVG still renders and
the native text-outline disclosure exposes the full hierarchy by keyboard.
Mermaid fences remain literal code; Mermaid execution is still separate work.

Local evidence: **182 Rust tests**, formatting, warnings-denied Clippy and
47-route/23-example deep-strict docs validation pass. Chromium and WebKit pass
relocated, offline nested-file acceptance with actual SVG/text bounds, no node
overlap, escaping, semantic outlines, theme/search/navigation, mobile overflow,
keyboard controls and JavaScript-disabled rendering. Both pass HTTP/watch
addition, invalid-diagram preservation and recovery plus real Ctrl-C. Four of
the five new native feature tests failed on the previous implementation.
Review also reproduced mobile page overflow when expanding a 32-level outline
with a 256-character label. The follow-up contains outlines in their own focused
scroll region, covered in both browsers with JavaScript enabled and disabled.
Consult the PR containing `a140800` for hosted six-target packaging, three-OS
browser acceptance and merge status. No native version is published here.

Prior milestone completion: [PR #23](https://github.com/mylife-inc/Zellij/pull/23)
merged head `18233d5` as `9fa1866`. PR/main CI `39093178918` / `39093718016` each
passed all 13 jobs, including Rust 1.96, all six package/install targets and
three-OS portable acceptance. Showcase passed 317 tests with 124 conditional
skips and no retry markers on both revisions. Image run `39093718039` passed
all 13 jobs, publishing four variants for Linux amd64/arm64 and passing the
published-image build/serve smoke test. Repository facts were refreshed:
public/default-main; native release remains `zellij-v0.1.3`.

### Portable Mermaid diagrams — 2026-10-17

Source `a9767fa` implements `markdown: {mermaid: true}` for portable Markdown.
The default remains literal code. Rust emits escaped source disclosures and
local assets, enforcing nonempty source, 64 KiB per fence and 32 diagrams per
page. Mermaid grammar/layout errors are browser errors, including after native
`check --strict`; the source remains visible and other graphs continue drawing.
Native input-limit failures remain source-file/ordinal `rendering` diagnostics
before publication. The browser enforces 500 edges and strict security settings
that author directives cannot weaken.

A single committed classic IIFE bundles Mermaid 11.17.2, KaTeX and diagram chunks
for moved `file://` exports. No Node, network, HTTP server or build-time browser
is needed by `zel`. Drawing requires browser JavaScript; disabled/unavailable
JavaScript retains readable source. Pages without diagrams do not load the
approximately 3.5 MB runtime; sites without diagrams omit it. MathML labels use
the shared local font. Remote author resources remain remote, and images inside
diagram source are not discovered/copied by the Markdown asset resolver.

`mermaid-client.js` serializes rendering and discards stale revisions across
site theme/mode and OS color-scheme changes. Duplicate IDs emitted by upstream
mindmaps are cleaned while preserving the first reference target. Native
mindmap CSS is scoped to its SVG. Bounded, focusable source disclosures avoid
very tall error pages. Source is indexed for offline search.

`scripts/prepare-portable-mermaid.mjs` generates/verifies the IIFE, notices and
provenance manifest; CI runs its byte comparison. The generator replaces the
hash-verified upstream embedded js-yaml 4.3.0 module with patched 4.3.2, checks
embedded source-map dependencies, and retains their exact-version notices.
The embedded Lodash 4.17.23 subset excludes the functions affected by its known
advisories; the generator asserts this. Ordinary npm audit misses such embedded
code. See `cli/crates/zellij-docs/assets/mermaid/README.md` for review evidence and
upgrade policy. This does not claim a zero-advisory dependency tree or exhaustive
security review, and the YAML shim changes only the portable bundle.

Local evidence: **186 Rust tests**, formatting, warnings-denied Clippy, bundle
reproduction, 47-route/23-example deep-strict documentation validation and the
production documentation build pass. Chromium/WebKit pass offline moved/nested
file navigation, flowchart/sequence/state/pie/mindmap/math labels, invalid syntax
and edge-limit isolation, protected directives, unique IDs, theme/system-mode
changes, search, mobile, disabled-JavaScript and missing-runtime fallbacks.
HTTP/watch acceptance passes opt-in changes, edits, invalid-source fallback,
native failure preservation and recovery in both browsers. Consult the PR
containing `a9767fa` for hosted six-target/three-OS acceptance and merge status.
No new native version is published here.

Follow-up source `ea39bfb` fixes a reproduced theme/font race: changing to Slate
while another render pass was active drew labels while its IBM Plex Sans request
was still pending. Initial font readiness did not cover later render revisions.
Each revision now explicitly loads its body font, awaits font readiness and
rechecks whether the theme became stale before measuring labels. Failed optional
font loads use the normal browser fallback instead of aborting all diagrams.
HTTP/watch regression tests change themes during drawing, delay real font
responses and assert all eight diagrams render after font readiness in both
Chromium and WebKit. Focused offline acceptance also passes in both browsers.
The original reproduction reported one pending font request and `loading`;
the fixed runtime reports zero pending requests and `loaded`.

[PR #25](https://github.com/mylife-inc/Zellij/pull/25) merged `bd791d1` as
`113283a` after all 13 PR jobs passed (`39097514862`). Its image run
`39098016565` passed all 13 jobs. Consult that PR for final main run
`39098016595` evidence, and the PR containing `ea39bfb` for final acceptance
of the font follow-up. No native release was published; `zellij-v0.1.3` was
reconfirmed as latest on 2026-10-17.

Prior milestone completion: [PR #24](https://github.com/mylife-inc/Zellij/pull/24)
merged head `46a25a8` as `1bddf89`. PR/main CI `39095187808` / `39095732531` each
passed all 13 jobs, including Rust 1.96, six package/install targets and three-OS
portable acceptance. Showcase passed 317 tests with 124 conditional skips and
no retry markers on both revisions. Image run `39095732463` passed all 13 jobs,
publishing four variants for Linux amd64/arm64 and passing published-image
build/serve smoke acceptance. Repository facts were refreshed: public/default-main;
latest native release remains `zellij-v0.1.3`.

## 1. Product and user's intentions

Zellij turns an authored folder of YAML, Markdown, MDX, and assets into a
product website with marketing pages and documentation. A shared Next.js engine
owns rendering, themes, components, navigation, and validation. A product's
content should not contain a fork of the engine or hand-maintained React for
ordinary content. The Rust binary is **`zel`**; the engine's Node helper is
**`zellij`**. Neither is the unrelated Zellij terminal multiplexer.

The user's outstanding intentions, stated on 2026-10-16:

- Give the sticky top bar more menu space: company logo at the left, search and
  theme controls at the right. The logo should link to the owner's website.
- Finish `zel`, including the desired operating-system coverage, explicitly
  including Linux, macOS, and Windows.
- Develop using a public `mylife-inc/Zellij` repository to avoid the current
  private-repository Actions billing constraint. Do not use `Zellij-Sealed`:
  its code is old and encrypted, and development should avoid encryption steps.
- Eventually use CodeSeal for long-term encrypted GitHub storage after the
  product is finished. The final distribution/secrecy model needs a separate
  decision: current public engine images intentionally carry source.
- The current request was research and brainstorming, not implementation.
  This restriction describes that request; future explicit implementation
  requests can authorize work.

### Additional CLI requirements — 2026-10-17

The user requires MkDocs-equivalent documentation workflows, with command syntax
free to differ: build a plain Markdown folder into a site that also works by
double-clicking `index.html`, serve an existing static site over HTTP, and a
`view` operation combining build, serve and browser launch. This is new required
product scope. Native build, serve and view are implemented above; portable
`view --watch` adds portable rebuild/reload, and `zel.yaml` configures identity,
branding and navigation. Richer rendering and additional configuration remain planned. The [portable documentation contract](specs/zel-portable-docs.md)
records plain-folder discovery, direct-file navigation/search/assets, offline
acceptance and cross-platform serve/view requirements. Keep a core MkDocs
capability comparison when implementing the Rust CLI. Today's Next static export
assumes HTTP and must not be advertised as satisfying direct-file viewing.

### CLI simplicity and delivery requirements — 2026-10-17

The user has paused implementation to discuss the command design. They want
short, memorable aliases for important long commands and conventions that let
users omit repeated source/output paths and options. MkDocs's `docs`/`site`
layout was an example, not a selected Zellij convention.

Requested workflows include:

- `zel ci <docs-folder>` as a short entry point for generating a GitHub pipeline.
  Depending on configuration, the pipeline should build the website, package an
  HTTP-serving image, push to a selected registry, and optionally deploy to a
  cloud provider. It should support multiple ways to trigger execution.
- A Markdown folder to a Docker image containing static HTML and an HTTP server.
- A Markdown folder to a Docker image containing the wrapper Next.js app.
- A Markdown folder to a generated static website deployed to Netlify.
- Documented canonical syntax, aliases, defaults, and examples for these flows.

These are requested capabilities, not claims of implementation. Existing image
building uses the hosted Next.js wrapper; direct plain-Markdown image workflows
and a separate static-server image path still need work. GitHub workflow
generation exists, but its default is validation only; `zel deploy` is a stub.
Specific image alias spellings remain proposals. The configured-site layout and
repository-root default are now agreed below. Preserve the distinction between portable HTML
(direct-file viewing) and the existing hosted static export (HTTP required).

The user's follow-up refines these requirements:

- Prefer the `image` command family over `docker`. Static-server and Next.js
  wrapper images should have distinct, memorable short commands that identify
  their contents without requiring a type-selection option.
- "Project" means the software product's GitHub repository. Its `docs/` can
  include specifications and other material alongside a dedicated Zellij site.
  Do not equate the repository with the documentation site or assume all of
  `docs/` is website input.
- Avoid a required nested `docs/zellij/docs/` layout. The subsequent decision
  below replaces both the earlier flat-source proposal and the optional
  `content`/`dist` layout.

Agreed configured-site convention (design decision, not yet implemented):

- The site root is the folder containing `zel.yaml`; its name is unrestricted.
- Markdown and its local assets belong under `<site-root>/content/`.
- Generated HTML belongs under `<site-root>/site/`, alongside `content/` and
  `zel.yaml`. It is outside the content source tree, although inside the site root.
- From a software repository root, `zel build` defaults to `docs/zellij/`,
  reading `docs/zellij/zel.yaml` and building `content/` into `site/` there.
- A custom location is supplied as `zel build <site-root>`: the argument names
  the parent of `zel.yaml`, not its `content/` child. A folder named `zellij`
  is only the conventional default location; the config identifies the site.
- Do not recursively guess a differently named site under the repository's
  broader documentation tree. The user explicitly supplies its location.

See the [configured-site contract](specs/zel-portable-docs.md#agreed-configured-site-layout--pending-implementation)
for examples and the distinction from existing command behavior. This discussion
has updated requirements only; command implementation remains unchanged.

### Full CLI scope and published reference comparison — 2026-10-17

The user emphasizes that build, preview, images, CI, and deployment are only
part of the intended CLI. They requested comparison with
[the deployed CLI reference](https://zellij.shebka.net/cli). The
[full command inventory and documentation audit](specs/zel-cli-inventory-2026-10-17.md)
records 19 registered top-level families / 32 leaf forms, five of which remain
stubs, plus seven additional command names in older design documentation.
Keep creation, schema inspection, conversion, validation/formatting, workflows,
and maintenance in scope when refining syntax and aliases.

The deployed and local references are not synchronized: all 20 published CLI
reference pages were retrieved, while `/cli/view` returned 404. The deployed
site predates native portable builds/previews and corrected JSON output. Local
docs are newer but still contain unsupported CI flags, an obsolete CI example
calling stubbed `deploy`, missing `ci publish` documentation, and incorrect
maintenance-command and image-tag descriptions. The inventory identifies
specific evidence and preserves current-source versus planned distinctions.
No CLI code or documentation deployment changed during this comparison.

### Full CLI contract and reference corrections — 2026-10-18

The user authorized continuing the full command-and-alias design. The
[contract draft](specs/zel-cli-contract-2026-10-18.md) accounts for all 39
inventoried forms, adds the requested static/Next image paths, and covers
aliases, argument roles, configured-site resolution, renderer selection,
shared settings, import compatibility, and publication/CI composition.
The site layout/default location is agreed; the draft's exact aliases, config
keys, workflow refinements, and implementation order remain proposals.

The draft distinguishes explicit-provider `deploy`, saved-settings `publish`,
and import-first `ship`, with a shared plan that builds once. It keeps portable
Markdown and hosted YAML/MDX as separate rendering paths under the common site
layout. It preserves legacy explicit content/export/tag arguments rather than
interpreting every path as a new configured site root.

Local reference corrections: `ci.mdx` now lists real generation options,
validation-only/default behavior, Netlify provisioning, actual triggers and
upload implementation, and `ci publish` (including the no-gh print-only case).
`doctor.mdx` distinguishes unregistered maintenance commands from stubs;
`image.mdx` correctly describes derived default tags instead of last-build memory.
The earlier audit remains a dated snapshot. At the end of the design discussion
these edits were local and uncommitted; the implementation milestone below
includes them. They have not been deployed or released.

Verification: generated validation-only/image/Netlify workflow fixtures with
the current CLI; Netlify used `--no-provision`, with no remote actions. Deep and
strict documentation validation passed (47 routes, 23 embedded examples).
All inventory forms appear in the draft; its 29 shortcut entries have distinct
spellings. These checks do not establish runtime support for proposed commands.

### Configured-site layout implementation — 2026-10-18

The user authorized continuing the CLI implementation. Branch
`codex/cli-site-layout`, source commit `efe8946`, is based on main `1aa8376`,
the merge of PR #27.
Repository inspection reconfirmed public visibility and main as default.

- `zellij-core::site` resolves an explicit site, cwd's `zel.yaml`, or
  `docs/zellij/zel.yaml`, in that order. No parent/recursive search. Bare site
  commands now fail with guidance when no configuration is present; existing
  unconfigured folders remain available explicitly, including `.`.
- `version: 1` requires `renderer: portable|hosted` and a real `content/`
  directory. Output defaults to sibling `site/`; optional `build.out` is relative
  to the config, and `preview.port` defaults to 3000. CLI overrides are cwd-relative.
  Config/content symlinks and Windows reparse points are rejected.
- Portable presentation retains strict validation and content-relative paths.
  Hosted settings stay in `content/site.yaml`; common hosted configs reject
  presentation fields instead of silently merging competing values. A common
  config no longer forces hosted sites into the native renderer.
- Build/check/serve/view, hosted dev, image build/dockerfile, and page/guide
  creation use site resolution. View remains portable-only; dev hosted-only.
  Explicit generated output serving, flat portable configs and raw content
  remain supported. Legacy explicit `.` retains hosted discovery.
- New portable/docs/project starters create `zel.yaml` with nested `content/`;
  default destination is `docs/zellij`. Generated project CI still uses raw
  `<site>/content`, compatible with released binaries. Imports remain unchanged.
- Watch pins its layout, port and output; it hashes content and sibling config,
  ignoring generated output and unrelated root files. Config removal, malformed
  YAML or renderer/layout changes retain the last good site and display errors;
  restoring configuration recovers. The compiler enforces the pinned layout too.
- Source/output disjointness uses the actual content root. Existing ownership,
  staging and failed-build preservation still apply. Configured default image
  tags use the site folder name, avoiding a shared `content:latest` default;
  explicit tags and legacy-folder tags retain their meanings.

Local validation: 209 Rust tests; formatting and warnings-denied Clippy;
Chromium/WebKit direct-file, offline, starter, HTTP preview and watch acceptance;
40 Rust/10 Node starters plus 10 guide-title checks; strict/deep docs validation
(47 routes, 23 examples). Browser tests include config/content edits, deletion
and recovery, ignored output changes, and configured serving without rebuilding.
The PR CI gates all supported native packages and the three OS acceptance suites;
consult its current status before treating those hosted checks as passed.

Previous main CI run `39109606782` failed container preparation because automatic
runtime discovery selected Podman after Docker did not pass its probe. Podman
could not see the locally built Docker image and tried registry pulls. The
container acceptance job now explicitly selects the Docker context that built
its image. Base-image publication run `39109606795` passed at main `1aa8376`.
The earlier PR #27 CI passed; that did not establish the later main run passed.

[PR #28](https://github.com/mylife-inc/Zellij/pull/28) carries the configured-site
work. Its first CI run (`39111452316`) caught the native installer acceptance
script still looking for the old `<folder>-site` output. The harness now checks
the configured `site/` output and runs bare check/build from the installed site's
root. Local x86_64 macOS release-archive installation, checksum/corruption tests,
installed native commands, relocated output and HTTP/Ctrl-C checks pass. The
installer's help text now calls the starter a documentation site.

Next: command aliases/help catalog, separate static/Next image commands, then
shared deployment/CI configuration. Global site/config/cwd selectors, portable
dev, hosted view, import-layout changes and advanced command stubs are pending.
The local reference and contract mark implemented settings separately from
proposals. No new CLI release or public documentation deployment in this increment.

### Container prerequisite implementation — 2026-10-18

The user authorized implementation after local reference reconciliation and
asked for deliberate handling of commands requiring Docker when it is absent.
The first increment is source commit `cd70f26` in
[PR #27](https://github.com/mylife-inc/Zellij/pull/27), on
`codex/cli-runtime-preflight`, based on `e8ec681` (PR #26).
Repository inspection reconfirmed `mylife-inc/Zellij` is public with
default branch `main`; no other open PR was present at implementation start.

- Runtime probes now distinguish missing clients, execution errors, failed
  `info` and timeouts. The OS resolves executables, fixing the old Windows
  `.exe` detection bug and Unix non-executable-file classification.
- Docker/Podman/nerdctl discovery is bounded: three seconds per info probe in
  a twelve-second overall budget, plus two seconds for version metadata.
  Image presence/identity inspection also has a two-second limit. Timed-out
  probe processes are killed and reaped; metadata timeout is not reported as
  an absent image. Build/server lifetimes are not capped by these probe limits.
- Nonempty endpoint environment variables select the associated runtime;
  conflicting runtime selections fail. Explicit endpoints are never replaced
  by a different runtime or local socket. Diagnostics print variable names,
  not their potentially credential-bearing values or raw probe errors.
- Automatic socket discovery uses Docker only: the previous Podman probes
  incorrectly used `DOCKER_HOST`, which does not select a Podman connection.
  Windows relies on configured client connections, without Unix socket probes.
- Missing/unavailable runtimes return exit 4 with platform-specific guidance.
  No automatic installation, service start, renderer switch or remote fallback.
  Portable commands remain independent; `image dockerfile` now works without
  a runtime and still respects `ZELLIJ_IMAGE`.
- Doctor retains its container-preflight exit contract: exit 4 on discovery
  failure, exit 0 when a runtime responds (even with unavailable metadata).
  It is not a prerequisite for portable builds or full site validation.

The local doctor/image references describe this policy and correct misleading
image-dependency claims. The original public-doc comparison and full proposed
command/alias contract are included with these changes. The published website
and downloadable release have not been updated by this milestone.

Regression coverage exercises missing-runtime command dispatch, preserved
source/output, native operations with broken endpoint settings, Dockerfile
generation, fallback, explicit-endpoint isolation, secret-free errors, bounded
metadata probes, and actual executable resolution. CI now runs the engine and
runtime-requirement tests on Linux, macOS and Windows. **201 local Rust tests**,
warnings-denied Clippy, formatting, diff checks and strict/deep documentation
validation (47 routes, 23 embedded examples) pass. The first sandboxed suite
could not bind loopback ports; the rerun with loopback access passed. Hosted CI
is a merge gate; consult PR #27 for its current checks and merge status.

The first hosted run of this increment (`39107995790`) exposed an older engine
test that assumed absolute paths begin with `/`. The Windows runtime probe tests
passed; the mount assertion now checks the native path API and canonical source
instead. A local doctor run also exposed duplicated diagnostics; detailed errors
now print once on stderr. The local environment currently explicitly selects a
Docker endpoint whose `info` call fails; no endpoint values were logged and no
environment/service changes were made. Real-container acceptance runs in CI.
The next Windows run (`39108320579`) passed the engine tests and exposed a test
isolation assumption: Windows can resolve a system Docker executable despite
an empty PATH. Its real info call timed out correctly. Windows CLI fixtures now
run a private copied binary beside invalid runtime placeholders, testing the
unexecutable-client path without contacting the runner's service; missing-client
classification remains covered by engine tests and Unix CLI fixtures.

Final reference reconciliation also replaces the obsolete TOML/upward-discovery
proposal in the local config, overview and delivery pages with the agreed
versioned YAML/site layout (still labeled unimplemented), documents build-once
delivery composition, and removes a stale claim that portable scaffolding was
unimplemented. Historical specifications remain preserved. The public site is
still older; no documentation deployment was triggered.

**Next implementation:** versioned `zel.yaml` and the common site resolver for
`docs/zellij/{zel.yaml,content/,site/}`, custom named roots and legacy explicit
paths, including watch/config changes. Alias registration, separate static/Next
image packaging and delivery/CI composition follow. None of that new syntax is
implemented by the runtime prerequisite change.

## 2. Read these sources in this order

1. This file and the [dated audit](specs/zellij-audit-2026-10-16.md).
2. [Content-model design](specs/zellij-content-model-design.md): current folder
   model, names, routing, ownership of navigation, and migration decisions.
3. [Engine specification](specs/zellij-engine-spec.md): architecture and quality
   bar. Some inventory counts and milestone text are historical.
4. [CLI specification](specs/zellij-cli-spec.md): desired surface, not a reliable
   statement that each command exists. Consult the later
   [full CLI inventory](specs/zel-cli-inventory-2026-10-17.md) for the
   published/local/source comparison and remaining command scope, and the
   [full contract draft](specs/zel-cli-contract-2026-10-18.md) for the latest
   command-and-alias design following the user's agreed folder layout.
5. [Product documentation](examples/zellij-docs/content): authoring guide,
   quickstart labs, CLI reference, and section examples. Read status callouts,
   but check implementation: some pages contradict the current code.
6. Source paths below. Schemas and command dispatch are stronger evidence of
   current behavior than prose claiming a feature is available.

Root [README](README.md), [engine README](engine/packages/zellij/README.md), and
the generated [CLI content reference](cli/crates/zellij-core/generated/context.md)
are useful supporting material. `zel context` describes how to author content;
it does not replace this architectural and development-status context.

## 3. Repository map

| Location | Responsibility |
| --- | --- |
| `engine/packages/zellij` | `@shebka/zellij`: Next/React rendering engine, schemas, build-time tools |
| `cli/crates/zellij-cli` | Clap command surface and orchestration; binary `zel` |
| `cli/crates/zellij-core` | Errors, project discovery, embedded schemas and reference |
| `cli/crates/zellij-scaffold` | Content/section/page/guide/project and GitHub workflow generators |
| `cli/crates/zellij-convert` | Converter registry, MkDocs and Docusaurus readers, MDX/Material normalization |
| `cli/crates/zellij-docs` | Native portable Markdown compilation/checks, structured diagnostics, offline assets/search and loopback HTTP preview |
| `cli/crates/zellij-engine` | Container runtime detection, check/dev/build, Dockerfile and image operations |
| `docker/base.Dockerfile` | Distributable wrapper/engine builder image |
| `scripts/install.sh`, `scripts/publish.sh` | POSIX binary installation and release/image publishing |
| `.github/workflows` | CI, base-image publishing, tagged releases |
| `examples/showcase` | Visual demonstration and Playwright browser suite |
| `examples/zellij-docs` | Zellij's own documentation website |
| `examples/payos`, `examples/codeseal-docs` | Real product integration examples |
| `specs` | Product contracts, design history, and audit |

There are **six Rust crates, four example apps, 23 section types, 14 themes,
and four page layouts**. Older references to 12/18/19 sections, 10 themes, or
three apps should not guide new work.

## 4. Architecture and execution

```mermaid
flowchart LR
  Content[Content folder] --> Source[FolderContentSource]
  Source --> Site[Validated Site model]
  Site --> Render[Next.js server rendering]
  Render --> Static[Static export]
  Render --> Server[Standalone site image]
  Content --> Prepare[Asset sync and search index]
  Prepare --> Render
  Schemas[Engine schemas] --> Embedded[Generated Rust reference]
  Embedded --> Zel[zel authoring commands]
  Zel --> Content
  Zel --> Container[Container engine adapter]
  Container --> Render
  Markdown[Plain Markdown] --> Portable[Native portable renderer]
  Zel --> Portable
  Portable --> Files[Movable HTML and local assets]
  Files --> Preview[Native loopback preview]
  Zel --> Preview
```

Rendering and preparation share content-source resolution. Search reads the
canonical Site model and uses parsed heading IDs shared with the TOC/rendering
pipeline. Preparation reads fresh data; rendering caches separately by content
and deployment configuration. The implemented development watcher refreshes
derived assets/search after content edits; automatic browser reload for every
prepared-file change still requires further acceptance.

- npm workspaces: `engine/packages/*` and `examples/*`; committed npm lockfile.
- Engine package version `0.1.0`; Rust workspace version `0.2.0-rc.6`. Post-rc.6
  source increments are identified at the top of this context and await the next candidate.
- Examples now pin Next `16.4.2`, React/React DOM `19.2.8`; TypeScript
  `5.9.3` range. Engine peer ranges are Next `~16.4.2`, React `19.2.x`.
- Manifests declare Node `^20.19.0 || >=22.12.0`; `.nvmrc`, CI and Docker use Node 24. Audit used Node
  `24.19.0`. Rust uses edition 2024; CLI/docs require 1.96 and helper crates
  declare 1.85. The math milestone adds a whole-workspace 1.96 CI check; consult
  its final results. Local validation uses Rust 1.99.0.
- Local package exports point at source, consumed through Next's
  `transpilePackages`. tsup also emits ESM/CJS/types and bundles the Node CLI.
  Build verification checks that client directives survive packaging.
- React Server Components are the default. Interactive leaves include the nav,
  search, theme controls, tabs/accordions, carousels, and motion.
- Static export and standalone server/image are the principal outputs.
  `withZellij()` configures Next; the wrapper delegates to the engine factories.

### Key source entry points

| Concern | Start here |
| --- | --- |
| Public API | `engine/packages/zellij/src/index.ts` |
| Next configuration | `engine/packages/zellij/next-config.mjs` |
| Shared content/prefix bootstrap | `engine/packages/zellij/deployment-config.cjs` |
| Runtime loading/cache | `src/config/runtime.ts` |
| Config and content schemas | `src/config/schema.ts`, `src/content/model-schema.ts` |
| Folder/legacy loaders | `src/content/folder-source.ts`, `src/content/legacy-source.ts` |
| Site assembly and validation | `src/content/site.ts` |
| Names/routes/assets | `src/content/naming.ts`, `paths.ts`, `asset-href.ts` |
| Route factories/metadata | `src/render/page.tsx`, `root-layout.tsx`, `seo.ts` |
| Section registry/schemas | `src/sections/registry.ts`, `schemas.ts` |
| Section rendering/layout | `src/render/section-renderer.tsx`, `sections-page.tsx` |
| Guide rendering | `src/render/guide-page.tsx`, `src/guide`, `src/mdx` |
| Header and branding | `src/chrome/nav-bar.tsx`, `nav-bar-client.tsx`, `nav-overflow.ts`, `use-nav-overflow.ts`, `src/styles/chrome.css` |
| Themes/motion | `src/themes`, `src/motion`, `src/styles/sections.css` |
| Search and heading IDs | `src/search`, `src/cli/write-search-index.ts`, `src/mdx/headings.ts` |
| Build-time operations | `src/cli/check.ts`, `sync-assets.ts`, `context.ts`, `init.ts` |
| Development preparation | `src/cli/dev.ts`, `watch-preparation.ts`, `write-prepared-file.ts`; packaged/browser/container fixtures in `scripts/check-dev-*.mjs` |
| Static reference validation | `src/cli/deep-check.ts`, `src/content/reference-schema.ts`, `src/content/inline-markdown.ts` |
| Book manifest and reading order | `cli/crates/zellij-docs/src/book.rs` |
| Shared equation numbers/references | `cli/crates/zellij-docs/src/equations.rs`; regression manuscript in `scripts/fixtures/equations` |
| Native/Next book adapters | `cli/crates/zellij-docs/src/markdown.rs`, `cli/crates/zellij-docs/src/hosted.rs`; `src/content/portable-document.ts` |
| Print model and execution | `cli/crates/zellij-docs/src/pdf.rs`, `cli/crates/zellij-docs/assets/book.tex`; `cli/crates/zellij-cli/src/pdf.rs` |
| Rust command truth | `cli/crates/zellij-cli/src/main.rs` — enum AND dispatch |
| Release truth | `.github/workflows/release.yml`, `scripts/install.sh` |

Paths beginning `src/` above are relative to `engine/packages/zellij`.

## 5. Content model and invariants

```text
content/
  site.yaml              identity, brand, theme, behavior, footer
  menu.yaml              optional explicit top navigation
  pages.yaml             section-composed pages
  sections/              reusable YAML or Markdown/MDX sections
  assets/                authored media
  01-guide/
    _sidebar.yaml        ordering, title, optional route override
    index.mdx
    installation.mdx
  02-reference/           another independent guide tree
    _sidebar.yaml
  examples/              optional YAML for SectionPreview
```

- These top-level files are optional; a guide-only site is valid to the engine.
- Numeric prefixes establish order and are stripped from public routes.
  `_sidebar.yaml` and page frontmatter can customize routes. A guide without
  an index redirects its root to its first listed page.
- `pages.yaml` references section files. Presentation overrides are limited to
  `background`, `id`, `motion`, `padding`, and `titleGradient`.
- `home` selects the home destination. Menus derive from content when absent;
  explicit menus can contain ordinary links, dropdowns, or mega-menu groups.
- `menuOverflowAfter` sets the count limit; the desktop client additionally
  measures available width and moves trailing entries into More as needed.
- Menu `page:` and footer references are resolved/validated. Literal `url:`
  entries deliberately bypass content lookup. Do not accidentally remove that
  deliberate escape hatch while expanding link validation.
- Unlisted guide pages still generate routes, with warnings; do not assume
  removing a sidebar entry hides or unpublishes a document.
- A `marketing/` directory selects the legacy loader. Compatibility remains,
  despite the design's original one-release migration window.
- Runtime memoization lasts for the process in production, 250 ms in dev.
  Entries are keyed by content/configuration paths and deployment prefix;
  failed loads are evicted. Content changes still require a production rebuild.

### Rendering inventory

Sections: `bento-grid`, `card-carousel`, `code-cards`, `cta-banner`, `faq`,
`feature-grid`, `gallery`, `hero`, `hero-split`, `logo-strip`, `markdown-prose`,
`matrix`, `media-panel`, `pricing-table`, `promo-grid`, `showcase`, `stats-band`,
`steps-panel`, `team-grid`, `testimonial`, `timeline`, `video-embed`, `zigzag`.

Themes: `midnight`, `paper`, `terracotta`, `mono`, `aurora`, `slate`, `forest`,
`sunburst`, `carbon`, `porcelain`, `cupertino`, `studio`, `iris`, `meridian`.
Layouts: `stack`, `stack-narrow`, `stack-sidebar`, `landing`.
Motion: `none`, `subtle`, `expressive`, with reduced-motion overrides.

MDX uses next-mdx-remote/RSC, GFM, Shiki, and heading slugs. Built-in components
include Callout, Steps, Tabs, CodeGroup, Card/CardGrid, Screenshot,
SectionPreview, Theorem, and Proof. Optional math renders MathJax SVG at build
time; Mermaid loads in the browser; YAML mindmaps render through D3 at build
time. Do not conflate installed optional packages with implemented Rust feature
selection: the latter is incomplete.

Assets are copied to `public/_zellij/assets` with fingerprint query hashes.
Guide search is a static FlexSearch index fetched when search first opens;
marketing sections are intentionally not indexed.

## 6. What `zel` actually does

An implemented dispatch branch does not imply all flags or edge cases are
complete; see the audit before using writers on valuable content.

| Status | Commands |
| --- | --- |
| Implemented native operations | `new docs`, `new section`, `new page`, `new guide`, `new project`, `explain`, `list sections/themes/formats/targets`, `context` (Markdown/JSON), `detect`, `convert`, `doctor`, portable `check` (`--strict`/`--json`), `build`, `serve`, `view` (optional `--watch`) |
| Implemented container orchestration | Hosted `check` (optional `--deep`/`--strict`), `dev`, `build` static/image; `image build/run/push/dockerfile` |
| Implemented workflows | `migrate` = detect/convert/check; `ci init github`; `ci publish` dispatches an existing workflow |
| Declared but returns “not implemented yet” | `new app`, `deploy`, `fmt`, `publish`, `ship` |
| Specified but not registered | `watch`, `eject`, `pull`, `upgrade`, `completions`, `version` subcommand (`--version` works) |
| Output options | `--json` works for `context`, every `list`, `explain`, `detect`, portable `check`; other commands (including hosted check) reject it before execution. `--verbose` / `-v` is rejected, not a successful no-op. |

Converters: MkDocs and a limited Docusaurus implementation. Computed JS/TS
sidebar configurations are not executed. Other formats in the spec are plans.
`new page` prints the `pages.yaml` entry for a person to paste; preserving
comments by avoiding a rewrite is deliberate.

Portable builds load `zel.yaml`; general command defaults in TOML are still
planned. Native portable diagnostics are structured; hosted JSON diagnostics,
a deployment-provider crate and broad feature/build-option plumbing remain planned. Historical docs differ on general
`zel.toml` versus `zellij.toml`; that future filename still needs reconciliation.
Generated GitHub workflows support validation, GHCR images, and Netlify through
its own CLI, despite the separate `zel deploy` command being a stub.

Docker, Podman, and nerdctl are probed; Colima socket handling exists.
`ZELLIJ_IMAGE` overrides the default `ghcr.io/mylife-inc/zellij-base:latest`.
Build/dev install app dependencies inside the container. A pulled base image
alone is therefore not a guarantee of offline builds. Dev watches and prepares
assets/search after content edits; the browser still needs a manual reload.

## 7. Where development stopped at the initial audit

The following is the historical baseline, superseded where indicated by the
implementation update above.

Local `main` is three commits ahead of both the tracking ref and the live remote
`main` observed on 2026-10-16:

| Commit | Work |
| --- | --- |
| `e97099f` | Reject successful-looking static builds that wrote no output, including Colima mount failures |
| `5001c37` | More faithful MkDocs/Material conversion, MDX escaping, generated React version constraints, clearer build-failure classification |
| `6356b05` | Apply `menuOverflowAfter` to authored menus as well as derived menus |

The last commit, dated 2026-09-25, explicitly leaves width-aware menu overflow
unfinished. The local `content-model` branch is contained in `main`; its one
commit ahead of its own tracking branch is not additional unmerged work.

Live repository facts on the audit date:

- `origin`: `git@github-mylife-inc:mylife-inc/Zellij.git`.
- **`mylife-inc/Zellij` already exists and is private**; default branch is
  `content-model`. Remote main is `70fc7c3`; remote content-model is `b34a8c3`.
- Recent Actions jobs were blocked before execution by billing/spending limits.
  That is separate from the local Clippy failure.
- Public binary releases live in `mylife-inc/releases`; latest observed release
  is `zellij-v0.1.3`, with macOS and GNU/Linux, each x86_64 and arm64.
- Native Windows was deliberately removed in commit `6a44d37`; WSL is the
  current documented Windows route. User's desired native support reopens that
  decision. No native Windows installer/release target currently exists.
- The live **base image `latest` contains both linux/amd64 and linux/arm64**.
  Do not repeat the older publishing-script comment saying it is amd64-only.
  Feature-tag variants and the relationship to unpublished local changes were
  not independently verified.

In the follow-up discussion on 2026-10-16, the user accepted reusing the existing
`mylife-inc/Zellij` repository and changing it to public during development.
Creating a replacement repository is no longer the proposed path. The user
then authorized work to begin; the implementation update above records the
resulting repository changes. Standard public GitHub-hosted runners avoid
minute charges, but normal usage/concurrency limits still apply. Later
encryption cannot retract plaintext copies already obtained while public.

## 8. Verification baseline and next work

At the audited commit: 227 engine unit tests passed, 82 Rust unit tests passed,
workspace typechecking passed, engine package build passed, Rust formatting
passed, and generated schema/reference matched their committed versions.
All four local example content roots passed the engine check.

Clippy with `-D warnings` failed at `zellij-convert/src/material.rs:29`.
Showcase production build was blocked by the local missing
`@tailwindcss/postcss` dependency; browser tests, Lighthouse, and current
production visual acceptance were not completed. Passing content validation
does not prove MDX compiles or all links/images work.

Remaining order after portable Mermaid implementation, subject to the user's v1 scope decision:

1. Complete hosted acceptance of portable Mermaid, including minimum Rust and all
   native targets; consult the PR containing `ea39bfb` for the font follow-up. The original Mermaid feature is merged in PR #25.
2. Finish the agreed CLI subset and remaining portable compatibility work in
   [the portable contract](specs/zel-portable-docs.md); native scaffolding, math,
   YAML mindmaps and opt-in Mermaid are implemented.
3. Choose and publish a new native version through the tested release path,
   preserving the existing v0.1.3 assets. Confirm release destination/version
   compatibility and documentation deployment; broader OS/signing decisions remain.
   No new native CLI binary has yet been released for the completed source fixes.
4. Complete broader accessibility, SEO and performance acceptance. Investigate automatic
   browser reload, and the recorded browser flakes. R5 retains advanced migration
   limits; R13's static gate does not execute arbitrary MDX components. Define a
   finite release gate: no known blocking defects, rather than a promise that
   undiscovered bugs cannot exist.

Versioned docs, i18n, remote content, CMS, comments, and A/B testing are explicit
v1 non-goals in the engine spec; do not turn them into release blockers without
a new product decision.

### Useful checks for the next session

```sh
git status --short
git log -6 --oneline
npm run typecheck
npm test
npm run build -w @shebka/zellij
cargo fmt --manifest-path cli/Cargo.toml --all --check
cargo clippy --manifest-path cli/Cargo.toml --workspace --all-targets -- -D warnings
cargo test --manifest-path cli/Cargo.toml --workspace
```

Then perform clean dependency installation/build/browser/release checks in an
appropriate environment. Do not claim a green full suite based on unit tests.
On the audit machine, `/usr/local/bin/node` was broken by a missing dynamic
library. Verification used Codex's bundled runtime; discover its path afresh
with `load_workspace_dependencies` if necessary. Do not silently repair the
user's system Node as part of an unrelated project task.

When returning to this project, refresh the dated repository/release facts,
read the relevant audit finding, inspect its source, and then implement only
the work the user has authorized.

### CLI shortcuts and completions — 2026-10-18

Source `271493f46d3b97ceed12623524505374e3e12190` on
`codex/cli-aliases` follows PR #28 / main `c6e1c51`. Before this work,
main CI `39112482978` and base-image publication `39112482938` both passed.

- `cli/crates/zellij-cli/src/args.rs` owns the argument schema. Thirteen
  flattened shortcuts share Args structs with their descriptive commands;
  a small catalog generates their parser variants, canonical mapping and help
  links. Same-level shortcuts use Clap visible aliases. Normalization happens
  before the global JSON policy and existing handler dispatch. No argv text
  substitution or arbitrary prefix abbreviation is used.
- Twenty-two top-level shortcuts are available: `init`, `nd`, `ns`, `np`, `ng`,
  `npr`, `ex`, `sections`, `themes`, `formats`, `targets`, `ctx`, `cv`, `mig`,
  `c`, `b`, `s`, `v`, `ir`, `ip`, `idf`, `comp`. Each retains its full form's
  options, defaults, validation, runtime requirements, JSON policy and status.
- `ci run` is the descriptive workflow-dispatch name; `ci publish` remains an
  exact compatible spelling. No workflow is generated or dispatched merely
  by adding the spelling. Actual dispatch semantics remain unchanged, including
  the existing successful guidance-only response when `gh` is missing.
- `completions <shell>` / `comp <shell>` emits Bash, Zsh, Fish, PowerShell or
  Elvish scripts to stdout from the same command tree. No profile writes,
  runtime discovery or network calls. Generator output is buffered so stdout
  IO errors use the normal CLI error path. The new locked clap_complete
  dependency is 4.6.11; its Clap requirement advances Clap to 4.6.7.
- Parser tests cover every alias, defaults and explicit arguments, global
  flags on both sides, help, errors, collisions, `--`, and names used as data.
  Real-binary tests compare scaffold/output bytes, JSON, failure statuses and
  side-effect-free unsupported JSON. Fake-gh tests compare both dispatch
  spellings for success and failure without contacting GitHub.
- Linux/macOS exercise actual Bash completion; Windows CI exercises Windows
  PowerShell completion for canonical and short forms. The three-OS portable
  matrix runs the parser and alias integration tests. Browser configured-site
  watch/serve acceptance now uses `v` and `s`; other cases retain full forms.

Local checks passed: **222 Rust tests**, formatting, warnings-denied Clippy,
strict/deep documentation validation (**49 routes, 23 examples**), Zsh
completion syntax validation, and the complete Chromium/WebKit HTTP/watch
suite including configured `v`/`s` acceptance.
[PR #29](https://github.com/mylife-inc/Zellij/pull/29) tracks the hosted native
package and cross-platform gates; verify its checks and merge status before
relying on that evidence. No new native release or public documentation
deployment is implied.

The local CLI guide now has shortcut and completion pages, plus updated CI,
configuration and maintenance status. The contract distinguishes the implemented
names from future capabilities: conversion remains raw hosted content, `view`
is portable-only, and `idf` has only the Next recipe without `--kind`. Stub and
future shortcuts (`na`, `si`, `ni`, `pub`, `netlify`, bare `ci [site]`) remain
unregistered. Next: separate static/Next image commands, then shared delivery/CI
configuration and direct Netlify deployment; retain the broader backlog.

Minor existing diagnostic debt observed during test development: invalid
`new section`/`new page --sections` messages still say there are “nineteen”
section types, although `list sections` reports the compiled 23 types. This
milestone preserves their current errors; remove that stale count in a later
help/diagnostics cleanup. Missing explicit serve/view folders also retain their
current IO exit 1 rather than an argument exit 2.

PR #29's first full CI attempt (`39115475197`) caught a Windows completion
fixture omission: Cargo's built `zel.exe` was not discoverable on the child
PowerShell PATH. The fixture now prepends its exact binary directory and checks
`Get-Command` resolves that executable before exercising native tab completion.
The parser and other alias execution tests had passed on Windows; the fix does
not change production CLI behavior. Consult the PR's latest checks for the
corrected Windows result.

The subsequent diagnostic run (`39116110828`) isolated the remaining Windows
failure to PowerShell 5.1's upstream native-completion limitation: a bare `-`
or `--` does not invoke registered completers at all
([PowerShell #2912](https://github.com/PowerShell/PowerShell/issues/2912)).
The fixture now exercises empty words and partial long options (for example
`--o`) with the same canonical/alias assertions; PowerShell 6+ also checks bare
`--`. A checksum-verified temporary Microsoft PowerShell 7.6.7 runtime passes
the same harness locally. The CLI reference documents the 5.1 workaround.
No production generator modification was needed. Earlier path-fixture fixes
remain, but were not the root cause of the empty completion result.


## Static-site container images — 2026-10-18

Source `5ed8fb49126ce3919abedbe532f6ec7ba63506e4` follows merged PR #29,
main `707c183072667b89f1044a508f2651ac50a59a22`. That main commit's CI
`39117232383` and base-image publication `39117232331` both passed. PR #29's
final source CI `39116486868` also passed all 13 jobs, including PowerShell.

- `image static [site]` / `si [site]` accepts either renderer and shares the
  existing configured-site/default-path resolver. There are now 23 top-level
  shortcuts. `--tag` overrides a normalized `<site-name>-static:latest` default.
- Portable sites compile with `zellij-docs` into a private temporary export;
  only that output is the image context. The export is removed after success
  or failure. Existing `site/` and configured `build.out` are untouched.
- Hosted sites use a multistage recipe to validate and export inside the builder.
  The installed engine's deployment helper resolves basePath; output is copied
  under that prefix in the final HTTP root. There are no host bind mounts in
  either new packaging path. The final image contains the export, not source
  YAML/MDX or Next build dependencies.
- The final `nginxinc/nginx-unprivileged:stable-alpine` image runs as `nginx`
  on port 3000, with directory indexes, extensionless HTML, ordinary assets,
  missing-route 404s, output-marker denial and an index health check. Existing
  `image run` / `ir` works when given its tag explicitly. No implicit push.
- `image dockerfile` / `idf --kind static|next` prints without a runtime. For
  portable static recipes the context is an already built HTML folder; hosted
  static and Next recipes take hosted content. Omitted kind preserves Next.
- Legacy `image build` / `build --target image` keep the Next recipe and original
  tag default. `image run` without a tag retains its legacy hosted-site default;
  it does not infer the new static tag. Shared image-build failures now use
  `BuildFailed` (exit 3), avoiding a claim that every Docker failure is content.
- Runtime detection precedes temporary compilation. Missing/unavailable runtime
  behavior remains exit 4; there is no automatic installation/start/fallback.

Local verification: **226 Rust tests**, formatting, warnings-denied Clippy,
strict/deep documentation validation (**49 routes, 23 examples**), and syntax
validation of `scripts/check-static-images.mjs`. New fake-runtime tests cover
export-only contexts, source/output preservation, cleanup on failure, explicit
and default tags, shortcut parity, JSON rejection, and Next compatibility.
The local Docker daemon was stopped; real image validation is delegated to the
CI acceptance script, which builds a portable image plus root/prefixed hosted
images and checks their HTTP routes, assets, health checks, non-root user and
read-only runtime filesystem. [PR #30](https://github.com/mylife-inc/Zellij/pull/30) tracks the hosted results;
verify its latest checks before treating container acceptance as established.

Scope decision: complete static packaging before introducing the plain-Markdown
Next adapter. The existing conversion emitter intentionally drops unlisted
pages and reports external navigation links as unmapped; it cannot be reused
unchanged for a faithful `ni`. The adapter needs explicit navigation/asset/link,
branding/theme and Markdown-feature mapping before registering `image next` /
`ni`. Then continue shared image/delivery/CI configuration, Netlify deployment,
CI generation and the remaining inventory. This milestone does not complete the
CLI, create a binary release, or redeploy the public documentation.


PR #30's initial full container run (`39141095556`) built all three images;
portable and unprefixed hosted HTTP acceptance passed. Its prefixed case caught
an incorrect test assumption: Next's exported redirect digest holds `/manual`,
and the browser router adds basePath. A local isolated export confirmed this
behavior in Chromium. Container acceptance now tests actual home redirect,
client navigation and reload in Chromium, without patching the engine. Review
also narrowed the NGINX denial rule to `.zellij-output`; valid dot-prefixed URL
segments must remain routable. The prefixed fixture now uses `/products/.docs`.
Consult the PR's latest run for final evidence, not that superseded assertion.

The first browser-enhanced run (`39141869721`) then caught an invalid fixture
URL: hosted Markdown needs `/_zellij/assets/logo.svg`, whereas `assets/logo.svg`
is the YAML/Screenshot convention. The rendering code correctly preserved the
unsupported relative URL. The fixture now uses the public URL and runs the
engine's deep/strict validation before image builds; no engine change was needed.
The dot-prefix server correction is source `eea044623ee402c0dba36ab127ec008a6ba6760e`.

The final-head run (`39142340217`) first hit a transient Docker info timeout
before any image build. On retry it built all three images and passed portable
and root-hosted browser checks. The prefixed HTTP resource scan incorrectly
fetched React's `<link rel="preconnect" href="/">` hint as a document. Local
export inspection confirmed actual links/assets were correctly prefixed. The
scanner now inspects real resource/navigation tags and excludes preconnect/DNS
hints; browser navigation/error checks remain. Use the latest PR checks for
acceptance after this fixture correction.

Real NGINX acceptance **passed** in run `39143717743` at source
`29e569509a0c8c73a9a90fb6eebd98af32dd0093`: portable, hosted-root and
hosted-prefix images, health checks, readonly/non-root runtime, HTTP resources,
and Chromium redirects/navigation/reload. All six package/install jobs passed.
A separate unchanged Windows portable suite hit its first file:// navigation's
10-second timeout (it passed on the earlier heads). Windows navigation now has
30 seconds for cold runner/browser startup; interaction and assertion timeouts
and all offline/content checks are unchanged. The latest PR run must validate
that final test adjustment and the rest of the regressions before merging.


## Next.js images and native Markdown adapter — 2026-10-18

Baseline refreshed: PR #30 merged as `7dc79bab90dd6bb852f3f4eb9187840bd3225463`.
Main CI `39145286725` and base-image publication `39145286638` both passed.
The latest public native release remains `zellij-v0.1.3` (2026-08-13); no release
or documentation deployment is part of this increment.

Implementation source `54e24d09a9a25941bc1b3642e42c0c9aee08f819`, tracked in
[PR #31](https://github.com/mylife-inc/Zellij/pull/31). Use its latest checks for
final hosted acceptance:

- `image next [site]` / `ni [site]` uses the common site resolver and a separate
  normalized `<site-name>-next:latest` tag. There are now 24 top-level shortcuts.
  Hosted content uses the existing Next wrapper. Runtime preflight is first;
  missing/unusable runtime remains exit 4, without installation/start/fallback.
- Portable content uses `zellij_docs::adapt_hosted` into a private TempDir;
  success and failure remove it without replacing `site/` or modifying sources.
  The adapter reuses native discovery, config/navigation validation, Markdown
  parsing, heading IDs, reference resolution, asset containment, math/mindmap
  validation and output-collision checks. All discovered pages survive nav
  omissions. Only referenced assets and branding bytes enter the context.
- The bridge is internal `manual/pN.zeldoc.json` version 1 data. It contains a
  constrained HAST tree plus title, description, route, plain text and headings.
  The engine validates allowed nodes/properties/URLs before rendering; no MDX
  evaluation or raw HTML injection is used. Raw HTML/JS-looking prose stays text.
  Root index maps to `/`; other native output paths keep `.html` (including
  nested index) to avoid collapsing distinct `foo.md` and `foo/index.md` pages.
  Next's reserved `/_next` folder is rejected with an actionable error.
- Hosted rendering shares Shiki, Maths/Mermaid/mindmap transforms, link/image
  components, theme and guide layout. Unknown code languages stay plain text.
  Actual native maths/Mermaid usage selects the required builder features;
  missing optional support fails explicitly instead of silently dropping it.
  Generated HAST uses `hast-util-to-jsx-runtime` 2.3.6, already in the lockfile
  transitively and now a direct engine dependency.
- Hosted guide loading, TOC, search and deep reference checks understand the
  generated data. Original H1/explicit IDs remain; the guide does not insert a
  duplicate title. Search indexes every page and heading. Separate navigation
  labels and nested external HTTP(S) links are supported in `_sidebar.yaml`;
  external entries stay out of the previous/next sequence. Root-mounted ordinary
  guide pages now avoid accidentally generating `//page` URLs.
- Native branding is copied to a safe generated filename so `#` or `%` in its
  original filename cannot become URL syntax. Other assets retain their paths
  with encoded URLs. Heading and TOC links encode explicit IDs correctly.
- Next recipes explicitly select standalone output and carry a generated health
  path including the site's basePath. Legacy `image build` / `build --target image`
  retain their hosted-only behavior and unsuffixed default tags. `idf --kind next`
  still needs hosted content; its portable error directs users to `ni`.

Validation established so far: 340 engine tests, workspace typechecking, engine
packaging, strict/deep documentation checks (49 routes, 23 examples), adapter
and fake-runtime integration tests, and a production Next wrapper in Chromium
using the actual CLI's generated content. Browser checks cover titles, theme,
company link, nested/external nav, custom anchors, navigation/reload, unlisted
search entries, assets/downloads, tables/tasks, maths, mindmaps, Mermaid, and
inert HTML/JavaScript-looking prose. Final local validation passed **229 Rust tests**, formatting and warnings-denied
Clippy. All-platform and real-container CI acceptance remains pending; consult
PR #31's latest checks. A stopped local Docker daemon
means real image/server verification belongs to CI; local wrapper acceptance
must not be represented as actual Docker image acceptance.

Next work after this milestone: shared image/delivery/CI settings, Netlify
integration, expanded workflow generation, then the remaining CLI inventory.
The CLI is still not complete. No user action is currently needed.


PR #31 run `39147053587` built the portable Next image and passed valid-page
browser checks, then found an existing standalone packaging defect: unknown
URLs triggered runtime content loading, but `/app/content` was absent, producing
500 instead of 404. Next's automatic dependency trace does not include those
dynamic filesystem reads. The shared CLI recipe and `docker/site.Dockerfile`
now carry the selected content into the non-root runtime; portable images carry
only the generated adapter content, not original Markdown or unrelated files.
The checked-in Dockerfile also mirrors standalone output and prefix-aware health
checks. Acceptance explicitly checks unknown paths as well as source-file URLs.
Use the latest PR checks for the corrected actual-container result.


## Saved image publication — 2026-10-18

Baseline refreshed: PR #31 merged as `2f54c94d9256c5a5954587649a15d883aac3dce9`.
All 13 pre-merge CI jobs passed at `81f28661f12ff5d0245effceb939257a1e45cdac`
in run `39147793875`, including actual static and Next container acceptance.
Base-image publication `39148891016` and all 13 main CI jobs in
`39148891023` passed, so the updated portable adapter renderer is published. The latest public native binary remains `zellij-v0.1.3` (2026-08-13).

This increment implements registry-only delivery; Netlify remains the next
provider adapter. No native binary release or public docs deployment is included.

- `zellij-core::delivery` owns typed image/publication settings and pure
  `ImagePublication` intent resolution. Versioned configs accept `image.name`
  and `publish: {action: push, image: static|next}`. Image references are
  validated before runtime probes; unknown/conflicting fields and credential
  fields fail. Legacy flat configs must not silently acquire these fields.
- `si` / `ni` resolve tags as explicit `--tag`, saved `image.name`, then their
  separate generated defaults. A configured name is explicit and shared by
  both kinds; users wanting both choose separate tags. Legacy `image build`,
  `build --target image`, and omitted-tag `image run` retain their old defaults.
  `ZELLIJ_IMAGE` still configures the builder, not the published artifact.
- `publish` / `pub` follows standard cwd/custom-root discovery and requires
  saved intent plus `image.name` or `--tag`. It never uses an autogenerated
  local tag as a publication destination. There are 25 top-level shortcuts.
  `--dry-run` prints a plan without content validation, runtime/credential
  probes, writes or remote requests. `--json` and `--to` are rejected before work.
- Image commands and publication share one build operation, with one runtime
  preflight for publication. A completed build is pushed once with its exact
  reference. A failed build cannot push; a failed push reports the completed
  build and a push-only retry command. Registry access is established only by
  pushing, not by preflight. Credentials stay with the container runtime.
- Image retag/push failures now have operation-specific errors while retaining
  exit 3. Config errors use 2, unavailable runtime 4, process/IO errors 1.
  Source and existing output remain unchanged; temporary contexts are cleaned
  on success and failure. There is no implicit push during local image builds.
- Native integration coverage exercises both renderers/kinds, exact tags,
  precedence, no-tool dry runs, invalid/legacy settings, failure ordering and
  cleanup. The existing static and Next CI fixtures now push their portable
  image to a disposable loopback registry, remove the local image, pull it back,
  and run HTTP/browser checks against the retrieved artifact. They do not push
  to external accounts or create provider deployments.

Local verification passed: **237 Rust tests**, warnings-denied Clippy, formatting,
strict/deep documentation validation (49 routes, 23 examples), and JavaScript
syntax checks for acceptance fixtures. Actual registry/container and all-platform
CI remain pending; record fixing source and the PR below for final evidence. Next: direct
Netlify delivery, shared provider composition, expanded CI generation, then the
remaining CLI inventory. The CLI remains incomplete; no user action is needed.

Implementation source: [`5b8bf91d731af36321f725f4dc4590519e1ba480`](https://github.com/mylife-inc/Zellij/commit/5b8bf91d731af36321f725f4dc4590519e1ba480).
The final local delivery regression (including legacy tags with saved new
settings) and warnings-denied Clippy passed again after that test addition.
Consult the `codex/cli-delivery` PR's latest checks for actual registry
publication and all-platform acceptance; local fake-runtime checks do not
establish a successful registry push.

Tracked in [PR #32](https://github.com/mylife-inc/Zellij/pull/32). The first CI
run (`39150307894`, head `6a0aeabb419016f51072e1201218765eaad91a19`)
passed actual static publication/pull/HTTP acceptance and 11 jobs before the
final shell-guidance refinement. Printed run/push/retry commands now quote
references, so bracketed IPv6 registry names do not become shell glob patterns.
Focused image/delivery tests pass after that refinement. Consult the PR's
latest run for final acceptance; the earlier run is not final-head evidence.


## Native Netlify delivery — 2026-10-18

Baseline refreshed: PR #32 merged as `89c590debecf26b17bfa8175133db4c4a6df4a68`.
Its final PR run `39150951658` passed all 13 jobs at `66d557c`; main CI
`39152137856` also passed all 13 jobs, and base publication `39152137704`
succeeded. The latest native release remains `zellij-v0.1.3` (2026-08-13).

- `DeliveryPlan` in core resolves registry push or Netlify static deployment.
  `deploy netlify` / `netlify`, saved `publish` / `pub`, and bare `deploy` share
  argument types/operations and build once. There are 26 top-level shortcuts.
- Versioned `publish` accepts `action: deploy`, `provider: netlify`, optional
  string `site`. --site-id/--site > NETLIFY_SITE_ID > publish.site. Explicit
  deploy supplies intent; `pub --to netlify` cannot silently replace a saved
  image push. Image tags/kinds and unsupported providers are rejected for
  Netlify. Both renderers retain their existing build behavior.
- Native HTTPS digest upload uses reqwest with rustls and SHA1; no Netlify CLI
  or Node dependency. NETLIFY_AUTH_TOKEN is the only credential source. No
  stored-login discovery, remote build/functions, site creation or login flow.
  Preflight checks access before building. Dry-run remains resolution only.
- Draft previews are default; --prod explicitly requests production. Output
  names the ready deployment and its HTTPS URL with the hosted prefix. It does
  not claim a locked site has changed its current live deployment.
- Private temporary exports preserve original source and previous site/output.
  Hosted `build_for_deployment` reuses the existing export operation and the
  engine's deployment-config.cjs resolver to physically mount files at basePath;
  ordinary `build` exports retain their old layout. Metadata/ownership markers
  are excluded from uploads. Hidden prefix segments and Unicode/space paths are
  retained, links/junctions and Netlify-invalid path characters rejected.
- Uploads use a manifest and one upload per requested digest. Redirects are
  refused and errors avoid response bodies/token values. HTTP requests have
  timeouts; create/upload/poll has a five-minute deadline. No automatic retry
  after an uncertain create. Failed builds cannot create deployments; failures
  report the completed stage and possible remote completion before retrying.
- Unit/integration tests use a private loopback API endpoint, never production
  endpoint overrides or real credentials. CI additionally runs an explicitly
  ignored hosted-container test using its checkout-built engine image and a
  loopback API, verifying actual uploaded HTML/links/assets/prefix and source
  preservation. It does not create a real Netlify deployment.

Validation/source/PR evidence is recorded below as checks finish. Next: saved
CI configuration/generation and remaining delivery/import composition, then
other incomplete CLI families. The full CLI remains unfinished. No action is
needed from the user for implementation or isolated acceptance.

Local verification: **245 Rust tests passed**, with one real-container acceptance
test intentionally ignored locally and explicitly run by CI. Warnings-denied
Clippy, formatting and strict/deep docs validation (49 routes, 23 examples) pass.
The mock HTTP fixture explicitly makes accepted sockets blocking, which is
required on macOS when its listener is nonblocking. Actual container and all-six
native package results remain pending in the PR; do not confuse local protocol
tests with successful live provider publication.

Implementation source: `37631df8dbb2455f3e52a4756bf1a50fb57708be`, tracked in
[PR #33](https://github.com/mylife-inc/Zellij/pull/33). Consult its latest checks
for final-head platform and hosted-container results. The public documentation
and released binary have not been updated by this PR.


CI selection was corrected after review: the hosted acceptance uses its full
Rust module/test name and first verifies that the named test exists, preventing
a zero-test pass. Hosted delivery stages beside the selected content rather
than in the OS temp directory, so a runtime sharing the project (such as Colima)
can access its export mount. Portable output still uses OS temp. The hosted
fixture checks that project-local staging is removed afterwards.


Run `39153907349` at `4a82895` exercised the real hosted export and completed all
mock API uploads/polling, then exposed a fixture assertion error: the landing
page redirects to `/manual`, so guide links/assets must be checked in
`manual.html`. The fixture now checks the landing redirect and actual guide
HTML separately. Use subsequent final-head checks for complete acceptance.


Run `39154796368` passed all six package jobs, all three portable suites,
engine/schema/showcase, and completed hosted staging/build/upload. Its remaining
fixture assertion still assumed that the root redirect serialized a prefixed
URL. An isolated local production export established the actual representation:
Next serializes `NEXT_REDIRECT;replace;/manual;307;` and its client router applies
basePath. The fixture checks that instruction, the prefix, and rendered links
in manual.html. Local browser acceptance confirms the prefixed landing redirect,
guide navigation, logo, reload and resource requests. The hosted CI test now runs
immediately after engine-image setup, before the longer image suites, to make
any further export/upload failures visible earlier. Final-head CI remains the
merge gate; prior failed runs are not represented as complete acceptance.


### Netlify final acceptance and merged baseline — 2026-10-18

PR #33 merged as `e9534f9705898f2041c08eb83693994c44189ac3`.
Final source/test head `372c42cb791ceca28e8bf88ae757608535eb4441` passed all
13 jobs in [run 39156106290](https://github.com/mylife-inc/Zellij/actions/runs/39156106290),
including six native platforms and actual hosted export/upload through the mock
API. The merged main also passed all 13 jobs in run `39157419834`, and base
publication `39157419858` succeeded. This completes the pending evidence above;
it does not constitute live Netlify account or released-binary acceptance.

### Saved CI settings and workflow generator — 2026-10-18

Baseline: main `e9534f9705898f2041c08eb83693994c44189ac3`; implementation on
`codex/cli-workflows`. Latest public native release remains `zellij-v0.1.3`
(2026-08-13), refreshed from GitHub on 2026-10-18. No new release, live deployment,
secret provisioning or public documentation deployment is part of this work.

- Versioned sites share typed `ci` settings: system (github), branch (literal),
  workflow (filename), zel-version (exact), triggers (pull_request/push/manual),
  tags (positive patterns), schedules (five numeric UTC cron fields), production.
  Unknown fields/unsupported events/credentials are rejected. Settings are
  removed before portable presentation validation and accepted by hosted roots.
- `zel ci [site]` and `ci init github --site-root <site>` share resolution/writing.
  Site discovery follows the existing convention; Git locates the selected
  site's repository. Workflows live at its workflow root, using relative paths
  and filters covering configuration/content/workflow changes. `./run` and
  --site-root disambiguate reserved action names. --content remains an alias.
- Generation serializes YAML, passes paths/values through step environment, and
  escapes path-filter metacharacters. Control characters and GitHub expressions
  in input paths are rejected. Atomic writes retain --force policy and reject
  linked destination ancestors, including junctions. No local runtime needed.
- PRs check/build without publishing. Allowed branch push/manual/schedule or
  configured tag triggers check and execute a single native build/delivery.
  Manual runs on other branches only verify. Saved static/Next image delivery
  uses publish; Netlify uses native deploy. No npx/Netlify CLI and no double build.
  Legacy --target image retains one Next image build followed by exact-tag push.
- GHCR uses scoped packages permission/token; other explicit registries use
  REGISTRY_USERNAME/PASSWORD secrets. Netlify ID precedence is flag > local env >
  saved site, falling back to repository variable/secret when absent. Credentials
  are scoped to publication steps. Draft default; ci.production/--prod is explicit.
- Local generation is the default even with credentials in the environment.
  Only --provision uploads NETLIFY_AUTH_TOKEN through gh stdin, after writing the
  workflow. Partial failure reports the remaining file and uncertain remote
  outcome. --no-provision remains accepted. No dispatch occurs while generating.
- Release boundary: minimum compatible tool contract is planned 0.2.0 (including
  candidates). Development 0.1.3 cannot supply a valid default pin, so users must
  explicitly select a future compatible release to prepare the file. Generation
  never claims that release exists. Compatible released builds will default to
  their own version. Installer and binary come from the exact release; engine
  builds pin its matching image. new project prepares the planned 0.2.0 workflow
  and explicitly reports the release prerequisite. Actual release acceptance is
  pending; 0.1.3 is not misrepresented as compatible.

Local validation: 248 workspace Rust tests passed (the existing real-container
acceptance remains intentionally ignored locally), strict/deep docs validation
passes 49 routes and 23 examples, and five generated variants pass actionlint
1.7.12. Integration executes generated portable check/build steps with spaces,
quotes, Unicode and shell metacharacters, and mocks gh to establish stdin-only,
explicit provisioning and failure reporting. Existing Netlify protocol tests use
loopback only. Final follow-up checks/PR/platform evidence are recorded below.

During verification the disk filled; cargo-generated target files and the prior
isolated Netlify export were removed, recovering about 6 GB. Subsequent local
Rust checks use CARGO_PROFILE_DEV_DEBUG=0 and CARGO_INCREMENTAL=0 to reduce disk
use. No authored files or unrelated projects were removed.

Next: compatible native release preparation/acceptance and remaining CLI
composition (ship/configured imports), new app, fmt/watch, portable dev, hosted
view and advanced maintenance. Full CLI completion remains open. Generated CI
files require release publication before real use; implementation/tests require
no user credential or account changes.


Workflow implementation source: `8904cdd11e17e17d2cb59bb481e9220741996a12`.
Final local follow-up checks passed the CI integration, core/scaffold suites,
warnings-denied Clippy, formatting and strict/deep documentation validation.
Cross-platform and real-container acceptance remains the PR merge gate; inspect
the GitHub checks associated with this source for final-head results.


The workflow milestone is tracked in [PR #34](https://github.com/mylife-inc/Zellij/pull/34).
A follow-up documentation sweep updates the new-project/existing-repository labs
and container/check guides that still showed zel.toml, unpinned installs,
check-only PRs, implicit provisioning and unimplemented deployment claims.


Release-readiness review also corrected the project starter's version fallback:
compatible future/candidate binaries pin their own version, while the current
incompatible development version prepares planned v0.2.0. It no longer keeps
an unconditional 0.2.0 pin after a later release changes generator commands.


Run `39159315383` at `23657c6` exposed a pre-existing Windows timeout-fixture
race: the native Netlify mock server indexed an empty request line when a client
closed TCP before sending headers. The fixture now ignores incomplete/cancelled
connections and records only complete requests, with a deterministic regression
covering empty connections, truncated headers and bodies followed by a valid
request. Native delivery behavior is unchanged. Final-head CI remains required.


### Implementation update — 2026-10-18, configured import and ship

PR #34 merged at `4bc5ef6cb5618ea98b208b45e9338b33d2ae5c65`. Final source
`d80875aff5179f28abcab745e569b8e2f7212912` passed all 13 CI jobs in run
39159674967. Merge CI 39160901027 and base image run 39160901001 also passed.
This closes the earlier pending final-head evidence for the workflow milestone.
The latest public native release is still zellij-v0.1.3 (2026-08-13); these new
CLI commands have not been released or deployed to the public docs site.

Current branch implements shared import preparation/reporting, configured output
and ship composition. Convert/cv and migrate/mig keep raw content as default;
--layout site writes version: 1, renderer: hosted plus content/. --from and
--dry-run are shared. Migrate validates after import; dry run skips writes/checks.
Equal-confidence format matches now refuse to guess; --from resolves them.
Windows junctions beneath imported content are rejected like Unix symlinks.

Ship requires source, --out and an explicit destination: --provider netlify
with --site-id/--site or NETLIFY_SITE_ID; or --image static|next with --tag.
Both converters emit hosted content and retain its runtime/HTTP requirements.
All input/conversion/output-path checks finish before writing. Conversion warnings
remain visible. A complete imported site is retained after any later failure.
Nonsecret destination settings are saved for later publish; credentials and
--prod are never saved. Netlify remains draft by default. Plan resolution and
publication share the existing delivery types/executor, building once and never
pushing/uploading after a failed build. No local persistent HTML export is implied.

Local evidence: 255 workspace Rust tests passed, followed by focused tests after
reducing import memory copying; formatting, warnings-denied Clippy, actionlint
1.7.12, strict/deep docs validation (49 routes, 23 examples), and MkDocs/Docusaurus
conversion checks against actual MDX rendering passed. New tests cover aliases,
layouts, ambiguous detection, source/asset preservation, invalid/inert plans,
missing runtimes/credentials, retained imports, exact tags and build/push failures.
CI adds all-platform import tests and real static/Next ship publication to a
loopback registry followed by browser navigation/assets checks. Container/publication
acceptance remains the merge gate; source/PR/check evidence will be appended.

No local daemon was started, external registry/account was used, or live deployment
or native release performed. Remaining CLI work includes new app, fmt/watch,
portable dev, hosted view, other providers and advanced maintenance. Compatible
release preparation/publication remains a separate milestone.


Import/ship implementation source: `0b59859` in
[PR #35](https://github.com/mylife-inc/Zellij/pull/35). The pull request records
final-head platform and container acceptance; consult its checks for merge status.


PR #35 run 39162890191 exposed a Windows configured-import path bug: joining
content/ with a generated relative path inserted a backslash rejected by the
portable write-plan validator. Prefixing the already-validated relative path
with a literal slash fixes the implementation; the Windows junction fixture also
uses OS-native path components for cmd.exe. All-platform rerun is required.


### Implementation update — 2026-10-18, editable Next.js wrappers

PR #35 merged at e7ff7e01bd291dd58f387f0c865c1111d8dfeb70. Its final source
cf8296b23f816c805575a77d0468e29c05b89b1e passed all 13 jobs in CI 39163178585;
merge CI 39164568012 and base image 39164567998 also passed. This closes the
pending Windows configured-import and final-head evidence above.

Current source implements `new app` / `na`. Scaffolding uses no Node, npm or
container runtime. Existing hosted content is selected by --site-root or normal
site discovery and referenced at its canonical absolute location, without copying.
--with-docs instead creates a configured hosted starter under docs/zellij in the
new app. Destination defaults to ., has no force override, and must be empty
apart from noncolliding dotfiles. Engine/content overlap and link protections
apply before writes. Portable and legacy marketing content are rejected with
explicit guidance; image next still supports portable adaptation.

A built local engine package is required because npm distribution does not exist.
--engine selects it; the sole implicit location is ./engine/packages/zellij in
the working directory. Identity, version, wrapper-config capability and required
CLI bundles are checked. npm file dependencies retain literal path punctuation;
percent encoding failed actual installation and was corrected. Windows extended
canonical prefixes are normalized for npm. Node/npm are required only to run the
created app. --name, --port and comma-separated --features are documented.

The engine initializer exposes wrapperTemplate(); scripts/prepare-app-scaffold.mjs
embeds its core files, dependency/override versions and feature metadata in Rust.
CI checks drift. Node initializer and Next images continue using that definition.
Versioned zellij.config.yaml contains version: 1 and content only; it is separate
from a documentation site's zel.yaml and content/site.yaml. Check, preparation,
Next bootstrap and rendering now agree on that selector. The previously defaulted
check content path was corrected. Unversioned legacy config keeps its validation.

Local validation: 259 workspace Rust tests passed; focused new-app tests and
warnings-denied Clippy passed after the npm-path/compatibility refinements.
Engine tests, workspace typechecking, engine build verification, template drift,
formatting, changed-workflow actionlint and strict/deep docs validation passed.
The isolated acceptance script installed the real copied engine through npm from
a path containing spaces/#/%, validated starter/external content, built Next with
Turbopack, typechecked, and exercised Chromium navigation, reload, assets, search,
prefixes and 404s. No workspace node_modules shortcut was used for that app.
Full-workflow actionlint also reports pre-existing shellcheck SC2046 warnings in
base-image.yml; the changed ci.yml passes its check.

CI adds native scaffold integration and isolated installed-wrapper acceptance
on Linux, macOS and Windows. Final PR/source/platform/container evidence will be appended.
No local daemon was started, live provider/registry publication performed, or
native release/public docs deployment made. Latest public native release remains
zellij-v0.1.3; all current CLI additions still need a compatible release. Remaining
work includes fmt/watch, portable dev, hosted view, other providers and maintenance.


Editable-wrapper implementation source: `3877db8fc6e68bc4ceee94534cd95f0305df0c54`,
in [PR #36](https://github.com/mylife-inc/Zellij/pull/36). That PR tracks final-head
Linux/macOS/Windows installed-app acceptance and existing container checks.


### Implementation update — 2026-10-18, YAML formatting and validation watch

PR #36 merged as d773c1832918a0c426513e10949367105d9e00b6. Final head
3b523091dd37af96bb6cc167681adf166d0df835 passed all 13 jobs in CI 39170572087.
Merge CI 39171833213 and base image 39171833271 also succeeded, closing the
previous editable-wrapper milestone's pending platform/container evidence.

Current source implements fmt and watch with configured-site discovery. Fmt
selects hosted YAML plus zel.yaml, or only zel.yaml for portable sites, excluding
attachments/generated output and refusing links/junctions. It uses pretty_yaml
0.6.0 / yaml_parser 0.3.0 syntax trees, schema ordering and compact collections,
with comments/anchors protected from reordering. An upstream normalization of
|+ trailing blank lines was caught by regression tests: authored block scalars
are now restored with adjusted structural indentation, and parsed-value equality
is required before writes. Staging closes file handles before publication, keeps
permissions, and checks concurrent edits. Invalid late input writes nothing;
replacement is atomic per file, not filesystem-wide on storage failure.

Watch reuses portable checking and the hosted engine. Byte snapshots detect
same-size/timestamp-preserving edits; debounce and checks are serial. A configured
session pins layout and renderer and recovers after invalid content/config edits
without scanning unrelated repository/output paths. Ctrl-C waits for current
validation, then exits0. Hosted runtime failure is startup exit4; portable checks
and formatting never probe external tools. No source/output/build/publication
side effects are part of validation watch.

Local tests cover formatting scope, check-only writes, late errors, permissions,
low descriptor limits, multiline values, comments, scalar types and idempotence.
Disposable copies of showcase, zellij-docs and codeseal-docs remain strictly/deeply
valid after formatting; Markdown/media stay byte-identical. The actual portable
watch process recovers from initial bad content, restored-size/timestamp edits,
malformed/deleted config and changed renderer, then exits cleanly on Ctrl-C.
All 265 workspace Rust tests, warnings-denied Clippy, formatting, changed-workflow
actionlint and strict/deep local documentation validation passed. CI adds formatter
corpus and portable-watch acceptance on Linux/macOS/Windows, including Windows
junction and console-signal coverage, plus hosted watch with the CI-built engine.
The portable matrix timeout is 30 minutes to allow cold installed-wrapper builds
and these added checks. Final source/PR/platform evidence will be appended.

No local daemon was started or live provider/release/public docs publication
performed. Public native release remains zellij-v0.1.3. Remaining work: portable
dev, hosted view, additional providers and maintenance, plus compatible release
and documentation publication. Historical audit findings above remain evidence,
not a claim that the product is complete or bug-free.


Formatting/watch implementation source: `47fbcfbb17d435d584a758093415160d42513bc7`
in [PR #37](https://github.com/mylife-inc/Zellij/pull/37). That PR tracks final-head
Linux/macOS/Windows formatting/watch and real-container validation acceptance.


PR #37 run 39175688739 caught a Windows formatting publication failure: the
contained-path validator expects portable relative separators, while WalkDir
returns OS-native paths. Splitting actual path components and joining with /
fixes that preflight without reinterpreting Unix backslashes. Windows junction
refusal and no-write tests already passed; all-platform acceptance must rerun
on the fix before merge.


### Implementation update — 2026-10-18, portable development and hosted preview

PR #37 merged as b2771c81a2e69871abb69557c16b999dcaddd93c. Final head
aaad79fa7b9aee3ab9073bcca8d93fcf54ab11cf passed all 13 jobs in CI 39176083582,
including the Windows formatter correction. Merge CI 39177511838 and base-image
run 39177511802 also succeeded. This closes the previous pending evidence.

Current source makes portable dev equivalent to view --watch --no-open, with
native validation/building, configured discovery/output/port, browser reload and
no external-tool probe. Hosted dev retains Next.js development; port 0 now fails
with guidance to use view --watch --port 0 instead of advertising an unusable URL.

View/v now supports hosted static exports, with a native loopback HTTP server
and an optional browser. The container engine returns a staged flat export and
its actual resolved basePath. Index/prefix validation and a before/after source
snapshot precede publication. Missing runtime, occupied port, export failure,
changed source during build or cancellation preserves existing output. Metadata
is removed before publication; ordinary build and deployment layouts are retained.

The shared watch worker serializes/debounces builds and serves the last good
site after errors. Hosted watches pin renderer/layout and basePath. Config or
prefix changes show diagnostics; restoration recovers. Changed inputs during a
hosted build discard its stage and trigger a new build. Ctrl-C stops HTTP and
waits for the active build to finish cleanup, discarding its result; it does not
immediately kill a long container build. Output/port remain fixed until restart.
The reload client is injected only into HTTP responses. Portable exports retain
file:// support; hosted exports continue to require HTTP serving.

Local validation: all 265 Rust tests and warnings-denied Clippy passed on the
final implementation, including the hosted-dev port-zero guard.
Chromium/WebKit portable preview and dev acceptance passed. A standalone Rust
runtime-protocol fixture exercises real native CLI orchestration and Chromium:
prefix serving, browser readiness, reload/error recovery, configuration recovery,
stale-build refusal, initial/rebuild cancellation and previous-output preservation.
That fixture does not replace the real container renderer. CI now runs it on
Linux/macOS/Windows and adds a separate actual hosted-engine browser test.
Strict/deep local documentation validation and changed-workflow actionlint passed.

No local daemon was started; no live provider, native release or public docs
publication occurred. Public native release remains zellij-v0.1.3. Remaining work
includes other deployment providers, maintenance commands, additional dev flags,
compatible release distribution and documentation publication. Final source/PR
and platform/container results will be appended.


Portable-dev/hosted-preview implementation source:
`e9e47deb23f68ee9dafcada6b4fe657377e4db05`, in
[PR #38](https://github.com/mylife-inc/Zellij/pull/38). That PR tracks the final-head
platform tests and real-container browser validation before authorized merge.


PR #38 run 39180630629 caught a real Next export routing defect in the native
server: a guide/ directory for child routes or RSC payloads hid guide.html when
that directory lacked index.html. Resolution now falls through to the flat HTML
page in that case, preserving existing directory-index redirects and containment.
Nine HTTP server regression tests and warnings-denied Clippy pass. The real-engine
browser test now runs earlier in CI, verifies the initial home redirect, and
selects the preview error notice separately from Next's accessibility announcer.
Final platform/container checks must rerun on this fix before merge.

Local confirmation also served an existing real Next export through hosted view
with the reload client injected. Chromium passed parent-route loading, navigation,
reload, hydration and search without a local container daemon. The hosted CI
search check waits for the client handler to attach instead of sending a shortcut
immediately after receiving server-rendered HTML.


PR #38 run 39181963487 passed the real exported home redirect, parent/child
navigation and images with the routing fix. Its search step caught a fixture
omission: hosted nav.search is opt-in, and this fixture had not enabled it.
The fixture now explicitly sets nav.search: true. No renderer behavior changed
for that test correction; final-head acceptance must still complete.


PR #38 run 39182453254 passed real hosted preview end to end. The Windows
orchestration fixture instead selected the runner's installed Docker client,
exposing an existing engine bind-path defect: Rust canonical Windows paths use
\\?\ prefixes that Docker rejects. Content and output mounts now use standard
drive/UNC syntax; Unicode, spaces and Unix backslashes remain intact. All 22
engine tests, Clippy and local hosted browser/lifecycle acceptance pass.
The native fixture now removes case variants of PATH/endpoint keys, explicitly
selects an isolated Podman executable, verifies that fixture performed the build,
and rejects verbatim mount paths. CI runs it before the slower wrapper tests.
Final-revision Windows and full-container validation are still required.


## PR #38 completion and maintenance follow-up — 2026-10-18

PR #38 merged as `b83de924231bce67d6478fa57124aee5c5faf21c` after all 13
checks passed on head `0437fede8e7127dc0cda703bd7a2908e03f0be57` in run
39183818152. This included 267 Rust tests, native Linux/macOS/Windows preview
acceptance, six native archives/installers and real-container hosted preview.
The post-merge base-image run 39185352421 succeeded. CI run 39185352441 first
failed in Windows ARM PowerShell 7 with `Internal CLR error (0x80131506)`;
PowerShell 5.1 had passed the same installer checks. Rerunning the failed job
without source changes succeeded. Both post-merge workflows are now green.

The maintenance branch implements `zel version [--json]`, `zel pull [--image]`
and `zel upgrade [version] [--check|--force] [--release-base-url]`. These commands
are already short, so no redundant aliases were added. Help/completion comes
from the same Clap schema. Version reporting is entirely local: CLI/target,
zel.yaml version 1, embedded engine package version, schema SHA-256 and selected
image; it does not claim a mutable container tag matches that embedded engine.
Pull uses the existing bounded runtime detection and endpoint rules, accepts
pinned SHA-256 digests, and leaves runtime installation/start/login to the user.

Upgrade keeps the established public `mylife-inc/releases` / `zellij-v<version>`
channel. Latest lookup considers only stable Zellij releases (up to 1,000
records); explicit versions permit prereleases but never downgrades. `--check`
changes no files; equal versions are a no-op unless explicitly forced. Archive
checksums are required, names/types/count/size and CPU/OS headers are checked,
and the candidate must report the requested version within five seconds.
Downloads use Rustls; credentials go only to the fixed API, never asset mirrors.
Staging is beside the executable, a persistent owned lock serializes upgrades,
and cancellation/verification failure preserves the current executable. Unix
commits with rename; Windows retains a working backup and attempts restoration
if the final rename fails. Retained Windows backups can be deleted after exit.
No self-deleting scripts or child helpers are needed for replacement.

Local native acceptance discovered macOS tar automatically including ._zel
AppleDouble provenance metadata. Packaging now sets COPYFILE_DISABLE=1 so the
archive contract is one executable. Acceptance uses the actual native binary
and loopback mirrors for combined/per-file checksums, corrupt/missing/unsafe
archives, version mismatches, forced replacement, cancellation, a higher-version
native fixture, and isolated Podman pull success/failure. The same script runs
on all six package targets. It never upgrades a user's installation or contacts
a real deployment provider/registry. Final platform results follow below.

Public native release is still zellij-v0.1.3, and source still reports 0.1.3;
`upgrade` therefore cannot deliver unreleased source features. Compatible release
publication/version selection remains separate work. Other provider integrations,
eject/features, remaining advanced options and final release/documentation
publication remain in the backlog. No local daemon, public release, live provider
deployment, sealing operation or branch deletion was performed.

Maintenance local verification: 273 Rust tests, warnings-denied Clippy and fmt,
strict/deep docs validation and the complete native package/install/upgrade
acceptance script passed on macOS x86-64. Native acceptance used a locally
packaged current debug executable and isolated fixture destinations, including
an upgrade to a higher-version Rust binary. Release-profile acceptance on all
six platforms and full-container CI remain the PR gates.

Maintenance implementation source d4d8c01867f68bd3a439f93a47e9989b295bcdcf
is tracked in PR #39. Live `upgrade --check` also confirmed the stable channel
returns 0.1.3 without changing files. Dependency review then found RustSec
RUSTSEC-2026-0367/0368/0369, issued October 17, affecting ZIP extraction helpers.
Our reader rejects links and does not call those helpers, but the dependency
has still been advanced to patched zip 9.0.5 with only deflate enabled. Its
fallible filename decoding is checked explicitly. Final CI must use this update.
Reference: https://rustsec.org/advisories/RUSTSEC-2026-0369.html

Patched ZIP source d6b6bdd1e3d8aa3b3e45b40b9d72241cb24f3fa5 passed all six
native package jobs in CI run 39189577614. Review then added a guard against
an old running process overwriting a newer executable already installed at its
path: after acquiring the lock, the on-disk executable must report the running
process's version before downloading. Native acceptance now also launches a
second updater during a held download and requires lock refusal. This follow-up
must pass the final full CI before merge.

Explicit-version `upgrade --check` is a local version comparison, not evidence
that a release or platform asset exists. Its output/documentation now say this
explicitly; only unpinned check mode queries the stable release channel. Actual
installation always verifies the downloaded archive before replacement.


### Maintenance merge / next milestone — 2026-10-18

PR #39 merged as 3f8392c8628ed8148171a729b38b6d7bcf9151fe after all 13 jobs
passed on final source 3173b717e2303f4c0d53bc1cfae6a3b07bc4ea5a (run
39192023066). Post-merge CI 39193324796 and Base image 39193324795 both
completed successfully on that merge. Local main was clean and synchronized.
The next milestone implements eject and features; it does not publish a native
release or deploy documentation.


### Eject and features implementation — 2026-10-18

The native CLI now registers `eject [site] --out <destination>` (or --with-docs)
and `features <selection> [--site-root <path>]`. Eject copies the selected engine
distribution's /app and /opt/zellij using a stopped runtime container. It stages,
validates, cleans up and publishes only into new/empty output. Vendored engine
sources use file:vendor/zellij; tsconfig excludes those source/test copies from
the wrapper's own typecheck. External hosted content is referenced; image starter
content moves to docs/zellij/content with zel.yaml. The historical 'no npm'
promise was wrong: the distribution includes sources, not installed dependencies.
Users run npm install afterward. Runtime discovery/prerequisites remain shared.
Cancellation/copy failures attempt cleanup; cleanup failure reports the container
name and does not publish. No local Docker daemon was started.

Features maps maths/diagrams to portable markdown.math/markdown.mermaid switches
or to a compatible editable wrapper's direct feature packages. Portable edits
preserve YAML comments and unrelated spelling; alias/anchor targets are refused.
Wrapper changes retain unrelated manifest data and lockfiles, with npm install
left explicit. Hosted content without a wrapper is refused with useful guidance.
The feature dependency mapping is shared with the generated engine template.

A pre-existing engine limitation was confirmed: npm optionalDependencies may be
installed even when a wrapper has no direct feature requirements. Hosted `none`
therefore removes direct requirements, not rendering capability or all installed
packages. Strict exclusion/explicit hosted capability switches remain follow-up
work. The current documentation and historical spec corrections state this;
portable toggles do control rendering. No zero-bug/completion claim is made.

Local checks: 277 Rust tests, warnings-denied Clippy, Rust formatting,
actionlint for the changed workflow, and deep/strict documentation validation
(55 routes / 53 guide pages) passed. Native runtime-protocol acceptance exercises
selected sources, digest overrides, movable engine references, feature changes,
nonempty output, failed copy, incompatible engine, links and real Ctrl-C cleanup.
CI adds the same extraction acceptance on Windows/macOS/Linux and a real-container
eject + npm install + validation + production build + typecheck/browser check.
Final CI evidence and merge status must be refreshed before relying on this work.
Remaining work includes other providers/advanced options, the hosted dependency
limitation, compatible native releases and documentation publication.


### Eject/features merge and hosted selection follow-up — 2026-10-18

PR #40 merged as b318f74dbbe617aeb8b6aa65947172f7b01c4f5c after all 13 jobs
passed on source 72a18ad6d00bab3751fd2f8d9f0839f01971ec24 (CI run
39198443284). Post-merge Base image 39200372897 passed; CI 39200372898 was
still running at this follow-up's start. No native release was published.

The next change closes the hosted limitation recorded above: explicit wrapper
package.json zellij.features controls rendering, including when dependencies
remain installed. Optional packages become optional peers; their devDependency
entries serve the engine's own build/tests and are not installed for consumers.
Reference: https://docs.npmjs.com/cli/configuring-npm/package-json/#peerdependenciesmeta
Next's supported package aliases handle absent disabled peers in both bundlers:
https://nextjs.org/docs/app/api-reference/config/next-config-js/turbopack#resolving-aliases
The local fresh-install test caught absolute-path Turbopack alias resolution;
using an exported engine subpath resolves the portability issue. Full acceptance
and merge evidence must be refreshed before treating this change as completed.


Final local hosted-feature acceptance passed: a standalone wrapper installed
without Mermaid/remark-math/rehype-mathjax and built/typechecked/rendered with
Turbopack; enabling all installed them and rendered real MathJax/Mermaid output;
switching to none with packages deliberately retained disabled both under Webpack.
Native mindmap SVG and browser-error checks passed in every state. Each npm state
is checked in a fresh Node process, avoiding stale negative package lookups.
No locked dependency versions changed. Source commit and hosted CI evidence will
be recorded after PR validation; no new native release or manual deployment occurred.


Hosted-feature source c27c64d3f4d1cfc3315383a83b9d43ef17d9c28a is reviewed in
[PR #41](https://github.com/mylife-inc/Zellij/pull/41). All local acceptance above
passed. Refresh the PR's final head/checks and merge status before relying on it.
The previous PR #40 post-merge CI 39200372898 and Base image 39200372897 both
completed successfully. Public native release remains zellij-v0.1.3 (2026-08-13).


PR #41 CI run 39202250897 passed the new standalone feature acceptance and all
container/image/export checks through section-template rendering. Its final
conversion check exposed a test-harness ESM/CommonJS interop error when bundling
the new shared config helper (dynamic require of node:fs). The conversion helper
now supplies Node createRequire, matching the production CLI bundle. Local MkDocs
and Docusaurus conversion acceptance passed after this correction, including
rendered routes, heading fragments and asset bytes. Final CI must be refreshed
on the fixing head; the earlier run is not a successful completion record.


### Hosted selection merge and shared CLI paths — 2026-10-18

PR #41 merged as 02eaae86132c68c21a450bfc90637c4e5b7e05bd after all 13 jobs
passed on final source 7a2e35322c9b058041e5d367665a769c6b5d004f (CI
39204069716). Post-merge CI 39205838063 and Base image 39205837958 both
completed successfully. This closes the preceding conversion-harness follow-up.

Source 58b7129ea18f4add46154fe8b123edc383626152 implements global --cwd and
--site-root. The parser normalizes site selection into existing handlers after
shortcut expansion, preserving discovery, output protection and watch reloads.
Main changes the process working directory once before handlers/subprocesses.
CLI paths use that directory; paths stored in configuration retain their existing
bases. Invalid cwd exits 2; --help/--version do not access it.

Site selection covers existing-site commands, including new page/guide/app,
features, CI generation and omitted-tag legacy image run. CI --content remains
supported. Positional sites, --dir, --content, --with-docs, explicit image-run
tags and unsupported command uses conflict before side effects. Global flags
may appear before or after commands, but only once. Clap otherwise discards
parent global values when a child repeats them; a post-parse token guard rejects
that ambiguity while preserving equals-form values and the -- delimiter.

Local verification: all 284 Rust tests, warnings-denied Clippy, Rust formatting,
diff checks and strict/deep documentation validation (55 routes, 53 guide pages,
23 examples) passed. Regressions exercise Unicode/spaced paths, nested Git
repositories, config-relative versus CLI-relative output, aliases, completions,
conflicts, missing paths and preservation of authored files. Existing CI runs
these parser/site-layout/workflow tests on Linux, macOS and Windows. Refresh
this branch's final CI/PR status before treating the milestone as merged.

Remaining work includes explicit --config file selection, other advanced flags
and deployment providers, matched native releases and documentation publication.
The public native release is still zellij-v0.1.3 (2026-08-13), predating these
features. No local Docker daemon, native release or manual deployment was started.


### Explicit configuration selection — 2026-10-18

PR #42 merged as 4cae19f628f8c8665a747452a8d56da8e20c1f2d after all 13 jobs
passed on 1bb7e04eed7917bbb8cdc4a2166dc671156d21e5 (CI 39207701478).
Post-merge Base image 39209473109 passed; CI 39209473176 was still completing
its final conversion check when this milestone was prepared.

Source 1653d84afb2b7ad065e0eb00c60ddd8d700c49db implements global --config.
For example: zel b --cwd ../product --config docs/manual/preview.yaml.
SiteInput distinguishes a directory from an explicitly selected file throughout
core loading, portable compilation/adaptation, hosted commands, image/delivery
composition, authoring, wrapper creation/extraction and CI. Site retains the
normalized input and actual config path, so downstream operations and watcher
reloads cannot rediscover a different file. No process-global configuration
variable or temporary config copying is used.

The selected YAML's parent determines root/content/output bases. Both versioned
and legacy flat portable formats work; a missing explicit file cannot fall back
to zel.yaml or raw Markdown. Regular-file and symlink/junction checks apply.
Parser normalization retains aliases and rejects repeated, conflicting or
inapplicable selectors before effects. --config and --site-root conflict, as do
positional sites, authoring --dir, CI --content, --with-docs and image-run tags.
The separate wrapper zellij.config.yaml format keeps its existing purpose.

Watch tracks the chosen file, including hidden legacy config names, recovers after
restoration and preserves previous exports on errors. Formatting and portable
feature changes edit that file; check JSON reports its actual name. CI serializes
the repository-relative file as SITE_CONFIG and consistently passes --config to
check/build/delivery; site-folder change filters cover it. User paths remain data,
not interpolated shell source. Default discovery and existing missing-content
errors remain compatible.

Local checks passed: 290 Rust tests, warnings-denied Clippy, formatting, targeted
CI workflow lint, and strict/deep docs validation (55 routes, 53 guide pages,
23 examples). Actual portable browser preview (Chromium/WebKit plus named-config
Chromium), named-config validation watch/Ctrl-C, hosted preview with an isolated
runtime fixture, and standalone wrapper npm install/build/typecheck/browser
acceptance all passed. CI additionally exercises named hosted configs with its
real engine and extraction path. Refresh final PR/head/checks before claiming
cross-platform completion or merge. No local Docker daemon was started.

Remaining work: shared output controls, additional deployment providers, other
advanced/converter gaps from the contract, matched native releases and published
documentation. Public zellij-v0.1.3 still predates these source capabilities;
this milestone does not publish releases or manually deploy a site.


### Shared CLI output controls — 2026-10-19

PR #43 merged as 8912aee8f7ed037fe8b491a5fdf5d916fe997637 after all 13 jobs
passed on ea6ada4e2b77b86ecf014cfa1f7291d50a2f06ed (CI 39211231453).
Both post-merge runs passed: CI 39213225423 and Base image 39213225268.
PR #42's previously pending main CI 39209473176 also completed successfully.

Source f7d141441652cc951a0f75ae98d83a672b1181f9 implements --quiet/-q, --verbose and
--no-color across the registered command tree and aliases. They are presentation
settings, initialized once before workers; there is no change to validation,
strictness, overwrite authority, browser launch or publication intent. Quiet and
verbose conflict before effects, including nested command placements. No -v flag
is introduced; v remains the view shortcut. Nonempty NO_COLOR also disables
styling, including forced-color parser help/errors and normalization errors.

core/reporting.rs distinguishes routine progress, requested human reports,
diagnostics and verbose stderr traces. Generated stdout data and JSON remain
unchanged. Native warnings/errors, conversion limitations, dry-run plans,
formatting findings, registration snippets and inspection/doctor/upgrade reports
survive quiet. Hosted validation retains its mixed full engine report. Quiet
preview workers still rebuild and recover, but suppress success/readiness output.
Verbose reports timing, selected site and runtime discovery without raw argument
or environment dumps; credential-bearing runtime endpoints remain redacted.

core/process.rs owns child lifecycle and optional output capture. Quiet suppresses
successful external logs, replaying the final 64 KiB per stream on failure. Color
filtering streams through anstream (already locked via Clap). Anonymous temporary
files use independent read/write handles, including on Windows; no undrained pipe
or descendant-held pipe delays cleanup. Files are unlinked before child start;
long-running commands may consume temporary disk until handles close. Explicitly
silent cleanup stays silent. Clients and run containers receive color preferences.
The existing human-text stream placement is retained for compatibility.

Local verification: 299 Rust tests passed, with the final focused 9-test output
suite rerun after log-reader refinements; warnings-denied Clippy, formatting,
workflow lint, and strict/deep docs validation passed (55 routes, 53 guide pages,
23 examples). Actual quiet preview tested build/HTTP/rebuild/failure preservation/
recovery/Ctrl-C. Named-config validation watch and no-color hosted preview with
an isolated runtime plus browser also passed. CI adds these tests on Linux,
macOS and Windows, real no-color hosted preview and quiet/no-color extraction.
Refresh the final PR head/checks before claiming hosted CI or merge completion.

Remaining: other deployment providers, advanced/converter gaps in the contract,
matched native release and published documentation. Public zellij-v0.1.3 predates
these source features. No local Docker daemon was started, no native release was
published, and no manual live deployment was performed in this milestone.


### Image platform selection — 2026-10-19

PR #44 merged as 2ada5fe0ad1e1227c70eb963746cca929fc72af5 after all 13 CI
jobs passed on 52cc3620cdfbdff5f8db4d443abe9a925619cbb4 (39216062489).
Its main CI 39218117100 and Base image 39218117079 both passed.

Fixing commit c9db8dfc1edbe7e702f7585e2680fbceb002435a implements single-platform Linux
image selection (linux/amd64 or linux/arm64) across image static/next/build,
si/ni, build --target image, saved publication/deploy, ship --image and CI.
Optional image.platform supplies the default; CLI overrides it. image.name is
now optional for local image builds, while publication still requires an
explicit destination. Plans show the platform, ship retains it in zel.yaml,
and generated CI freezes it in both jobs. A follow-up uses native ARM64 runners
for generated workflows, avoiding known QEMU/Node failures in release builds.
HTML/Netlify flags and unsupported/combined platform values fail before work.

Podman builds omit Docker's unsupported --progress option. nerdctl pushes use
--all-platforms to preserve a foreign single-platform local image. Build failure
keeps diagnostics, explains base/emulation requirements and never proceeds to
push. No local emulator/service is installed or started. See contract stage 9a.

Local: 304 Rust tests in 38 suites, final focused delivery tests, warnings-denied
Clippy, formatting, workflow and generated-workflow lint, and strict/deep docs
validation (55 routes, 53 guide pages, 23 examples) passed. A first focused run
had one 3-second fake-runtime probe timeout; the complete run with four test
threads passed. CI adds real ARM64 portable build/push/pull/HTTP acceptance and
an AMD64 override. Refresh final PR checks before claiming hosted acceptance.

Remaining: combined multi-platform manifests, further advanced options/providers,
conversion fidelity, matched native release and documentation publication.
Foreign-architecture hosted/Next execution is not covered by the small ARM64
portable acceptance fixture; dispatch for all renderer/kind pairs is covered.
No manual live provider/registry deployment or native release was performed.

Follow-up be57372ec7e32c4edad9bce538ac78ec9c914d5f selects native ARM64 generated-workflow runners.
All nine workflow tests, scaffold tests, generated workflow lint, Clippy,
formatting and strict/deep docs validation passed after this refinement.


### Temporary build/image features — 2026-11-02

PR #45 merged as 42bf4c906d1ca6be64940da8aaddfb9d63f1192c after all 13 CI
jobs passed on 5c75428ca495a0afcae722a678607cafb6e4c368 (39220991457).
Refreshed on November 2: main CI 39223399959 and Base image 39223399906 both
passed. The latest public native release remains zellij-v0.1.3 (August 13).

Fixing commit aa179631e7371d9c7185a07ec1929452e2262af5 adds --features to build/b,
image static/next/build, si/ni and hosted image dockerfile. Accepted complete
selections are maths, diagrams, maths,diagrams (either order), all and none.
The override does not edit authored configuration or content. Portable builds
apply it in memory; hosted exports and image recipes apply it to the disposable
wrapper before npm install. Explicit none overrides an enabled base image.
Portable Next adapters retain the selected rendering features and install the
matching dependencies, including an explicit none for an empty selection.

A shared typed parser rejects malformed/unknown/duplicate selections before
work. Hosted feature selection requires featureSelectionVersion 1 from the
engine manifest; older engines fail clearly instead of silently mishandling
none. Previous exports survive build/compatibility failures. Portable static
Dockerfiles take prebuilt HTML and direct users to build/si for feature changes.
Preview/dev, delivery/ship and generated CI retain saved configuration or engine
image variants; this temporary flag is not implemented for those commands.
See contract stage 9b and the updated build/image/features references.

Local verification: all 310 Rust tests in 38 suites passed. After the final
engine compatibility guard, all 22 engine tests and 9 static-image tests passed;
warnings-denied workspace Clippy, formatting, workflow lint, script syntax and
strict/deep documentation validation passed (55 routes, 53 guide pages,
23 examples). Hosted CI now exercises all/none folder exports and static images,
enabling features in a hosted Next image, and refusal of an incompatible engine.
Cross-platform and actual-container results are pending for this branch; refresh
the final PR head/checks before claiming hosted acceptance or merge completion.

Remaining: combined multi-platform manifests, other advanced options/providers,
conversion fidelity, matched native releases and documentation publication.
No local Docker daemon was started, native release published or manual live
provider/registry deployment performed for this milestone.

Earlier handoff: `codex/cli-build-features` was pushed, but no PR had
been created. The GitHub connector returned 403 for PR creation (read access
works), and `gh auth status` reports no login, including outside the sandbox.
The user was asked to run `gh auth login --hostname github.com --web
--git-protocol ssh --skip-ssh-key`. Automatic approval review rejected opening
arbitrary Safari state because of unrelated private browser content; do not
retry that route. After authentication, create the PR, validate all CI jobs on
its final head, merge under the user's existing authorization, and sync main.

Authentication resolved on November 2 after the user logged in. The interactive
shell sets `XDG_CONFIG_HOME=/Users/hamid/Dev/config`; use that setting with `gh`
in noninteractive commands. The account is mylife-inc and uses the system
keyring. Do not read or print credential files/tokens. PR creation, final CI,
merge and local main synchronization are now the continuation steps.


### PR #46 dependency and schema compatibility follow-up — 2026-11-02

[PR #46](https://github.com/mylife-inc/Zellij/pull/46) is open. Its first run,
41500959209 at 5dee8187c858c3fac3cfc7f509d41cd14f8a97ce, failed npm audit:
Zod 4.4.3 is affected by [GHSA-mrhx-c7xc-m3w5](https://github.com/advisories/GHSA-mrhx-c7xc-m3w5).
Fixing commit 1c9374665c64bff29bfd76054c9d28a75f5f9134 raises the engine dependency
and lockfile to Zod 4.6.7. npm audit --audit-level=low now reports zero known
vulnerabilities as checked on November 2; this is a dated audit result.

The update exposed primitive unions emitted as JSON Schema type arrays instead
of anyOf. Section template generation now handles those arrays, and Rust field
descriptions retain their alternatives. The generated schema was refreshed;
text context and generated YAML starters remain unchanged. Regressions exercise
actual stats-band values and timeline dates, alongside every section starter.

All 365 engine tests and 311 Rust tests (38 suites), workspace typechecks,
engine packaging, warnings-denied Clippy, formatting and strict/deep docs
validation passed locally. Final CI on the updated PR head remains required
before merge; neither the initial failing run nor local checks establish hosted
acceptance. Authentication works with XDG_CONFIG_HOME=/Users/hamid/Dev/config.


### Preview/development feature selection — 2026-11-02

PR #46 merged as eea28d6053622f9d692d6b56aa4150492d9f0f21 after all 13 CI
jobs passed on e5d619687e6460bb454a06a7a722715998bc9378 (41502232990).
Both post-merge runs now passed: CI 41506317359 and Base image 41506317357.
The latest public native release is still zellij-v0.1.3 (August 13), refreshed
on November 2; source capabilities below are not a claim about that executable.

Fixing commit 6ab08a3bcee51133a5fa176d3af2eb5bfb5bd554 adds --features to view/v
and dev for both portable and hosted sites. Typed complete selections match
build/image. Overrides survive every watched rebuild, portable configuration
edit and recovery, without changing authored files. Omitting the flag on a
later invocation restores saved/default selection. Serve still serves prebuilt
output and does not accept rendering options. See contract stage 9c.

Hosted dev and preview apply the override before dependency installation, using
the existing engine compatibility guard. Hosted export/dev dependency caches
now include the canonical explicit feature selection, preventing a build with
another selection from uninstalling a live dev session's packages. Equivalent
all and maths,diagrams share a cache; omission and none remain distinct. Old
caches are not deleted. Hosted dev now distinguishes content-validation errors
from dependency/startup failures instead of reporting all failures as bad content.

Local verification: 312 Rust tests in 38 suites, warnings-denied Clippy,
formatting, workflow lint, JavaScript syntax and strict/deep docs validation
passed (55 routes, 53 guide pages, 23 examples). Actual Chromium preview/watch
acceptance covers config edits, rendering, failure preservation, recovery,
source immutability, default restoration and Ctrl-C. Hosted orchestration with
an isolated runtime fixture also passed, including feature propagation on
rebuilds and cancellation. An initial full Rust run had one existing fake-runtime
probe timeout while other checks were active; the isolated full two-thread run
passed. Do not treat this as a runtime-probe implementation fix.

CI now checks actual hosted preview math/Mermaid rendering through rebuilds,
then explicit none in a subsequent preview; hosted development checks both in
a browser before and after edits. Final cross-platform/container CI and merge
are pending for this branch. No local Docker daemon was started or native release
published. Remaining work includes delivery/CI temporary controls, advanced
build options, combined image manifests, providers beyond Netlify, conversion
fidelity, and matched native release/documentation publication.


### PR #47 development-origin regression — 2026-11-02

First CI run 41511951845 at c98e226e565da6dcd1c9b5eec263fb090fff6b6a
passed 10 jobs, including real hosted preview rendering and five native package
targets. The CLI job failed the new hosted dev Mermaid browser check. Both
macOS ARM jobs were cancelled before any step ran because GitHub reported
hosted runner capacity constraints; these cancellations are not test results.

The browser check exposed an existing hydration failure: Next binds 0.0.0.0
inside the container, but the engine excluded loopback addresses from allowed
origins. Opening 127.0.0.1 blocked the HMR connection and left client components
unhydrated. This was reproduced locally with the real Turbopack dev server.
Fixing commit b23e1d417dd93f2d708f5a4aa86c28a3b14033bf allows the exact IPv4
and IPv6 loopback hosts (127.0.0.1 and [::1]). Direct wrapper browser acceptance
now uses the container's wildcard binding and checks real math/diagram output,
editing, search, recovery and origin enforcement. Requests from unrelated
hosts, including a hostname resembling a loopback IP, still return 403.

After the fix, real local Chromium/Turbopack acceptance and all 365 engine tests
passed. Final CI on the updated PR head, including the actual container and
both macOS ARM jobs, is still required before merge. No local Docker daemon
was started. PR: https://github.com/mylife-inc/Zellij/pull/47.


### Rendering features in delivery and CI — 2026-11-02

PR #47 merged as 42641950fdcfd75d43737a1f40ecbdf64a457a48 after all 13 jobs
passed on final head 27415a15a1b8159d3fab19fdd5e18c84051cdb26 in CI run
41517322451. This includes the actual hosted-development regression and both
macOS ARM jobs. Post-merge Base image run 41520378666 passed; CI 41520378824
had 12 of 13 jobs passed with CLI still running at this checkpoint. Refresh that
run before relying on a final post-merge result. Latest public native release
remains zellij-v0.1.3 (August 13), refreshed on November 2.

Fixing commit bf78d8346b5149668d03b7773fadc73acf2b9d26 extends the shared typed
--features selection to publish/pub, deploy/netlify, ship and generated CI.
Delivery forwards the invocation-only selection to one portable/hosted static
or image build. Dry plans display it. Source, saved configuration and previous
local exports remain unchanged. Failed builds do not publish. Ship retains its
existing destination/platform settings but not this override, and explains how
to repeat the selection on a later publish after success or failure.

CI generation freezes the canonical selection in verification and publication
build commands for every supported delivery form, including named configs,
platform selections and legacy image generation. Validation and prebuilt image
push commands do not receive rendering flags. Regenerating without the flag
removes the override. Dispatch does not accept it. Existing compatible-release
pins, event gates and explicit credential provisioning are unchanged.
See contract stage 9d and updated publish/deploy/ship/ci/features references.
The deployment reference's stale claim that CI ignores saved delivery settings
was also corrected.

Local verification: all 315 Rust tests in 38 suites, warnings-denied Clippy,
formatting, script syntax and strict/deep docs passed (55 routes, 53 guide pages,
23 examples). Ten generated workflows passed actionlint. Tests execute generated
portable commands, inspect selected HTML uploaded to a loopback API, cover all
renderer/image combinations, and retain ship/config/output failure guarantees.
Hosted CI now checks features in Netlify export/upload, portable Next image
publication, and imported static/Next images through browser rendering. Final
cross-platform/container CI and merge are pending for this branch.

No local container daemon was started, native release published, real provider
site deployed or external registry image manually published. Remaining work:
advanced build options, combined multi-platform images, additional providers,
conversion fidelity and matched native release/documentation publication.


### Release candidate isolation and runner capacity — 2026-11-02

PR #48 merged as b1d636934d2be06b92e01a02b150c11dcbb3dfe9 after all 13 jobs
passed on b365ad6e97c7c524a4dd396419c3d38c53e8265f (run 41523296482).
The earlier PR #47 post-merge CI 41520378824 also passed. PR #48 post-merge
Base image 41526351348 passed, but CI 41526351538 failed: 11 jobs passed, the
CLI container group exhausted hosted runner disk during npm install in the
Next ship image, and macOS ARM native packaging was cancelled before its first
step. Neither the disk failure nor the cancellation proves a product regression;
neither counts as successful acceptance. Logs identify ENOSPC, including failure
to write the runner's own logs. Commit 7c0397c298935c0ce43fac01948d5a96e6e7bd04
reclaims unused Docker builder cache between large acceptance groups while
retaining tagged engine images. Hosted verification remains required.

Fixing commit 0e5638f3b24ca057f26465f01bb1bdbc514d62d2 centralizes release policy
in scripts/release-policy.mjs and applies it to tagged CI and local helpers:

- Candidates publish exact image tags only, become GitHub prereleases, and
  cannot update stable/latest image aliases, deploy production docs or replace
  shared installer scripts. Branch builds only update main feature variants.
- Both repository release pages now use version-pinned attached Unix/PowerShell
  installers and correct image tags without the leading v. The previous
  candidate notes invoked a stable installer and referenced nonexistent v-prefixed
  image tags. Existing published aliases are not changed retroactively.
- The emergency local publisher now uses the same policy, including --prerelease
  and --latest=false. Both release helpers reject workspace version mismatches
  before tagging/building. Candidate preflight never enables production docs.
- The release workflow still allows native publication after an independent
  image failure. Complete candidate acceptance requires checking all jobs and
  final downloaded artifacts; a release page alone is not proof of completion.

Local verification: 12 release policy tests passed, including actual publishing
shells and manifest commands with fake Docker/GitHub/Git executables; no remote
publication is possible through those fixtures. Workflow lint, Bash/Zsh syntax,
workspace version check and strict/deep documentation validation also passed
(55 routes, 53 guide pages, 23 examples). Tests run in ordinary CI and tagged
release verification. Final hosted CI and merge are pending for this branch.

See specs/zel-release-procedure.md for candidate preparation and stable promotion.
No version bump or release tag was made, no native release was published, no
manual provider/registry deployment occurred, and no local Docker daemon was
started. The intended next candidate is 0.2.0-rc.1, after reviewed versioning and
full CI. Current native source and latest public release still say 0.1.3.


Release review follow-up: c7931f250214117cf58b2955db86399866b66946 changes
zimage-push's default from latest to local and zimage-all to local feature tags,
matching zimage. Explicit raw pushes remain available; release helpers apply
channel policy. Generated release notes now show matching ZELLIJ_IMAGE settings
for both Unix and PowerShell. Fourteen release tests pass locally, including
execution of the branch manifest command and development image helpers.

PR #49: https://github.com/mylife-inc/Zellij/pull/49. Initial run 41533203275
passed all six native package/installer targets, engine and schema checks before
the final follow-up was pushed; it does not certify the updated head. Refresh
PR #49's final head/checks before merging. Read-only setup checks confirmed that
RELEASES_GITHUB_TOKEN, NETLIFY_AUTH_TOKEN and NETLIFY_SITE_ID exist by name;
DEPLOY_DOCS is unset. Secret values were neither read nor printed. Presence does
not establish token validity or destination permissions. The development repo
has no published releases; the shared repo's newest Zellij release is still
zellij-v0.1.3. No user setup action is currently needed for this milestone.


### Native 0.2.0-rc.1 preparation — 2026-11-02

PR #49 merged as d324f9ef51a6c36d47304ef765adb6725aa12bff after all 13 jobs
passed on 136a206de28a24e1bdf7232f9d9332f80abd23a5 (41534811176). Its post-merge
Base image run 41538755022 passed, confirming main-only publication. Post-merge
CI 41538755131 failed the engine watcher test: 364 of 365 engine tests passed;
the assertion read nested/new.svg after observing an earlier logo edit and could
race the later asset preparation. This is an invalid test completion signal,
not evidence of permanent asset loss. Production preparation still serializes
updates and uses the existing short directory swap; it is not a whole-site
transaction. The test now forces separate edit batches and polls the requested
new asset. Refresh the complete post-merge run for other jobs' final results.

Preparation commit 03ec06f2c2bf93aee88c629449dd3a5bc2ce36f0 sets the Rust workspace
and its six package lock entries to 0.2.0-rc.1. Bare zel ci now defaults to its
own compatible version, and its regression checks this pin plus preservation on
invalid input. Release notes in specs/releases/v0.2.0-rc.1.md describe supported
commands, platforms and remaining limits and are included in both release pages.
The public stable release remains zellij-v0.1.3 until a new publication occurs.

The release workflow and local ztest/zcheck helpers also used npx vitest at the
repository root, but the lockfile installs Vitest inside the engine workspace.
That command failed locally with vitest not found. They now use the existing
npm test workspace command, matching ordinary CI.

Published release acceptance is a new reusable/manual-tag workflow. After
publication it downloads the exact archive, per-file checksum, SHA256SUMS and
both attached installers from the development repository and (when published)
mylife-inc/releases. It checks hashes and source installer bytes, compares the
two repositories, installs through their actual public URLs, and runs the native
archive/installer/upgrade/build/serve/view acceptance on six native targets.
Windows public installation uses both PowerShell 5.1 and PowerShell 7. Installer
line endings are fixed to LF so source comparisons agree on Windows. The four
published image variants are pulled and executed on native Linux AMD64 and ARM64;
architecture, engine contract and feature selection are checked. Acceptance jobs
have read-only repository permissions and receive no publishing/cloud secrets.
They can be rerun on a published tag without republishing assets.

Local verification: 365 engine tests, 315 Rust tests in 38 suites, 25 release
policy/download tests, workspace typechecks, engine packaging, warnings-denied
Clippy, Rust formatting, workflow lint and strict/deep docs passed (55 routes,
53 guide pages, 23 examples). Download tests use fake HTTP responses and image
tests use a fake Docker executable; no local Docker daemon was started. Locked
npm ci restored dependencies and reported zero advisories at this checkpoint.

Next: final-head PR CI and merge, then tag the reviewed candidate and monitor
all release and post-publication acceptance jobs. Do not call a release page
alone complete: native publication can still succeed while image jobs fail.
No tag, native release or production website deployment has occurred in this
preparation commit. Repository visibility and the old sealed repository are
unchanged; development remains in public mylife-inc/Zellij.


### Published 0.2.0-rc.1 and download recovery — 2026-11-02

PR #50 merged as 61881e805fdf34af12757e33ca4c1703c314825e after all 13 CI jobs
passed on 7b6838154cce0eaf69788c58a63abe0d1a2e5efa (41542526979). Annotated tag
v0.2.0-rc.1 points to that merge commit. The candidate is now public in both
repositories, each with six archives, six per-file checksums, SHA256SUMS and
both installers (15 assets):

- https://github.com/mylife-inc/Zellij/releases/tag/v0.2.0-rc.1
- https://github.com/mylife-inc/releases/releases/tag/zellij-v0.2.0-rc.1

Release run 41546241822 built all six native targets, published all four exact
engine image variants for Linux AMD64/ARM64, and built the documentation.
Production Netlify deployment and shared stable-installer updates were skipped.
Both release pages are prereleases; scoped stable Zellij remains zellij-v0.1.3.
Main Base image run 41546163136 also passed. Main CI 41546163311 had 11 passing
jobs with CLI and Windows portable acceptance still running at this checkpoint;
refresh it before describing its final result.

The original release run is NOT green: 27 jobs passed, while both Windows
public-acceptance jobs failed after completing the development-repository
installation and native suite. Their next fetch, from the shared repository,
failed with ECONNRESET/UND_ERR_SOCKET. A failed-jobs retry reproduced this.
This was an acceptance downloader failure; no checksum, installer or binary
assertion failed. Do not erase these failures or move the published tag.

Fix f9c6b45 adds at most three attempts for connection errors/timeouts. Each
attempt discards partial bytes and retains size, checksum and installer/source
checks. Missing/oversized assets and integrity failures are not retried. Fix
29029e16e2caf30b1b1dbb99e43f64b95d5ee76d permits an explicit published version in
manual acceptance runs. The harness checks out refs/tags/<version> separately,
so source comparisons and native tests still use the actual release commit.

Published release acceptance 41548199714 PASSED all eight jobs using harness
29029e1 against unchanged source tag v0.2.0-rc.1 at 61881e8. All six native
targets downloaded both repositories' assets, verified byte equality/checksums,
and exercised public installation. Windows ran PowerShell 5.1 and 7; its x64
log confirms a reset was recovered on attempt 2. Both native Linux architectures
pulled and executed all four published image variants. Use this successful
follow-up run together with the original publication run as candidate evidence:
https://github.com/mylife-inc/Zellij/actions/runs/41548199714

Local verification: 29 release/download tests, workflow lint and diff checks
passed. The actual published Intel macOS archive also passed temporary
installation from both repositories, replacement/refusal/upgrade tests, portable
build, moved-output preview, serve/view and Ctrl-C. No local Docker daemon was
started, and no global zel installation or PATH was changed. The download fix
changes acceptance infrastructure; candidate archives/installers/images were
not rebuilt or replaced. The fix branch still requires final PR CI and merge.

Next: finish the workflow-fix review, then exercise candidate-generated CI and
provider delivery against explicitly selected test destinations before stable
promotion and matched production documentation. Other tracked scope remains:
advanced build options, combined multi-platform site images, additional
providers and converter fidelity. Candidate acceptance does not mean the full
product has zero bugs or that every historical feature proposal is implemented.


Candidate documentation follow-up: local installation, quickstart, CLI and
contract pages now identify the published 0.2.0-rc.1 candidate, show pinned
attached Unix/PowerShell installers and the matching engine image, and use the
available candidate in copyable CI pins. Stale statements that source still
reports 0.1.3 or Windows packages await publication were corrected. Stable
0.1.3 remains explicitly distinguished from the candidate. The current-source
sections were refreshed while dated implementation history was preserved.
Strict/deep documentation validation passed: 55 routes, 53 guide pages and
23 examples. Both release pages also link the successful acceptance run and
retain the original publication failures. Production docs remain undeployed.


### Candidate acceptance closeout — 2026-11-02

[PR #51](https://github.com/mylife-inc/Zellij/pull/51) merged as
49d9a148f40df2b747d7ce6a0f46efc5a16c3954 after final CI
[41549182222](https://github.com/mylife-inc/Zellij/actions/runs/41549182222)
passed all 13 jobs on 9ab6232758f535cf517acff263523fc5ee96353b. This includes all
six native package/installer jobs, three portable/browser platforms, engine,
schema, showcase and full container delivery acceptance. The separate candidate
source main CI 41546163311 also finished with all 13 jobs passing, superseding
the pending result in the preparation record above.

Published-artifact acceptance remains 41548199714: eight passing jobs against
unchanged v0.2.0-rc.1 source 61881e8, using download harness 29029e1. Both release
pages now link that result and preserve the original Windows download failures.
An additional local smoke used the actual published Intel macOS executable to
generate portable `zel ci` and hosted `zel new project` workflows in temporary
repositories: both pinned 0.2.0-rc.1 and its matching engine and passed actionlint.
This checked generation; live execution of those newly generated workflows and
real provider delivery at designated test destinations remain the next gate.

The candidate is available for acceptance use. Stable 0.1.3, production docs,
and the published candidate binaries/installers/images retain their prior state.
No local Docker daemon was started. The next work is live candidate delivery
acceptance and an explicit stable-promotion decision, alongside the tracked
advanced build/provider/conversion backlog. Do not infer a zero-bug or complete
historical-feature claim from these passing checks.


### Generated workflow and delivery acceptance — 2026-11-02

Refreshed PR #51 post-merge status: CI 41552573220 and Base image 41552573125
both passed on 49d9a14. Candidate v0.2.0-rc.1 remains at 61881e8; no new release
or stable promotion is part of this source change.

Fixing commit 0a79f9504b233b39ad311eb112789e4299290a88 addresses a multi-site CI
queue collision. Every generated workflow previously had the display name
Documentation and used github.workflow in its concurrency group, so distinct
site workflows on the same ref shared a queue. GitHub allows only one pending
run per group by default, allowing a new site's run to replace another pending
site run. See https://docs.github.com/en/actions/concepts/workflows-and-actions/concurrency.
The generator now names workflows by their configured filename and derives a
bounded group key from SHA-256 of that filename plus github.ref. The hash
preserves case-sensitive file identity even though GitHub group names are
case-insensitive. sha2 was already locked for the CLI; no third-party version
changed. Tests cover independent sites, stable regeneration and case-only names.
This generator fix is newer than the published candidate and needs a later
native release. Existing workflows need deliberate regeneration/review.

Six committed acceptance workflows are generated without hand-editing from
scripts/fixtures/release-delivery. They use the corrected source generator,
but install and execute the public 0.2.0-rc.1 binary and exact engine image.
They cover native portable and hosted static builds, static/Next image delivery,
and portable/hosted Netlify delivery. PR events only validate/build. Publication
requires manual dispatch on main; Netlify never receives --prod. Dedicated image
destinations are ghcr.io/mylife-inc/zellij/acceptance-static:0.2.0-rc.1 and
acceptance-next:0.2.0-rc.1. A separate manual read-only inspection workflow pulls
those packages on fresh runners and verifies HTTP pages/assets, non-root users,
health checks, registry digests and source-file isolation.

The preparation helper regenerates workflows in a temporary Git repository,
compares their exact bytes, requires independent queues and checks PR/manual
triggers plus draft-only publication. Ordinary CI checks generated-workflow
drift. The shared HTTP checker supports either public draft renderer and rejects
missing pages/styles, altered assets and exposed sources. No local Docker daemon
was started. No actual registry or Netlify publication has occurred at this
preparation checkpoint. A user question about the Netlify test destination is
pending: existing documentation-site draft previews or a separate supplied site.
Do not manually dispatch Netlify delivery until that target choice is settled.

Local verification: 317 Rust tests in 38 suites, warnings-denied Clippy, Rust
formatting, all seven acceptance workflows plus CI through actionlint, exact
workflow regeneration, and three HTTP-checker tests passed. The HTTP checker
also passed against a real native portable build/temporary server. Hosted
fixture deep/strict validation passed (3 routes, 2 guide pages), and product
docs passed (55 routes, 53 guide pages, 23 examples). Full PR CI, actual generated
GitHub execution, registry publication/pulls and selected Netlify drafts remain
pending. Preserve their eventual run IDs and distinguish source-generator
acceptance from capabilities actually shipped in the candidate binary.


### Live delivery results and Netlify account discovery — 2026-11-02

PR #53 merged as bc3955bbd14a586a9d2da8375ff6f0f8717cb95b after all 19 executed
checks passed on acbdb94: full CI 41561583999 (13 jobs) and six actual generated
workflow checks (41561584047, 41561584073, 41561584038, 41561584027,
41561584019, 41561584037). The four PR publication jobs were correctly skipped.
Post-merge Base image run 41564319833 passed; CI 41564319708 was still running
at this checkpoint and must be refreshed before stating its result.

Manual generated workflows installed the public 0.2.0-rc.1 executable and
published dedicated test packages. Static run 41564355308 passed. Next run
41564361875 initially uploaded layers but GHCR denied the manifest; attempt 2
passed with no code or permission changes. The first failure remains evidence
of an undiagnosed registry failure, not a fixed product defect.

Fresh-pull inspection initially exposed an acceptance-checker mistake:
41565212699 passed static and could not find the not-yet-published Next tag;
41565578836 passed static but probed /index.html for Next, whose adapted home
route is /. Fix d2844ab9dcf28851e838aa9464cc06642c026d4b checks the HTTP root,
adds a regression fixture with no /index.html route, and retains startup status
and bounded container logs on failure. Four local HTTP tests passed (the new
case failed before the fix). Corrected run 41565958521 passed BOTH jobs against
the unchanged published images, checking AMD64, non-root users, health checks,
actual pages/assets and source-file isolation. Registry digests:

- ghcr.io/mylife-inc/zellij/acceptance-static@sha256:831a2eaec96ffa66d23133659fecdbb104128461f7b57f67e817b9c29a923740
- ghcr.io/mylife-inc/zellij/acceptance-next@sha256:fd0cf3db3087c70e0bcfd3d9a452231f7874bd964f252de569fe36864ee6fb50

The user authorized drafts on the existing Zellij Netlify site. The official
OpenAPI contract was checked: createSiteDeploy takes a deployFiles JSON body
with draft/async fields; the client sends draft: true without --prod.
https://github.com/netlify/open-api/blob/master/swagger.yml
Portable draft run 41564369100 failed GET-site preflight with HTTP 404 BEFORE
building or creating any deployment. The user's screenshot confirmed both
secret names exist, but not their current access. A temporary repository
variable selected the known public domain zellij.shebka.net; run 41565327817
also failed preflight with 404. That temporary variable was removed and the
existing secrets were never read or changed. Hosted draft dispatch was held.
No draft URL exists and live Netlify acceptance is NOT passed.

The user then reported separate GitHub accounts (hamidbm and mylife-inc), with
the existing Netlify deployment originating from hamidbm, and proposed a new
Netlify project associated with mylife-inc. This is a migration discussion,
NOT authorization to delete the live site or move its domain yet. Preserve
existing production while comparing every published route and its information
with current local documentation; prepare a separate staging destination before
any cutover. Netlify account/token access, not GitHub repository identity alone,
determines API access. User requires no loss of published information.

Public response fingerprints for /, /cli, /guide/quickstart were identical
before and after the failed preflights (130579, 124251, 159412 bytes). SHA-256:
8e1204979df3a294646e794f378d91e02f4186e289522890a26b161b22cb47f7;
8a895d38e705e5c6f5ab502996b3c316757d3a2683b913f0a935c28dc6d01247;
fdc8a5c4d7f67e5ced91a019c9693d165919fe4bbac0852df2f6a5042a83022d.
This proves those three bodies matched, not whole-site deployment identity.
No local Docker daemon was started. Candidate assets/tag, stable channel and
production docs were not promoted. The generator fix still needs a later native
release; the checker fix requires PR CI and merge. Full content parity and the
Netlify account migration remain separate unfinished work.


### Netlify staging and published-content preservation — 2026-11-02

PR #54 merged as 35a2261f8112f05a3365c64e8659712dd6e28f64 after all 13 CI jobs
passed in 41569977319. PR #53 main CI 41564319708 also passed. Both published
acceptance images remain verified by 41565958521; Netlify drafts have not passed.

The user reports separate hamidbm/mylife-inc identities and confirmed that the
intended destination is unavailable in Netlify's transfer dialog. Proceed with
a separate staging project under the intended account, while retaining the
existing production project and zellij.shebka.net domain. Deletion, transfer,
domain cutover and production replacement remain unapproved.

Source fix 8370ec4fc43ef2af6e775096df7010b624b608d9 preserves old documentation
bookmarks and corrects repository links. A fresh public inventory found 46
pages; all exist in the 55-route local site. Thirteen normalized page bodies are
identical and 33 contain reviewed corrections/expansion/reorganization. All 29
referenced content images match source bytes. The initial comparison identified
82 lost heading IDs; 74 static aliases and eight restored headings now retain
them. Local Chromium verified all 282 original heading IDs, 46 original and nine
new routes, 29 image hashes, fragment navigation, search and current GitHub links.
Strict/deep validation (55 routes, 53 guide pages, 23 examples), static webpack
build, workflow lint and diff checks passed. Normal Engine CI now runs this
preservation check. Full PR CI/merge for this source fix remain pending.

See specs/zellij-netlify-migration-2026-11-02.md and specs/evidence for the
comparison, legacy mapping, limitations and migration gates. Local ignored
artifacts: backups/zellij-production-2026-11-02.tar.gz (public crawl, not a full
Netlify backup) and backups/zellij-staging-2026-11-02.zip (validated 55-page
static upload). The report records their hashes. A full production deploy ZIP
from Netlify is still needed before cutover; the crawl cannot discover unlinked
files or provider settings. All generated build-only source changes were restored.

The user has the staging upload and instructions to create a separate project,
keep its temporary netlify.app address, and update the existing GitHub Netlify
secrets for it. Await the new URL and readiness before running draft acceptance
there. No credentials were read, no local Docker daemon started, and no existing
production content replaced. Native candidate/stable channels are unchanged;
the multi-site workflow fix still requires a later native candidate.

### Preservation acceptance and staging handoff — 2026-11-02

[PR #55](https://github.com/mylife-inc/Zellij/pull/55) merged as
`f1ca1ff04f049366a1175b4dfe2bd6e78a6c34a1` after all 13 jobs passed in
[41573372928](https://github.com/mylife-inc/Zellij/actions/runs/41573372928)
on final head `27947739e2bfdfa126f323aca847e970627c71bc`. The new Engine gate
built the docs with preserved authored images and passed route, legacy-anchor,
asset and browser/search acceptance. Six native package targets, three portable
platforms, full container delivery, schema and showcase checks also passed.
This resolves the pending PR result above. PR #54 main CI `41572436263` passed.
PR #55 post-merge CI `41576036593` and Base image `41576036573` were still
running at this closeout; refresh them before relying on their results.

Read-only DNS inspection found `zellij.shebka.net` pointing to
`zellij-docs.netlify.app`; the domain's nameservers are `ns1.hostmonster.com`
and `ns2.hostmonster.com`. No DNS setting changed. The user has the validated
staging ZIP and instructions; the new temporary URL and confirmation that its
GitHub secrets are ready remain pending. A temporary remote checker is prepared
to compare all 55 served HTML bodies with the validated artifact, then check
routes, anchors, images and search. Only its syntax has been checked; remote
preservation and both Netlify CLI draft deployments remain unverified.

Next: verify the selected staging URL, run portable and hosted draft acceptance,
retain the old project's complete deploy ZIP/settings, and review the full
staging docs before any separately approved production/domain cutover. The
original live project stays available. No Netlify credentials, production
content, candidate release assets or stable channel changed in this closeout.


### Live Netlify draft acceptance and staging browser finding — 2026-11-02

The selected project is https://zel-stage.netlify.app. The user made its main
site and previews public and updated both Netlify GitHub secrets. Portable run
41587034632 and hosted run 41587142130 passed using public candidate 0.2.0-rc.1
on main 3114219. Both draft URLs passed HTTP pages/assets/source-isolation
checks and browser search/navigation with no captured console errors.

The full documentation preserves all 55 authored routes and 29 original images;
all 282 original heading IDs and 82 compatibility targets were checked. Search
and all nine added pages work. Raw HTML includes two Netlify additions, which
are explicitly distinguished from authored content. Headless checks encountered
Netlify's connection challenge; ordinary browser checks succeeded functionally
but exposed repeated React #418 hydration errors on the full documentation.

A local /cli/completions control is clean; exact Netlify HTML and its injected
hosting comment alone reproduce #418, while its badge script alone is clean.
The user was asked to turn off the staging badge to test whether this removes
the comment. The last read still found both additions, so resolution is pending.
Do not claim a clean full-documentation browser run. Details, draft URLs and
limits are in the migration report, latest audit entry and
specs/evidence/zellij-netlify-acceptance-2026-11-02.json. Detailed local evidence
is retained under ignored backups/zellij-netlify-acceptance-2026-11-02.

Before/after fingerprints matched for three routes on each public main site;
production/domain were not changed. Old Netlify full-deploy backup/settings and
a separate cutover decision remain outstanding. No secret values were read and
no local Docker was started. Candidate/stable channels remain unchanged; the
workflow queue fix still needs a later native candidate. Earlier post-PR55 runs
41576036593/41576036573 were cancelled, superseded by successful main CI
41576279817 and Base image 41576279748 on PR56 merge 3114219.


### Badge-off result and whitespace isolation — 2026-11-02

The user disabled Powered by Netlify badge. Fresh HTTP checks on `/`,
`/cli/completions` and `/guide/quickstart/new-project` returned 200 with the
badge script absent but the hosting comment/newline still present. Removing
only that observed block makes each response identical to the validated upload.
A fresh normal-browser visit to `/cli/completions` still emitted React #418.

Further controlled tests isolate the leading LF: newline-only before viewport
metadata reproduces #418; the same Netlify comment without its leading LF is
clean. The saved loopback reproduction also verified clean original HTML,
one error with a newline, and no error with a minimal comment. Hydration was
confirmed via the mounted theme control. A development build reproduces the
mismatch at `<style id="zellij-theme">`; installed React DOM's hydration scan
stops at the extra text node. This refines the earlier comment-level finding;
the badge setting did not fix it.

See specs/zellij-netlify-hydration-2026-11-02.md for the ready-to-send Netlify
support request and specs/evidence/serve-head-whitespace-repro.mjs for the
read-only reproduction. The report has not been sent. No engine workaround or
warning suppression was added. The preservation checker currently observes
uncaught page errors, not recoverable errors logged through console.error;
extend that coverage when implementing the remedy. Full-docs remote hydration
acceptance remains open. No production/domain/release/secret changes occurred.
Temporary development servers were stopped and their generated source changes
were removed. The source branch started from PR #57 merge df07f28.


### Browser acceptance hardening — 2026-11-02

The user reports filing the hosting bug with Netlify and emailing support.
Development continues while that external issue is unresolved. The preservation
checker now captures console.error as well as uncaught page errors, records the
page URL, and waits for the docs theme control's mounted label on every full
navigation. It fails before continuing when a page logs an error. Integration
regressions run the actual checker against temporary copies of the built docs:
one with the observed head newline and one with an uncaught exception. Normal
preservation and both negative controls run in Engine CI. Syntax checks,
actionlint and diff checks pass locally; hosted verification is pending.
No product rendering workaround, production deploy or release was made here.


PR #59 first run 41594298255 passed full-docs preservation with the new error
capture and hydration wait, but its negative hydration test assumed that React
always uses console.error. Hosted Chromium reported it through pageerror.
The test now accepts either channel for React #418; a third control modifies
only an existing JavaScript asset to emit a console-only sentinel. That control
specifically guards the old console-capture gap without changing the HTML tree.
The uncaught-exception control passed on the first run. Final CI is pending.


### Candidate 0.2.0-rc.2 preparation — 2026-11-02

The user authorized continuing with browser acceptance and the next candidate.
Workspace version and only the six local package versions in Cargo.lock are
0.2.0-rc.2; external dependencies are unchanged. Reviewed candidate notes list
the per-workflow queue fix, public-download recovery, preservation corrections,
and strengthened hydration acceptance. The hosting newline issue remains open
with Netlify and is explicitly not claimed fixed. rc.1 assets are immutable;
normal installation documentation continues to target the published candidate.

Local release/download tests: 29 passed. Native CLI unit/workflow tests: 26 unit
and 11 integration tests passed; one real-container test remains intentionally
ignored locally and is a CI gate. The first sandboxed unit run failed seven
local API fixtures because loopback binding was denied; the identical rerun
with loopback permission passed. Version preflight confirms v0.2.0-rc.2.
Final-head CI, review/merge and publication remain pending; no rc.2 tag exists
and stable/production channels have not changed.


### Browser acceptance complete; rc.2 prepared — 2026-11-02

PR #59 merged as 93dca084a018f7ff56e6259441beca4a82874c32. Source commits
68db620 and 97f9f9f capture both browser error channels and wait for mounted
controls. All 13 checks passed on final head 97f9f9f in run 41594855467. Engine
verified all 55 routes, 282 original headings, 82 compatibility mappings, 29
original image hashes and search with no captured browser errors after
hydration. All three negative controls passed: head newline/React #418,
uncaught exception, and a console-only error in an existing JavaScript asset.
This resolves the acceptance coverage gap, not Netlify's HTML transformation.
The first run 41594298255 was superseded/cancelled after its Engine negative
test failed on an overly specific reporting-channel expectation; retain that
failure separately from the passing final run.

PR #60 merged as 0fbf0a12c3cf7074c4620e2bb84cc5d8e5fd7469. All 13 checks
passed on final head 04653b0 in run 41595462484. The merge and checked head both
have tree 4353ca74faba1aad6f6d8e9a4444ad1aa95cf2d0. Version preparation is in
249d0b1, with final scope/verification notes in 04653b0. The reviewed release
notes are specs/releases/v0.2.0-rc.2.md. All six archive/installer platforms,
three portable platforms, engine, schema, real containers and showcase passed.

Automatic approval review rejected executing `zrelease v0.2.0-rc.2`: creation
and publication of a public tag/assets requires explicit user approval beyond
preparation/validation. The command did not execute. No tag, release assets,
versioned images, stable promotion or production deployment was created by this
attempt. Local non-publishing `zcheck` was started separately; its final result
is recorded in the follow-up below. The source remains ready for a specifically
authorized prerelease from 0fbf0a1. After approval, refresh checks and publish to
mylife-inc/Zellij and mylife-inc/releases with matching exact GHCR tags, then
verify the downloaded public artifacts and released delivery workflows.

The user reports filing the Netlify bug and emailing support@netlify.com; a
provider response remains pending. Preserve the old project/domain and obtain
the full original deploy backup before any separately approved cutover.


#### Final local validation — 2026-11-02

The first full local `zcheck` passed typechecking, 365 engine tests, schema
consistency, Rust formatting and warnings-denied Clippy, then failed four
`static_images` mock-client probes at the three-second deadline. Repeating that
suite yielded two timeouts; one affected test passed in isolation. A separate
experiment, without Zellij, launched 12 new exit-zero shell scripts concurrently:
five exceeded three seconds and the slowest took 4.841 seconds. Sequential
re-execution took 0.012–0.022 seconds. This establishes startup latency outside
the product; the OS-level cause was not diagnosed.

The entire `zcheck` was then rerun with `RUST_TEST_THREADS=1`: exit 0, all
365 engine tests and 317 Rust tests passed, with one intentionally ignored
real-container test covered by CI. All ten mock-image tests passed. Product deadlines and CI
parallelism are unchanged, and no local container daemon was started. Default
parallel local execution remains a documented environment limitation, not a
claimed product fix. Raw logs are retained under ignored
backups/zellij-rc2-preparation-2026-11-02/ with hashes in the evidence JSON.
Publication still requires the specific user approval described above.

At this record, post-merge Base image run 41598251612 passed. Main CI rerun
41598251651 has 11 successful jobs, with CLI and Windows portable checks still
in progress; it is not recorded as passed. Both final PR runs already passed
all 13 jobs. Refresh the main run before publication.


### Hosted development auto-reload — 2026-11-03

The user asked to continue implementing the remaining CLI capabilities. Hosted
`dev` and updated editable wrappers now publish an opaque development revision
only after successful search/asset preparation. Development layouts poll that
revision and reload the current URL; image-only edits are included. Invalid
edits do not announce a reload. The development runtime cache is keyed by the
revision so a reload inside the normal 250 ms memo window cannot reuse the old
revision. Production layouts omit the poller. Watcher shutdown removes its
signal; one-shot preparation removes a signal left by an interrupted process.

Focused regressions and all 369 engine tests pass, as do workspace typechecks,
the packaged engine build, installed-CLI watcher/child lifecycle acceptance,
actionlint, syntax/diff checks, and deep/strict validation of all 55 docs routes
and 23 examples. Browser acceptance now requires automatic content and image-only
reloads, no reload on invalid edits, recovery and removed navigation entries,
and no production polling. Final hosted CI and fixing commit remain pending.
No local container daemon was started and no release was published.

This extends the unpublished rc.2 candidate notes. The earlier proposed release
commit 0fbf0a1 predates this work; any later release proposal must use a reviewed
commit containing its final advertised scope. Publication approval is still
pending. Remaining CLI work includes hosted JSON validation, advanced options,
combined image manifests, additional deployment/CI providers and converters.


### Hosted JSON validation — 2026-11-03

Hosted `zel check --json` now consumes a versioned engine report. It preserves
real content/deep findings and warning severity, including strict warning-only
failures. Native success is exit 0, content/strict failure 3, unavailable runtime
or inaccessible mount 4, configuration conflict 2, and incompatible/failed engine
report 1. Normal JSON mode writes one stdout report and no stderr diagnostics
after site/checker selection; explicit verbose diagnostics still use stderr.
The Node check command emits the same validation schema but retains exit 1 for
failed validation. Hosted `pages` counts loaded routes, not compiled HTML files.

The protocol validates version, target, mounted root, requested options, status
and findings before accepting success. The structured host root preserves
quotes, backslashes and Unicode; diagnostic messages stay verbatim so authored
URLs cannot be mistaken for container paths. Older engine
images fail with image-selection/pull guidance rather than parsing terminal
text. Runtime/configuration failures also receive structured reports. Content
mounts remain read-only. Invalid CLI flags and discovery failures before the
checker is selected retain ordinary CLI errors.

Regressions cover engine findings, strict warnings, bootstrap failures, malformed
or inconsistent reports, native option forwarding, host path escaping and
missing runtimes. The installed-package/real-container acceptance script now
exercises JSON success, deep failure, strict warnings and recovery.

Implementation commits: `9a3d740`, `06e9d1e`, `9d89dd5`; review and hosted
acceptance are tracked in [PR #63](https://github.com/mylife-inc/Zellij/pull/63).
Local verification passed: 321 Rust tests (one real-container test intentionally
ignored locally), warnings-denied Clippy, formatting, 372 engine tests,
workspace typechecks, engine package build and installed JSON checks. All 55
documentation routes and 23 examples pass deep/strict validation; actionlint,
JavaScript syntax and diff checks pass. Rust fixtures used serial execution and
loopback access for mock HTTP services. Two obsolete tests expecting hosted
JSON rejection were updated to assert its structured runtime failures.

The dependent reload implementation is `3ddd2cd` plus image-cache correction
`8d3d4f1`, tracked in [PR #62](https://github.com/mylife-inc/Zellij/pull/62).
Both Chromium and WebKit passed automatic content/image reload, invalid-edit
preservation and recovery in job 126401231393 of run 41777437141. Existing
showcase browser coverage also passed 317 tests. Current full platform/container
results remain available in the PR checks; refresh them before relying on a
release source. Neither implementation publishes a release or deploys production.

Remaining CLI scope includes additional deployment and CI providers, combined
multi-platform image manifests/build arguments, broader import fidelity,
hosted dev host/browser options, and advanced build overrides. The Netlify
hosting-injected hydration failure remains a separate provider issue.


### Development host and browser options — 2026-11-03

Implementation `222ffda` adds `zel dev --host`, `--open` and
`--browser <executable>` for portable and hosted sites. Defaults remain portable
loopback (`127.0.0.1`) and hosted IPv4 wildcard (`0.0.0.0`). `localhost` and
literal IPv4/IPv6 addresses are accepted; wildcard browser URLs use loopback.
Portable preview validates request hosts against the connection's local address,
including dual-stack address normalization, while retaining path/source guards.

Hosted development owns a uniquely named container. Successful engine preparation
publishes its session and current `basePath` with the development revision. The
CLI confirms both container metadata and the matching HTTP response before
announcing/opening the URL, once per session. Failed startup, occupied ports and
unrelated listeners cannot cause a browser launch. Current engine metadata and
a reachable published port are required; after five minutes without confirmation,
manual-opening guidance is printed and development continues. Ctrl-C uses bounded
runtime stop/removal, reaps the foreground client and retains the dependency cache.
Quiet cancellation does not replay runtime logs as an unexpected failure.

Local verification: 322 Rust tests passed (one real-container test intentionally
ignored), warnings-denied Clippy, formatting, 374 engine tests, workspace
typechecks and package build. All 55 documentation routes and 23 examples pass
deep/strict checks. Native fake-runtime/browser acceptance passed IPv4/IPv6,
readiness delay, prefix handling, one-time launch, wrong-session rejection,
startup/port/browser failures, quiet mode and cancellation during startup/serving.
No local Docker or GUI browser was used. The new native acceptance runs on all
three CI operating systems; real-container/browser coverage now checks explicit
loopback publishing, prefixed browser opening and container/port cleanup. CI
results must be checked before merging; this entry records local evidence only.

Prior milestones are now merged: PR #62 as `e6a83bbc7fff5ed383d146a04fa2b5c25d83bb29`
with all 13 checks passing in run `41780233137` on `caf5e3c`; PR #63 as
`a3a1571a2a479b8d8dd679a30b31a23b1c3afe21` with all 13 passing in `41780276435`
on `09d1b65`. Each merge tree matches its tested head. Main `a3a1571` CI
`41782982430` and Base image `41782982399` also passed. Earlier failed runs remain
part of the dated history above.

Remaining CLI scope: advanced build overrides, image build arguments and combined
multi-platform manifests, additional deployment/CI providers, and broader import
coverage/fidelity. The Netlify hosting-injected hydration issue remains external.
No release publication, production deployment or domain change is included.


### Development options — acceptance follow-up, 2026-11-04

PR #64's core implementation is `222ffda`. Follow-up `d0eae61` releases the
session's temporary identity before the inaccessible-mount diagnostic exits;
native regressions verify cleanup for invalid content and inaccessible mounts.
Run `41795843334` on `d0eae61` passed 12 of 13 jobs, including all three native
development-option suites and six native package/install jobs. Its CLI job
failed before starting a container: the new browser fixture had made the app
directory nonempty before scaffold initialization. This was an acceptance setup
error, not evidence that container startup or cleanup had passed.

Harness correction `3b95629` initializes the app first, waits for shutdown output
to drain with a bounded deadline, and runs real development acceptance before
long image-publication checks. The scaffold/launcher setup, JavaScript syntax,
actionlint and diff checks pass locally without Docker or a GUI browser. Final
container results are tracked in [PR #64](https://github.com/mylife-inc/Zellij/pull/64).
Earlier run `41794505393` was canceled after the startup cleanup correction;
it did not establish final acceptance. No release or production deployment was made.


### Development options — final acceptance, 2026-11-04

PR #64 merged as `706f5430772a13ba9428aeef0d60f3237fd2dd4f` after all
13 checks passed in run `41798421655` on `bae81ce2cdc63616ed20c8b8c39b71dd9653b49b`.
The tested head and merge have identical trees. Refreshed main CI `41801848062`
and Base image `41801847816` both passed. This supersedes the pending acceptance
status above while retaining its earlier failure/correction history.

### Hosted build overrides — local implementation, 2026-11-04

The current branch adds invocation-only `--base-path`, `--site-url` and
`--no-check` to hosted `build`/`b`, static/Next image commands (`si`/`ni`), legacy
Next image builds and hosted Dockerfile generation (`idf`). Portable Markdown
adapted into a Next image accepts them; pure portable HTML/static images reject
them before runtime probing. Deploy/publication/CI generation do not yet accept
these three flags. Existing delivery callers retain default behavior.

An explicit prefix overrides the authored/wrapper prefix; `/` clears it. The
public site URL is an HTTP(S) origin only, with the path supplied separately.
Both reach rendering, SEO metadata, image layout and health checks. Next images
retain settings at runtime, including unknown-route content loads. Authored
configuration is validated before applying overrides and is never rewritten.
`--no-check` removes only the separate preflight; preparation, schema, rendering
and output ownership validation remain active. Failed exports preserve the last
successful site.

A typed Rust settings value crosses the CLI/engine boundary as versioned,
hex-encoded JSON in `ZELLIJ_BUILD_OVERRIDES`. User strings never become shell or
Dockerfile instructions. Engine capability `buildOverridesVersion: 1` is checked
before installation; older images fail with upgrade guidance. The shared Node
bootstrap validates/normalizes the protocol and supplies both Next configuration
and content sources. The production runtime cache keys include these settings.

Implementation: `62ec4f1`. Local verification passed: 323 Rust tests (one real
container test intentionally ignored locally), warnings-denied Clippy, formatting,
415 engine tests, workspace typechecks, package build and isolated installed
checker acceptance. All 55 documentation routes and 23 examples pass deep/strict
validation. Native fake-runtime acceptance covers aliases, generated recipes,
root reset, preflight selection, invalid arguments, portable rejection and source/
output preservation. JavaScript syntax, actionlint and diff checks pass. Rust
fixtures ran serially with loopback access for their existing mock services.
No local Docker or GUI browser was used. CI adds the native fixture on all three
operating systems plus real export/static-image/Next-image URL and health checks;
final platform/container results are pending.
Remaining CLI scope includes image build arguments and combined multi-platform
manifests, propagation of these options to delivery/CI where appropriate,
additional deployment/CI providers and broader import fidelity. Netlify's
hosting-injected hydration problem remains external. No release publication,
production deployment or domain change is included.


### Hosted build overrides — first CI follow-up, 2026-11-04

[PR #65](https://github.com/mylife-inc/Zellij/pull/65) tracks implementation
`62ec4f1` and context/probe regression `23d471e`. In run `41814544611`, the real
hosted override export checks passed, including canonical/sitemap/robots URLs,
root reset, invalid-content preservation and old-engine rejection. Engine,
showcase and all six native package/install jobs passed. The Windows portable
job failed earlier in the existing development-options fixture with an HTTP
`ECONNRESET`, before reaching the new override fixture.

Correction `fc3397f`: the fake development server used a single socket read, which could respond and
close before a complete request arrived. The correction explicitly sets accepted
sockets to blocking mode (Windows can inherit listener mode), consumes bounded
complete headers, adds a deliberately fragmented request regression, and labels
HTTP failures. It adds no blanket retry and does not change production server
behavior. The superseding CI run must verify Windows and full image acceptance;
this partial run is not claimed as complete acceptance.

The corrected native development suite, including fragmented headers, passes
locally with fake browser executables and loopback sockets. JavaScript syntax,
actionlint and diff checks pass. The new build-override fixture now runs before
longer native preview fixtures in each platform job.


### Hosted build overrides — final acceptance, 2026-11-04

PR #65 merged as `58623ff683e7597471607c59825966dec44d158e` after all 13
checks passed in run `41815732030` on `a92b80c59bee232773267c20dc07ea98015e42d0`.
The merge and tested head have identical trees. This includes Windows acceptance
after fixture correction `fc3397f`, real hosted exports, both image kinds and
browser/health checks. Earlier failed/canceled evidence remains above.
Main Base image run `41819057643` passed; main CI `41819057870` was still running
when refreshed for this next milestone. Native rc.1 remains the latest published
release; no rc.2 publication or production/domain change was performed.

### Build overrides in delivery and generated CI — 2026-11-04

Implementation `c20d48b` extends `--base-path`, `--site-url` and `--no-check` to
`netlify`, `deploy`, `publish`/`pub`, `ship`, and GitHub workflow generation.
The same typed options reach the single hosted export or image build before
upload/push. Pure portable static delivery rejects them before runtime or remote
work; portable-to-Next publication accepts them. Dry runs show normalized values,
including `/` for an explicitly cleared prefix, and the skipped content preflight.
Netlify credentials/site access still precede the build. Failed builds never
create a deployment or push. Temporary exports, authored files and prior local
output retain their existing protections.

`ship` keeps its converted site and saved delivery destination after failure,
but never saves invocation-only build settings. Both success and failure guidance
say to repeat the applicable overrides. Bare saved `deploy` uses discovery or
`--site-root`; its first positional remains the explicit provider selector.

Generated workflows serialize URL settings as environment values and pass quoted
arguments only to build/delivery commands. GitHub expression syntax in values is
rejected. `/` retains its root-reset meaning. `--no-check` removes the separate
workflow deep/strict check and skips the build's preflight; preparation/schema/
rendering, credential/runtime checks and publication event gates remain active.
Without it, the workflow validates authored content as before. The generator
requires version 0.2.0-rc.2 or later when these flags are used; it still does not
publish or verify release availability. Omission on regeneration restores default
checks/settings. Existing generated release-acceptance workflows are byte-identical.

Local validation passed: 325 Rust tests (one real-container test intentionally
ignored locally), warnings-denied Clippy, formatting, all 12 workflow tests and
11 parser tests after the command enum allocation correction. Native fake-runtime
coverage verifies publication aliases, no push on failed builds, imported-site
retention, dry plans, root reset, portable rejection and unchanged source. A Bash
replay verifies shell-looking prefix values remain literal; workflow failures
preserve the previous file. Mock Netlify tests retain authentication preflight.
Generated workflows pass actionlint with and without preflight; all 55 docs
routes and 23 examples pass deep/strict validation. Script syntax/diff checks pass.
No local Docker or GUI browser was used.

CI now exercises prefixed imported static and Next images through push/pull,
browser navigation, canonical/sitemap metadata and health checks. Hosted Netlify
acceptance uses an overridden deployment prefix/origin and verifies invalid
content under `--no-check` cannot create a deployment. Final platform/container
results must be refreshed from the pull request before merging.

Remaining CLI work includes image build arguments, combined multi-platform image
publication, additional deployment/CI providers and broader import fidelity.
The Netlify hosting-injected hydration issue remains external. Candidate release
publication and production migration remain separate, pending decisions.


### Delivery overrides — CI harness correction, 2026-11-04

[PR #66](https://github.com/mylife-inc/Zellij/pull/66) tracks `c20d48b` and
context record `d634507`. Run `41821800581` failed the macOS workflow test because
its system Bash cannot execute the existing `${REPOSITORY,,}` expression in the
Ubuntu-only legacy image job. Correction `0c1802a` runs full workflow-shell replay
on Linux, matching the generated runner. All operating systems still check the
workflow data, option placement, release compatibility, rejection/preservation,
and native delivery forwarding. No product behavior or generated workflow changed.
The focused regression, formatting and diff checks pass locally after correction.
Final CI acceptance is pending on the replacement head; the first run is not
claimed as complete acceptance.


### Delivery overrides — final acceptance and main follow-up, 2026-11-04

PR #66 merged as `b7a076af0a72302f9c44f66e295ae9b208983ed8` after all 13
checks passed in run `41822262323` on `c3b94705fdb096ce5f699e9c30172b414aa937c4`.
The tested head and merge have identical trees. The changed Netlify export/upload
and static/Next ship image tests passed, as did Linux/macOS/Windows acceptance.
The earlier macOS harness failure remains recorded above.

Main Base image run `41826282353` passed. Main CI `41826282286` passed 12 jobs;
Windows ARM packaging compiled Rust successfully, then PowerShell crashed with
`Internal CLR error (0x80131506)` while creating the ZIP. Only that failed job
was rerun; attempt 2 was queued when this entry was written. This is not claimed
as a source fix or a fully successful main run. Native rc.1 remains the latest
published release; no rc.2 tag/release or production/domain change occurred.

### Image build arguments — 2026-11-04

Implementation `86b6d2d` adds repeatable explicit `--build-arg NAME=VALUE` to
static/Next image commands and aliases, legacy image builds, printed Dockerfiles,
image publication, image `ship`, and generated GitHub image workflows. Values
are typed argv data. Names are ASCII identifiers; values allow Unicode, spaces,
additional equals signs and literal shell syntax, but no control characters.
Empty values are explicit and repeated names use the last value. Environment
lookup is not implicit. Zellij-owned settings, historical FEATURES and automatic
platform arguments are reserved for their dedicated flags.

Recipes declare names only, before builder commands. Hosted final runtime stages
do not redeclare arguments or receive added ENV settings. Portable static images
apply them only during container packaging, after native Markdown compilation.
Proxy/BUILDKIT arguments are forwarded without extra declarations. Printed
recipes require the values again when built. Build arguments are not secrets:
builders can retain them in metadata, and build scripts can bake them into files.
Authored content and saved configuration are unchanged. Dry plans omit values;
ship success/failure guidance explains that arguments must be repeated.

Static-folder builds, Netlify delivery and Netlify ship reject image arguments
before output, credentials or remote requests. Image build failures do not push.
Generated workflows pass numbered environment assignments as quoted arguments
only to build/publication commands, never checks or prebuilt pushes. GitHub
expressions are rejected before writing; older CLI pins are rejected (minimum
0.2.0-rc.2, still unpublished). Regeneration without arguments removes them.

Local verification: 328 Rust tests passed, one real-container test intentionally
ignored locally; warnings-denied Clippy and formatting passed. Native fake clients
verify Docker/Podman/nerdctl argv, aliases, portable/hosted recipes, publication,
ship retention, omission/reset, dry plans, early rejection and unchanged source.
Existing override acceptance and all six generated release workflow comparisons
pass. Generated static/Next/legacy image workflows pass actionlint with combined
build arguments, features, URL overrides and ARM64 selection. All 55 docs routes
and 23 examples pass deep/strict checks; script syntax and diff checks pass.

CI adds the native fixture on all three operating systems. Real static/Next ship
acceptance uses a custom builder script that consumes an exact literal argument
and emits a public JSON file, then verifies the pulled artifact over HTTP and
checks that argument names are absent from runtime ENV. No local Docker or GUI
browser was used. Final CI acceptance must be refreshed before merging.
Remaining work includes combined multi-platform image publication, additional
providers and broader import fidelity. Netlify's hosting-injected hydration issue
and the separate release/production-migration decisions remain pending.


### Image arguments — first CI correction, 2026-11-04

[PR #67](https://github.com/mylife-inc/Zellij/pull/67) tracks implementation
`86b6d2d` and context `bcb0401`. Run `41880580238` failed the new Windows native
fixture during the fake Docker info probe, before building. Rust on Windows
searches the executable and system directories before inherited PATH; setting
only the fixture PATH does not isolate an installed system Docker. Correction
`45f44bf` copies zel beside its fake clients and verifies each selected probe
writes the fixture marker. Production runtime discovery is unchanged. Corrected
native acceptance, JavaScript syntax and diff checks pass locally; final Windows
and real-container acceptance remain pending on the replacement head.

The earlier main CI `41826282286` now passed on attempt 2: Windows ARM packaging
job `126744911367` succeeded without source changes after the isolated PowerShell
CLR crash. This is retry recovery, not a diagnosed product fix. No release was
published and no production site changed.


### Combined image publication and smaller runtimes — 2026-11-04

The user authorized the next CLI milestone and requested smaller images.
Implementation `a29a792121ce0cb3fa6ba027e243173d6e0a40e7` adds `--platform all`
(and either ordering of the explicit AMD64/ARM64 comma list) to registry
publication and ship. `image.platform: all` saves the default. Single-platform
local commands remain compatible; combined selection on a local-only image
command fails before runtime discovery with publication/override guidance.
Netlify still rejects explicit image platform flags.

Both renderers and static/Next image kinds share the same recipe/argument paths.
Docker uses the caller's selected Buildx builder and direct registry output;
Podman uses an invocation-owned lowercase manifest, pushes all architectures,
and cleans the manifest after success/failure; nerdctl uses its image exporter.
No builder, emulator or daemon is installed/started. Combined publication does
not promise a local image; failure guidance acknowledges possible remote layers
and recommends inspecting the registry before retrying. Ship retains its
converted site and saved architecture choice after delivery failure.

Generated combined CI verifies both architectures without credentials on PRs.
Publication events use native AMD64/ARM64 runners, unique per-run/attempt staging
tags, and a dependent final job joining immutable digests only after both jobs
succeed. Staging tags remain for registry retention policies. Existing event
restrictions, permissions, explicit configuration selectors and build overrides
are retained. Minimum CLI pin is 0.2.0-rc.2 (still unpublished); single-platform
workflows, including all six release acceptance files, are unchanged.

Static images use upstream NGINX Alpine slim without optional dynamic modules.
Next images trim npm, Yarn and Node headers in a separate stage and copy that
filesystem into a fresh runtime, so removed tools do not remain in inherited
layers. Node's matching Alpine libraries, certificates, licenses and shell are
kept, alongside traced app dependencies, public assets and runtime content.
New CI jobs build/run both image kinds on both native architectures, replay the
generated publication/manifest commands against loopback registries, and compare
uncompressed image size against the old runtime bases with identical site data.
The existing container job also tests direct combined Buildx static publication,
both architecture pulls/HTTP serving, and the existing Next browser suite.

Local verification: 329 Rust tests passed, one real-container test ignored;
warnings-denied Clippy, formatting, isolated native runtime acceptance, existing
image-argument acceptance, all six generated release-workflow comparisons, and
12 combined generated workflows through actionlint passed. Docs deep/strict
checks passed all 55 routes and 23 examples. An initial parallel Rust run hit
three known local fake-shell info-probe timeouts; the full serial run passed.
No local Docker or browser was used. Actual image sizes and final CI acceptance
remain pending for this branch; do not claim measured savings or a merge yet.
Remaining CLI areas include additional providers and broader import fidelity;
release publication and Netlify production migration remain separate decisions.

Prior milestone evidence: PR #67 merged as `557d8b6` after all 13 checks passed
in run `41882538968` attempt 2 on `ba37a8a`; merge/head trees match. The first
attempt's existing Docker info timeout recovered on retry without a code change.
Main base-image run `41889745285` passed; main CI `41889745157` was still running
at this entry. The Netlify hosting-comment hydration issue remains external.


### Combined publication — first CI fixture correction, 2026-11-04

[PR #68](https://github.com/mylife-inc/Zellij/pull/68) tracks this milestone.
Compatibility correction `c215266` preserves Node's original container entrypoint
in the trimmed runtime; all 25 engine tests passed afterward. Run `41893692905`
on `ec35452` built/pushed/pulled static and Next images on both native Linux
architectures. Static serving and size comparison passed. The new Next acceptance
fixture then requested `/index.html`, but the portable-to-hosted adapter maps the
home page to `/` (confirmed in source and the generated route table). The server
started successfully; the fixture's incorrect route caused both failures.
The fixture now requests the renderer's actual home and reports HTTP status on
failure. Full container acceptance, final size figures and merge remain pending.


### Combined publication — native acceptance and size accounting, 2026-11-04

Run `41894634123` on `eec6ca1` passed both native image jobs and the dependent
combined-manifest job (`126803607170`, `126803607619`, `126805836087`). Both static
and Next images build, push, pull, serve both documents, deny Markdown-source
access and pass health checks on AMD64 and ARM64. The generated final command
joins the exact platform digests. Node/npm/Yarn/header checks also pass.

The first size report used Docker's Size field and called it uncompressed;
that label is not portable between classic and containerd image stores. The
runner-local comparison showed reductions, but raw AMD64 and ARM64 counts must
not be compared directly. Acceptance now also pushes the identical-payload
baseline to the disposable loopback registry and sums compressed layer sizes
from its platform manifest. This supplies consistent download-size comparisons
and keeps Docker-reported sizes explicitly labeled. Final compressed figures
and full final-head CI remain pending. No live external registry was used.

Prior main CI `41889745157` is now successful. Superseded PR run `41893692905`
required force-cancellation after its macOS preview job did not stop following a
normal cancellation request; this unblocked the corrected run, not a product fix.


### Combined publication — final runtime footprint pass, 2026-11-04

Read-only inspection of the tested AMD64 image archive confirmed that most Next
runtime space is Node itself and traced app dependencies, with negligible source
maps. It also exposed retained Alpine package-index caches. The final recipes
remove those caches. Static packaging additionally removes the unused alternate
NGINX debug executable and copies the prepared filesystem into a fresh stage so
upstream deleted/cache layers do not inflate downloads. NGINX's entrypoint,
non-root user and SIGQUIT shutdown signal are explicitly retained. Node's
entrypoint remains intact. Required runtime libraries and traced app dependencies
are not pruned. Native CI verifies cache absence, NGINX startup metadata and the
existing serving/health behavior, and measures compressed registry layers against
the former runtime bases. No local container or browser was started.


### Import heading anchors and completed image milestone — 2026-11-04

PR #68 merged as `e7b0d7987787ee4e0856bb3d3edb280eaec0c9ae` after all
16 checks passed in run `41897666269` on `f5c0a5c`; merge and tested trees match.
Both native image jobs and the generated combined-manifest job passed, as did
direct combined Buildx publication, native package/install checks on six targets,
portable acceptance on three OS families, Next browser acceptance and the full
CLI/container suite. Base image post-merge run `41902611084` passed; main CI
`41902611421` had 14 of 16 successful jobs when this record was written.
The final compressed-layer measurements are in
`specs/evidence/zellij-image-publication-2026-11-04.json`: static images decreased
from 27.1–27.5 MB to 6.6–7.0 MB (74.7–75.6%), and Next images from about 84.3 MB
to 75.8–75.9 MB (10%). These compare identical fixture payloads and exclude small
manifests/configuration and attestations; they are not universal site sizes.

Source `d5b6fac` addresses another portion of audit R5: imports now preserve
explicit document-level ATX/setext heading IDs in `{#id}` notation and Docusaurus
MDX/HTML comment forms. The converter recognizes headings with Markdown parser
offsets, preserves inline formatting/rewritten links, and emits literal MDX
`h1`–`h6` elements. A leading H1 becomes a literal anchor below the layout's
single title. Title extraction ignores code examples and removes ID syntax.
Duplicate/unsupported IDs and nested identified headings fail before output
writes; the error identifies the source document. Classes and other heading
attributes are not carried over. Automatic source/plugin slugs and implicit
leading-H1 anchors still need review; this does not close all of R5.

The engine's shared heading reader now indexes literal JSX heading IDs for TOC
and search without executing expressions or assuming spread/dynamic/component
IDs. Existing deep validation checks the actual literal IDs, including collisions
with generated anchors. No new runtime dependency was added.

A new regression exposed the old fence scanner treating shorter or different
fences inside examples as closing fences. Shared marker/width tracking fixes
that in both Material demotion and MDX escaping. Balanced markup spanning a
replaced heading still uses the complete document as its recognition context.

Local validation: 56 converter unit tests and 29 CLI conversion/import/write
safety tests passed; 54 engine heading/TOC/search/deep-check tests passed.
Warnings-denied workspace Clippy, formatting, engine typechecking/packaging,
and the docs deep/strict check (55 routes, 23 examples) passed. The real native
CLI-to-engine fixture passed for MkDocs and Docusaurus: explicit title IDs,
formatted and linked headings, Unicode IDs, reference URLs, unique rendered IDs,
search/TOC targets, strict links and asset bytes. Its existing hosted CI step
also checks full static exports, one page title and rendered TOC links.
No local container or browser ran. Final-head CI and merge are pending for
this import milestone; source functionality remains unpublished rc.2.

Other providers, broader source-format fidelity and release/production migration
remain separate work. Netlify's hosting-comment hydration issue remains external;
the old production site/domain is retained. No new release was published.


### Imported metadata and Docusaurus document IDs — 2026-11-04

PR #69 merged as `b1ff5081013a89b8c98ed95715d8bc114c8a1fe5` after all
16 checks passed in run `41905598125` on `fb02026`; the merge and tested trees
match (`fda6beec21e54a0813a023a2ee256b155277e91c`). Its post-merge base-image
run `41910675985` passed; main CI `41910675972` was still running when this
record was written. Both post-merge runs for PR #68 (`41902611421` and
`41902611084`) finished successfully.

Source `9e296cc` addresses another portion of audit R5. Both importers now use
an authored YAML title and description as active page metadata, retaining the
original metadata in comments. Title precedence is front matter, document H1,
sidebar label, then source path without extension. Sidebar labels are emitted
independently using the engine's existing page/label entries. The shared inert
YAML reader handles BOMs, CRLF and `...` delimiters, reports malformed metadata
with its source path, and validates the types of supported fields. Retained
comments also handle Unicode YAML line separators so their text cannot become
active output fields. No runtime dependency or engine schema was added.

Docusaurus imports inventory the actual `.md`/`.mdx` sources and resolve custom
IDs using the default numeric-prefix rules and `parse_number_prefixes: false`.
A custom ID replaces the basename only. `sidebar_label` overrides a sidebar
item label. Legacy filename IDs remain accepted when unambiguous; conflicts
between canonical IDs and legacy names, duplicate IDs, missing sidebar targets
and ambiguous `.md`/`.mdx` stems fail before writes. Routes and relative links
still derive from physical source paths; custom `slug` values, plugin paths and
custom JavaScript prefix parsers are not applied. Plain `sidebars.json` is now
parsed directly. The inventory rejects symlinks/junctions before reads and
normalizes native path separators for portable lookup.

Draft/unlisted documents are excluded and reported, including explicit sidebar
entries and folder-derived navigation. An explicit empty sidebar stays empty.
Unsupported autogenerated branches leave omitted documents in the report;
they no longer cause all documents to be promoted when no supported entries
remain. Empty navigation and groups emit YAML arrays rather than null values.
Dynamic/missing sidebars retain the documented alphabetical fallback, with
exclusions applied. This remains a partial fidelity improvement, not full
Docusaurus execution or closure of R5.

Local verification: 60 converter unit tests and 37 CLI metadata/reference/
import/write-safety tests passed. Warnings-denied workspace Clippy, formatting
and docs deep/strict checks passed (55 routes, 23 examples). Both real CLI-to-
engine fixtures passed for metadata, distinct sidebar labels, custom IDs,
physical routes, rendered MDX, heading/search/TOC targets and asset bytes. The
metadata CLI suite is now in Linux/macOS/Windows CI; the existing full-export
fixture additionally checks page metadata and sidebar labels in generated HTML.
No local container or browser ran. Final-head hosted CI and merge are pending.

This remains unpublished rc.2 functionality. Release preparation must be
refreshed before any publication, which still needs explicit approval following
the earlier automatic approval rejection. Netlify's hosting-comment issue,
production cutover, other deployment/CI providers and broader import fidelity
remain separate work. The existing production site/domain remains intact.


Follow-up review fix `0fb7e14`: plain JSON sidebar strings are always parsed as
inert data, so labels containing ellipses, backticks or `require()` do not
trigger JavaScript fallback and accidentally include omitted files. All nine
metadata CLI fixtures pass locally after this correction (38 CLI tests across
the milestone's four suites, plus 60 converter tests). PR #70 hosts the final
native/full-export acceptance; CI on the corrected head is pending.


Native CI follow-up: run `41913945831` on `1bc4b1f` passed eight metadata
fixtures on Windows but failed while creating the ninth fixture's junction,
before invoking the importer. `d3db17c` builds that path with native Windows
separators, matching the existing passing junction fixtures, logs setup errors,
and removes the junction before cleanup. All nine fixtures pass locally after
the correction; final-head Windows and full acceptance must still pass.


### Import metadata closure and autogenerated sidebars — 2026-11-04

PR #70 merged as `5001718ed39f022247494b5452e1352642c1152b` after all
16 checks passed in run `41914893785` on `b550285`. Tested and merged trees
match (`87a230cde310e97dada489935212e19aa1e5eb29`). Post-merge base-image run
`41918572383` succeeded. Main run `41918572402` also succeeded after retrying
only its macOS ARM64 packaging job: the original job was cancelled while
queued, with no runner assigned or steps executed. This was not a product
failure. The previous PR #69 main run `41910675972` finished successfully.

Source `94296ac` expands data-only Docusaurus `autogenerated` sidebar slices.
Each selected directory contributes its contents, with nested folders becoming
categories. Manual entries and separate slices retain their relative positions.
Within a slice, finite `sidebar_position`, category metadata positions and
numeric filename prefixes determine order, with source names breaking ties.
Category `_category_.json`, `.yml` and `.yaml` labels/positions are supported;
category metadata overrides linked-index metadata, then folder-derived defaults.
Index/README/same-folder-name conventions and explicit local/full document ID
links become the first child page, or a single page link for a lone index.
`link: null` disables the index convention. Repeated navigation references now
emit each source page only once.

Drafts, unlisted pages, partials and hidden Markdown are excluded and reported;
partials/hidden documents are skipped before reading their contents. Explicit
empty sidebars remain empty. Missing or dynamic sidebars retain alphabetical
fallback. JSON/JSON5 sidebar data is parsed before dynamic-code detection, so
JavaScript-looking labels/comments remain inert data. Invalid positions,
malformed/duplicate category metadata and unsafe or missing directories fail
before output writes. Generated paths use portable separators on Windows.

Local verification: **58 converter unit tests and 47 CLI integration tests**
passed, covering metadata, references, import delivery, autogenerated sidebars
and safe writes. Warnings-denied workspace Clippy, formatting and documentation
deep/strict validation passed (55 routes, 23 examples). Both actual CLI-to-engine
fixtures passed, including generated hierarchy, labels, routes, MDX rendering,
heading/search/TOC targets and asset bytes. CI now runs the new suite on Linux,
macOS and Windows, and the full-export fixture checks the generated category
label in HTML. Hosted CI and merge are pending; no local container or browser
was used.

This is a further partial resolution of audit R5, not complete Docusaurus
compatibility. Generated category landing pages, manual category links,
category styling/collapse fields, custom JavaScript generators, custom slugs
and plugin routing remain unsupported and documented for review. This remains
unpublished rc.2 functionality; release preparation and approval, Netlify's
hosting-comment issue/production cutover, and additional deployment/CI providers
remain separate work.


CI follow-up: run `41955787336` on `8fa5b20` passed 15 jobs, including all
native import suites, but the CLI job failed an existing development-browser
fixture assertion: navigation count changed from 9 to 10 after the test used
Back and then wrote an invalid edit. Both the search index and server revision
remained unchanged across that edit. The fixture recorded its baseline before
waiting for the restored page to acknowledge the last valid asset revision.

Fix `a4c4e44` waits for two successful revision polls in the same document
before recording that baseline, and rechecks the valid heading/image. It keeps
the existing assertions that invalid edits preserve the revision, navigation
count, content and image. No product runtime was changed. JavaScript syntax and
12 deterministic reload/watcher unit tests pass locally. Browser confirmation
and all final-head CI gates remain pending; no local browser/container ran.


### Category landing pages and manual category links — 2026-11-04

PR #71 merged as `eb9070595acf23299eeb01167db1d68f3c762866` after all 16
checks passed in run `41957491288` on `85778f2`. Tested and merged trees match
(`46d90f4f77bfa88c1aa55b23605b5cd1fb643f38`). Both standalone and container
versions of the corrected reload fixture passed, including its invalid-edit
preservation assertions. Post-merge base-image run `41960754932` succeeded;
main run `41960754928` was still running when this entry was prepared.

Source `40d665e` adds Docusaurus generated category indexes and manual category
document links. Manual links resolve full document IDs (with existing unambiguous
legacy filename compatibility), import linked documents outside the child list,
put the linked page first, remove a duplicate direct child, and collapse a lone
index to one link using the category label. Draft/unlisted linked pages remain
excluded. Autogenerated category document links retain local-ID-first lookup.

Generated-index metadata from manual categories and JSON/YAML folder metadata
creates ordinary editable MDX pages with titles, descriptions and nested lists
of imported child links. The title defaults to the category label. Pages live
under `01-guide/category/<normalized-label>/index.mdx`; numbered alternatives
avoid routes occupied by generated pages or inventoried authored documents,
including drafts, unlisted and orphan sources. Repeated autogenerated slices
reuse the same folder's generated page. Empty/fully excluded categories report
that no landing page was created. No source files are changed.

The Docusaurus adapter keeps synthetic pages in memory and feeds them through
the shared emitter/reference resolver. Relative references are rewritten using
the same normalized routes as authored documents. Metadata prose is encoded as
literal text, including markup, Markdown punctuation and module-looking text,
so descriptions/labels cannot introduce MDX code or extra links. No dependency,
engine schema or public CLI syntax changed.

Local verification: **58 converter unit tests + 53 CLI integration tests**
passed, plus formatting, warnings-denied workspace Clippy, documentation deep/
strict validation (55 routes, 23 examples), and both real CLI-to-engine fixtures.
The fixtures validate manual category links, nested generated pages, descriptions,
search, plain-text rendering, routes and asset bytes. CI adds the new native
suite on Linux/macOS/Windows; full-export acceptance verifies generated pages
and links with a deployment prefix. Hosted CI/merge remain pending. No local
container or GUI browser ran.

This is another partial resolution of audit R5. Generated-index source slug,
image, keywords and other unsupported link fields are reported; custom source
URLs, plugins/generators, category styling and embedded DocCardList components
remain unsupported. The generated output is a nested Markdown link list, not
Docusaurus cards. Source rc.2 remains unpublished; release preparation/approval,
Netlify's hosting-comment issue and production cutover, additional cloud/CI
providers and broader conversion fidelity remain separate tasks.


### Explicit Docusaurus URL slugs — 2026-11-04

Prior milestone closure: PR #72 merged as `ab768ea635750b13a092ad97c43f4b3e6782b0cd`
after all 16 checks passed in run `41963452333` on `1d4e5b6`. Tested/merged
trees match (`7e00781c79dc7d72c25ca36da857f8e1aefa4b9c`). The post-merge
base-image workflow `41966348546` passed. Main workflow `41966348612` has a
Docker `info` timeout in CLI job `127044130186`, before the first hosted build;
the Windows portable job is still running. A targeted retry was refused while
the run remained active; revisit after completion. This is not a diagnosed
product fix. PR #71 main `41960754928` and base `41960754932` both passed.

Source `6e9f942` addresses another part of audit R5. Explicit Docusaurus document
and generated-category slugs become validated `route` front matter beneath
`/guide`, using the engine's existing route override. Absolute document slugs
start at that guide root; relative slugs resolve from the physical document's
parent directories, applying default numeric-prefix removal unless disabled
for that document. Literal relative dot segments may stay within the docs root.
Category slugs resolve from the docs root regardless of their physical folder.
Only explicit slugs change routes; other documents retain existing defaults.

Imported filenames, sidebar source references and asset bases stay unchanged.
Converted file links, category child links, navigation and search use the new
routes. Case is retained; percent encoding is decoded once, Unicode is normalized
to NFC and URL links are encoded. Trailing slashes are removed. Colliding routes
(including case-only and Unicode-equivalent paths), malformed encoding, encoded
separators/dot segments, root escapes, queries/fragments/hosts and nonportable
segments fail before writes. Unknown source `route` metadata remains inert;
MkDocs slug metadata is unchanged. Explicit category slugs are reserved before
generated defaults, so defaults receive suffixes independently of sidebar order
without changing their physical filenames. No engine schema/dependency changed.

Local verification passed: **60 converter unit tests + 60 CLI integration tests**,
workspace Clippy with warnings denied, formatting, deep/strict docs validation
(55 routes, 23 examples) and both real CLI-to-engine fixtures. These fixtures
exercise actual route loading, MDX rendering, sidebar/search links, category
pages, heading anchors and asset bytes with uppercase and Unicode/space URLs.
CI adds native slug coverage on Linux/macOS/Windows and full static-export HTML
checks with `/products/docs`. Final-head hosted acceptance and merge are pending.
No local container or GUI browser ran.

Remaining import work includes original deployment prefixes/root-relative source
URLs, plugin routing/generators, category styling, generated-index images/keywords
and embedded DocCardList components. R5 is still only partially resolved. Source
rc.2 remains unpublished; release preparation/approval, Netlify's hosting-comment
issue/production cutover and additional deployment/CI providers remain separate.


#### Unicode static-export follow-up — 2026-11-04

PR #73 run `41968082371` on `a9251b6` passed 13 jobs, including all native
import suites and packages. ARM64 image acceptance failed before image building
with another Docker `info` timeout. The CLI job passed its prior acceptance
steps but failed the new Unicode HTML assertion in the last conversion fixture.
The combined manifest job was skipped because ARM64 had failed. Do not claim
this run as final acceptance or the runtime timeout as fixed.

A native command-line Next 16.4.2 webpack export reproduced the Unicode page
as a 404 HTML shell. Instrumentation showed page params containing encoded
`Caf%C3%A9`/`Release%20Notes`, while metadata received decoded `Café`/`Release Notes`.
The hosted CI build using Next 16.4.5/Turbopack showed the same missing page link.
Fix `9eb97ef` matches exact registered routes first, then tries one safe decoding
step for page lookup and active sidebar navigation. Literal percent routes retain
precedence; malformed encodings, encoded delimiters and dot segments do not
become different routes. The route inventory and static params stay unchanged.

All **423 engine tests** and engine typechecking pass. The actual native webpack
export now contains the Café H1, the prefixed document link and an active sidebar
link, with no Next error shell. This was a command-line build only: no local
container or browser ran. Full-export acceptance retains its original links and
adds explicit page-title/active-link checks. The same conversion check now runs
immediately after the CI engine image is ready, before the long container suite;
no acceptance steps were removed. All final-head CI gates must pass again.

Prior PR #72 post-merge main workflow `41966348612` subsequently passed on its
targeted CLI-job retry, with no source change; base `41966348546` also passed.
Recurring Docker info timeouts remain a follow-up. Docker's official
[info implementation](https://github.com/docker/cli/blob/master/cli/command/system/info.go)
probes client plugins, and the failed ARM64 log left a Compose helper to clean
up. This is a diagnostic lead, not a proven cause or fix.


#### Selected-runtime budget and CI prerequisite correction — 2026-11-04

The next run, `41971949013` on `9eea726`, again hit the three-second Docker
`info` limit on ARM64 immediately after Docker had built the engine image and
started the local registry. It also exposed an error in the preceding CI edit:
the earlier conversion-export step had been placed in the native portable job,
which does not build `zellij-ci`, so it failed trying to pull that missing image.

Source `0d605a6` restores the conversion check to the CLI container job after
image creation/runtime selection and before the longer hosted tests. A parsed
workflow assertion verifies that ordering and that the portable matrix does not
contain the container-only step. All existing acceptance steps remain.

The same source lets an explicitly selected runtime use the existing full
**twelve-second discovery budget**: there are no fallback clients/sockets to
reserve time for in that mode. Automatic discovery still allows three seconds
per probe within twelve seconds overall. Metadata probes remain two seconds,
endpoint selection/secrecy remain unchanged, and no service is started. This is
bounded tolerance for slower healthy runtimes, not a claim to have diagnosed or
fixed Docker's plugin latency. A real CLI fixture with a four-second fake Docker
info response proves the selected endpoint succeeds without calling Podman.

Local verification: 9 runtime-requirement CLI tests and 25 engine Rust unit tests
passed, alongside the prior 120 import/safety Rust tests, 423 engine JavaScript
tests and native Unicode export. Clippy, formatting and refreshed deep/strict
documentation checks passed. Final-head CI must pass before merging PR #73.


### 2026-11-04 — import asset policy and optional validation

PR #73 merged as `a4b575ddbe35b9a0f4c176526bd1a704e682194d` after all 16
checks passed in [CI 41972577587](https://github.com/mylife-inc/Zellij/actions/runs/41972577587)
on final head `208e339234ae336489cb4e46f573c53530a10a95`. Tested/merged tree:
`02a8c4c6c7eee05e05ab8a1403698d9c3d17e31c`. Both post-merge workflows passed:
main CI `41975635129` and base image `41975635128`. This closes the preceding
pending status; earlier failed runs remain recorded above.

Source `ae7c611` exposes the converter library's existing asset-copy policy as
`--assets copy|skip` on `convert`/`cv`, `migrate`/`mig` and `ship`. Copy remains
the default. Skip preserves original asset references and reports omissions;
document links still resolve to imported routes. Path containment, symlink and
hidden-source protections still apply before writing. Omitted assets may need
manual placement/reference repair before publishing. The original design's
`link` policy is not implemented; Clap rejects it before any output is written.
This import-only policy is not saved as a delivery/build setting.

`convert`/`cv --validate` uses migrate's existing hosted check after writing
content or site layout. Missing runtimes return exit 4, content-check failures
return exit 3, and both retain the completed import for review and `zel check`.
Migrate always validates; accepting the shared flag does not run a second check.
Dry runs never validate, write, probe runtimes or invoke delivery. This supersedes
the original design's suggestion to report validation as skipped on an unavailable
runtime: an explicitly requested check must not report success when it did not run.

Local verification passed: 54 CLI import regressions, 27 CLI unit tests (one
existing ignored test), 60 converter tests, warnings-denied workspace Clippy,
formatting, deep/strict docs (55 routes, 23 examples), and both real CLI-to-engine
conversion fixtures. Unit tests requiring loopback sockets passed outside the
socket-restricting sandbox. No local Docker daemon or browser was used. CI's
real export fixture now tests `convert --validate` alongside migrate, and image
publication acceptance exercises explicit asset copying. Final-head native
three-OS and hosted CI remain required before merge.

Public release remains `zellij-v0.2.0-rc.1`; these controls are unpublished rc.2
source. No release, production Netlify deployment or domain change was requested.
Netlify's hosting-comment hydration defect remains with support. Import source
URL/plugin fidelity and additional deployment providers remain incomplete.


### 2026-11-05 — website-link import and CLI completion status

PR #74 merged as `74abf77a6592633288692a1af21fee9905f3ace6`, after all 16
checks passed in [41991356062](https://github.com/mylife-inc/Zellij/actions/runs/41991356062)
on `5846c16c698aaa1212df6a42a89afe83c54aa877`. Tested/merged tree:
`4f698704576125ff784b72469b8786c1a18bb430`. Both post-merge workflows passed:
main `41995023512` and base image `41995023365`.

The user asked whether all CLI commands are implemented and authorized the
suggested website-link work. Every currently registered command family has
handlers, but the entire original specification is not complete. Source
`8f46bd4` replaces the generic dispatch stub with an exhaustive match; normalized
aliases are the only unreachable forms. Future missing handlers now fail to
compile. The current status table is `specs/zel-cli-status-2026-11-05.md`;
additional providers, converters, CI adapters and interactive authoring remain.

The same source maps root-relative and configured same-origin HTTP(S) Markdown
links through source URL conventions to emitted Zellij routes. MkDocs uses
site_url/use_directory_urls, including README/index and direct HTML addresses.
Docusaurus reads direct literal url/baseUrl and a single classic preset/docs
plugin routeBasePath. Default IDs, number-prefix and index conventions, explicit
slugs and explicit category slugs participate. Queries/fragments are unchanged;
URL paths preserve case, normalize Unicode and decode percent encoding once.
Referenced website assets use contained source roots and the existing copy/skip
policy. Other origins are untouched; no URLs are fetched and no JS is executed.

Known omitted or colliding source URLs remain unchanged with reports. Invalid
supported routing settings and unsafe local paths fail before writes. Computed,
custom, localized, versioned and multiple-instance Docusaurus routing is reported
for review, as are implicit generated-category URLs. Relative HTML URLs and
HTML/JSX attributes remain outside the new mapping. Destination deployment
settings are not inferred from the old site's prefix.

Local checks passed: 60 converter unit tests, 63 CLI import tests, 7 alias tests,
workspace Clippy with warnings denied, formatting, deep/strict documentation
(55 routes, 23 examples), and both CLI-to-engine fixtures. CI now includes the
new tests on three native OSes and real hosted exports containing source website
links. No local Docker or browser was used. Final-head CI is required before merge.

Upstream rules were verified against the MkDocs configuration/file URL code and
Docusaurus create-doc/slug code; source links are in the conversion reference.
Refreshed public tag is `v0.2.0-rc.1` (some older entries incorrectly prefixed
that candidate tag with `zellij-`). rc.2 remains unpublished, and its prior
publication proposal predates this work. Netlify support's hosting-comment
hydration issue and production migration remain separate.


#### Windows path follow-up — 2026-11-05

Initial PR #75 CI `42000267890` on `d65ded6` passed the real converted-link
HTML export step but failed a new Windows URL regression. WalkDir returned
native backslash paths while the MkDocs listed-path inventory used forward
slashes. Nested listed documents were mistakenly added as orphans; their
inconsistent route spelling then made website URLs ambiguous. The same path
handling could also produce nonportable names for folder-derived navigation.

Source `74bacea` normalizes discovered relative path components to `/`, as the
Docusaurus inventory already does. The existing nested URL regression and an
additional nested, no-nav fixture cover both cases on every CI platform.
All 31 targeted local import tests pass after the correction, alongside the
prior converter/alias checks (131 distinct Rust tests total). Full final-head
CI, including the Windows rerun, is still required before merge. This is a
code defect correction, not a retry of an unexplained CI failure.


### 2026-11-05 — relative website document links

PR #75 merged as `2c5ac7caae98af6bf4ec552c467d2614904ef1fe` after all 16
checks passed in [42001094217](https://github.com/mylife-inc/Zellij/actions/runs/42001094217)
on final head `bb2527a5f50b3b9ecd870c5bb5e3a9f9c20962b1`. Tested and merged
tree: `185ce605b71138a58d700b8e7a6c9e9f1850f77c`. This closes the preceding
Windows/final-PR pending status. Base-image post-merge run `42005279024` passed;
main `42005279080` was still running at this refresh.

Source `d141846` adds relative website document URL lookup to both importers.
The base is the original page URL directory, including MkDocs directory/HTML
style, Docusaurus slugs and literal `trailingSlash` settings. Missing Docusaurus
slash policy requires agreement across both possibilities. Known omitted or
ambiguous destinations, image references to documents, and conflicts between
website targets and extensionless filename inference are reported unchanged.
Ordinary Markdown file links retain priority, including missing/omitted files;
existing regular assets (including HTML downloads) retain copy/skip semantics.
Website lookup maps known documents only and never creates a new asset root.
Source paths still pass containment/symlink checks; invalid input fails before
writes, and dry runs remain inert. Query/fragment text and code examples survive.

Compatibility: an extensionless link previously guessed from filenames can now
be reported when its URL interpretation is ambiguous. Use an explicit Markdown
file link or root-relative website URL to state its destination. The original
shared reference fixture now declares `trailingSlash: true`; a new fixture
asserts reporting when that policy is missing. This is an intentional contract
correction, not removal of the existing file-link acceptance assertions.

Local verification passed: **132 Rust tests** (60 converter unit tests and 72
CLI import tests), warnings-denied workspace Clippy, formatting, actionlint,
deep/strict documentation (55 routes, 23 examples), and both actual CLI-to-engine
MDX/rendering fixtures. Eight new native regressions cover URL/file distinctions,
category links, Unicode, omissions, asset controls, ambiguity and unsafe inputs.
CI includes them on Linux/macOS/Windows; the real-container export fixture also
asserts a relative source link in generated HTML. Those final-head CI gates
remain required before merge. No local Docker daemon or browser was used.

Primary behavior references: MkDocs structure/pages.py and configuration docs;
Docusaurus Markdown links and trailingSlash docs, linked in the local reference.
Relative website assets, HTML/JSX attributes, computed/custom/versioned routing
remain review tasks. All command families still have handlers, but the wider
provider/format/interactive specification remains incomplete. Public candidate
`v0.2.0-rc.1` is unchanged; no rc.2 publication or production Netlify migration
was requested. The Netlify hosting-comment issue remains with support.


#### Windows preview harness follow-up — 2026-11-05

PR #75 post-merge main run `42005279080` failed the Windows native preview
step in job `127170334382`. HTTP serving, Chromium watch acceptance and the
Windows import tests had passed. The WebKit watch iteration then asserted that
stdout already contained “Watching content and configuration” immediately after
start() observed “Serving at…”. The CLI emits those lines separately, and the
Windows console helper relays stdout asynchronously; a single pipe chunk is
not a guarantee that both messages have arrived. The assertion failed before
browser startup/tracing, so the original log had no browser trace artifact.

Source `a1003bd` uses the harness's bounded wait for the watch announcement,
retains the assertion, detects early process exit, and includes captured stdout
and stderr on startup failure. No product timing or watcher behavior is changed,
and no arbitrary sleep/retry or acceptance skip is added. Node syntax and diff
checks pass. Real Windows/browser execution remains CI-only. The original
failed post-merge run is retained; new final-head CI must verify this fix.

### 2026-11-05 — PR #76 closure and VitePress import source

PR #76 merged as `e8f525d1e765f0a455de192b200866377f2dd45b` after all 16
checks passed in [42007978630](https://github.com/mylife-inc/Zellij/actions/runs/42007978630)
on `903c66d236bd9a48746a6be6070d9446482d2c93`. Merge and tested head share
tree `1f4e626af5a93b747c1b28471f04e4b57e704eab`. This closes relative website
links (`d141846`) and the Windows watch-announcement relay race (`a1003bd`).
Post-merge base-image run `42011273390` passed. Main `42011273809` has 15
successful jobs, including Windows preview; CLI acceptance is still running.

Source `38b9d41` adds the registered VitePress converter to `detect`,
`list formats`, `convert`/`cv`, `migrate`/`mig` and `ship`. It reads exactly one
`.vitepress/config.{js,ts,mjs,mts}`, accepting literal exports or the standard
`import { defineConfig } from 'vitepress'` wrapper without evaluation. Supports
contained `srcDir`, title/description, nested and multiple declarative sidebars,
inherited item bases, linked groups, page metadata, Markdown links, source
`base`/`cleanUrls` URLs and referenced source/public assets. Multiple sidebars
become prefix-labelled groups in lexical order; omitted documents are reported,
and explicit empty/false sidebars stay empty. Absent sidebars derive visible
Markdown navigation. Absolute HTTP(S) URLs stay external without an origin.

Dynamic config, custom routing/exclusions/locales, dynamic parameter files and
custom output/cache roots fail before writes. Vue components, top navigation,
home layouts and custom Markdown extensions remain manual migration work;
source text/metadata and reports are retained. Public assets use a separate
`assets/imported/public/` namespace; conflicting source/public root URLs fail
before writes, including with `--assets skip`. Existing shared preflight retains
source isolation, symlink/junction rejection, route collisions and inert dry
runs. This is a documented subset, not complete VitePress fidelity.

Local verification passed: **140 Rust tests** (62 converter unit tests and 78
import/inspection integration tests), Clippy with warnings denied, formatting,
actionlint and diff checks; deep/strict docs cover 55 routes and 23 examples.
The Node-only CLI-to-engine fixture passes for all three formats, checking
rendered routes, titles, heading/TOC/search anchors, query/fragment links and
copied bytes under a deployment prefix. Hosted export and three-OS VitePress
acceptance are added to CI; final-head hosted/native verification and merge
remain pending. No local Docker or browser/GUI tests were run.

Semantics were checked against VitePress 1.6.4 source and its
[routing](https://vuejs.github.io/vitepress/v1/guide/routing),
[sidebar](https://vuejs.github.io/vitepress/v1/reference/default-theme-sidebar),
[asset](https://vuejs.github.io/vitepress/v1/guide/asset-handling) and
[config](https://vuejs.github.io/vitepress/v1/reference/site-config) documentation.
Current vitepress.dev defaults to a newer development version; this milestone
does not claim that version's full behavior. The published native release
remains rc.1. Release publication and production Netlify migration are separate.

### 2026-11-05 — VitePress closure and Jekyll documentation imports

PR #77 merged as `0729235ed7d8e3a0835b324e045e5e19a15d79e3` after all 16
checks passed in [42014697065](https://github.com/mylife-inc/Zellij/actions/runs/42014697065)
on `155f09c0feb9cc77477f1d7fbc197fc8e2943012`. Its tree matches the tested
`3f5edcf76ad3ab9e5e5028a8643124200ae5c38b`. Final fix `155f09c` prevents a
URL in a sidebar query string from making a local VitePress page look external.
Both post-merge workflows passed: main `42017999796`, base image `42017999836`.
PR #76's previously pending main run `42011273809` also passed, completing
both of that merge's post-merge workflows. Local main was clean and synchronized
at the start of this milestone; native release remains `v0.2.0-rc.1`.

Source `0073bca` registers `jekyll` with detection, formats, convert/migrate and
ship. It supports one YAML config, contained source/data/output roots, front-
matter `.md`/`.markdown` pages, enabled documentation collections and literal
scoped defaults. Supported navigation data arrays (`name/link` or `title/url`,
optional children, optionally wrapped in `main`) retain order and nesting.
Without a navigation file, published pages precede enabled collections; literal
collection order lists precede filename order. Empty explicit navigation stays
empty; omitted/unpublished/navigation-excluded pages are reported.

Root-relative, same-origin and relative website links use source URL settings,
index conventions, supported permalink styles, literal page permalinks and
collection `:collection`/`:path`/`:output_ext` patterns. These map to existing
filename-derived Zellij routes, not destination permalink overrides. Referenced
static assets must be eligible under the Jekyll source inventory: excluded,
internal data/template and generated files cannot be published through links.
Canonical source roots fix macOS alias comparisons; BOM/CRLF template detection
is covered. Original metadata is retained alongside effective defaults.

Limits are explicit: no Ruby/Liquid/theme/plugin execution, blog migration,
dated collections, glob policies/scopes, forced includes, TOML import or
arbitrary routing patterns. Unsupported navigation/configuration fails before
writes; Liquid/theme presentation is reported for manual work. The importer
adds no dependencies. Hugo remains the next importer; Sphinx and other
provider/CI/interactive work remain in the broader CLI backlog.

Local verification passed: **155 Rust tests** (64 converter unit tests plus
91 import/inspection integration tests), final targeted Jekyll regressions,
warnings-denied Clippy, formatting, actionlint and diff checks. Deep/strict docs
cover 55 routes and 23 examples. Node-only rendering passes all four formats,
including a Jekyll collection, source-relative assets, anchors, search/TOC and
website links. Three-OS import acceptance and real hosted export join CI;
final-head hosted/native checks and merge remain pending. No local Docker or
browser/GUI test ran. User docs, current CLI status and unpublished rc.2 release
notes were updated with behavior, command examples and limits.

Source semantics were checked against Jekyll 4.4.1's
[configuration](https://jekyllrb.com/docs/configuration/options/),
[collections](https://jekyllrb.com/docs/collections/),
[navigation tutorial](https://jekyllrb.com/docs/step-by-step/06-data-files/),
[permalinks](https://jekyllrb.com/docs/permalinks/) and matching tagged source
for page/collection URLs, publication and scoped defaults. This remains a
conservative documentation subset. Production documentation deployment,
Netlify migration and native release publication are separate pending work.

#### Windows ARM64 acceptance follow-up — 2026-11-05

Initial Jekyll PR #78 run `42084376374` failed native ARM64 Windows job
`127445423055` after the binary built and both PowerShell installers passed.
The visible error was `EBUSY` deleting the installed executable from the
harness's disposable directory. The unconditional cleanup could replace an
earlier assertion error, so the original cause cannot be inferred from that
log alone. Source `e460f40` preserves both errors with `AggregateError`, includes
command arguments in failed-command diagnostics, and bounds retries only for
filesystem cleanup (10 retries with 100 ms linear backoff). No installer or
acceptance assertion is retried or skipped. Node syntax and diff checks pass;
new final-head Windows and full CI evidence are required before merge.

### 2026-11-05 — Jekyll closure and Hugo documentation imports

PR #78 merged as `6190e51bcfca91a874f508af3f2a523492319440` after all 16
checks passed in [42085346212](https://github.com/mylife-inc/Zellij/actions/runs/42085346212)
on `ffdeca7f8d245979e9dcd4b7d58e2d40a093ca92`. Its tree equals tested tree
`c594f2570f1664de72cdec025206748e965218d8`. Windows ARM64 job `127449174329`
passed both installers, native upgrades/refusals/cancellation, portable output,
HTTP serve/view and real Ctrl-C. The earlier masked cleanup failure remains
historical evidence; this passing run does not establish its original cause.
Post-merge base image `42090020402` passed. Main `42090020416` has 15 passing
jobs with CLI acceptance still running at this refresh. Local main was clean
and synchronized before the Hugo branch; published native candidate remains rc.1.

Source `5631e6c` registers Hugo detection and conversion through convert/cv,
migrate/mig and ship. One `hugo.*` or legacy `config.*` TOML/YAML/JSON file
selects contained content/static roots. Markdown pages accept optional front
matter in all three formats, case-insensitive keys, UTC/offset ISO dates,
publication flags and supported page build rules. Draft, future, expired,
headless and unlisted pages stay excluded unless a supported explicit build
flag permits them. Date decisions are made once at conversion time.

Hugo section `_index.md` maps to visible `index.mdx`; leaf bundles keep their
index pages and do not promote Markdown resources into pages. `DocTree` now
has an explicit source-to-output filename map, leaving original source keys
available for Markdown links and asset lookup. Shared emit/rewrite defaults
remain unchanged for existing importers. Derived section navigation uses
weight, descending date, case-folded link title and source path. Theme menus
and locale-specific collation are outside this subset.

Supported deployment prefixes, pretty URLs, regular-page slugs/URLs and
source-file links map to filename-derived Zellij routes. Eligible referenced
content/static resources retain their bytes; implicit sections preserve asset
filename case. Static/resource URL conflicts and unsafe paths fail before
writes. Explicit section/bundle route overrides, menus, split configuration,
modules, cascades, localization, custom date rules and complex routing require
manual migration. Shortcodes and aliases remain reported source metadata/text;
no Go, template or theme execution takes place.

Tagged Hugo v0.169.0 source inspection found that leading ASCII whitespace and
inline opening delimiters still introduce front matter. The importer matches
this behavior so draft rules cannot be bypassed; Org front matter and malformed
end delimiters fail explicitly. Official references:
[configuration](https://gohugo.io/configuration/introduction/),
[front matter](https://gohugo.io/content-management/front-matter/),
[bundles](https://gohugo.io/content-management/page-bundles/),
[publication options](https://gohugo.io/content-management/build-options/),
[URLs](https://gohugo.io/content-management/urls/) and the tagged
[front matter parser](https://github.com/gohugoio/hugo/blob/v0.169.0/parser/pageparser/pagelexer_intro.go),
[path helper](https://github.com/gohugoio/hugo/blob/v0.169.0/common/paths/path.go),
[ordering](https://github.com/gohugoio/hugo/blob/v0.169.0/resources/page/pages_sort.go).

Local verification passed: **171 distinct Rust tests** (66 converter unit,
105 import/inspection integration), including final targeted regressions after
parser/resource corrections; warnings-denied Clippy, formatting, actionlint and
diff checks; deep/strict docs covering 55 routes and 23 examples; and Node-only
CLI-to-engine rendering for all five importers. The Hugo fixture checks section
pages, a leaf bundle, links, Unicode asset paths, titles, headings/search and
static resources. The same fixture's hosted export and three-OS import tests
are in CI; final-head hosted/native acceptance and merge are still pending.
No local Docker, browser or GUI tests ran. The TOML and date dependencies use
minimal features; no runtime dependency installation is added for users.

The conversion reference, guides, detection/help overview, current CLI status,
and unpublished rc.2 notes were updated. Sphinx imports and the remaining
provider/CI/interactive/advanced-option backlog are still incomplete. Native
release publication and production Netlify migration remain separate work.

#### Hugo resource ownership correction — 2026-11-05

Source `f7f0121` keeps implicit top-level section resources independent of a
draft home page. A section's visible image remains eligible while a resource
owned by the omitted home page stays excluded. The expanded real-CLI fixture
passes all 14 Hugo tests; warnings-denied Clippy passes. This joins PR #79 and
requires the new final-head CI run before merge.

#### Hugo Windows path correction — 2026-11-05

PR #79 run `42094771583` failed Windows import job `127482680944`: native
`with_file_name` changed nested `_index.md` output to a backslash-containing
relative path, which the portable-path validator correctly refused. Source
`771e40e` normalizes generated output filenames from path components to `/`.
The existing section/link integration fixture covers the failure. All 14 Hugo
tests and warnings-denied Clippy pass locally; new Windows and full final-head
CI evidence are still required before merge. The safety validator is unchanged.


### 2026-11-05 — Hugo closure and Sphinx reStructuredText imports

PR #79 merged as `7a03f50c54421ba03a4a00f1be4cc5c747fccd65` after all 16
checks passed in [42095877145](https://github.com/mylife-inc/Zellij/actions/runs/42095877145)
on `8102782f0252c97d222236bd6b9b3161d71045ce`. Tested and merged tree:
`cf3541bcae5aeac20caf35e665a97758c95c6ef0`. Both post-merge workflows passed:
main `42099649036` (all 16 jobs) and base image `42099648776`. This closes the
Windows section-path correction `771e40e` and implicit-section resource fix
`f7f0121`. PR #78's previously pending main run `42090020416` passed too; both
of that merge's workflows are green. Main was clean and synchronized when this
branch began. The published native candidate remains `v0.2.0-rc.1`.

Source `968b469` registers Sphinx in detect/formats/convert/cv/migrate/mig/ship.
The directory containing `conf.py` is the source root. Configuration is parsed
as literal Python data with no Python interpreter or dependency installation.
Supported settings include project, root_doc/master_doc, .rst source_suffix,
UTF-8, explicit exclusions and html_baseurl. Executable/dynamic configuration,
extensions and unsupported content-transforming settings fail before writes.
Theme/presentation values are not applied as destination theme configuration.

Explicit toctrees preserve order, captions, labels and nested page groups.
Hidden trees define navigation without inline lists; self does not recurse.
Navigation cycles, excluded/missing targets, unsafe paths and route/label
collisions fail before output. Unlisted pages are reported and not published,
unlike Sphinx's ability to build orphan pages. The reStructuredText adapter
handles headings, paragraphs, emphasis, inline literals, flat lists, literal/code
blocks, standard admonitions, images, named hyperlinks and doc/ref/download
roles. Source HTML and directive-looking code remain literal. Section labels
and supported implicit IDs feed links, TOC and search. Alternate old heading IDs
require review. Unsupported syntax produces a source-specific preflight error;
this is a conservative subset, not full Sphinx/MyST/API-extension fidelity.

.rst links and HTML-builder website addresses map into filename-derived guide
routes. Contained referenced eligible assets retain bytes; source-root-relative
images/downloads do not inherit html_baseurl. Excluded sources, templates,
configuration and script files cannot be copied as downloads. Resource paths
are checked even under --assets skip. No source tool or configuration code runs.
No new dependencies were added.

The same source fixes shared MDX escaping of inline code containing shorter
backtick runs: literal spans match equal-length delimiters, preserving braces
and angle brackets instead of escaping code text. Regression tests cover this
and real Sphinx-to-engine rendering covers literal HTML, role text and code.

Local verification passed: **183 distinct Rust tests** (68 converter unit and
115 import/inspection integration), with final targeted Sphinx regressions after
resource/punctuation corrections; warnings-denied Clippy, formatting, actionlint
and diff checks; strict/deep documentation (55 routes, 23 examples); and Node-only
rendering for all six import formats. Hosted exports and the new three-OS Sphinx
suite are CI gates; final-head CI and merge are pending for this source. No local
Docker, browser or GUI tests ran. Logs use /private/tmp/zellij-sphinx-*; durable
scope and upstream references are in the conversion reference.

User documentation, quickstarts, CLI status and unpublished rc.2 notes were
updated. Remaining work includes richer Sphinx/other-format fidelity, provider
adapters, GitLab/CircleCI, interactive authoring and advanced options. Native
release publication needs separate explicit authorization; the old rc.2 proposal
predates this work. Production Netlify migration and the vendor's hosting-comment
hydration issue remain separate. No production deployment or domain change ran.


#### Sphinx literal resource follow-up — 2026-11-05

Source `6b5b1d2` preserves literal local image/download filenames before the
shared Markdown URL resolver. A source filename containing `%20` remains
separate from one containing a space; both original bytes and encoded emitted
URLs are covered. Root-relative resource paths retain source-root semantics.
Sphinx can also select localized image variants based on `language`, so nonempty
language settings are now rejected with guidance to choose resources explicitly
in a separate source copy. Absent/None/empty language settings remain supported.
The official [asset collector](https://github.com/sphinx-doc/sphinx/blob/master/sphinx/environment/collectors/asset.py)
was inspected read-only; no Python/Sphinx execution was introduced.

All 11 Sphinx integration tests, warnings-denied Clippy, formatting, strict docs
and all six Node rendering fixtures pass, including the percent/space ambiguity.
The expanded hosted export fixture checks real HTML URLs and copied bytes too.
Local distinct Rust coverage is now 184 tests. Initial PR #80 CI on `a850843`
had 14 passing jobs and no failures when this correction was prepared; that
older run does not validate this source. New final-head CI and merge are pending.


### 2026-11-05 — Cloudflare Pages native deployment

The next provider is Cloudflare Pages Direct Upload for generated static HTML.
`zel deploy cloudflare`, `zel cloudflare` / `cf`, saved `publish` / bare `deploy`,
`ship --provider cloudflare`, and `ci --deploy cloudflare` share provider intent.
The destination is an existing project name: explicit `--site` / `--site-id`,
then `CLOUDFLARE_PAGES_PROJECT_NAME`, then same-provider `publish.site`.
Changing providers never reuses the old provider's saved destination or env ID.
`CLOUDFLARE_ACCOUNT_ID` and `CLOUDFLARE_API_TOKEN` stay in the environment;
there is no account/project creation, Wrangler invocation or stored-login lookup.

Project preflight precedes the temporary build. Preview uses `zel-preview`,
independent of the local Git branch, and rejects a project whose production
branch has that name. Explicit `--prod` uses the preflight production branch.
The adapter inventories generated files, uses Cloudflare's BLAKE3/base64/extension
asset keys, upload JWTs, missing-hash lookup, bounded batches and multipart
manifest creation, then waits for deployment-stage success. Account tokens and
upload JWTs are never combined in a request. Identity/environment/URL checks,
redirect refusal, redacted provider errors and bounded requests apply. Creation
POSTs are never automatically retried; uncertain deployments require dashboard
inspection. Both renderers preserve source and previous output.

Supported exports contain at most 20,000 files, each at most 25 MiB. Symlinks,
junctions, special files and reserved Workers/Functions/custom rules are rejected;
this adapter does not deploy a Next server. Hosted URL/feature overrides retain
existing behavior. CI generates provider-specific variables/secrets, excludes
publication credentials from PR verification, supports explicit token provisioning,
and requires rc.2 or later. Published rc.1 remains unchanged. Docs explain setup,
preview behavior, unsupported forms and pending live-account acceptance.

Protocol references inspected read-only:
[deployment API](https://developers.cloudflare.com/api/resources/pages/subresources/projects/subresources/deployments/methods/create/),
[official upload implementation](https://github.com/cloudflare/workers-sdk/blob/main/packages/wrangler/src/pages/upload.ts),
[asset hashing](https://github.com/cloudflare/workers-sdk/blob/main/packages/deploy-helpers/src/deploy/helpers/hash.ts).
Rust mock HTTP tests cover uploads, caching, batches, Unicode/percent filenames,
validation ordering, failures, timeouts and portable output. GitHub CI adds a
real hosted-export/Cloudflare-protocol fixture. No live provider account was used.

PR #80 closure: all 16 checks passed on `fb622d1f84c3516e6ba6d0affbd21cd39b043b44`
in `42109434915`; merge `c067b5ae17a8a6f3e724e0c625383cccd041cd74` has identical
tested tree `550b8b21d753a5dc33e84848a391a76c5b2c6052`. Post-merge base-image run
`42113725968` passed. Main `42113725983` hit Windows EPERM in
`scripts/preview-watch.mjs:130` during the fixture's source-directory rename;
all Rust tests had passed. The follow-up adds Windows-only, bounded retries for
EPERM/EACCES/EBUSY on the two fixture renames; other errors still fail immediately.
Windows/hosted final-head CI must validate this follow-up before merge.

Local validation: all **471 Rust workspace tests passed**, with the two real
hosted deployment tests deliberately ignored locally and selected explicitly
in CI. Final CLI/unit and CI-generator rechecks cover updated provider-token
provisioning. Warnings-denied workspace Clippy, Rust formatting, Node syntax,
workflow actionlint (including an emitted Cloudflare workflow), and strict/deep
Zellij documentation checks passed (55 routes, 23 examples). No local Docker or
browser process was started. Final-head hosted/native CI and merge remain pending.

Implementation commit **`8898a30`** contains the Cloudflare adapter, provider
resolution/CI changes, matching documentation and bounded Windows fixture rename
retry. Local evidence: `/private/tmp/zellij-cloudflare-workspace.log` (471 passed),
`zellij-cloudflare-final-tests.log` (40 CLI unit + 16 workflow tests passed),
`zellij-cloudflare-clippy.log`, and `zellij-cloudflare-docs.log`.
Final-head GitHub checks, hosted export acceptance and merge are pending.

Cloudflare follow-up **`6cd0517`** rejects API deployments marked `is_skipped`,
even if their stage otherwise appears successful. The protocol fixture covers
this case; all 13 local Cloudflare tests, Clippy and strict docs pass. The CLI
overview now explicitly lists both static providers. Initial PR #81 CI
`42118339455` on `8f2f327` had 11 passing jobs and no failures when this final
correction was prepared; it does not validate this follow-up. Final-head CI
and live Cloudflare account acceptance remain pending.

### 2026-11-05 — Vercel prebuilt static delivery

PR #81 (Cloudflare Pages) merged as `9cde253e29a50e3bb2ea7b90355f274be569e629`
after all 16 checks passed on `b1bf1dac48cd19541e976d8a3408cb62ea6570ed` in
run `42119403884`. Tested and merged trees match
`c3d659a4c13a245292d69a370f44fecde9380137`. Post-merge base-image run
`42123170846` passed; main `42123170594` was still running with no failures
when this entry was prepared. Cloudflare live-account acceptance remains pending.

Current source adds `zel vercel` / `zel vc`, `deploy vercel`, saved `publish`,
`ship --provider vercel`, and `ci --deploy vercel`. Destination precedence is
explicit `--site` / `--site-id`, `VERCEL_PROJECT_ID`, then same-provider
`publish.site`. IDs (`prj_…`) and project names work. Provider changes do not
reuse another provider's saved destination or environment ID. The shared saved
site deserializer now enforces string typing, with destination syntax checked
by the selected provider when Site loads; Netlify validation stays strict.

The native adapter lives in `cli/crates/zellij-cli/src/vercel.rs`. It uses
`VERCEL_TOKEN` and optional `VERCEL_TEAM_ID`, checks an existing project before
building, then uses the shared temporary static export builder. Portable output
requires no external runtime; hosted output retains container requirements and
URL/feature overrides. Both preserve source files and prior output. No Vercel
CLI, stored login, account/project creation, configuration/DNS changes or remote
source build is involved. Default deployments are previews; `--prod` is explicit
and ship never persists that choice. Production waits for alias assignment.

The protocol uploads SHA-1-addressed assets to `/v2/files`, then creates exactly
one `/v13/deployments?prebuilt=1` request containing `.vercel/output/static/*`
and generated Build Output API v3 configuration. Content hashes deduplicate
uploads; source/config files and Zellij's ownership marker are excluded. The
routing configuration retains literal `.html` links and adds existing-file
fallbacks for hosted clean paths and directory indexes. Full deployment polling
checks ID, project, target environment, failure/alias status and HTTPS deployment
hostname before reporting readiness. Provider bodies/tokens stay out of errors,
redirects are refused and request/deployment budgets are bounded. Creation is
never automatically retried; uncertain results require dashboard inspection.

Initial limits are 15,000 static files / 100 MiB total; provider plan limits may
be lower. Links/junctions/special files, reserved Vercel config/output paths and
ambiguous control/#/?/backslash paths fail before uploads. Unicode/space/percent
filenames remain intact. Server functions, middleware, custom Vercel rules and
live-account acceptance are outside this milestone. Native publication remains
separate; published rc.1 does not contain these changes.

GitHub generation uses only the Vercel token and project/team variables or secrets
on the publish step; PR verification has no deployment credentials. Explicit
`--provision` copies only `VERCEL_TOKEN` via stdin after writing the workflow.
A compatible rc.2-or-later pin is required, without claiming it is published.
CLI docs, current status and unpublished release notes describe setup and limits.

Official protocol references inspected read-only:
[REST deployment](https://vercel.com/docs/rest-api/deployments/create-a-new-deployment),
[file uploads](https://vercel.com/docs/rest-api/deployments/upload-deployment-files),
[project preflight](https://vercel.com/docs/rest-api/projects/find-a-project-by-id-or-name),
[prebuilt query in the official client](https://github.com/vercel/vercel/blob/main/packages/client/src/utils/query-string.ts),
[Build Output configuration](https://vercel.com/docs/build-output-api/configuration).
Authenticated response fields were cross-checked against Vercel's OpenAPI schema.

Local tests cover protocol/identity/target selection, uploads and duplicates,
credential redaction, redirects, deadlines, failures, output preservation, CLI
aliases/provider isolation, ship retention and workflow secret provisioning.
CI explicitly selects a real hosted-export/mock-upload test, including base paths,
assets and failed-build preservation. Final local workspace checks and final-head
GitHub acceptance are pending at this entry. No local Docker or browser ran.

Vercel implementation commit **`28ea6f7`** passed all **487 local Rust workspace
tests**, with three real hosted-provider tests deliberately ignored locally and
selected explicitly in CI. Warnings-denied Clippy, formatting, strict/deep docs
(55 routes / 23 examples), workflow actionlint and an emitted Vercel workflow
passed. Evidence: `/private/tmp/zellij-vercel-workspace.log`,
`zellij-vercel-clippy.log`, `zellij-vercel-docs.log`, and generated fixture
`/private/tmp/zellij-vercel-ci-ba1_mb0e/.github/workflows/docs.yml`.
Final-head GitHub acceptance and merge remain pending. No local Docker/browser
or live provider deployment ran.

PR #82 initial run `42126186780` exposed a Windows-only fixture setup error:
`bad?.html` is not a legal Windows filename, so the test failed before calling
Zel's validation. Fix **`4475d66`** uses `bad#.html`, legal on all supported OSes
but rejected by the same upload-path rule. No assertion or Windows test is
skipped. All 11 local Vercel tests pass after the correction (one real hosted
test remains CI-only); formatting passes. Log:
`/private/tmp/zellij-pr82-windows.log`, with local recheck in
`/private/tmp/zellij-vercel-final-tests.log`. Final-head CI is still required;
the initial run does not validate this correction.


### 2026-11-05 — GitHub Pages branch publication

Current source adds `deploy github-pages`, `github-pages` / `ghp`, saved static
publication, import-and-ship and GitHub workflow generation. The destination is
an existing GitHub.com `owner/repository`; `--site` / `--site-id` overrides
`GITHUB_PAGES_REPOSITORY`, then same-provider `publish.site`. Authentication uses
only `GITHUB_PAGES_TOKEN`, a dedicated token with Contents and Pages write access.
No gh-login discovery, implicit GITHUB_TOKEN selection or external Git process is
used for deployment. Generation still uses Git for repository discovery and
explicit provisioning uses gh stdin, as with the earlier adapters.

This adapter has no preview mode and requires `--prod`, even for a dry-run plan;
CI generation accepts `--prod` or saved `ci.production: true`. Ship saves provider
and destination but never production intent or credentials. The destination must
already use Pages branch publishing (`build_type: legacy`) from `gh-pages /`;
the repository cannot be archived or have gh-pages as its default/source branch.
No repository, branch, site, DNS, protection or settings changes are performed.

The entire gh-pages root is replaced by a static snapshot. Git trees use no base
tree, commits retain the old head as their parent, and the branch is updated with
force:false after another repository/Pages/ref check. Concurrent changes fail.
The command explicitly requests a Pages build and accepts success only for its
published commit, then rechecks configuration/ref. Stale success, missing build,
failed/superseded build and timeouts cannot report success. A failed request can
leave Git objects or a published commit; mutations are never automatically
retried, resumed or rolled back. Ten minutes bounds the upload/publish phase;
individual requests/connects retain 60/15-second limits and reject redirects.
Response bodies/token values are omitted from errors.

Hosted deployments derive invocation defaults for basePath and site origin from
the Pages URL, overriding authored defaults without modifying source. An explicit
prefix must equal or be below the site's mount. Upload strips that mount once,
so /repository links have branch-root files rather than a doubled path. Explicit
site origin remains an SEO override. Portable output stays relative. CNAME comes
from the existing API configuration; conflicting exports fail. .nojekyll is
added and .zellij-output excluded. Git/workflow/Jekyll configuration, links and
special files are rejected. Limits: 5,000 static files, 25 MiB/file, 100 MiB total,
200 distinct binary/large uploads, 6 MiB tree JSON. Small UTF-8 files are inlined
within a 2 MiB budget; other assets use Git SHA-1 blobs and base64 byte upload.

Official protocol references inspected: [Pages API](https://docs.github.com/en/rest/pages/pages),
[publishing sources](https://docs.github.com/en/pages/getting-started-with-github-pages/configuring-a-publishing-source-for-your-github-pages-site),
[Git trees](https://docs.github.com/en/rest/git/trees),
[Git references](https://docs.github.com/en/rest/git/refs),
[Git blobs](https://docs.github.com/en/rest/git/blobs), and
[Git commits](https://docs.github.com/en/rest/git/commits).
Tests use local API fixtures; a real hosted export test is explicitly selected
by CI using its already-running engine image. Local Docker and browser execution
remain excluded. Live Pages account acceptance is pending; no live site or branch
was deployed. Broader provider work includes S3, Fly and Railway; GitLab/CircleCI,
wizard and richer imports remain separate. Native rc.2 and production Netlify
migration remain unpublished.

Prior milestone closure: PR #82 merged as `c8b485c567ecf108bb30771039ad317b75950b8b`
after all 16 checks passed on `b12af984d6bfdc220b8c4e06ad6598fff38c183f` in
`42126815665`; tested/merged tree `c79bd527ff19ccb198ca5eeb8b0bec0589f43c0f` matches.
Post-merge base-image `42130753789` and main CI `42130753939` both passed.
PR #81 post-merge `42123170594` and `42123170846` also both passed. The published
native candidate remains v0.2.0-rc.1; no release action was taken this milestone.


Local verification: all **505 Rust workspace tests passed**, with four real
hosted-provider tests intentionally ignored locally and explicitly selected in
CI. The final CLI/import/workflow recheck passed after credential-environment
isolation and production-before-conversion assertions. Warnings-denied Clippy,
Rust formatting, strict/deep docs (55 routes / 23 examples), workflow actionlint
and an emitted Pages workflow passed. Evidence: `/private/tmp/zellij-pages-workspace.log`,
`zellij-pages-final-tests.log`, `zellij-pages-clippy.log`, `zellij-pages-docs.log`,
and `/private/tmp/zellij-pages-ci-zfwwsu3_/.github/workflows/docs.yml`.
Final-head GitHub native/hosted checks and merge remain pending. No live provider
publication, local Docker or browser process was started.


GitHub Pages implementation commit **`b72e851`** contains the adapter, CLI/CI
integration, regression tests and matching docs. Local verification above covers
that implementation; final-head native/hosted CI and live account acceptance
remain pending. The subsequent context-only commit records this source identity.


PR #83 run `42135062241` exposed a Windows response-read deadline diagnostic:
expiration while reading the HTTP body was reported as a generic read failure.
Fix **`5dc95ec`** recognizes both reqwest's wrapped I/O timeout and the operation
deadline, retaining response/token redaction. A regression delays the response
body after headers. The original stale/missing-build timeout assertions remain;
no Windows test or assertion was disabled. All 13 local Pages tests passed
(one hosted test remains CI-only), with warnings-denied Clippy and formatting.
Distinct local Rust coverage is now 506 tests. Evidence:
`/private/tmp/zellij-pr83-windows.log` and `zellij-pages-timeout-tests.log`.
The first run cannot validate this correction; final-head CI and merge remain
pending, as does live GitHub Pages account acceptance.

### 2026-11-05 — GitHub Pages milestone closure and S3 implementation

PR #83 merged as **`e25d8d9c45e959c61da229fa8d3b2911021ab2f5`** after all
16 checks passed on final head `b11bdd697076a0005423da4967ddc887005c6db1`
in run `42135824730`. Tested and merged tree:
`c872806f12719cf8d366991514c56d837adf8b70`. Attempt 1 completed 15 checks;
the remaining Linux job stalled downloading Ubuntu packages at about 45 kB/s.
Only that unfinished job was cancelled and retried on the unchanged head;
attempt 2 passed. This was infrastructure delay, not a disabled assertion.
Post-merge base-image run `42139995450` and main CI `42139995418` both passed.
Evidence: `/private/tmp/zellij-pr83-premerge.json`, `zellij-pr83-ci.json`,
`zellij-pr83-merge.json` and the final PR description.

Current work adds native AWS S3 website uploads to `deploy s3`, shortcut `s3`,
saved publication, ship and GitHub CI generation. Destination is an existing
commercial-region general-purpose bucket, optionally with an ASCII prefix.
`--prod` is mandatory even for plans; CI may use saved `ci.production: true`.
Environment credentials, `AWS_REGION` and `S3_EXPECTED_BUCKET_OWNER` are required.
Temporary access keys require `AWS_SESSION_TOKEN`. No AWS CLI/profile/metadata
credential lookup or custom endpoint is used. Secrets never enter configuration
or argv. Explicit CI provisioning validates all keys before writing, then sends
required and supplied optional secrets individually over stdin; partial failures
retain the workflow and may have set some secrets.

The native SigV4 adapter checks bucket region, expected owner and website setup
before building. It requires index.html with no redirects/routing rules. It
builds once into temporary output and retains source/previous local output.
Portable keys receive the destination prefix; hosted invocation defaults derive
from that prefix and the website origin. Hosted files already contain the prefix
and are uploaded without duplication. Non-index HTML also gets extensionless
copies for clean links; collisions fail before uploads.

All key ETags/missing states are captured before conditional PUTs. Assets upload
first, then HTML, selected index last. SHA-256 receipts are verified. Configuration
is rechecked before and after upload. S3 publication remains non-atomic; failures
can leave partial changes. There is no automatic retry, rollback, prune, bucket
configuration, public-access/policy/ACL change, DNS or CloudFront setup. Old keys
remain. Website URLs use HTTP; uploads use HTTPS. Success means objects uploaded,
not that public website access was verified. Limits: 5,000 objects, 25 MiB/file,
100 MiB total including clean-route copies, 1,024-byte keys, 512-byte destination
prefix. Redirects are refused, provider bodies redacted, responses bounded to
256 KiB, with 60/15-second request/connect limits and a 15-minute upload budget.

Docs, command help, inventory, contract, current status and draft release notes
are updated together. Tests cover published AWS signing vectors, byte/path
preservation, sessions, conditional writes, failed receipts, drift, quotas,
preflight-before-build, source preservation and explicit multi-secret provisioning.
The hosted real-export fixture is selected only in GitHub CI. No local Docker,
Playwright or live AWS mutation was run. Live AWS/Cloudflare/Vercel/Pages acceptance,
Fly/Railway, other CI systems, wizard and advanced imports remain separate work.
Native rc.2 remains unpublished pending explicit release authorization. Netlify's
hosting-comment hydration issue and production migration remain pending.

Local verification: **528 distinct Rust tests passed** (524 in the full workspace
run, plus four added regression cases in the final focused runs). The final CLI
binary suite passed 78 tests with five real hosted-provider tests intentionally
ignored locally and explicitly selected in CI. The final delivery/import/workflow
suites passed. Warnings-denied Clippy, Rust formatting, strict/deep documentation
(55 routes / 23 examples), repository actionlint and an emitted S3 workflow passed.
Evidence: `/private/tmp/zellij-s3-workspace.log`, `zellij-s3-final-tests.log`,
`zellij-s3-bin-final.log`, `zellij-s3-clippy.log`, `zellij-s3-docs.log`, and
`/private/var/folders/1p/q48lnmt15932t129b48z_mrr0000gn/T/zellij-s3-ci-gjue2ymx/.github/workflows/docs.yml`.
Final-head hosted/native CI and merge remain pending, as does live AWS acceptance.

S3 implementation commit **`db164b5`** contains the adapter, CLI/CI integration,
regression tests and matching documentation. Local verification above covers this
implementation; final-head native/hosted CI and live account acceptance remain
pending. This subsequent context-only commit records the source identity.


### 2026-11-05 — Fly image deployment to existing apps

Current source implements `fly`, `deploy fly`, saved publication, import-and-ship
and GitHub workflow generation. `--kind static|next` selects the image; saved
`publish.image` is supported only for Fly deployments. Static is the default.
Explicit `--prod` is mandatory. App selection uses explicit site, FLY_APP, then
same-provider saved site. FLY_ORG guards organization identity; FLY_API_TOKEN
supports one raw or scheme-prefixed token. Credentials are never saved in YAML.

The native Machines API adapter requires 1–10 already-started, healthy web
Machines, HTTP services on port 3000, autostop disabled, and no additional process
groups, mounts, injected files or entrypoint overrides. It preserves full config
and changes only the image. Existing shared image builders produce small static
or Next wrappers for linux/amd64; no local runtime is installed or started.
The selected runtime must already be authenticated to registry.fly.io. Unique
tags and a validated repository digest identify the image for deployment.

Preflight precedes building. After one build/push, identity, membership and
versions are rechecked. All leases are acquired before updates, with optimistic
version guards. Sequential updates wait for the new instance, matching digest,
and fresh passing checks. Final state is checked before releasing leases.
Five-minute rollout budget, ten-minute leases, bounded/redacted HTTP responses,
no redirects, retries or automatic rollback. Failures may leave a partial rollout
and retained images. This does not create/configure apps, Machines, networking,
secrets, scaling or Fly release records; fly.toml is not applied. Public DNS/TLS
and reachability are not acceptance claims.

GitHub PR jobs build the selected image without credentials; authorized publish
events log in with stdin, deploy once and log out. Provisioning remains explicit.
Live Fly account acceptance is pending. Shared real static/Next image acceptance
runs in existing CI; protocol/fake-client fixtures do not prove live Fly behavior.
Railway, other CI systems, wizard and advanced imports remain separate work.
Rc.2 is unpublished pending explicit release authorization; production Netlify
migration remains blocked by the reported hosting-comment hydration issue.

Protocol references: Fly [Machines API](https://docs.fly.io/machines/api/machines-resource),
[Apps API](https://docs.fly.io/machines/api/apps-resource),
[registry](https://docs.fly.io/blueprints/using-the-fly-docker-registry), and
[official SDK token normalization](https://github.com/superfly/fly-go/blob/master/tokens/tokens.go).
See [user setup and exact limits](examples/zellij-docs/content/02-cli/deploy.mdx#fly).
Final source identity and verification are recorded in the subsequent entry.


PR #84 closure: S3 merged as `9e3dd79e48c2cd256e16f9e24085fcf5a45b78c4`
after all 16 checks passed on `d141aa8ed44fff8643601e2f1a93b21053cc9be2`
in run `42150590098`, attempt 2. Only the Ubuntu job was retried after slow
Azure package downloads exhausted its time limit; assertions were unchanged.
Tested/merged tree: `4d85791532ca267dff6a1bf028307f7196c1924e`.
Both post-merge workflows passed: main `42155106031`, base image `42155106146`.

Fly local verification: **541 distinct Rust tests passed** (538 in the full
workspace run, plus three added cases in final focused runs). Final CLI binary,
delivery, import and workflow suites passed 149 tests, with five real hosted
provider fixtures intentionally ignored locally and selected in existing CI.
Warnings-denied workspace Clippy, formatting, strict/deep documentation (55 routes,
23 examples), repository actionlint and an emitted Fly workflow passed.
Evidence: `/private/tmp/zellij-fly-workspace.log`, `zellij-fly-final-tests.log`,
`zellij-fly-clippy.log`, `zellij-fly-docs.log`, and generated workflow
`/private/var/folders/1p/q48lnmt15932t129b48z_mrr0000gn/T/zellij-fly-ci-22txs7eu/.github/workflows/docs.yml`.
No local Docker/browser or live Fly mutation ran. Final-head cross-platform CI
and merge remain pending; live provider acceptance remains a separate limitation.


Fly implementation commit **`a9b650c`** contains the native adapter, static/Next
selection, delivery/import/CI wiring, protocol and fake-tool regression tests,
and matching documentation. Local verification above covers this source.
Final-head GitHub CI and merge remain pending; live Fly account acceptance is
not implied by loopback fixtures or shared image tests.


Fly compatibility follow-up **`ecad3ca`** accepts the API's default init object
with `tty: false` while retaining rejection of custom command/TTY overrides.
Update receipts may omit `updated_at`; readiness then uses the subsequent GET's
timestamp only after verifying the new instance ID and exact requested config.
The baseline fixture now includes Fly's default init shape, and a new regression
covers sparse update receipts. All 10 Fly tests and warnings-denied workspace
Clippy pass. Distinct local Rust coverage is **542** tests. Evidence:
`/private/tmp/zellij-fly-compat-tests.log`, `zellij-fly-compat-clippy.log`.
PR #85's first run on `1b272fa` cannot validate this correction; the final head
requires its own complete CI run before merge. Live Fly acceptance remains pending.


### 2026-11-05 — Fly PR #85 closure

Fly implementation `a9b650c` and API compatibility correction `ecad3ca` merged
in PR #85 as `30a02721d6d703adffe9ccc93ecc5b4e9b7cf5a0`, after all 16 checks
passed on final head `101e5d4dd196e593a460b1375f32fa2b0bac2347` in run
`42160670652`. Tested/merged tree: `9ab7fa9981ded274ba132aa3a568d6e3fe419d44`.
Both post-merge workflows passed: main `42163948890`, base image `42163948889`.
Native releases remain rc.1 and stable 0.1.3; no rc.2 tag/release was created.
Live Fly account acceptance remains pending.


### 2026-11-05 — Railway image deployment to existing services

Current source adds `railway` / `rw`, `deploy railway`, saved `pub`, imported
`ship`, and GitHub CI delivery. Static/Next images use the existing compact
builders with linux/amd64. `image.name` or `--tag` is required; supported remote
registries are GHCR, Docker Hub, GitLab, Quay and MCR. The image is built once,
pushed, and its unambiguous SHA-256 RepoDigest is used as the service source.
Bounded digest inspection is shared with Fly; Docker Hub's abbreviated receipt
names are normalized without admitting other repositories or ambiguous hashes.

The native public GraphQL adapter uses an environment-scoped project token in
`RAILWAY_TOKEN` and requires expected `RAILWAY_PROJECT_ID` and
`RAILWAY_ENVIRONMENT_ID`. Destination precedence is explicit service UUID,
`RAILWAY_SERVICE_ID`, then same-provider `publish.site`. `--prod` is mandatory;
`--kind` overrides same-provider `publish.image`, defaulting to static. Ship
requires an explicit tag and saves the registry/image/provider selection without
credentials or implicit production permission. CI gets the tag from image.name.

Preflight verifies token scope, project/environment/service identity, no staged
changes or attached volumes, a successful existing image deployment, startup
healthcheck, PORT=3000, compatible HOSTNAME, and no custom start/pre-deploy/cron,
repository config or sleeping. A complete volume inventory is required (at most
100 per environment). It rechecks the service after building, patches only the
environment-scoped image source, calls serviceInstanceDeployV2 once, validates
the returned deployment identity, polls within five minutes, and rechecks the
source/settings/latest deployment before success. Responses are capped, redirects
refused and GraphQL partial errors rejected without printing response details.
No automatic retry, rollback, resource creation or variable write occurs.

The API update/deploy sequence has no atomic lock; concurrent external writes
can still race. Failed operations can leave a new source or deployment running;
the registry image remains. SUCCESS reports Railway startup deployment status,
not independent attestation of the running digest, public DNS/TLS, reachability,
or continuing health. Live Railway account acceptance is pending.

GitHub verification jobs build the selected image without credentials. Publish
jobs log in to the explicit registry (GHCR GITHUB_TOKEN/packages:write, otherwise
REGISTRY_USERNAME/PASSWORD) and scope the Railway token to the deploy step.
Railway needs its own pull access; local registry login is not sufficient.
Release availability is not checked by generation; rc.2 remains unpublished.

Protocol references: Railway [authentication](https://docs.railway.com/integrations/api),
[services](https://docs.railway.com/integrations/api/manage-services),
[healthchecks](https://docs.railway.com/deployments/healthchecks), and
[official CLI schema at bd4d490](https://github.com/railwayapp/cli/blob/bd4d490c11d084c39d7cdea114708a9995d1ccc0/src/gql/schema.json).
The experimental environmentApplyChangeSet endpoint is not used.
See [setup and exact limits](examples/zellij-docs/content/02-cli/deploy.mdx#railway).
Final source identity and verification will be recorded below.


Railway local verification: **556 Rust tests passed**, with five existing real
hosted-provider fixtures intentionally ignored locally and selected in CI.
The final Railway import/workflow fixtures also passed after isolating their
processes from inherited Railway environment variables. Warnings-denied workspace
Clippy, formatting, strict/deep documentation (55 routes, 23 examples), repository
actionlint and an actual emitted Railway workflow passed. Evidence:
`/private/tmp/zellij-railway-workspace.log`, `zellij-railway-final-focused.log`,
`zellij-railway-clippy.log`, `zellij-railway-docs.log`; the workflow path is saved
in `/private/tmp/zellij-railway-generated-workflow-path`.
No local Docker/browser or live provider mutation ran. Final-head GitHub CI and
merge remain pending; live Railway account acceptance remains separate.


Railway implementation commit **`3ace654`** contains the adapter, shared digest
handling, CLI/CI/import wiring, regression fixtures and matching user docs.
The local verification above covers this implementation. Final-head GitHub CI
and merge remain pending; no live Railway acceptance or native release is implied.


The final Railway timeout fixture leaves five seconds for loopback requests and
uses the polling interval to exhaust the remaining budget, avoiding a 100 ms
Windows scheduler race. Production deadlines are unchanged. All nine Railway
protocol tests passed again; evidence: `/private/tmp/zellij-railway-protocol.log`.
PR #86 must validate its final head before merge.

## 2026-11-06 — Railway closure and additional CI systems

Railway PR #86 merged as **`b24b3dcca70a8f3ff0dd5ac7696dfdd9f1d40927`** after
all 16 checks passed in run **42178970612**, attempt 2, on final head
`fdf2f6a2e01fe74c0b4080c64d0c5389ea7be9d9`. The tested and merged source trees
both equal `5918d4e341abfe3e566f887d218642809ff8d912`. The initial Windows ARM
PowerShell 7 installer job hit an internal CLR error after its package and
Windows PowerShell checks passed; the targeted unchanged-head retry passed.
No assertions were relaxed. Both post-merge workflows now passed: main
**42183211689** (16 jobs), base image **42183211386** (13 jobs). The authenticated
release inventory still has native `zellij-v0.2.0-rc.1` as the newest candidate
in `mylife-inc/releases`. No rc.2 was published.

Current implementation adds GitLab and CircleCI standalone configuration
renderers alongside the existing GitHub generator. Shared validation/preparation
retains build overrides, explicit configuration files, version gates, provider
production rules and image-kind selection. CLI defaults to saved `ci.system`
(GitHub when absent); `--system` and `ci init <system>` override it. CircleCI
accepts `circle` and `circleci`; conflicting explicit choices fail before writes.
Default paths are `.gitlab-ci.yml` and `.circleci/config.yml`; custom filenames
write `.gitlab/ci/<file>` and `.circleci/<file>`. Users select custom configuration
paths in the CI project settings. No existing root configurations are merged.
`--force` only replaces the selected file; symlink/junction parents are rejected.
Remote `ci run` and credential `--provision` remain explicitly GitHub-only.

GitLab uses native Linux SaaS runners, Ubuntu 24.04, and Docker-in-Docker with
TLS only for hosted/image/manifest jobs. Temp files live under the runner's
shared builds directory for bind-mount visibility. Self-managed runners need
privileged Docker executors, shared build/TLS volumes and appropriate tags.
Publication requires protected refs and a dedicated generated environment;
users must scope protected credentials to that environment and disable their
availability in MR pipelines. Built-in GitLab job credentials still exist.
CircleCI targets Cloud + GitHub App/OAuth integration with native Ubuntu machine
executors. Publication jobs alone attach the `zellij-publish` context (override
with `ci.publish-context`). Users must configure context project/user/ref
restrictions; workflow conditions alone are not credential authorization.
No CircleCI Server or other VCS integration is claimed.

Both support the existing seven deployment adapters and explicit static/Next
registry images. GHCR uses user-supplied registry credentials outside GitHub;
GitLab's built-in registry pair is a fallback only for its matching registry
and only when neither custom credential is set. Docker passwords go through
stdin, using an isolated temporary Docker configuration. `platform: all` runs
native AMD64/ARM64 builds, transfers validated immutable digest receipts via
GitLab artifacts/CircleCI workspaces and publishes the final manifest only
after both succeed. No staging-tag cleanup is automatic. Schedules are accepted
as events but must be created remotely by the user; requested cron expressions
are printed rather than silently treated as provisioned. These generators do
not add path filters.

Primary references checked on 2026-11-06:
- [GitLab DinD](https://docs.gitlab.com/ci/docker/docker_in_docker/) and
  [native hosted runners](https://docs.gitlab.com/ci/runners/hosted_runners/linux/).
- [GitLab variable protection](https://docs.gitlab.com/ci/variables/).
- [CircleCI configuration](https://circleci.com/docs/reference/configuration-reference/),
  [pipeline events](https://circleci.com/docs/guides/orchestrate/pipeline-variables/)
  and [contexts](https://circleci.com/docs/guides/security/contexts/).
The generator uses `pipeline.event.name`, not the deprecated trigger-source
value. Docker DinD is pinned to 29.9.3, listed in the
[official Engine release notes](https://docs.docker.com/engine/release-notes/29/).

User documentation lives in `examples/zellij-docs/content/02-cli/ci.mdx`; command
help, shortcuts, runtime guidance and the current status table are synchronized.
GitHub-hosted native image acceptance now replays generated GitLab/CircleCI
publication scripts with the candidate binary and a disposable registry, then
replays manifest scripts against immutable AMD64/ARM64 receipts. This exercises
commands on GitHub runners, not an actual GitLab/CircleCI control plane. Live
provider-runner acceptance needs account access and remains separate. Native
release publication still requires its own explicit authorization.

Local verification: **564 Rust tests passed**, five existing container-backed
provider fixtures intentionally ignored locally and selected by CI. The final
CLI parser adjustment also passed all 11 parser tests and all 29 CI workflow
integration tests. Warnings-denied workspace Clippy, formatting, strict/deep
docs (55 routes, 23 examples), JavaScript syntax and repository actionlint pass.
Emitted portable, hosted, combined-image and Netlify configurations pass both
services' official JSON schemas; the same four GitHub cases pass actionlint.
Schema validation is structural, not live pipeline acceptance.
Evidence: `/private/tmp/zellij-external-ci-workspace.log`,
`zellij-external-ci-final-args.log`, `zellij-external-ci-final-workflows.log`,
`zellij-external-ci-clippy.log`, `zellij-external-ci-docs.log`; generated examples
are located by `/private/tmp/zellij-external-ci-generated-path`.
No local Docker/browser or live provider mutation ran. Final-head GitHub image
acceptance and merge are pending at this checkpoint.

Implementation commit **`9d3ba34`** contains the GitLab/CircleCI generators,
shared command preparation, CLI/config selection, regression fixtures, native
GitHub replay harness and user documentation. The local verification above
covers this source; the following context-only commit records its identity.
Final-head GitHub acceptance and merge remain pending.

## 2026-11-06 — CI generator closure and section authoring

GitLab/CircleCI PR #87 merged as `b590cb2dd1309601525f663285dfc98c1dc00406`
after all 16 checks passed on `3eb218812b33163d228cf0ef92bbb5d576081079` in
run `42201229972` (attempt 1). Tested and merged tree:
`7398f45e8e1e30797fba9c70436f5617efeac79d`. Both post-merge workflows passed:
main `42205322276` (16 jobs), base image `42205322053` (13 jobs). The native
AMD64/ARM64 jobs replayed generated provider publication and manifest scripts
against a disposable registry; actual GitLab/CircleCI runner acceptance remains
separate. The published native candidate remains rc.1; no rc.2 was published.

Current implementation adds `zel new section [type] [file]` / `zel ns`, with
`-i`/`--interactive`, repeatable scalar `--set FIELD=VALUE`, bounded list
`--count FIELD=N`, and pricing `--tiers`/`--highlight` shortcuts. Omitting the
type launches the wizard only when stdin and stderr are terminals; pipelines
without a type fail promptly. Explicit interactive mode also accepts piped
answers. Prompts use stderr; generated YAML uses stdout unless a destination is
supplied. Enter keeps starter values; `:q`, EOF and interruption write nothing.
Existing files and output links remain protected. Original no-option templates
are byte-for-byte unchanged.

The engine owns section schemas, templates and new shared cross-field metadata
(`sections/authoring-rules.ts`), exported into the native embedded schema.
`zellij-scaffold::authoring::Section` edits fresh templates and validates the
current schema vocabulary, exclusive media alternatives and matrix row widths.
Unknown future constraints fail closed until supported. Optional parent objects
are seeded; newly added list entries use fresh defaults rather than duplicating
authored content. Dotted array indexes are zero-based; pricing highlight numbers
are one-based. Conflicting assignments fail before writes, and wizard resizing
cannot discard explicit flags. Plain string scalars preserve Unicode, quotes,
newlines and YAML-looking values through the existing YAML string encoder.
`zellij-cli/src/authoring.rs` owns prompting and output handling.

This is section authoring, not a whole-site/page wizard or arbitrary YAML edit
command. Validation checks section shape, not filesystem assets, route targets
or renderer prerequisites. Review TODOs and run site validation afterward.
User docs, short-command reference, CLI spec and current status are updated;
the previous section heading anchor remains available for old bookmarks.

Local verification: **574 Rust tests passed**, five existing hosted-provider
fixtures remain CI-only; **424 engine tests passed**. Workspace Clippy with
denied warnings, formatting, TypeScript checks, strict/deep docs (55 routes,
23 examples), actionlint and embedded-schema comparison pass. The new Node
acceptance script validates all 23 authored types against actual engine Zod
schemas, including optional unions, media choices, arrays, pricing, Unicode and
piped prompting. GitHub's Windows/macOS/Linux portable jobs now run it alongside
native authoring integration tests. A local terminal session confirmed automatic
wizard selection, output creation and Ctrl-C cancellation without an output
directory. No local Docker/browser or live provider mutation ran.
Evidence: `/private/tmp/zellij-authoring-{workspace,engine-full,parity,typecheck,
clippy,docs}.log`. Final-head cross-platform CI and merge remain pending.

Implementation commit **`ff42301adc435f697b8bb729f2aceb91e11f8eca`** contains the section wizard, native
validation, shared engine metadata, cross-platform regression checks and user
documentation. The verification above covers this source; the following
context-only commit records its identity. Final-head GitHub CI is pending.

## 2026-11-06 — Section authoring closure and full starter

Section authoring PR #88 merged as `5078b51e8e01d326fc2b391da4381a00099bf126`
after all 16 checks passed on `fe69695b152c09e3a55a426bed91ebe8f7b6fadc` in
run `42210390950` (attempt 1). Tested and merged tree:
`578c819fd9c049a6d68c38e55111d1a151a90629`. The PR body records the verified
merge and source-tree equality. Post-merge base image `42214445273` passed all
13 jobs. Main run `42214445315` passed 15/16: Windows ARM installer job
`127880840580` failed upgrading a separate installation to the higher fixture
version with Windows error 32 after download. The log did not identify the
individual file operation or lock holder; it is not evidence of a schema or
section-authoring failure, nor a clean post-merge run.

Current source adds bounded retries for Windows sharing/lock violations (32/33)
during candidate permission changes, staged-archive removal and the Windows
rename/restore sequence. Each operation has a five-second budget; other errors
fail immediately. Errors identify the operation. Cancellation is checked before
the old executable moves, then commit/restoration complete without cancellation.
Existing checksum/version/architecture checks and backup retention are unchanged.
The Windows x64/ARM64 package jobs now run tests with actual exclusive file
handles as well as the existing real self-upgrade fixture. Unit cases also cover
non-Windows behavior, permanent errors, exhausted budgets and cancellation.
The exact original lock holder remains unknown; this handles the observed error
class rather than claiming a particular antivirus/process diagnosis.
References: [Windows error codes](https://learn.microsoft.com/en-us/windows/win32/debug/system-error-codes--0-499-)
and [file move/replacement sharing requirements](https://learn.microsoft.com/en-us/windows/win32/fileio/moving-and-replacing-files).

`zel new docs --full` / `zel nd --full` now produces the normal home page, an
Examples menu/catalogue, and a linked page/YAML example for each of the 23
engine-registered types. `--guide` adds the existing guide normally; `--minimal`
conflicts before writes. The default stays `docs/zellij`; custom destinations,
names and themes retain the existing convention. Examples have live sample
copy and commented optional fields, a small authored SVG and a 1,691-byte
generated MP4. No remote media, new dependencies or local runtime are required
to create the files. Hosted checking/building/previewing still needs its normal
runtime. The site has 25 routes, or 26 with the guide; its sample video is a short
gray clip intended for replacement.

`zellij-scaffold::configured_full_docs` uses the embedded engine templates and
native section authoring validator. `Scaffold::files()` retains its text-only
iterator; the new `entries()` includes binary media. Text and binary destinations
share one preflight/staged write and output receipt, including no-overwrite and
symlink handling. Full generation is an explicit entry point; existing plain,
minimal, app and project scaffolds retain their current output. Local sample
media and provenance are under `cli/crates/zellij-scaffold/assets/`.

The new integration fixtures exercise default/custom directories, Unicode names,
all types, catalogue links, exact asset bytes, optional guides, invalid flag
combinations, force behavior and binary collision preflight. The Node boundary
script runs strict/deep engine checks on both guide variants on Windows/macOS/
Linux. Its separate CI-only `--build` mode exports the guide variant and verifies
all 26 routes, rendered section markers, catalogue links and exact media bytes.
User creation/shortcut/upgrade docs, CLI spec and status are synchronized.
No local Docker/browser or live provider mutation ran. Native rc.2 remains
unpublished; the actual release channel still has candidate rc.1 and stable
v0.1.3. Final-head CI and merge remain pending at this checkpoint.

Local verification: **579 Rust tests passed**, with five existing hosted-provider
fixtures reserved for CI. The final cancellation assertion also passed all seven
upgrade unit tests. Warnings-denied workspace Clippy and the final CLI Clippy
check pass, as do formatting, actionlint, strict/deep documentation (55 routes,
23 examples) and the full-starter engine boundary script. The first unprivileged
CLI-unit attempt lacked loopback socket permission; the full suite above ran
with loopback access and passed. Actual Windows lock tests and hosted full-site export still need final-head
GitHub acceptance.
Evidence: `/private/tmp/zellij-full-{workspace,starter-tests,upgrade-tests,
engine-check,clippy,final-clippy,docs}.log`.

Windows upgrade recovery source: **`608e5e8db1f7c77380493063f7fb73c467c9fe83`**.
It adds operation-specific diagnostics, bounded sharing-lock retries and native
Windows file-lock regression coverage. Final-head GitHub acceptance is pending.

Full starter implementation: **`54f601dc9701a11ef3d2819874b26c96950cac9f`**.
The preceding local verification covers this source and Windows recovery
`608e5e8`; this context-only follow-up records their identities. Final-head
GitHub acceptance and merge remain pending.


## 2026-11-06 — Full starter closure and site-aware doctor

PR #89 merged as **`479eac4b899266311af84a5b4f8f55ea1b881356`** after all 16
checks passed on **`80ba76c5f58f191b33e5dc2f8e753966f105e4e6`** in
[42267735005](https://github.com/mylife-inc/Zellij/actions/runs/42267735005),
attempt 1 without retries. Tested and merged tree:
`30afe44fade65556abe6607e12446f4c6705b6fc`. Windows x64 and ARM64 each passed
all eight upgrade tests (including real exclusive file handles) and the actual
higher-version self-upgrade fixture. The hosted full starter exported 26 routes,
with section markers, catalogue links and exact SVG/MP4 bytes verified.
Implementation identities remain `608e5e8` for Windows recovery and `54f601d`
for the full starter; the preceding entries remain the pre-CI history.

Post-merge base-image run `42272858763` passed all 13 jobs. Main run
`42272858943` passed 15 jobs but Ubuntu portable job `128081619870` reached its
30-minute limit while apt downloaded Playwright system dependencies. Native
fixture tests had passed; the log ended during a libflite download. This is not
a successful browser acceptance result. A targeted retry of that job at the
unchanged merge commit is in progress; no code or timeout was changed.

Current doctor work moves preflight reporting into
`cli/crates/zellij-cli/src/doctor.rs`. `doctor [site]` accepts `--site-root`,
`--config` and `--cwd` through the shared argument resolver. Implicit selection
uses cwd/`docs/zellij` config precedence, then legacy hosted discovery. Without
a site it retains runtime-only diagnostics; arbitrary Markdown folders require
an explicit path. Invalid selected configs never trigger fallback discovery.

Reports include root/config/layout/renderer/content/default output/preview port.
Portable sites validate native settings and skip all container probes. Hosted
and runtime-only diagnostics retain bounded runtime discovery, endpoint
redaction and image inspection without pulling or running containers. Resolved
site information remains visible when runtime availability fails.

`doctor --json` emits schema version 1, `ok`, `exit_code`, binary version/target,
site, runtime, image and error. Handled failures remain a single stdout report;
argument/working-directory errors precede it and use ordinary stderr. Verbose
tracing is stderr-only. Runtime states distinguish not-required, not-checked,
available and unavailable; missing image/version metadata does not fail doctor.

This is configuration/runtime preflight, not content, output ownership or
writability, port availability, credential or remote-provider validation. Use
`check` for content. Nothing is written and no service is started. Regression
coverage uses isolated fake clients and covers discovery, custom configs, Unicode
paths, raw/legacy layouts, missing/invalid selection, read-only behavior, hosted
failure reports, runtime-only operation, and endpoint/probe-output redaction.
The command reference, global selector/JSON docs and CLI status are updated.
Final local suite, final-head CI and merge remain pending.


PR #89 post-merge follow-up: main run `42272858943`, attempt 2, completed
successfully with all 16 jobs at unchanged `479eac4`. The targeted Ubuntu retry
passed browser acceptance. The first attempt's dependency-download timeout is
retained above; no source change was needed.

Doctor local verification: **586 Rust tests passed**, five existing hosted-provider
fixtures ignored locally and reserved for CI. All 16 runtime integration tests,
including the seven added doctor cases, passed. Warnings-denied workspace Clippy,
formatting, actionlint and strict/deep documentation checks (55 routes, 23 examples)
passed. Evidence: `/private/tmp/zellij-doctor-{workspace,clippy,docs}.log`.
Final-head cross-platform acceptance and merge remain pending.

Doctor implementation: **`1c2091b922e136b70118ba59549eedb00d2c35c0`**. The local verification above
covers this source. Final-head GitHub CI and merge remain pending.


Doctor CI follow-up: run `42297958566` on `8ee56f9` caught a missing legacy
`/cli/doctor#exit-code` target after the heading was renamed to describe JSON.
The documentation now retains that ID as an explicit legacy anchor. Existing
content-preservation assertions remain unchanged. This failure was a real
bookmark regression; the corrected final head must pass its own complete CI.

Bookmark fix: **`22277623932783505d555da2a693bbb9c8ed05f7`**. Strict/deep documentation checks pass
(55 routes, 23 examples); the browser preservation check runs only in CI.


## 2026-11-06 — Doctor closure and Sphinx table imports

PR #90 merged as **`e6733067bb49ee7f831896f2659a2dfa7300c203`** after all 16
checks passed on **`0fb8da2fa97df6cbbdf035fe8b407780aed93b28`** in
[42298531414](https://github.com/mylife-inc/Zellij/actions/runs/42298531414),
attempt 1. Tested and merged tree: `28d7416e410217080dfc31793d6d2d25b9f83a80`.
Doctor cases passed on Windows/macOS/Linux; all six native packages, both image
architectures/manifests and broader hosted/browser tests passed. Preservation
retained 46 original routes, 282 original heading IDs, 82 compatibility mappings
and 29 byte-identical images. Source `1c2091b` and bookmark fix `2227762` are
identified in the preceding history. Post-merge main run `42303348733` passed
all 16 jobs; base-image run `42303348916` passed all 13. Both were refreshed
before this milestone. The current repository release inventory lists rc.1;
no rc.2 publication was attempted.

Current work adds rectangular Sphinx grid/simple tables and optional captions
through `.. table::`, without Python or a source build. The table parser in
`cli/crates/zellij-convert/src/sphinx/tables.rs` validates border geometry and
uses `unicode-width` (locked 0.2.3) for wide/combining character columns. Simple
rows preserve first-column continuation semantics, paragraph breaks and the
unbounded final column. Grid cells preserve their common margin and multiline
content. Both forms retain headerless tables and multiple header rows.

The RST block model now has a Table node; cells reuse the existing supported
block parser, sharing a 16-level table/admonition depth limit. Rendering emits
semantic HTML table/head/body/caption/cell tags with Markdown inside cells.
This preserves supported inline markup, lists, admonitions, code, images and
references through the existing MDX, link rewriting, asset, heading and search
pipeline. Source HTML stays literal. Tables have at most 128 columns, 10000
rows, 100000 cells and a 16384-column border. No filesystem links, outside reads,
source execution or network fetching are introduced.

Spanning cells, malformed geometry, list/csv tables, table directive options,
and headings/section labels/toctrees inside cells still fail with the source
filename before output is created. Existing unsupported RST content remains
unsupported inside cells; no lossy flattening is attempted. The conversion
reference and CLI status describe these limits. A small missing comma in the
JSON command reference is corrected too.

The shared `scripts/fixtures/sphinx-tables.rst` is exercised by native CLI
integration tests and `scripts/check-conversion.mjs`. Node-only MDX rendering
and deep checks passed locally across all six importers; the Sphinx fixture
verifies semantic headers, Unicode, captions, blocks, escaped source HTML,
code containing pipes/braces, links, exact assets and searchable cell text.
The same script checks the real hosted export in GitHub CI. Local Rust tests
cover malformed/spanning/unsafe forms, boundaries, dry run and asset skip.
No local Docker or browser ran. Final local suite and cross-platform CI remain
pending; live providers, native release and Netlify migration remain separate.

References: [Docutils table syntax](https://docutils.sourceforge.io/docs/ref/rst/restructuredtext.html#tables),
[table directive](https://docutils.sourceforge.io/docs/ref/rst/directives.html#table),
[Unicode width implementation](https://github.com/unicode-rs/unicode-width).


Local verification: the workspace suite passed **592 tests**, with five existing
hosted-provider fixtures reserved for CI. After tightening the depth guard for
empty cells, all **15 Sphinx CLI integration tests** passed, including the new
16-level acceptance/17-level rejection case (593 distinct native cases across
these checks). Final warnings-denied workspace Clippy, formatting, actionlint,
strict/deep documentation (55 routes, 23 examples), and Node-only MDX rendering
across all six converters passed. Logs are under
`/private/tmp/zellij-tables-{workspace,import-final,clippy-final,engine-final,docs}.log`.

CI additionally runs `scripts/check-imported-tables.mjs` from conversion export
acceptance. It checks browser hydration, semantic header/body cells, local image
loading and document navigation under the deployment prefix, capturing console
and runtime errors and rejecting external requests. The existing Chromium setup
step moves before conversion acceptance; it is not duplicated. This helper is
not imported by local server-render-only checks. Final-head CI and merge remain
pending.

Sphinx table implementation: **`2b06d934c3cc7d77456298368d930c492dfb3d5d`**. Local evidence above covers
this source, including the final empty-cell depth guard. Final-head GitHub
acceptance and merge remain pending.


Sphinx table browser-fixture follow-up: initial PR #91 run `42314567470` on
`926f76a` failed because the new fixture server rejected `/products/docs`
without a trailing slash. The hosted Sphinx export had succeeded; browser
navigation requested the prefix root and the unchanged HTTP/console assertions
correctly caught its 404. Fix **`aa409e6393344f505655afbfcbb796e571c52554`**
serves that exact root while retaining prefix containment and adds an explicit
root-response check before browser launch. No product parser or error assertion
changed. Node syntax and diff checks passed locally; no local browser or Docker
ran. The corrected final head requires a new complete CI run before merge.


Sphinx table browser acceptance then exposed a product bug in run `42315268885`
on `e910edb`: the common MDX link renderer passed copied download URLs through
Next Link. Its page-data prefetch requested a nonexistent path such as
`/_zellij/assets/imported/docs/notes.txt/__next._tree.txt`, producing HTTP 404.
The table semantics, local image and document navigation assertions had already
passed; the final no-browser-errors assertion caught the download prefetch.

Fix **`ba8a68ecb9704823706967956c8962acc6adfeed`** renders URLs in the reserved
`/_zellij/assets/` namespace as ordinary anchors with the site's basePath.
Page links retain Next navigation; external/fragment handling and source query,
fragment and encoded filename text are retained. This applies to all converters.
Node-only regression checks cover root/prefixed sites and all six importer
fixtures; the CI browser test now clicks the Notes link, requires a native
navigation response and compares the exact bytes. It still rejects HTTP,
console and runtime errors. The [Next Link documentation](https://nextjs.org/docs/app/api-reference/components/link#prefetch)
confirms its production page-data prefetch behavior; the failure log establishes
this reproduction.

All 426 engine tests, workspace typechecks, Node-only conversion rendering/deep
checks for all six importers, strict/deep docs (55 routes, 23 examples), script
syntax and diff checks passed locally. Evidence:
`/private/tmp/zellij-tables-{engine-tests,typecheck,asset-render}.log`.
No local browser or Docker ran. Final-head CI remains required before merge.


## 2026-11-06 — Sphinx table closure and list/CSV directives

PR #91 merged as **`6b3b4b4c25d5f20c5daefd6ffce0c6760f7e5d80`** after all 16
checks passed on **`eda6ab6328d1cf978f7a9a862d08c7b70a56afe0`** in
[42316324469](https://github.com/mylife-inc/Zellij/actions/runs/42316324469),
attempt 1. Tested and merged tree: `6af059aa6f40eaec855351210567f5b0db692eef`.
The table cases passed on Windows/macOS/Linux; browser acceptance confirmed
hydration, semantic cells, image loading, document navigation and native asset
navigation with exact bytes. All six native packages, both image architectures,
combined manifests and broader hosted/browser tests passed. Preservation retained
46 original routes, 282 heading IDs, 82 compatibility mappings and 29 images.
The parser (`2b06d93`), fixture root fix (`aa409e6`) and shared download fix
(`ba8a68e`) are recorded above with their original failures and verification.

Post-merge base-image run `42319859200` passed. Main `42319859257` completed
15 successful jobs; its showcase job `128241510788` hit the 20-minute limit
while downloading Ubuntu browser dependencies, before application/browser tests.
The log ends in the `playwright install --with-deps` apt download, with no test
failure. A targeted retry on unchanged `6b3b4b4` was requested; its outcome is
pending. Native rc.1 remains the newest published candidate; no release or
production migration was attempted.

Current work adds `list-table` and inline `csv-table` to the Sphinx converter.
`cli/crates/zellij-convert/src/sphinx/table_directives.rs` parses the two-level
row/cell bullet structure and strict quoted CSV without new dependencies or
source execution. Both support captions, `header-rows`, `stub-columns` and
`widths: auto`; CSV adds supplemental/multiline `header`, literal Unicode
`delim`/`quote`, `delim: space` and `keepspace`. Short CSV rows are padded,
while list rows must be uniform. A body row and a data column must remain after
headers/stubs. The existing 128-column/10000-row/100000-cell and 16-level limits
apply. The common RST table node now carries a stub count; body row headings
render as `<th scope="row">` through the existing MDX pipeline.

Cell bodies retain the existing supported RST constructs, reference resolution,
asset containment and search behavior. Option parsing now distinguishes a field
from a role at the start of CSV content, preserving `:download:` cells, and reads
CSV header continuation lines without treating them as body data. External CSV
file/URL/encoding options, custom escape rules, character-code notation, numeric
widths, alignment/classes/names, spanning cells and unsupported nested constructs
remain explicit errors before output. No CSV external input is read or fetched.

The shared fixture now contains eight tables, four header groups, 17 column
headings and seven row headings. Native regression coverage checks retained
content, literal code, links, exact assets, dry runs, skip mode, invalid directives,
unsafe paths and the list-table nesting bound. CI's existing browser helper now
checks row-header roles, supplemental headers, padded cells and a second local
image while keeping navigation/download and error assertions. Documentation,
CLI status and the contract describe the accepted syntax and remaining limits.
Local final verification and final-head CI remain pending.

References: [Docutils list-table](https://docutils.sourceforge.io/docs/ref/rst/directives.html#list-table),
[CSV table](https://docutils.sourceforge.io/docs/ref/rst/directives.html#csv-table),
and the [upstream parser](https://github.com/docutils/docutils/blob/master/docutils/docutils/parsers/rst/directives/tables.py).


List/CSV implementation: **`bc70c45296d01c5b7369142db610c1da2fba15f1`**.
Local verification passed **600 Rust workspace tests**, with five existing
hosted-provider fixtures reserved for CI. All 18 Sphinx CLI cases and four new
table-directive unit cases pass. Warnings-denied workspace Clippy, formatting,
strict/deep docs (55 routes, 23 examples), script syntax, and Node-only rendering,
deep checks and searchable-cell assertions across all six converters pass.
Evidence: `/private/tmp/zellij-directives-{workspace,clippy,import,unit,docs,search}.log`.
No local Docker/browser or live provider operation ran. Final-head CI and merge
remain pending; main's unchanged-head showcase retry is still running.


### 2026-11-06 — Directive boundaries and prior main-CI closure

PR #91 main run `42319859257`, attempt 2, now has all 16 jobs successful on
unchanged `6b3b4b4`. Retried showcase job `128249568949` passed. This closes
the earlier dependency-download timeout without a source change; the original
failure and its limited verification are retained above.

Review of the new Sphinx directive parser found that leading blank lines were
being discarded before option parsing. A code block beginning with `:header:`
could therefore be rejected as if it contained directive options. CSV header
continuations could also consume the separator and misclassify following data.
Fix **`6b695697e7b95c476225a29e8aca079d64f6dd6d`** retains that boundary,
requires a blank line between options and content, and rejects unsupported
multiline table captions instead of silently importing them as CSV rows.

Regression coverage checks literal option/include-looking code, multiline CSV
headers followed by inline roles, missing separators and multiline captions.
The shared native/rendered/browser fixture includes the literal field text.
All 19 Sphinx CLI cases and 75 converter tests pass, as do warnings-denied
Clippy, formatting, strict/deep docs (55 routes, 23 examples), script syntax
and Node-only rendering/search checks for all six importers. The final complete
Rust workspace suite passed **601 tests**, with five existing hosted fixtures
reserved for CI. Fresh final-head CI and merge remain pending.

Evidence: `/private/tmp/zellij-directives-final-{import,convert,clippy,docs}.log`
and `/private/tmp/zellij-directives-exact-{workspace,render}.log`. No local Docker/browser
or production/provider mutation ran. PR #92's earlier CI head does not validate
this follow-up; final-head CI is required before merge.

The option/content distinction follows the [Docutils directive block parser](https://github.com/docutils/docutils/blob/master/docutils/docutils/parsers/rst/states.py).


## 2026-11-06 — Sphinx directive closure and nested lists

PR #92 merged as **`0c4d79e80a232b403282d56cb56f2e7841c5fdc5`** after all
16 checks passed on **`2eeb9c6d967274484f7f30015ed1662859c70427`** in
[42324258639](https://github.com/mylife-inc/Zellij/actions/runs/42324258639),
attempt 1. Tested and merged tree: `fe9232807c1325496e1a97b9f99eb479950dd56e`.
All 19 Unix/18 Windows Sphinx tests passed. Hosted browser acceptance covered
semantic tables, column/row headings, local images, navigation and exact native
asset downloads. Documentation preservation retained 46 original routes, 282
heading IDs, 82 compatibility mappings and 29 byte-identical images. Both
post-merge workflows passed on unchanged `0c4d79e`, attempt 1: main
`42326727478` (all 16 jobs) and base-image `42326727358`. Native rc.1 remains
the latest published candidate; no native release or production migration ran.

Nested-list source **`27889f655594059eb528383f3abd23b68acafb2a`** adds
`sphinx/lists.rs` and a recursive `Block::List`. RST list items now retain
nesting, wrapped/multiple paragraphs, literal code, tables, admonitions, images
and references. Supported markers are ASCII and standard Unicode bullets,
decimal `1.`/`1)`/`(1)` forms and corresponding `#` auto-enumerators. Explicit
start values and sequence/style restarts become separate semantic HTML lists.
Empty items are retained. Unicode bullets use display-column indentation,
not UTF-8 byte length. Rendering shares the existing reference/asset and search
pipeline through convert/cv, migrate/mig and ship. No dependency was added.

Malformed indentation or block separators, alphabetic/Roman enumeration,
out-of-range counters and unsupported nested constructs fail before output.
Decimal counters must fit 0–2147483647; each list has at most 10000 items.
Lists share the existing table/admonition depth limit of 16, including empty
items. Existing external CSV, merged-cell, extension and source-execution limits
remain. The conversion reference and CLI contract/status now describe this scope.

A shared native/render/browser fixture covers six unordered lists, five numbered
lists, numbering restarts, Unicode, an empty item, literal code, lists in tables
and a table in a list. The hosted CI helper checks the actual parent/child DOM
structure, numbering, nested media, document navigation and native downloads
while retaining hydration and console/HTTP error assertions.

Local tests caught and fixed a Unicode-marker byte/column mismatch before commit.
The first additional-table rendering assertion assumed a caption had no nested
paragraph element; the corrected assertion checks the retained caption text and
table count, with browser containment checks. Neither failure was waived.
Focused validation passed 23 Sphinx CLI cases and 78 converter unit tests,
warnings-denied workspace Clippy, formatting, strict/deep docs (55 routes,
23 examples), Node syntax and Node-only rendering/search checks for all six
importers. The complete Rust workspace suite passed **608 tests**, with five
existing hosted-provider fixtures reserved for CI. Final-head CI and merge
remain pending. Evidence: `/private/tmp/zellij-lists-{workspace,import,unit,clippy,docs,render-verified}.log`.
No local Docker, browser, GUI or live provider operation ran.

Reference: [Docutils list syntax and indentation](https://docutils.sourceforge.io/docs/ref/rst/restructuredtext.html#bullet-lists).


Nested-list hosted assertion follow-up: initial PR #93 run `42333097022` on
`967df6a` failed in CLI job `128283842248` after the Sphinx export succeeded.
The new list-count assertion scanned the complete exported page (12 unordered
lists, including navigation/sidebar), while its expected six lists describe
only the authored article. The earlier MDX-only render had correctly passed.

Fix **`eb4b83154d3082412bb37d4f178338b6d915e7d5`** requires and selects the
exported `<article>` before counting. Expected counts, numbering, content and
all browser assertions remain unchanged; no product code changed. Node syntax
and diff checks pass. No local browser or Docker ran. The initial run did not
reach the new nested-list browser assertions, so a fresh full final-head CI run
is required before merge.


## 2026-11-06 — Nested-list CI closure

PR #93 merged as **`2ed992d977929ad15509618fcbf163b287139f54`** after
all 16 checks passed on **`1b29b4a52df87c8ad6663f4f7c506aa87d253bbd`**
in [42333764306](https://github.com/mylife-inc/Zellij/actions/runs/42333764306),
attempt 1. The tested and merged tree was
`42ccffe4af1735815377a5840e2b0cfffde6362f`. Hosted browser acceptance passed
nested list structure, numbering, literal code, local images, document navigation,
exact native downloads and hydration/error checks. Documentation preservation
retained 46 original routes, 282 heading IDs, 82 compatibility mappings and
29 byte-identical images. All six native packages and both Linux image
architectures/combined manifests passed. Both post-merge workflows passed on
unchanged `2ed992d`, attempt 1: main `42336073179` (all 16 jobs) and base-image
`42336073120` (all 13 jobs). The initial assertion-scope failure and correction
remain recorded above. Native rc.1 remains the latest published candidate.


## 2026-11-06 — Sphinx alphabetic and Roman numbering

Source **`67b313f955bb7c9ab3d4af71381ab3fef5d019a0`** adds lower/upper alphabetic and Roman list styles to the
shared Sphinx importer. All three punctuation forms and matching automatic
markers retain starts, case, nesting and the existing block/reference pipeline
through native HTML list attributes. Context resolves ambiguous I/V/etc.;
wrapped alphabetic/Roman prose remains prose. Canonical Roman values are
limited to the browser counter range 1–3999 and alphabetic values to 1–26.
Malformed/mixed-case Roman markers and automatic overflow fail before output.
No Python, source evaluation or new dependency was introduced.

The same change fixes a decimal-list bug: an explicit enumerator after `#`
must start a new list even if its value happens to be consecutive, matching
Docutils. It also corrects obsolete overview/quickstart claims that tables and
nested lists are unsupported, and clarifies that the remaining automatic
numbering limitation concerns toctrees. Existing containment, shared depth and
item bounds remain. Other Sphinx extensions, spanning cells, substitutions and
source-execution limits remain documented; this does not complete all imports.

Local focused validation passed **24 Sphinx CLI tests** and **81 converter unit
tests**, warnings-denied workspace Clippy, formatting, strict/deep documentation
validation (55 routes, 23 examples) and Node-only rendering/search for all six
importers. A fixture heading underline initially failed validation; the fixture
was corrected and rerun successfully. The final conversion-notice wording also
passed the 24 Sphinx CLI cases. Full workspace verification passed **612 tests**,
with five existing hosted-provider fixtures reserved for CI. Final-head
hosted/native CI and merge remain pending.

The shared native/render/browser fixture now contains nine ordered lists,
including all four letter/Roman styles. CI additionally checks computed browser
styles on lists and items, nested typed-list containment and a section link,
while retaining existing media/navigation/hydration/error assertions. No local
Docker, browser or GUI ran, and no native release or production migration ran.
Evidence: `/private/tmp/zellij-numbering-{unit-final,import-final,message-check,clippy,docs,render,workspace}.log`.
References: [Docutils enumeration](https://docutils.sourceforge.io/docs/ref/rst/restructuredtext.html#enumerated-lists),
[sequence parser](https://github.com/docutils/docutils/blob/master/docutils/docutils/parsers/rst/states.py),
[Roman browser counters](https://www.w3.org/TR/css-counter-styles-3/#lower-roman).


Numbering browser-selector follow-up: PR #94 run `42338890305` on `863d301`
failed in CLI job `128302002257` after all importer exports and the preexisting
Sphinx browser assertions succeeded. The new `ol[type="a"]` selector matched
both `a` and `A` lists because HTML type attribute selectors compare without
case, causing a count of two where one exact-case sequence was expected.

Fix **`496f97984810ed87b382eef536cc7b17f216c95b`** reads the four typed lists' exact DOM attribute values
and compares them in JavaScript. It retains exact starts, direct item counts,
computed list/item styles and parent-child type assertions, and also requires
exactly four typed lists. Product output and Rust code are unchanged. Node
syntax and diff checks pass. The earlier run did not reach the new computed
style assertions, so fresh final-head CI remains required. No local browser
or Docker ran. The 612-test Rust result remains valid for unchanged Rust code.


Numbering display follow-up: PR #94 run `42339535816` on `00be13b` failed in
CLI job `128303947326`. Exact-case DOM selection succeeded, but the first
alphabetic list and its item computed to `list-style-type: none`. Inspection
identified Tailwind Preflight's general list reset, which also hides ordinary
prose bullet/decimal markers. The semantic attributes alone did not establish
visible numbering; the initial implementation's browser acceptance was incomplete.

Fix **`21c663b69a1c2219b2123915f863837020c84b5b`** adds a prose-only `list-style-type: revert-layer` rule inside
Tailwind's `base` layer. This undoes that layer's reset while preserving HTML
presentation hints (including case-sensitive ol types) and native nested bullet
styles. A plain `revert` would also discard author presentation hints. Navigation
and component styling retain their existing rules. CI now also checks ordinary
bullet/decimal styles and typed items' `display: list-item`; none of the original
computed-style expectations were relaxed.

Local Tailwind/PostCSS compilation passed with the rule retained in the built
stylesheet (`/private/tmp/zellij-numbering-styles.css`), as did script syntax and
diff checks. Rust code remains unchanged and its 612-test suite remains valid.
The stylesheet change requires fresh final-head hosted browser acceptance.
No local browser/Docker ran. Final CI and merge remain pending.
References: [Tailwind Preflight](https://tailwindcss.com/docs/preflight),
[CSS presentation-hint origins](https://www.w3.org/TR/css-cascade-5/#preshint),
[layer rollback](https://www.w3.org/TR/css-cascade-5/#defaulting-keywords).


## 2026-11-06 — Numbering and prose-marker CI closure

PR #94 merged as **`3f14b99b533e9e63b6f25d40c8ce00c2a2cb4fc2`** after all
16 checks passed on **`fcbaa6b91e477a177f604a710f5c867228cbe9ff`** in
[42340198858](https://github.com/mylife-inc/Zellij/actions/runs/42340198858),
attempt 1. Tested/merged tree: `f13f7b7394f52a28a4d28ce84f27709404df6db8`.
Hosted browser checks passed exact-case alphabetic/Roman styles, ordinary
bullet/decimal markers, list-item display, nesting, media, navigation and native
downloads with no hydration errors. Sphinx tests passed 24 cases on Unix and
23 on Windows. All six native packages and both image architectures passed.
Documentation retained 46 original routes, 282 heading IDs, 82 compatibility
mappings and 29 byte-identical images. Both post-merge workflows passed on
unchanged `3f14b99`, attempt 1: main `42342065734` (all 16 jobs), base-image
`42342065709` (all 13 jobs). Earlier selector/display failures remain recorded.
Native rc.1 remains the latest published candidate; no release or production
migration ran.


## 2026-11-06 — Sphinx reusable text and images

Source **`958b87743468d47057a967e8e91654cad00fb787`** adds document-local
`replace`/`image` substitutions, forward/nested references and named substitution
hyperlinks to the shared Sphinx converter. Names normalize whitespace, prefer
exact case and otherwise require a unique lowercase match. Images retain alt
text and direct/named targets. URL targets ending in underscores remain URLs.
Literal `version`/`release` values come from configuration and can be overridden
by authored definitions. Code blocks, inline code and escaped pipes stay literal.
Existing reference/asset handling applies through convert/cv, migrate/mig and
ship, including copy/skip/dry-run containment checks. No source is executed.

All definitions in published documents are validated before output, including
unused ones. Missing/ambiguous names, duplicate definitions, cycles, nested links
and unsupported constructs fail. Bounds are 1000 authored definitions per page,
256-byte names, 64 KiB replacement paragraphs, and (for each validation/render
pass) depth 16, 10000 references and 4 MiB processed expansion text. Headings and
navigation labels still need plain text; global transforms, anonymous links,
date/unicode and unsupported options remain manual migration tasks. The current
conversion docs, quickstart, contract and CLI status describe this scope.

The shared MDX pipeline marks Markdown images within prose for inline display,
including linked images; standalone figures keep their block presentation.
This corrects the CSS reset's default block behavior for inline images without
adding browser JavaScript or dependencies. The native/render/browser fixture
covers formatted reusable text, nested and forward definitions, copied images,
section/download links, lists/table cells/captions, literal code and page scope.

Focused checks pass **28 Sphinx CLI cases**, **86 converter unit tests**,
**427 engine tests**, workspace type checking, warnings-denied Clippy,
formatting and engine package build. All six converters pass Node-only
render/search checks; strict/deep docs retain 55 routes and 23 examples.
The final-source full Rust workspace rerun is in progress; its result and
final-head hosted/native CI must be checked before merge.

Local regression work caught a discarded blank separator that could admit a
second replacement paragraph; parsing now rejects it. The first rendered-code
assertion assumed syntax highlighting left a directive as contiguous HTML text;
it now checks the code's retained text, with the same browser assertion. Clippy
also caught test-module placement, corrected before the final lint pass.
Evidence: `/private/tmp/zellij-substitutions-{import-exact,unit-final,engine-full,typecheck,package,clippy-final,render-exact,docs,workspace-final}.log`.
No local Docker/browser/GUI, native release or production/provider mutation ran.
References: [Docutils substitution definitions](https://docutils.sourceforge.io/docs/ref/rst/restructuredtext.html#substitution-definitions),
[replacement text](https://docutils.sourceforge.io/docs/ref/rst/directives.html#replacement-text).


#### 2026-11-06 — Substitution final-suite and inline-layout review

The final Rust workspace run on `1041870` completed successfully: **621 passed,
5 ignored**, across 51 suites. Source **`26d2d01`** then corrected the shared
image-layout helper after a new SSR regression reproduced missing inline classes
in compact Markdown lists and GFM table cells. It recognizes text in the same
inline flow, excluding nested blocks, so image-only figures stay blocks even
when a nested list contains text. The expanded regression passes, as do all
**427 engine tests** and workspace type checking. No Rust source changed after
the full Rust result. Earlier passing checks remain recorded above.

Evidence: `/private/tmp/zellij-substitutions-workspace-final.log`,
`zellij-substitutions-image-review.log` (reproduced failure),
`zellij-substitutions-image-review-fixed.log`,
`zellij-substitutions-engine-review.log`, and
`zellij-substitutions-typecheck-review.log`. Initial PR #95 CI `42348865898`
on `1041870` does not validate this follow-up. Final-head hosted/native
acceptance and merge remain pending. No local browser/container was run.


## 2026-11-06 — Substitution merge and post-merge follow-up

PR #95 merged as **`4dcb12c99dffd30d86f20fdeed9c609c465aa7f7`** after all
16 checks passed on **`d9b47a0e0659aa2c4097dded139b7a1c03a1f8ec`** in
[42349216986](https://github.com/mylife-inc/Zellij/actions/runs/42349216986),
attempt 1. Tested/merged tree: `f1a0108156e95189cc86b5be30ac2759a2997a6a`.
Hosted browser acceptance passed substitution text/markup, inline/standalone
images, named/image navigation, literal code and exact native downloads, with
no captured hydration errors. Sphinx cases passed 28 on Unix and 27 on Windows.
All six native packages, both image architectures and combined manifests passed.
Documentation retained all 46 original routes, 282 heading IDs, 82 compatibility
mappings and 29 byte-identical images. Local main matched the tested tree.

Post-merge base-image run `42351594623`, job `128340710709`, built maths/amd64
successfully but failed actions/upload-artifact during FinalizeArtifact with
`ECONNRESET`. A targeted failed-job retry on unchanged `4dcb12c` passed all 13
base-image jobs (attempt 2), including published-image acceptance. Main run
`42351594650` has 15 passing jobs and CLI acceptance still running at this
checkpoint. This was an artifact transport failure; no product fix or relaxed
assertion was needed. Native rc.1 remains the latest published candidate.


## 2026-11-06 — Native site-creation wizard

Source **`b556b88`** adds opt-in `new portable` / `init -i` and `new docs` /
`nd -i` creation. Native prompts choose name, embedded-catalogue theme, starter
and a separate hosted guide. Explicit flags skip matching questions; ordinary
commands retain their defaults and never prompt. Names/themes validate before
writes, fixing hosted creation's acceptance of invalid values. The shared
line-oriented prompt remains on stderr, including under quiet; EOF/:q cancels.

A read-only scaffold preflight checks the root/common configuration before
questions. The final plan checks every destination before writes; cancellation
with force leaves old content intact. Portable output still has no overwrite
flag. Existing templates generate the files, with no new dependency, client code,
installer, network or runtime. Fixed relative next-step suggestions also avoid
interpolating a destination path into a copied shell command. Docs, shortcuts,
CLI status and contract describe exact defaults, scope and remaining page/app
wizard work.

All **629 Rust workspace tests pass** (5 existing hosted-provider cases ignored
locally and selected in CI), including 8 site-wizard cases on Unix and all prior
section/starter/alias tests. Tests cover every starter/guide combination, byte
equivalence with flags, literal Unicode metadata, CRLF and numeric selections,
invalid answers, quiet prompts, cancellation at every question, force preservation,
symlink containment and native portable builds without tools on PATH. The initial
new HTML assertion used `data-theme` instead of the established
`data-zellij-theme`; corrected to the actual contract before final verification.

Warnings-denied Clippy, formatting, workflow actionlint and script syntax pass.
Node-only engine checks validate full ordinary/wizard starters with all 23
section types and exact media; strict/deep project docs retain 55 routes and
23 examples. CI extends the existing Linux/macOS/Windows fixtures: portable
wizard starters are built, moved and viewed offline with/without JavaScript,
and a hosted wizard full starter exports all 26 routes. Final-head CI and merge
are pending. No local Docker/browser/GUI or live provider mutation ran.
Evidence: `/private/tmp/zellij-site-wizard-{focused-final,workspace,clippy,engine,docs}.log`.


### 2026-11-07 — Windows installer harness desktop-startup correction

Initial PR #96 CI `42353311383` on `306f16c` failed in Windows ARM package job
`128345889969`. The Windows PowerShell installer checks passed; the subsequent
`pwsh` invocation crashed with `Internal CLR error (0x80131506)` in
`TaskbarJumpList.CreateElevatedEntry`, on its background desktop initialization
thread. The failure was retained; the installer assertions were not relaxed.

Source **`4df0d31`** starts only automated PowerShell child processes with Node's
`windowsHide: true` in native package/installer fixtures. The PowerShell source
checks startup visibility before its JumpList COM calls; libuv maps this option
to `SW_HIDE`, so the correction targets that desktop-only path. The CLI and
console/signal fixture launch behavior is unchanged. Both Windows shells still
run the full checksum, malformed-archive, preservation, installed-binary and
upgrade checks. There is no catch-and-ignore or automatic retry around assertions.

Node syntax and diff checks pass. The 629-test Rust result remains valid for
unchanged Rust code; final-head Windows/native and full CI must pass before
merge. No local browser/container/GUI ran. References:
[PowerShell JumpList guard](https://github.com/PowerShell/PowerShell/blob/master/src/Microsoft.PowerShell.ConsoleHost/WindowsTaskbarJumpList/TaskbarJumpList.cs),
[libuv process startup](https://github.com/libuv/libuv/blob/v1.x/src/win/process.c).


### 2026-11-07 — Preserve hosted name compatibility in the site wizard

PR #96 run `42353774620` on `05a412e` passed 15 jobs, including all six
native packages. Windows ARM job `128347405063` passed both `powershell.exe`
and `pwsh` installer/checksum/archive/upgrade checks after `4df0d31`; its
console and signal assertions stayed unchanged. All three portable-platform
jobs passed wizard-generated standard/minimal offline navigation, assets,
search and no-JavaScript acceptance in Chromium and WebKit.

The CLI job `128347404872` failed at the existing scaffold-name acceptance:
new validation rejected a hosted `--name` containing newline/tab characters.
Those names were already supported through escaped YAML, so treating them as
invalid was a compatibility regression. Source **`65de2c8`** restores literal
hosted name values, including with `-i` when the name flag skips its question.
Interactive name answers and portable names still exclude control characters;
all names must be nonempty, and themes must be in the embedded catalogue.
The user reference and CLI contract now distinguish these cases. This corrects
the broader name-validation claim in the initial wizard entry above.

All 9 focused wizard tests pass, including the new literal-name/interactive
regression. The unchanged Node-only scaffold acceptance passes all 40 Rust
starters, 10 Node starters and 10 guide titles with exact metadata and rendered
links. Strict/deep project documentation validation passes 55 routes and
23 examples. Full Rust rerun and final-head hosted/native CI remain pending;
the earlier run cannot validate this correction or the hosted wizard export.
No local Docker/browser/GUI, native release or provider mutation ran.
Evidence: `/private/tmp/zellij-site-wizard-compat-{focused,scaffolds,docs}.log`.

PR #95's post-merge main run `42351594650` subsequently passed all 16 jobs on
unchanged `4dcb12c`, so both its post-merge workflows are now green. Its base
image workflow retained the previously recorded artifact-upload retry.


## 2026-11-07 — Site wizard merge and page authoring follow-up

PR #96 merged as **`0849faf348526ef8a27225f89fc01f0e032864ed`** after all
16 checks passed on `b98811bd894bf5eb82d79f78bba35877167e8870` in
[42356363457](https://github.com/mylife-inc/Zellij/actions/runs/42356363457),
attempt 1. Tested/merged tree: `136c390adae14417580ab323d44c778f192d0527`.
The final local suite passed 630 Rust tests (5 existing hosted cases selected
separately in CI), and CI passed all 427 engine tests. Hosted literal names
were restored by `65de2c8`; Windows installer startup was corrected by `4df0d31`.
All six native packages passed, as did wizard-generated portable browser/no-JS
acceptance on Linux/macOS/Windows and the 26-route hosted wizard export.
Documentation retained all 46 original routes, 282 heading IDs, 82 compatibility
mappings and 29 byte-identical images with no captured browser errors.

Post-merge base-image run `42357950120` passed all 13 jobs. Main run
`42357950211` has 14 passing jobs, with Windows portable and CLI acceptance still
running at this checkpoint; no failures. Native rc.1 remains the latest published
candidate. No native release or production/provider change was made.

Source **`dd7c625`** adds optional `new page -i` / `np -i`. It asks only for a
missing name and missing ordered section list, accepting exact catalogue names
or displayed numbers; Enter selects hero. Unknown/duplicate types retry, while
invalid supplied selections fail before questions. Ordinary commands keep their
required name, default hero, templates and printed registration report.
The shared Prompt stays on stderr, including with quiet; the pages.yaml report
stays on stdout. EOF/:q/interruption during questions writes no section files.
The entire writer plan still preflights collisions and containment before writes.
Authored pages.yaml/menu.yaml remain untouched; manual registration, TODO content
and placeholder media review are explicit in the docs. No dependency, runtime,
network or new renderer behavior was introduced.

All 8 focused page-wizard tests pass on Unix (7 portable to Windows), covering
flag equivalence, ordered references, Unicode/CRLF, numeric choices, defaults,
aliases, quiet reports, cancellation, late collisions, symlinks and global
site/config/cwd selection. The 7 existing alias and 10 safe-write cases pass.
Initial new test fixtures used the nonexistent type `features` instead of
`feature-grid` and an unsupported custom content-folder setting; corrected to
the established catalogue and content/ convention before the passing run.
Node-only engine acceptance validates a wizard page with all 23 types in order,
checks exact templates and preserves authored registration bytes. CI runs this
on all three native OSes and checks the ordered section markers in hosted HTML.
Warnings-denied Clippy, formatting, actionlint, diff checks and strict/deep docs
(55 routes, 23 examples) pass. Full Rust workspace rerun and final-head CI/merge
remain pending. No local Docker/browser/GUI ran.
Evidence: `/private/tmp/zellij-page-wizard-{focused,focused-fixed,engine,clippy,docs-final,workspace}.log`.
App wizards, arbitrary YAML editing, broader imports and live-provider acceptance
remain separate work; the published rc.1 does not include this feature.


## 2026-11-07 — Page wizard merge and app creation follow-up

PR #97 merged as **`c10c1e1f822bfe6649a2f7ef8715137ca9b6b23c`** after all
16 checks passed on `ad9e7672d3d3720fee1fc4e7227e374f826200e3` in
[42359044185](https://github.com/mylife-inc/Zellij/actions/runs/42359044185),
attempt 1. Tested/merged tree: `8ecc554b938d75b959a129e0a1d1222a961b1207`.
The local suite passed 638 Rust tests (5 existing hosted cases selected separately
in CI), and CI passed all 427 engine tests. All six native packages, both image
architectures and the combined manifests passed. The real hosted export rendered
the wizard page's 23 sections in their selected order. Documentation retained all
46 original routes, 282 heading IDs, 82 compatibility mappings and 29 byte-identical
images with no captured browser errors. Both post-merge workflows passed on their
first attempts: main `42361341195` (16 jobs), base image `42361341102` (13 jobs).
PR #96's main `42357950211` also completed successfully on its first attempt;
both of its post-merge workflows are green.

Source **`326c2b3`** adds optional `new app -i` / `na -i`. It asks only for missing
engine path, content mode/source, display name, dev/start port and optional
features. Existing hosted content stays referenced in place; the starter choice
creates docs/zellij inside the new wrapper. Flags skip their questions, numeric
choices are supported, and invalid answers retry. Standalone --features none
explicitly selects no optional dependencies. The shared single-choice Prompt
retains the site wizard's behavior and stderr/quiet semantics.

All planning is read-only. EOF/:q/interruption while answering writes no wrapper,
and the existing full-plan writer still enforces destination, overlap, symlink
and collision checks before writes. Explicit configuration and --cwd semantics
are preserved. No package installation or runtime/server starts during creation;
the engine must already be built. Existing app creation's nonempty-destination
error now says to choose another destination instead of suggesting unsupported
--force. Ordinary flags and generated wrapper templates retain their behavior.

All 12 app-scaffold tests pass on Unix (10 are portable to Windows), including
both content modes and all feature choices, byte equivalence with flags, inherited
ports, quiet prompts, Unicode/CRLF, numeric selections, explicit config/cwd,
cancellation at each question, collisions and symlink preservation. Existing
alias/site/page/section tests pass (42 focused cases total). One initial expected
feature-metadata order was corrected to the existing maths/diagrams order; product
ordering was unchanged. A wizard-generated starter with the real built engine
passes strict/deep validation without installing dependencies. Warnings-denied
Clippy, formatting, Node syntax and diff checks pass; strict/deep project docs
validate 55 routes and 23 examples. The final Rust workspace suite passed all
646 tests, with 5 existing hosted cases selected separately in CI.

CI's existing native app acceptance now installs and builds wizard-generated
starter/external wrappers and tests production browser navigation, reload, media
and search on Linux/macOS/Windows. Final-head CI and merge remain pending. No
local Docker/browser/GUI, native release or provider mutation ran. Published
rc.1 predates these wizards; arbitrary YAML editing, broader imports and live
provider acceptance remain separate work.
Evidence: `/private/tmp/zellij-app-wizard-{focused-fixed,final-focused,engine,clippy,docs,workspace-final}.log`.


## 2026-11-07 — App wizard closure and structured section authoring

PR #98 merged as **`2d7e4a58a16fb379290109f67950266e7fb982f9`** after all
16 checks passed on `268c9361ae12940cfd135f59a9e5d8deb980671e` in
[42390317045](https://github.com/mylife-inc/Zellij/actions/runs/42390317045),
attempt 1. Tested/merged tree: `b26ef496a637bc38bb171fd30f122ad6318fa3a4`.
The local suite passed 646 Rust tests (5 existing hosted cases selected separately
in CI), and CI passed 427 engine tests. Wizard-generated wrappers passed real
installation, production build, navigation, reload, media and search on Linux,
macOS and Windows. All six native packages, both image architectures and combined
manifests passed. Documentation retained all 46 original routes, 282 heading IDs,
82 compatibility mappings and 29 byte-identical images without captured browser
errors. Both post-merge workflows passed on attempt 1: main `42392744587`
(16 jobs), base image `42392744541` (13 jobs). Remote main and native releases were
refreshed before the follow-up; rc.1 remains the latest published candidate.

Source **`f6e2c7e`** extends new section / ns with repeatable --set-json FIELD=JSON
and --unset FIELD. Complete JSON values, including objects and lists, replace the
selected field after embedded-schema validation. Optional properties can be
removed without creating missing parents; required properties, list elements,
unknown fields and out-of-range indexes fail. Engine defaults still apply to
omitted settings. These options create new sections, not edits to existing YAML.

Processing order is counts, JSON replacements, scalar refinements, then removals.
A scalar/removal can refine a descendant of a JSON object/list. JSON replacements
cannot overlap; scalar JSON values cannot be converted to objects by a descendant
override. Duplicate paths, contradictory counts (now including --tiers), and
media selections that discard another supplied value fail before writing.
Complete tiers/tier-object values suppress the highlight question and conflict
with --highlight; their highlighted values can be supplied directly. The wizard
retains complete subtrees, refinements and omissions. Matrix and media rules,
new-file output, cancellation, symlink and collision protection remain in force.

All 12 section-authoring CLI tests pass on Unix (11 portable to Windows), covering
complete nested values, typed scalars, deterministic refinements/removals, invalid
JSON/schema/path cases, conflicts, quiet/cwd, cancellation and preservation.
The 7 library authoring cases and existing alias/site/page/app tests pass (48
focused CLI cases total). Node-only acceptance round-trips all 23 authored section
types through JSON and the engine schemas, including literal Unicode/multiline
text and explicit omissions. The expanded 25-route template fixture validates a
structured hero; CI additionally asserts its final headline, link and anchor in
hosted HTML alongside the unchanged 23-section wizard page.

Warnings-denied Clippy, formatting, Node syntax, actionlint and diff checks pass;
strict/deep project docs retain 55 routes and 23 examples. The complete Rust
workspace passed 655 tests, with 5 existing hosted cases selected separately in
CI. Final-head CI and merge remain pending. No local Docker/browser/GUI,
native release or live provider mutation ran. User docs, CLI contract, specification
and status distinguish new-section authoring from general existing-file editing.
Evidence: `/private/tmp/zellij-structured-section-{focused,focused-final,library-final,engine,templates,clippy-final,docs,workspace}.log`.


## 2026-11-07 — Structured section merge and Sphinx literal inclusions

PR #99 merged as **`6c01f4a30c9314a03132a8bda2f0fce6bd30cdb4`** after all
16 checks passed on `a3681ff5623b46253e3b5f73e48cc31c516c47c0` in
[42402060708](https://github.com/mylife-inc/Zellij/actions/runs/42402060708),
attempt 1. Tested/merged tree: `cea5d6fda64d9d7743a8c109b67e3135117d8111`.
Local main and origin/main were equal and clean before this follow-up. Both
post-merge workflows passed on attempt 1: main `42404134767` (16 jobs), base
image `42404134783` (13 jobs). The final local suite passed 655 Rust tests
(5 hosted cases selected separately in CI); CI passed 427 engine tests, all six
native packages, both image architectures and combined manifests. Documentation
retained 46 original routes, 282 heading IDs, 82 compatibility mappings and 29
byte-identical images without captured browser errors. Native rc.1 remains the
latest published candidate.

Source **`4cbbc7a6d10f4c97aae200dc43dd3122bf8c25f8`** adds Sphinx
`literalinclude` for local UTF-8 code/text files. The parser records explicit
include nodes; after navigation selects published documents, a bounded resolver
reads contained regular files and replaces those nodes with literal code blocks,
including nested lists/admonitions/table cells. No Python, included RST, remote
file or source code is executed. Inclusion alone does not publish a raw source
asset. File paths are document-relative or source-root-relative, with literal
Unicode/space/percent/hash filenames; hidden, excluded, symlinked and escaping
paths fail. Unpublished pages do not open their include targets.

Language/encoding, tab stops, start/end markers, ordered one-based line ranges
and numeric/common-prefix dedent are supported. Filters run in Sphinx order;
missing markers, out-of-range lines and non-whitespace dedents fail rather than
silently losing content. UTF-8 BOM handling follows the chosen encoding; line
endings normalize to LF. Bounds are 8 MiB/file, 32 MiB unique input and expanded
text, and 100,000 explicitly selected lines. Unsupported options (line numbers,
captions, pyobject, diff, prepend/append, etc.) fail before writes. Parsed includes,
MyST, extensions and untitled RST fragments remain unsupported. Sphinx's
[directive documentation](https://www.sphinx-doc.org/en/master/usage/restructuredtext/directives.html#directive-literalinclude)
and upstream directive source were inspected read-only to check filter semantics.

All 37 Sphinx CLI tests pass on Unix (35 on Windows), including nine new tests
covering options, selection, nesting, resource bounds, read containment, literal
content, source immutability and no-output failures in normal/dry-run/assets-skip
modes. Node-only real-engine acceptance passes all six importer fixtures; the
expanded Sphinx fixture retains nine routes, five literal blocks, selected code
text, ordinary prose search, heading anchors and existing assets. The first Node
run exposed an incorrect test assumption that fenced code is indexed; the test
now checks prose, consistent with the engine's documented search policy. No
search behavior changed. CI additionally checks exported HTML, hydration,
nested DOM structure and reload without executing the included examples.

Warnings-denied workspace Clippy, formatting, Node syntax, actionlint and diff
checks pass. Strict/deep docs retain 55 routes and 23 examples. The full Rust
workspace run and final-head CI/merge remain pending. No local Docker/browser/GUI,
native release, Netlify production or live provider mutation ran. User docs,
CLI contract/specification/status and this context describe the supported subset.
Evidence: `/private/tmp/zellij-literalinclude-{focused,engine-final,clippy,docs,workspace}.log`.


### Exact filename inventory follow-up

The first complete local Rust run passed 664 tests (5 hosted cases selected
separately in CI). Review then identified a case-insensitive-filesystem exclusion
bypass in the initial literalinclude resolver: a differently capitalized target
could resolve an excluded file. Fix **`e4be0eaca31f4fb07186120337279740b0d8ad5f`**
requires an exact match in the already-filtered source file inventory, before
contained-path and bounded-file reads. Regression cases cover both excluded-file
case aliases and differently capitalized eligible files. The user reference now
requires exact filename case on every OS. All 37 Sphinx CLI tests pass after the
fix. Final workspace/engine checks and final-head GitHub CI remain pending.


### Final local verification and Windows fixture correction

The complete workspace rerun after the inventory fix passed **664 Rust tests**
(5 hosted cases selected separately in CI). Final Clippy and Node conversion
checks also passed; strict/deep docs retain 55 routes and 23 examples. Fixture-only
commit `766c889` avoids the reserved Windows NUL filename for the control-byte
case. Run `42412362640` then exposed a Windows assertion expecting raw CRLF
checkout bytes where conversion intentionally produces LF. Test correction
**`317e5072200c3102bc001a593e0f4e0fc99b3993`** explicitly writes CRLF on every OS,
checks normalized output, and checks that source bytes remain untouched. Node
acceptance normalizes the expected fixture similarly. All 37 Sphinx tests and
all six importer engine fixtures pass after this correction. No product behavior
was changed for this test fix. Final-head CI and merge remain pending.
Evidence: `/private/tmp/zellij-literalinclude-{workspace-final,clippy-final,docs-final,engine-inventory,portable-fixture,crlf,engine-crlf}.log`.


## 2026-11-07 — Literal inclusion closure and search code boundaries

PR #100 merged as **`9fb29c8f5a3abe0804d151cef6d6be710ffa1bc3`** after all
16 checks passed on `77de5e41ae166974f49677d4eb681b645ab5062d` in
[42412853970](https://github.com/mylife-inc/Zellij/actions/runs/42412853970),
attempt 1. Tested/merged tree: `a0e48d25c9c9e90c4074fd463f45afea78e1dfb8`.
Main and origin/main matched and were clean before this follow-up. Both automatic
post-merge workflows passed on attempt 1: main `42415654053` (16 jobs) and base
image `42415653990` (13 jobs). Final-head acceptance included 664 local Rust tests
(5 hosted cases selected separately in CI), 427 engine tests, all six native
packages, both image architectures and combined manifests. Sphinx acceptance
passed 37 CLI tests on Unix and 35 on Windows; real exported code, nested DOM,
hydration and reload passed. Documentation retained 46 original routes, 282
heading IDs, 82 compatibility mappings and 29 byte-identical images without
captured browser errors. Remote state/releases were refreshed; rc.1 remains the
latest native release. No native tag/publication or production deployment ran.

Source **`f9676f5261781b1fe854fd93aa0185cb7a50d825`** fixes a confirmed MDX search bug: fixed-three-character
regular expressions could close a longer code fence too early and index example
text. A four-backtick or four-tilde block containing a three-character fence and
`fenced_only_sentinel` previously leaked that token into prose search. The initial
regression run reproduced 18 failures before the implementation changed.

The existing shared MDX/heading parser now records original Markdown code-block
and inline-code ranges in one full-document parse. Search masks those ranges
without changing UTF-16 offsets or CR/LF characters, then slices passages using
the same rendered heading IDs. This handles longer/mixed/unfinished fences,
quotes/lists/components, inline backtick runs, literal escaped/unmatched markers,
JSX attributes and optional math parsing without evaluating authored JavaScript.
Heading labels retain their code text. A section inside a component is never
reparsed as an incomplete MDX fragment. Standalone `mdxToText` accepts complete
MDX with optional parser plugins and reports malformed syntax; portable document
payloads continue through their existing separate indexing path.

The packaged CRLF fixture also exposed a pre-existing guide-loader bug: the
leading H1 was not removed under CRLF, duplicating the title and shifting repeated
heading anchors. The same source commit handles LF, CRLF and CR when removing the
leading title, retaining later headings and literal code unchanged. Loader cases
and the actual packaged preparation fixture verify stable canonical anchors.

All **453 engine tests** pass (26 more than the baseline), including the 40 search
cases, shared rendering/heading coverage and three line-ending loader cases.
All 37 Sphinx CLI tests still pass with the extended fixture. All six Node-only
importer/rendering fixtures pass; packaged preparation passes with actual bundled
CLI artifacts, CRLF input and optional math. Engine build verification, workspace
typechecking, strict/deep docs (55 routes, 23 examples), Node syntax, actionlint
and diff checks pass. CI additionally reads exported search JSON and exercises
the real search dialog: prose finds the correct heading, while a code-only token
returns no results. No local container/browser/GUI was used. User guide, engine
specification and durable context/status are updated. Final-head CI/merge remain
pending; broad import/provider/release gaps remain as previously documented.
Evidence: `/private/tmp/zellij-search-boundaries-{before,focused,engine-final,typecheck-all,build-final,sphinx,conversion,packaged-final,docs-final}.log`.


### 2026-11-08 — Local Sphinx CSV table files

`csv-table :file:` now imports local UTF-8 data through the same bounded reader
as `literalinclude`, after navigation chooses published documents. Headers,
stub columns, captions, nested cell markup, substitutions, links, images and
searchable text survive. CSV source files are not copied unless explicitly
linked; copy/skip/dry-run share the same input checks. UTF-8/BOM policy, CRLF/CR
normalization and tab delimiters are documented in the conversion reference.
Exact-case inventory lookup, containment, exclusions and symlink rejection apply.
Files are limited to 8 MiB; both resource types share 32 MiB unique-input and
expanded-text budgets, plus existing table dimensions and a 16-level nesting cap.
No source Python/Docutils, remote CSV URL or authored code executes.

Docutils assigns CSV cell source paths to the data file, while supplemental
`:header:` rows retain the directive source. Nested CSV files follow those paths;
Sphinx roles/images/downloads/literalinclude retain the original RST context.
A regression also exposed loss of indentation inside multiline CSV header
options; continuation indentation is now relative to the first-line value.
References: [Docutils table implementation](https://github.com/docutils/docutils/blob/master/docutils/docutils/parsers/rst/directives/tables.py),
[Sphinx CSV patch](https://github.com/sphinx-doc/sphinx/blob/master/sphinx/directives/patches.py),
[asset collectors](https://github.com/sphinx-doc/sphinx/blob/master/sphinx/environment/collectors/asset.py),
and [literal includes](https://github.com/sphinx-doc/sphinx/blob/master/sphinx/directives/code.py).

The initial positive regression failed against the prior implementation. Local
46 Sphinx CLI tests and 453 engine tests pass, along with all six Node import/render
fixtures and strict/deep docs (55 routes, 23 examples). The first broader Rust
run found an obsolete test expecting tab delimiters to fail; that expectation
was removed after actual tab-delimited import passed. The final workspace run,
final-head hosted/native CI and merge remain pending at this entry. No local
Docker/browser/GUI, native release, live provider deployment or production
migration ran. Remote CSV and broader Sphinx/domain/include fidelity remain out
of scope, as do the pending live-provider/release tasks in the CLI status.

Previous milestone closure: PR #101 merged as `cac41b71c9cfbc6767b5f718a0a163beb3b36d35`,
with the identical tested tree `f44ae2e59657a33789f9e06def3c0ccc85106e7f`.
All 16 checks passed on `7ce9419ce0aab665f32d20d6aa3e2a25bbe0c21c` in
run `42418585801`. Both post-merge workflows passed: CI `42421313719` and base
image `42421313696`. Native candidate rc.1 remains the newest published release.


Local CSV implementation **`a7b32b017525626c8fc5c4ce26a037e6b2df57bf`**
passes the final local workspace run: **673 Rust tests**, 0 failures, with 5
hosted-export tests intentionally ignored locally and selected separately in CI.
All 46 Sphinx CLI cases, 453 engine tests, warnings-denied workspace Clippy,
formatting, six actual-CLI Node/rendering fixtures, strict/deep docs (55 routes /
23 examples), script syntax and diff checks pass. Evidence is under
`/private/tmp/zellij-local-csv-{workspace-final,focused-final,engine,clippy-final,render-final,docs-final}.log`.
The multiline-header regression initially failed and passes after preserving
relative option indentation. Final-head cross-platform/native/hosted CI and merge
remain pending; these local results do not establish a bug-free or complete CLI.


### 2026-11-08 — Public asset URL linking

The user selected existing public HTTP(S) URLs for `--assets link`, with an
explicit `--asset-base-url`; copying remains the default. Current source adds
this policy to convert/cv, migrate/mig and ship through the shared converter.
Eligible local asset paths become base-plus-root-relative URLs; copy-only
namespace prefixes are omitted. Queries/fragments and encoded filenames survive;
document links retain Zellij routing. No asset bytes are read/copied in link
mode, no symlinks are created and no remote URLs are fetched. Distinct local
files mapping to one public URL fail before writes. Missing/excluded resources
retain the usual reports and containment checks. CSV/include content inputs
remain readable independently of asset policy. Absolute URLs persist in content
for subsequent build/publish commands; the public server remains required.

The URL base must be HTTP(S), with a host and no credentials/query/fragment,
whitespace, backslashes, malformed escapes or dot segments. It is required only
for link mode; other policy/base combinations fail. Reports list linked files
and public URLs, including in dry runs. The user reference documents source
roots and the lack of source-tool hashed/CDN layout emulation. Native release
rc.1 remains unchanged. Full validation and final-head CI/merge are pending.

PR #102 closure: all 16 checks passed on `6d0b5c321d831606e383c14c127e20f579bdaca3`
in run `42530352694`, attempt 1; merge `245eacd2bfde84492d3a337020d0a75e1784aeb0`
has identical tree `56cf10ab6ce261ad070ccdc2f677e8e41abf20f1`. Both post-merge
workflows passed: CI `42532456398` and Base image `42532456352`.


Public asset linking implementation **`1b2e2e4fca5d6d46ea24708276e6636d9193a1a0`**
passes all **681 Rust workspace tests** (5 real hosted-export cases intentionally
selected only in CI). All 25 import/delivery integration tests pass, including
all six importers, public/static asset roots, URL encoding/suffixes, invalid
options, collisions, missing targets, symlinks and no asset-byte reads. The
initial end-to-end test failed against the old CLI. One later test fixture
needed a Markdown paragraph break before its reference definition; product
behavior was unchanged by that fixture correction.

Warnings-denied workspace Clippy, formatting, strict/deep docs (55 routes / 23
examples), Node script syntax and diff checks pass. All six real CLI-to-engine
Node fixtures verify copy and link policies with unchanged routes, remote URL
rendering and no local asset copies. CI adds an actual linked static export and
Next image browser/download acceptance through a separate loopback asset server.
No local browser/container/GUI, live provider mutation or release publication
ran. Evidence: `/private/tmp/zellij-public-assets-{before,focused-final,workspace,clippy,render,docs}.log`.
Final-head GitHub CI and merge remain pending.


### 2026-11-09 — Refreshed rc.2 preparation (unpublished)

PR #103 merged public asset URL linking as
`fa0b20683855df69b30d0dde1366c22fe0bf3e15`, with the identical tested tree
`9c6250dd38090ae996e9542f0a65a99b20ac7a6c`. All 16 checks passed on final
head `ff90234de9d80af75de991e7087e51515acad980` in run `42557158727`.
Both post-merge workflows passed: CI `42560161757`, Base image `42560161729`.

The candidate notes now cover source through PR #103, including all six
importers, public asset URLs, expanded Sphinx support, provider adapters,
GitLab/CircleCI generation, native wizards and recent bug fixes. The old rc.2
target `0fbf0a1` is superseded; its 2026-11-02 evidence remains historical.
Source and lockfile still agree on unpublished `0.2.0-rc.2`. Both candidate
tags are absent, the shared release secret is present (permissions untested),
and generated release pages pin installers and all four candidate image tags.
No stable tags/shared installer files or production documentation are changed.

The latest main run passed all 16 jobs, including installation of six native
packages on their matching OS/architecture. All six downloaded artifact ZIP
digests, contained package checksums and single-executable archive contracts
were independently verified. These baseline CI packages are not public-release
acceptance; release builds need their own final hashes. All 29 local release
policy/download tests pass using fake publication commands and loopback fixtures.
No local container, browser, live provider deployment or native publication ran.

See `specs/evidence/zellij-rc2-preparation-2026-11-09.json` and the refreshed
`specs/releases/v0.2.0-rc.2.md`. This documentation preparation still needs its
own final-head CI. The exact resulting reviewed merge commit must be recorded
in the PR closure and approved before publication; the earlier approval-review
rejection explicitly required authorization for public prerelease tags/assets.
Known live-provider, runner, importer and Netlify limits remain in the release
notes and CLI status. The graph remains a pre-asset-link CLI snapshot and excludes
root `examples/**` under the user's explicit graph scope.


### 2026-11-09 — rc.2 publication and delivery acceptance refresh

The user explicitly approved `v0.2.0-rc.2` from
`54e636faf2162c36e50a91507f1f35be63a660d0`. Preparation PR #104 passed all 16
checks on `cb9b863aa78eaeb7aa09026b5f9f33799601eead` in run `42762346632`;
its merge retained tree `189c93d4edba6262abf15f06044b1178d14f8928`.
Both post-merge workflows passed: CI `42765943453`, Base image `42765943694`.
The standard release helper passed typechecking, schema drift, formatting,
warnings-denied Clippy, 453 engine tests and 681 Rust tests. Five real hosted
cases were intentionally selected in CI, not run locally. The approved tag
was pushed without modifying the reviewed source.

Release run **`42768544795` passed all 29 jobs**. Native archives/installers are
published in `mylife-inc/Zellij` and `mylife-inc/releases`; all six native
public-download/install jobs passed. All four engine variants are published
for Linux AMD64 and ARM64; both native public-image jobs passed. Public archive
checksums match across repositories, and attached installers match the tagged
source. The source tag and public image digests are fixed in
`specs/evidence/zellij-rc2-release-2026-11-09.json`. Production documentation and
shared stable installer updates were skipped. The old rc.1 release and the
historical rc.2 preparation/approval-rejection records are preserved.

Current follow-up retargets six saved delivery fixtures and their generated
GitHub workflows to rc.2, with dedicated new static/Next acceptance image tags.
Active CLI/quickstart documentation now installs and describes rc.2; remaining
provider, runner, importer and Netlify limitations stay explicit. All six
workflows match actual CLI generation; strict/deep docs pass (55 routes and 23
examples), and all 33 release/download/HTTP helper tests pass. This follow-up
still needs final-head PR checks and manual delivery acceptance after merge;
public package acceptance is not evidence of those separate delivery runs.

No local container/browser/GUI, production Netlify deployment or domain change
ran. Other provider secret/variable names are absent from this repository; the
user has been asked for an existing non-secret live-test destination while the
release and approved Netlify staging work continue.


### 2026-11-09 — rc.2 installed-CLI delivery acceptance closure

PR #105 merged as `5677577c75a4307cec5d4b0c7cd949f99aeb618a`, retaining tested
tree `58d0ed02fd77c5b94ad3960669a9ce0630c179d1` from head
`9fb6bbd32602856c0b75050335c53b912f2eea2c`. All 16 main PR checks passed in
`42770299276`, and all six generated-workflow PR verification runs passed.
The saved workflows now install published rc.2, use its hosted engine image,
and publish only to the dedicated new rc.2 acceptance image tags or approved
Netlify staging drafts. Active installation and command documentation was
updated with the release.

All six manual workflows passed on the merged source: portable `42773738576`,
hosted `42773741515`, static image `42773744556`, Next image `42773747457`,
Netlify portable draft `42773751667` and Netlify hosted draft `42773754795`.
Both public draft URLs passed HTTP page, linked-asset and source-file isolation
checks. Fresh runners pulled both newly published site images and passed HTTP
acceptance in `42774184245`; the static and Next immutable image digests,
draft URLs and job/source identities are retained in
`specs/evidence/zellij-rc2-delivery-acceptance-2026-11-09.json`.

Post-merge Base image `42773711571` passed; main CI `42773711512` was still
running when this entry was prepared. The complete final-head PR suite and
all manual acceptance runs above passed on the same source tree. This entry
changes audit/status evidence only, without modifying that tested runtime,
workflows, fixtures, authored site documentation or immutable release assets.

Live acceptance for the other provider accounts and actual GitLab/CircleCI
runners still needs suitable destinations. Advanced importer limits remain
in the CLI status. The Netlify hosting-comment hydration issue remains open:
HTTP fixture acceptance is not a browser hydration test or production migration.
No local Docker/browser/GUI or production Netlify/domain change ran.


### 2026-11-09 — Jekyll TOML configuration imports

Source `c9f1fbf7e67b6f87908bfd0db879bd61a2fb94e0` adds `_config.toml` to the Jekyll conversion
path used by `convert`/`cv`, `migrate`/`mig` and `ship`. Exactly one YAML or TOML
configuration is accepted; ambiguous files still require explicit consolidation
in a source copy. TOML keys remain case-sensitive, tables and arrays retain
structure, booleans/numbers retain their types, and date/time values become
text rather than Serde's private datetime map. The existing settings/defaults,
collection ordering, visibility, navigation, URL and asset validation applies.
Configuration parsing is data-only, bounded to 64 levels, and does not execute
Ruby, Liquid, plugins or source commands. Page front matter remains YAML; blog
posts, dated collections and config merging remain outside the supported subset.

The regression first failed against the previous TOML rejection. All 15 Jekyll
integration tests now pass, covering metadata, routes, URLs/assets, nested
defaults, ordering, case/type preservation, BOM/CRLF, ambiguity, invalid syntax
and settings, symlinks, unchanged sources and dry runs. All **684 Rust workspace
tests** pass; 5 real hosted cases are intentionally selected in CI only. The
final focused parser run also checks the documented nesting boundary.
Warnings-denied workspace Clippy, formatting, strict/deep documentation
(55 routes / 23 examples), Node syntax and diff checks pass.

The actual CLI-to-engine fixture now covers YAML and TOML Jekyll sites, alongside
the five other importers, under copy and public-link asset policies. All seven
Node rendering fixtures pass. CI will also export both Jekyll variants through
the real engine image. Conversion reference and quickstart document TOML with
an example and explicitly mark it as source-only after rc.2. Final-head PR CI
and merge remain pending for this change. No local Docker/browser/GUI, provider
deployment or new native publication ran; published rc.2 remains immutable.

PR #105's previously pending post-merge checks are now closed: all 16 main CI
jobs in `42773711512` passed, as did Base image `42773711571`, both on
`5677577c75a4307cec5d4b0c7cd949f99aeb618a`. The rc.2 delivery evidence now appends
that closure while preserving the original in-progress snapshot. PR #106 merged
the prior audit-only record as `1650e7d9318b311c72f857d216ca1815493108b7`.


### 2026-11-10 — Jekyll TOML acceptance closure

[PR #107](https://github.com/mylife-inc/Zellij/pull/107) merged as
`55639024c75cc8e50e1b6a923f9f023a5c5edfb5` after all 16 checks passed on
`db3aa01afb03a4e523cd6012cc6d6df811e30fec` in run `42778216082`,
attempt 1. The merge retains tested tree
`443cb5a2003d2aca3ddd9adfab6896746e984b63`. This includes all six native package
checks, native Windows/macOS/Linux regressions, both Linux image architectures
and real-engine export of both YAML and TOML Jekyll fixtures. Source implementation
is `c9f1fbf7e67b6f87908bfd0db879bd61a2fb94e0`; local 684 Rust tests, Clippy,
formatting, seven conversion/rendering fixtures and strict documentation checks
passed. The new configuration format is merged source, newer than published rc.2.

Automatic post-merge CI `42782791785` and Base image `42782792095` are not
yet complete at this refresh. They remain separate from the successful PR
checks; consult their current results before claiming they passed.
This closure edits audit/status documents only. Published rc.2 assets and tags
are unchanged. Other provider/runner acceptance, advanced importer limitations
and the reported Netlify hosting-comment hydration issue remain open.


### 2026-11-10 — Jekyll collection metadata sorting

Source `a3c3663ceae66daf7a20f3d01c50def1390e5a66` adds collection `sort_by` to
Jekyll imports in the shared convert/migrate/ship path. Derived navigation uses
effective front matter after scoped defaults, sorting comparable numeric or
text values ascending, with missing/null values last and source paths as ties.
64-bit integers retain exact ordering against integers and finite floats.
Literal, case-sensitive keys are not interpreted as nested paths. Generated
slug/extension and ASCII-title fallbacks follow source basenames; non-ASCII
generated titles need explicit metadata. Mixed families, nonfinite values,
structured/boolean values, generated date/category/tag/excerpt fields and
date/time-shaped values have explicit pre-write diagnostics and a numeric-rank
or literal-order workaround. This remains a data-only subset of Jekyll sorting.

The same change fixes explicit collection order lists: an array, even empty,
takes precedence over sort_by. Missing or unpublished entries are skipped and
reported rather than aborting conversion; unlisted pages retain filename order.
Explicit navigation data remains authoritative. Source files are unchanged,
and dry-run/error cases do not create output. User documentation covers these
rules and marks the new capabilities as source-only after published rc.2.

The new regression failed against the old sort_by rejection. All **691 Rust
workspace tests** pass (5 real hosted cases remain intentionally selected only
in CI), including **21 Jekyll integration tests** and exact numeric boundary
checks. Workspace warnings-denied Clippy, formatting, Node syntax, diff checks
and strict/deep documentation validation (55 routes / 23 examples) pass.
All **eight CLI-to-engine fixtures** pass locally: the existing seven plus a
TOML collection fixture asserting inherited ranks, ties, missing values and
engine sidebar order. CI also exercises that fixture through real hosted export.
Final-head CI and merge are pending for this milestone.

PR #107's previously pending post-merge workflows are now verified successful:
all 16 main CI jobs in `42782791785` and all 13 Base image jobs in `42782792095`
passed. PR #108 merged its audit closure as
`ea097410403b57c65b774d46e4d81ec2d0271c0b`. These earlier results do not stand
in for this milestone's final-head checks. No local Docker/browser/GUI,
provider deployment, production-domain change or new release publication ran.
The published `v0.2.0-rc.2` tag/assets remain immutable at `54e636f`.
Live provider/runner acceptance, advanced importer limitations and the Netlify
hosting-comment hydration issue remain open; passing tests do not establish
zero bugs or completion of the entire CLI design.


### 2026-11-10 — Jekyll collection sorting acceptance closure

[PR #109](https://github.com/mylife-inc/Zellij/pull/109) merged as
`47ceb61bbb77c19faf24b0ab08ac6f9069248050` after all 16 checks passed on
`37a5bc4c4c5f9f3f69beb0b0f1053dc79df207e1` in run `42795255252`,
attempt 1. The merge retains tested tree
`78065d94d21333b3e363df05f3b5bd33596fff2e`. Source implementation is
`a3c3663ceae66daf7a20f3d01c50def1390e5a66`. Native Windows/macOS/Linux regressions,
all six native package checks, both Linux image architectures, browser checks
and actual engine exports passed. The new collection fixture also verifies
inherited numeric ranks, missing values and tie ordering in exported sidebar
links. Local 691 Rust tests, warnings-denied Clippy, formatting, eight conversion
fixtures and strict documentation checks passed.

Automatic post-merge CI `42799993837` and Base image `42799993455` are not
yet complete at this refresh. They remain separate from the successful PR
checks; consult their current status before claiming they passed. This closure
updates only audit/status documents. Collection sorting, stale-order handling
and the preceding TOML work are merged source newer than the immutable rc.2
release. Provider/runner live acceptance, advanced importer limitations and the
Netlify hosting-comment hydration issue remain open. No local Docker/browser,
production-domain change or new release publication ran.


### 2026-11-10 — live provider acceptance and Cloudflare corrections

Source `21fb99c` fixes two problems found using a disposable Cloudflare Pages
project. Successful asset acknowledgement responses can omit `result`; only
acknowledgement endpoints now accept that form, while data payloads and the
success flag remain required. Portable exports without a top-level `404.html`
were served with Cloudflare's SPA fallback, returning home HTML with status 200
for nonexistent paths. The deployment's private staging directory now receives
a fallback 404 page if needed, preserving existing regular 404 files and
rejecting links/non-files. These fixes are newer than immutable rc.2.

Both regressions failed before their fixes. Focused tests pass (16 Cloudflare
cases; the real hosted case is selected in CI). Current-source portable
Cloudflare and GitHub Pages live checks pass for nested pages, exact authored
logo bytes, linked assets, source-file isolation and missing-page status.
Hosted and Railway live tests are still in progress. The manual acceptance
workflow now supports explicitly identified disposable provider destinations,
retains evidence and deletes only exclusively owned test image versions.

The user authorized test deployments and removal afterward. Cloudflare uses
existing authorized Ziryab credentials without changing Ziryab. GitHub Pages
uses a separate disposable repository; the initial temporary Pages setup on
mylife-inc/Zellij was removed and absence verified. Railway has an isolated
project/service, but creating its project token returned Not Authorized; a
user-created environment token is needed for `zel rw`. Track cleanup and exact
live results before claiming this milestone complete. No production domains
or existing customer projects were modified.

PR #110's main `4b782c6` post-merge workflows both passed: CI `42800161133` and
Base image `42800161483`. Earlier runs `42799993837` and `42799993455` were
superseded/cancelled, not successful. Full validation, final-head CI and merge
for the new Cloudflare corrections remain pending at this entry.


### 2026-11-10 — Cloudflare, GitHub Pages and Railway live acceptance closure

[PR #111](https://github.com/mylife-inc/Zellij/pull/111) merged as
`d1d03d4c75023689946318c08b4ecbcdce0c0a2a` after all 16 final-head checks passed on
`98b5d0cd94c931fce30d6efffb3fe2be90c7baab` in run `42898053745`.
Source `21fb99c` fixes Cloudflare success-only asset acknowledgements and adds
an upload-only fallback 404 page. Source `2c3f794` replaces Railway's nullable
change-count check with explicit empty staged-patch verification, without
requesting decrypted patch variables. Source `a5ee146` allows all instances to
be stopped during startup and requires running instances at SUCCESS. Scope,
malformed-state, staged-change, configuration-drift and failure checks remain.
Both Railway regressions were reproduced against the previous implementation.
Source `98b5d0c` updates the hosted-prefix fixture to allow only the required
root 404 outside the mount and verifies preservation of its prefixed 404; the
previous assertion failed in CI `42895754326`. Runtime behavior is unchanged
from the live-tested source.

All **six live cases passed**: Cloudflare and GitHub Pages portable/hosted sites
in `42891873828` (source `21fb99c`), plus Railway static/Next images in
`42895763990` (source `a5ee146`). Checks fetched home/nested pages, exact authored
logo bytes and linked assets, and required 404 responses for source and missing
paths. Railway reported SUCCESS for the exact newly returned deployment IDs;
HTTP checks then verified their public routes. These are current-source fixture
checks, not a retest of the unchanged rc.2 binary or live browser hydration.
Normal CI separately passed the browser and cross-platform acceptance suites.
Local final-head verification: **695 Rust tests**, 5 hosted-only ignored cases,
warnings-denied Clippy, formatting, seven Node delivery/cleanup checks, workflow
lint and strict/deep documentation validation (55 routes / 23 examples).

Cloudflare's project, GitHub Pages' publishing branch/site and the entire
Railway project were removed. Public Cloudflare test hostnames no longer
resolve; GitHub and Railway test URLs return 404. All three temporary Actions
secrets were removed. Registry cleanup `42896298737` deleted the two exclusively
tagged Railway test image versions after project deletion. An earlier failing
harness had deleted an image while its deployment could still be starting;
cleanup now runs as a separate explicit operation after service removal, and
shared/release versions are protected. The empty disposable GitHub repository
`mylife-inc/zellij-live-20261110-684141` remains: deleting it returned HTTP 403
because the current OAuth token lacks delete_repo. Its website is removed.
The original mylife-inc/Zellij Pages setup was also removed and absence verified.

See [the evidence](specs/evidence/zellij-live-provider-acceptance-2026-11-10.json)
for source/run IDs, deployment receipts, immutable image digests and cleanup.
The user supplied the Railway project token after API token creation returned
Not Authorized; this credential blocker is resolved. The disposable project
and its temporary GitHub secret are now deleted. Existing Ziryab projects, Netlify production/staging sites and
custom domains were unchanged. Published rc.2 remains immutable; these four
fixes require current source or a later release. Fly, Vercel, S3 and real
GitLab/CircleCI runner acceptance, advanced importer limitations and Netlify's
reported hosting-comment hydration issue remain open. Post-merge CI
`42903488658` and Base image `42903488649` are still running on `d1d03d4`
at this audit refresh and must be checked separately; this record does not claim zero bugs or a complete
implementation of every planned CLI capability.


### 2026-11-10 — live-provider cleanup and post-merge verification completed

The user deleted the empty disposable repository
`mylife-inc/zellij-live-20261110-684141`; a fresh GitHub repository API lookup
returns HTTP 404. This closes the prior delete_repo permission blocker. The
earlier HTTP 403 remains recorded in the evidence as the initial deletion
attempt. All temporary provider destinations, Actions secrets and exclusively
tagged test image versions from this acceptance session are now removed.

Both post-merge workflows for PR #111 merge
`d1d03d4c75023689946318c08b4ecbcdce0c0a2a` passed: all 16 CI jobs in
`42903488658` and all 13 Base image jobs in `42903488649`. These results close
the pending checks in the preceding entry. PR #112 merged the initial live
acceptance documentation as `a5daa5f4af44f063d82075e492369a5b37f1d446`.
This update changes only context/status/evidence; published rc.2 is unchanged.
Fly, Vercel, S3 and real GitLab/CircleCI runner acceptance, documented importer
limitations and the reported Netlify hosting-comment hydration issue remain open.


### 2026-11-10 — local Sphinx document includes

Source `3a19947a8b6dfb5a236e813468aca1feba04ee1e` adds parsed local RST `include` fragments through
convert/cv, migrate and shared ship conversion. Included headings, explicit
labels, links, images and substitutions join the owning document; supported
toctrees and nested lists/admonitions/tables retain their existing behavior.
Navigation-driven parsing permits untitled include-only `.rst` files without
publishing them as pages. Unlisted source documents are reported but no longer
parsed or used to read dependencies. Referenced fragments are not misreported
as unconverted orphan pages.

Includes use the same bounded resource reader as local CSV/literalinclude: exact
case, containment, exclusions, hidden/internal-tree and symlink checks; UTF-8
text only; 8 MiB per file and shared 32 MiB input/expanded budgets. Repeated
fragments are allowed, while cyclic paths, chains over 32 active source files,
more than 10,000 expansions and duplicate headings/labels fail before writes.
Options support UTF-8/BOM policy, Python-style zero-based line slices, nonempty
literal start-after/end-before markers and bounded tab expansion. Parser/code/
literal/presentation options, standard include libraries and explicitly excluded
fragments remain unsupported; code excerpts use literalinclude.

The official Sphinx Include/relfn2path implementations and Docutils include
reference were inspected. Sphinx include paths, document roles and assets stay
relative to the owning page, including nested fragments; CSV file directives
retain the actual containing fragment/cell source. Heading state is shared
across includes, while nested block limits remain enforced. Configuration
version/release substitutions now fill gaps after all declarations are parsed:
a regression reproduced the previous default/CSV-included-declaration collision.
No Python, Docutils, source extensions or remote source fetches are executed.

Local validation: all 53 Sphinx CLI tests pass, including seven new include
regressions; warnings-denied workspace Clippy, formatting, actual CLI-to-engine
rendering fixtures and strict/deep docs (55 routes / 23 examples) pass. The
initial broader workspace run passed 702 tests (5 hosted-only cases ignored);
the exact final source is being rechecked after the substitution-order fix.
GitHub final-head hosted/browser/native verification and merge remain pending.
Browser coverage adds included-heading hydration, images, navigation and search;
it runs only in CI, with no local Docker/browser/GUI. Published rc.2 is unchanged.

References: [Sphinx include](https://github.com/sphinx-doc/sphinx/blob/master/sphinx/directives/other.py),
[document-relative paths](https://github.com/sphinx-doc/sphinx/blob/master/sphinx/environment/__init__.py),
[Docutils include](https://docutils.sourceforge.io/docs/ref/rst/directives.html#including-an-external-document-fragment).


### 2026-11-10 — Sphinx include acceptance closure

[PR #114](https://github.com/mylife-inc/Zellij/pull/114) merged as
`ee866afd634cea5107941520c231bdc3fa6b270d` after all 16 checks passed on
`8f360bb13f6428680925c05e76441fddc81616a4` in run `42925938421`.
The merge retains tested tree `d5005a3ee4c22218ac5814e398667134bba85364`.
Source `3a19947` implements parsed document fragments and resolves the
configuration-default/substitution collision. Browser assertion follow-up
`8f360bb` uses the accessible heading name and verifies its ID independently
of the decorative anchor marker; superseded run `42925681071` was cancelled.

Final-source local validation passed all **702 Rust tests** (5 hosted-only
cases reserved for CI), including 53 Sphinx CLI cases, plus warnings-denied
Clippy, formatting, actual CLI-to-engine rendering and strict/deep docs
(55 routes / 23 examples). Final-head CI passed the six native package checks,
Linux/macOS/Windows regressions, both image architectures and real hosted
exports/browser tests. Included heading IDs, images, document navigation and
search passed in the generated site under its deployment prefix.

Automatic post-merge CI and Base image workflows are still running at this
refresh; their exact IDs/statuses are in the
[evidence](specs/evidence/zellij-sphinx-includes-2026-11-10.json). They are separate from the successful PR checks.
Parsed includes require current source or a later release; published rc.2
remains unchanged. Existing include limits, pending live Fly/Vercel/S3 and
GitLab/CircleCI runner acceptance, and the reported Netlify hosting-comment
hydration issue remain open. No local Docker/browser/GUI or live provider
deployment ran for this milestone.


### 2026-11-10 — Vercel live acceptance preparation and Sphinx post-merge closure

PR #114's previously pending post-merge workflows both passed on
`ee866afd634cea5107941520c231bdc3fa6b270d`: CI `42930701772` passed 16 jobs and
Base image `42930701756` passed 13. The Sphinx evidence retains the earlier
queued/in-progress snapshot and now records this closure separately.

The user authorized temporary accounts for remaining external verification.
Browser control failed twice before opening a signup page. No account, paid
plan or provider resource was created. Vercel team signup/token setup was
requested as the first handoff; the other provider trials should begin when
their tests are ready. Credentials belong in Actions secrets, never chat or
committed files.

Implementation `9c2510de1e2f3b720d60c9fe910ebe04b52e4d7e` adds a manual Vercel
workflow that builds current zel, creates a new disposable project, runs four
portable/hosted preview/production HTTP cases, then independently deletes the
confirmed project and verifies API 404. Project name, ID and team are checked
before deletion. Name collisions never modify existing projects; uncertain
create outcomes retain a recovery journal and require inspection. Public
previews apply only to the new fixture. The workflow does not delete accounts
or teams. Forced runner loss/cancellation can still require manual cleanup.

Validation passed 47 release/delivery helper tests, including 11 new lifecycle
regressions, plus actionlint for both changed workflows and `git diff --check`.
These are local simulated API/HTTP tests, not live Vercel acceptance. PR CI and
live account execution are still pending at this refresh. Rust/engine behavior
and published rc.2 are unchanged. Vercel/S3/Fly live tests, GitLab/CircleCI runner
acceptance, advanced import limits and the Netlify hosting-comment hydration
issue remain open. See [the external acceptance runbook](specs/zel-external-acceptance-2026-11-10.md).


### 2026-11-10 — Vercel empty-upload response defect fixed

During external acceptance preparation, official Vercel documentation and its
upload client confirmed that `POST /v2/files` succeeds with HTTP 200 and an empty
body. The adapter incorrectly required JSON. Replacing the local fixture's
upload response with an empty body reproduced `asset upload: invalid API
response` before creating a deployment.

Fix `36028b4a2a2a39a0bf914fc4be90947e3c02959f` accepts empty successful upload
acknowledgements and JSON objects. Project preflight, deployment creation and
status still require valid JSON; malformed/error/oversized upload responses,
HTTP failures and redirects retain their failure behavior. Both the native and
hosted export fixtures now exercise empty upload acknowledgements. The CLI
reference and external acceptance runbook describe the fix as newer than rc.2.

Verification passed **704 Rust tests**, with 5 hosted-only cases reserved for
CI, warnings-denied Clippy, formatting and strict/deep documentation checks
(55 routes / 23 examples). The 47 helper tests and actionlint passed for the
unchanged acceptance harness/workflows. Initial PR #116 CI `42937801644` on
`03fe013` predates this adapter fix and does not validate it. Fresh final-head
CI and live account acceptance remain pending. No account or provider resource
was created, and published rc.2 remains unchanged. See [the evidence](specs/evidence/zellij-vercel-upload-2026-11-10.json).


### 2026-11-10 — Vercel upload fix and acceptance workflow merged

[PR #116](https://github.com/mylife-inc/Zellij/pull/116) merged as
`d1d5b70cbea17014a37d59c12fa9f3f0408975b3` after all **16 final-head checks**
passed on `306e39243690da9ae9011589a2e1cf078ab2cbd3` in run `42938805590`.
The merge has the identical tested tree `0436ee28ce00e97e12034746c467714ddff61622`.
This includes six native packages, Linux/macOS/Windows checks, both Linux image
architectures, hosted Vercel export/upload acceptance and browser regressions.
Local validation passed 704 Rust tests (5 hosted-only cases exercised in CI),
47 helper tests, warnings-denied Clippy, formatting, strict/deep documentation
and actionlint. Source `36028b4` fixes the reproduced empty-upload response bug.

The manual disposable-project workflow is available on main. Actual Vercel
account acceptance is still pending the temporary team/token handoff; none of
its four live cases or cleanup has run. Browser control could not initialize,
so no new provider account was created. Published rc.2 is unchanged and lacks
this fix. Remaining external providers/runners and import limits stay open.
Automatic post-merge CI `42942094151` and Base image `42942094126` were queued
at closure; these are separate from the successful final-head checks. See the
[Vercel evidence](specs/evidence/zellij-vercel-upload-2026-11-10.json) and [setup runbook](specs/zel-external-acceptance-2026-11-10.md).

A read-only Netlify follow-up also confirmed that Node default fetch still
receives the hosting annotation with its leading LF on `/`, `/cli/completions`
and `/guide/quickstart/new-project`; Python urllib receives clean HTML for the
same routes. All six responses returned 200 with no badge script. This matches
the earlier client-dependent behavior and is **not** a new browser hydration
pass. The migration blocker remains unresolved; no site/domain changed. See
[the HTTP follow-up evidence](specs/evidence/zellij-netlify-html-followup-2026-11-10.json).


### 2026-11-10 — Vercel token verified; Pro trial team still needed

The user supplied the team slug `my-life-inc-s-projects` and confirmed saving
`ZEL_LIVE_VERCEL_TOKEN`. Read-only Actions run `42964231875`, source `2876459`,
resolved it to `team_JYk8MJycfmZ3sldE048ljr7Z` and verified token access. Vercel
reported `billing.plan=hobby`, `billing.status=active`, and no trial. No test
project or deployment was created. Credentials stayed inside Actions; only
selected non-secret identity/plan fields were logged.

The user was asked to create a temporary **Pro Trial** team without adding a
payment method, replace that existing secret with a token scoped to the trial
team, and send the new slug. This follows the agreed free-trial test setup and
Vercel's restriction of Hobby to personal, non-commercial use. Do not silently
enable a paid plan. Browser control was retried and still failed before opening
a page; its diagnostic identifies the Codex session's symlinked writable root.
No Codex security setting was changed.

Temporary branch `codex/vercel-account-preflight` contains a read-only version
of the existing dispatch workflow, solely to resolve the user-supplied slug.
It is not merged and must not replace the normal acceptance workflow. It can
resolve the replacement team's slug, then be removed after the account handoff
is complete. Current main's tested Vercel workflow remains unchanged.

PR #116's post-merge CI `42942094151` passed all 16 jobs, and Base image
`42942094126` passed all 13 jobs on `d1d5b70`. This closes their earlier queued
snapshot. Actual live Vercel cases remain unrun. See [account preflight](specs/evidence/zellij-vercel-account-preflight-2026-11-10.json)
and [upload-fix verification](specs/evidence/zellij-vercel-upload-2026-11-10.json).


### 2026-11-10 — Sphinx wildcard navigation; Vercel deferred

Implementation `703a367774fda2c4cfdb810a948185fcba687d31` adds Sphinx
`toctree :glob:` and `:reversed:` across convert/migrate/shared ship conversion.
The importer expands case-sensitive, suffixless document patterns once before
page discovery, navigation and rendering. It preserves per-pattern sorted
order, explicit-entry precedence, per-tree candidate sets, reverse order,
hidden navigation and owning-document paths for included toctrees. `*`, `?`,
`**`, character classes and negation follow Sphinx's matcher behavior. Empty
matches are reported; existing exclusions, filename validation, containment,
cycle and destination checks remain active. A 4096-byte resolved-pattern limit
and shared 10,000-match budget bound expansion. No dependency or runtime-image
size change is introduced.

The original real-CLI regression failed on the unsupported `:glob:` option.
All 56 Sphinx CLI integration tests now pass. Node-only actual-CLI/engine checks
pass for all six importers; the Sphinx fixture now has 13 routes and verifies
reversed inline/sidebar navigation. Strict/deep documentation checks pass for
55 routes and 23 examples. GitHub CI also exercises the hosted HTML and browser
navigation fixture. Full workspace/Clippy and remote CI results are recorded
in [the evidence](specs/evidence/zellij-sphinx-toctree-2026-11-10.json).
These capabilities are newer than published rc.2. `exclude_patterns` globs,
automatic numbering, MyST/domains/extensions and broader format fidelity remain
unfinished; this is not a zero-bug claim.

The user explicitly deferred Vercel live verification because creating the
trial team required a payment method. No Vercel project/deployment was created.
The temporary `ZEL_LIVE_VERCEL_TOKEN` GitHub secret and remote
`codex/vercel-account-preflight` branch were removed and their absence verified.
This did not revoke the provider token or delete the user's account/team.
The adapter and tested manual lifecycle workflow remain available. Fly/S3 and
real GitLab/CircleCI acceptance remain pending, and the Netlify hydration
blocker is unchanged. No further release publication or domain migration ran.

Behavior references: [Sphinx toctree](https://www.sphinx-doc.org/en/master/usage/restructuredtext/directives.html#directive-toctree),
[directive implementation](https://github.com/sphinx-doc/sphinx/blob/master/sphinx/directives/other.py),
[pattern matcher](https://github.com/sphinx-doc/sphinx/blob/master/sphinx/util/matching.py).


### 2026-11-10 — Sphinx wildcard navigation merged

PR #119 merged as `206acc42b3e6bada11c046e6e5295005756bcbde` after all
16 checks passed in run `42966645061` on final head
`73c109f2ed89583522e6ade4f91a65017f571057`. The merged tree is identical to
the tested tree, `bed4922f143d243170e7fd27202bcea6347f9b18`. Local validation
passed 709 Rust tests (five hosted cases reserved for CI), warnings-denied
Clippy, formatting, all six actual-CLI/engine import checks and strict/deep
docs validation. CI passed all six native packages, portable browser checks
on Linux/macOS/Windows, both image architectures, hosted wildcard navigation
and the full container/deployment/preview/ship regression suite.

The automatic post-merge runs are **not yet complete** at this snapshot:
Base image `42970277159` is running and main CI `42970277215` is queued.
Refresh them before claiming post-merge closure. The earlier entry's pending
local/PR gates are now closed; the evidence records the exact results.
Vercel remains user-deferred, and the documented import/external-verification
limits and Netlify hydration blocker remain open. Published rc.2 is unchanged.


### 2026-11-11 — Source exclusion globs and preview navigation lifecycle

Implementation `cabaa8fcab24e4fbcec5ab275c6b9ccb4ea14cc5` adds Sphinx
`exclude_patterns` globs, reusing the toctree matcher. File extensions are part
of matching; directory matches exclude descendants, including those discovered
by a toctree glob. Source names match case-sensitively after NFC normalization.
Sphinx source discovery requires a slash for leading `**/`; this differs from
the HTML builder's asset-copy matcher. Patterns are never opened as paths.
Limits are 1024 patterns, 4096 bytes each and 128 KiB combined. Existing
containment, symlink/junction and hidden/internal rules remain intact.

Excluded pages are reported and never published. Explicit excluded document or
include/CSV/code targets fail before writes. Images/downloads retain the
existing manual-repair report and original reference, without copied bytes or
public-URL rewriting, for copy/link/skip alike. Configured theme/static/template
directories still require explicit paths; only default `include_patterns` is
supported. Hosted import acceptance retains 13 routes and proves unsupported
excluded documents never reach parsing/export. User documentation is updated.

The preceding PR's automatic post-merge CI initially failed: two fresh wrapper
installs received npm HTTP 404 for `lucide-react-1.72.0.tgz`; the package now
returns HTTP 200, so only failed jobs were retried without dependency changes.
The macOS trace recorded a WebKit status-fetch error during navigation. Its
unchanged-source retry passed; CLI/Windows are still running at this snapshot.
Base image `42970277159` passed all 13 jobs. The earlier evidence retains both
the failure and retry history; no post-merge all-green claim is made yet.

Preview fix `50c1fe6ff7998b3d09c4bf18bd899a7a1788008a` cancels active status
requests and timers on `pagehide`, resumes exactly one polling loop on a
back/forward-cache `pageshow`, and discards responses from the previous page
lifecycle. Two deterministic tests failed before the fix (no cancellation and
stale-body reload); all three lifecycle tests pass after it. Existing browser
acceptance still checks navigation, reload, recovery and runtime errors without
suppressing WebKit errors. This is also newer than published rc.2.

Local verification: **715 Rust tests passed**, five hosted cases reserved for
CI; **60 Sphinx CLI cases** pass. Warnings-denied Clippy, formatting, all six
actual-CLI/engine import checks, **50 Node helper tests**, actionlint and
strict/deep docs validation (55 routes / 23 examples) passed. New-source CI is
pending; see [the evidence](specs/evidence/zellij-exclusions-preview-2026-11-11.json).
Vercel remains explicitly user-deferred. Advanced importer limits, remaining
external account verification and Netlify's hydration blocker remain open. No
new native release or production-domain migration occurred.


### 2026-11-11 — Exclusions and preview lifecycle merged

PR #121 merged as `48f95c96b672aefe7ba4ed3dd407d9748613009e` after all
16 checks passed in run `42983924584` on final head
`3d324906b451bd8fd0d1f97a24bc36526e517914`. The merged tree is identical to
the tested tree, `d2021f5c07d17b68e28cb66f88a9fe249d333c53`. In addition to
715 local Rust tests and 50 Node helper tests, CI passed all six native
packages, portable browsers on Linux/macOS/Windows, both Linux image
architectures, hosted Sphinx exclusion acceptance and the full CLI suite.
The macOS/WebKit preview checks passed with the new lifecycle fix.

The preceding PR #119 post-merge runs are now closed: Base image
`42970277159` passed all 13 jobs, and CI `42970277215` passed all 16 jobs
after retrying only its three failed jobs on unchanged source. The initial
npm tarball failures and WebKit navigation error remain recorded in the
evidence; this supersedes the earlier pending snapshots.

PR #121 automatic post-merge verification also passed, without retries:
Base image `42988073094` passed all 13 jobs and CI `42988073228` passed
all 16 jobs on merge `48f95c96b672aefe7ba4ed3dd407d9748613009e`. The new
preview lifecycle passed macOS/WebKit acceptance both before and after merge.
The evidence records these results separately from final-head PR verification.

User documentation and the current CLI status are updated. Custom Sphinx
`include_patterns`, automatic numbering, MyST/domains/extensions and the other
documented advanced importer gaps remain open. Live Vercel verification remains
user-deferred; Fly/S3 and real GitLab/CircleCI acceptance remain pending. The
Netlify hydration blocker is unchanged. Published rc.2 and the production
domain are unchanged; these new source capabilities are not yet released.

Next bounded importer task: custom Sphinx `include_patterns`. Upstream
`Project.discover` applies these patterns to document discovery, with exclusions
taking precedence; inclusion patterns match full filenames rather than directory
ancestors. Keep the eligible local-resource inventory separate so selected pages
can still use fragments, code/CSV files and assets outside the page-selection
patterns. Cover absent versus empty patterns, root/toctree misses, glob candidates,
NFC/case/separator behavior, resource retention and pre-write failures. This is
planning only, not implemented; see the upstream project/matching sources linked
in the exclusion evidence.


### 2026-11-11 — Sphinx source inclusion patterns

Implementation `47b3293b8effac68da8a97efa3ac1929f1191e57` supports custom
`include_patterns` in literal Sphinx configuration. The default remains `['**']`;
an empty list selects no documents. Patterns match complete relative filenames,
not directory ancestors, with the existing NFC/case/separator semantics and
per-setting limits. Exclusions take precedence. Root/explicit toctree targets
must be selected before any output is written, and glob toctrees see only
selected documents. Unselected pages are reported without parsing their content.

Page selection is separate from the contained resource inventory. Selected pages
retain local parsed includes, code/CSV resources and eligible images/downloads
outside the selected page patterns. Used fragments neither become standalone
pages nor produce unselected-page warnings. Explicit exclusions, hidden/internal
rules, containment and copy/link/skip behavior still apply. No dependency or
runtime image change is introduced. The hosted fixture retains 13 routes and
verifies an unsupported unselected page is not parsed/exported.

Three real-CLI regressions failed before the implementation. Local verification
now passes **720 Rust workspace tests**, with five hosted cases reserved for CI,
including **64 Sphinx CLI cases**. Warnings-denied Clippy, formatting, all six
actual-CLI/engine import checks and strict/deep documentation validation
(55 routes / 23 examples) pass. Cross-platform CI and merge are pending;
see the inclusion evidence. This is newer than published rc.2.

The user requested continuing until CLI completion. The working target remains
the original command contract and supported declarative import formats; full
source-generator/plugin/theme parity was asked about separately and is not
assumed. The broader completion audit and external acceptance remain ongoing.
Vercel live verification stays explicitly deferred, and there is no zero-bug
claim or authorization for a new release/domain migration.


### 2026-11-11 — Disposable Fly acceptance prepared

Source `b6a78de7c212f52bf98167aa055952ca5651be67` adds a manual Fly
acceptance workflow and a journaled lifecycle harness. It creates one isolated
temporary app with shared IPv4 and a small web Machine, exercises real portable
static and hosted Next deployments, checks public HTTP content, and verifies
app deletion by API 404. Identity changes, name collisions and ambiguous creation
fail explicitly; cleanup never guesses ownership. Images build before trial
compute starts, and only the known fixture restarts between cases.

All 10 lifecycle tests and all 60 helper tests pass; actionlint passes. PR CI
and live Fly verification are pending. No account or remote resource was
created. Browser initialization still fails on the symlinked Codex writable
root, so the external acceptance runbook contains the necessary account-side
steps. The token must be scoped to an eligible no-card trial organization and
saved directly in Actions; no payment method is authorized. Vercel stays
user-deferred. S3 and actual GitLab/CircleCI runner acceptance remain open.

See [external acceptance](specs/zel-external-acceptance-2026-11-10.md) and
[the completion gates](specs/zel-cli-completion-2026-11-11.md). The original
command contract uses conservative, non-executing importers; extending source
plugins/themes is separate from command-handler completion. The user's scope
clarification is still pending; no completion or zero-bug claim is made.

### 2026-11-11 — Sphinx merged; disposable S3 acceptance prepared

[PR #123](https://github.com/mylife-inc/Zellij/pull/123) passed all 16 jobs in
run `42998033906` at head `99d6449b03172f34c291d166afa41423d8fa91bb` and merged
as `872db136bd7cf10e3722b9c05d180881a7e4d799`. The merged tree exactly matches
the tested tree `f465fe23f4950675e734bfcae35ba6b499af8417`. Post-merge CI
`43002975098` and Base image `43002974973` are pending. Earlier pending notes
above are retained as dated snapshots; the inclusion evidence has the closure.

Source `11232ba8756e696a729ef0ff5b11573f14203285` prepares a manual S3
acceptance workflow and ten lifecycle tests. It creates one owned, tagged
`us-east-2` fixture bucket, runs portable/hosted first and replacement uploads,
checks public HTTP content and verifies bounded object/bucket cleanup. Existing
bucket names, account-policy blocks, versioning/tag changes and uncertain
creation fail explicitly. Account-wide settings remain read-only. AWS CLI is
used only to administer the fixture; the product has no new runtime dependency.
The restricted IAM policy and account/secret instructions are in the external
acceptance runbook. No AWS signup or cloud operation occurred.

All 70 helper tests and actionlint pass locally. Fly/S3 PR CI and both live
checks remain pending, with actual GitLab/CircleCI runner acceptance still open.
All registered command families have implementation handlers; supported import
subsets remain explicit. No additional release or production-site migration is
authorized or claimed complete.

### 2026-11-11 — Active CLI and quickstart documentation reconciled

Source `e3a78b458227c18d0d1ead147378833df3859b3b` corrects 14 active reference
and guide pages that still described implemented commands as planned, rc.2 as
unpublished, flat content paths, unavailable browser opening/reload, automatic
page registration, and the old section count. The migration lab now preserves
the original through review, uses an explicit configured site and explains the
portable path for loose Markdown. Wrapper guidance uses complete `eject` or
`new app` flows instead of copying `/app` without its engine. Historical audit
records remain intact, as do the deployment/DNS and optional Netlify CLI topics.

The documented native scaffolding commands were executed in a disposable folder.
Strict/deep checks pass for all 55 routes and 23 examples. All 14 changed MDX
pages render with the actual engine and streaming React SSR, preserving 89
published heading IDs. No local browser/container ran; full Next/browser
preservation acceptance remains a CI gate. See the docs reconciliation evidence.
The Fly/S3 harness is in [PR #124](https://github.com/mylife-inc/Zellij/pull/124);
its live runs still need account setup. No release or production migration ran.

### 2026-11-11 — Verification closure; remaining work needs account access

Sphinx inclusion PR #123 is fully closed: main CI `43002975098` passed all 16
jobs and Base image `43002974973` passed all 13, without retries. The source
merge is `872db136bd7cf10e3722b9c05d180881a7e4d799`.

Fly/S3 acceptance PR #124 passed all 16 jobs in `43003339641` at `5230d48`,
then merged as `af98abc356a09f964bf249b0830a62ded01613ed`. GitHub registered
both manual workflows as active (Fly `414572067`, S3 `414572068`). No live run
or cloud resource creation occurred. The helper gate passes 70 tests, including
ten lifecycle cases per provider. See the Fly/S3 preparation evidence.

Documentation PR #125 passed all 16 jobs in `43005542302` at `bf496d1`, including
the built site's browser/route/bookmark/asset preservation checks, and merged
as `6f8abd337afde39bed8f3fd38a29f9deb61a0a34`. Both PR merges were checked
against their exact tested trees. Duplicate push builds were skipped for these
test/workflow/docs-only changes; neither altered runtime code or dependencies.

The newest published candidate is still `v0.2.0-rc.2`, refreshed after merging.
No new version, release, account, paid plan or production-site migration was
created. Four build-only GitLab/CircleCI fixture directories are prepared in
`/private/tmp/zellij-runner-fixtures-20261111-eQ3BrY`; the external runbook has
the durable regeneration recipe and the remaining image/manifest/event tests.

All registered command families have handlers, and no unimplemented command
branch was found in the final scan. Remaining `TODO` strings are editable
scaffold placeholders. Live Fly/S3 and GitLab/CircleCI acceptance still need
account-side access because browser initialization fails before signup. The
Fly trial/token handoff and the AWS/GitLab/CircleCI account-availability question
remain unanswered. Vercel stays user-deferred. Importers retain their documented
subsets; optional full source-tool parity was not assumed. Use the completion
ledger for the remaining gates; passing checks is not a zero-bug proof.

## 2026-11-11 — CLI release before remaining provider verification

The user explicitly moved live deployment verification to the end and authorized
finishing the CLI, cutting a release and updating docs. Current command dispatch
and the reference cover all documented command families; no implementation
stubs remain. The release target is **v0.2.0-rc.3**, including the changes since
rc.2. Conservative importer boundaries remain explicit. Fly/S3 and actual
GitLab/CircleCI acceptance are deferred follow-ups; Vercel remains user-deferred.
No account setup or live provider run is needed for this release.

The workspace/lockfile and active docs select rc.3. Release notes summarize
Jekyll TOML/sorting, Sphinx includes/navigation/source patterns, provider response
and Railway startup fixes, and preview page lifecycle recovery. The preparation
record is [here](specs/evidence/zellij-rc3-preparation-2026-11-11.json). Normal
CI, merge, publication and public-artifact checks remain pending at preparation.
Do not treat this paragraph as evidence that assets are published.

### Rc.3 release closure — 2026-11-11

PR #127 merged as `58be9b1ab992424576572ee8e2ba736f134f727c`; all 16 CI
checks passed on `2b48747`, with an identical merged tree. The initial run
found an rc.2 constant in the external-CI replay fixture; the fixing commit
`2b48747` derives the expected pin from the actual binary. Both image
architectures then passed. The original failed/cancelled run remains recorded.

The release helper passed 720 Rust tests and 453 engine tests plus all local
preflight gates before creating **v0.2.0-rc.3**. Release run `43018433622`,
explicitly dispatched against that immutable tag after duplicate merge-push CI
was skipped, passed all 29 jobs without reruns. Six native public installer
targets and four engine variants on native Linux AMD64/ARM64 passed. Both
release repositories carry matching sets of 15 assets. Stable installer
updates and production docs deployment were skipped; the updated docs built.

The implementation/release/docs milestone is complete within the documented
command and importer contract. No new live provider checks, account setup or
production cutover were performed. Fly/S3, actual GitLab/CircleCI and the
user-deferred Vercel checks remain later work; frozen manual delivery fixtures
still pin their previously verified rc.2. Exact assets/digests and verification
are in [the rc.3 record](specs/evidence/zellij-rc3-release-2026-11-11.json).


Composer review correction: the initial `328489b` source inspected all union
alternatives while collecting references, so a heading such as
`[Literal title](/not-a-link)` incorrectly failed deep checking. A regression
reproduces the false positive; reference traversal now skips object alternatives
whose literal selectors disagree with the parsed value. The same text in a
`text` node remains an actual checked link. Fresh hosted acceptance is required.

Composer review, 2026-11-12: hosted run `43262466867` exposed a formatter
interoperability bug: `zel fmt` removed quotes from `"01"`, accepted as a string
by the Rust parser but a number by the engine parser. A native regression fails
on `f955784`; the fix keeps numeric-looking strings quoted, and corpus acceptance
now compares all YAML values using the engine parser before/after formatting.
The same run's three new surface assertions assumed dark inverted text differs
from base text; Cupertino intentionally shares that foreground. Assertions now
check the actual inverted foreground/background tokens. No theme was changed.
Screenshot captures now write explicit PNG files for artifact retention. Fresh
hosted acceptance is required; the failed/superseded run is not a passing gate.

Composer visual review on `e701cb1` (run `43264032824`): all 335 showcase
browser cases passed with 124 existing conditional skips and no retries; all
new composition cases passed. Retained light/dark desktop and phone images
exposed a reveal wrapper that prevented equal grid-card height, plus gradient
paint missing glyph descenders at tight line-height. The follow-up gives reveal
wrappers a stretching grid cell, keeps group content top-aligned, and adds paint
room without changing line layout. Browser assertions now check card heights and
heading alignment. Screenshots disable CSS transitions to avoid mid-theme-change
captures. Documentation acceptance explicitly includes `/sections/compose` and
its hydrated preview/source tabs. A new final-head run is required.

Composer closure — 2026-11-12: PR #143 merged as `ba5f0ad`, matching tested
`6b10098`. Reference selection was fixed in `f955784`, numeric-text formatting in
`e701cb1`, and visual alignment/gradient paint in `6b10098`. All 20 checks and all
13 development-image jobs pass. Documentation acceptance preserves 46 original
routes, 282 heading IDs, 82 compatibility mappings and 29 byte-identical images;
all 11 added routes also pass, including the new preview/source tabs. Published
rc.5 and production documentation/domain remain unchanged. Earlier pending and
failed checkpoints above are superseded, not erased.

The user confirmed package-level Admin access on 2026-11-12. Image retirement
can resume only with a refreshed rc.5 plan after the composer engine publication.
The previous 187-version rc.4 plan remains historical and must not be executed.
