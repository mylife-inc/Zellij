# Browser scheduling comparison — 28 November 2026

This investigation follows the [historical timing review](zellij-browser-timing-review-2026-11-27.md).
The published CLI remains rc.25. No product behavior, browser dependency,
assertion deadline or normal CI retry policy changes in this experiment.

## Question and method

Does reducing CPU availability reproduce either the Linux WebKit stale overflow
menu with delayed frames or the Chromium documentation hydration timeout?
The two symptoms have separate checks; the experiment does not assume a shared
cause.

The isolated `codex/browser-scheduling-probe` branch at `57fb39f2f6b7ebde6e4b2f5d7885514a53d49d81`
adds only an investigation workflow and harness. It uses current main `8cd1aff`
with the rc.25 focus correction, the locked browser dependencies, Ubuntu runners
and Node 24. The old reproduction's showcase content, navigation measurement
hook and relevant chrome/theme styles are unchanged; the separate navigation
focus correction remains present.

Each browser check runs on a separate runner under three profiles:

- **Available:** inherit the runner's available CPU affinity.
- **One CPU:** the harness and children inherit affinity restricted to the first
  available logical CPU.
- **Contended:** the same single-CPU affinity, shared with one CPU-bound Python
  process at normal scheduling priority.

Builds and browser installation happen before the restriction. Host sampling
records selected CPUs, cumulative CPU ticks, load and CPU-pressure information
every two seconds. The host pressure/load values describe the machine, not only
the restricted process group. They are supporting measurements, not a browser
frame trace. There are no frame overrides or active browser heartbeat loops on
the passing path.

The WebKit check uses two workers, zero retries, 32 distinct contexts and five
narrow/open/Escape/widen cycles in each. The original five-second assertions and
critical Escape-to-resize interval remain unchanged. Failure-only snapshots and
retained traces use the existing diagnostics. Browser processes are reused across
tests until Playwright retires a failed worker.

The Chromium check runs the two existing positive documentation preservation
cases six times each: whitespace and hosting-comment/CRLF annotation. It uses the
actual rebuilt documentation export, original request handling, 15-second
hydration deadline, 90-second subprocess deadline and 120-second test deadline.
Every round has a retained outcome; later rounds do not replace earlier failures.

## Results

[Run 46131796113](https://github.com/mylife-inc/Zellij/actions/runs/46131796113)
reproduces the stale-menu/missing-frame symptom under restricted CPU affinity.
Its six jobs complete on attempt 1. The overall result is failure because both
restricted resize jobs correctly retain failed assertions; these are not retry
passes or a release qualification run.

| CPU profile | WebKit resize cases | Chromium documentation checks |
| --- | --- | --- |
| Available (four logical CPUs) | 32 passed, 0 failed | 12 passed, 0 failed |
| One CPU | 29 passed, 3 failed | 12 passed, 0 failed |
| One CPU plus competitor | 29 passed, 3 failed | 12 passed, 0 failed |

Every resize case has exactly one result at retry zero; none is skipped and there
are no runner errors. The 96 cases request up to 480 cycles, but failed cases
stop early; 480 is not a completed-cycle count. All 36 documentation checks
pass their existing deadlines and preserve the authored routes, headings,
compatibility mappings and assets without captured browser errors.

All six resize failures are the original five-second `More`-count assertion
after widening to 1600 pixels, not a case deadline or focus assertion. Each
immediate snapshot and its follow-up still show only More, visible/focused
status and no captured page errors. DOM evaluation plus the timer sample returns
in 287–331 ms. Five samples contain zero animation callbacks; one contains one
callback but still retains More. The failure traces and snapshots are retained
in the [machine-readable evidence](evidence/zellij-browser-scheduling-2026-11-28.json),
with the exact GitHub artifact SHA-256 values and archives. Captured trace
requests are confined to the repository showcase's loopback server and contain
no authorization or cookie headers.

The selected CPUs average 88.3% non-idle across four CPUs in the normal resize
run, 98.1% on the restricted CPU and 100% with a competitor. This percentage
includes all work on those CPUs, not only the browser. Process-driver warnings
also occur in the passing control and do not establish a native cause.

This is new evidence that restricted scheduling can reproduce the resize
symptom. It does not identify the native rendering failure or prove the same
CPU conditions existed in the historical CI failures. The hydration timeout
remains unreproduced in this experiment.

## Trigger reduction

An additional bounded comparison at `f4cf635fd14420c7211542ac6b49b9024bd6ec02`
uses the one-CPU profile with five arms: unchanged showcase; only Escape's native
default cancelled; showcase CSS transitions/animations/filters disabled; a
small DOM-only navigation fixture with effects; and that fixture without effects.
The DOM fixture uses the pure item-fitting calculation but no React, Next.js,
Zellij navigation component or application focus hook. Each arm requests 32
cases and five cycles per case with the same assertion deadlines.

[Run 46132460136](https://github.com/mylife-inc/Zellij/actions/runs/46132460136)
passes all five arms: 32 cases each, 160 cases / 800 cycles total, zero retries,
failures, skips or runner errors. Because the unchanged one-CPU control also
passes, this run provides **no differential evidence** that Escape cancellation
or removal of effects prevents the failure. Neither the DOM-only fixture nor the
full-site comparisons establish a necessary rendering trigger. The symptom
remains intermittent even under restricted affinity. All five original artifacts
are hash-verified and retained with the evidence.

## Native scheduling capture

A focused comparison at `b1f96438956b0526e59f5b1a32c07a581d6707da` keeps the unchanged showcase, two workers,
original assertions and zero retries. It requests 64 cases per restricted profile
(one CPU, and one CPU plus competitor). Its host sampler also records numeric
CPU counters, process state and wait channel for native WebKit/MiniBrowser
processes. Arguments and environments are excluded. These measurements can help
distinguish expensive rendering from idle native processes while frames stop;
they are not native stack traces and sampling can itself affect timing.

[Run 46132933692](https://github.com/mylife-inc/Zellij/actions/runs/46132933692)
again reproduces the stale-menu assertion: each profile passes 61 cases and fails
three, with zero retries, skips or runner errors. All six snapshots initially
show only More at 1600 pixels. Four samples record no animation callback and
two record one; one of the latter recovers all five menu entries during capture.
The twelve failures across the two scheduling runs are retained, including this
late recovery. Passing subsequent cases do not erase them.

The native sampler matched process names and captured **MiniBrowser only**:
the rendering subprocesses did not match that filter. UI-process polling with
little CPU activity therefore cannot establish idle rendering or identify a
native deadlock. The retained evidence explicitly preserves this diagnostic
limitation.

The follow-up at `52fa2b06eab1b430b35828ab014417fcb778b1cd` matches executable basenames as well as process names,
excludes arguments/environments, and requires captured web-process coverage.
[Run 46133839279](https://github.com/mylife-inc/Zellij/actions/runs/46133839279)
passes the coverage check and captures `MiniBrowser`, `WPENetworkProcess` and
`WPEWebProcess`. The one-CPU arm passes 31 cases and fails one; the contended arm
passes 30 and fails two. These are again five-second stale-menu assertions,
with no retries, skips, runner errors or captured page errors. The two contended
failure samples record no animation callback; the one-CPU sample records one.

Both concurrent web processes advance their CPU counters during each failure
window. Over pairs of samples strictly within the five seconds before capture,
they use approximately 45–50% of one CPU each in the one-CPU failure, and
26–38% each in the two contended failures. The main-thread wait channel often
shows polling, but that does not mean the process's other threads are idle.
This is evidence of continuing native work, not an identified deadlock or a
specific expensive rendering function. The sampler does not bind a native PID
or stack to the failing page, so it must not assign one process's work to a
particular page. Both concurrent web processes are retained in the evidence.

In total these bounded runs retain **15 stale-menu/frame-delay failures** on
current source. The next diagnostic target is the native work consuming CPU
during the delay: thread-level profiles/stacks and precise page/process
attribution. A browser update or product change would still require comparison
against the failing condition; neither a timeout increase nor passing repetitions
would establish a correction. The separate hydration timeout remains unresolved.

## Boundaries

This is a bounded scheduling experiment, not a reconstruction of every process
running during the original failures. Each profile uses a separate hosted runner;
CPU affinity and contention are deliberate perturbations, while build/run noise
is not fully controlled. The current source contains the independently proven
focus correction; this experiment must not reclassify that correction as the
historical missing-frame or hydration root cause.

No live provider deployment, external catalog change or distribution retirement
is part of this work. The probe workflow stays on its investigation branch.
