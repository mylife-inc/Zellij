# Zellij readiness — 20 November 2026

This is the current checkpoint for the older completion and expansion records.
Their rc.6–rc.11 statements are dated history, including statements that
production documentation or later print/composer features were still pending.
They must not be read as today's backlog.

The released candidate is **v0.2.0-rc.15**, with 37 source CI and 40 release
jobs passed. Public downloads, local upgrade, channel and documentation status
are recorded in the [release evidence](evidence/zellij-rc15-release-2026-11-20.json).
The private client handoff stays rc.14. This checkpoint does not declare stable
1.0 or absence of undiscovered bugs.

| Area | Current state | Remaining boundary or task |
| --- | --- | --- |
| Documented CLI command families | Implemented, with native tests and the recorded release acceptance. | Provider-specific live behavior and external runners require separate acceptance. |
| Rich double-click website | Standalone build, check, watch and view share the verified cached renderer. First use downloads it; later operations support `--offline`. No user-installed Docker, Node or Python is required for these operations. | Hosted development, explicit static/Next exports, container images and PDF retain their documented prerequisites. |
| Documentation and publishing | rc.15 guides are deployed at `zellij.shebka.net`; automatic run 44730090071 and live browser checks pass. | Keep release pins and production evidence current when executable behavior changes. Production migration itself is complete. |
| Branding, menus and themes | Separate company/product branding, centered navigation, full-width desktop panels, mobile drilldown, responsive theme colors and the Kie theme are implemented. | Kie is an inspected design interpretation; blocked live reference access did not establish a 100% pixel-identical clone. |
| Composer and templates | Responsive nested layouts, supported media/semantic/interactive nodes, bounded scroll effects, eight inline built-in section types, nine editable presets and nine website/book starters are implemented. | The composer is bounded. Arbitrary executable components, arbitrary section embedding and unrestricted animation timelines are not promised. |
| Books and PDF | Shared publication model, technical/study editions, equations/statements, numeric citations, glossary/index, print styling, Arabic RTL and French/German/Spanish print are implemented. | Arbitrary citation styles/BibTeX, unrestricted font inventories and broader format fidelity are separate expansion requests. Existing PDF generation uses its external toolchain. |
| Installation | Public standalone packages for six native targets, Homebrew and Scoop pass release and upgrade acceptance. | rc.15 includes ownership protection and verified public native x64/ARM64 WinGet manifests. Microsoft PR #484934 is submitted; CLA now passes after the user accepted personally. Microsoft must review its bare-executable probe (exit 2); catalog acceptance and ordinary catalog-ID installation remain pending. |
| Private client handoff | The installed rc.14 package preserves all 468 source files and all 535 generated files. Actual Intel Mac validation, build, watch, preview and Chromium/WebKit acceptance pass for all 200 routes; installed regeneration preserves all 1,111 package files. | Four native Windows script combinations pass using a public fixture. Actual private-client execution on Windows, Apple Silicon and physical iPhone remains distinct acceptance. See the [handoff evidence](evidence/zellij-handoff-authoring-2026-11-20.json). |
| Retention | Owned channels and both acceptance images have verified rc.15 replacements. Inventory 44873543634 and all 29 deletion safeguards pass. | Execute the reviewed rc.12-only retirement plans. Preserve rc.11/rc.13 private rollbacks, current rc.14 handoff/upgrade fixtures and rc.15. The earlier broader plan was superseded before deletion. |
| Live providers and generated runners | Adapters and build/protocol harnesses exist. Earlier live Netlify, Cloudflare, GitHub Pages and Railway checks have their recorded evidence. | Fly, S3 and actual GitLab/CircleCI runner acceptance stay at the end by user instruction. Vercel remains explicitly deferred after payment was required. |

Two historical browser timing findings remain unexplained: WebKit navigation
overflow after resize and one documentation hydration-readiness timeout.
Bounded repetitions passed, and failure diagnostics were added; passing later
runs alone does not establish a root cause or resolve them. See the
[investigation](zellij-browser-timing-investigation-2026-11-13.md). Investigate
new recurrences using retained evidence rather than silently erasing these
findings or repeatedly running unrelated suites.

Released rc.15 adds registry/file ownership detection before self-upgrade,
verified x64/ARM64 portable manifests and native install/upgrade/uninstall
acceptance. Its Microsoft catalog submission is pending. See the
[distribution contract](zel-winget-distribution-2026-11-20.md) for validation,
retention and external acceptance boundaries. The native upgrade fixture uses
real rc.14 archives; retain them until that dependency is deliberately migrated.

Next order: execute and independently verify the reviewed rc.12-only retirement,
follow Microsoft catalog review, and investigate any browser timing recurrence
with its retained diagnostics. Complete private Windows acceptance when a machine
is available, then return to deferred external acceptance. A supported
command may be implementation-complete while one of these acceptance or
distribution tasks remains open.
