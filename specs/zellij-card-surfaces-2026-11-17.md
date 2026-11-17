# Independent card surfaces — 2026-11-17

Follow-up to the Kie theme review. Feature and pricing cards paint their own
page-colored surface, but the surrounding section's inverted/accent foreground
can reach card headings, prices, feature lists and ghost actions. The corrected
shared CSS anchors card text to its own foreground; section headings outside
cards retain their section color. Theme-specific button styling remains active.

Hosted browser acceptance clones the actual exported cards under default,
inverted, accent and gradient sections, in light/dark and desktop/mobile views.
Every card retains its default foregrounds; headings, prices, lists and linked
features resolve the card's foreground token. The measured matrices are retained.
All existing template screenshots must retain their reviewed appearance.

Initial acceptance was pending; the verified result follows. No local
browser/container is used; public rc.9 is unchanged.

## Verified result

Commit `7b6f052` merges through PR #197 as `84dcce6` with an identical tree.
All 32 CI jobs in `44086609777` pass. The initial macOS ARM package job never
acquired a hosted runner and was cancelled before any step ran. GitHub rejected
retry while the original CLI job was running; after it completed, retry attempt 2
passed the package and all eight dependent installer lifecycle cases without
source changes. This was runner capacity, not a product/test failure.

Each website architecture passes 144 foreground measurements, covering nine
cards, four backgrounds and desktop/mobile light/dark views. All 132 template
captures and 112 website captures per architecture retain their reviewed bytes
(356 total). Four measured matrices and all job conclusions are retained in
[the evidence](evidence/zellij-card-surfaces-2026-11-17.json).
The duplicate main CI is cancelled only after whole-tree equivalence is verified;
development-image run `44094639060` passes all 13 jobs. Four main variants publish
for amd64/arm64, and the published image builds and serves a site. Their manifest
digests are retained in the evidence. The later records/prose-only follow-up
changes no runtime, test, template or workflow inputs from tested `7b6f052`.
