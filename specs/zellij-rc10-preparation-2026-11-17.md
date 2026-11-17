# CLI rc.10 preparation — 2026-11-17

Baseline `8cb6a99`, after print styling, multilingual books, Kie theme and card
contrast fixes. Rust workspace/versioned recipes already select rc.10. The final
implementation `7b6f052` passes all 32 jobs in `44086609777` (attempt 2 after a
macOS runner-capacity cancellation). Merge `84dcce6` has the identical tree;
`8cb6a99` adds records/previews and one strict-checked prose correction only.
All 13 development-image jobs in `44094639060` pass.

## Publication boundary

This preparation changes release notes and records only. Runtime, templates,
dependencies, workflows and acceptance inputs remain identical to tested
`7b6f052`; there is no new version or implementation change to retest. Normal
release preflight still runs before the immutable rc.10 tag. Release/public
acceptance must pass for all native packages, four images on both architectures,
multilingual/styled PDFs and four website starters. Compare downloaded public
outputs with the retained source baselines; do not substitute rebuilt artifacts.

Then publish verified Homebrew/Scoop recipes, exercise real rc.9-to-rc.10
upgrades while rc.9 is still available, update active guides and saved consumers,
and retire only reviewed superseded artifacts after all replacement gates pass.
Keep existing template identities and unrelated shared-repository products.
Local browsers/containers/LaTeX remain off; actual rendering runs on hosted CI.
Production hosting and live-provider verification remain deferred.

Publication, channel/docs migration and retirement are pending at this checkpoint.
