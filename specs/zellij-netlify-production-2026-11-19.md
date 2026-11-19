# Zellij documentation production deployment — 2026-11-19

Status: **published and verified** at https://zellij.shebka.net/ after the user's
explicit authorization. This supersedes the earlier local-review/pending-cutover
status. The existing production project was updated; no account migration, DNS
change, staging-project replacement or CLI release was performed.

## Published artifact

- Source: `e341880d5a08a16c0f525e69a9c497a816dd660d` on `codex/rich-offline-export`.
- Netlify project: `zellij-docs` (`91774174-d4c2-41b5-aa4b-ac61eeadbf51`).
- Production deploy: `6aff4d3001a5a3ee682b3ff2`; published at `2026-11-19T19:26:20.633Z`.
- Artifact: 588 files, SHA-256 `265132d92a32f8f8652035229e1fa965c68c1329e4a8d2e67606dafd49c99396`.
- Reviewed documentation: 59 content routes. It retains explicit distinctions
  between released rc.11 and forthcoming engine/CLI functionality.

The reviewed source `8168d79` was built in an isolated local tree so the user's
local preview remained available. The exact ready draft was promoted without a
rebuild. A first browser pass found the missing conventional favicon. Commit
`1b63560` added explicit icon metadata; that passed the draft check but Chrome
still requested `/favicon.ico` on the public domain. Commit `e341880` supplies
that conventional file, wrapping the existing 112 × 112 PNG logo in an ICO
without changing its pixels. The final production build passed, and all 59
article texts and authored assets match the reviewed pre-icon artifact. The
intermediate deployments remain recorded in the machine-readable evidence.

## Verification and limits

- Complete local preservation: all 46 old routes, 13 added routes, 282 original
  heading IDs, 82 compatibility mappings and 29 byte-identical original images;
  current repository links; no captured console/page errors after hydration.
- Netlify's file inventory matches the digest of every one of the
  588 uploaded artifact files. Its API lowercases
  path names, so comparison normalizes case after confirming no source case
  collisions. One additional provider-generated `netlify.toml` contains empty
  configuration sections and no credentials.
- Public HTTP content checks pass for the homepage, build, templates, books and
  composer reference. The hosted logo bytes match the source.
- Normal Chrome browser checks pass on the final draft and production domain:
  homepage hydration, client navigation, search-to-composer navigation, theme
  switching, template/book pages, icon metadata and conventional favicon bytes.
  No `console.error` or `pageerror` was captured in those flows.
- A rapid headless draft crawl encountered Netlify HTTP 403 at `/guide/motion`
  and stopped. It is **not** a complete remote browser acceptance pass. The
  complete route/anchor preservation check is local; live browser coverage is
  limited to the flows above. No access challenge was bypassed.

The current engine contains the controlled hydration remedy from `360cb40`.
These live checks did not reproduce the old hydration failure; they do not
claim that Netlify has removed or changed its HTML annotation globally.

## Retained recovery material

The original deployment `6a7cb3dc45caa1c8e95f9c03` remains available in
Netlify. Before publishing, every file was downloaded through Netlify's raw-file
API and checked against its size/SHA-1 metadata. The ZIP also passed CRC checks.

- Complete original backup: `589` files, `26341511` bytes.
- ZIP SHA-256: `f605a953e8ee1dd70562f78bba1b4302905ded181f8caf2b090ff25dec71895b`.
- Local ignored archive: `backups/zellij-netlify-2026-11-19/previous-production-files.zip`.
- The same protected local directory retains site settings, original file
  inventory, deploy metadata, all publication artifacts and verification logs.
  Raw settings/credentials are not committed to the repository.

For rollback, restore the retained original deploy in this same Netlify project
(or republish the verified original ZIP if necessary). Restoring a deploy
changes production, so perform it only in response to an authorized rollback or
a material deployment regression. No rollback was needed during this task.

See [machine-readable evidence](evidence/zellij-netlify-production-2026-11-19.json).
