# Cached Next development — 25 November 2026

Status: implementation and qualification in progress on
`codex/cached-next-development`. Current public CLI remains rc.21.

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
read-only dependency references while isolating generated files. Requalification
of that change, browser checks and all six native platforms remains pending.

The initial complete Rust run reported a timeout in the unrelated Cloudflare
large-batch fixture during concurrent native/browser and compilation work
(155 pass, one failure, five expected ignored in the CLI unit binary). Preserve
this initial result; an isolated rerun and the complete suite must pass before
publication. No deployment-provider live tests were performed.

Local Darwin x64 package measurement: 125,407,980 bytes, 30,767 entries after
removing unused Sharp/libvips, generated Mermaid/Lucide maps and the Cytoscape
fCoSE demo. The runner revision can change this count/size. No additional Node
binary is included. Other native sizes are not measured yet.

The six-platform native workflow packages and runs cold download, offline
restart, invalid edit recovery, port/startup failures, tamper rejection,
cancellation and cleanup. Linux x64 additionally runs the full browser reload,
search, image, maths/Mermaid and font checks. Public release acceptance repeats
native development against each published destination. These gates must pass
before calling rc.22 released or complete.
