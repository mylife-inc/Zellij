# Cancellable renderer acquisition — 23 November 2026

Status: source implementation and package acceptance. This does not activate the
cached PDF backend or publish a new CLI candidate. Released rc.19 still uses its
container for PDF generation.

## Baseline

PR #249 merged as `0935d5c`, identical in tree to tested `b18ce11`. Normal CI
`45278436633` passes all 37 jobs with one conditional skip; PDF qualification
`45278435861` passes all seven jobs without retries. The six platforms pass
114 manuscripts / 342 typesetting passes and 84 Rust regression groups. All
269 Linux page rasters match the native renderer. Main CI `45282777504` and
base image `45282777241` also pass. Earlier browser timing findings are not
reclassified as fixed.

## Findings and implementation

The existing renderer cache downloads synchronously on the caller thread. The
renderer process is cancellable, but an initial stalled download can prevent a
preview from stopping promptly. One-shot build/check has no acquisition signal
owner, and validation watch installs its handler after acquisition. Hashing and
extraction also lack cancellation checks. This matters both for existing rich
commands and for the future PDF backend's reuse of the private Node runtime.

`package_transfer` now shares the editable-source cache's bounded network-worker
pattern with the rich renderer. It accepts the caller's cancellation flag and
checks it while waiting for headers or body data. A bounded channel carries at
most two 64 KiB queued chunks; the caller alone writes files. Cancellation drops
the receiver and the caller's staging directory. A blocked network worker may
remain until the existing HTTP timeout or process exit, but cannot write cache
files or publish a later result. This is not a promise to synchronously abort a
blocked OS network call.

Both consumers retain HTTPS/loopback and redirect restrictions, response and
archive limits, version/platform checks, checksums, staged extraction and offline
cache reuse. Source acquisition and rich commands share signal registration so
sequential operations do not compete to install a process-wide Ctrl-C handler.
Rich extraction and hashing check cancellation between bounded reads/writes and
before publication. Check cancellation produces its existing structured JSON
failure form; preview/watch finish normally without starting a renderer.

Previously, renderer validation checked each listed file but allowed unlisted
files and directories. It now inspects the complete tree, rejects unlisted,
missing, oversized or nonregular entries and path case aliases, and rejects
symbolic links and Windows reparse points/junctions. Dangling cache roots are
reported as damaged rather than mistaken for a missing download. Cached files
are preserved on failure; the CLI does not silently repair or delete them.
Only platform path separators are normalized, so a Unix filename containing a
literal backslash cannot alias a manifested nested path.

These checks do not authenticate a user-selected mirror independently of HTTPS
and its supplied checksum, or prevent arbitrary concurrent hostile filesystem
replacement. Existing trust and output-ownership boundaries still apply.

## Verification

Rust regressions cover inventory additions/removals/aliases, size bounds,
truncated archive trailers, cancellation before acquisition and during a read,
and source transfer status/size/truncation/redirect handling. Unix-only link
regressions supplement the platform-neutral inventory checks.

The initial PR run `45304356856` at `06e2678` also exposes an independent engine
watch defect: `watch-preparation.test.ts` observes a changed development revision
without an authored edit. The watcher snapshots before preparation creates
`public/_zellij`. It excludes that generated directory but records the newly
created `public` parent, scheduling one extra preparation/revision. A fixed
200 ms settling delay in the old test could conceal it on faster runs.

A deterministic snapshot comparison reproduces this without timers. The fix
omits only the generated parent's directory bookkeeping entry while still
visiting authored siblings under `public`. The regression checks creation of
generated files and `.next`, addition/removal of an authored public file, and
removal of the generated parent. The existing live-watcher test now records its
revision immediately after startup instead of waiting for the unwanted rebuild.
This explains this specific revision failure, not the historical WebKit findings.

The existing real-package acceptance now cancels build, JSON check, validation
watch and static preview during checksum headers, checksum body and archive body
waits: twelve scenarios per native target. It uses a real private console on
Windows to deliver Ctrl-C, an empty user PATH, a separate cold cache for each
case, and an already generated site. Every cancellation must finish promptly,
remove its download/output stages and preserve the previous site's bytes.
Warm/offline reuse and corrupt/unlisted cache rejection still run against the
actual packaged renderer. Editable-source acceptance exercises the shared
transfer independently. Final-head results and any initial failures belong in
the implementation PR.

Remaining PDF work: distribution review, optional versioned package with original
notices and matching sources, cache/download/dispatch/offline propagation,
diagnostics/user docs and installed/public acceptance. The unresolved historical
URL-package license wording is unchanged; this increment provides no new
redistribution clearance.
