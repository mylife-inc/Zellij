# Remaining external acceptance — 2026-11-10

Cloudflare Pages, GitHub Pages and Railway already passed live acceptance and
their disposable resources were removed. Vercel, AWS S3, Fly.io and actual
GitLab/CircleCI runner execution remain unverified. Tests against simulated
provider responses do not close this gap.

**Current decision:** the user deferred Vercel verification after trial signup
required a payment method. No live Vercel case ran and no test project was
created. The temporary `ZEL_LIVE_VERCEL_TOKEN` Actions secret and remote
`codex/vercel-account-preflight` branch were removed. This removes the GitHub
copy of the credential; provider-side token revocation was not performed. The
account/team and tested adapter/workflow remain unchanged. The Vercel setup
steps below are retained for a future explicitly resumed test, not a current
user handoff.

The user authorized creating temporary accounts for these remaining tests.
Account signup was attempted through browser control, but the browser tool
failed during initialization twice before opening a page. No account, paid
subscription or provider resource was created. The initial requested handoff was
Vercel signup and a team-scoped token in a GitHub Actions secret; it is now
superseded by the user's deferral above. Other provider
signups should follow when their tests are ready, so trials are not consumed
while waiting for implementation or another provider.

## Protocol defect found while preparing acceptance

Reviewing the official file-upload protocol found a concrete defect: `zel vc`
required JSON after `POST /v2/files`, but Vercel documents HTTP 200 with an empty
body. Changing the local HTTP fixture to that documented response reproduced
`asset upload: invalid API response` before deployment creation.

Fix `36028b4a2a2a39a0bf914fc4be90947e3c02959f` accepts empty successful asset
acknowledgements and JSON objects, while retaining strict JSON for project,
creation and deployment-status responses. Malformed/error acknowledgements,
redirects, HTTP failures and oversized responses still fail before creation.
The hosted-export fixture now uses the empty acknowledgement too. This fix is
newer than published rc.2 and does not itself establish live account success.
See the [upload API](https://vercel.com/docs/rest-api/deployments/upload-deployment-files)
and [official upload client](https://github.com/vercel/vercel/blob/main/packages/client/src/upload.ts).

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


## Account handoff result — 2026-11-10

The token works. Read-only run `42964231875` resolved `my-life-inc-s-projects`
to `team_JYk8MJycfmZ3sldE048ljr7Z`, but this team is on active Hobby with no
trial. The user was asked to create a Pro Trial team, update the same secret
with a token scoped to it, and send its slug. No projects/deployments were
created. Browser control still cannot initialize in this Codex session.

The temporary `codex/vercel-account-preflight` branch makes GET requests only.
It can resolve the new slug without retrieving the token from Actions. Do not
merge that operational branch into main; retain the tested lifecycle workflow
on main and remove the operational branch after this handoff is complete.
See [the account evidence](evidence/zellij-vercel-account-preflight-2026-11-10.json).


## Fly: disposable acceptance prepared — 2026-11-11

Source `b6a78de7c212f52bf98167aa055952ca5651be67` adds
[the manual workflow](../.github/workflows/acceptance-fly.yml) and
[lifecycle harness](../scripts/check-fly-live.mjs). Ten isolated lifecycle tests
pass; the complete helper gate passes 60 tests and actionlint passes. This is
preparation, **not live Fly acceptance**. Cross-platform PR CI is still pending.

The workflow warms both image builds before starting trial compute. It creates
one randomly named app in the explicitly selected organization, with an isolated
private network, a shared IPv4 address and one shared-CPU/512-MiB web Machine.
It copies the pinned rc.2 acceptance image into that app's registry for initial
health, then runs the real current CLI for portable static and hosted Next
deployments. Public HTTP checks cover pages, exact logo bytes, rendered assets,
missing routes and source-file isolation. These are not browser hydration tests.

Trial Machines currently stop after five minutes. The fixture is stopped and
started before each case to reset its test window; the harness never restarts
or retries a deployment midway. Prewarming reduces build time but does not
guarantee a case fits the trial window. If the trial prevents acceptance, record
that limit and settle the next step before using a paid plan.

`always()` cleanup deletes only the app identified by its confirmed creation
receipt, matching ID/name/organization, and polls until the API returns 404.
The app deletion covers its Machine and assigned networking. Image digests are
recorded; physical registry-blob garbage collection is provider-managed and is
not independently verified. The account/organization is never deleted by this
workflow. Name collisions cause no writes. An ambiguous app-creation outcome
without a receipt requires manual inspection; no deletion by guessed name runs.
Runner loss or forced cancellation can prevent cleanup: retain the journal and
inspect only its recorded organization/app before any manual deletion.

### One-time account prerequisite

Browser automation was retried on 2026-11-11 but failed before opening any page
because Codex's writable-root configuration traverses the `~/.codex` symlink.
No account, token, subscription, app or Machine was created in that attempt.
This is an environment failure, not an approval rejection.

1. Open [Fly signup](https://fly.io/app/sign-up) with the intended GitHub identity.
   Use an eligible no-card trial. If unavailable, stop before adding a payment
   method; do not create extra identities to bypass trial eligibility.
2. Note the trial organization's slug and confirm its trial is active without
   a payment method. Use a dedicated test organization when available.
3. Authenticate the official Fly CLI to that account, then create an expiring
   organization-scoped token:

   ```sh
   fly tokens create org --org YOUR_ORG_SLUG --name zellij-cli-acceptance --expiry 24h
   ```

4. Save the token directly as `ZEL_LIVE_FLY_TOKEN` in
   [Zellij Actions secrets](https://github.com/mylife-inc/Zellij/settings/secrets/actions).
   Send only the organization slug and trial confirmation in chat. Never send
   the token. App-scoped tokens cannot provision the disposable app.
5. Dispatch the reviewed workflow with that organization, an available region
   (default `iad`), and `trial_confirmed=true`. The workflow never accepts a token
   as an input or writes it into evidence. No custom domains or existing apps
   are changed.
6. Record both cases and confirmed cleanup from `fly-live-evidence`; remove the
   temporary Actions secret and revoke the token when testing finishes. Account
   deletion, if requested, must first verify it holds only this test's data.

References checked: [trial limits](https://fly.io/docs/about/free-trial/),
[app creation and deletion](https://docs.fly.io/machines/api/apps-resource),
[Machine configuration and lifecycle](https://docs.fly.io/machines/api/machines-resource),
[organization tokens](https://fly.io/docs/security/tokens/).
