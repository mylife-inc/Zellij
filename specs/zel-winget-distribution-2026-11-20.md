# WinGet distribution — 20 November 2026

Status: rc.15 is released after 37 source CI and 40 release jobs passed.
Public x64/ARM64 WinGet acceptance passes with the actual published HTTPS URLs.
[Microsoft catalog PR #484934](https://github.com/microsoft/winget-pkgs/pull/484934)
is submitted; the CLA and catalog checks remain separate requirements. No
ordinary catalog install is advertised yet. The private client handoff remains rc.14.

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

Initial run `44710734699` compiled both architectures, then both manifest
validators reported unsupported portable Scope and missing schema headers.
Those packaging issues are corrected in `6b0d6e4`. Cancellation was requested
for the superseded run, but both validator failures completed; neither is a
pass. Run `44711498242` passes both native
ownership suites, including real isolated registry fixtures, before its
superseded lifecycle jobs were cancelled. Full CI `44711923286` caught an rc.14
image-recipe pin after the rc.15 source bump, fixed in `2343b35`; that run was
superseded. At that earlier checkpoint, final lifecycle and full CI evidence were pending.
The later passing source and release runs are recorded below. CI and releases call the same
lifecycle workflow, and release publication requires it to pass.

Microsoft's [manifest guidance](https://learn.microsoft.com/en-us/windows/package-manager/package/manifest)
and [submission process](https://learn.microsoft.com/en-us/windows/package-manager/package/repository)
require a separate community pull request and validation. Confirm identifier
availability, published URLs, package eligibility and lifecycle evidence before
submission. On 2026-11-20, the public `manifests/s/Shebka` path returned
404 and an open-PR search for `Shebka.Zel` found no entry; recheck immediately
before submission. Never advertise `winget install --id Shebka.Zel --exact` as available
until the catalog accepts it and an ordinary public install succeeds.

Retirement must account for external catalog entries: remove/update references
before deleting archives they serve. The native upgrade fixture currently needs
rc.14; the private rollback also needs rc.13 renderers. Neither may be removed
merely because a newer version is published. This extends the existing scoped
consumer audit, not a blanket promise to retain every candidate forever.

## Native acceptance checkpoint

Run `44715233447` passes both lifecycle jobs with harness `2758907` and the
exact native artifacts from source `6b0d6e4`/run `44713052307`. CLI source trees
match. Both native suites pass 12 upgrade/ownership tests. The manifests validate
without warnings under schema 1.28. Real rc.14-to-rc.15 upgrades, deliberately
incorrect checksums, launcher invocation, default/custom user and machine
installations, self-upgrade refusal, separate standalone copies, empty-PATH
portable builds, uninstall and cache preservation pass. See the
[receipts and failure history](evidence/zellij-winget-distribution-2026-11-20.json).

Local manifests lack catalog correlation, so acceptance uses WinGet's recorded
`Shebka.Zel__DefaultSource` product code for uninstall. Public catalog `--id`
lookup remains a separate post-submission check. WinGet v1.29.380 rejects
`--location` on a local-manifest upgrade, and an upgrade without it relocates a
custom portable install to its default location. Acceptance therefore tests a
real default-location upgrade and a separate custom installation. It does not
claim that WinGet preserves a custom location during such an upgrade. No
checksum or security check is disabled; the bad-checksum assertion requires the
specific rejection message, not a generic help-text match.

Manual harness-only reruns may use `binary_run` to reuse previously verified
artifacts; receipts identify both source run and harness commit. Ordinary CI
and tagged releases always use their own native artifacts. Diagnostics are
copied into one runner-temporary directory before upload to avoid cross-volume
artifact roots.

## Published artifact checkpoint

Source CI `44716061788` passes all 37 jobs and release `44724715433` passes
all 40 required jobs on merged source `e1f6ff3`. Public WinGet run `44727865933`
passes both architectures using the checksum-verified published ZIPs and exact
published YAML, including automatic native architecture selection. The manual
`public_version` workflow input repeats these checks without rebuilding binaries.
The source/public receipts remain distinct from catalog-ID installation.

[Release evidence](evidence/zellij-rc15-release-2026-11-20.json) records the exact
assets, jobs, local upgrade, managed channels and catalog PR. Microsoft's CLA
bot requires the account owner's personal acceptance; the agent has not signed
it. Retain the submitted rc.15 archive URLs while the catalog references them.
