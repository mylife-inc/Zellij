# Netlify migration and content-preservation review — 2026-11-02

Status: **staging is public at https://zel-stage.netlify.app; both CLI draft
deployments and their HTTP/browser checks passed.** Full-docs content checks pass,
but a Netlify HTML-injection hydration error remains under investigation. The
user created the project and set its visibility/credentials; the agent created
only the two authorized draft deployments. Keep the existing production project
and domain in place; deletion, transfer and production cutover remain unapproved.
The user requires that the replacement retain the information currently
published at https://zellij.shebka.net.

## Account finding and recommendation

The user reports that the existing deployment originated from GitHub account
`hamidbm`, whereas current development uses `mylife-inc/Zellij`. GitHub identity
and Netlify project ownership/access are separate. A repository link can be
changed, and a Netlify project can be transferred between eligible teams; a
new GitHub repository does not by itself require deleting the Netlify project.
Prefer preserving/transferring the existing project if the intended team is
eligible. A separate staging project is another option, with the production
domain left on the original project until validation and an explicit cutover.

Netlify's current instructions:

- [Transfer a project](https://docs.netlify.com/manage/projects/transfer-project/):
  the source Team Owner needs Owner/Developer membership on the destination
  team and transfers must be allowed. Inspect the transfer dialog and any
  plan/feature effects before confirming.
- [Repository linking](https://docs.netlify.com/build/git-workflows/repo-permissions-linking/):
  an existing project's linked repository can be changed. Linking enables
  continuous deployment, so do not turn it on casually during comparison.
- [Download a deploy](https://docs.netlify.com/deploy/manage-deploys/manage-deploys-overview/#download-a-deploy):
  open the current successful production deploy, choose **Download**, then
  **Download ready** for the complete deployed-file ZIP. Preserve that ZIP and
  the deploy ID before a transfer/cutover. It is not the original source tree.

The user inspected the transfer dialog and reported that the intended
destination is unavailable. Prepare a separate staging project under the intended
Netlify account/team, retaining the current live project and domain. Neither repository visibility nor CodeSeal
migration is part of this hosting change.

## What has been verified

Compared the live sitemap, public search index and internal page links with
the actual source-loaded route inventory and a fresh local static build at
`0d28063aad1194a166fb599ed9eae1b86e348027` (product source from PR #53).
Fetched all discovered production pages successfully. Extracted main/article
text, excluding navigation, scripts, SVG and explicitly hidden content. This
compares information, not interactive behavior or pixel appearance.

| Check | Result |
| --- | --- |
| Published pages discovered | 46; all HTTP 200 |
| Local authored routes | 55 |
| Published routes missing locally | **0** |
| New local routes | 9 CLI reference pages |
| Identical normalized main text | 13 pages |
| Revised main text | 33 pages; removed/replaced blocks reviewed as described below |
| Directly referenced same-origin assets fetched | 40; all HTTP 200 |
| Published authored images matched to source | **29/29 byte-identical** |
| Old heading IDs absent in new rendering | **82 across 16 pages at baseline; all restored below** |
| Local documentation build | Passed with static export and webpack; no Docker |

New routes: `/cli/completions`, `/cli/eject`, `/cli/features`, `/cli/pull`,
`/cli/shortcuts`, `/cli/upgrade`, `/cli/version`, `/cli/view`, `/cli/watch`.

The identical-text pages are `/`, `/why`, `/guide`, `/guide/content-model`,
`/guide/converting`, `/guide/pages-and-sections`,
`/guide/quickstart/existing-docs`, `/guide/quickstart/from-scratch`, and all five
section-family pages (`grids`, `media`, `narrative`, `openers`, `proof`).

The [machine-readable comparison](evidence/zellij-production-comparison-2026-11-02.json)
contains all route response hashes, changed-page sizes/headings, asset matches
and every missing legacy heading ID. Page count and longer text do not prove
that all old information survives. A staging comparison has been prepared; production cutover still needs
review of the actual remote staging deployment.

## Revised information and remaining gaps

The removed/replaced blocks across all 33 revised pages were reviewed. They
contain intentional corrections and expanded/reorganized explanations:

- Nineteen section types became 23; Windows now has native candidate packages.
- Planned CLI commands became implemented commands, including serve, publish,
  formatting, maintenance and wrapper extraction.
- Historical `zel.toml`/upward discovery and `docs`/`dist` examples were replaced
  by the user-agreed `zel.yaml`/`content`/`site` convention.
- Portable direct-file output, static-server images, editable Next wrappers,
  saved publication, bounded runtime discovery and real CI triggers add detail.
- Claims about automatic authentication, unsupported providers/options, image
  contents, ignored JSON and validation guarantees were corrected or qualified.
- Obsolete approximate package sizes and automatic optional-feature claims were
  replaced by explicit feature-selection behavior.

Do not restore obsolete commands as executable instructions merely to match old
wording. Preserve useful explanations and future-product scope with clear status.
The original CLI inventory and contract retain dated scope and prior decisions.
The preservation snapshot below retains the fetched old prose for line-by-line
review. No missing route or authored image was found. This is not a claim that
the wording is identical or that automated text comparison proves equivalence.

Before any new production publication:

1. Review the prepared staging site against the retained production snapshot.
   The source review classified the 33 revised pages as corrections, expansion
   and reorganization; obsolete promises remain available in the archive and
   dated CLI inventory. Resolve any additional missing material identified in
   this review rather than relying on page or word counts.
2. Re-run the preservation check against the deployed staging output. The
   **82 legacy heading links are now mapped** using 74 static aliases and eight
   restored headings. All **282 original heading IDs** resolve exactly once in
   the local browser check; fragment navigation includes sticky-header offsets.
3. Verify the corrected repository links: navigation/footer now reference
   `https://github.com/mylife-inc/Zellij`. Company branding is unchanged.
4. Preserve a complete Netlify deploy ZIP, original deployment/source revision
   if available, and relevant domain/redirect/build settings. The public crawl
   cannot discover unlinked/unindexed files or provider settings.
5. Verify the selected Netlify account can access the selected project. The two
   prior attempts failed preflight, so no live draft acceptance exists yet.
6. Deploy the complete documentation to a draft/staging URL, check every route,
   legacy anchor, asset, navigation and search, and compare content again.
7. Confirm production cutover separately. Keep the old project/deploy available
   for recovery; deletion is not needed to validate a replacement.

## Local preservation snapshot

A local, Git-ignored archive is retained at:

`backups/zellij-production-2026-11-02.tar.gz`

- Size: 12,619,071 bytes.
- SHA-256: `6b3a447049d8b7e1d7768bff5145dbc6d2fc0becdac030f1cf964f44f555f422`.
- Contains fetched HTML, text/headings/blocks, response hashes, 40 referenced
  assets, search index, sitemap, comparison results and the audit scripts.
- This is a public-content preservation snapshot, **not a complete redeployable
  Netlify backup**, source checkout, or copy of credentials/settings. Full ZIP
  download from the existing Netlify deployment remains a separate prerequisite.
- Raw responses were not committed to GitHub. The small comparison manifest is
  tracked; the archive remains in this checkout for future review.

The build regenerated some source images and `next-env.d.ts`; those audit-only
side effects were restored to their pre-build committed state. Image equality
above uses the restored original source assets. The build output itself is
still a local ignored artifact, not a deployed replacement.

## Delivery status entering this migration discussion

PR #53 merged as `bc3955b` after 19 executed checks passed. Both dedicated
static and Next test images published from the released `0.2.0-rc.1` CLI and
passed fresh-pull acceptance in run `41565958521`, using checker fix `d2844ab`.
That checker correction merged in PR #54 as `35a2261` after all 13 CI jobs
passed in `41569977319`. PR #53 main CI `41564319708` also passed. Initial failures remain recorded in the main audit.

Portable Netlify runs `41564369100` (stored site ID) and `41565327817` (temporary
public-domain override) both received HTTP 404 at GET-site preflight, before
building or creating a deployment. The override was removed. Secret values
were never read or changed; production was not deployed. The home, CLI and
quickstart response hashes matched before and after. The hosted draft was held.


## Prepared staging artifact and regression checks

- Staging upload: `backups/zellij-staging-2026-11-02.zip` (27,007,238 bytes),
  with index.html at its root and a successful ZIP integrity check.
- SHA-256: `d774bdcfacf9dcaeb628c7d72900bd274067611d376593a4295262f81864f21b`.
- Strict/deep content validation passed: 55 routes, 53 guide pages, 23 examples.
- Static webpack build passed using the checked-in authored images. Image
  generation was deliberately not rerun for this upload.
- `scripts/check-docs-preservation.mjs` passed in local Chromium: 46 original
  routes, nine added routes, 282 original heading IDs, 82 compatibility mappings,
  29 exact image hashes, current repository links, old-fragment navigation and
  an actual search-result navigation. External browser requests were blocked.
- The compatibility mappings are in
  [the alias manifest](evidence/zellij-legacy-heading-aliases-2026-11-02.json).
- Normal Engine CI now builds this documentation and runs the same check. The
  test uses preserved production hashes/IDs, so removing a page, anchor or
  original image fails rather than silently reducing the site's coverage.
- A matching headless browser was installed only in the temporary
  `/private/tmp/zellij-playwright-cache`; no local Docker daemon was started.

The user has been given the staging ZIP/folder and instructions to create a new
manual-deploy project in the intended Netlify account, retain its temporary
netlify.app address, and set the existing GitHub Netlify secrets to that new
project. New destination details and readiness are pending. These instructions
do not authorize moving the production domain or deleting the original site.

## Completed source acceptance and user handoff

[PR #55](https://github.com/mylife-inc/Zellij/pull/55) merged as
`f1ca1ff04f049366a1175b4dfe2bd6e78a6c34a1`. All 13 CI jobs passed in
[41573372928](https://github.com/mylife-inc/Zellij/actions/runs/41573372928)
on final head `27947739e2bfdfa126f323aca847e970627c71bc`, including the new
documentation-preservation gate. PR #54 main CI `41572436263` also passed.
Source acceptance does not establish acceptance of a remote Netlify deployment.

The next user actions are:

1. Log into the intended Netlify account/team and create a separate project by
   manual deployment using the staging ZIP above, or the extracted files with
   `index.html` at the root. Keep its temporary `netlify.app` address.
2. Update `mylife-inc/Zellij` repository Actions secrets: `NETLIFY_SITE_ID` must
   identify the new project; `NETLIFY_AUTH_TOKEN` must have access to it. Save
   values directly in GitHub, not in chat. Do not revoke an old token if other
   sites still rely on it.
3. Send the temporary public URL and confirm that both secrets are ready. The
   agent can then verify the full staging docs and run portable/hosted CLI
   deployment acceptance as drafts on that selected project.
4. Before considering cutover, download the old project's full currently
   published deploy ZIP and retain its deploy ID and relevant build, redirect
   and domain settings. Review the staging content and remote checks first.

Read-only DNS observation on November 2: `zellij.shebka.net` has CNAME
`zellij-docs.netlify.app`; `shebka.net` uses `ns1.hostmonster.com` and
`ns2.hostmonster.com`. These settings have not changed. The existing project
remains the production destination and recovery reference.

A read-only remote checker is prepared to compare the 55 served HTML responses
with the validated upload, then verify legacy headings, images and search in
Chromium. It has passed syntax validation only. The staging URL is still needed
to run it; neither remote preservation nor live Netlify draft acceptance is
claimed as passed.


### Live Netlify draft acceptance and staging browser finding — 2026-11-02

The user created https://zel-stage.netlify.app, made the main site public,
updated NETLIFY_SITE_ID and generated a new NETLIFY_AUTH_TOKEN. Secret names and
update dates were checked without reading values; no repository variable
supersedes the site-ID secret. RELEASES_GITHUB_TOKEN already exists and was
left unchanged. The user subsequently made previews public after the first
portable preview request returned HTTP 401. Netlify treats main-site and
preview visibility separately.

Both actual generated workflows ran on 311421951ac9c6d3bf1904f150b0e1c79d85ab81
using the public 0.2.0-rc.1 CLI and exact engine image. No --prod flag was used.

- Portable run 41587034632 passed; draft:
  https://6ae97831f41a2555aa3f49e9--zel-stage.netlify.app/
- Hosted run 41587142130 passed; draft:
  https://6ae978a309307ab3058e7340--zel-stage.netlify.app/

After preview visibility changed, both passed scripts/check-delivery-http.mjs:
actual pages, exact logo bytes, linked styles/scripts/fonts and 404 responses
for source/configuration paths. In the normal in-app browser, a search for
zirconium opened /guide/next.html in the portable draft and /manual/next in the
hosted draft. Neither flow emitted a captured console error. These small test
fixtures are separate from the complete documentation at the staging root.
Before/after SHA-256 fingerprints for /, /cli and /guide/quickstart matched on
both the old production site and the staging main URL. This covers six public
responses, not whole-site deployment identity.

The full 55-route staged documentation matches the validated upload after
removing only Netlify's observed added hosting comment and HUD script; raw
responses are not all byte-identical. All 29 authored image hashes match.
The paced headless browser verified all 46 original routes and 282 heading IDs,
then received a Netlify connection-verification page (HTTP 403) on the first
added route. The earlier run stopped at /cli/serve with the same status. These
runs are incomplete, not clean full-browser passes. A normal in-app browser
opened all nine new pages, checked all 82 compatibility targets/scroll margins,
followed an old fragment on each of the 16 affected pages, and navigated search
to /cli/doctor#runtime-selection.

That browser also recorded repeated React #418 hydration errors on the full
documentation. Controlled local comparison of /cli/completions found no errors
for the original build, an error with byte-identical Netlify HTML, an error with
only the observed hosting comment at its original position, and no error with
only the badge script. Thus the injected comment is sufficient to reproduce
this failure; the isolated script is not. This is a hosting interaction, not
missing authored content, and has not yet been resolved. The user was asked to
turn off the staging Powered by Netlify badge so we can test whether its setting
also removes that comment. Do not claim the setting has changed or the error
is fixed until the actual remote response and browser have been checked.

Evidence: specs/evidence/zellij-netlify-acceptance-2026-11-02.json and ignored
backups/zellij-netlify-acceptance-2026-11-02. The latter retains raw logs,
fingerprints and diagnostic scripts. No authentication challenge was bypassed,
credentials read, local Docker started, production site replaced, domain moved
or stable release promoted. Original full Netlify deploy ZIP/settings and an
explicit production cutover decision remain outstanding. The workflow queue
fix still needs a later native candidate; published rc.1 assets are unchanged.

Refresh of earlier pending checks: PR #55 post-merge runs 41576036593 and
41576036573 were cancelled and must not be called passed. They were superseded
by main CI 41576279817 and Base image 41576279748 on PR #56 merge 3114219;
both completed successfully.
