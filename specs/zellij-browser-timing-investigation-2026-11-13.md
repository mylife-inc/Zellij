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
