# Browser timing follow-up — 27 November 2026

Both historical timing findings remain open. This review finds two additional
WebKit resize failures that passed on Playwright retry. Successful job status
alone must not be described as a clean first attempt. No product fix follows
from these observations yet.

The subsequent [rendering reduction](zellij-overflow-focus-commit-2026-11-27.md)
finds and fixes a separate focus-restoration race while frames continue. Its
natural failures and controlled before/after regression do not close either
historical timing finding described here.

## Review scope

The [evidence](evidence/zellij-browser-timing-review-2026-11-27.json) inventories
98 ordinary CI runs created between 22 November 04:02 UTC and 27 November
07:10 UTC: 48 successful, 44 cancelled and six failed. It reviews 108 attempt
inventories and 89 browser-job logs, including successful Showcase jobs and
cancelled jobs that reached browser execution. All requested logs were retrieved.

The two logs with retry markers identify the same overflow test. No additional
documentation hydration timeout was found beyond the already-recorded
`45007103274` / `137161009186` recurrence. This is bounded evidence: separate
release, documentation-publication and provider workflows are outside the
review; successful Engine jobs were inspected by step outcome rather than by
reading every log. Cancelled tests may lack final results.

## Additional resize recurrences

Both failures occurred in `measured overflow › moves trailing links into More
and restores them when space returns`, using Playwright 1.62.1 / WebKit 2336
(26.5), Linux, two workers and the Desktop Safari device profile. The profile's
Safari user agent does not mean these were tests on macOS Safari.

| CI run / Showcase job | Source | Initial result | Retry result |
| --- | --- | --- | --- |
| [45085352094 / 137402593006](https://github.com/mylife-inc/Zellij/actions/runs/45085352094/job/137402593006) | `d738c6f` | More remained after widening from 901 to 1600 pixels; 5-second assertion failed | Passed; final summary 551 passed, one flaky, 169 skipped |
| [45133775474 / 137555053722](https://github.com/mylife-inc/Zellij/actions/runs/45133775474/job/137555053722) | `77a909c` | Same assertion and viewport transition | Passed; same final summary |

The first failed at 23:54 UTC on 22 November; the second at 07:53 UTC on
23 November. Both Showcase jobs finished successfully; the first overall CI
run failed in another job. Neither retry resolves the initial failure.

GitHub artifact hashes were verified before extracting the matching trace,
error context and state. The evidence retains artifact/member hashes, both
complete state snapshots and the relevant trace events. The original
[22 November trace](evidence/browser-resize-2026-11-27/45085352094-trace.zip)
and [23 November trace](evidence/browser-resize-2026-11-27/45133775474-trace.zip)
are also retained here (631,899 bytes combined), so analysis remains possible
after the temporary artifacts expire on 30 November. Their captured requests
are confined to the repository showcase's loopback server and contain no
authorization or cookie headers.

### What the traces establish

- In the first recurrence, the first recorded wide frame arrived 5,001 ms after
  the resize call completed. That was near the assertion deadline. The subsequent
  diagnostic snapshot already showed all five links restored and recorded two
  animation callbacks. This is a late recovery, not a permanently incorrect
  overflow calculation.
- In the second recurrence, the trace contains no wide frame after the resize.
  Both diagnostic snapshots still showed only More. Synchronous evaluation and
  the 250 ms page timer completed, but no animation callback arrived during the
  sample. The document reported itself visible and focused, with loaded fonts,
  a 1600-pixel viewport and hydrated theme controls.
- Both failures occurred after Escape closed the menu and focus returned to
  More. The normal test installs no frame overrides. Navigation measurements
  use ResizeObserver followed by requestAnimationFrame, so stopped rendering
  can leave the previous item count while ordinary JavaScript still responds.

These observations narrow the problem to delayed rendering-related work. They
do not identify why frames stopped, establish that Escape caused it, or prove a
specific native browser defect. A trace's lack of screencast frames alone is
not proof that no painting occurred; the separate callback sample strengthens
that observation only over its measured interval. The earlier focus correction
`f80596b` remains a separate resolved defect.

## Controlled comparison

The isolated investigation branch compares the original test with a variant
that cancels only Escape's native default action. Application propagation,
assertions, five-second deadline and product code stay unchanged. Each variant
runs 60 times with two workers and zero retries, against the same locked
Playwright version. Browser-process logs and structured results are retained.
There are no frame wrappers or extra animation loops on the passing path.

Run [46063043070](https://github.com/mylife-inc/Zellij/actions/runs/46063043070)
uses `48f02bacd5d6c711e7f7c61a3b6ccc695a216f91`. Both variants pass 60/60,
with zero retries, failures or skips. Their checked JSON reports contain exactly
one result at retry zero for every case. Neither arm reproduces the failure,
so there is no differential evidence implicating Escape or supporting a runtime
change. Browser-process logging can itself affect timing.

Those isolated repetitions launch 60 fresh browser processes per arm. In both
historical failure logs the overflow case follows the two leading header tests,
while another worker runs focus regressions. A second bounded run preserves the
whole unchanged header group, including normal browser reuse between its cases:
[46063731462](https://github.com/mylife-inc/Zellij/actions/runs/46063731462),
head `6c7114774959083757ae1d609025c3b43fe91a49`, 20 repetitions, two workers,
zero retries. All 500 cases pass: 25 scenarios, each exercised 20 times, with
one result at retry zero for every case. There are 40 browser launches across
the two workers and 20 groups. This includes the preceding cases but does not
reproduce the historical stall or establish a root cause. Both findings remain
open after 620 passing controlled trials in total.
The probe workflow stays on its investigation branch; none of its code changes
the product or normal CI.

## Make retries visible in ordinary CI

Source `d63f491` / `6264495` closes the reporting gap. The Showcase Playwright
step now writes a JSON report alongside its line output. A subsequent Actions
summary separates expected outcomes, flaky tests, unexpected outcomes, skips
and runner errors. A flaky result emits a warning and points the reviewer to the
retained evidence by artifact name; assertions, retry policy and the test
command's success/failure decision are unchanged. Missing reports do not
produce a fabricated zero-failure summary. Malformed counts fail reporting.

Normal CI uploads the structured report with its traces and state captures for
14 days, including successful runs. Four focused tests cover count separation,
malformed input, global runner errors and the real command's summary/warning
behavior for a passing retry. All four pass locally; Actionlint and whitespace
checks pass. Scoped hosted check
[46065076326](https://github.com/mylife-inc/Zellij/actions/runs/46065076326)
passes the four tests and consumes the actual full-group report. The retained
Actions summary correctly reports 500 expected outcomes and zero flaky,
unexpected, skipped or runner-error results. Its two reporting-source hashes
match the files being merged; the report hash matches the 500-case run. The
product/CLI code is unchanged; native builds and provider deployments do not
need repeating for this reporting change.

## Remaining boundary

The existing hydration evidence cannot distinguish a stalled page from delayed
scheduling; diagnostic correction `bfb450b` now retains immediate state separately
if it recurs. This review supplies no basis for changing the hydration check,
product rendering, timeouts or retry policy. Rc.24 remains current. No new CLI
release, production publication or provider deployment is part of this review.
