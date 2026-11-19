# Standalone rich website builds

Status: rc.12 implementation in progress, based on main `fbcb558`. No public
renderer assets have been published. rc.11 is unchanged.

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
