# CLI completion audit — 26 November 2026

Audited source: `c583efbaa515b2454d683b845baf137dbfaedddc` (PR #263).
Its CLI tree is unchanged from published `v0.2.0-rc.24`, source `2c3b7da`.
The repository, installed native CLI, published release and latest workflow
status were refreshed before this review. No open product pull request remained.
The [machine inventory](evidence/zellij-cli-completion-audit-2026-11-26.json)
records help routes, option spellings, source identity and verification limits.

## Result and corrections

No missing command family or stub handler was found in the documented rc.24
surface. This is a bounded implementation/coverage review, not proof of zero
bugs or every possible argument, operating-system and provider combination.

The review found three reporting defects and one manifest cleanup:

1. The cache roadmap still described stock Next development as proposed and
   cached PDF as opt-in. All five selected migrations have shipped; PDF became
   the default in rc.21 and stock development shipped in rc.24. The roadmap is
   reconciled; arbitrary custom Next dependencies remain outside that bundle.
2. Release/readiness/context records still described the final guides as
   pre-merge. PR #263 and automatic production deployment have completed. Their
   receipts are appended to the release evidence; prior qualification and
   failure evidence is retained. The old completion table is explicitly labeled
   historical and points here for current status.
3. The `--json` help description omitted `templates`, although parsing,
   execution and the public template guide already support it. The source help
   now lists it. This wording correction does not change execution and is not
   retroactively part of immutable rc.24 binaries; it will accompany the next
   candidate. It alone does not require a release.

4. Cargo reported unused workspace declarations for `anyhow` and `miette`.
   Neither is referenced by any member manifest or Rust source. Removing those
   declarations changes no member dependency or lockfile and clears the warnings.

Source corrections: `61bcad2`; parser tests, rebuilt help, formatting and
Clippy verify the correction without altering the dependency lockfile.

## Method and command coverage

The installed rc.24 CLI successfully renders all **82 help paths**, comprising
75 leaf paths and seven parent/root pages. These counts include flattened
shortcuts and provider aliases; they are not counts of distinct capabilities.
All **81 distinct long-option spellings** occur in public MDX documentation.
This textual check detects missing option names, not incorrect explanations or
incomplete behavior coverage. The source help correction adds no option.

`src/args.rs` supplies parsing, help and completion from one Clap schema.
`src/main.rs` has exhaustive dispatch, with aliases normalized before handlers;
no catch-all stub accepts a new command silently. Shortcut tests compare minimal
and explicit arguments, selectors, invalid arguments, defaults and completions,
and assert that every registered alias has a test. No `todo!` or `unimplemented!`
macro was found in the Rust workspace. Remaining textual TODOs are authored
scaffold placeholders; importer messages disclose unsupported source constructs.

Paths in this table are relative to `cli/crates/zellij-cli/` unless prefixed
with `scripts/` or another crate. Each row records existing evidence, rather than
claiming every test was rerun on every platform during this audit.

| Surface | Handler / implementation | Relevant verification |
| --- | --- | --- |
| `new`, `create`, `init`, authoring shortcuts, `templates [show]` | `templates/`, `site_authoring.rs`, `authoring.rs`, `page_authoring.rs`, `app.rs`, core/scaffold crates | `tests/site_templates.rs`, `site_authoring.rs`, `section_authoring.rs`, `page_authoring.rs`, `full_starter.rs`, `app_scaffolding.rs`; public template/source acceptance |
| `list`, `explain`, `context`, `detect` and their shortcuts | `main.rs`, embedded core schema, converter registry | `tests/inspection_output.rs`, `aliases.rs`; converter unit tests; schema and authored-section checks |
| `convert`, `migrate`, `ship` | `imports.rs`, `delivery.rs`, `zellij-convert` | `tests/conversion_references.rs`, six format-specific import suites, metadata/navigation/URL/asset fixtures, `import_delivery.rs`; real-engine conversion acceptance |
| `build`, `b`, `bs`, `bp` | `main.rs`, `rich_build.rs`, `rich_build/cache.rs`, `pdf.rs`, portable docs and engine crates | `tests/portable_build.rs`, `book_publication.rs`, `hosted_check.rs`, `safe_writes.rs`; six-platform public/native and rich file/HTTP acceptance |
| `check`, `lint`, `watch`, `fmt` | `validation.rs`, `watch.rs`, `formatting.rs` | `tests/portable_check.rs`, `hosted_check.rs`, `formatting.rs`; `scripts/check-validation-watch.mjs`, cached-engine checks |
| `dev`, `serve`, `view`, `vw` | `development.rs`, `development/cached.rs`, `development/process.rs`, `preview.rs`, `preview/rich.rs` | Portable docs server/watch tests; `scripts/check-cached-development.mjs`, `check-dev-options.mjs`, `check-preview.mjs`; native six-target and Linux/Windows browser acceptance |
| `export pdf`, `pdf` | `pdf.rs`, `pdf_package.rs`, shared book model, isolated `scripts/pdf-wasm/` adapter | `tests/pdf_export.rs`, `book_publication.rs`; source, six-platform public cached-PDF and both Next-image PDF architecture checks |
| `image static/next/build/run/push/dockerfile` and shortcuts | `images.rs`, `main.rs`, `zellij-engine`, `published_image.rs` | `tests/static_images.rs`, `runtime_requirements.rs`, `delivery.rs`; image delivery and architecture acceptance; Dockerfile generation needs no daemon |
| `deploy`, provider shortcuts, `publish` | `delivery.rs`, seven provider modules | `tests/delivery.rs`, `netlify.rs`; provider protocol/unit tests and explicit hosted-cache fixtures; live-provider evidence remains separately scoped |
| `ci [init/run]` | `ci.rs`, `zellij-scaffold/src/pipeline.rs`, `pipeline/external.rs`, `main.rs` | `tests/ci_workflows.rs`, generated-workflow and installed delivery checks; actual GitLab/CircleCI runners deferred |
| `eject`, `features` | `eject.rs`, `source_package.rs`, `features.rs` | `tests/features.rs`, `app_scaffolding.rs`; installed editable-source, ejection and wrapper acceptance |
| `doctor`, `version`, `pull`, `upgrade`, `completions` | `doctor.rs`, `maintenance.rs`, `maintenance/upgrade/`, `args.rs` | `tests/runtime_requirements.rs`, `inspection_output.rs`, upgrade unit tests, shortcut/completion tests; native installation and eight actual channel upgrades |
| Global selectors/output controls and aliases | `args.rs`, `args_tests.rs`, `zellij-core` site/reporting modules | `tests/site_layout.rs`, `output_controls.rs`, `user_home.rs`, `aliases.rs`; parser coverage for conflict rejection before side effects |

The significant option contracts are also separated by existing suites: output
ownership and overlap, offline/cache integrity, global selection and precedence,
PDF opt-in/default backend, image kinds/platforms, formatting/check exits,
provider dry runs/credentials/limits, import asset strategies, template variables,
and install ownership. This audit found no new behavioral defect in those
contracts. Executable foreign configuration, arbitrary import plugins and custom
application dependencies are deliberate boundaries, not silent fallback paths.

## Verification and publication

- Fresh `cargo test --workspace --locked`: **863 passed, zero failed, five
  ignored**. The five hosted-cache provider cases are named explicitly in
  `.github/workflows/ci.yml` and run there with `--ignored --exact`; they are not
  silently dropped from release qualification.
- The help/manifest corrections pass all 13 focused parser tests, rebuilt help,
  formatting and warnings-denied Clippy (zero warnings). The lockfile is unchanged.
- Existing rc.24 release run `45791545516`: 55 passing jobs, one conditional
  legacy WinGet skip. Both repositories have 58 matching assets. Six public
  native checks, six public cached-PDF checks, both Next-image PDF architectures,
  independent Mac checks and eight real Homebrew/Scoop upgrades pass.
- Final-guide PR run `45802055700`: 37 passes and one conditional skip.
  Production run `45806880663` publishes source `c583efb`; ten live guides pass
  HTTP/content/hydration checks with no captured browser errors. Preservation
  retains 46 original plus 15 added routes, 282 original heading IDs,
  82 compatibility mappings and 29 original images. The duplicate main CI was
  intentionally cancelled after verifying qualified-tree equivalence; it is
  not counted as a passing run.

These publication results were completed before this audit and are not rerun
claims. Detailed receipts remain in the [rc.24 record](zellij-rc24-release-2026-11-26.md)
and [release evidence](evidence/zellij-rc24-release-2026-11-25.json).

## Remaining work, in order

1. **Stabilization from fresh evidence.** The two historical browser timing
   findings remain open: WebKit navigation after resize and documentation
   hydration readiness. The latter recurred in run `45007103274`; diagnostic
   fix `bfb450b` improves evidence without claiming the root cause. Latest
   qualification reports no new recurrence. Do not spend time repeating
   already passing stress tests without a new artifact or a controlled
   hypothesis. The isolated PDF Wasm mitigation is retained; an upstream
   compiler/runtime root-cause fix has not been established.
2. **Retention inventory.** Older preview assets can be reviewed for retirement
   separately, preserving actual upgrade/catalog inputs, immutable template
   identities, failure history and the PDF runtime's corresponding sources and
   notices. This audit performs no deletions; the next inventory must identify
   the exact obsolete artifacts and their remaining consumers.
   Follow-up: the [scoped rc.16–rc.18 retirement](zellij-rc16-rc18-retirement-2026-11-26.md)
   is now complete with retained-state and public-download verification.
3. **Deferred external acceptance.** Fly, S3, actual GitLab/CircleCI runners and
   Microsoft catalog approval remain separate from implemented commands. Vercel
   is explicitly user-deferred after payment was required. Do not restart these
   before the requested end-of-work verification stage.

Optional expansions are not completion blockers for the documented contract:
remote/native OCI builders; arbitrary editable-app dependency management; full
foreign generator/plugin parity; unrestricted composer components/animation;
BibTeX/custom citation styles and unrestricted fonts. The Kie theme remains an
inspected interpretation, not an established pixel-identical copy.

No additional CLI command family needs implementation on the evidence reviewed.
Candidate rc.24 remains the published candidate. A stable 1.0 decision needs an
explicit acceptance/release decision and cannot be justified as “zero bugs.”
