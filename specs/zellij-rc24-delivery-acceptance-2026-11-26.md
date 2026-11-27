# Rc.24 generated delivery acceptance — 26 November 2026

Completed **27 November 2026**. [Machine-readable evidence](evidence/zellij-rc24-delivery-acceptance-2026-11-27.json)
records exact workflow/job identities, source commits, digests and verification limits.

## Scope

Move the six generated GitHub delivery workflows from rc.19 to the already
published rc.24 CLI and matching engine. Regenerate both registry inspection
targets and configuration hashes with them. This uses only repository-owned
fixtures; it does not change CLI behavior or publish another native release.

The separate Fly, S3, Vercel and combined live-provider workflows remain on
their recorded rc.19 engine pin. They are deferred and are not dispatched here.
Rc.19 remains retained. Previous acceptance records, including the corrected
rc.18/rc.19 image-inspection history, remain dated evidence.

## Preparation

Baseline: `e0e639285ddf2a41b71f5495f7b75189ba088799`. Published candidate:
`v0.2.0-rc.24`, source `2c3b7daa3be5b0f218b4584a8074e77b4f25d848`.

- All six configurations select `ci.zel-version: 0.2.0-rc.24`.
- Static and Next publication select their dedicated acceptance packages with
  the `0.2.0-rc.24` tag. The generated engine pin follows the same version.
- `prepare-delivery-acceptance.mjs --write` runs the verified public rc.24
  executable. Both that executable and the current source build reproduce
  all six committed workflows exactly.
- The six existing HTTP/metadata tests pass, including rejection of stale
  image targets and changed configuration bytes. Actionlint and diff checks pass.
- PR jobs validate and build without publishing. Image publication still
  requires a manual dispatch on `main`. Netlify publication stays draft-only
  and is not part of this checkpoint.

## Qualification

[PR #267](https://github.com/mylife-inc/Zellij/pull/267) qualified source
`24784c90e0cf429927df3d4aa2eff5461ec5428d` and merged as
`624bfb387fb0ef652207f366e2b8c054a5e65c3e`. Both have tree
`adf8a042c331368b17c369a3b92b2722fcdcf307`.

CI [45956665792](https://github.com/mylife-inc/Zellij/actions/runs/45956665792)
passes all 37 executable jobs on attempt 1, with one conditional placeholder
skipped. The showcase reports 552 passes, 169 conditional skips and no retries;
Chromium/WebKit development reload and recovery checks also pass.

All six generated verification workflows pass on attempt 1. Logs confirm each
installed rc.24; both hosted exports use the rc.24 renderer cache. The four
publication jobs are skipped on PRs, as intended.

| Verification workflow | Run |
| --- | --- |
| Portable | [45956665799](https://github.com/mylife-inc/Zellij/actions/runs/45956665799) |
| Hosted | [45956665861](https://github.com/mylife-inc/Zellij/actions/runs/45956665861) |
| Static image | [45956665776](https://github.com/mylife-inc/Zellij/actions/runs/45956665776) |
| Next image | [45956665782](https://github.com/mylife-inc/Zellij/actions/runs/45956665782) |
| Netlify portable, build only | [45956665761](https://github.com/mylife-inc/Zellij/actions/runs/45956665761) |
| Netlify hosted, build only | [45956665723](https://github.com/mylife-inc/Zellij/actions/runs/45956665723) |

Duplicate automatic main CI/base-image builds were skipped after verifying
qualified-tree equivalence. They are not counted as additional passing runs.

## Publication and independent pulls

Manual main runs publish the static image in
[45960440498](https://github.com/mylife-inc/Zellij/actions/runs/45960440498) and
the Next image in [45960444256](https://github.com/mylife-inc/Zellij/actions/runs/45960444256).
Both pass on attempt 1 at `624bfb3`. Fresh-runner inspection
[46026610903](https://github.com/mylife-inc/Zellij/actions/runs/46026610903)
passes both images at that same source on attempt 1; its live-provider job is
skipped. Each pulled tag agrees with the regenerated, hash-bound metadata.

| Image | Pulled registry digest | Compressed layers |
| --- | --- | --- |
| `acceptance-static:0.2.0-rc.24` | `sha256:9c013347387226d1c08697f03609b597716955a2e1f415cdd5b3127f8a2f6eb4` | 7,290,374 bytes |
| `acceptance-next:0.2.0-rc.24` | `sha256:8aade9c99c068fd409c51a31cfb3e316527707a3bffc459dda5c128371bca222` | 79,262,259 bytes |

Checks cover Linux amd64, nonroot `nginx`/`nextjs` users, startup, the image's
health command, expected pages/assets and source-file isolation. Anonymous
registry reads independently resolve the same tag digests and confirm public
access. Sizes are compressed layer-byte totals for these fixtures, not installed
sizes or registry savings. Separate source CI covers both Linux image architectures.

Rc.24 remains the published native candidate. No new native release, channel
update, production documentation deployment, provider deployment or retirement
occurred. The deferred provider workflows still need their rc.19 inputs.
Historical browser timing findings and the PDF Wasm mitigation remain separate;
this checkpoint does not establish zero bugs or stable 1.0 readiness.
