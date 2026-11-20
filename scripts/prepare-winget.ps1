# Test-runner provisioning only. End users do not run this script.
$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'
if ($env:GITHUB_ACTIONS -ne 'true') { throw 'Use a disposable GitHub Actions runner.' }
$version = 'v1.29.380'
$root = Join-Path $env:RUNNER_TEMP ('zel-winget-runtime-' + [guid]::NewGuid())
New-Item -ItemType Directory $root | Out-Null
$assets = @{
    'Microsoft.DesktopAppInstaller_8wekyb3d8bbwe.msixbundle' = '65dea9c01ce08ee7b763366b27c0e651f97db857c11ca9b9c301826c10092f2e'
    'DesktopAppInstaller_Dependencies.zip' = 'ba875afe9d190f61218985ac0292a99d1db710bf93e13c68944ca9d89f0d82d1'
}
foreach ($name in $assets.Keys) {
    $file = Join-Path $root $name
    Write-Host "Downloading $name"
    Invoke-WebRequest -UseBasicParsing -TimeoutSec 180 "https://github.com/microsoft/winget-cli/releases/download/$version/$name" -OutFile $file
    if ((Get-FileHash -LiteralPath $file -Algorithm SHA256).Hash -ne $assets[$name]) { throw "WinGet checksum mismatch: $name" }
}
$deps = Join-Path $root 'dependencies'
Expand-Archive -LiteralPath (Join-Path $root 'DesktopAppInstaller_Dependencies.zip') -DestinationPath $deps
$arch = if ($env:TARGET -eq 'aarch64-pc-windows-msvc') { 'arm64' } else { 'x64' }
Add-Type -AssemblyName System.IO.Compression.FileSystem
$packages = @(Get-ChildItem (Join-Path $deps $arch) -Recurse -File | Where-Object { $_.Extension -in @('.appx', '.msix') })
if ($packages.Count -lt 3) { throw 'Missing WinGet native dependencies.' }
foreach ($package in $packages) {
    $archive = [IO.Compression.ZipFile]::OpenRead($package.FullName)
    try {
        $reader = New-Object IO.StreamReader($archive.GetEntry('AppxManifest.xml').Open())
        try { [xml]$manifest = $reader.ReadToEnd() } finally { $reader.Dispose() }
        $identity = $manifest.Package.Identity
        $installed = Get-AppxPackage -Name $identity.Name | Where-Object { [version]$_.Version -ge [version]$identity.Version -and ($_.Architecture.ToString() -ieq $arch -or $_.Architecture.ToString() -eq 'Neutral') }
        if (-not $installed) { Add-AppxPackage -Path $package.FullName }
    } finally { $archive.Dispose() }
}
Add-AppxPackage -Path (Join-Path $root 'Microsoft.DesktopAppInstaller_8wekyb3d8bbwe.msixbundle')
$app = Get-AppxPackage -Name Microsoft.DesktopAppInstaller
$winget = Join-Path $app.InstallLocation 'winget.exe'
if ((& $winget --version).Trim() -ne $version) { throw 'Unexpected WinGet version.' }
& $winget settings --enable LocalManifestFiles
if ($LASTEXITCODE -ne 0) { throw 'Cannot enable local test manifests.' }
"ZEL_TEST_WINGET=$winget" | Out-File -FilePath $env:GITHUB_ENV -Encoding utf8 -Append
