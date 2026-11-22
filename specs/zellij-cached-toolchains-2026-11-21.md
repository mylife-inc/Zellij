# Cached toolchain opportunities — 21 November 2026

Status: the first validation migration is implemented in source after rc.17;
acceptance and release tracking are in the [validation record](zellij-cached-static-validation-2026-11-21.md). The HTTP migration is also implemented in source; see its [acceptance record](zellij-cached-http-export-2026-11-21.md). Wrapper, PDF and Next development packages remain proposed work.
Reviewed source: `dbdf4fc` plus the version-only rc.17 preparation. The rc.17
candidate ships the accepted overflow-focus correction, not the migrations below.

## Existing behavior

The rich `file` build, check, validation watch and `view --watch` use a verified,
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

## Candidate migrations

| Priority | Commands or path | Reusable artifact | Work required |
| --- | --- | --- | --- |
| 1 — implemented in source | Explicit rich `check --target static` and matching validation watch | Existing cached validator | Reuse nonexecuting shared validation while preserving `static` report identity, JSON/error semantics and configured targets. The source path now uses the cached validator, including optional import validation; released rc.17 still selects ContainerEngine. |
| 2 — implemented in source | `build --target static`, `view --target static`, build/watch and static-site deployment build steps | Extended cached renderer with a supported HTTP export mode | Preserve actual hosted routing, base paths, metadata, 404 behavior, search, no-JavaScript output and asset URLs. Do not silently replace the Next export contract with file-oriented link rewriting. |
| 2, downstream — implemented in source | Netlify, Cloudflare Pages, GitHub Pages, S3 and supported static Vercel deployments; corresponding generated CI | Same HTTP export plus existing native provider clients | Native HTTPS upload adapters already exist; the shared rich-site build is the remaining container dependency. Test the build/protocol boundary with repository-owned fixtures; live provider acceptance remains separately deferred. |
| 3 | Default `eject` and engine selection for `new app` | Separate versioned wrapper sources plus built engine package | Default ejection currently copies sources from a stopped container. A complete editable package could remove that extraction dependency while preserving vendoring, features, starter content and staged-path checks. The current compiled renderer archive is not an editable source distribution. Explicit custom `--image` extraction must retain its image semantics; running the extracted Next app still needs its development toolchain. |
| 4 | `pdf`, `build --pdf`, configured PDF preview/book exports | Optional platform-specific LuaLaTeX/typesetting package and supported fonts/packages | Preserve the publication model, index, references, languages and appearance. Match current LuaLaTeX capabilities, failure preservation, cancellation, resource limits and isolation; a different TeX engine is not an automatic drop-in replacement. |
| 5 | Rich Next.js `dev` and editable wrapper workflows | Optional larger Node/Next/dependency package | Preserve actual Next development behavior and custom app support. `view --watch` already offers a lighter native preview. Arbitrary custom dependencies require their own resolution and cannot be inferred from a fixed template bundle. |
| Retain builder | `image static`, `image next`, image publishing and image-based Fly/Railway delivery | Rendering may reuse caches, but images still need a builder | A downloaded renderer does not build an OCI image or start a daemon. Removing the local builder would be a separate remote-builder or native-image-building feature. |

## Implementation boundaries

Keep `zel` small. Download optional capabilities only when requested, pin
versions and supported platforms, include license notices, verify archive and
extracted-file integrity, support an offline prepared cache and avoid global
PATH/package-manager mutations. Toolchain cache paths must remain under the
resolved `ZELLIJ_HOME`; installed executables continue to respect Homebrew/Scoop
ownership. Reuse the existing renderer cache for operations it already supports.

A future package layout for genuinely additional tools can use
`cache/toolchains/<tool>/<version>/<platform>/`; this is a proposal, not a path
recognized by rc.17. Do not introduce a second Node download when the existing
renderer runtime can safely serve the operation.

Each migration needs output-equivalence tests for its promised target, an
empty-PATH first download, offline reuse, interrupted/corrupt downloads,
cancellation and failed-output preservation on all six supported native targets.
Optional typesetting must not enlarge ordinary web downloads or change authored
content. Keep documented prerequisites accurate until a migrated command is
released and its artifacts have passed public-download acceptance.

## Next package: editable sources

A maintainer-only `npm pack --dry-run --ignore-scripts --json` at source
`15b6625` inventories the built engine at 7,560,437 compressed bytes,
22,599,158 unpacked bytes and 1,421 entries, with no bundled dependencies.
This is a feasibility measurement, not a published optional package.
A wrapper/source download can therefore remain separate from the ordinary
renderer and need not include another Node runtime.

The next implementation should package complete engine sources, built CLI
entry points, fonts/licenses and the matching editable wrapper. Default ejection
can consume it; explicit custom-image extraction must retain its selected-image
behavior. Cached sources should be copied into the new wrapper's vendor folder
so package installation cannot mutate the shared cache. Explicit local
`new app --engine` inputs retain their current validation and semantics.

Require CLI-version matching, archive and per-file integrity, portable path and
symlink checks, offline cache use, cancellation and staged destination publication.
Run the resulting wrapper's install/check/build outside the repository on all
supported platforms. Removing extraction prerequisites does not yet remove
Node/npm from running an editable Next app; that is the separate development
package milestone below the PDF work in the priority table.

## Source map

- `cli/crates/zellij-cli/src/rich_build.rs` and `rich_build/cache.rs`: private runtime, build/check requests and verified cache.
- `cli/crates/zellij-cli/src/validation.rs` and `watch.rs`: target-specific validation.
- `cli/crates/zellij-cli/src/main.rs`: build target dispatch.
- `cli/crates/zellij-cli/src/preview/rich.rs`: explicit HTTP export/preview.
- `cli/crates/zellij-cli/src/delivery.rs`: shared static build before native deployment.
- `cli/crates/zellij-cli/src/eject.rs` and `app.rs`: stopped-image source extraction and local wrapper/engine validation.
- `cli/crates/zellij-cli/src/development.rs`: rich Next development server.
- `cli/crates/zellij-cli/src/pdf.rs`: isolated LuaLaTeX execution.
- `engine/packages/zellij/src/offline/standalone.ts`: existing build/check entry point.
- `scripts/build-standalone-engine.mjs` and `package-standalone-engine.mjs`: compilation and per-platform runtime packaging.
