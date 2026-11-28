# Downloaded renderer focus qualification

This harness copies the controlled callback-order regression from
`examples/showcase/e2e/header-layout.spec.ts`. It opens a generated showcase with
`file://`, blocks all HTTP requests and requires no page errors. Both resize
directions run twice in Chromium and WebKit, with retries disabled.

Create two isolated showcase fixtures using byte-identical
`examples/showcase/content` and `zel.yaml` containing `version: 1` and
`renderer: hosted`. Build each with its verified published CLI, an empty `PATH`,
an isolated `ZELLIJ_HOME`, and `ZEL_ENGINE_BASE_URL` pointing to that candidate's
public release. No locally compiled engine is used. Use each resulting absolute
`file://.../site/index.html` URL in turn:

```sh
ZEL_FOCUS_URL='file:///absolute/path/to/site/index.html' \
ZEL_FOCUS_RESULTS='/absolute/path/to/results' \
node node_modules/playwright/cli.js test \
  --config specs/evidence/rc25-public-focus/playwright.config.ts
```

The local qualification uses installed Google Chrome (`channel: chrome`) and
Playwright WebKit. Reports retain traces on failure and attach the HTTP-request
and page-error arrays for every case. A failure counts as the reproduced bug
only if every setup assertion passes and the final focused-navigation assertion
fails; unrelated setup failures do not establish the regression.

See `../zellij-rc25-release-2026-11-27.json` for exact outcomes, versions,
source/run identities and hashes. The controlled scheduling reproduces the
focus-request ordering bug; it does not reproduce or close the two historical
browser rendering/hydration timing findings.
