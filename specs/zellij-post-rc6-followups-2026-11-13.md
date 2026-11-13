# Implementation follow-ups after rc.6 — 2026-11-13

First implementation follow-up after rc.6 distribution: deterministic runtime
selection in generated container CI. `cli/crates/zellij-scaffold/src/pipeline.rs`
at rc.6 generates hosted GitHub jobs without selecting Docker, even where
registry login and combined-manifest steps already call Docker directly.
`pipeline/external.rs` owns the other generators; retain GitLab's explicit
service endpoint and review CircleCI's machine-runtime selection separately.
Portable web-only jobs must remain free of container prerequisites. Cover
hosted rendering, enabled PDF, static/Next images and multi-architecture jobs.
The precise cause of each short Docker probe timeout remains unrecorded; explicit
runtime selection and a bounded runner preflight should avoid accidental fallback
to a different image store without weakening runtime limits.

Evidence: public PDF job 132127218691 failed on unconfigured rootless Podman;
new acceptance harness 3d26a96 passes against unchanged rc.6 binaries. Generated
Next consumer 43497959746 failed before building when all discovery probes timed
out, then passed one unchanged-head retry. Historical rc.5 consumer verification
also recorded a discovery timeout. Those reruns did not establish a fix; the
generator change and its verification are recorded below.

Implemented by PR #158 (`dd0f620`, matching tested `bd0ae9b`): one shared runtime
requirement covers hosted rendering, enabled PDF and image delivery. GitHub
jobs and CircleCI machine scripts select the default Docker context; GitLab
retains its TLS service endpoint. All container jobs, including the combined
manifest job, check Docker with a 60-second timeout and five-second kill grace
before validation/build/login. Portable HTML and disabled PDF remain native.
The generated GitHub acceptance workflows are refreshed while retaining the
public rc.6 execution pin. Tests cover selection, disabled PDF, `--no-check`,
both architectures, manifests and fail-closed shell execution. All 23 CI jobs in
`43508089027` and six consumer workflows pass without workflow reruns. This
includes real native/combined publication with the generated preflight and
endpoint. See [the evidence](evidence/zellij-ci-runtime-selection-2026-11-13.json).
The original daemon probe delays remain unexplained; this fix prevents selecting
a different runtime/image store. rc.6 is unchanged.

The [header-resize focus finding](zellij-header-resize-focus-2026-11-13.md)
was reproduced and resolved in PR #160 (`dece6c9`). No-retry stress passes
252 targeted and 636 full-workload cases; all 23 PR CI jobs and all 13 development
image jobs pass. A second focus-reset defect during header updates was fixed
with a stable drawer callback. The original failures remain recorded.
Next: resume the distribution feature below.

Next distribution feature: Scoop ownership protection before publishing an
owned bucket. `cli/crates/zellij-cli/src/maintenance/upgrade.rs` currently checks
only an executable in `bin/` with `share/zel/installation-manager`. The Homebrew
contract is verified; a Scoop package layout needs its own deliberate marker
location, accurate upgrade guidance and native Windows install/upgrade/uninstall
acceptance. Do not publish a managed package that the current self-upgrader can
silently replace. An isolated copy of the actual public rc.6 macOS binary refused
self-upgrade with a `bin/zel` plus `share/zel/installation-manager` containing
`scoop\n`, preserving the binary's hash. This supports reusing that deliberate
layout, but is not Scoop/Windows acceptance; accurate Scoop upgrade guidance and
native lifecycle tests still need implementation. Then bundle the fixes and
package support into a new candidate.

Scoop implementation notes from the current official manifest contract: generate
an owned bucket manifest from the immutable release checksum inventory, with
separate `64bit`/`arm64` downloads, `extract_to: bin`, the `bin/zel.exe` shim and a
UTF-8-without-BOM `scoop\n` ownership marker under `share/zel`. This deliberately
reuses the guard already present in rc.6; add specific `scoop update zel` guidance
before the next candidate. Keep shell profile changes opt-in. The next gate is
actual Scoop install, upgrade and uninstall on disposable Windows x64/ARM64
runners, with binary architecture, marker bytes, refused self-upgrade and a
portable build verified. Only then publish the owned bucket and installation
guide. No bucket has been created by the header-focus work.

References checked on 2026-11-13:
[app manifest properties](https://github.com/ScoopInstaller/Scoop/wiki/App-Manifests)
and [install hook variables](https://github.com/ScoopInstaller/Scoop/wiki/Pre-Post-(un)install-scripts).

Remaining book work: numbered equations/theorems and shared references,
bibliography/index/glossary, multilingual/RTL print, Next-image PDFs, and
technical/study/custom print templates. Existing elementary math is implemented.
Live provider and external-runner verification, production-domain migration and
broader untagged build-history cleanup remain separate deferred work.
