$ErrorActionPreference = 'Stop'
# Keep native CLI exit codes intact in both Windows PowerShell and PowerShell 7.
$PSNativeCommandUseErrorActionPreference = $false
function Show-Usage {
    Write-Output 'Usage: .\generate.ps1 [build|check|watch|view|serve] [zel options]'
    Write-Output '  build  Regenerate site/ (default); --offline uses the cached renderer.'
    Write-Output '  check  Validate source without generating a website; try --deep --strict.'
    Write-Output '  watch  Repeat validation when source changes.'
    Write-Output '  view   Build, serve and open a browser; add --watch for live reload.'
    Write-Output '  serve  Serve the existing site/ without rebuilding it.'
}
$operation = 'build'
$rest = @($args)
if ($rest.Count -gt 0) {
    $first = [string]$rest[0]
    if ($first -in @('-h', '--help', 'help')) { Show-Usage; exit 0 }
    if ($first -in @('build', 'check', 'watch', 'view', 'serve')) {
        $operation = $first
        $rest = @($rest | Select-Object -Skip 1)
    } elseif (-not $first.StartsWith('-')) {
        [Console]::Error.WriteLine("Unknown operation: $first")
        Show-Usage
        exit 2
    }
}
$architecture = [Environment]::GetEnvironmentVariable('PROCESSOR_ARCHITECTURE', 'Machine')
if (-not $architecture) { $architecture = $env:PROCESSOR_ARCHITEW6432 }
if (-not $architecture) { $architecture = $env:PROCESSOR_ARCHITECTURE }
$binary = switch ($architecture) {
    'ARM64' { 'zel-windows-arm64.exe' }
    'AMD64' { 'zel-windows-x64.exe' }
    default { throw "Unsupported Windows architecture: $architecture. Install the appropriate zel release." }
}
$zel = Join-Path $PSScriptRoot $binary
if (-not (Test-Path -LiteralPath $zel -PathType Leaf)) {
    throw "Missing bundled CLI: $binary. Extract the complete documentation package."
}
$project = Join-Path $PSScriptRoot 'zellij'
switch ($operation) {
    'build' { & $zel build $project --features diagrams @rest }
    'check' { & $zel check $project @rest }
    'watch' { & $zel watch $project @rest }
    'view' { & $zel view $project --features diagrams @rest }
    'serve' { & $zel serve (Join-Path $PSScriptRoot 'site') @rest }
}
$code = $LASTEXITCODE
if ($null -eq $code) { throw 'The CLI did not return an exit status.' }
if ($code -ne 0) { exit $code }
if ($operation -eq 'build' -and $rest -notcontains '--help' -and $rest -notcontains '-h') { Write-Host "Ready: $(Join-Path $PSScriptRoot 'site/index.html')" }
exit 0
