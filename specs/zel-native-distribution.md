# Native package and installer contract

Status: implemented in source; hosted acceptance is recorded on the implementing
pull request. No new version tag or public binary release is created by this work.
The existing `zellij-v0.1.3` release predates the portable documentation workflow
and contains no native Windows packages. Do not replace those existing assets.

## Package matrix

| Target | Archive member / format | Native build and install test environment |
| --- | --- | --- |
| x86_64-unknown-linux-gnu | `zel` / tar.gz | Ubuntu 22.04 x64 |
| aarch64-unknown-linux-gnu | `zel` / tar.gz | Ubuntu 22.04 ARM64 |
| x86_64-apple-darwin | `zel` / tar.gz | macOS 15 Intel |
| aarch64-apple-darwin | `zel` / tar.gz | macOS 15 Apple Silicon |
| x86_64-pc-windows-msvc | `zel.exe` / zip | Windows Server 2022 x64 |
| aarch64-pc-windows-msvc | `zel.exe` / zip | Windows 11 ARM64 |

These are tested environments, not a claim about every OS version. GNU/Linux
uses the Ubuntu 22.04/glibc 2.35 build baseline. Alpine/musl, 32-bit platforms,
BSD, macOS signing/notarization and Windows Authenticode/MSI are not included.
Windows builds statically link the MSVC runtime. Portable operations require
no Docker or Node after installation; hosted container commands still require
a compatible container runtime. Windows packaging acceptance does not establish
Docker Desktop integration for every hosted command.

Runner labels were checked against [GitHub's runner reference](https://docs.github.com/en/actions/reference/runners/github-hosted-runners).
Static CRT selection follows [Rust's linkage reference](https://doc.rust-lang.org/reference/linkage.html#static-and-dynamic-c-runtimes).

## Build and release path

`scripts/package-native.mjs <version|workspace> <target> [outdir]` reads the
committed workspace version, rejects a mismatched tag or unsupported target,
and runs `cargo build --locked --release` for that target. It writes
`zel-vVERSION-TARGET.tar.gz` or `.zip` and its `.sha256`. Existing output filenames
are refused. Packaging needs Node 24 plus Rust and platform archive tools;
running `zel` does not. `scripts/publish.sh binary` delegates to this path.

`.github/workflows/native-packages.yml` runs the same six-target package and
installer tests in ordinary CI and tagged releases. Archives are uploaded only
after acceptance; normal CI retains them as artifacts without creating releases.
The release workflow requires a version tag matching `cli/Cargo.toml` before
starting publication jobs. Its binaries job calls the tested reusable workflow.
Both installers and combined `SHA256SUMS` accompany release assets. The optional
shared-repository publication retains its existing token requirement; no new
credential is needed for CI package tests. The old `zrelease-local` fallback is
not this six-target acceptance path and should not be used for the next release.

## Installers

`install.sh` handles GNU/Linux and macOS; `install.ps1` handles native Windows
from Windows PowerShell 5.1 or PowerShell 7. Windows selects the machine's
architecture even when the shell runs through emulation. Unix defaults to
`~/.local/bin`; Windows defaults to `%LOCALAPPDATA%\Programs\Zellij\bin`.
Neither installer edits PATH, profiles, or persistent environment settings.

Both accept `ZEL_VERSION`, `ZEL_INSTALL_DIR`, and a pinned-version
`ZEL_RELEASE_BASE_URL` for a directory of mirrored assets. Mirrors use HTTPS;
loopback HTTP is allowed for local distribution/testing. Windows also exposes
`-Version`, `-InstallDir`, `-ReleaseBaseUrl`, `-Repository` and `-Scope` parameters.
Latest discovery skips prereleases; use an explicit version for a release candidate.
Windows checks up to ten pages of scoped GitHub release metadata; pin a version
if it cannot find a stable release. No API token is sent to mirror downloads.

The downloaded archive must match `SHA256SUMS` or its per-archive `.sha256`.
Windows accepts only a single regular `zel.exe` entry at the ZIP root. Unix
extracts only the named `zel` member and rejects symlinks. Both run the candidate's
`--version` before replacing an installation. A bad checksum, archive, version
or candidate execution leaves the old executable intact. Staging uses a unique
file in the destination directory, then replaces that directory entry. This
does not overwrite unrelated `zel.new` files or truncate hard-linked inodes.
Windows replacement can fail while the installed executable is locked; stop it
and retry. The legacy Unix checksum-skip environment flag remains explicit;
the Windows installer has no checksum bypass.

## Acceptance

`scripts/check-native-install.mjs` serves real built packages from a loopback
HTTP fixture and runs the actual installers, including both Windows shells.
It verifies per-file/combined checksums, replacement, corrupt/missing checksums,
unsafe archives, wrong versions, existing destination conflicts and preservation.
It checks the installed executable's version and actual CPU architecture.

With external tools removed from PATH, that installed executable creates,
strictly checks and builds a portable starter. Acceptance inspects relocated
HTML/search/assets and serves pages through both `serve` and `view --no-open`.
Real Ctrl-C is verified, including the existing private-console harness on
Windows. The separate portable browser matrix continues to establish actual
offline Chromium/WebKit navigation, search and no-JavaScript behavior; package
tests alone do not claim a six-platform browser matrix or GUI opener acceptance.

Before publishing a new version, update the workspace version/lockfile in git,
run the complete CI gates, and tag that reviewed commit. Release naming,
documentation publication and the long-term shared repository remain deliberate
release decisions; this work does not tag, deploy the documentation, or change
repository visibility.
