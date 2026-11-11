# Netlify staging hydration reproduction — 2026-11-02

Status: **diagnosed, unresolved**. This report is prepared for review/support;
the user subsequently reports filing the bug and emailing support@netlify.com.
A provider response is pending. No production deployment or domain changed.

## Ready-to-send support request

Subject: Added whitespace before viewport metadata causes React hydration failure

Our static Next.js documentation at
https://zel-stage.netlify.app/cli/completions emits React error 418 when
hydrating. The uploaded HTML runs cleanly on a local static server. Some
Netlify-served responses contain an extra newline and a hosting annotation
between the charset and viewport meta elements. The annotation starts with
`<!-- This site is hosted on Netlify.` and contains `utm_source=ai-legible`.

We disabled **Powered by Netlify badge**. This removed the HUD script, but the
annotation and newline remain, and a fresh browser visit still reports 418.
There was no redeploy between these observations. The same annotation is
present on `/` and `/guide/quickstart/new-project`.

Controlled tests on the exact same static build isolate the cause:

| HTML served locally | Captured hydration errors |
| --- | --- |
| Unmodified upload | 0 |
| Exact observed Netlify HTML | 1 |
| Observed annotation including its leading newline, before viewport | 1 |
| Only the added newline before viewport | 1 |
| Observed annotation without its leading newline | 0 |
| Only the HUD badge script | 0 |

The failure also reproduces in Next's development build. Its diagnostic points
to the root layout's `<style id="zellij-theme">`: React cannot locate the
expected style while scanning past the intervening metadata/whitespace.
Next.js is 16.4.2 and the app's React/React DOM dependencies are 19.2.8.

Can Netlify disable this annotation for this project, or change its insertion
so it does not introduce a text node into `<head>`? Is there a documented
project setting or response header to opt out of this transformation?
Please confirm the appropriate remedy; the badge setting alone did not do so.

The observed responses vary by client: Node's default fetch and the normal
in-app browser exposed the issue; earlier Python urllib responses matched the
upload without additions. This is an observation, not a confirmed rule for
which requests Netlify transforms. No authentication bypass is involved.

## Reproduce locally without Netlify credentials

Use the validated `examples/zellij-docs/out` build, or extract
`backups/zellij-staging-2026-11-02.zip` into a temporary directory. This script
reads the existing build, serves only on loopback, and does not modify it:

```sh
node specs/evidence/serve-head-whitespace-repro.mjs
# Or pass the extracted build directory as the first argument.
```

Open the three printed URLs in fresh tabs. After client hydration, inspect
the browser console. `plain` is the original build, `newline` adds exactly
one LF before `<meta name="viewport"`, and `comment` adds a small HTML comment
at the same position without surrounding whitespace. The latter is a minimal
comment control, independent of Netlify's marketing text. Stop with Ctrl+C.

The blank line is not part of the authored theme stylesheet or content.
Inspection of the installed Next-bundled React DOM code supports the causal
diagnosis: `getNextHydratable` returns text nodes, while `canHydrateInstance`
stops scanning when it encounters a non-element. Ordinary comments are skipped.
This is a framework/host interaction, not evidence that every hosted page or
every React version behaves identically.

## Decision and remaining work

Keep staging available for review and leave the old production site intact.
Obtain a supported opt-out or a tested framework-compatible remedy, then
repeat remote hydration and interaction checks. No verified opt-out was found
in the reviewed Netlify documentation. No warning suppression, client-side
HTML cleanup, dependency patch or hosting workaround was added to the engine.

The existing preservation checker observes uncaught `pageerror` events; it does
not yet collect recoverable React errors reported through `console.error`.
Extend that gate when implementing the remedy. The earlier preservation pass
establishes its stated route/anchor/image checks, not a clean remote hydration
result. The two small released-CLI draft fixtures passed their observed flows
separately; this finding concerns the complete documentation site.

References: [React error 418](https://react.dev/errors/418),
[Netlify badge settings](https://docs.netlify.com/manage/projects/powered-by-netlify-badge/),
[acceptance evidence](evidence/zellij-netlify-acceptance-2026-11-02.json), and
[migration report](zellij-netlify-migration-2026-11-02.md).


## Follow-up — browser acceptance

The user reports sending the issue to Netlify. The checker is being hardened to
capture console.error and wait for a mounted control on each page. Regression
tests require the actual command to reject a head newline and an uncaught
exception; their hosted CI result is pending. This addresses the test coverage
gap described above, not the hosting transformation itself.


PR #59 first run 41594298255 passed full-docs preservation with the new error
capture and hydration wait, but its negative hydration test assumed that React
always uses console.error. Hosted Chromium reported it through pageerror.
The test now accepts either channel for React #418; a third control modifies
only an existing JavaScript asset to emit a console-only sentinel. That control
specifically guards the old console-capture gap without changing the HTML tree.
The uncaught-exception control passed on the first run. Final CI is pending.


## Acceptance gap resolved — 2026-11-02

PR #59 merged as 93dca08 after all 13 checks passed on 97f9f9f in run
41594855467. The checker now collects console.error/pageerror and waits for
hydration. All 55 clean pages passed, and real-checker regressions rejected the
head newline, an uncaught exception and a console-only error. This supersedes
the pending coverage work above; the actual Netlify injection remains unresolved.


## Read-only follow-up — 2026-11-10

Node default fetch still receives the hosting annotation and its leading LF on
`/`, `/cli/completions` and `/guide/quickstart/new-project`. Python urllib
receives clean HTML on those same routes. All six responses returned 200 and
no badge script. This matches the original client-dependent behavior; it does
not resolve the issue or constitute a new browser hydration test. Production
migration remains pending. No deployment, configuration or domain was changed.
See [the HTTP evidence](evidence/zellij-netlify-html-followup-2026-11-10.json).

## Engine remedy under verification — 2026-11-11

The user approved continuing with this issue while keeping live provider
verification deferred. The root layout now emits theme CSS as a React-managed
stylesheet (`href` and `precedence`), whose identity includes a SHA-256 digest
of the CSS. React locates that resource independently of ordinary head-child
hydration. Changed CSS receives a new identity because managed stylesheet
contents are immutable. The synchronous preference bootstrap runs at the start
of the body, before visible content, retaining early restoration without an
ordinary script node in the head.

This uses the documented [React stylesheet API](https://react.dev/reference/react-dom/components/style)
and [inline script behavior](https://react.dev/reference/react-dom/components/script).
It adds no warning suppression, DOM cleanup, dependency patch or export rewrite.
The existing root-attribute suppression for restored visitor preferences is
unchanged. Published `v0.2.0-rc.3` artifacts are unchanged; sites need an updated
engine and a rebuild to receive the remedy.

Five server-rendered root-layout cases cover stylesheet identity, CSS overrides
and all four theme-control combinations. Hosted regressions run the actual
preservation command against all 55 documentation routes after injecting either
LF or a hosting comment surrounded by CRLF. A changed-body-text mismatch replaces
the now-benign head-newline negative control; uncaught and console-only errors
must still fail acceptance. Chromium and WebKit preference fixtures also receive
the annotation while testing pre-hydration restoration, navigation/reload,
disabled controls, unavailable/invalid storage and system mode changes.

Hosted browser results are pending. This controlled reproduction does not replace
a later live Netlify browser check. The production project, staging deployment,
account settings and domain are unchanged; production migration remains deferred.

## Controlled reproduction fixed — 2026-11-11

[PR #129](https://github.com/mylife-inc/Zellij/pull/129) merged as
`360cb4067869bb7e0ffe59834beaeec6fe2cf3b8`, with tree
`8055f29d9ca59c42b6104fd83c3c057d4a01a053` identical to tested head
`938a8bdeeaac3cfc63b7c351278167de4b8d6914`. All 16 jobs in
[CI run 43073305403](https://github.com/mylife-inc/Zellij/actions/runs/43073305403)
passed on the first attempt. Duplicate merge-push workflows were deliberately
skipped for that identical tree; no new image or release was published.

The real preservation command passed the plain build and both head-mutation
cases across all 55 routes. It retained 282 original heading IDs, 82
compatibility mappings and 29 byte-identical authored images. All three
negative controls failed as expected. Chromium and WebKit passed the annotated
preference fixtures and development reload checks. Showcase acceptance had 317
passes and the same 124 existing skips as the rc.3 baseline. Native packages,
portable sites, generated static/Next images, imported sites and CLI export
checks also passed. Locally, 458 engine tests, the build, workspace typechecks,
strict/deep docs validation and static checks passed.

This supersedes the pending controlled-remedy status above. It does not claim
live Netlify acceptance: that check and production migration remain deferred.
The fix is available in source and requires an updated engine plus a site
rebuild; immutable rc.3 artifacts do not contain it. The original observations
and failed baseline remain part of this report. See the
[machine-readable evidence](evidence/zellij-hosted-head-hydration-2026-11-11.json).
