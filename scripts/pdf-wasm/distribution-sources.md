# Optional PDF toolchain — original sources and rebuild inputs

This is a private qualification archive. It is not a CLI installation package,
does not grant new licenses and does not declare release review complete.

SOURCE-MANIFEST.json inventories every packaged file by path, size and SHA-256.
It binds the runtime, format and font input locks and records their matched
original sources. The manifest excludes itself; the external verification receipt
records its hash and the archive hash. Original archives are copied byte for byte.

- upstream contains the complete pinned run, documentation and source archives
  for the union of the runtime, format and font inputs, the corresponding engine
  source archive, supporting notices and original provenance metadata.
- scripts/pdf-wasm contains the exact resource rebuild and verification programs,
  recipes and their tests. Python is a maintainer tool, never an installed zel
  dependency. The generated Zellij manuscript acceptance suite lives in the
  Zellij repository and is not part of this compiler/source archive.
- notices contains original component notices and the complete EC distribution.

To verify source correspondence without downloads, from this directory use:

    python3 scripts/pdf-wasm/sources.py verify upstream verification/runtime
    python3 scripts/pdf-wasm/format_sources.py prepare upstream verification/format-inputs
    python3 scripts/pdf-wasm/font_sources.py prepare upstream verification/font-inputs

Use new output paths; these commands preserve existing outputs. The selected
original bytes, package revisions, notices and frozen metadata must all agree.
The large provenance metadata is included so inventory regeneration does not
depend on a live service.

For the compiler itself, upstream/corresponding-source.tar.xz retains the original
SOURCE-MANIFEST.json, REBUILD.md, RELINK.md, complete component sources and original
build receipts. Follow those build/relink instructions in an isolated maintainer
workspace. The JS/Wasm output must match the reviewed engine entries in
scripts/pdf-wasm/lock.json; an unexplained mismatch is not repaired by repinning
hashes. Compiler build dependencies belong to maintainers, not end users.

The format.mjs and fonts.mjs programs rebuild the format and metadata using the
verified compiler and prepared original inputs. See scripts/pdf-wasm/README.md
for their invocations. They require two identical builds and exact reviewed
output hashes. compact.py verifies and packages the compiler, format and runtime
resources; it does not add the notices directory automatically. The eventual
runtime packager must include the matching notices and source download details.

This archive deliberately retains original license statements, including the
unresolved historical wording in url.sty. See notices/NOTICES.json for review
items. No engine or resource license is replaced by Zellij's project terms.
