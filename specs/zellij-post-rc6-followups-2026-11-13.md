# Implementation follow-ups after rc.6 — 2026-11-13

First implementation follow-up after rc.6 distribution: deterministic runtime
selection in generated container CI. `cli/crates/zellij-scaffold/src/pipeline.rs`
currently generates hosted GitHub jobs without selecting Docker, even where
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
also recorded a discovery timeout. Treat this as an open reliability issue,
not as a bug proven resolved by rerunning.

Next distribution feature: Scoop ownership protection before publishing an
owned bucket. `cli/crates/zellij-cli/src/maintenance/upgrade.rs` currently checks
only an executable in `bin/` with `share/zel/installation-manager`. The Homebrew
contract is verified; a Scoop package layout needs its own deliberate marker
location, accurate upgrade guidance and native Windows install/upgrade/uninstall
acceptance. Do not publish a managed package that the current self-upgrader can
silently replace. Then bundle the fixes and package support into a new candidate.

Remaining book work: numbered equations/theorems and shared references,
bibliography/index/glossary, multilingual/RTL print, Next-image PDFs, and
technical/study/custom print templates. Existing elementary math is implemented.
Live provider and external-runner verification, production-domain migration and
broader untagged build-history cleanup remain separate deferred work.
