# Standalone rich website builds

**Released 2026-11-19:** PR #211 merged as `73979de`, exactly matching the
35-job passing source tree at `5a03e13`. Immutable `v0.2.0-rc.12` is published
in both repositories. All 38 release jobs pass, including the six native
standalone builds and final public-artifact acceptance. Homebrew and Scoop
have each passed four public lifecycle/actual rc.11-to-rc.12 upgrade jobs.
The installed Mac CLI also passes first-download and byte-identical offline
builds with an empty PATH, followed by a clean Chromium/WebKit file crawl.
The final published Mac Intel archives are 10,026,371 bytes (CLI) and
48,098,596 bytes (separately cached renderer).

Updated public guides in PR #212 pass strict/deep validation, a 59-route
Chromium/WebKit file crawl, all 35 general CI jobs and six build-only consumers.
They merge as `de409a6` with an identical tested tree. Automatic production run
`44645212486` passes both jobs and verifies deploy `6aff93c9718dfd08ed1c874c`.
The public build guide independently matches the checked artifact. See
[release evidence](evidence/zellij-rc12-release-2026-11-19.json).

All 224 website and 420 PDF-page captures match source verification. Six of 56
additional book-browser captures differ in theme-icon shading or PDF-link focus
outlines; their precise cause remains unresolved. They are recorded separately,
not silently counted as pixel matches. Live-provider checks, reviewed old-candidate
retirement and migration of the private HP handoff scripts remain separate tasks.

The older checkpoints below are retained as history and no longer describe
whether the CLI, renderer and updated public guides are available.

## Earlier implementation checkpoints

Status: rc.12 implementation in progress, based on main `fbcb558`. No public
renderer assets have been published. rc.11 is unchanged.

Final-source CI `44628797826` at `d3cc2d5` passed 34 of 35 jobs, including all
six native renderer packages, the actual packaged Linux file-browser test,
Homebrew/Scoop and the engine/site/book checks. Its final CLI integration job
found four inline HTTP-export invocations that still relied on the old default:
the preservation test attempted to download the not-yet-published rc.12 renderer
and received 404. These invocations now explicitly select `--target static`,
including the following all-section template export. No application code changes
were needed. A new complete CI run is required before merge and publication.
The prepared rc.12 public guides also pass a separate 59-route Chromium/WebKit
file crawl using the exact earlier CI-built shared renderer; they are not live yet.

The user requires the complete rich website, opened through `file://`, from
Markdown/MDX/YAML and `zel.yaml`, with only `zel` installed. The CLI stays small;
a separate renderer is downloaded on first use and shared across sites. It is
not the native manual-style Markdown output and does not hide a container.

## Design

- `rich_build.rs` prepares a safe staged output, obtains the matching renderer,
  launches its absolute private runtime path, then atomically publishes success.
- `rich_build/cache.rs` selects six platform variants, downloads over HTTPS,
  verifies the archive checksum before extraction, rejects unsafe paths/links,
  bounds download/expansion sizes, and verifies every manifest-listed file
  before execution. Cached corruption is an error with recovery guidance.
- `offline/standalone.ts` uses the shared content loader, validator, asset sync,
  search generator, section renderer and rich offline exporter. It generates
  the page shell and metadata without compiling a Next application.
- `build-standalone-engine.mjs` compiles dependencies, styles and browser code
  on our build machines. `package-standalone-engine.mjs` adds the checksum-pinned
  official Node runtime and full runtime/dependency/font notices. No package
  manager, compiler or archive executable is invoked on the user's machine.
- `zel-renderer-vVERSION-PLATFORM.tar.gz` and `.sha256` accompany each native
  release in both repositories. The default download uses the long-term shared
  `mylife-inc/releases` repository, scoped tag `zellij-vVERSION`.

Hosted `zel build` defaults to `file`. Existing portable inference is unchanged.
`--target static` retains Next HTTP export; images, hosted check/development and
PDF typesetting retain their existing prerequisites. `--offline` is specific to
rich builds and refuses an absent cache. `zel doctor` inside a rich site reports
cache readiness without downloading or executing it; outside a site it retains
container diagnosis.

Cache locations and override/mirror behavior are documented in the public build
reference. First use needs internet. A prepared cache can be transferred with
the exact CLI/platform for a disconnected handoff. MDX is trusted local code,
not sandboxed. External videos/URLs still need their external services. Custom
Next application code and arbitrary Tailwind compilation remain wrapper features.

## Verification

Local Mac Intel development checks initially passed the actual CLI download,
cached offline rebuild, failed-content preservation and cache tamper rejection.
The 16-route showcase opens in Chromium and WebKit through file:// with zero
HTTP requests, page errors or failed assets; search navigation also passes.
The expanded driver adds download-checksum rejection and all 25 section types
through the full starter. Full Rust, workflow and browser checks are in progress.

Six native package jobs run this driver with an empty PATH on Ubuntu x64/ARM64,
macOS Intel/ARM64, and Windows x64/ARM64. A separate Linux browser job verifies
the exact generated output. Published acceptance repeats the driver against
both public release URLs using the installed public CLI. These hosted/public
results are pending and must be recorded before claiming release readiness.

The local compressed Mac Intel renderer is approximately 48.1 MB. It is separate
from the native CLI archive. Measure final published sizes before reporting them.
No claim of universal OS compatibility or zero bugs is made.

## Local and initial hosted verification update

Implementation commits `becc1ed` and `d4e9b9e` are under review in
[PR #211](https://github.com/mylife-inc/Zellij/pull/211). The complete local Rust
workspace passes 820 tests with five existing hosted-only tests ignored; Clippy
and types pass. Engine tests pass 537 cases, plus three new route-safety cases.
The standalone driver now covers the 28-route full starter (25 section types
and a math guide), checksum rejection, cache tampering, failed-output retention,
route traversal and base-path/site-URL overrides. Its child PATH is empty.
Chromium and WebKit pass the 16-route prefixed showcase and the complete
59-route documentation with zero captured page errors, failed assets or HTTP
requests. The documentation's Openers page intentionally demonstrates two
additional H1s; its local crawl expects those three authored headings.

Testing the real documentation exposed MathJax's dynamic package-version lookup
inside a bundled module. The publication compiler now defines MathJax's supported
`PACKAGE_VERSION` constant from its installed package. The new full-starter math
assertion requires rendered MathJax SVG, so a silent optional-library fallback
cannot pass. Runtime and font licenses are included in the renderer distribution;
font licenses also accompany the generated website.

Initial CI `44623359389` passes the standalone download/cache/full-section tests
on Linux x64/ARM64, Windows x64 and macOS ARM64 at this checkpoint. Remaining
native jobs are pending. The Engine job correctly caught the standalone image
recipe still selecting rc.11 after the workspace bump; the recipe is corrected
to rc.12 and all 20 local release-policy tests pass. The next run must pass on
the updated final source, including route preflight and shared OG asset resolution.
No release or production publication has occurred for this feature.

The final packaging workflow compiles the platform-independent JavaScript/CSS
once on Linux, then supplies those identical files to all six native jobs.
Each job adds and verifies its platform's official runtime and executes the real
CLI against the resulting package. This removes six redundant npm/TypeScript
builds, including a slow npm installation on the first Windows ARM64 runner;
it does not substitute cross-compilation for native runtime acceptance.

The route/metadata and image-version corrections are committed in `80119b6`.
The Linux-built artifact from corrected CI `44625255756` also builds all 59
documentation routes on Mac; Chromium and WebKit report zero captured page
errors, failed assets or HTTP requests, and the homepage is visually checked.

Distribution review found old container-only wording in Homebrew/Scoop hints
and five rich template READMEs. The hints are corrected. `product-studio`,
`product-launch`, `software-platform`, `creative-studio` and `api-platform` gain
new 0.1.1 identities requiring CLI rc.12; no published 0.1.0 bytes are rewritten.
Only their manifest version/minimum CLI and README instructions change; authored
website content and artwork are unchanged. All nine templates pass deterministic
packaging, local/bundled/remote parity, cache and strict-content acceptance.
All 26 distribution-policy/formula tests pass. Final CI must include these
documentation-bearing archive changes before tagging rc.12.

All six native package jobs in `44625255756` now pass, including Windows ARM64,
using the same Linux-compiled renderer. This is successful pre-publication
platform evidence for `80119b6`, not a final-source or public-release result.

Final integration review also makes generated GitHub/GitLab/CircleCI delivery
checks choose `--target static` for hosted sites or `--target portable` for native
sites. They therefore continue checking the HTTP publication path, independently
of the new interactive default, and do not download an unrelated renderer during
container acceptance. A regression covers both renderers in all three generators.
New rich sites and template creation now suggest `zel build` first; live
development is explicitly optional and retains its container prerequisite.

CI `44627745387` correctly detects saved acceptance-workflow drift after the
generator change in `3ae872f`. Regenerate the four affected verification commands
through `prepare-delivery-acceptance.mjs --write`, retaining their public rc.11
execution pins until rc.12 is actually published. The local drift check and
workflow lint pass after regeneration. This is a saved-fixture correction; no
provider deployment is performed.
