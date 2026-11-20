# Zellij home and installer ownership

Status: released and verified as rc.13; managed channels, local installation,
private client handoff and public documentation are updated.

The user selected `ZELLIJ_HOME` as the common root for Zellij-managed persistent
downloads. An unset or empty value defaults to the operating-system user's
home plus `Dev/config/zellij`, including Windows. An explicit value must be an
absolute path; the CLI never interprets a literal `~` as a shell expression.

| Resource | Location |
| --- | --- |
| Standalone installer executable | `ZELLIJ_HOME/bin/zel` (`zel.exe` on Windows) |
| Version/platform renderer packages | `ZELLIJ_HOME/cache/engines` |
| Remote template catalogs and archives | `ZELLIJ_HOME/cache/templates` |

Explicit `ZEL_INSTALL_DIR` / PowerShell `-InstallDir`, `ZEL_ENGINE_CACHE` and
template `--cache-dir` keep their narrower precedence. Homebrew installs into
its own Cellar and prefix; Scoop retains its managed package directory/shim.
Both executable distributions use the same resource-home policy at runtime.
`zel upgrade` continues replacing only its own standalone executable; changing
`ZELLIJ_HOME` does not relocate an already installed binary or bypass package
manager ownership. Installer output describes PATH setup without editing profiles.

Build output and authored template destinations remain in the selected project.
Short-lived download/build staging is still cleaned up automatically. Homebrew,
Scoop and container runtimes retain their own package/image storage; Zellij does
not change a third-party tool's global storage settings.

Existing cache locations are not silently moved or erased. Users can copy a
compatible version/platform renderer directory into the new engines directory;
the existing manifest/file verification runs before it is executed. Version
changes still need their matching renderer. Explicit legacy cache overrides
continue to work. A normal first build can populate the new location instead.

Verification adds custom Unicode/space home installation to both native
installers, invalid-home refusal before asset download, explicit install/cache
precedence, native renderer cache/offline acceptance through `ZELLIJ_HOME`,
remote template cache reuse, and read-only doctor diagnostics. The root resolver
has independent tests for default, empty, explicit, missing and relative values.
Cross-platform hosted results and the user's local migration are complete; see
the verification record below.

The private HP handoff is a separate authorized migration. Preserve all chapters,
routes, anchors, downloads and authored rich sections, replace its Python/container
regeneration pipeline with directly editable rich content, and verify the complete
double-click site before replacing the client's existing package.

## Completed verification

Source PR #213 passes all 35 CI jobs and merges as 9206950 with an identical
tested tree. Windows initially exposed
a mixed-separator path join; fixing commit 39503d7 joins components separately
and passes the same native checks. All 38 release jobs pass for rc.13, including
public downloads on six native platforms. Both repositories have the same 39
asset digests. Four Homebrew and four Scoop actual rc.12-to-rc.13 upgrade/lifecycle
jobs pass. Scoop's previous-release installer comparison needed the test-only
fix c1c7daa (PR #215); 21 local regression tests and the corrected Windows matrix
pass without changing immutable release artifacts.

The user's standalone executable/profile and previous renderer are deliberately
migrated with rollback copies. The private client handoff now regenerates its
200-route rich file website through the bundled CLI alone. Public-binary builds
with no system tools and an offline rebuild are byte-identical. Chapter text,
titles, routes, recorded anchors and all 244 referenced resources are preserved.
Chromium/WebKit cover every route and focused interactions; final script and
ZIP/manifest checks pass. Client-specific execution remains Intel macOS only.

Public guide PR #214 passes documentation validation and six released-artifact
consumer workflows. Production deployment 6affc4c6de4fdf5feecb44d7 is
verified by run 44670647146; live pages show the new policy. Cancelled duplicate
code runs are not counted as passes. Full details, including initial failure
history, are in [the release evidence](evidence/zellij-rc13-release-2026-11-19.json).
