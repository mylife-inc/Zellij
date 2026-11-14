# Scoop distribution — 2026-11-13

Status: source implementation; hosted Windows lifecycle acceptance and public
bucket publication are pending. Public rc.6 and its assets remain unchanged.

`scripts/scoop-manifest.mjs` generates a manifest from one exact release's
checksums. It selects x64/ARM64 Windows ZIPs from the shared release repository,
uses Scoop's executable shim, and extracts the executable into `bin/zel.exe`.
The pre-install hook writes exact UTF-8/no-BOM `scoop\n` bytes to
`share/zel/installation-manager`. It runs after extraction and before publishing the executable shim, reusing
the ownership guard in rc.6.
The source CLI now gives `scoop update zel` guidance; published rc.6 already
refuses the same operation with its generic package-manager message. That
message improvement will ship in the next candidate. Neither self-upgrade path
replaces a managed binary, including `--force` or a custom download base.

The owned bucket will be `mylife-inc/scoop-bucket`. Its manifest is updated only
from verified release checksums; no automatic upstream version discovery or
unreviewed download URL is added. Preview status and the matching engine image
are explicit in package notes. The package does not install Node or a container
runtime, or edit shell profiles. Scoop owns its shim. Future tagged releases
emit `zel.json` beside `zel.rb` and require Scoop acceptance before publication.
Existing immutable releases are not retrofitted with new assets.

## Acceptance contract

`.github/workflows/scoop.yml` runs on native Windows x64 and ARM64 under both
Windows PowerShell 5.1 and PowerShell 7 (four combinations). CI consumes
the current native-package artifacts; manual dispatch can consume a verified
public version and optionally the actual public bucket. The official Scoop
v0.6.0 source is pinned at `e6aa3b366bdee8ed138c1e0f7b85192ebdd35d0f`. Its
supported portable configuration lives in a fresh disposable directory,
including spaces in its name. No Scoop installation runs on the user's machine.

`scripts/check-scoop.mjs` exercises the actual Scoop Git bucket and package
lifecycle: architecture-specific extraction, shim execution in Windows PowerShell
5.1/PowerShell 7, exact marker bytes, self-upgrade refusal and executable
preservation, and a native portable creation/check/build. It rejects a corrupt
checksum update while preserving the installation, then performs a normal
Scoop version update and uninstall, checking that shims and the app directory
are removed. The lifecycle test advances a **fixture package revision** only;
it does not claim to test a new CLI release or publish a fictitious version.
The real binary version remains checked before and after the package update.
Public-bucket acceptance additionally installs from the actual HTTPS bucket
and release URLs, then uninstalls. Real rc.6-to-next-candidate acceptance remains
part of the next release's verification.

Local verification: manifest tests, targeted upgrade tests, workspace Clippy,
formatting, JavaScript syntax and Actionlint. Record actual hosted results and
public bucket identity here before describing installation as available.

References: [Scoop manifest contract](https://github.com/ScoopInstaller/Scoop/wiki/App-Manifests),
[portable config bootstrap](https://github.com/ScoopInstaller/Scoop/blob/e6aa3b366bdee8ed138c1e0f7b85192ebdd35d0f/lib/core.ps1),
and [Scoop update handling](https://github.com/ScoopInstaller/Scoop/blob/e6aa3b366bdee8ed138c1e0f7b85192ebdd35d0f/libexec/scoop-update.ps1).

The first public-archive probe (`43530387032`) exposed harness bootstrap errors:
Scoop requires a Git URL rather than a raw Windows path, and its portable root
needs the shim/bucket directories. The corrected harness uses a file URL and
propagates nested Scoop command exit codes as its standard PowerShell shim does.

The second probe (`43531108782`) reached native installation, ownership refusal
and portable builds. Update then exposed Scoop's assumption that its default
`main` bucket exists. The disposable Git fixture now uses that required bucket
name instead of omitting it; no community bucket contents are modified.
