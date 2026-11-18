# CLI rc.11 preparation — 2026-11-18

Baseline `0d25d84`, after the five verified composer increments: video/container
layouts, layers/semantic nodes/stages, selectors/explorers, scroll stories and
controlled built-in reuse. Rust workspace and standalone image recipes already
select rc.11. Final implementation `2f69966` passes all 32 jobs in `44397663238`;
merge `0d25d84` preserves those runtime/guide/acceptance inputs and adds records.

## Scope and gates

Prepare versioned notes and retain the accepted runtime, schemas, templates,
dependencies and browser/PDF inputs. A fresh audit found that GitHub withdrew
GHSA-fc5q-39xc-vjpf today at 15:38:56 UTC. Both full and production npm audits now
report zero findings. Remove the temporary exception and require zero findings
at every severity, preserving the original disposition as dated history. Seven
regression tests cover the stricter gate, including reappearance of the former
advisory. This changes audit tooling, not a dependency or runtime.

Strengthen package acceptance to compare the installed CLI schema/hash and every
editable preset with the tagged source, without external tools. Public image
acceptance compares the complete exported authoring model and presets for all
four variants on both architectures. These checks catch a correctly versioned
package that accidentally contains stale authoring data.

Run source CI and normal local release preflight before the immutable rc.11 tag.
Verify published native packages/checksums, all four images on both architectures,
book/PDF and website artifacts. Compare public output with retained source output;
never substitute locally rebuilt artifacts. Publish matching Homebrew/Scoop
recipes and exercise real rc.10-to-rc.11 upgrades while rc.10 is still available.
Then update active public guides and saved consumers, publish/verify replacement
images and retire only the exact reviewed superseded Zellij distributions.

The nine current site/book template archive identities remain unchanged. Browser,
container and actual LaTeX execution stays on hosted CI. Production-domain
migration and live-provider tests remain deferred. Candidate policy continues to
build the documentation without deploying the production site automatically.

All 52 audit/release/helper tests pass after the fixture correction below. Fresh
npm installation and engine packaging pass. Local CLI/engine checks match the
complete schema, schema hash and all nine preset bytes. Normal preflight passes
817 Rust tests (five hosted-only cases ignored), 527 engine tests, workspace
types, schema drift, formatting and warnings-denied Clippy. All 32 jobs in
corrected full hosted run
[`44415698968`](https://github.com/mylife-inc/Zellij/actions/runs/44415698968)
pass on `f913d16`, first attempt. This includes all six native installations,
all managed, website, image and PDF checks, plus 526 showcase cases without
retries (140 conditional skips). Preservation retains 46 original routes,
13 added routes, 282 heading IDs, 82 compatibility mappings and 29 byte-identical
images without captured browser errors; all six probes pass.
See [the preparation evidence](evidence/zellij-rc11-preparation-2026-11-18.json).
The final records-only update preserves every tested runtime, dependency, active
guide, workflow and acceptance input. Publication, channel/docs migration and
retirement are pending.


## First preparation CI finding

Run `44415309056`, Engine job `135193923475`, failed in the mocked public-image
acceptance test. The new context command correctly required schema data; the
old fake Docker executable returned package metadata for every run. Correct the
fixture to distinguish the two commands and assert every variant's offline schema
invocation. A negative case returns a schema without presets and requires the
acceptance process to fail. This is a test-fixture correction, not a product or
published-image failure. Retain the initial result; corrected CI follows.


## Retained release baseline

Development-image run `44415370224` passes all 13 jobs on `0d25d84`. All four
public `main` manifests contain native Linux amd64 and arm64 images. The engine,
CLI, dependency, Docker, template and website/PDF driver inputs remain unchanged
through preparation source `f913d16`.

Source-run `44397663238` captures are retained for final public comparisons.
All 224 website captures and 420 PDF page captures have identical decoded RGBA
pixels to the reviewed rc.10 evidence, on matching architectures. These are
source baselines, not rc.11 public-artifact results. Real rc.11 downloads still
need separate acceptance. The isolated previous standalone install reports
`zel 0.2.0-rc.10`; Homebrew `aa82b4e` and Scoop `992b4e1` remain public until the
new generated recipes pass their publication gates. No rc.10 artifact is removed
at this preparation checkpoint.
