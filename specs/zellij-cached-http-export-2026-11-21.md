# Cached HTTP export — 21 November 2026

Status: implemented in source for rc.18; hosted acceptance is being completed
before the candidate is published.

The downloaded renderer now supports explicit `file` and `static` modes. Both
use the same precompiled components, private Node runtime, fonts and browser
bundle. File mode retains relative double-click navigation. Static mode emits
flat HTTP pages, prefixed navigation/assets, metadata, sitemap/robots, a 404 and
redirects that work without JavaScript. Configured analytics are included only
in HTTP mode. No user-installed Node, npm, Python or container runtime is used.

`build --target static`, its aliases and `view --target static` now use this
cache; both support `--offline`. Preview shares the existing native HTTP server,
serial watch loop, pinned layout/base path, cancellation and staged publication.
The CLI rejects an older renderer's file output rather than publishing it as a
static export. Unsafe/colliding flat route paths are rejected before page writes.

Static Netlify, Cloudflare Pages, Vercel, GitHub Pages and S3 delivery reuse this
HTTP build. Existing native provider preflight and upload behavior is retained.
Exports stage in an OS temporary directory and are mounted at the resolved
prefix before upload; GitHub Pages/S3 retain their provider-specific mapping.
An unsuccessful build does not begin deployment or change previous local output.
Image build/push and Fly/Railway image delivery still need a builder.

Workflow generation omits container setup/preflight for web-only workflows
pinned to rc.18 or later. Earlier pins retain their original prerequisite.
PDF/image workflows retain container setup. Generation does not promise that an
arbitrary version pin has been published. Public rc.17 remains unchanged; guide
additions are explicitly labelled upcoming.

Local verification currently passes:

- 832 Rust tests, with 5 explicitly gated renderer integrations; Clippy with
  warnings denied and formatting.
- All 544 engine tests, engine/showcase/docs type checks and strict/deep docs
  validation (59 routes).
- Cached validation/export integration, including nested routes, anchors,
  encoded assets and diagnostic/JSON behavior.
- Real HTTP browser preview: nested prefix, home redirect, navigation/search,
  maths/diagrams, features off, rebuild/recovery, previous-output preservation,
  source/config preservation and shutdown. Separate no-JavaScript checks cover
  reading, navigation, redirect and a 404 with a prefixed home link.
- Cached-renderer protocol preview: pinned layout, stale-input rejection,
  malformed outputs, cancellation during initial build/rebuild, and cleanup.
- All five real-renderer/synthetic-provider integrations, including prefix
  mounting, static assets/runtime, native uploads and failed-build isolation.
  These are local API fixtures, not live provider verification.

Initial HTTP integration found missing `next/link`/router base-path application
and missing deployment-directory prefix mounting. Both were corrected and the
unchanged route/upload contracts pass. The first protocol preview used the old
15-second container-fixture readiness bound; verifying a copied Node executable
under concurrent debug tests exceeded it. Its limit now permits 60 seconds,
while still checking actual readiness, publication and shutdown. Original
results are retained; no historical browser timing finding is closed here.

The Intel Mac packaged renderer also passes empty-PATH cold download, both
export targets, offline reuse, all 28 starter pages, route-collision rejection,
metadata and analytics-mode checks, cache integrity, and file/static preview in
Chromium and WebKit. Both targets use one renderer download. The source test
initially retained version rc.17; it did not alter public rc.17 assets. The rc.18
version bump and final hosted matrix are being checked next. No new release or live provider action is
claimed by this record.
