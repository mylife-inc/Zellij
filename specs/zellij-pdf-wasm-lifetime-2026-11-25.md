# PDF Wasm code lifetime — 25 November 2026

Current status, 26 November: mitigation `5425125` passes all nine rc.24 source
PDF qualification jobs and is unchanged in tagged release `45791545516`,
which is still running. Rc.21 remains installed and in installation channels.
Rc.22/rc.23 failures and immutable tags remain retained; neither failed
candidate promoted CLI assets. See [the rc.24 record](zellij-rc24-release-2026-11-26.md).

## Observed failure

PR #260's source passed all 37 CI jobs and nine PDF jobs. Tagged release
`45764526352` later failed Windows x64 PDF job `139724415082` on the French
study fixture, after the preceding fifteen manuscripts passed three passes
each. Node 24.22.3 aborted with `0x80000003` and:

```text
Check failed: jit_page_->allocations_.erase(addr) == 1.
```

The native stack includes V8 thread isolation and Wasm cleanup. The Rust
supervisor reported failure and preserved the previous output. This was a
runtime abort, not an assertion about PDF content or a download timeout.
The raw-log hash and qualified source receipts are retained in
[rc.22 evidence](evidence/zellij-rc22-release-2026-11-25.json).

[Node issue #66366](https://github.com/nodejs/node/issues/66366) reports the same
signature during repeated Wasm instance/worker teardown on Node 24, including
Windows and Linux. [V8's correction](https://github.com/v8/v8/commit/9b8ca54d5a)
changes the lifetime handling of import wrappers marked for deletion. The
[Node backport](https://github.com/nodejs/node/pull/66376) is still open at this
checkpoint. Inspection of Node's tagged `v24.22.3` source confirms that
`WasmImportWrapperCache::MaybeGet` still increments the reference before checking
whether the wrapper is dying.

This evidence strongly points to that upstream defect; it is not a claim of a
deterministic minimal reproduction of Zellij's specific crash. A passing retry
would not resolve it, and no retry of the failed release job was requested.

## Scoped mitigation

- Rust and JavaScript PDF supervisors explicitly pass `--no-wasm-code-gc` to
  their bounded, isolated Node process, alongside the existing 128 MiB heap cap.
- Direct maintainer compilation applies the same setting before creating pass
  workers. Node rejects this flag in Worker `execArgv`, so it must be configured
  in the process before worker creation. Ambient startup hooks remain excluded.
- Only Wasm compiled-code collection is disabled. Ordinary JavaScript heap
  collection, the 512 MiB guest-memory cap, per-pass workers, deadlines,
  cancellation, output validation and atomic publication remain in place.
- Code retained by this policy is reclaimed with the PDF process. The normal
  website renderer and cached Next development do not use this setting.
  Direct maintainer APIs are for bounded jobs, not a shared long-lived Node host.

The workaround is supported by the upstream issue's reported comparisons. It
does not patch V8 itself. A future private-runtime upgrade can remove it only
after verifying that the upstream correction is included and repeating PDF
qualification. No arbitrary runtime flags are exposed through the CLI.

## Distribution scope

Only `probe.mjs` and `process.mjs` change among the reviewed adapter inputs.
Their new hashes are recorded after inspecting those changes. Compiler bytes,
resources, recipes, original archives, dependency notices and license files
are unchanged. The adapter remains inside the owner's approved GPL-2.0-only
scope; Rust and the website engine retain their existing terms. Updated
matching adapter sources must accompany the replacement runtime release.

The replacement candidate is rc.23 because rc.22's tag and images already
exist. The five new website template archives were never published; their
minimum CLI is updated to rc.23 before first publication. Existing published
template identities remain unchanged.

## Verification checkpoint

The new supervisor regression fails against the previous Rust binary because
its actual Node process lacks the fixed Wasm flag. With the revised binary,
all **27** process and Rust-supervisor tests pass, including real compilation,
startup-hook isolation, cancellation, deadlines, crashes, malformed protocol,
cleanup and output preservation. The test checks that ambient runtime options
cannot override the fixed policy.

The first local lifecycle invocation accidentally selected an older compact
fixture and failed its inventory check before compilation. The current fixture
was verified against the committed inventory before the successful run. That
fixture correction is not evidence about the V8 crash.

The full local language/long-book matrix passes, including the French study
fixture, Arabic books and the long-book fixture. Exact rc.23 runtime/source
packaging passes against the unchanged, reviewed original archives. All 52
Python distribution tests, 22 release-policy/catalog tests, nine template
fixtures, strict documentation validation (61 routes), formatting and
warnings-denied Clippy pass. Six-platform CI and public-download acceptance
remain required before publishing rc.23.
