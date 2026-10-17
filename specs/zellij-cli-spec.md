# `zel` — the Zellij CLI

**Specification**

A single native binary through which a person does everything with Zellij:
scaffold a content folder, convert one from another tool, validate it, build it,
serve it, and ship it.

Status: **original design draft**. Consult [the project context](../ZELLIJ_CONTEXT.md)
for current implementation status; this command inventory is not proof that a
command is implemented.

The user's 2026-10-17 requirements add plain-Markdown-folder builds with direct
`file://` viewing, serving an existing output, and a combined build/serve/browser
`view` workflow. See [the portable documentation contract](zel-portable-docs.md)
for acceptance criteria and the MkDocs capability comparison.

---

## 1. What it is for

Zellij's engine is a Next.js package. Today, using it means having Node, a
clone of a private repository, and enough familiarity with npm to know why
`install-links` matters. That is a reasonable ask of the people who build the
engine and an unreasonable one of the people who write documentation.

The CLI closes that gap. Someone with the binary and Docker can take a folder of
Markdown and end up with a running site, without installing Node, without an
account on a private repository, and without learning what a wrapper app is.

### Principles

1. **No Node on the host.** The binary depends on nothing but itself and, for
   the commands that build, a container runtime.
2. **The engine validates, not the CLI.** `zel check` runs the real schemas
   inside the container. Reimplementing them in Rust would produce a second
   source of truth that disagrees with the first the week after it is written.
3. **Never write over someone's work.** Conversion and scaffolding emit into a
   new folder and refuse to overwrite without `--force`.
4. **Say what could not be done.** A converter that silently drops a page it did
   not understand is worse than one that fails.
5. **Every command works offline once the image is pulled.**

### The Node question, settled

"Self-sufficient" means *no Node installed on the user's machine*. It does not
mean reimplementing the engine.

| Runs where | What |
| --- | --- |
| Rust, natively | Scaffolding, conversion, config, project discovery, deployment orchestration |
| Container | `check`, `build`, `dev` — the engine and its schemas |
| Embedded in the binary | The agent context file, generated from the schemas at release time |

The context file is the interesting case: it is *generated* by Node in CI,
committed, and compiled into the binary with `include_str!`. So `zel context`
needs neither Node nor Docker, and still cannot drift.

---

## 2. Distribution

| Artefact | Where | Who can get it |
| --- | --- | --- |
| `zel` binary | GitHub Releases, per platform | Anyone with the release URL |
| `ghcr.io/mylife-inc/zellij-base` | GHCR, public | Anyone |
| Engine source | Private repository | The team |

Platforms: `x86_64-unknown-linux-gnu`, `aarch64-unknown-linux-gnu`,
`x86_64-apple-darwin`, `aarch64-apple-darwin`. Windows is served by WSL, where
`zel` is a Linux binary — a native build would put the CLI on the far side of
the filesystem boundary from every container mount it works through.

Install: a shell script that picks the right asset, plus Homebrew later.
Checksums published alongside; the installer verifies before moving into place.

---

## 3. What it can do

    zel <command> [arguments] [--flags]

Planned global flags: `--quiet`, `--verbose`, `--json`, `--no-color`, `--yes`,
`--config <file>`, `--cwd <dir>`.

Current Rust output contract (2026-10-17): `--json` is implemented for `context`,
all four `list` commands, `explain` and `detect`. Other commands reject it with
exit code 2 before executing. `--verbose` / `-v` is not implemented and is
rejected; the earlier successful no-op has been removed. Other flags in the
list above remain plans, not advertised capabilities.

Two shapes of command, deliberately. **Primitives** do one thing and compose.
**Workflows** run a sequence people actually ask for as a unit — converting a
folder, building an image and pushing it is one intention, not three, and a tool
that only offers the three makes its user assemble something the tool already
knows how to assemble.

### 3.1 Creating

| Command | Does |
| --- | --- |
| `zel new app [dir]` | A Next.js wrapper app with the engine installed. `--with-docs` adds a starter content folder so there is something to open. `--from <dir>` wraps an existing content folder instead. |
| `zel new docs [dir]` | Just the content folder — for a repository that keeps documentation beside its source and builds elsewhere. `--full` writes one example of every section type. |
| `zel new section <type> [name]` | A section file with its required fields filled in and its optional ones listed as commented defaults. |
| `zel new page <name>` | A page in `pages.yaml`, and the section files it references. |
| `zel new guide <name>` | A guide folder with `_sidebar.yaml` and an index page. |
| `zel eject [dir]` | Pulls the wrapper app out of the base image onto disk, engine already installed. No clone, no npm. |

`zel new section` is the one people will use daily. Nineteen types with their
own fields is more than anyone holds in their head, and the failure it prevents
— writing a plausible section that the build rejects — is the most common way
to lose ten minutes.

    $ zel new section pricing-table sections/home/plans.yaml
    $ zel new section --interactive
    ? Which section type?     › pricing-table
    ? How many tiers?         › 3
    ? Highlight which tier?   › Team
    ✓ Wrote sections/home/plans.yaml

Interactive when a terminal is attached and no arguments are given; every
prompt has a flag, so the same thing is scriptable:

    zel new section pricing-table --tiers 3 --highlight Team --out sections/home/plans.yaml

### 3.2 Understanding

| Command | Does |
| --- | --- |
| `zel explain <type>` | Every field of a section type, its defaults, and a worked example ready to paste. The answer to "what does a bento grid take again?". |
| `zel list sections` | All nineteen, one line each. |
| `zel list themes` | All fourteen, with what distinguishes them. |
| `zel list formats` | What `convert` recognises. |
| `zel list targets` | What `build` can produce. |
| `zel context [file]` | The reference an AI agent authors a content folder from. |

`explain` and `context` are the same knowledge at two scales: one section for a
person, the whole model for an agent. Both are generated from the schemas, so
neither can drift.

### 3.3 Converting

| Command | Does |
| --- | --- |
| `zel detect <dir>` | Says what tool a documentation folder was written for, and how sure it is. Changes nothing. |
| `zel convert <src> --out <dir>` | Converts into a new folder. `--from` to force a format, `--dry-run` to see the plan, `--assets copy\|link\|skip`, `--validate` to check the result. |

`--validate` is opt-in because it is the one part of conversion that needs a
container. Asked for and available, the result is checked against the real
schemas. Asked for and unavailable, the conversion still completes and the tool
says what was skipped and why:

    ✓ Converted 47 pages into ./content
    ! Skipped validation: no container runtime found.
      Install Docker and run `zel check ./content` to validate.

Not a failure. Someone converting documentation on a machine without Docker has
still got their documentation converted, and telling them otherwise would be
punishing them for a dependency they did not choose to need.

Detection reports a confidence, and an ambiguous tree asks rather than guesses.
When nothing is recognised the command does **not** attempt a conversion: it
explains that the tree has no machine-readable structure and points at
`zel context` and an AI agent, which is the honest answer rather than a
plausible wrong one.

**What a converter can and cannot do.** Every one of these tools describes a
*documentation tree*. Zellij has that, and also marketing pages built from
nineteen section types with no equivalent anywhere else. So a converter produces
a faithful guide folder, a `site.yaml` carrying whatever identity it can find,
and a `pages.yaml` with a single hero for the author to build on. It does not
invent a landing page, and it says so in the report every time.

Formats, in the order they are worth doing:

| Format | Structure | Difficulty |
| --- | --- | --- |
| MkDocs | `mkdocs.yml`, `nav:` is a tree | Easy — pure YAML |
| VitePress | `.vitepress/config.*`, `sidebar` | Easy when JSON/YAML, harder when TS |
| Docusaurus | `sidebars.js` + `docusaurus.config.js` | Hard — JavaScript, not data |
| Sphinx | `conf.py` + `toctree` in reST | Hard — Python, and reST is not Markdown |
| Jekyll / Hugo | front-matter conventions, `_data` | Medium |
| Plain Markdown | No config at all | Infer from folder shape, or fall back to the agent |

Docusaurus deserves a note: `sidebars.js` is a JavaScript module, so parsing it
is not enough. The plan is to read the common declarative shapes with a JS
*parser* rather than an interpreter, and to decline — clearly — when the file
computes its exports. Half-evaluating someone's JavaScript is how a converter
produces a wrong sidebar that looks right.

### 3.4 Working

| Command | Does |
| --- | --- |
| `zel check [dir]` | Real schemas; implemented `--deep` static-reference/syntax checks and `--strict` warnings gate. JSON remains planned. |
| `zel dev [dir]` | Development server with hot reload: content is bind-mounted, so editing a Markdown file updates the browser. This is the command an author leaves running. |
| `zel build [dir]` | Builds. See targets below. |
| `zel serve [dir]` | Serves what `build` produced, locally. |
| `zel fmt [dir]` | Canonical formatting for the YAML. |
| `zel watch [dir]` | Re-runs `check` on every change, without a server. For a repository that only wants its documentation kept valid. |

### 3.5 Building — what comes out

There are **two** targets, and the choice is about where the HTML is produced:

| Target | Produces | Runs on |
| --- | --- | --- |
| `static` *(default)* | A folder of HTML, CSS and JS | Netlify, Cloudflare, GitHub Pages, S3 — free tiers everywhere |
| `image` | A container running the site on port 3000, ~150 MB | Railway, Fly.io, Cloud Run, ECS, any registry |

`static` is the default because most documentation sites are read far more than
they are written, a CDN serves them better than a server would, and it is the
only target that costs nothing to host.

The trade-off worth knowing: static loses `next/image` optimisation, so images
are served as authored. For a documentation site that is usually the right
trade; for a marketing site heavy with photography it is not, and `image` exists
for exactly that.

Neither target is a single-page application. Both ship real HTML for every
route, with React rehydrating only the interactive parts. The difference is when
the HTML is written to disk, not whether it exists.

### Two rejected targets

An earlier draft listed four. Two of them were not products:

- **`node`** — a traced Node server, uncontainerised. That is an artefact for a
  platform to consume, not a choice a user makes. Vercel builds it for you; on
  any other host it would go in a container, which is `image`.
- **`image:static`** — nginx serving the static folder. That is the static site
  in a box: a packaging preference. Anyone who wants it can copy `dist/` into
  whatever base image their organisation requires.

Offering four choices where there are two made the decision look harder than it
is. This was found by a reader of the documentation who could not tell the four
apart, which is the correct test.

### 3.6 Optional features

Mathematics and diagrams cost about fifty and a hundred megabytes to install,
and most documentation wants neither. They are optional dependencies of the
engine, so a site that does not ask for them never fetches them — and the
engine renders without them rather than failing over a package nobody wanted.

There are two ways to the same place, and they exist for different people:

| | |
| --- | --- |
| `ghcr.io/…/zellij-base:latest-maths`, `:latest-diagrams`, `:latest-all` | Baked in. Pick the image and build. |
| `--build-arg FEATURES=all` on the plain image | Chosen at build time, one image to pull. |

The second is what `zel build --features` uses. Since the base image installs
nothing, the two cost the same: the feature only decides what a *site build*
fetches, never what an image weighs. The variants exist because naming an image
is sometimes clearer than remembering a flag.

`zel features <list>` applies the same change to an app on disk, which is what
the build argument runs inside the container.

### 3.7 Shipping

| Command | Does |
| --- | --- |
| `zel deploy <provider> [dir]` | Checks, builds and uploads. |
| `zel image build` | Builds a deployable image. Writes the multi-stage Dockerfile itself; the user never sees one. |
| `zel image push [registry]` | Pushes it. |
| `zel image run` | Runs it locally — the last check before it goes anywhere. |
| `zel image dockerfile` | Prints the Dockerfile, for a pipeline that must own the build. |
| `zel ci init github\|gitlab\|circle` | Writes a pipeline that does all of the above on push. |

**The provider decides the target.** A CDN cannot run a container and a
container host has nothing to do with a folder of HTML, so passing `--target` to
`deploy` is an error naming the conflict rather than a silent override.

| Provider | Target |
| --- | --- |
| `netlify`, `cloudflare`, `ghpages`, `s3`, `vercel` | `static` |
| `fly`, `railway` | `image` |

Credentials are read from the environment only — never a flag, never
`zel.toml`, never written to disk. A missing one is reported *before* the build,
because discovering it afterwards wastes the build.

### 3.8 Workflows

One command for a sequence somebody would otherwise script, and get subtly
wrong — forgetting to validate before building is the usual one.

| Command | Runs |
| --- | --- |
| `zel migrate <src> --out <dir>` | `detect` → `convert` → `check` → report. The whole of use-case 2. |
| `zel publish [dir]` | `check` → `build` → `deploy`. Stops at the first failure. |
| `zel ship <src>` | `convert` → `check` → `image build` → `image push`. Existing documentation to a pushed image, in one line. |

    $ zel ship ./docs --registry ghcr.io/acme --tag v2.1.0
    ✓ Detected MkDocs (certain)
    ✓ Converted 47 pages, 3 unmapped — see report
    ✓ Content valid
    ✓ Built image:static — 31 MB
    ✓ Pushed ghcr.io/acme/docs:v2.1.0

Every workflow is the primitives in order, with nothing that cannot also be
done by hand. A workflow that could do something its primitives could not would
be a place for behaviour to hide.

### 3.9 Housekeeping

| Command | Does |
| --- | --- |
| `zel doctor` | Container runtime, image, versions, disk. The first thing to ask for in a bug report. |
| `zel pull` | Pulls or updates the base image. |
| `zel version` | Binary, image and engine versions. |
| `zel upgrade` | Self-update from GitHub Releases. |
| `zel completions <shell>` | Shell completions. |

---

## 4. Architecture

A Cargo workspace, one crate per concern, so a converter can be added without
touching anything that is not a converter.

    cli/
    ├── Cargo.toml                 workspace
    ├── crates/
    │   ├── zellij-cli/            the binary: argument parsing, output, exit codes
    │   ├── zellij-core/           content model, project discovery, config, errors
    │   ├── zellij-convert/        the converter registry and every converter
    │   ├── zellij-engine/         talking to the container
    │   └── zellij-deploy/         deployment targets
    └── tests/                     end-to-end, against real fixture projects

### 4.1 Converters as plugins

A converter implements one trait and registers itself. Adding MkDocs support
means adding a file, not editing a dispatcher.

```rust
pub trait Converter: Send + Sync {
    /// Stable identifier, and what `--from` accepts.
    fn id(&self) -> &'static str;

    /// How strongly this source looks like the format, 0.0 to 1.0.
    /// A converter that finds its own config file returns high confidence;
    /// one that infers from folder shape returns low, so an ambiguous tree
    /// prompts instead of guessing.
    fn detect(&self, source: &Path) -> Result<Confidence>;

    /// Reads the source and produces a content folder in memory.
    /// Nothing is written until the whole conversion has succeeded, so a
    /// failure halfway through leaves no half-built folder behind.
    fn convert(&self, source: &Path, options: &ConvertOptions) -> Result<Conversion>;
}

pub struct Conversion {
    pub files: Vec<OutputFile>,
    pub assets: Vec<AssetCopy>,
    /// Everything the converter could not map, for the report. A page it did
    /// not understand belongs here, never silently dropped.
    pub unmapped: Vec<Unmapped>,
    pub notes: Vec<String>,
}
```

Registration through `inventory`, so a converter is discovered by existing
rather than by being listed somewhere.

The pipeline is the same for every format: **detect → parse into a
tool-agnostic intermediate → emit Zellij → validate → report.** The intermediate
is what keeps `emit` written once instead of once per format, and it is where
"MkDocs nav" and "VitePress sidebar" become the same thing.

### 4.2 Talking to the engine

`zellij-engine` owns every container invocation behind one interface, so the
rest of the tool never shells out to `docker` directly and a future podman or
containerd backend is one implementation rather than a rewrite.

```rust
pub trait Engine: Send + Sync {
    fn check(&self, content: &Path) -> Result<CheckReport>;
    fn build(&self, content: &Path, out: &Path, mode: BuildMode) -> Result<()>;
    fn dev(&self, content: &Path, port: u16) -> Result<DevSession>;
}
```

Two implementations: `DockerEngine` (the default) and `LocalEngine`, which uses
a Node toolchain when one is present — useful to the engine's own developers,
never required of anyone else.

### 4.3 Errors and output

`thiserror` for typed errors, `miette` for reporting, so a bad `site.yaml`
prints with the offending line marked rather than as a stack trace. Full
structured diagnostics remain a design goal. Currently, inspection commands
support `--json` with these contracts:

- `context`: the exact generated engine object (`sectionTypes`, `themes`,
  `sectionTemplates`, `sections`, `files`, plus generator metadata). A filename
  redirects the payload into that file and the confirmation to stderr.
- `explain`: the selected section's complete JSON Schema.
- `list`: one object containing `sections`, `themes`, `formats` or `targets`.
  Section entries have `type`, `fields`, `required` (shared fields excluded);
  themes have `name`, `label`, `description`; formats have `id`, `label`;
  targets have `id`, `description`.
- `detect`: `{ source, matches }`, with `id`, `label`, numeric `confidence` and
  `strength` on each match. No match in an existing folder is a successful
  empty array. Missing/unreadable paths fail; a non-directory is invalid usage.

JSON success writes only the payload to stdout (or the requested context file).
Failures write human diagnostics to stderr, leave stdout empty, and exit
nonzero. Arrays use engine registration order for sections/themes, ID order
for formats, static/image order for targets, and descending confidence then ID
for detection. Consumers key by identifier and tolerate additive fields.

Exit codes: `0` success · `1` general failure · `2` invalid usage ·
`3` content invalid · `4` container runtime unavailable · `5` network failure.

### 4.4 Testing

- Unit tests per converter, over checked-in fixture projects.
- **Golden-file tests**: each fixture has an expected content folder, and the
  test asserts the conversion matches it byte for byte.
- **The real gate**: every fixture's conversion is fed to `zel check`. A
  converter is correct when the engine accepts what it produced — not when it
  produces what a developer expected.
- `trycmd` for the command surface itself, so `--help` output stays honest.

---

## 5. Configuration

Optional `zellij.toml` at the project root:

```toml
[project]
content = "docs"

[engine]
image = "ghcr.io/mylife-inc/zellij-base"
tag   = "latest"

[build]
mode = "standalone"   # or "static"

[deploy]
target = "netlify"
```

Precedence: flags, then environment (`ZELLIJ_*`), then file, then defaults.

---

## 6. Open questions

1. **Which converter first?** MkDocs is easiest and the cleanest proof of the
   plugin design. Docusaurus is the most common in the wild and the hardest.
   Leaning MkDocs, on the grounds that the first one built is really a test of
   the plugin boundary rather than of the format.

**Settled**

- The binary is `zel`. `zellij` collides with the terminal multiplexer of the
  same name, whose users commonly alias it to `zj`.
- The default build target is `static`.
- `zel convert --validate` is opt-in, and degrades to a message rather than a
  failure when no container runtime is present.
- `zel new section` prompts only for fields with no sensible default, and writes
  the rest as commented lines with a note on what each means. The goal is that
  nobody has to remember a section's fields — not that every field is answered
  in a prompt, which for a pricing table would be worse than opening an editor.
- `zel dev` is in scope. It is the command an author leaves running while they
  write, which makes it worth its complexity: a bind-mounted content folder and
  a container running `next dev`, so editing a Markdown file updates the
  browser. `zel eject` still exists for anyone who would rather run the app
  themselves.

## 7. Order of work

1. Workspace, `zellij-core`, error and output plumbing, `doctor`, `version`.
2. `context`, `new` — neither needs Docker, both are immediately useful.
3. `zellij-engine` with the Docker backend; `check`, `build`, `serve`, `dev`.
4. `zellij-convert` with the trait, the registry, and MkDocs.
5. VitePress, then Jekyll/Hugo, then Docusaurus.
6. `eject`, `new section`, `explain`, `fmt`.
7. Deployment and `ci init`.
8. Release packaging and the install script.
