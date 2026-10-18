param([Parameter(Mandatory)][string]$Binary, [Parameter(Mandatory)][string]$Archive, [string]$EntryName = 'zel.exe')
$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.IO.Compression.FileSystem
$zip = [IO.Compression.ZipFile]::Open($Archive, [IO.Compression.ZipArchiveMode]::Create)
try {
    [IO.Compression.ZipFileExtensions]::CreateEntryFromFile($zip, $Binary, $EntryName) | Out-Null
} finally { $zip.Dispose() }
