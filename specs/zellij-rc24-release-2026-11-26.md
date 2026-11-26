# Rc.24 cached development release — 26 November 2026

Status: tagged release qualification is running. Rc.21 remains the installed
candidate and the published installation-channel version. Do not promote the
prepared rc.24 installation guides until public-download acceptance succeeds.

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

## Retained failures and publication gate

Rc.22's Windows PDF abort and rc.23's two development failures remain recorded.
Neither candidate published CLI assets or advanced installation channels; their
tags stay fixed. Rc.24's initial source CI also retains two setup-related browser
cancellations and the targeted retry: rich-file acceptance passed, while showcase
again exhausted its total job limit. The revised CI completed successfully.

Release `45791545516` must finish before installation-channel promotion. Its
public assets, independent native download check, actual standalone upgrade,
Homebrew/Scoop upgrades, template ledger and final documentation publication
receipts belong in [the release evidence](evidence/zellij-rc24-release-2026-11-25.json).
Historical browser timing findings and deferred provider/catalog verification
remain separate; this record does not claim stable 1.0 or zero bugs.
