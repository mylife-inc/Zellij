# Rc.20 cached PDF release preparation

Status updated 25 November: rc.20 is published and all 55 release jobs pass.
Six public cached-PDF jobs verify both repositories and all eight actual channel
upgrades pass. Revised guides await pull-request checks and automatic production
publication. Container remains the default.

## Behavior

Rc.20 exposes the reviewed compiler through `--pdf-backend cached` (or
`ZEL_PDF_BACKEND=cached`) for PDF exports, PDF-enabled builds and previews.
The default remains `container`. Cached compilation downloads the optional
versioned compiler, reuses the renderer's private Node and requires no system
Docker, Node, Python or TeX. Cached resources live below `ZELLIJ_HOME` and are
verified before reuse. Offline operation never downloads; existing output is
retained on failure. Supported print-content boundaries are unchanged.

The complete matching sources remain separate from the small runtime. Each
release offers both archives and their checksums, preserving the scoped adapter
license and original component terms. No package deletion is part of this work.

## Gates

PR #254 merged as `67a411e`, with the tested tree of `0d83284`. Its regular CI
`45548294602` passes 37 jobs and its PDF CI `45548294563` passes eight, both
without retries. Post-merge CI `45552922780`, base images `45552922767` and docs
`45552922736` pass. This establishes packaging qualification, not public availability.

The expanded PDF workflow builds the actual rich renderer and checks it with
the actual reviewed compiler on six native targets. The child CLI has an empty
PATH. It performs a cold PDF export, verifies both per-file and combined checksum
records, streams the complete matching-source archive, then builds a PDF-enabled
book and a rich product website offline. Both operations reuse the same private
Node. Receipts bind the exact runtime/source identities and are compared with
the independent source replay. The expanded job allowance is 60 minutes;
product compiler/cancellation deadlines are unchanged.

After publication, a separate six-platform job installs each public CLI through
its published installer and repeats the real-package checks against both release
repositories. It compares their receipts. Anonymous transfers have size/time
bounds; only transient transport failures retry, starting each stream afresh.
Sources are streamed rather than buffered as a 430 MB object. Older release
tags without this driver remain verifiable by the existing acceptance workflow.

The cached backend will remain opt-in until these public checks pass. A later
default change needs its own command, documentation and generated-CI coverage.
The retained historical browser timing findings and deferred external-provider
verification are separate; passing release checks does not prove zero bugs.

## Verification

Local Intel Mac acceptance passes with the actual renderer and compiler:
cold PDF, complete source transfer, offline PDF-enabled build and rich
`product-studio` build all succeed with an empty PATH. The four installed-command
groups also pass, including corruption/output preservation and watched offline
previews. The 52 Python regressions, six transfer regression groups, 19 targeted
Rust PDF tests, workflow lint, formatting and 59-route documentation validation pass.
All 41 release-policy/download regressions pass with the new public acceptance job.
The reviewed runtime is 11,792,851 bytes; matching sources are 430,327,849 bytes.
Exact hashes and receipts are in [local evidence](evidence/zellij-rc20-cached-pdf-local-2026-11-24.json).

The first new fixture selected the
portable `product-docs` template and correctly failed the rich-output assertion;
it now uses `product-studio` and passes. The rejected run is not counted as
real-renderer acceptance.

## Published qualification

PR #255 merged as `65249fa7b685879538bfcc4719414e90d15b176e`, tree-identical to
tested `0396005`. Regular CI `45561362507` passes 37 jobs and PDF CI `45561362593`
passes nine; documentation validation also passes. Post-merge main CI, base
images and documentation runs `45564988471`, `45564988493` and `45564988393` pass.

Release `45565105796` passes 55 jobs with one conditional skip. Attempt 1 stops
at a Windows source-acquisition timeout (`curl` exit 28 for the pinned original
`amsmath.tar.xz`); an independent cold download matches its original SHA-512.
Only the failed job and its dependents are retried. Attempt 2 passes without
changing the tag, sources, archive hashes or qualification gates. The failure
log and retry identity are retained in the evidence.

Both `mylife-inc/Zellij` and `mylife-inc/releases` publish 46 assets with matching
names, sizes and digests. The six public PDF jobs install each repository's CLI,
verify complete corresponding sources and compare runtime/source receipts.
All twelve receipts match the reviewed pair. Independent Intel Mac acceptance
repeats both repositories; the installed default-location CLI also builds cold
and offline with default download URLs and an empty PATH.

The revised guides pass a local production build and browser preservation:
59 routes, 282 original headings, 82 compatibility mappings and 29 byte-identical
images, with no captured errors after hydration. An initial local preparation
failure came from absent workspace dependencies; the successful check uses the
locked dependency tree and a rebuilt engine.

Homebrew commit `feb88a6` and Scoop commit `05b8843` adopt the exact recipes
generated from the published checksum inventory. All eight actual rc.19-to-rc.20
upgrades, ownership checks, portable builds and uninstall cases pass in runs
`45574903028` and `45574914352`. Earlier release assets remain intact.
The [publication evidence](evidence/zellij-rc20-release-2026-11-25.json) retains
the package identities, public receipts, local checks and per-job upgrade proofs.
