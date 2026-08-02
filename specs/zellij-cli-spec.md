# The Zellij CLI — specification

A single native binary through which a person does everything with Zellij:
scaffold a content folder, convert one from another tool, validate it, build it,
serve it, and ship it.

Status: **draft, for sign-off.** Nothing here is implemented yet.

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
2. **The engine validates, not the CLI.** `zellij check` runs the real schemas
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
committed, and compiled into the binary with `include_str!`. So `zellij context`
needs neither Node nor Docker, and still cannot drift.

---

## 2. Distribution

| Artefact | Where | Who can get it |
| --- | --- | --- |
| `zellij` binary | GitHub Releases, per platform | Anyone with the release URL |
| `ghcr.io/mylife-inc/zellij-base` | GHCR, public | Anyone |
| Engine source | Private repository | The team |

Platforms: `x86_64-unknown-linux-gnu`, `aarch64-unknown-linux-gnu`,
`x86_64-apple-darwin`, `aarch64-apple-darwin`, `x86_64-pc-windows-msvc`.

Install: a shell script that picks the right asset, plus Homebrew later.
Checksums published alongside; the installer verifies before moving into place.

---

## 3. Command surface

    zellij <command> [arguments] [--flags]

Global flags: `--quiet`, `--verbose`, `--json`, `--no-color`, `--yes`,
`--config <file>`.

### 3.1 Starting a project

| Command | Does |
| --- | --- |
| `zellij new [dir]` | Writes a content folder that already validates. `--minimal` for the smallest thing that builds, `--full` for one example of every section type. |
| `zellij eject [dir]` | Extracts the Next.js wrapper out of the base image onto disk, engine already installed. For someone who wants to run `next dev` themselves. |
| `zellij context [file]` | Writes the reference an AI agent authors from. Embedded in the binary; needs nothing else. |

`zellij new` covers use-case 3. `zellij context` covers use-case 1, and the
fallback half of use-case 2.

### 3.2 Converting from another tool

    zellij convert <source> --out <dir> [--from <format>] [--dry-run]

Reads a documentation tree written for another tool and emits a Zellij content
folder beside it. Never in place.

| Flag | Does |
| --- | --- |
| `--from` | Force a format instead of detecting. |
| `--out` | Destination. Required — there is no default that could not surprise someone. |
| `--dry-run` | Report what would be written, write nothing. |
| `--assets copy\|link\|skip` | What to do with images. Default `copy`. |

Detection reports a confidence, and an ambiguous tree asks rather than guesses.
When nothing is recognised, the command does **not** attempt a conversion: it
explains that the tree has no machine-readable structure and points at
`zellij context` and an AI agent, which is the honest answer.

**What a converter can and cannot do.** Every one of these tools describes a
*documentation tree*. Zellij has that, and also marketing pages built from
nineteen section types with no equivalent anywhere else. So a converter produces
a faithful guide folder, a `site.yaml` carrying whatever identity it can find,
and a `pages.yaml` with a single hero for the author to build on. It does not
invent a landing page. This is stated in the report, every time, so nobody
expects otherwise.

Formats, in the order they are worth doing:

| Format | Structure | Difficulty |
| --- | --- | --- |
| MkDocs | `mkdocs.yml`, `nav:` is a tree | Easy — pure YAML |
| VitePress | `.vitepress/config.*`, `sidebar` | Easy when JSON/YAML, harder when TS |
| Docusaurus | `sidebars.js` + `docusaurus.config.js` | Hard — JavaScript, not data |
| Sphinx | `conf.py` + `toctree` in reST | Hard — Python, and reST is not Markdown |
| Jekyll / Hugo | front-matter conventions, `_data` | Medium |
| Plain folder | No config at all | Fall back to the agent |

Docusaurus deserves a note: `sidebars.js` is a JavaScript module, so parsing it
is not enough. The plan is to read the common declarative shapes with a JS
*parser* rather than an interpreter, and to decline — clearly — when the file
computes its exports. Half-evaluating someone's JavaScript is how a converter
produces a wrong sidebar that looks right.

### 3.3 Working on content

| Command | Does |
| --- | --- |
| `zellij check [dir]` | Validates. Runs the engine in the container. `--json` for CI. |
| `zellij dev [dir]` | Development server with hot reload, content bind-mounted. |
| `zellij build [dir]` | Builds the site. `--out <dir>` to extract the result. |
| `zellij serve [dir]` | Serves a built site locally. |
| `zellij add <type> [name]` | Writes a new section file of that type, with its required fields filled in and its optional ones listed as comments. |
| `zellij list sections` | Every section type and what it is for. |
| `zellij fmt [dir]` | Canonical formatting for the YAML files. |

`zellij add` is the authoring aid that matters most: nineteen section types is
more than anyone will remember, and a stub with the right fields is faster than
reading a reference.

### 3.4 Shipping

| Command | Does |
| --- | --- |
| `zellij image build [dir]` | Builds a deployable image from a content folder. Generates the multi-stage Dockerfile; the user never writes one. |
| `zellij image push` | Pushes to a registry. |
| `zellij deploy <target>` | `netlify`, `vercel`, `cloudflare`, `s3`, `ghpages`. Static output where the target wants static. |
| `zellij ci init <provider>` | Writes a pipeline — `github`, `gitlab`, `circle` — that builds and publishes on push. |

### 3.5 Housekeeping

| Command | Does |
| --- | --- |
| `zellij doctor` | Checks the container runtime, the image, versions, and disk. First thing to ask for in a bug report. |
| `zellij pull` | Pulls or updates the base image. |
| `zellij version` | Binary version, image version, engine version. |
| `zellij upgrade` | Self-update from GitHub Releases. |
| `zellij completions <shell>` | Shell completions. |

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
prints with the offending line marked rather than as a stack trace. Every
command supports `--json` with a stable schema, because a CLI that can only be
read by a human cannot be used by a pipeline.

Exit codes: `0` success · `1` general failure · `2` invalid usage ·
`3` content invalid · `4` container runtime unavailable · `5` network failure.

### 4.4 Testing

- Unit tests per converter, over checked-in fixture projects.
- **Golden-file tests**: each fixture has an expected content folder, and the
  test asserts the conversion matches it byte for byte.
- **The real gate**: every fixture's conversion is fed to `zellij check`. A
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

1. **Is `zellij dev` worth the complexity?** It needs a bind mount, a
   long-running container and signal handling. The alternative is `zellij eject`
   plus the user's own `npm run dev`. I lean towards shipping `eject` first and
   `dev` once the rest is proven.
2. **Static or standalone by default?** Static deploys anywhere and costs
   nothing to host, but loses `next/image` optimisation. Standalone keeps
   everything and needs a Node runtime. Proposal: static by default, since most
   documentation sites are hosted on a CDN.
3. **Does `convert` need to run `check` itself?** It requires Docker, which the
   other conversion commands do not. Proposal: attempt it, and degrade to a
   warning when no container runtime is present rather than failing.
4. **Which converter first?** MkDocs is the easiest and a clean proof of the
   plugin design. Docusaurus is the most common in the wild and the hardest.

---

## 7. Order of work

1. Workspace, `zellij-core`, error and output plumbing, `doctor`, `version`.
2. `context`, `new` — neither needs Docker, both are immediately useful.
3. `zellij-engine` with the Docker backend; `check`, `build`, `serve`.
4. `zellij-convert` with the trait, the registry, and MkDocs.
5. VitePress, then Jekyll/Hugo, then Docusaurus.
6. `eject`, `add`, `list`, `fmt`.
7. Deployment and `ci init`.
8. Release packaging and the install script.
