# Picomatch development advisory — 2026-11-17

## Closure — 2026-11-18

GitHub withdrew [GHSA-fc5q-39xc-vjpf](https://github.com/advisories/GHSA-fc5q-39xc-vjpf)
on November 18 at 15:38:56 UTC. The upstream maintainer rejects the vulnerability
classification in [the linked discussion](https://github.com/micromatch/picomatch/pull/235).
Fresh production and full repository npm audits both report zero findings.
No dependency version or runtime behavior changed; withdrawal is not a code fix.
The temporary exception is removed. The audit gate now rejects every finding,
including this advisory if a registry reports it again, and still rejects command
failures, malformed reports and inconsistent counts. Seven regression tests cover
the strict contract. The original diagnosis and temporary disposition below
remain historical evidence, not an active exception.

## Original finding and disposition

Documentation CI `44163958595`, Engine job `134324898500`, failed at the
unchanged `npm audit --audit-level=low` gate. Preserve that failure: the
[upstream advisory](https://github.com/advisories/GHSA-fc5q-39xc-vjpf) was reviewed
and updated on November 17 and identifies a ReDoS involving attacker-supplied
repeated extglob patterns. npm currently publishes 4.0.7 as latest; the advisory
includes all versions through 4.0.7 and lists no patched version. The installed
4.0.5 is affected. This is **not fixed** by this disposition.

## Reachability and release scope

The seven findings are one root advisory propagated through picomatch, fdir,
tinyglobby, sucrase, tsup, Vite and Vitest. Every reported installation is marked
`dev: true` in the lockfile. `npm audit --omit=dev --json` reports zero findings. A fresh native-generated
wrapper with all optional features also has zero findings across its full
403-dependency inventory (including its own development dependencies).
Zellij's runtime/content imports do not use these packages. tsup's patterns are
literal repository-controlled source paths in `tsup.config.ts`; Vitest scans
the repository's `src` test directory. Neither is exposed as a site endpoint
accepting a visitor's glob. Untrusted repository configuration is already code
execution in these developer tools and must not be run in a privileged context.

The native Rust CLI does not ship these Node tools. The base-image build stage
uses them, then discards its `node_modules`; the public engine and generated
wrapper do not install engine-owned development dependencies. The packaged
engine includes its manifest but does not invoke tsup/Vitest at runtime.
Static exports contain no Node server. npm's inventory is not an audit of every
dependency bundled inside upstream frameworks; for example Next includes its
own compiled matcher. The production result is a scanner result, not a claim
that all runtime code is vulnerability-free.

## Temporary disposition

Until **2026-12-01 00:00 UTC**, `scripts/check-dependency-audit.mjs` permits only
this exact advisory/range and its reviewed seven-package development chain at
the current versions and locations. It still prints both complete npm audit
reports and an explicit warning. It fails for any production finding, another
advisory, dependency promotion/version/location change, malformed report,
registry/command failure or expiry. Tests exercise these rejection paths.
No severity threshold is raised and no package wildcard is excluded.

This bounded deferral is justified by the absence of an upstream fix and the
reviewed developer-only input paths. It preserves rc.10's immutable native and
image artifacts; it makes no dependency, product runtime or template change.
Do not label the full audit clean or this advisory resolved. Recheck upstream
before December 1, install a verified fix when available, run the normal engine
checks and remove this disposition. If the exposure changes before then, fail
the gate and re-review rather than extending the exception automatically.
