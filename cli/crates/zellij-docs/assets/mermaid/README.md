# Portable Mermaid runtime

`mermaid.js`, `licenses.txt`, and `manifest.json` are generated from the repository
lockfile with `node scripts/prepare-portable-mermaid.mjs --write`. The same command
without `--write` verifies byte-for-byte reproducibility in CI. Cargo embeds the
committed files; the installed `zel` binary never runs Node or downloads anything.
The classic IIFE contains every diagram chunk and KaTeX. ES modules and dynamic
chunk loading would not work reliably from a relocated `file://` export.

The entry point is `../mermaid-client.js`. Only pages with enabled Mermaid fences
load the script. The approximately 3.5 MB uncompressed bundle is a deliberate
offline tradeoff. Published sites also include this bundle's third-party notices
and provenance manifest. Installed dependency notices are gathered from exact
locked packages; fastdom's MIT notice is in its README.

## Upstream embedded code review (2026-10-17)

Mermaid 11.17.2's ESM distribution itself embeds js-yaml 4.3.0. npm overrides and
`npm audit` do not inspect that code. The generator replaces its exact, SHA-256
verified module with the installed **js-yaml 4.3.2** exports. This removes the
component versions affected by GHSA-5p4m-2wfm-xmqj and GHSA-2883-xcg3-v3hh from
the portable bundle. This shim applies to the portable bundle, not the hosted
engine's dependency installation. Mermaid's observed YAML callers use JSON_SCHEMA,
which does not enable the ordered-map/merge tags involved in those advisories;
this is dependency hardening, not evidence of an exploit in the hosted renderer.

`@mermaid-js/parser` also embeds Langium, Chevrotain, language-server libraries,
path-browserify and subsets of Lodash. `embedded-notices.json` preserves the
license texts, registry tarball URLs and integrity identifiers of those exact
versions. These notices were obtained from official npm tarballs and verified
against registry hashes. The source-map content for path-browserify (whose map
omits its version) exactly matches npm's 1.0.1 `index.js`; the generator verifies
that hash. Source maps are used for provenance checks, not shipped to readers.

The embedded Lodash 4.17.23 subset omits `template`, `templateSettings`, `unset`,
`omit` and `_baseUnset`, the affected code for GHSA-r5fr-rjxr-66jc and
GHSA-f23m-r3pf-42rh. The generator asserts their absence and records the reviewed
file subset in the manifest. A whole-package advisory still reports that version;
do not describe this as a zero-advisory scan. The other embedded versions and the
locked npm graph had no reported advisories at review time.

On an upgrade, review the upstream source maps, dependency advisories, YAML shim,
and runtime behavior. Update exact-version notices as needed. The generator fails
if the embedded vendor set, vulnerable Lodash subset, or YAML module changes.
Run native, relocated offline browser and live preview acceptance before shipping.
Do not substitute upstream `mermaid.min.js`: it embeds additional dependency
versions independently of Zellij's lockfile and KaTeX override.
