# Native Windows installer. Compatible with Windows PowerShell 5.1 and PowerShell 7.
# Download this script, then run: .\install.ps1 -Version v0.2.0
# ReleaseBaseUrl is an optional directory of mirrored assets and requires Version.
param(
    [string]$Version = $env:ZEL_VERSION,
    [string]$InstallDir = $env:ZEL_INSTALL_DIR,
    [string]$Repository = 'mylife-inc/releases',
    [string]$Scope = 'zellij',
    [string]$ReleaseBaseUrl = $env:ZEL_RELEASE_BASE_URL
)
$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'
Set-StrictMode -Version 2.0
$temporary = $null
$staged = $null
try {
    if ([Environment]::OSVersion.Platform -ne 'Win32NT') { throw 'Use install.sh on Linux and macOS.' }
    if ($Repository -notmatch '^[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+$' -or $Scope -notmatch '^[A-Za-z0-9-]+$') { throw 'Invalid repository or release scope.' }
    if ($ReleaseBaseUrl -and -not $Version) { throw 'ReleaseBaseUrl requires an explicit Version.' }
    $arch = [Environment]::GetEnvironmentVariable('PROCESSOR_ARCHITECTURE', 'Machine')
    if (-not $arch) { $arch = $env:PROCESSOR_ARCHITEW6432 }
    if (-not $arch) { $arch = $env:PROCESSOR_ARCHITECTURE }
    switch ($arch) {
        'AMD64' { $target = 'x86_64-pc-windows-msvc' }
        'ARM64' { $target = 'aarch64-pc-windows-msvc' }
        default { throw "Unsupported Windows architecture: $arch" }
    }
    # Windows PowerShell 5.1 may otherwise select older TLS protocols.
    [Net.ServicePointManager]::SecurityProtocol = [Net.ServicePointManager]::SecurityProtocol -bor [Net.SecurityProtocolType]::Tls12
    if (-not $Version) {
        $headers = @{ 'User-Agent' = 'zel-installer'; Accept = 'application/vnd.github+json' }
        $token = $env:GITHUB_TOKEN
        if (-not $token) { $token = $env:GH_TOKEN }
        if ($token) { $headers.Authorization = "Bearer $token" }
        for ($page = 1; $page -le 10 -and -not $Version; $page++) {
            $releases = @(Invoke-RestMethod -Uri "https://api.github.com/repos/$Repository/releases?per_page=100&page=$page" -Headers $headers -TimeoutSec 60)
            foreach ($release in $releases) {
                if (-not $release.draft -and -not $release.prerelease -and $release.tag_name -match "^$([regex]::Escape($Scope))-(v[0-9]+\.[0-9]+\.[0-9]+)$") {
                    $Version = $Matches[1]
                    break
                }
            }
            if ($releases.Count -lt 100) { break }
        }
        if (-not $Version) { throw "No stable $Scope release found in $Repository; specify -Version for a prerelease." }
    }
    if ($Version -notmatch '^v[0-9]+\.[0-9]+\.[0-9]+(-[0-9A-Za-z.-]+)?$') { throw 'Version must be a version tag such as v0.2.0 or v0.2.0-rc.1.' }
    $archive = "zel-$Version-$target.zip"
    $base = "https://github.com/$Repository/releases/download/$Scope-$Version"
    if ($ReleaseBaseUrl) { $base = $ReleaseBaseUrl.TrimEnd('/') }
    $uri = [Uri]$base
    if (-not $uri.IsAbsoluteUri -or ($uri.Scheme -ne 'https' -and -not ($uri.Scheme -eq 'http' -and $uri.IsLoopback)) -or $uri.Query -or $uri.Fragment -or $uri.UserInfo) {
        throw 'ReleaseBaseUrl must be HTTPS (or loopback HTTP), without credentials, query or fragment.'
    }
    if (-not $InstallDir) { $InstallDir = Join-Path $env:LOCALAPPDATA 'Programs\Zellij\bin' }
    $InstallDir = [IO.Path]::GetFullPath($InstallDir)
    $destination = Join-Path $InstallDir 'zel.exe'
    if (Test-Path -LiteralPath $destination) {
        $existing = Get-Item -Force -LiteralPath $destination
        if ($existing.PSIsContainer -or ($existing.Attributes -band [IO.FileAttributes]::ReparsePoint)) { throw 'The existing zel.exe must be a regular file.' }
    }
    $temporary = Join-Path ([IO.Path]::GetTempPath()) ("zel-install-" + [Guid]::NewGuid().ToString('N'))
    [IO.Directory]::CreateDirectory($temporary) | Out-Null
    $download = Join-Path $temporary $archive
    # No token is forwarded to asset mirrors or redirect destinations.
    Invoke-WebRequest -UseBasicParsing -Uri "$base/$archive" -OutFile $download -TimeoutSec 60
    $checksum = $null
    foreach ($file in @('SHA256SUMS', "$archive.sha256")) {
        try { $body = (Invoke-WebRequest -UseBasicParsing -Uri "$base/$file" -TimeoutSec 60).Content } catch { continue }
        if ($body -is [byte[]]) { $body = [Text.Encoding]::UTF8.GetString($body) }
        foreach ($line in ($body -split "`n")) {
            if ($line.Trim() -match ("^([0-9a-fA-F]{64})\s+\*?(?:\./)?" + [regex]::Escape($archive) + '$')) { $checksum = $Matches[1]; break }
        }
        if ($checksum) { break }
    }
    if (-not $checksum) { throw "No valid checksum found for $archive. Nothing was installed." }
    # Use .NET directly: a PS5.1 child of PS7 can inherit module paths that hide
    # the Get-FileHash script function. Hashing needs no PowerShell module.
    $hasher = [Security.Cryptography.SHA256]::Create()
    $stream = [IO.File]::OpenRead($download)
    try { $actual = [BitConverter]::ToString($hasher.ComputeHash($stream)).Replace('-', '') }
    finally { $stream.Dispose(); $hasher.Dispose() }
    if ($actual -ne $checksum) { throw 'Checksum mismatch. Nothing was installed.' }
    Add-Type -AssemblyName System.IO.Compression.FileSystem
    $zip = [IO.Compression.ZipFile]::OpenRead($download)
    $candidate = Join-Path $temporary 'zel.exe'
    try {
        # Extract exactly the package contract. Do not unpack arbitrary ZIP paths.
        if ($zip.Entries.Count -ne 1 -or $zip.Entries[0].FullName -cne 'zel.exe' -or (($zip.Entries[0].ExternalAttributes -shr 16) -band 0xF000) -eq 0xA000) {
            throw 'The archive must contain exactly one regular zel.exe at its root.'
        }
        [IO.Compression.ZipFileExtensions]::ExtractToFile($zip.Entries[0], $candidate, $false)
    } finally { $zip.Dispose() }
    $reported = & $candidate --version
    if ($LASTEXITCODE -ne 0 -or ($reported -join "`n").Trim() -cne "zel $($Version.Substring(1))") { throw 'The binary version does not match the requested release. Nothing was installed.' }
    [IO.Directory]::CreateDirectory($InstallDir) | Out-Null
    $staged = Join-Path $InstallDir ('.zel-install-' + [Guid]::NewGuid().ToString('N') + '.exe')
    [IO.File]::Copy($candidate, $staged, $false)
    if ([IO.File]::Exists($destination)) {
        [IO.File]::Replace($staged, $destination, [System.Management.Automation.Language.NullString]::Value)
    } else {
        [IO.File]::Move($staged, $destination)
    }
    $staged = $null
    Write-Host "Installed $destination ($Version, $target)"
    Write-Host 'Add the installation directory to your user PATH if needed, then open a new terminal.'
    Write-Host 'Next: zel new portable docs; zel build docs; zel view docs --watch'
} catch {
    [Console]::Error.WriteLine("error: " + $_.Exception.Message)
    exit 1
} finally {
    if ($staged -and [IO.File]::Exists($staged)) { [IO.File]::Delete($staged) }
    if ($temporary -and [IO.Directory]::Exists($temporary)) { [IO.Directory]::Delete($temporary, $true) }
}
