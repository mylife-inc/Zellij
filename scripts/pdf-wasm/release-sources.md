# Zellij optional PDF compiler — matching source

This is the matching source archive for the version recorded in
SOURCE-MANIFEST.json. It contains complete original upstream archives under
upstream/, the exact adapter/build sources under scripts/pdf-wasm/, and the
runtime's original notices under notices/. SOURCE-MANIFEST.json hashes every
file and binds the exact distribution review. Third-party permissions and the
adapter's scoped GPL-2.0-only grant remain separate from Zellij's other code.

Verify original source correspondence offline, using new output directories:

    python3 scripts/pdf-wasm/sources.py verify upstream verification/runtime
    python3 scripts/pdf-wasm/format_sources.py prepare upstream verification/format-inputs
    python3 scripts/pdf-wasm/font_sources.py prepare upstream verification/font-inputs

The standalone source verifiers do not themselves approve distribution or run a
C compiler; their qualification markers do not override the enclosing reviewed
SOURCE-MANIFEST.json. See scripts/pdf-wasm/README.md for format/font rebuilds.

## Compiler rebuilding and modifications

upstream/corresponding-source.tar.xz includes the full compiler, TeX Live,
Emscripten and port sources, original build receipts, REBUILD.md and RELINK.md.
For the selected LuaHBTeX JS/Wasm, use engine-generation snapshot
**a97db8a62ed86b118bcfcc8784b7752aa8e988ac**, as the engine receipt records.
The generic upstream RELINK.md starts from another snapshot; use the selected
engine generation for this compiler. Preserve the original upstream documents.

The verified Emscripten SDK is 3.1.46, image digest
sha256:2491bc4bf6caf8c41993660822341bc72759cb577363dfe0781f0a2d05f7d357.
Use the original wasm-build/Dockerfile.luatex and build-luatex.sh in that snapshot.
Replace the Dockerfile's additional Git fetch with a named build context holding
the supplied source/texlive tree (`COPY --from=texlive / /src/texlive-source`).
The native phase generates code/tools; the Wasm phase rebuilds target libraries
and performs the final link. These are maintainer build tools, not installed-CLI
prerequisites. Original JS/Wasm hashes must match scripts/pdf-wasm/lock.json.

To exercise compatible library changes, apply scripts/pdf-wasm/relink-proof.patch
at the extracted source archive root with `patch -p1`. It changes kpathsea,
Graphite2 and zziplib and adds a small C export that calls all three modified APIs.
The expected compiler identities and successful local execution are recorded in
relink-evidence.json. Verify the modified compiler using the packaged script:

    node scripts/pdf-wasm/relink.mjs ORIGINAL_COMPACT_RESOURCES MODIFIED_COMPILER NEW_OUTPUT

It checks the exact modified compiler, unchanged import mapping, all three
executed library markers, three typesetting passes and the resulting PDF.
The proof binaries are not shipped. Any intentional source
change requires rebuilding and reviewing its resulting compiler/import mappings;
do not repair an unexplained mismatch by silently replacing production pins.

Recipients may modify and relink these supplied sources and debug changes to the
LGPL-covered portions. Preserve the applicable notices and modified sources
with any redistributed modified compiler. These permissions do not depend on
continued availability of the rest of Zellij's repository.
