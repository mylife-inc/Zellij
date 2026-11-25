# Zellij optional PDF compiler — notices and source

This package contains the isolated GPL-2.0-only PDF adapter and its reviewed
LuaHBTeX compiler/resources. Original component terms remain in force; the Rust
CLI and website engine are not relicensed by this package. See
ZELLIJ-PDF-ADAPTER-SCOPE.md, ZELLIJ-PDF-ADAPTER-LICENSE.txt and
ZELLIJ-ADAPTER-NOTICE.txt for the scoped grant and original attribution.

NOTICES.json maps each preserved notice to its exact original path and archive.
ENGINE-SELECTION.json maps the selected compiler libraries to the original
component registry and source notices. DISTRIBUTION-REVIEW.md explains the
selection, generated files and distribution arrangement; its JSON counterpart
binds the exact reviewed inputs. These records do not change upstream licenses.

The originals directory includes the complete unchanged EC distribution and the
complete selected URL 3.3 distribution. The hash-named licenses files preserve
all original bytes. The supporting ConTeXt mreadme.tex document is unchanged and
attributed to Hans Hagen and the ConTeXt Development Team under
[CC BY-SA 4.0](https://creativecommons.org/licenses/by-sa/4.0/).

The runtime manifest's source_archive field identifies the matching source
archive by filename, size and SHA-256. NOTICES.json contains its versioned
sourceDownload URL and development-repository sourceMirror. If a destination
is no longer offered, use the same release from which this runtime came; it
contains complete original package archives, compiler/Emscripten sources,
adapter sources and rebuild/relink instructions. Source is optional for normal
PDF generation and is not downloaded into the compiler cache automatically.

Preserve both runtime and matching source access when migrating repositories.
No encrypted source copy or written-offer-only arrangement replaces that access.
This reviewed package's existence alone does not assert that a release has
already been published or that its public downloads have passed acceptance.
