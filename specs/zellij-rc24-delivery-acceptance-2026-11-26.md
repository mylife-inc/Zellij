# Rc.24 generated delivery acceptance — 26 November 2026

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

Pending: six generated PR verification jobs and the repository's required CI.
After their qualification and merge, dispatch only static and Next image
publication, then run the image inspector in `images` mode on fresh runners.
Record run IDs, source commits and pulled registry digests before declaring
this migration complete. A skipped publication job is not publication evidence.
