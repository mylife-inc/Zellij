# Rc.25 navigation focus release — 27 November 2026

Status: rc.25 is published and installed locally. All 55 release jobs pass on
attempt 1. Independent public Mac verification and the downloaded-renderer
focus regression pass. The owned Homebrew/Scoop recipes are published and all eight actual
rc.24-to-rc.25 upgrades pass. The qualified guides await production publication.

## Qualified source

[PR #271](https://github.com/mylife-inc/Zellij/pull/271) merged as `988e321`,
tree-identical to qualified `eac6ca0` (`c9b36b9` tree). Source CI `46095168788`
passes all 37 executable jobs; PDF qualification `46095168755` passes all nine.
The browser report contains 556 passes, 173 conditional skips, zero flaky
results, zero retry attempts and zero runner errors.

The fresh pre-tag audit finds no production or development advisories. The
ordinary release helper passes typechecks, schema drift, formatting, Clippy,
556 engine tests and 863 Rust tests before creating the immutable tag. The tag
points to `988e321`. Duplicate main CI `46098006538` was cancelled only after
checking tree equality and absence of observed failures; it is not a passing
run. All 13 main-image jobs in `46098006447` pass independently.

The initial local policy run caught the standalone Docker recipe still selecting
rc.24. That pin was corrected before final source qualification. The superseded
initial source runs were cancelled and are not counted as passes. Corrected local
qualification passes 41 release-policy/download guards and eight recipe guards.

## Changes shipped

The candidate packages the navigation focus correction from PR #270 (`f287b74`).
An older queued callback no longer consumes a newer resize request before the
corresponding menu commits. Newer keyboard or pointer choices still take
precedence. The source investigation retains controlled and natural evidence in
[the focus record](zellij-overflow-focus-commit-2026-11-27.md).

Global `--json` help now lists the already supported `templates` command. Unused
workspace dependency declarations were removed without changing resolved
versions. No new command, schema, template archive identity or PDF license scope
is introduced. Rebuild existing sites with rc.25 to receive the renderer fix.

## Public verification

Release [46098349000](https://github.com/mylife-inc/Zellij/actions/runs/46098349000)
passes all 55 jobs, including six public native installations, six complete
cached-PDF checks against both repositories and both Next-image PDF architectures.
Both public repositories contain 58 matching assets. Downloaded Homebrew/Scoop
recipes match the checksum-derived generators. Existing template archive
identities retain their published bytes.

The identical repository showcase fixture was built using public rc.24 and
rc.25 CLI/renderer downloads, isolated resource homes and an empty PATH. The
same [controlled harness](evidence/rc25-public-focus/README.md) fails all eight
final focus assertions with rc.24 and passes all eight with rc.25 across
Chromium and WebKit. Both runs have no HTTP requests or page errors; the after
run has no retries or runner errors. This demonstrates the published renderer
fix without closing the unrelated historical timing findings.

Independent Intel Mac acceptance passes against both public repositories:
checksums/installers, native upgrade protections, authoring, file/HTTP builds,
validation/watch recovery, editable sources, cached development, offline restart,
edit recovery, shutdown and template verification. Exact structured receipts,
run identities, asset inventories and hashes are retained in
[the release evidence](evidence/zellij-rc25-release-2026-11-27.json).

## Channels and guides

The owned Homebrew and Scoop PRs each pass four native lifecycle jobs. Their
published recipes are exact copies of the verified release attachments. Explicit
workflow dispatches `46102768093` (Homebrew) and `46102773789` (Scoop) pass all
eight real rc.24-to-rc.25 upgrades on attempt 1. Merge commits skip the redundant
automatic run; these actual upgrade workflows run the full lifecycle. Each
completed job log confirms the old and new executable versions. The installed
local CLI matches the public archive and passes default-home cold rich generation,
offline rebuild and validation with an empty PATH.

The installation guide draft passes strict local validation and documentation
workflow `46098484138`: 46 original plus 15 added routes, 282 original heading IDs,
82 compatibility mappings and 29 byte-identical original images, with no captured
browser errors. Its runtime inputs match the release tag. Final evidence-only
edits do not change those validated site inputs. All channel upgrade verification is complete. Production publication is the
remaining delivery step.

The historical WebKit rendering stall and documentation hydration timeout remain
open. Deferred provider/catalog verification and retirement are separate; no old
distributions are removed here. This is not stable 1.0 or a zero-bug claim.
