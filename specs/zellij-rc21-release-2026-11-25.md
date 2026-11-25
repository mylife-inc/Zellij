# Rc.21 release preparation and timeout follow-up

Status: rc.21 is published and installed; channel upgrades pass. Revised guides
pass local checks and are ready for pull-request validation and publication.

## Short workflow checkpoint

PR #257 merged as `0adba60`, tree-identical to tested `f00aca6`. The PR passed
53 checks, including 37 general CI jobs and all nine PDF qualification jobs.
The first Mac portable job was canceled before runner assignment; its targeted
retry passed unchanged. Production documentation `45688978580` and base images
`45688978546` passed. Live browser checks confirmed the Quickstart index, book
guide, website-template guide and shortcut reference returned HTTP 200, hydrated,
contained the new recipes and captured no console or page errors.

## Post-merge timeout finding

Main CI `45688978899` passed 36 jobs but its Mac portable job `139469052544`
failed `vercel::tests::deadline_stops_polling_without_recreating_deployment`.
The test expected a timeout message after a 150 ms deployment deadline. It had
printed the created deployment ID, but its assertion did not include the actual
error. This differs from the earlier unassigned-runner cancellation and must
not be recorded as a passing run or assumed to be a capacity failure.

Two hundred local executions of that test did not reproduce the intermittent
failure. Inspection found a concrete diagnostic gap: a timeout after receiving
HTTP headers was mapped to a generic response-read error. A controlled loopback
fixture sends partial bytes, holds the connection until the client returns, and
reproduced `deployment status: could not read response` before the fix.

Fixing commit: `08a0b28`. The Vercel client now recognizes the elapsed deadline, the I/O timeout kind and
the underlying HTTP-client timeout when reading a body. It reports a timeout
without including partial bytes, URLs or credentials, and advises checking the
dashboard before retrying. Ordinary truncated responses remain read errors.
No deployment creation is retried. The existing end-to-end deadline assertion
now prints its redacted error if it fails again.

The controlled defect is demonstrated; the old CI assertion did not capture
enough information to prove it took this exact path. Preserve that uncertainty
even if subsequent platform checks pass. Separate historical browser timing
findings and deferred external-provider/catalog verification are unchanged.

## Publication gates

Local verification passes 858 Rust workspace tests (five existing hosted-renderer
tests ignored locally), warnings-denied Clippy and both production/development
dependency audits. The 15 runnable Vercel tests pass, including the controlled
stall, truncation, redaction and one-creation deadline assertions. A further 200
executions of the original deadline test pass after the change.

Run the targeted Vercel regressions, workspace tests, Clippy and dependency audit,
then require exact-head CI before merging the fix and tagging the candidate.
The tag must remain immutable. Verify the actual public CLI, renderer, PDF
runtime and corresponding sources from both release repositories before
promoting the installation channels, updating the local CLI and changing the
guides from upcoming to released rc.21. Keep publication receipts and actual
rc.20-to-rc.21 channel-upgrade evidence distinct from source tests.

## Publication result

PR #258 merged as `cd7608e`, matching the tested tree at `178c8e0`. PR CI
`45703096716` and post-merge CI `45707147021` each pass all 37 jobs without
reruns. Base images `45707147017` pass. The annotated release tag remains fixed
at that merge. Release `45707661537` completes with 55 passing jobs and one
conditional skip on its first attempt.

Both public repositories expose 46 matching assets. The six public PDF jobs
verify actual compiler/renderer bytes and the complete matching source archive;
all six native installation jobs and both Linux image architectures pass.
Independent Mac checks repeat both public repositories. The installed standalone
CLI also passes cold/default-home and offline PDF, rich file and HTTP builds
with an empty PATH. Default PDF now uses the verified cache; explicit container
selection remains supported and ordinary builds still opt in to PDF.

Homebrew and Scoop use the exact public recipes, and all eight actual
rc.20-to-rc.21 upgrades pass. Three new immutable book archive identities are
registered from their verified public catalog. No prior releases or images were
retired. Compiler source and notices retain their distribution obligations.

The revised guides and use-case Quickstarts pass strict validation, a production
build and browser preservation across 61 routes, 282 original headings,
82 compatibility mappings and 29 unchanged images with no captured browser
errors. Their pull request and automatic production deployment follow this
checkpoint. Exact hashes, run IDs and upgrade logs are in the
[publication evidence](evidence/zellij-rc21-release-2026-11-25.json).
