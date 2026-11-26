# Cached toolchain opportunities — 21 November 2026

Status refreshed **26 November 2026:** all five selected cache migrations have
shipped. Rich validation and HTTP export ship in rc.18 (`faaee97`), editable
sources in rc.19, optional PDF in rc.20, cached PDF as the default in rc.21,
and stock Next development in rc.24 (`2c3b7da`). Rc.24 is published, installed
and verified through both owned installation channels. See the
[release record](zellij-rc24-release-2026-11-26.md) and
[CLI completion audit](zellij-cli-completion-audit-2026-11-26.md).
The original opportunity review used `dbdf4fc` plus rc.17 version preparation;
immutable earlier candidates and their dated evidence remain unchanged.

Historical PDF sequence: the [offline feasibility probe](zellij-cached-pdf-feasibility-2026-11-22.md)
led to PR #255 (`65249fa`) and the reviewed rc.20 runtime/source pair. Rc.20
required `--pdf-backend cached`; the generated-CI and book-template transition
shipped in rc.21. Rc.24 retains the isolated compiler's scoped Wasm code-GC
mitigation (`5425125`). These milestones supersede the earlier proposals below.

## Existing behavior

Rich `file` and HTTP `static` build, check, validation watch and `view --watch` use a verified,
version/platform-specific renderer under `ZELLIJ_HOME`, defaulting to
`~/Dev/config/zellij`. It includes a private Node executable: users do not
install Node, npm, Python or a container runtime for these operations. The Rust
CLI launches that executable by absolute path and retains offline/cache-integrity
and staged-output guarantees. `serve` is native and needs no renderer download.

Portable builds/checks, scaffolding, imports, formatting, schema/context commands
and CI-file generation are already native. MkDocs/Sphinx configuration is parsed
as supported data rather than executed in Python. Python is not a general
end-user prerequisite of the Rust CLI. Building Zellij itself and its release
acceptance suites still use the maintainer toolchain; this proposal concerns
installed CLI users. CI generation locates its repository with Git; `zel ci run`
uses the GitHub CLI for workflow dispatch. These requirements are separate from
rendering dependencies.

## Migration disposition — all selected priorities released

| Priority | Commands or path | Reusable artifact | Work required |
| --- | --- | --- | --- |
| 1 — released in rc.18 | Explicit rich `check --target static` and matching validation watch | Existing cached validator | Reuse nonexecuting shared validation while preserving `static` report identity, JSON/error semantics and configured targets. The released path uses the cached validator, including optional import validation; historical rc.17 selects ContainerEngine. |
| 2 — released in rc.18 | `build --target static`, `view --target static`, build/watch and static-site deployment build steps | Extended cached renderer with a supported HTTP export mode | Preserve actual hosted routing, base paths, metadata, 404 behavior, search, no-JavaScript output and asset URLs. Do not silently replace the Next export contract with file-oriented link rewriting. |
| 2, downstream — released in rc.18 | Netlify, Cloudflare Pages, GitHub Pages, S3 and supported static Vercel deployments; corresponding generated CI | Same HTTP export plus existing native provider clients | Native HTTPS upload adapters already exist; the shared rich-site build now uses the cache. Test the build/protocol boundary with repository-owned fixtures; live provider acceptance remains separately deferred. |
| 3 — released in rc.19 | Default `eject` and engine selection for `new app` | Separate versioned wrapper sources plus built engine package | The source update removes default stopped-container extraction using a separate verified archive while preserving vendoring, features, starter content and staged-path checks. The compiled renderer remains separate from the editable-source distribution. Explicit custom `--image` extraction must retain its image semantics; running the extracted Next app still needs its development toolchain. |
| 4 — released in rc.20; default since rc.21 | `pdf`, `build --pdf` / `bp`, configured PDF preview/book exports | Optional reviewed LuaHBTeX WebAssembly compiler, fonts and resources; reuses the renderer's private Node | Cached PDF is now the default. Generated CI and versioned book templates reflect that choice; explicit container tests remain. Rc.24 verifies complete matching sources and offline PDF/rich builds on all six native targets and retains the scoped compiler mitigation. |
| 5 — released in rc.24 for stock development | Rich Next.js `dev` | Separate optional dependency package, reusing private Node and matching editable sources | Cold/offline development, edit recovery, integrity, cancellation and session/port cleanup pass on six native targets. This preserves real Next development for the stock wrapper. Arbitrary custom dependencies still require their own development toolchain; a fixed bundle cannot resolve them. `view --watch` / `vw` remains a static-export preview. |
| Retain builder | `image static`, `image next`, image publishing and image-based Fly/Railway delivery | Rendering may reuse caches, but images still need a builder | A downloaded renderer does not build an OCI image or start a daemon. Removing the local builder would be a separate remote-builder or native-image-building feature. |

## Generated workflow comments — released in rc.19

Rc.18 jobs already omitted container setup for web-only builds, but the banner
still mentioned an engine image. Rc.19 corrected both generators without changing
immutable rc.18 assets. Rc.21 subsequently updated PDF defaults and generated-CI
coverage. Current generators retain builder guidance for image operations and
explicit container paths. No banner migration remains open.

## Implementation boundaries

Keep `zel` small. Download optional capabilities only when requested, pin
versions and supported platforms, include license notices, verify archive and
extracted-file integrity, support an offline prepared cache and avoid global
PATH/package-manager mutations. Toolchain cache paths must remain under the
resolved `ZELLIJ_HOME`; installed executables continue to respect Homebrew/Scoop
ownership. Reuse the existing renderer cache for operations it already supports.

The released layout uses capability-specific `cache/engines`, `cache/sources`,
`cache/pdf` and `cache/development` directories. The original generic
`cache/toolchains/<tool>/<version>/<platform>/` sketch was not adopted and is not
an accepted configuration path. Additional tools should reuse the existing Node
runtime when compatible.

Each migration needs output-equivalence tests for its promised target, an
empty-PATH first download, offline reuse, interrupted/corrupt downloads,
cancellation and failed-output preservation on all six supported native targets.
Optional typesetting must not enlarge ordinary web downloads or change authored
content. Keep documented prerequisites accurate until a migrated command is
released and its artifacts have passed public-download acceptance.

## Editable-source package — implemented in rc.19

A maintainer-only `npm pack --dry-run --ignore-scripts --json` at source
`15b6625` inventories the built engine at 7,560,437 compressed bytes,
22,599,158 unpacked bytes and 1,421 entries, with no bundled dependencies.
This is a feasibility measurement, not a published optional package.
A wrapper/source download can therefore remain separate from the ordinary
renderer and need not include another Node runtime.

The rc.19 implementation packages engine sources, built CLI entry points,
fonts/licenses and the matching editable wrapper. Default ejection
can consume it; explicit custom-image extraction must retain its selected-image
behavior. Cached sources should be copied into the new wrapper's vendor folder
so package installation cannot mutate the shared cache. Explicit local
`new app --engine` inputs retain their current validation and semantics.

Require CLI-version matching, archive and per-file integrity, portable path and
symlink checks, offline cache use, cancellation and staged destination publication.
Run the resulting wrapper's install/check/build outside the repository on all
supported platforms. Removing extraction prerequisites does not remove Node/npm from running a
custom editable Next app. Rc.24 removes those system prerequisites for the stock
`zel dev` workflow using its separately verified dependency package.

## Source map

- `cli/crates/zellij-cli/src/rich_build.rs` and `rich_build/cache.rs`: private runtime, build/check requests and verified cache.
- `cli/crates/zellij-cli/src/validation.rs` and `watch.rs`: target-specific validation.
- `cli/crates/zellij-cli/src/main.rs`: build target dispatch.
- `cli/crates/zellij-cli/src/preview/rich.rs`: explicit HTTP export/preview.
- `cli/crates/zellij-cli/src/delivery.rs`: shared static build before native deployment.
- `cli/crates/zellij-cli/src/eject.rs` and `app.rs`: default cached-source wrappers, explicit stopped-image extraction and local-engine validation.
- `cli/crates/zellij-cli/src/source_package.rs`: versioned source retrieval, file inventory/integrity, offline reuse and staged publication.
- `scripts/package-editable-sources.mjs` and `check-editable-sources.mjs`: universal source packaging and six-platform installed-wrapper acceptance.
- `cli/crates/zellij-cli/src/development.rs`, `development/cached.rs` and `development/process.rs`: cached/container dispatch, stock wrapper preparation, supervised Next process and cleanup.
- `cli/crates/zellij-cli/src/development_package.rs`: optional dependency package integrity and offline retrieval.
- `scripts/check-cached-development.mjs`: installed cold/offline development acceptance.
- `cli/crates/zellij-cli/src/pdf.rs` and `pdf_package.rs`: container/cached PDF dispatch, reviewed package integrity and private Node reuse.
- `scripts/pdf-wasm/`: isolated GPL-2.0-only adapter, original component review, source correspondence and paired release packaging.
- `scripts/check-published-cached-pdf.mjs` and `verify-pdf-release.mjs`: public native installs, complete source transfers and compiler/renderer/offline acceptance.
- `engine/packages/zellij/src/offline/standalone.ts`: existing build/check entry point.
- `scripts/build-standalone-engine.mjs` and `package-standalone-engine.mjs`: compilation and per-platform runtime packaging.
