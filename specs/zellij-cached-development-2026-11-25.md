# Cached Next development — 25 November 2026

Status: implementation `3d7ab74`, with harness/release-recipe corrections in
`cc193d6`, is in draft PR #260. Initial local Webpack qualification passed;
subsequent native CI exposed issues described below. Revised local Turbopack
qualification passes; six-platform qualification remains in progress. Current public CLI remains rc.21.

## Product scope

The user authorized continuing toolchain removal. Rich `zel dev` now defaults
to an optional, exact-version native dependency package. It reuses private Node
from the renderer cache and the existing editable-source package. No Docker,
Node/npm or Python installation is needed by consumers. Dependencies are only
downloaded for Next development; ordinary web builds keep their smaller package.

`--dev-backend container` or `ZEL_DEV_BACKEND=container` preserves the existing
container workflow and explicit image selection. `--offline` requires verified
renderer, source and development packages and rejects the container backend.
Portable development keeps its native preview path.

This is the stock documentation wrapper. Custom application code/plugins and
arbitrary npm dependencies in ejected Next apps remain separate workflows.
Images are served unoptimized, preserving the original files used by static
exports. This lets the optional package omit Sharp/libvips entirely.

## Design

- Six native dependency archives, with archive and complete-file SHA-256
  verification, exact CLI/platform/Node-major matching, bounded extraction,
  portable paths, and rejection of links, aliases and unlisted cache entries.
- `$ZELLIJ_HOME/cache/development/<version>-<platform>/` stores dependencies.
  Each run creates a disposable wrapper under `cache/development-sessions/`.
  Generated assets, feature selection, TypeScript configuration and `.next`
  belong to the session. Verified dependencies are shared via directory links
  (junctions on Windows); no package manager runs against them.
- The private runner waits for a startup handshake. Rust first owns a Unix
  process group or a Windows kill-on-close job, then permits descendants.
  Stdin EOF requests graceful shutdown; cleanup also terminates descendants.
- The runtime uses the standard Turbopack development bundler. Its helper
  processes receive a private PATH containing only the verified Node directory;
  the user's PATH is not modified. A narrowly scoped additional root exposes
  the verified dependency package to the isolated wrapper.
- The existing exact-session readiness marker prevents opening another service
  on an occupied port. Content watching retains successful preparation during
  invalid edits and resumes after correction.
- Platform packages retain dependency licenses and source references. The
  separate PDF adapter's license scope is unchanged.

## Qualification evidence and findings

Four local package safety tests pass, as do Clippy, the dependency lock audit
(zero reported vulnerabilities) and deep/strict documentation validation.
The first debug-binary acceptance hit its startup deadline while hashing and
extracting dependencies; it did not establish end-to-end acceptance.

The first optimized run served the actual Next site with an empty PATH, updated
search/assets, preserved the prior preparation during an invalid edit, and
recovered. It then exposed slow cleanup from copying over 30,000 dependency
files into every session. The implementation was changed to share verified
read-only dependency references while isolating generated files. The revised layout passes local Darwin x64 acceptance: cold download with an
empty PATH, real Next pages, edits/recovery, cancellation at checksum headers,
checksum body and archive body, partial-cache cleanup, offline restart, occupied
ports, invalid initial content, cache tamper rejection and full session cleanup.
Chromium also passes automatic reload, search, image-only changes, maths/Mermaid,
all 13 local theme fonts, added/deleted pages and invalid-edit retention without
captured browser errors. Native qualification on all six platforms is pending.

The initial complete Rust run reported a timeout in the unrelated Cloudflare
large-batch fixture during concurrent native/browser and compilation work
(155 pass, one failure, five expected ignored in the CLI unit binary). Preserve
this initial result; the complete rerun with two test threads passes **863 tests**, with five
expected skips. Clippy passes after the session-layout change. No deployment-provider live tests were performed.

Local Darwin x64 package measurement: 125,409,475 bytes, 30,767 entries after
removing unused Sharp/libvips, generated Mermaid/Lucide maps and the Cytoscape
fCoSE demo. This is the shared-dependency runner package. No additional Node
binary is included. Other native sizes are not measured yet.

The six-platform native workflow packages and runs cold download, offline
restart, invalid edit recovery, port/startup failures, tamper rejection,
cancellation and cleanup. Linux x64 additionally runs the full browser reload,
search, image, maths/Mermaid and font checks. Public release acceptance repeats
native development against each published destination. These gates must pass
before calling rc.22 released or complete.

The inherited-container setting was also scoped to hosted development, so a
portable `zel dev --offline` session remains native even when
`ZEL_DEV_BACKEND=container` is set. The existing host/browser fixture now checks
this combination and passes, including IPv4/IPv6 and real Ctrl-C cleanup.

First PR CI findings: a formatting-only diff and the standalone Docker recipe's
old rc.21 default were corrected in `cc193d6`. The local 20-test release-policy
suite passes. The current CI run must still finish before merge/publication.

Native CI `45745470433` at `cc193d6` passed Linux ARM64 and both Macs with the
initial Webpack runner. Linux x64 browser acceptance exposed a real HTTP 500
while reloading an image: Next's manifest reader reported an incomplete JSON
file. Windows x64 also failed to finish the first page compilation within the
acceptance deadline; the captured log did not establish its cause. These failures
are not waived or hidden by retries. The runner now follows ordinary wrappers'
Turbopack default, and browser acceptance exercises repeated image changes with
concurrent page requests. The first local Turbopack trial proved that its CSS
worker requires private Node on the child PATH; that is now supplied without
adding any system prerequisites. Revised qualification remains required.

Windows additionally exposed a manifest checker that compared CRLF checkout
bytes with generated LF bytes. The checker now normalizes only CRLF, preserving
exact content validation. A synthetic CRLF checkout passes locally. Dependency
packaging uses a fail-fast Bash step on every runner, so a failed manifest check
cannot be obscured by a later successful command. Quiet startup/shutdown and
portable offline development with an inherited container preference also have
explicit regression coverage.

The revised Darwin x64 Turbopack run passes the complete native and Chromium
acceptance: cold/offline/quiet sessions with an empty system PATH, cancellation,
cache tampering, invalid content, occupied ports, fonts, maths/Mermaid, search,
page additions/deletions, and four consecutive image reloads with concurrent
HTTP page requests. All requests return HTTP 200, and no browser errors are
captured. Post-change Clippy and JavaScript syntax checks pass. The revised
local archive measures 125,407,493 bytes with 30,767 files. These results do not
substitute for the outstanding Linux and Windows native checks.

At `e7d6a98`, native CI `45749096647` passes both Linux architectures (including
the expanded Chromium reload test on x64) and both macOS architectures. Windows x64 and ARM64
stop during archive creation: the fail-fast Bash step selects Git's GNU tar,
which interprets drive letters as remote hosts. The packager now explicitly
selects Windows' native System32 tar, following the existing renderer packager.
This is a maintainer packaging correction; consumers never need tar installed.
The manifest CRLF checks passed on both Windows targets. Native CI now exercises
the development package before the longer independent editable-wrapper build,
and retains Rust compilation caches after failed acceptance. All existing checks
remain required. Windows runtime qualification is still outstanding.

The source review also found obsolete container requirements in five rich
website starter READMEs and included guides. These now describe cached HTTP
builds and Next development at new immutable versions: product-studio and
api-platform 0.1.3; product-launch, software-platform and creative-studio 0.1.4.
Their minimum CLI is rc.22. Older published template identities and all book
starter bytes remain unchanged. Template qualification accompanies this update.

Local template qualification passes all six CLI integration tests and the full
nine-starter driver: deterministic archives, local/bundled/remote parity, exact
version selection, offline reuse and strict content validation. Previously
published book and handbook archives match their recorded immutable hashes.
The separate PDF qualification run `45749096450` at `e7d6a98` passes all nine
jobs; no PDF compiler/adapter behavior is changed by the Windows tar correction
or starter instructions.

At `f231e11`, both Windows archives package successfully. Native run
`45751882986` still fails the first-page deadline on x64 and ARM64 after Next
reports readiness; both Turbopack and the earlier Webpack runner show this
symptom. The captured output does not yet distinguish a compiler stall, slow
compilation, dependency-path behavior or repeated cancellation by the fixture's
10-second HTTP polling deadline. A separate opt-in Windows probe compares
`none`/`all` features, records CPU/generated-output progress and holds one
uninterrupted request for at most ten minutes. Normal native qualification keeps
its existing deadline and complete lifecycle checks; probe success alone is not
release acceptance.

PDF run `45751882812` also records an unrelated Ubuntu comparison failure:
Docker and Podman `info` calls and the local Docker socket probe time out. The
identical PDF behavior passed all nine jobs in `45749096450`; the failed run is
retained, and required final qualification must still pass before publication.
The diagnostic matrix additionally compares an ASCII path without spaces with
the Unicode/spaces fixture. The diagnostic-only path passes locally on Darwin
x64 with features disabled: the uninterrupted first page takes 13.4 seconds,
and process/session cleanup succeeds. That result validates the probe, not a
Windows fix. No runtime code or ordinary acceptance deadline is changed.

Diagnostic run `45755240637` fails all three Windows cases. With one continuous
request, generated output and compiler CPU stop changing after the first minute;
plain paths and disabled optional features make no difference. Node's HTTP
client ends each request at its own five-minute header limit, before the
diagnostic's intended ten-minute limit. This is not evidence of slow compilation
or an automatic source-scanning loop. The next diagnostic isolates copied
dependencies, a common Turbopack root and CSS processing; variant packages are
confined to the opt-in probe and never uploaded or included in releases.
Requests are now bounded to three minutes, with normal acceptance unchanged.
The Quickstart's final folder diagram and remaining source-candidate development
requirements are corrected without claiming rc.22 is already published.

Run `45757025081` also stalls with a common compiler root. Its first copy probe
stops before Next because a diagnostic replacement missed CRLF; this is fixed
and checked with a synthetic CRLF runner. Its CSS probes were not conclusive:
one still imported font CSS, and the other accidentally bundled native addons
by importing Tailwind directly. The corrected run `45757908338` confirms both
stylesheets process quickly (about 113 ms total); removing both CSS imports and
capturing output to files still leave the request stalled. No CSS/runtime
packaging change is justified by these results. Further probes compare a minimal
page, application render markers and detaching Next from the parent control pipe.
