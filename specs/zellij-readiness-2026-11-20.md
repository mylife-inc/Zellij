# Zellij readiness — 20 November 2026

This is the current checkpoint for the older completion and expansion records.
Their rc.6–rc.11 statements are dated history, including statements that
production documentation or later print/composer features were still pending.
They must not be read as today's backlog.

The released candidate is **v0.2.0-rc.14**. Source, public artifacts, Homebrew,
Scoop, local installation and production documentation are verified in the
[rc.14 release evidence](evidence/zellij-rc14-release-2026-11-19.json).
This checkpoint does not declare stable 1.0 or absence of undiscovered bugs.

| Area | Current state | Remaining boundary or task |
| --- | --- | --- |
| Documented CLI command families | Implemented, with native tests and the recorded release acceptance. | Provider-specific live behavior and external runners require separate acceptance. |
| Rich double-click website | Standalone build, check, watch and view share the verified cached renderer. First use downloads it; later operations support `--offline`. No user-installed Docker, Node or Python is required for these operations. | Hosted development, explicit static/Next exports, container images and PDF retain their documented prerequisites. |
| Documentation and publishing | rc.14 guides are deployed at `zellij.shebka.net`; the automatic production workflow is operational. | Keep release pins and production evidence current when executable behavior changes. Production migration itself is complete. |
| Branding, menus and themes | Separate company/product branding, centered navigation, full-width desktop panels, mobile drilldown, responsive theme colors and the Kie theme are implemented. | Kie is an inspected design interpretation; blocked live reference access did not establish a 100% pixel-identical clone. |
| Composer and templates | Responsive nested layouts, supported media/semantic/interactive nodes, bounded scroll effects, eight inline built-in section types, nine editable presets and nine website/book starters are implemented. | The composer is bounded. Arbitrary executable components, arbitrary section embedding and unrestricted animation timelines are not promised. |
| Books and PDF | Shared publication model, technical/study editions, equations/statements, numeric citations, glossary/index, print styling, Arabic RTL and French/German/Spanish print are implemented. | Arbitrary citation styles/BibTeX, unrestricted font inventories and broader format fidelity are separate expansion requests. Existing PDF generation uses its external toolchain. |
| Installation | Public standalone packages for six native targets, Homebrew and Scoop pass release and upgrade acceptance. | WinGet packaging/acceptance is the next distribution task. Check registry eligibility and compatibility with the user's preview-retirement policy before submitting it; no submission or acceptance is claimed. |
| Private client handoff | The installed rc.14 package preserves all 468 source files and all 535 generated files. Actual Intel Mac validation, build, watch, preview and Chromium/WebKit acceptance pass for all 200 routes; installed regeneration preserves all 1,111 package files. | Four native Windows script combinations pass using a public fixture. Actual private-client execution on Windows, Apple Silicon and physical iPhone remains distinct acceptance. See the [handoff evidence](evidence/zellij-handoff-authoring-2026-11-20.json). |
| Retention | rc.14 consumers have migrated in the owned public channels. | Inventory remaining older releases/images and their consumers before applying a new reviewed retirement plan. Preserve private rollback renderer availability before deleting a version it requires. Do not reuse an earlier executed deletion plan. |
| Live providers and generated runners | Adapters and build/protocol harnesses exist. Earlier live Netlify, Cloudflare, GitHub Pages and Railway checks have their recorded evidence. | Fly, S3 and actual GitLab/CircleCI runner acceptance stay at the end by user instruction. Vercel remains explicitly deferred after payment was required. |

Two historical browser timing findings remain unexplained: WebKit navigation
overflow after resize and one documentation hydration-readiness timeout.
Bounded repetitions passed, and failure diagnostics were added; passing later
runs alone does not establish a root cause or resolve them. See the
[investigation](zellij-browser-timing-investigation-2026-11-13.md). Investigate
new recurrences using retained evidence rather than silently erasing these
findings or repeatedly running unrelated suites.

WinGet work includes an implementation prerequisite, not just a manifest:
[`zel upgrade` ownership detection](../cli/crates/zellij-cli/src/maintenance/upgrade.rs)
currently reads the Homebrew/Scoop marker under a `bin/` parent layout. Add and
test ownership handling for the actual WinGet installation layout before
publishing that channel, so self-upgrade cannot bypass its package manager.
The planned package uses the verified Windows x64/ARM64 binaries; validate
installation, upgrade and uninstall on both architectures. Microsoft's
[manifest guidance](https://learn.microsoft.com/en-us/windows/package-manager/package/manifest)
and [submission process](https://learn.microsoft.com/en-us/windows/package-manager/package/repository)
remain the external acceptance reference. No manifest has been submitted.

Next order: complete private Windows acceptance when a machine is available,
prepare and verify WinGet distribution, perform the scoped older-candidate
retention audit, then return to deferred external acceptance. A supported
command may be implementation-complete while one of these acceptance or
distribution tasks remains open.
