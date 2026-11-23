# PDF process supervision — 23 November 2026

Status: maintainer qualification for the optional cached PDF renderer. Released
rc.19 still uses a container for PDF. This increment publishes no runtime archive
and makes no new prerequisite claim or public command available.

## Baseline

PR #247 merged as `fa919f8`, identical in tree to tested `80ae68a`. Its seven PDF
jobs passed in `45217036115`: six identical compact resource archives, 114
manuscript cases / 342 passes and 269 native-matched Linux page rasters.
All 37 normal jobs passed in `45217035909` attempt 2, with one conditional skip.
GitHub could not acquire an Apple Silicon runner in attempt 1; the targeted retry
passed without code changes. Post-merge base image `45223943723` passes. Main CI
`45223943954` also encountered an unstarted Apple Silicon job and is being
retried; the implementation PR records its final result.

The missing integration evidence was the process boundary a Rust caller will
supervise: cancellation during a pass, parent shutdown, hard termination, output
ownership and executable startup environment.

## Implemented contract

`scripts/pdf-wasm/process.mjs` provides a child entry point and a maintainer
adapter that starts a separate absolute Node executable. It uses the verified
compiler/resources and generated inputs, with three passes and strict final
typesetting/PDF validation. It resolves no resources over HTTP and invokes no
system tools. The source package includes the new runner and tests.
The process and worker old-generation budgets both use 128 MiB, because Node's
process-level V8 flag overrides the worker option. This is not a total RSS cap.

The parent keeps an otherwise empty stdin pipe open. EOF or unexpected input
cancels the compiler. Cooperative signals and a total deadline also abort it.
`probe.mjs` now accepts an abort signal, terminates its active worker in `finally`,
checks cancellation between passes and before publication, and refuses an output
that appeared during compilation. Worker deadlines cannot exceed 120 seconds.

The adapter owns the outer temporary directory containing all child output. It
parses bounded progress events, captures bounded diagnostics, and requires both
a successful exit and completion event. Cancellation/deadline closes the control
pipe, then forcibly kills the child after one second if necessary. The child is
reaped before cleanup. Before publishing, the adapter independently checks
regular, size-limited PDF/log/receipt files, the PDF hash and three passes, then
rechecks cancellation and destination. Prior output is never used as working
storage or deliberately removed.

The initial hook regression failed because Node's spawn implementation copied
`NODE_V8_COVERAGE` from the maintainer environment despite an environment
allowlist. Explicitly setting an empty value prevents that automatic propagation.
The test checks neither a preload marker nor coverage files appear outside the
runner's stage. Workers also receive an empty environment and no execution flags.

## Verification and limits

Twelve real-compiler regressions cover successful output, cancellation at all
three passes and after completion, deadlines/recovery, hard process death, parent
pipe closure, corrupt resources, invalid TeX, existing/late output and observer
errors. Unix additionally suspends a child with SIGSTOP to prove the hard-kill
fallback; that one test is explicitly skipped on Windows. Parent-pipe cancellation
must finish before the independent child deadline, avoiding a false pass from
merely waiting for timeout.

The default full manuscript matrix now uses the process adapter and records its
execution mode. The legacy in-memory format/font override path remains a separate
maintainer comparison. Exact hosted results and source archive receipts belong
in the implementation PR, including failed attempts. CI uploads no runtime or
source archive.

Node VM contexts are not the security boundary, and heap/file/log budgets do not
establish a hard total RSS cap. The trusted, pinned Wasm engine and host imports
remain the guest boundary. A forcibly terminated caller can leave its owned
temporary directory; its child still observes pipe closure. Arbitrary filesystem
races remain outside the ownership guarantee.

Remaining: final isolation/distribution review, a versioned archive with notices
and matching-source access, Rust cache/download/dispatch, offline/cancellation
propagation through all PDF callers, diagnostics/user documentation and
installed/public-package acceptance. Public rc.19 is unchanged.
