# Zellij home and installer ownership

Status: rc.13 implementation under verification; public rc.12 retains its
previous defaults until the next candidate is published.

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
Cross-platform hosted results and the user's local migration are pending.

The private HP handoff is a separate authorized migration. Preserve all chapters,
routes, anchors, downloads and authored rich sections, replace its Python/container
regeneration pipeline with directly editable rich content, and verify the complete
double-click site before replacing the client's existing package.
