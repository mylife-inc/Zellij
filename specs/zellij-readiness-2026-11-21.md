# Zellij readiness — 21 November 2026

Updated 22 November: the published candidate is **v0.2.0-rc.19**, source
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
| Documented CLI command families | Implemented, with native tests and the recorded release acceptance. | Provider-specific live behavior and external runners require separate acceptance. |
| Rich websites | File and HTTP static build, check, watch and view share the verified cached renderer. First use downloads it; later operations support `--offline`. Native static deployment builds and optional import validation use the same cache. No user-installed Docker, Node or Python is required for these web rendering operations. | Default editable wrapper generation now also uses a verified source cache. Running Next apps, explicit image extraction, container images and PDF retain their documented prerequisites. |
| Documentation and publishing | rc.19 guides and generated consumers pass their seven targeted gates; the diagnostic follow-up also passes Engine. PR #240 merges as `9e7c58e`; automatic production `45010872211` and nine live-page hydration/content checks pass. Refreshed fixture images pass fresh-pull HTTP/health/nonroot inspection. | Keep prerequisites and version pins current. The retained browser timing finding remains unresolved despite these passing gates. |
| Branding, menus and themes | Separate company/product branding, centered navigation, full-width desktop panels, mobile drilldown, responsive theme colors and the Kie theme are implemented. | Kie is an inspected design interpretation; blocked live reference access did not establish a 100% pixel-identical clone. |
| Composer and templates | Responsive nested layouts, supported media/semantic/interactive nodes, bounded scroll effects, eight inline built-in section types, nine editable presets and nine website/book starters are implemented. | Three website starters ship corrected inline guides at template version 0.1.3 in rc.16, while prior 0.1.2 archive bytes remain unchanged. The composer is bounded; arbitrary executable components, arbitrary section embedding and unrestricted animation timelines are not promised. |
| Books and PDF | Shared publication model, technical/study editions, equations/statements, numeric citations, glossary/index, print styling, Arabic RTL and French/German/Spanish print are implemented. | Arbitrary citation styles/BibTeX, unrestricted font inventories and broader format fidelity are separate expansion requests. Existing PDF generation uses its external toolchain. |
| Installation | Public rc.19 packages cover six native targets. Tagged source packages pass real installed-wrapper builds on every target; independent public Mac and installed empty-PATH source-cache acceptance pass. All eight actual rc.18-to-rc.19 Homebrew/Scoop upgrades pass. | rc.19 retains ownership protection and native x64/ARM64 WinGet manifests. The last recorded Microsoft PR #484934 checkpoint has a passing CLA and requests moderator approval; catalog acceptance remains separate. |
| Retention | The 20 November rc.12 retirement is complete and recorded in its dated evidence. No releases or image versions are retired during rc.19 publication. | Preserve rc.11/rc.13 compatibility pins, rc.14 WinGet upgrade fixtures and rc.15 catalog submission and rc.18 channel upgrade tests until their dependencies are deliberately retired. Do not reuse completed deletion plans. |
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
Optional LuaLaTeX and Next development packages follow.
The [source-backed roadmap](zellij-cached-toolchains-2026-11-21.md) records their
scope and acceptance requirements. Priorities 1–2 (static validation and HTTP
export/deployment builds) are complete in rc.18; priority 3 (editable sources)
is complete in rc.19. Images still need a builder;
explicit custom-image extraction must preserve its selected-image behavior.
Investigate any original timing recurrence using retained diagnostics. Catalog
review and deferred external acceptance remain separate from this work.
A supported command may be implementation-complete while one of these acceptance
or distribution tasks remains open.
