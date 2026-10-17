# Zellij project context

Before working on Zellij, read [ZELLIJ_CONTEXT.md](ZELLIJ_CONTEXT.md). It maps the
architecture, content model, implementation status, repository state, and the
user's product intentions.

For the initial review's findings and verification evidence, read
[the 2026-10-16 audit](specs/zellij-audit-2026-10-16.md).

These documents are a dated baseline, not a replacement for inspecting current
code. Refresh branch, remote, release, and test status before relying on them.
Keep the context current when later work changes architecture or settles a
product decision. Record resolved findings with the fixing commit and relevant
verification; do not silently erase the audit history.

This project is the Zellij product website engine and its Rust `zel` CLI, not
the unrelated terminal multiplexer. The user does not want development to use
the old encrypted `Zellij-Sealed` repository. Consult the context before any
repository migration, visibility change, or publishing work.
