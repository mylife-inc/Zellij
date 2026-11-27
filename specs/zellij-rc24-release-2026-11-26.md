# Rc.24 cached development release — 26 November 2026

Status: rc.24 is published, installed locally and available through the owned
Homebrew/Scoop channels. All 55 release jobs pass on their first attempt.
Independent public Mac and installed default-home checks pass. All eight actual
rc.21-to-rc.24 channel upgrades pass. The updated installation guides are
merged and published through the normal production workflow.

## Qualified source

[PR #262](https://github.com/mylife-inc/Zellij/pull/262) merged as `2c3b7da`,
with a tree identical to tested `a3933b6`. The immutable `v0.2.0-rc.24` tag
points to that merge. Source CI `45786080712` passes 37 jobs with one conditional
skip; PDF qualification `45786080629` passes all nine jobs. Six native targets
pass cold/offline cached development, interrupted-transfer cleanup, invalid-edit
recovery, integrity checks, port release and session removal. Windows/Linux
Chromium and the real Windows file-lock fixture pass. The showcase reports
552 passing cases, 169 conditional skips and no retries.

The automatic duplicate main CI was cancelled only after verifying the merge
tree matches the qualified source and no failure was reported in the duplicate.
The tagged release repeats its own qualification and public-download checks.
Source-candidate documentation was published independently by `45791394799`.

## Changes being shipped

- Cached `zel dev` reuses private Node, editable sources and a separate optional
  development dependency package under `ZELLIJ_HOME`. It needs no separately
  installed Docker, Node/npm or Python. Ordinary static builds keep their smaller
  runtime download; custom Next applications retain their own toolchain needs.
- Windows preparation retries temporary sharing locks without removing the old
  file. Persistent locks preserve that file and report failure. Fix: `288913c`.
- Native acceptance waits for each completed edit's final reload marker before
  submitting the next edit. The previous stale-marker assertion is reproduced
  by a controlled barrier. Fix: `288913c`.
- The two affected browser CI jobs allow 30 minutes overall after observed
  Ubuntu package setup took 17–19 minutes. Browser assertions and individual
  deadlines are unchanged. Fix: `a3933b6`.
- The bounded PDF process retains the rc.23 Wasm code-GC mitigation (`5425125`).
  Its reviewed adapter inputs and previously approved PDF-only license scope
  are unchanged. Matching compiler sources remain paired with the runtime.
- Five website templates have new immutable versions and rc.24 minima; earlier
  published template archive bytes remain protected by the ledger.

Hosted development reads live authored files. A rejected edit requests no reload
of its own, but manual navigation or an already scheduled valid reload can read
new content errors. The guide distinguishes this behavior from `zel vw`, which
serves completed static exports.

## Retained failures and publication evidence

Rc.22's Windows PDF abort and rc.23's two development failures remain recorded.
Neither candidate published CLI assets or advanced installation channels; their
tags stay fixed. Rc.24's initial source CI also retains two setup-related browser
cancellations and the targeted retry: rich-file acceptance passed, while showcase
again exhausted its total job limit. The revised CI completed successfully.

Release `45791545516` passes all 55 jobs, with one conditional legacy WinGet
job skipped. Both public repositories contain 58 matching assets. Recipes and
all nine template archives verify; five new identities join the ledger without
changing prior archive hashes. Six public cached-PDF jobs verify both release
repositories, full matching sources and offline PDF/rich builds. Independent
Mac checks cover both repositories. The actual standalone upgrade then passes
cold and offline development, validation, file/HTTP builds and PDF commands with
an empty PATH and the default resource home. All four caches were initially
absent. Exact receipts and channel upgrade results are retained in
[the release evidence](evidence/zellij-rc24-release-2026-11-25.json).

PR #263 merged as `c583efb`, tree-identical to tested `b6df903`. CI run
`45802055700` passes all 37 jobs; documentation run `45802055641` preserves
46 original plus 15 added routes, 282 original heading IDs, 82 compatibility
mappings and 29 byte-identical original images, with no captured browser errors.
Showcase reports 552 passes, 169 conditional skips and no retries. The duplicate
main CI `45806880688` was cancelled after tree equivalence and absence of observed
failures were checked; it is not counted as a passing run.

Automatic production run `45806880663` publishes and verifies `c583efb` at
`zellij.shebka.net` (26 November, 11:04:56 UTC). Ten live guide pages return
HTTP 200, hydrate and contain the expected candidate instructions, with no
captured browser errors. Main image run `45806880747` passes all 13 jobs.
These completed receipts supersede this record's earlier pre-merge checkpoint.
Historical browser timing findings and deferred provider/catalog verification
remain separate; this record does not claim stable 1.0 or zero bugs.

Follow-up, 27 November: [generated delivery acceptance](zellij-rc24-delivery-acceptance-2026-11-26.md)
now selects rc.24 throughout its six workflows and hash-bound inspector targets.
PR #267 passes all six verification workflows and 37 CI jobs; both manual image
publications and fresh pulls pass. Anonymous registry reads match the digests.
This adds acceptance evidence, not another native release or live-provider run.
