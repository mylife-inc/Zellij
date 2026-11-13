# Optional book PDF pipeline — 2026-11-12

Base: `6f26ea7`. This increment extends PR #149's publication snapshot with a
bounded direct Markdown-event-to-LaTeX adapter and optional isolated execution.
No Pandoc Markdown reparse occurs. The official `pandoc/latex:3.11-ubuntu` image
supplies LuaLaTeX/fonts and is pinned to multi-platform index
`sha256:24cd38c7f0e9f7020cb3faaa5122eca719f5e72039f45d15b699bb3168055d44`.
Its amd64 and arm64 entries were verified from the registry before implementation.

Commands: `build --pdf`, `export pdf` / `pdf`, and `pdf --check`. Normal builds
remain native. Standalone output defaults to `pdf/`; combined builds use
`site/_zellij/pdf/`. Public export artifacts include the TeX, copied images,
publication metadata, log and PDF receipt. PDF and web share a content revision;
publication remains one guarded directory replacement. Plain rebuild removes
both the old PDF and its link. No configuration silently enables external tools.

The first print template is English/LTR A5 with Latin Modern fonts, parts,
chapters, appendices, title page, TOC, running headings, folios and bookmarks.
Paragraphs, lists, code, quotes, footnotes, tables, validated math and page/heading
links use existing parsed events. PNG/JPEG images are staged under generated
names. Unsupported media/language/layout cases are rejected rather than omitted.
This is not complete LaTeX parity or the complete book expansion milestone.

Runtime mounts only staged inputs read-only and disposable output read-write.
The root is read-only; network, capabilities, privilege escalation and shell
escape are disabled. Resources and calls are bounded; a named-container guard
handles cancellation and cleanup. Three passes must finish without unresolved
references, missing glyphs or overflow, and the PDF/receipt must be verified
before publication. A Docker/Podman/nerdctl Linux runtime remains an explicit
optional prerequisite. No local runtime or GUI/browser is used for acceptance.

References checked during implementation:
- https://github.com/pandoc/dockerfiles (official image source/packages)
- https://pandoc.org/MANUAL.html#creating-a-pdf (backend context)
- Pandoc's `--sandbox` documentation explicitly excludes PDF engine IO, so it
  is not used or described as the isolation boundary here.

Validation in progress: native compiler/CLI fixtures, real hosted LuaLaTeX on
Linux amd64 and arm64, extracted text/order, image content, font embedding,
bookmarks/links, complete PDF checks and selected page captures. Exact outcomes,
failed attempts and fixing commits will be retained after verification.

Initial local print test assumed an internal page index in its hyperlink
assertion. The compiler orders pages independently of publication order; the
test now checks that the link targets the correct chapter's actual heading anchor.
The first filtered CLI test invocation selected zero cases; the complete binary
suite subsequently passed 120 tests with five existing hosted-only ignores.

Remaining: configuration-driven PDF builds, PDF-aware preview/delivery,
multilingual/RTL fonts and layouts, bibliography/index/glossary, richer equation
and theorem references, custom/technical/study templates and new distribution
channels. Production documentation/domain, native rc.5 and live provider
verification remain unchanged.

First hosted attempt `43353600220` at `6444fb1` failed on both native PDF
architectures: the pinned upstream image includes memoir but lacks its required
`xpatch.sty`. The print template now uses the standard book class plus its own
small chapter treatment, without a larger image or runtime package installation.
Review also corrected synthetic title/footnote link targets, unreferenced
footnote handling, six-column width accounting and flat-source output placement.
A local compile caught an ambiguous Result type in that placement fix; it is
explicitly typed. These failed attempts remain part of the record.

Second hosted attempt `43354162015` at `c8dbd2c` generated valid PDFs on both
architectures, passing text/metadata/font/link/structure checks and uploading
page captures. Its intentional-overflow fixture used a repeated English word
that TeX could hyphenate, so the expected failure did not occur. The fixture now
uses an unbreakable monospace token; the overflow guard is unchanged. Native
cancellation and deadline regressions also verify cleanup and old-output retention.

Visual review also caught a section heading orphaned above its table. The classic
template now reserves space for a heading and following content; hosted acceptance
asserts that the table heading and first rows share a page and captures every page.

Final review after run `43355088979` found that passing the host UID directly
fails with rootless runtime mappings. Runtime metadata now identifies rootless
Docker/Podman/nerdctl within a bounded query; unknown responses fail before the
container starts. Explicit `userns=host` uses the runtime's outer namespace:
UID/GID 0 maps to the caller for rootless runtimes, while rootful Unix runtimes
retain the staged directory owner's IDs. No capabilities are restored. The
image name is fully qualified for noninteractive Podman pulls. Hosted acceptance
now also exercises actual rootless Podman; the previous two-architecture artifacts
remain preliminary until the new final head passes.

Mapping references:
- https://docs.podman.io/en/stable/markdown/podman-run.1.html#userns-mode
- https://docs.podman.io/en/latest/markdown/podman-info.1.html
- https://docs.docker.com/reference/api/engine/version-history/
- https://github.com/containerd/nerdctl/blob/main/pkg/cmd/system/info.go
