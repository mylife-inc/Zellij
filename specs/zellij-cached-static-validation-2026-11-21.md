# Cached static validation — 21 November 2026

Status: implemented in source after rc.17; release remains pending.

`check --target static` (including `c`) and `watch --target static` now use the
same verified cached validator as rich `file` checks. Both accept `--offline`.
No container discovery, user-installed Node or Python participates. A prepared
cache works without network access; first use without `--offline` downloads the
renderer matching the CLI version and platform under the resolved resource home.
The explicit legacy cache override remains supported. `ZELLIJ_IMAGE` continues
to select image operations, but no longer selects the static validator.

The public JSON target remains `static`, with exactly one report and no mixed
stderr. Deep/strict findings, content exit 3, source paths and diagnostic messages
remain intact. Missing or damaged caches are actionable exit-2 preparation
failures rather than exit-4 container discovery failures. Cached engine protocol
errors remain distinct from content errors. Validation does not execute MDX or
write site output. Human static checks return errors to composed import commands,
so `migrate` and `convert --validate` still explain where converted sources were
retained after a failed check. Dry runs do not prepare or execute the renderer.

Validation watch pins renderer/layout and content scope, serializes edits,
retains invalid content for recovery, and cancels the active renderer on Ctrl-C.
It does not start an HTTP server or browser. HTTP static builds/previews,
deployment builds, image commands, Next development and PDF are unchanged in
this milestone. The matching CLI/renderer candidate has not yet been published;
the current public rc.17 behavior is preserved in the guides with a clearly
labelled upcoming-source section.

Local verification: protocol tests and the full Rust suite pass
829 tests with 5 existing ignores. Real current-source cached watch acceptance
passes initial failures, same-size edits, configuration recovery, pinned scope
and Ctrl-C with no tools on PATH. Native-package acceptance now starts its cold
download with an explicit static check and exercises both rich validation targets,
including JSON, MDX nonexecution, strict findings and watch recovery on all six
supported platforms. CI's other validation integrations prepare a cache from
the current checkout, avoiding accidental use of older published renderers.

The first local full-suite attempt found three obsolete import-fixture assumptions
(two runtime-exit expectations and one trailing separator in a mocked source
path). Updating those fixtures preserved converted-source retention checks;
the corrected full suite passes. This is not a claim that unrun hosted gates
have passed or that the historical browser timing findings are resolved.

Packaged public rc.17 renderer acceptance also passes with the modified CLI on
Intel macOS: a cold static check downloads into an empty resource home, then
both rich validation targets, offline builds, watch recovery/cancellation,
cache-integrity/traversal rejection and all 28 starter pages pass with an empty
PATH. This verifies the existing renderer protocol; hosted six-platform
acceptance of the modified CLI remains pending. No release assets were changed.
