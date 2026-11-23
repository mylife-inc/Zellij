# Cached PDF source correspondence — 22 November 2026

Status: maintainer qualification, not an installed PDF backend or a release.
Public/installed `zel` remains **v0.2.0-rc.19**. The runtime probe inventory and
all existing publication/PDF behavior are unchanged by this increment.

## Original-source verification

The earlier feasibility record verified the mirror's resource hashes. This
increment independently compares those resources with the frozen TeX Live
package archives, using the original database's SHA-512 archive checksums.
The database itself matches the engine mirror's pinned SHA-256 provenance.

- All **193 package-backed resources** match the original archive bytes.
- **58 packages / 152 run, documentation and source archives** are locked;
  their combined compressed size is **111,674,484 bytes**.
- **Two generated metadata resources** have matching snapshot hashes but do
  not have the same package-file proof. Their source-input closure stays open.
- A separate **144,966,596-byte** ConTeXt documentation archive supplies the
  notice referenced by `supp-pdf.mkii`.
- The **154,968,364-byte** engine source archive matches the published hash;
  its release manifest matches the exact JS, Wasm and compressed format in the
  runtime lock. The original engine and format generation revisions are retained.

These source archives are maintainer inputs. They do not enlarge the proposed
runtime download, and no source or engine binary is committed or published here.
The checked-in inventory contains URLs, hashes, package/license metadata and
resource-to-source mappings only.

The reproducible verifier is [`scripts/pdf-wasm/sources.py`](../scripts/pdf-wasm/sources.py).
It collects the original notices, rejects missing/changed source or notice bytes,
checks archive checksums before reading members, and never extracts archive paths
or follows archive links. `verify` has no network operation. Existing verification
outputs and corrupt cached files remain intact. Downloads are staged, verified
and retried at most three times before failure.

Python is a **maintainer/CI prerequisite only**, not a proposed installed `zel`
PDF dependency. The installed backend will reuse the cached private Node runtime.

## Notice findings retained for packaging

Three packages had no notice paths in the mirror metadata. Original archives
supply EC's `copyrite.txt`/README, the `url.sty` header and the `supp-pdf.mkii`
header. The last references ConTeXt's manual, now retrieved from the matching
frozen distribution. The LPPL text is also collected from the LaTeX package.

EC's original notice requires its full distribution. The final artifact must
preserve that requirement rather than ship only the observed metrics subset.
The `url.sty` header says “version 2 or later”, while
[CTAN's package record](https://ctan.org/pkg/url) labels it LPPL and the
[LaTeX Project's license page](https://www.latex-project.org/lppl/) identifies
1.3c as the latest version. The verifier preserves the original wording and
records the discrepancy; it does not silently rewrite the upstream license.

Complete component/font redistribution review, format and generated-data source
closure, rebuild/relink qualification, optional runtime packaging, Rust cache
integration and installed/public-package acceptance remain open. Passing source
correspondence does not mark these items complete.

## Verification

The final local real-archive check passes all 193 resources, **73 TeX notices**
and **32 engine notices**. All **13 synthetic tests** pass. The negative suite covers checksum corruption, runtime byte
mismatch, missing/empty notices, unmapped/duplicate resources, wrong revisions,
missing checksums, archive links/duplicate members, cache links, metadata pins,
engine/format receipt mismatches and incomplete download recovery.
The hosted workflow runs the source verifier independently from the six-platform
runtime probe and uploads only its JSON receipt. Current hosted results will be
recorded after completion. Initial source job `137510810815` in run
`45119828519` passes the 13 tests but fails downloading mirror metadata with HTTP
403. A local read-only comparison reproduces 403 for Python's default user agent
and 200 for the explicitly identified `Zellij-PDF-source-verifier/1` client on
both metadata URLs. The downloader now identifies itself and names a failing
input. URLs, hashes and validation requirements are unchanged. The failed setup
run provides no hosted source acceptance.

A clean engine rebuild is being investigated separately from the source-inventory
check. No byte-reproducibility or complete-source/relink claim follows merely
from matching the archive manifest.

## Preview follow-up and main state

Main CI `45088661314` and base-image run `45088661271` at `d4b0569` pass.
The earlier `45085352094` WebKit failure remains unresolved. Its retained trace
places the `preview.json` access-control page error about 4 ms after a navigation
begins from `index.html#anchor` to `guide/extra.html`, before deliberate shutdown.
The current poller already aborts on `pagehide` and catches fetch rejections.
This observation narrows the scenario, but does not establish a product or
browser/test-harness cause. No assertion, runtime behavior or timeout is changed
in this increment, and the later passing run is not treated as its fix.
