# Remaining external acceptance — 2026-11-10

Cloudflare Pages, GitHub Pages and Railway already passed live acceptance and
their disposable resources were removed. Vercel, AWS S3, Fly.io and actual
GitLab/CircleCI runner execution remain unverified. Tests against simulated
provider responses do not close this gap.

The user authorized creating temporary accounts for these remaining tests.
Account signup was attempted through browser control, but the browser tool
failed during initialization twice before opening a page. No account, paid
subscription or provider resource was created. The first requested handoff is
Vercel signup and a team-scoped token in a GitHub Actions secret. Other provider
signups should follow when their tests are ready, so trials are not consumed
while waiting for implementation or another provider.

## Vercel: ready-to-run disposable project workflow

Workflow: `.github/workflows/acceptance-vercel.yml`.
Harness: `scripts/check-vercel-live.mjs`.

1. Create a temporary Vercel team using an eligible free Pro trial without a
   payment method. For this company product, do not assume the personal Hobby
   plan applies. Vercel documents one Pro trial per user; do not create extra
   identities to bypass trial eligibility. If ineligible, settle an existing
   account or paid test budget with the user before deployment.
2. Save a token scoped to that team as `ZEL_LIVE_VERCEL_TOKEN` in
   `mylife-inc/Zellij` Actions secrets. Keep credentials out of workflow inputs,
   committed files, artifacts and chat. Retain the non-secret `team_…` ID.
3. Dispatch **Vercel disposable live acceptance** on the reviewed source with
   the `team_id` input. The workflow builds the current Rust CLI, uses the
   pinned rc.2 engine image, and creates a randomly named
   `zellij-live-YYYYMMDD-xxxxxx` project. It does not connect a Git repository,
   add a domain, or expose source files. Public previews apply only to this
   newly created project.
4. Four real `zel vc` cases exercise portable/hosted content in preview and
   production modes. HTTP checks cover home/nested pages, exact logo bytes,
   linked resources, missing pages and source-file isolation. This is HTTP
   acceptance, not a browser hydration test. All four deployment receipts and
   results must be recorded before calling Vercel acceptance complete.
5. An `always()` cleanup step independently reads the creation journal, checks
   the project ID/name/team, deletes that exact project and verifies API 404.
   This removes the project's temporary deployments too. It never deletes the
   team/account, existing projects, domains or paid subscriptions.
6. Download `vercel-live-evidence` and record the source/run, four cases and
   cleanup status in durable evidence. Remove `ZEL_LIVE_VERCEL_TOKEN` and revoke
   the temporary token once no more acceptance runs need it. Account/team
   deletion requires checking that it contains only this test's data.

The create request is not retried: a timeout may have created the project even
without returning its ID. The journal preserves the chosen name before sending
that request. If the outcome is uncertain, cleanup fails explicitly instead of
guessing ownership. Inspect that recorded name in the recorded team before
manual deletion. The workflow also needs manual cleanup after a runner loss or
forced job cancellation that prevents its cleanup step from running. Preserve
the journal until deletion is confirmed. A successful command or workflow step
alone is insufficient evidence that resources were removed.

Local regression tests exercise name collisions, identity drift, uncertain
creation, protected previews, failed HTTP checks, cleanup verification and
credential-safe API errors. They use simulated responses and never contact
Vercel. Normal PR CI runs these tests. The real workflow is manual and needs the
temporary account secret; no live success is claimed at this preparation stage.

Official references checked for this work:
[Pro trial](https://vercel.com/docs/plans/pro-plan/trials),
[project creation](https://vercel.com/docs/rest-api/projects/create-a-new-project),
[preview authentication](https://vercel.com/docs/deployment-protection/methods-to-protect-deployments/vercel-authentication),
[project deletion](https://vercel.com/docs/rest-api/projects/delete-a-project).

## Other accounts: prepare when their test window is ready

| Service | Account prerequisite | Intended disposable scope |
| --- | --- | --- |
| Fly.io | New eligible account; current trial advertises credit without a card, limited duration and automatic stopping of trial Machines after five minutes. Recheck before signup. | One app with explicitly configured web Machines; test static and Next images, then remove the app/Machines and test image versions. |
| AWS S3 | Eligible new-account Free plan, or an explicitly budgeted existing account. The Free plan excludes customers who already have or previously had an AWS account. Signup may require additional identity/payment verification. | One website bucket with explicit owner/region and limited credentials; test portable/hosted exports, then remove objects, bucket and temporary credentials. |
| GitLab | Verified user account and a namespace with hosted runner capacity. Email verification is required; phone/card checks may also apply. | One temporary project executing the actual generated pipeline, with artifact inspection and deletion afterward. |
| CircleCI | Account, MFA and repository integration. Current CLI signup still hands off to a browser. | One temporary project/repository executing the actual generated pipeline; remove temporary integration/resources afterward. |

No payment details, identity documents or verification codes belong in project
files or chat. Complete user-facing verification through the provider UI. Do
not describe a free trial as guaranteed eligibility or silently enable paid
usage. Trial credit and compute limits need to accommodate the test, including
image pull/build time and cleanup.

Official signup references:
[Fly trial](https://docs.fly.io/about/free-trial),
[AWS Free Tier](https://aws.amazon.com/free/free-tier-faqs/),
[GitLab verification](https://docs.gitlab.com/security/identity_verification/),
[CircleCI signup](https://circleci.com/docs/guides/getting-started/first-steps/).
