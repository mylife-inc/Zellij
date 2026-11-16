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
