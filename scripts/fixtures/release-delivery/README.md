# Generated delivery acceptance

These sites exercise complete GitHub workflows from the current CLI generator,
using the **selected 0.2.0-rc.19 executable and exact engine image** at execution
time. This candidate includes the current generators and multi-site queue fix.
Run only after the selected release publication succeeds. Previous rc.1/rc.2 verification
records remain evidence for those versions; retargeting does not rerun live delivery.

| Workflow | PR behavior | Manual run on main |
| --- | --- | --- |
| acceptance-portable.yml | Native validation and portable build | Same |
| acceptance-hosted.yml | Hosted validation and static export | Same |
| acceptance-static.yml | Build portable static image | Publish ghcr.io/mylife-inc/zellij/acceptance-static:0.2.0-rc.19 |
| acceptance-next.yml | Build portable Next.js image | Publish ghcr.io/mylife-inc/zellij/acceptance-next:0.2.0-rc.19 |
| acceptance-netlify-portable.yml | Native validation and portable build | Create a Netlify draft preview |
| acceptance-netlify-hosted.yml | Hosted validation and static export | Create a Netlify draft preview |

Image destinations are dedicated acceptance packages. Run
`acceptance-inspect-images.yml` after both image publications; fresh runners pull
the registry copies and check pages, assets, the container health check and
source-file isolation. The generated workflows do not enable push/schedule/tag
publication. The Netlify workflows use the repository's NETLIFY_AUTH_TOKEN and
NETLIFY_SITE_ID settings, and never use `--prod`. Select the intended test site
before manually dispatching either Netlify workflow. PR verification receives
no publication credentials. Never put tokens in these configurations.

Regenerate after changing saved settings or the generator:

```sh
cargo build --manifest-path cli/Cargo.toml
node scripts/prepare-delivery-acceptance.mjs cli/target/debug/zel --write
```

Omit `--write` to check the committed workflows. The script runs the real CLI
in a temporary Git repository, compares exact generated YAML, checks unique
queues, and verifies manual-main-only publication and draft-only Netlify mode.
It also regenerates each image inspector target and its configuration hash. The
comparison checks that the target agrees with both the saved settings and the
generated publication step. Inspection rejects edited settings until metadata
is regenerated; it needs no additional npm dependencies.
It does not build images or deploy. CI runs this comparison. Generated YAML
should be changed through its configuration and generator, not hand-edited.

After a selected Netlify draft is ready, use its printed unique preview URL:

```sh
node scripts/check-delivery-http.mjs https://DEPLOY-ID--SITE.netlify.app portable
node scripts/check-delivery-http.mjs https://DEPLOY-ID--SITE.netlify.app hosted
```

Record workflow IDs, source/generator commits, released version, image digests,
preview URLs and verification outcomes in the project audit. A successful local
generator check is not evidence that GitHub execution or external delivery passed.

Previous rc.15 checkpoint (2026-11-20): all six PR verification workflows pass.
Static and Next image publication runs `44730196617` and `44730200827` pass;
fresh-pull HTTP/health/user checks pass in `44730534861`. No provider deployment
was performed in this checkpoint. See
[the release evidence](../../../specs/evidence/zellij-rc15-release-2026-11-20.json).

Historical rc.16 checkpoint (2026-11-21): all six generated PR verification
workflows pass on `e9ce97a`, merged with an identical tree as `2e8be27`.
Static and Next publication runs `44883674082` and `44883676471` pass; fresh-pull
HTTP/health/nonroot checks pass in `44883840228`. No provider deployment was
performed by these fixture workflows. The separate docs workflow publishes the
authorized production site. See [the rc.16 release evidence](../../../specs/evidence/zellij-rc16-release-2026-11-21.json).

Historical rc.17 checkpoint (2026-11-21): all six generated PR verification
workflows pass on `127fb13`, merged with receipt-only additions as `01f90e9`.
Static and Next publication runs `44951056149` and `44951058940` pass;
fresh-pull HTTP/health/nonroot checks pass in `44951268820`. Provider deployment
is not part of this checkpoint. See [the release evidence](../../../specs/evidence/zellij-rc17-release-2026-11-21.json).

Current rc.19 checkpoint (2026-11-22): all six generated PR verification
workflows pass on `9bfbf1c`, merged with an identical tree as `9e7c58e`.
Static and Next publication runs `45010926037` and `45010928505` pass; fresh-pull
HTTP/health/nonroot checks pass in `45011278526`. No live-provider workflow is
run in this checkpoint. See [the rc.19 release evidence](../../../specs/evidence/zellij-rc19-release-2026-11-22.json).

Correction — 26 November: run `45011278526` actually pulled rc.18, because the
inspection metadata had not followed the rc.19 publication pin. Its logs remain
evidence for rc.18. Fix `45eb4fa` binds inspection metadata to the source
configuration and generation checks. Fresh run `45921906069` now verifies both
actual rc.19 images. See the [retention review](../../../specs/zellij-retention-review-2026-11-26.md).
