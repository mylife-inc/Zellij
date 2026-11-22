# Browser timing investigation — 2026-11-13

Status: the two intermittent findings remain unexplained. Repetition did not
reproduce either failure. This milestone adds failure evidence to CI; it makes
no product-runtime change and does not claim to fix either finding.

## Findings and bounded investigation

The [rapid-focus record](zellij-rapid-focus-restoration-2026-11-13.md) retains
the original failures and the separate, resolved focus defect:

- Run `43539410410`, job `132276191750`: WebKit retained the More menu after
  widening to 1600 pixels. Its trace has no painted frame after that resize;
  this does not establish why the update did not finish.
- Run `43540700668`, first Engine job `132280394586`: the hosting-annotation
  preservation check exceeded its 15-second hydration-readiness wait. The
  unchanged Engine rerun passed; no captured browser error explained the delay.

Investigation [43547198588](https://github.com/mylife-inc/Zellij/actions/runs/43547198588)
uses source `4f7e71f6d13ff40e50e424581c3721200eca90b9`, based on `6fceee8`:

- WebKit repeats the original overflow case 120 times with four workers and
  zero retries: 120 passes. Diagnostic wrappers record frame scheduling and
  ResizeObserver delivery without adding frames on the passing path.
- The actual documentation export passes 20 rounds of both annotation
  acceptance cases: 40 passes, zero retries. The other test names are excluded
  by the name filter. All 20 per-round logs are checked and hashed in the
  [evidence](evidence/zellij-browser-timing-investigation-2026-11-13.json).

The investigation's frame wrappers and browser tracing can affect timing, so
these passes cannot prove the original failures are absent. Instrumentation and
probe workflows stay on the investigation branch. The bounded investigation is
complete; repeat failures should be investigated using the additional evidence
below. The open findings need not block unrelated book implementation, but must
remain visible when assessing the next candidate.

## Failure-only evidence in normal CI

Source commits `34d3053` and `f70e942` add a shared browser-state capture used by
the header tests and documentation-preservation checker. It records the route,
readiness and visibility, viewport, focus, navigation measurements and fonts,
theme controls and drawer state. Two observations span a 250 ms timer and up to
two animation callbacks. A 1.5-second Node deadline bounds browser evaluation;
closed or unresponsive pages return a capture error. This uses the page's
current animation-frame function, which a test fixture may override. It is not
proof that native rendering or painting occurred.

The passing header path installs no observers or frame loops. The documentation
checker, when `ZELLIJ_BROWSER_DIAGNOSTICS` is set, also records pending and failed
network requests; it writes state only on failure. Up to 100 pending and 100
failed requests are included. The original browser/runtime/readiness failure
is rethrown even if capture or file writing fails. Existing assertions,
timeouts and retry settings remain unchanged.

CI uploads documentation JSON and header JSON beside retained traces for 14
days. Header JSON is explicitly written through Playwright's per-test output
path before attachment, so the line reporter does not leave it only in memory.
Expected failures in the preservation regressions use disposable directories
and verify their evidence before cleanup; they do not create misleading
unexpected-failure artifacts.

## Verification

Local verification passes the three bounded-capture tests, JavaScript syntax,
showcase typechecking, engine build, Actionlint and whitespace checks. Browser
acceptance runs only on disposable hosted runners.

The initial evidence runs `43548836499` and `43548885977` each pass the real
documentation-preservation check and all six positive/negative browser cases.
Their header probe intentionally fails two assertions as designed, then exposes
a harness defect: attaching a JSON body does not create the file expected by
the artifact uploader. The source correction `f70e942` explicitly persists it.
Those failed runs are retained, not counted as final acceptance.

Final hosted verification of the corrected source passes in
[43549176065](https://github.com/mylife-inc/Zellij/actions/runs/43549176065),
head `914fb03c1819eecddc1dc64f310265c35dd060c9`:

- Both intentional Chromium/WebKit assertion failures retain complete,
  downloadable JSON. The two captures were downloaded, checked and retained
  in the evidence with their hashes. The WebKit capture has no animation
  callbacks during its 265 ms observation despite a measured navigation state;
  Chromium records two. This observation alone does not diagnose the original
  overflow timeout or establish a paint failure.
- The ordinary showcase suite passes 378 cases with 140 conditional skips,
  zero failures and zero retries.
- Documentation preservation passes 46 original and 13 added routes, 282
  original heading IDs, 82 compatibility mappings and 29 byte-identical images.
  All six positive/negative browser regressions and three bounded-capture tests
  pass. No unexpected documentation failure artifact is produced.

The six changed source files match the hosted branch exactly, apart from its
extra deliberate-failure fixture. That fixture and the verification workflow
are not included in the merge. No product runtime or dependency changes need
another native-package, container or release run for this verification-only
increment. The scoped hosted jobs, local checks and source hashes above provide
its acceptance evidence.

Public rc.6, development engine images and production documentation are
unchanged. The next substantive feature remains shared numbered equations and
references across web/PDF, followed by further book semantics and templates.
Live provider verification and production-domain migration remain deferred.

## Follow-up — 2026-11-20

A read-only review of the ordinary CI history inspected 111 runs created from
14–20 November: 36 successful, two failed and 73 cancelled. The first attempts
of both failed runs and all six retried runs were checked. Their failed steps
identify the original hydration timeout and three separately documented CLI
issues: a publication/CDN race, outdated integration-command defaults and the
already-fixed fragment-colon adapter defect. The cancelled Showcase attempt
stopped during system-dependency installation before browser tests started;
its unchanged retry passed. The other retried cancellations were native jobs.

This bounded review found no new evidence for either browser timing finding in
those eight attempts. It did not exhaustively investigate the 73 entirely
cancelled workflows and does not establish a fix or absence of recurrence.
Existing failure-only capture remains the next source of diagnostic evidence.
No runtime change or repeated stress suite was justified by this review. See
[the dated follow-up evidence](evidence/zellij-browser-timing-followup-2026-11-20.json).

## Follow-up — 2026-11-21

A fresh source/log review reproduces a separate desktop overflow focus defect:
a queued callback can steal a newer focus or pointer choice after the menu has
already resized. Fix `f80596b` and its failing-before/passing-after regression
are recorded in the [overflow follow-up](zellij-overflow-focus-2026-11-21.md).
The original overflow and hydration timeouts remain unexplained; do not treat
this separate correction or later passing coverage as their root-cause fix.


## Recurrence — 22 November 2026

Main CI [45007103274](https://github.com/mylife-inc/Zellij/actions/runs/45007103274),
Engine job `137161009186` at `3d3b426`, reproduces the hosting-annotation
preservation timeout at `/cli/dev#what-it-is-for`. The ordinary source run
`45004886385` passed all 37 required jobs on an identical tree; that pass does
not resolve this later failure. No unchanged rerun is counted as a correction.

The [retained diagnostic](evidence/zellij-browser-timing-recurrence-2026-11-22.json)
records no browser errors, 100 capped pending requests and 100 capped aborted
requests, and a 1.5-second state-capture timeout after the original 15-second
readiness timeout. The request inventory spans prior navigations: aborted
prefetches are not proof of a network defect. The capture combines an immediate
DOM snapshot with a page timer before returning either; its timeout cannot
distinguish an unresponsive page from stalled page scheduling. It does not
establish that React failed to hydrate, that the hosting comment caused the
delay, or that a particular product component is defective.

The original finding remains open. The follow-up now returns the immediate DOM
snapshot independently from frame/timer observations, retaining it if scheduling
stalls or sampling throws. Request records carry timestamps and the visit during
which they were observed; the capped inventories keep the most recent entries.
That attribution describes observation time, not proof that a request belongs to
the current document. The original 15-second assertion and 1.5-second capture
deadline remain unchanged. No release tag or product runtime is changed.

Two deterministic VM regressions execute the actual capture function with stalled
callbacks or a failing frame sampler. Both fail against the previous capture
implementation and pass after the change; all six capture tests pass. The real
documentation regressions continue to check rejection of browser/hydration errors,
acceptance of hosting annotations and capture of missing client hydration. These
are diagnostic improvements, not evidence that the original timing failure is
fixed.
