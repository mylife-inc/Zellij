# Standalone workflow guidance correction — 20 November 2026

The released rc.15 behavior was correct, but several guide paragraphs still
said rich sites required HTTP or that default `check`/`build` started containers.
The repository README also sent a newly scaffolded portable site to the legacy
sibling output path instead of `docs/site/index.html`.

The deployment, wrapper, quickstart and eject guides now distinguish the
cached rich renderer from explicit Next exports, container development and
provider delivery. Migration's optional post-conversion validation and hosted
provider builds still deliberately use containers; those instructions remain.
No implementation behavior or prerequisites are changed by this correction.

Three website starters had correct READMEs but obsolete inline guide prose.
`product-launch`, `software-platform` and `creative-studio` therefore receive
new template identities 0.1.3 with corrected instructions. Their design, assets,
minimum CLI and engine requirements are unchanged. Published 0.1.2 archives
remain immutable; corrected bundled/remote templates await the next CLI
candidate. Current public CLI and renderer assets stay rc.15.

Local verification passes strict/deep checking of all 59 documentation routes
and 33 examples, all six existing native catalog/creation tests, and two
independent reproducible packages of all nine templates. The six unchanged
archives match their published digests. Using the actual public rc.15 binary
with an empty system PATH, the README's portable commands produce the documented
path and all three corrected website starters validate and build offline; their
rendered guide HTML contains the corrected instructions. No private client data
is used. Hosted documentation acceptance and publication follow.

The user confirmed that no Windows computer is currently available. Keep full
private Windows handoff acceptance pending, separately from passing public
Windows fixtures. WinGet still awaits Microsoft catalog review after its passed
CLA check; the two historical browser timing findings remain unresolved.
See the [verification evidence](evidence/zellij-standalone-guidance-2026-11-20.json).
