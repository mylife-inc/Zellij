# Generated delivery acceptance

These sites exercise complete GitHub workflows from the current CLI generator,
using the **selected 0.2.0-rc.4 executable and exact engine image** at execution
time. This candidate includes the current generators and multi-site queue fix.
Run only after rc.4 publication succeeds. Previous rc.1/rc.2 verification
records remain evidence for those versions; retargeting does not rerun live delivery.

| Workflow | PR behavior | Manual run on main |
| --- | --- | --- |
| acceptance-portable.yml | Native validation and portable build | Same |
| acceptance-hosted.yml | Hosted validation and static export | Same |
| acceptance-static.yml | Build portable static image | Publish ghcr.io/mylife-inc/zellij/acceptance-static:0.2.0-rc.4 |
| acceptance-next.yml | Build portable Next.js image | Publish ghcr.io/mylife-inc/zellij/acceptance-next:0.2.0-rc.4 |
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
