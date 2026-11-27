# Zellij readiness — 21 November 2026

Updated 26 November: the published and installed candidate is **v0.2.0-rc.24**,
source `2c3b7da`. All 55 release jobs pass on their first attempt, including six
public native installations, six complete cached-PDF checks against both
repositories and both Next-image PDF architectures. Both repositories contain
58 matching assets. Independent public Mac checks and installed cold/offline
operations with the default resource home and empty PATH pass. Homebrew and
Scoop publish the exact verified recipes; all eight actual rc.21-to-rc.24 upgrades pass. Five new website-template identities are preserved in the ledger.
The revised guides are published from PR #263 (`c583efb`), with all 37 PR CI
jobs passing and automatic production run `45806880663` successful. Preservation
covers all 61 routes; ten live guides pass HTTP/hydration checks with no captured
browser errors. See [the rc.24
release record](zellij-rc24-release-2026-11-26.md).

Earlier implementation checkpoint: [cached Next development](zellij-cached-development-2026-11-25.md)
and the [PDF mitigation](zellij-pdf-wasm-lifetime-2026-11-25.md) passed all
37 source CI and nine PDF jobs for rc.23. Its tagged release then caught a
[development acceptance race and Windows file lock](zellij-development-acceptance-2026-11-25.md).
The corrected commit-boundary fixture and bounded Windows replacement retries
are being qualified for rc.24;
no rc.22 or rc.23 CLI release was promoted.

Updated 25 November: the published candidate is **v0.2.0-rc.21**, source
`cd7608e`. All 55 release jobs pass on their first attempt, including six native
public installations, six cached-PDF checks against both repositories and both
Linux image architectures. Both repositories contain 46 matching assets.
Independent public Mac checks, installed default-home cold/offline builds and
all eight actual rc.20-to-rc.21 Homebrew/Scoop upgrades pass.

`zel pdf` now uses the cached compiler by default. `bp` builds web plus PDF,
`bs` builds HTTP static output, `vw` previews with watching, and `lint` performs
deep strict validation. Updated book templates have new immutable versions.
The revised guides pass local production build and 61-route browser preservation;
PR #259 merged as `0d7e3a1`; production publication `45720791311`,
post-merge CI `45720791571` and images `45720791454` all pass. Eight public
guide pages were checked successfully. See the
[rc.21 release record](zellij-rc21-release-2026-11-25.md) and
[publication evidence](evidence/zellij-rc21-release-2026-11-25.json).
Historical browser timing findings and the exact earlier Mac deadline failure
remain unproven; the controlled response-body timeout defect is fixed in
`08a0b28`. This is not a stable-1.0 or zero-bug claim.

Earlier 25 November checkpoint: the published candidate was **v0.2.0-rc.20**, source
`65249fa`. All 55 release jobs pass, including six native public installations
and six cached-PDF jobs checking both release repositories. Both repositories
offer 46 matching assets. Independent public Mac checks and installed cold/offline
builds using the default resource home pass. Use `--pdf-backend cached` to make
PDFs without installed Docker, Node, Python or TeX; `container` remains the
default. All eight actual rc.19-to-rc.20 Homebrew/Scoop upgrades pass.
See the [rc.20 release record](zellij-rc20-cached-pdf-2026-11-24.md) and
[publication evidence](evidence/zellij-rc20-release-2026-11-25.json).
The first Windows qualification attempt had an original-source download timeout;
its targeted retry passed unchanged. This is not a zero-bug or stable-1.0 claim.

Previous 22 November checkpoint: the published candidate was **v0.2.0-rc.19**, source
`3d3b426`. All 37 required source jobs, 40 release jobs and eight actual
Homebrew/Scoop rc.18-to-rc.19 upgrades pass. Both public repositories have 42
matching assets; independent public Mac checks and the installed empty-PATH
source-cache checks pass. The final guides are published and nine live pages
are verified. See the [release evidence](evidence/zellij-rc19-release-2026-11-22.json).
The separate main CI has 36 passes and the retained historical documentation
timeout recurrence; this checkpoint does not declare stable 1.0 or zero bugs.

This is the current checkpoint for older completion and expansion records.
Their rc.6–rc.17 statements are dated history, including prior pending print,
composer, documentation and cached-renderer milestones. They must not be read
as today's backlog. The [validation](zellij-cached-static-validation-2026-11-21.md)
and [HTTP export](zellij-cached-http-export-2026-11-21.md) cache migrations now ship.

| Area | Current state | Remaining boundary or task |
| --- | --- | --- |
| Documented CLI command families | Implemented; the [26 November source/help/test audit](zellij-cli-completion-audit-2026-11-26.md) reconciles handlers, options, aliases and verification. | Provider-specific live behavior and external runners require separate acceptance. The audit does not prove every option combination or zero bugs. |
| Rich websites and development | File/HTTP static build, check, watch and view share the verified renderer cache. Static deployment builds and optional import validation reuse it. Rc.24 adds cached `zel dev`, reusing private Node and matching sources; cold/offline acceptance passes on six targets. These operations need no separately installed Docker, Node/npm or Python. | Custom Next apps, explicit image extraction and container image operations retain their documented prerequisites. Development has its own optional dependency download. PDF remains opt-in and uses the separate cached compiler by default. |
| Documentation and publishing | Rc.24 guides retain short workflows and book/website-template Quickstarts, adding cached development and accurate prerequisites. The published guide build preserves 61 routes, 282 original heading IDs, 82 compatibility mappings and 29 original images. PR #263 passes all 37 CI jobs; production run `45806880663` and ten live guide checks pass. | The workflow continues to enforce preservation for each documentation publication. Keep prerequisites and version pins current. Historical browser timing findings remain unresolved. |
| Branding, menus and themes | Separate company/product branding, centered navigation, full-width desktop panels, mobile drilldown, responsive theme colors and the Kie theme are implemented. | Kie is an inspected design interpretation; blocked live reference access did not establish a 100% pixel-identical clone. |
| Composer and templates | Responsive nested layouts, supported media/semantic/interactive nodes, bounded scroll effects, eight inline built-in section types, nine editable presets and nine website/book starters are implemented. | Five website starters ship new immutable versions in rc.24 with cached-development instructions; previous template archive bytes remain unchanged. The composer is bounded; arbitrary executable components, arbitrary section embedding and unrestricted animation timelines are not promised. |
| Books and PDF | Shared publication model, technical/study editions, references, citations, glossary/index, print styling and supported Latin/Arabic print are implemented. Rc.20 adds a verified optional compiler under `ZELLIJ_HOME/cache/pdf`, reusing private Node. Public acceptance verifies complete matching sources and offline PDF/rich builds on six targets. | Rc.21 selects cached PDF by default, updates generated CI and publishes new book-template versions. Explicit container selection remains available. Arbitrary citation styles/BibTeX, unrestricted font inventories and broader format fidelity remain expansion requests. |
| Installation | Public rc.24 packages pass all six native installation jobs and independent Mac acceptance. The installed default-home cold/offline checks pass. Homebrew and Scoop recipes match the release's verified checksums; all eight actual rc.21-to-rc.24 upgrades and lifecycle checks pass. | Ownership protection and native WinGet manifests remain included; Microsoft catalog approval stays separate. |
| Retention | The [26 November cleanup](zellij-rc16-rc18-retirement-2026-11-26.md) removed six rc.16–rc.18 releases, six tags, 240 assets and 72 image versions. Independent inventory and fresh rc.19 image pulls pass; public rc.24 downloads remain valid. The earlier inspector-version mismatch is fixed. | Preserve the retained compatibility/catalog/upgrade inputs, PDF source/runtime pairs, failure tags and unreferenced history. The completed execution plan must not be reused. |
| Live providers and generated runners | Adapters and build/protocol harnesses exist. Earlier live Netlify, Cloudflare, GitHub Pages and Railway checks have their recorded evidence. | Fly, S3 and actual GitLab/CircleCI runner acceptance stay at the end by user instruction. Vercel remains explicitly deferred after payment was required. |

The cached-renderer current-page defect found during rc.18 is corrected
in `e3bc39a`: prefixed routes now match sidebar items in both rich modes, with
SSR and hydrated regression coverage. See the HTTP migration record.

A separate desktop overflow focus defect is corrected in source `f80596b`:
delayed menu restoration now respects a newer focus or pointer choice. The
regression fails on the old engine and passes locally in Chromium/WebKit.
All 37 required source CI jobs and documentation validation pass. The final
full browser run passes 552 cases without failures or retries (169 conditional
skips). PR #229 is merged; production documentation and main images are updated. The correction now ships in the rc.17 renderer and matching images. See the [implementation record](zellij-overflow-focus-2026-11-21.md).

Two historical browser timing findings remain unexplained: WebKit navigation
overflow after resize and one documentation hydration-readiness timeout.
Bounded repetitions passed, and failure diagnostics were added; passing later
runs alone does not establish a root cause or resolve them. See the
[investigation](zellij-browser-timing-investigation-2026-11-13.md). Investigate
new recurrences using retained evidence rather than silently erasing these
findings or repeatedly running unrelated suites. The [20 November follow-up](evidence/zellij-browser-timing-followup-2026-11-20.json)
reviewed failed/retried CI attempts without identifying new evidence for either
finding. On 22 November, main CI `45007103274` reproduced the documentation
timeout. The retained capture has no browser errors and cannot distinguish
unresponsive evaluation from stalled scheduling. Diagnostic follow-up `bfb450b`
now retains immediate state independently of page timers and timestamps requests.
Its regressions pass; the original timing finding remains unresolved.

Released rc.19 retains registry/file ownership detection before self-upgrade,
verified x64/ARM64 portable manifests and native install/upgrade/uninstall
acceptance. The last recorded rc.15 Microsoft catalog checkpoint awaits moderator
approval. See the
[distribution contract](zel-winget-distribution-2026-11-20.md) for validation,
retention and external acceptance boundaries. The native upgrade fixture uses
real rc.14 archives; retain them until that dependency is deliberately migrated.

The versioned editable wrapper/engine source package for default `eject` and
`new app` is published and verified in rc.19.
The optional cached LuaHBTeX package first shipped in rc.20; cached Next
development now ships in rc.24. The isolated PDF process also retains the
scoped Wasm code-GC mitigation described in its dated qualification record.
The [22 November PDF feasibility work](zellij-cached-pdf-feasibility-2026-11-22.md)
adds an offline Wasm probe and fixes a Colima generated-input share defect in
source (`0619869`). Sixteen local manuscripts pass, and the initial six-platform
probe verifies 96 cases and 145 native/Wasm page comparisons. Ten regressions
cover the hardened prototype (`43b8c12`); PR #242 records its current checks.
That initial probe did not change the released PDF prerequisite. Subsequent qualification
includes strict process supervision, complete cache validation, original source
and notice packages, and 120 manuscripts across six native targets (PR #252).
The [24 November CLI integration](zellij-cached-pdf-cli-2026-11-24.md) connects
the opt-in cached backend and private Node to installed commands. Its synthetic
release-profile acceptance does not authorize distribution; the
[scoped licensing decision](zellij-pdf-distribution-decision-2026-11-24.md) is approved
and applied to the isolated adapter. PR #254 (`67a411e`) completes the reviewed
component/relink evidence and paired runtime/source packaging; its eight PDF
jobs and 37 regular jobs pass. PR #255 (`65249fa`) adds real-renderer integration
and complete source-download acceptance on all six platforms. Rc.20 publishes
the reviewed runtime/source pair; all six public cached-PDF jobs check both
repositories successfully. Container stays the default in rc.20.
The [source-backed roadmap](zellij-cached-toolchains-2026-11-21.md) records their
scope and acceptance requirements. Priorities 1–2 (static validation and HTTP
export/deployment builds) are complete in rc.18; priority 3 (editable sources)
is complete in rc.19; priority 4 (optional cached PDF) ships in rc.20, and its
default transition ships with shortcuts and new book templates in rc.21. Images still need a builder;
explicit custom-image extraction must preserve its selected-image behavior.
Investigate any original timing recurrence using retained diagnostics. Catalog
review and deferred external acceptance remain separate from this work.
A supported command may be implementation-complete while one of these acceptance
or distribution tasks remains open.
