# Optional PDF component and source review

This review covers the exact LuaHBTeX runtime, TeX inputs, generated format/font
metadata and isolated PDF adapter recorded in distribution-review.json. It is a
maintainer distribution decision for these inputs, not an upstream endorsement,
a change to third-party licenses or a grant over the rest of Zellij.

The release packager compares the entire reviewed input inventory, rather than
trusting a completed-review boolean. Changed compiler, component, source, notice,
recipe or adapter-license inputs require a new review. Packaging also verifies
the current adapter sources against the matching source archive.

## Compiler and adapter

The original ENGINE-COMPONENTS.json selects GPL-2.0-only for LuaHBTeX with its
Xpdf 4.06 backend. The 28 archives observed in the completed relink are each
mapped to exactly one component of that original registry. The review retains
the registry, original component notices, source notices and the selection map.
The registry's upstream link-inventory reference is not used as evidence: its
named a97db8a inventory is absent from the archive. Zellij supplies the observed
archive selection from its separately qualified build instead, without editing
the original registry or its development-status wording.

The selected components retain GPL-2.0-only/GPL-2.0-or-later, LGPL-2.0-or-later,
LGPL-2.1-or-later, MIT/NCSA, Apache-2.0 with LLVM exception, public-domain,
Libpng, Zlib, HarfBuzz and embedded-library notices as recorded by the originals.
The alternative LGPL terms are selected for Graphite2 and zziplib. Complete
source is retained for the compiler, Emscripten libraries and embedded components.
Only the LuaHBTeX family is selected; unrelated XeTeX/TECkit/FreeType components
are not inferred to be linked merely because their notices share the archive.

PR #244 reproduced the original compiler bytes. PR #253 rebuilds the Wasm phase
with compatible changes to kpathsea, Graphite2 and zziplib; all three modified
APIs execute on three successful typesetting passes. relink-evidence.json and
relink-proof.patch retain that evidence and the exact modifications. The source
instructions identify the correct engine-generation snapshot. The modified test
binaries are not part of the release. Recipients may modify and relink the
supplied sources, including debugging changes to the LGPL-covered portions.

Shebka's original scripts/pdf-wasm adapter is GPL-2.0-only under the owner's
24 November 2026 approval. LICENSE-SCOPE.md defines that grant; original MIT
attribution is preserved. The Rust CLI and website engine retain their terms.

## TeX packages, fonts and generated files

Every selected original file is checksum matched to a frozen package archive.
All run, documentation and source archives selected for each package are supplied
unchanged in the separate matching source download. Original catalogue identifiers
are provenance metadata; original file notices govern the respective files.

- LPPL packages: preserve their notices, source/documentation archives and work
  manifests. Runtime selection and hash-based storage are documented here; file
  contents and the virtual filenames presented to the interpreter stay original.
  The complete original works are available from the same release as the runtime.
- Latin Modern text/math fonts: preserve the GUST Font License, copyright notices,
  manifests and original archives. Font programs are unchanged. The generated
  database indexes only those original fonts and has its own reproducible recipe.
- Computer Modern metrics and Knuth support: original files and names are
  unchanged, with the complete archives and original notices retained. The
  English hyphen.tex notice explicitly permits unchanged redistribution; its
  original file is collected as a notice as well as retained in the source.
- EC: distribute every original 00files.txt entry, including the manifest,
  copyright notice and sources, inside both runtime notices and source package.
  The packager verifies filenames and sizes; this is not just a licence-text copy.
- URL: use unchanged original URL 3.3 with its unversioned LPPL notice and all
  four original distribution files. The later ambiguous URL 3.4 is not selected.
- Luaotfload and Lualibs: preserve GPLv2 texts, copyright headers and complete
  original archives, including their ConTeXt-derived source.
- Hyphenation: preserve each selected file's actual terms. Spanish pattern data
  has an MIT/X11 notice, while its older documentation README identifies LPPL;
  both are retained without relabeling one as the other's license. French and
  Arabic notices, German MIT-data/LPPL-documentation notices and loader permissions
  remain intact. Public-domain language configuration is unchanged.
- Unicode: preserve the Unicode copyright/permission notice from the original
  unicode-data README alongside its LPPL wrapper notice and complete sources.
- Other MIT, CC0/public-domain and per-file permissions: retain the original
  headers, documentation and source archives. No source file is relicensed by
  this review.
- supp-pdf.mkii: preserve its original copyright/header and referenced ConTeXt
  licensing documentation; use its GPL option. The supporting mreadme.tex document
  itself remains unmodified under [CC BY-SA 4.0](https://creativecommons.org/licenses/by-sa/4.0/),
  attributed to Hans Hagen and the ConTeXt Development Team in its original header.

The generated Zellij format is named zellij-lualatex.fmt, not an upstream format
replacement. Its input list, original sources, generation procedure and exact
output hash are retained. Font metadata and the OpenType-only map similarly have
explicit generation/selection records. These generated files are distinguished
from the unchanged original packages and do not imply upstream maintenance.

## Publication and retention

Publish the matching runtime and source archive together, each with its checksum,
on the same release. The runtime records the exact source archive's name, size
and SHA-256; notices give its versioned download location. No written-offer-only
arrangement or encrypted-source substitute is used. The source download is
optional for ordinary users and is not placed in the installed compiler cache.

Keep that source available while either repository distributes the runtime,
including repository migrations or sealing. Preserve existing recipients' granted
permissions. Retiring releases must account for both repositories and must not
leave an offered runtime without its source. The paired-package build and later
public-download acceptance are separate gates.

Relevant original terms include [GPLv2 section 3](https://www.gnu.org/licenses/old-licenses/gpl-2.0.html),
[LGPLv2.1 section 6](https://www.gnu.org/licenses/old-licenses/lgpl-2.1.html) and
[LPPL 1.3c](https://www.latex-project.org/lppl/lppl-1-3c/). The exact versions and
component-specific notices are preserved in the package rather than replaced by
these links or this review summary.
