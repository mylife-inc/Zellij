# WinGet distribution — 20 November 2026

Status: source rc.15 in progress. Public rc.14 and the private client handoff
remain unchanged. No Microsoft community catalog submission or acceptance.

## Package contract

`Shebka.Zel` identifies Shebka's documentation CLI, distinct from the terminal
multiplexer. Native Windows x64 and ARM64 archives contain `zel.exe` at their
root. WinGet uses ZIP/nested-portable installation with the `zel` alias and
WinGet-selected scope; user/machine and custom installation locations are
covered by acceptance. Portable manifests must not hard-code `Scope`. No new executable
installer or wrapper runtime is added. The package uses the existing
proprietary license designation; public source visibility does not change it.

`scripts/winget-manifest.mjs` generates the three version/installer/locale YAML
files from a release tag and authoritative SHA256SUMS, using the community
PR template's current 1.28 schema. Missing, malformed or
duplicate checksums fail closed. URLs target the durable `mylife-inc/releases`
mirror. Versions before rc.15 are refused because they lack manager ownership
protection. The release workflow packages the manifest set in
`zel-winget-vVERSION.zip`, covered by the combined checksum list, only after
native acceptance. Generating this bundle does not publish to the catalog.

## Managed upgrades

Before networking, Windows `zel upgrade` reads WinGet's portable uninstall
records in HKCU and both applicable HKLM views. A matching product-code prefix,
exact `WinGetPackageIdentifier` and portable installer type are required.
`TargetFullPath` (single-executable packages) or `InstallLocation/zel.exe` (our
ZIP contract) must refer to the running file's actual file identity. Custom
locations, Unicode paths, launcher links and hardlinks are supported; a separate
byte-identical copy is standalone. Registry inspection errors fail closed with
an actionable explanation. No registry record is changed by the CLI.

A managed copy reports `winget upgrade --id Shebka.Zel --exact`. Homebrew and
Scoop retain their existing ownership rules. `ZELLIJ_HOME` stores resources;
it does not override the package manager's executable location. WinGet
uninstallation removes its installation/alias, preserving downloaded resources.

The registry contract was checked against Microsoft's tagged
[PortableARPEntry.cpp](https://github.com/microsoft/winget-cli/blob/v1.29.380/src/AppInstallerCommonCore/PortableARPEntry.cpp)
and portable installer/flow implementation at the same tag. Registry fixtures
write only isolated temporary HKCU test keys and remove their own subtree.

## Acceptance and publication

The reusable workflow provisions checksum-pinned Microsoft WinGet v1.29.380
on native Windows Server 2025 x64 and Windows 11 ARM64. Microsoft supports
WinGet on those systems; older Server 2022 is not the acceptance target.
Tests validate the production manifest schema, then use the exact verified
ZIP bytes through a loopback server for pre-publication lifecycle testing.
A local-only rc.14 manifest tests an actual rc.14-to-rc.15 executable upgrade.
It is not distributed. Tests cover managed self-upgrade refusal, a standalone
copy, a portable build without system tools, uninstall and cache preservation.
Native registry tests cover incorrect identities/types and stale paths.

Initial run `44710734699` compiled both architectures, then ARM64 manifest
validation reported unsupported portable Scope and missing schema headers.
Those packaging issues are corrected; x64 setup and superseded jobs were
cancelled rather than counted as passes. Run `44711498242` passes both native
ownership suites, including real isolated registry fixtures, before its
superseded lifecycle jobs were cancelled. Full CI `44711923286` caught an rc.14
image-recipe pin after the rc.15 source bump, fixed in `2343b35`; that run was
superseded. Final lifecycle and full CI evidence are pending. Local workspace
Rust tests, Clippy and the manifest tests pass; final source/Windows evidence
must be recorded before merging or releasing. CI and releases call the same
lifecycle workflow, and release publication requires it to pass.

Microsoft's [manifest guidance](https://learn.microsoft.com/en-us/windows/package-manager/package/manifest)
and [submission process](https://learn.microsoft.com/en-us/windows/package-manager/package/repository)
require a separate community pull request and validation. Confirm identifier
availability, published URLs, package eligibility and lifecycle evidence before
submission. Never advertise `winget install --id Shebka.Zel --exact` as available
until the catalog accepts it and an ordinary public install succeeds.

Retirement must account for external catalog entries: remove/update references
before deleting archives they serve. The native upgrade fixture currently needs
rc.14; the private rollback also needs rc.13 renderers. Neither may be removed
merely because a newer version is published. This extends the existing scoped
consumer audit, not a blanket promise to retain every candidate forever.
