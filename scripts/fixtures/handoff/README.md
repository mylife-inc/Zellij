# Bundled documentation handoff scripts

These generic wrappers accompany a handoff containing `zellij/zel.yaml`,
`zellij/content/`, `site/` and the four Mac/Windows CLI binaries named in the
scripts. They are not automatically emitted by `zel create`.

The default operation builds the rich, double-clickable website with diagrams.
`check`, `watch`, `view` and `serve` forward options to the matching CLI command.
No client documentation, names, assets or credentials belong in this fixture.

`scripts/check-handoff-scripts.mjs` downloads a checksum-verified public release
and tests these exact scripts with a disposable two-page documentation project
(three public routes including the root home alias).
The Windows matrix covers x64/ARM64 and Windows PowerShell 5.1/PowerShell 7.
This checks the scripts and public CLI, not any private client collection.

Run the **Handoff script acceptance** workflow against a published CLI version.
The harness requires Node and (for Windows console testing) Rust on the test
runner. The handoff's build/check/watch/view processes run without those tools
on PATH; users only need the bundled CLI and its downloaded renderer.
