# CLI rc.9 preparation — 2026-11-15

Baseline `b599749`, after the website collection, centered header and mobile
section views. The Rust workspace and standalone image recipe already select
rc.9. Public rc.8 remains unchanged during preparation.

## Scope and verification

Collect the three website starters and shared navigation improvements in
versioned release notes. Add website acceptance on native Linux amd64/arm64 in
both source CI and public release verification. Source mode builds the current
CLI/engine. Public mode checks out the immutable tag, installs checksum-verified
public CLI bytes, anonymously pulls the exact engine tag and runs the tagged
browser driver. Public mode never rebuilds a replacement CLI or engine.

The driver renders all three website starters and reuses the existing palette,
responsive layout, centered-header, content, local-artwork, no-JavaScript,
contrast and motion checks. A separate authored temporary site adds a real
company logo and grouped menu, then verifies owner/product alignment, root and
section views, Back focus, Close, reduced motion and an intercepted external
new-tab destination. Bundled/published template sources remain unchanged.

Public PDF and website checks share the existing install/identity/cleanup logic.
Tests preserve version and image-architecture/digest rejection, token removal,
failed-driver propagation and retained receipts. Older releases without the new
website driver explicitly skip only that capability. Native/template/image/PDF
acceptance remains independently required.

## Publication boundary

Full source CI and local release preflight must pass before an immutable rc.9
tag. Verify all native packages, exact images, public downloads, PDFs and website
captures. Then update Homebrew/Scoop, active docs and saved consumers, and check
actual rc.8-to-rc.9 upgrades before retiring the exact superseded rc.8 artifacts.
Preserve all known template archive hashes and unrelated shared-repository
products. Record observed outcomes and earlier failures; never move a published
tag. Local browser/container/LaTeX execution remains on hosted runners.
Production hosting and live-provider verification remain deferred.

## Source acceptance

All 32 source CI jobs in `43821076647` pass at `abc556a`, including real website
builds and navigation on both Linux architectures. Each retains 85 captures;
the amd64 website captures match the previously reviewed template layouts.
All 103 ordinary template captures remain byte-identical. New light/dark
root/section screenshots are reviewed and retained; ARM samples are reviewed
separately rather than assuming cross-architecture raster identity.

The 34 PDFs retain all 312 reviewed page layouts and 30 TeX sources. Showcase
passes 418 cases without retries (140 conditional skips). Preservation retains
46 original and 13 added routes, 282 original heading IDs, 82 mappings and 29
byte-identical images without captured browser errors. Local release-helper
checks pass all 45 cases, actionlint, syntax, version and eight-template packaging.
See [verification evidence](evidence/zellij-rc9-preparation-2026-11-15.json).

The final records-only update changes no implementation, workflow, template or
runtime inputs from tested `abc556a`. The older-release capability dispatch,
normal release preflight, publication, channel migration and scoped retirement
remain pending.

PR #191 merges as `f03f803`, whose tree matches reviewed `7071bb9`.
The actual rc.8 compatibility dispatch `43824360532` passes both architectures,
explicitly skipping the unavailable website capability. This does not claim
that rc.8 includes the new website collection. Normal release preflight follows
at the merge; release publication and retirement results are recorded separately.

The normal preflight subsequently passes 802 Rust and 491 engine tests; rc.9
is published at the immutable merge. All 37 release jobs and public artifact
comparisons pass. See [the release record](evidence/zellij-rc9-release-2026-11-15.json)
for managed upgrades, documentation migration and retirement.


## Publication follow-through — 2026-11-16

The immutable tag is `f03f803`. All 37 release jobs, public PDF/website comparisons,
managed channel lifecycles and actual rc.8-to-rc.9 upgrades pass. PR #192 merges
updated guides/consumer pins after 32 CI jobs and six consumer checks pass.
Scoped rc.8 retirement and post-cleanup inventory/HTTP checks are complete.
See [release evidence](evidence/zellij-rc9-release-2026-11-15.json) and
[retirement evidence](evidence/zellij-rc9-image-retirement-2026-11-16.json).
The earlier pending statuses above describe the preparation checkpoint.
