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

Seven audit regressions and 45 release/helper tests pass. Fresh npm installation
and engine packaging pass. Local CLI/engine checks match the complete schema,
schema hash and all nine preset bytes. Normal source preflight and hosted CI
are running; publication, channel/docs migration and retirement are pending.
