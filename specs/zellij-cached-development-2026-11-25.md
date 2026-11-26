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
