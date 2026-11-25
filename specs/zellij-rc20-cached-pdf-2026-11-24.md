# Rc.20 cached PDF release preparation

Status: implementation and local verification complete; hosted gates and
publication pending. Rc.19 remains published.

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

Hosted checks and publication remain. The first new fixture selected the
portable `product-docs` template and correctly failed the rich-output assertion;
it now uses `product-studio` and passes. The rejected run is not counted as
real-renderer acceptance.
