[CmdletBinding()]
param(
    [string]$InnoCompiler,
    [string]$SourceRevision = ""
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
$repoRoot = [IO.Path]::GetFullPath((Split-Path -Parent $PSScriptRoot))
Push-Location $repoRoot
try {
    $manifest = Get-Content -LiteralPath "plugin/Cargo.toml" -Raw
    $versionMatch = [regex]::Match($manifest, '(?m)^version\s*=\s*"([^"]+)"')
    if (-not $versionMatch.Success) { throw "Plugin version is missing from plugin/Cargo.toml" }
    $version = $versionMatch.Groups[1].Value

    if (-not $InnoCompiler) {
        $programFilesX86 = [Environment]::GetFolderPath([Environment+SpecialFolder]::ProgramFilesX86)
        $InnoCompiler = Join-Path $programFilesX86 "Inno Setup 6/ISCC.exe"
        if (-not (Test-Path -LiteralPath $InnoCompiler)) {
            $command = Get-Command ISCC.exe -ErrorAction SilentlyContinue
            if ($command) { $InnoCompiler = $command.Source }
        }
    }
    if (-not (Test-Path -LiteralPath $InnoCompiler)) {
        throw "Install Inno Setup 6 or pass -InnoCompiler with the path to ISCC.exe."
    }
    $InnoCompiler = (Resolve-Path -LiteralPath $InnoCompiler).Path

    & cargo run --locked --package xtask -- bundle emberfat --release
    if ($LASTEXITCODE -ne 0) { throw "VST3 build failed with exit code $LASTEXITCODE" }
    $bundle = Join-Path $repoRoot "target/bundled/emberfat.vst3"
    $binary = Join-Path $bundle "Contents/x86_64-win/emberfat.vst3"
    if (-not (Test-Path -LiteralPath $binary -PathType Leaf)) { throw "Built VST3 binary is missing: $binary" }

    # Recreate only this repository's generated staging folder. Repeated builds cannot nest bundles.
    $dist = [IO.Path]::GetFullPath((Join-Path $repoRoot "dist"))
    if ($dist -ne (Join-Path $repoRoot "dist")) { throw "Unexpected staging path: $dist" }
    if (Test-Path -LiteralPath $dist) { Remove-Item -LiteralPath $dist -Recurse -Force }
    New-Item -ItemType Directory -Path $dist | Out-Null
    $stagedBundle = Join-Path $dist "EmberFat.vst3"
    Copy-Item -LiteralPath $bundle -Destination $stagedBundle -Recurse

    $licenses = Join-Path $stagedBundle "Contents/Resources/Licenses"
    New-Item -ItemType Directory -Path $licenses -Force | Out-Null
    Copy-Item -LiteralPath "plugin/assets/fonts/OFL.txt" -Destination (Join-Path $licenses "Barlow-OFL.txt")
    Copy-Item -LiteralPath "plugin/assets/fonts/PermanentMarker-LICENSE.txt" -Destination $licenses
    Copy-Item -LiteralPath "THIRD_PARTY_NOTICES.md" -Destination $licenses
    & $InnoCompiler "/DMyAppVersion=$version" "installer/windows/EmberFat.iss"
    if ($LASTEXITCODE -ne 0) { throw "Installer compilation failed with exit code $LASTEXITCODE" }

    $installer = Join-Path $dist "EmberFat-Windows-Setup.exe"
    $checksum = (Get-FileHash -LiteralPath $installer -Algorithm SHA256).Hash.ToLowerInvariant()
    "$checksum  EmberFat-Windows-Setup.exe" | Set-Content -LiteralPath (Join-Path $dist "SHA256SUMS.txt") -Encoding ascii
    if (-not $SourceRevision) { $SourceRevision = "local-working-copy" }
    @("EmberFat $version", "Source: $SourceRevision", "Format: Windows x64 VST3") |
        Set-Content -LiteralPath (Join-Path $dist "BUILD.txt") -Encoding utf8
    if ($env:GITHUB_OUTPUT) { "version=$version" | Add-Content -LiteralPath $env:GITHUB_OUTPUT }
    Write-Output "Ready: $installer (version $version)"
} finally {
    Pop-Location
}
