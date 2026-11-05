<#
.SYNOPSIS
Builds and packages Publication Orchestrator Studio (POS) for release.
Bundles pos.exe (with embedded templates & maps), assets/ (excluding maps & templates),
and sample datasets with exclusions (including samples/DUAL_LANG/maps).
#>
param (
    [string]$OutDir = "dist",
    [switch]$NoZip,
    [switch]$Clean
)

$ErrorActionPreference = "Stop"
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
Set-Location $ScriptDir

Write-Host "==> Building POS in Release mode..." -ForegroundColor Cyan
cargo build --release --manifest-path app/Cargo.toml
if ($LASTEXITCODE -ne 0) {
    Write-Error "Cargo build failed"
    exit $LASTEXITCODE
}

$ReleaseDir = "$OutDir/pos-windows-x64"
if ($Clean -and (Test-Path $ReleaseDir)) {
    Remove-Item -Recurse -Force $ReleaseDir
}
New-Item -ItemType Directory -Force -Path $ReleaseDir | Out-Null

Write-Host "==> Bundling pos.exe..." -ForegroundColor Cyan
Copy-Item "target/release/pos.exe" -Destination "$ReleaseDir/pos.exe" -Force

Write-Host "==> Bundling assets/ (including designcraft-portable, excluding maps & templates)..." -ForegroundColor Cyan
New-Item -ItemType Directory -Force -Path "$ReleaseDir/assets" | Out-Null
Get-ChildItem -Path "assets" | Where-Object { $_.Name -ne "maps" -and $_.Name -ne "templates" } | ForEach-Object {
    if ($_.Name -eq "designcraft-portable") {
        robocopy $_.FullName "$ReleaseDir/assets/designcraft-portable" /E /XD "Recovery" "Cache" "GPUCache" /XF "lock" "SingletonLock" /R:1 /W:1 /NFL /NDL /NJH /NJS /nc /ns /np | Out-Null
    } else {
        Copy-Item -Recurse $_.FullName -Destination "$ReleaseDir/assets" -Force
    }
}

Write-Host "==> Bundling samples with exclusion rules..." -ForegroundColor Cyan
# 1. samples/brs: exclude folder csv and md, exclude *.designcraft, *.pdf
$brsDest = "$ReleaseDir/samples/brs"
New-Item -ItemType Directory -Force -Path $brsDest | Out-Null
Get-ChildItem -Path "samples/brs" -Recurse | ForEach-Object {
    $rel = $_.FullName.Substring((Resolve-Path "samples/brs").Path.Length + 1)
    $parts = $rel.Split([System.IO.Path]::DirectorySeparatorChar, [System.IO.Path]::AltDirectorySeparatorChar)
    if ($parts -contains "csv" -or $parts -contains "md") {
        return
    }
    if ($_.PSIsContainer) {
        New-Item -ItemType Directory -Force -Path (Join-Path $brsDest $rel) | Out-Null
    } else {
        if ($_.Extension -eq ".designcraft" -or $_.Extension -eq ".pdf") {
            return
        }
        Copy-Item $_.FullName -Destination (Join-Path $brsDest $rel) -Force
    }
}

# 2. samples/DUAL_LANG: INCLUDE maps/ (sample geojson + csv), exclude *.designcraft, exclude *.csv outside maps
$dualDest = "$ReleaseDir/samples/DUAL_LANG"
New-Item -ItemType Directory -Force -Path $dualDest | Out-Null
Get-ChildItem -Path "samples/DUAL_LANG" -Recurse | ForEach-Object {
    $rel = $_.FullName.Substring((Resolve-Path "samples/DUAL_LANG").Path.Length + 1)
    $parts = $rel.Split([System.IO.Path]::DirectorySeparatorChar, [System.IO.Path]::AltDirectorySeparatorChar)
    if ($_.PSIsContainer) {
        New-Item -ItemType Directory -Force -Path (Join-Path $dualDest $rel) | Out-Null
    } else {
        if ($_.Extension -eq ".designcraft") {
            return
        }
        if ($_.Extension -eq ".csv" -and (-not ($parts -contains "maps"))) {
            return
        }
        Copy-Item $_.FullName -Destination (Join-Path $dualDest $rel) -Force
    }
}

# 3. samples/SINGLE_LANG: exclude folder maps, exclude *.csv, *.designcraft
$singleDest = "$ReleaseDir/samples/SINGLE_LANG"
New-Item -ItemType Directory -Force -Path $singleDest | Out-Null
Get-ChildItem -Path "samples/SINGLE_LANG" -Recurse | ForEach-Object {
    $rel = $_.FullName.Substring((Resolve-Path "samples/SINGLE_LANG").Path.Length + 1)
    $parts = $rel.Split([System.IO.Path]::DirectorySeparatorChar, [System.IO.Path]::AltDirectorySeparatorChar)
    if ($parts -contains "maps") {
        return
    }
    if ($_.PSIsContainer) {
        New-Item -ItemType Directory -Force -Path (Join-Path $singleDest $rel) | Out-Null
    } else {
        if ($_.Extension -eq ".csv" -or $_.Extension -eq ".designcraft") {
            return
        }
        Copy-Item $_.FullName -Destination (Join-Path $singleDest $rel) -Force
    }
}

if (-not $NoZip) {
    $zipFile = "$OutDir/pos-windows-x64.zip"
    Write-Host "==> Compressing release to $zipFile..." -ForegroundColor Cyan
    if (Test-Path $zipFile) {
        Remove-Item $zipFile -Force
    }
    Compress-Archive -Path "$ReleaseDir/*" -DestinationPath $zipFile -CompressionLevel Optimal
    Write-Host "==> Release archive created: $zipFile" -ForegroundColor Green
}

Write-Host "==> Release bundle completed successfully in: $ReleaseDir" -ForegroundColor Green
