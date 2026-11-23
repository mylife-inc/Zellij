# Rust PDF supervision — 23 November 2026

Status: engine implementation and maintainer acceptance. This does not switch
the installed CLI to cached PDFs, publish a toolchain, or add a public command.
Released rc.19 still uses a container for PDF generation.

## Baseline and retained findings

PR #248 merged as `83f3608`, identical in tree to tested `bf6445b`. PDF run
`45232766997` passes all seven jobs: 114 manuscripts / 342 passes across six
platforms, with 269 native-matched Linux page rasters. Normal CI `45232766939`
passes 37 jobs with one conditional skip after targeted retries. One initial
JavaScript-disabled WebKit equation-link click timed out; its unchanged retry
passes, but the cause remains unexplained. PR #248 retains its evidence and the
separate Showcase job timeout and unstarted macOS runner-capacity cancellations.

Post-merge base-image run `45244541942` passes. Main run `45244541989` initially
passes 36 jobs and skips one conditional check; GitHub cancels the unstarted
Apple Silicon Homebrew job for runner capacity. Attempt 2 repeats that unstarted
capacity cancellation, with no failed test assertions. This increment's final
verification is recorded in the implementation PR.

## Implemented Rust boundary

`zellij_core::process::ProtocolChild` owns a direct child and an empty stdin
liveness pipe. stdout/stderr are private temporary captures, independent of the
user's quiet/color settings. Polling drains bounded byte deltas without waiting
for pipe EOF or writing protocol bytes to the console. Oversized or truncated
captures fail. These are enforced polling budgets, not OS disk quotas.

Cancellation closes stdin, waits at most one second for cooperative exit, then
kills and reaps the direct child. Dropping an unfinished guard kills/reaps it on
error. This is not a process-tree kill: the PDF compiler uses worker threads
inside the supervised Node process. The existing external-tool progress API is
unchanged.

`zellij_engine::pdf_runner` launches the absolute private Node/script paths,
clears inherited environment/startup hooks, preserves only OS bootstrap values,
and applies the qualified 128 MiB old-generation budget. It checks ordered,
PID-matched ready/pass/completion messages, bounded deadlines, successful exit,
and the complete result inventory. The caller must first verify the runtime,
script and resource package; this module does not download or select them.

The Rust owner creates an outer private stage. Before publishing into a new
destination it independently checks regular size-limited PDF/receipt/log files,
the PDF header/trailer and checksum, three passes, and agreement between the
final log and pass three. It rechecks cancellation and destination ownership.
The child guard is dropped before the stage on every error. The initial real
test rejects valid output because the new inventory omitted the three per-pass
logs; the corrected inventory validates all six output files, rather than
loosening rejection of unexpected output.

The existing JavaScript acceptance adapter remains available. The child ready
event now supplies the actual Node version, which the Rust receipt records with
`execution.kind: rust-process`. No extra JavaScript parent process is needed.

## Verification and remaining integration

The `pdf_process` Rust example is a maintainer harness for the production engine
interface. Its test controls are not parsed by the installed CLI. Fourteen
regressions exercise real three-pass builds under every quiet/color combination,
empty PATH and excluded startup hooks, cancellation before/during/after passes,
deadlines and recovery, forced parent/child death, the hard-kill fallback,
invalid protocol/output, observer/startup errors and output preservation.
The manuscript matrix runs through this Rust interface on all six native targets.
Exact final-head results, initial failures and source receipts belong in the PR.

A forcibly terminated caller can leave its owned outer stage; the compiler must
still stop on stdin EOF. Arbitrary hostile filesystem races and a hard total RSS
cap remain outside these guarantees. The Node VM is not an OS security sandbox.

Remaining: final isolation/distribution review, a versioned optional archive with
original notices and matching-source access, Rust cache/download/dispatch and
offline propagation, diagnostics/user docs, and installed/public acceptance.
The original URL package documentation repeats its ambiguous LPPL-version
wording; inspecting it does not establish new redistribution permission.
