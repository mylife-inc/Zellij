# Zellij readiness — 21 November 2026

The released candidate is **v0.2.0-rc.18**, source `faaee97`, with all 37
required source CI and 40 release jobs passed. Public-download, channel and production
documentation status is recorded in the [release evidence](evidence/zellij-rc18-release-2026-11-21.json).
This checkpoint does not declare stable 1.0 or absence of undiscovered bugs.

This is the current checkpoint for older completion and expansion records.
Their rc.6–rc.17 statements are dated history, including prior pending print,
composer, documentation and cached-renderer milestones. They must not be read
as today's backlog. The [validation](zellij-cached-static-validation-2026-11-21.md)
and [HTTP export](zellij-cached-http-export-2026-11-21.md) cache migrations now ship.

| Area | Current state | Remaining boundary or task |
| --- | --- | --- |
| Documented CLI command families | Implemented, with native tests and the recorded release acceptance. | Provider-specific live behavior and external runners require separate acceptance. |
| Rich websites | File and HTTP static build, check, watch and view share the verified cached renderer. First use downloads it; later operations support `--offline`. Native static deployment builds and optional import validation use the same cache. No user-installed Docker, Node or Python is required for these web rendering operations. | Editable wrapper extraction, Next development, container images and PDF retain their documented prerequisites. |
| Documentation and publishing | rc.18 guides and regenerated consumer workflows pass the seven targeted checks in PR #236. Preservation covers 46 original routes, 13 additions, 282 historical headings and 29 unchanged images. | Production publication and fixture-image refresh are tracked in the current release evidence; the established automatic Netlify publication workflow remains in use. |
| Branding, menus and themes | Separate company/product branding, centered navigation, full-width desktop panels, mobile drilldown, responsive theme colors and the Kie theme are implemented. | Kie is an inspected design interpretation; blocked live reference access did not establish a 100% pixel-identical clone. |
| Composer and templates | Responsive nested layouts, supported media/semantic/interactive nodes, bounded scroll effects, eight inline built-in section types, nine editable presets and nine website/book starters are implemented. | Three website starters ship corrected inline guides at template version 0.1.3 in rc.16, while prior 0.1.2 archive bytes remain unchanged. The composer is bounded; arbitrary executable components, arbitrary section embedding and unrestricted animation timelines are not promised. |
| Books and PDF | Shared publication model, technical/study editions, equations/statements, numeric citations, glossary/index, print styling, Arabic RTL and French/German/Spanish print are implemented. | Arbitrary citation styles/BibTeX, unrestricted font inventories and broader format fidelity are separate expansion requests. Existing PDF generation uses its external toolchain. |
| Installation | Public rc.18 packages cover six native targets; independent Mac public-download, installed upgrade and empty-PATH file/static acceptance pass. All eight public Homebrew/Scoop actual rc.17-to-rc.18 upgrade/lifecycle cases pass. | rc.18 retains ownership protection and verified public native x64/ARM64 WinGet manifests. Microsoft PR #484934 (rc.15) has a passing CLA, no executable-error label and a request for moderator approval. Installation Validation remains neutral; catalog acceptance and ordinary catalog-ID installation remain pending. |
| Retention | The 20 November rc.12 retirement is complete and recorded in its dated evidence. No releases or image versions are retired during rc.18 publication. | Preserve rc.11/rc.13 compatibility pins, rc.14 WinGet upgrade fixtures and rc.15 catalog submission and rc.17 channel upgrade tests until their dependencies are deliberately retired. Do not reuse completed deletion plans. |
| Live providers and generated runners | Adapters and build/protocol harnesses exist. Earlier live Netlify, Cloudflare, GitHub Pages and Railway checks have their recorded evidence. | Fly, S3 and actual GitLab/CircleCI runner acceptance stay at the end by user instruction. Vercel remains explicitly deferred after payment was required. |

The cached-renderer current-page defect found during this release is corrected
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
finding; it makes no runtime change or resolution claim.

Released rc.18 retains registry/file ownership detection before self-upgrade,
verified x64/ARM64 portable manifests and native install/upgrade/uninstall
acceptance. The rc.15 Microsoft catalog submission awaits moderator approval. See the
[distribution contract](zel-winget-distribution-2026-11-20.md) for validation,
retention and external acceptance boundaries. The native upgrade fixture uses
real rc.14 archives; retain them until that dependency is deliberately migrated.

Next implementation: a versioned editable wrapper/engine source package for
default `eject`, followed by optional LuaLaTeX and Next development packages.
The [source-backed roadmap](zellij-cached-toolchains-2026-11-21.md) records their
scope and acceptance requirements. Priorities 1–2 (static validation and HTTP
export/deployment builds) are complete in rc.18. Images still need a builder;
explicit custom-image extraction must preserve its selected-image behavior.
Investigate any original timing recurrence using retained diagnostics. Catalog
review and deferred external acceptance remain separate from this work.
A supported command may be implementation-complete while one of these acceptance
or distribution tasks remains open.
