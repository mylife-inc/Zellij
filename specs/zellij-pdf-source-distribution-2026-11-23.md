# PDF source and notice packaging — 23 November 2026

Status: private maintainer qualification. No PDF runtime or source archive is
published by this change. Installed/public rc.19 still uses its container PDF
backend. Rich website generation already uses the downloaded renderer.

## Verified baseline

PR #246 merged as `7f25082`, identical in tree to tested `1364bb3`. All 37 normal
and seven PDF jobs pass (`45155144409`, `45155144329`), followed by main CI
`45159553137` and base-image run `45159552912`. All six targets produce identical
10,845,108-byte resource archives and pass 114 manuscripts / 342 passes. Linux
matches all 269 native page rasters. Additional inspection of those hosted PDFs
matches 197 embedded font entries with Unicode maps, 1,224 named destinations and
eight URL annotations; this inspection is recorded in PR #246.

## Concrete source and notice layout

`distribution.py` regenerates the runtime, format and font inventories from the
frozen metadata and compares them with their checked-in locks. The union has 65
packages, 351 distinct original resources and 173 cached inputs: 170 original
archives plus three metadata inputs. Conflicting revisions, archives, resources,
cache filenames or incomplete notices fail before publishing an output.

The notice package maps 114 original notice records to unchanged bytes and
retains their original paths and source archives. Identical text may share a
hash-named file; original licence metadata is not rewritten into a new grant.
The adapter's existing upstream MIT notice is included separately.

The complete EC distribution is verified against the original `00files.txt`.
Every one of its 121 named files must occur exactly once, with the listed size,
as a regular member of the checksum-pinned originals. All are copied with their
original names and bytes under `originals/ec`, including documentation, source,
manifest and copyright notice. This implements the complete-originals packaging
requirement; the eventual runtime must still include that package.

The matching source archive contains all original run/doc/source archives,
compiler corresponding sources, frozen provenance metadata, exact rebuild
programs/recipes and the notice package. Its README points to the preserved
compiler rebuild/relink instructions. A complete source manifest hashes every
file; an external receipt hashes that manifest and both archives. Source and
notice archives are separate so ordinary runtime downloads need not carry the
large compiler/source collection.

## Preservation and verification

Source bytes are revalidated when staged, after correspondence checking. Existing
outputs are refused. Failure cleans only the tool's newly created staging
directory. The archives have fixed member order, modes, owners and timestamps;
each is reread to check member type, identity and complete content hashes.

Nine new synthetic regressions cover conflicting provenance, unsafe/colliding
cache names, complete EC originals, malformed manifests, missing/duplicate/linked
members, reproducibility, byte preservation, altered inputs, pre-existing and
late-created outputs, and modified/duplicate/linked archive content. Together
with the existing source tests, 33 Python tests pass locally. The full original
source preparation and packaged offline replay also pass, as do the 19 real-engine
Node tests with the new source-lock receipts. Final archive and hosted receipts are recorded
in the implementation pull request.

CI prepares the package without uploading either archive, then reruns runtime
verification and format/font input preparation from its copied programs and
upstream files. Only a JSON distribution receipt is retained. The six-platform
PDF suite continues to verify the rebuilt format and font metadata against the
new source-lock identifiers. Input byte inventories and reviewed generated
format/font hashes are unchanged; source locks change only to record resolved
historical findings and their policy hash.

The first hosted attempt at `81be418` passes source packaging and replay, but
the two Linux probe jobs in `45214762953` fail before tests during pinned-input
downloads with `ECONNRESET`. Their logs are retained; they are not PDF failures.
The maintainer downloader now uses at most three attempts for transient transport
failures/deadlines and selected temporary HTTP responses. Partial buffers are
discarded; integrity/size failures, permanent HTTP/TLS errors and existing outputs
still fail immediately. Eight regressions include real loopback resets before
headers and mid-body, a stalled body, exhausted retries and preservation of output.
Final-head results belong to the implementation PR; the failed attempt is not
replaced by an earlier successful run.

## Remaining work

The source policy preserves completed engine/format/font findings as resolved
records with evidence. The original `url.sty` license-version wording remains
unresolved; no author clarification or replacement permission is claimed. Final
component redistribution review and real matching-source delivery remain before
publishing the optional toolchain.

Production isolation and cancellation, Rust cache/download/dispatch, diagnostics,
generated CI/user documentation and installed/public acceptance are separate
remaining tasks. No new CLI candidate, provider verification, licensing change
or third-party message is included here.
