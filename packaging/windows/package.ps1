<#
.SYNOPSIS
  Build, sign and package DesignCraft for Windows.

.DESCRIPTION
  Produces, in $env:DIST (default: dist/release):
    designcraft-<version>-windows-<arch>.msi            per-machine installer (WiX v5)
    designcraft-<version>-windows-<arch>-portable.zip   designcraft.exe + designcraft-cli.exe + portable.txt

  The binaries link the C runtime statically (+crt-static), so neither the MSI nor the portable
  zip needs the Visual C++ redistributable. Signing is delegated to sign.ps1 (skipped with a
  warning when no signing secrets are set).

  Needs: Rust (MSVC toolchain + the target), the Windows SDK (rc.exe, signtool.exe),
  and WiX v5: dotnet tool install --global wix --version 5.0.2

.EXAMPLE
  pwsh packaging/windows/package.ps1 -Arch x64
  pwsh packaging/windows/package.ps1 -Arch x86 -SkipBuild
  pwsh packaging/windows/package.ps1 -Arch arm64     # cross-compiled; needs the MSVC ARM64 build tools
#>
param(
  [ValidateSet('x64', 'x86', 'arm64')] [string] $Arch = 'x64',
  [switch] $SkipBuild,
  [switch] $PortableOnly
)
$ErrorActionPreference = 'Stop'
$Root = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path

function Invoke-Native([string] $What, [scriptblock] $Block) {
  Write-Output "==> $What"
  & $Block
  if ($LASTEXITCODE -ne 0) { throw "$What failed with exit code $LASTEXITCODE" }
}

# The version lives in one place: [workspace.package] version in the root Cargo.toml.
$Version = $env:DESIGNCRAFT_VERSION
if (-not $Version) {
  $inPkg = $false
  foreach ($line in Get-Content (Join-Path $Root 'Cargo.toml')) {
    if ($line -match '^\s*\[') { $inPkg = ($line.Trim() -eq '[workspace.package]'); continue }
    if ($inPkg -and $line -match '^\s*version\s*=\s*"([^"]+)"') { $Version = $Matches[1]; break }
  }
}
if (-not $Version) { throw 'could not read [workspace.package] version from Cargo.toml' }
# MSI ProductVersion is numeric (major.minor.build); pre-release tags are dropped there.
$MsiVersion = ($Version -split '-')[0]

$Target = switch ($Arch) { 'x64' { 'x86_64-pc-windows-msvc' } 'x86' { 'i686-pc-windows-msvc' } 'arm64' { 'aarch64-pc-windows-msvc' } }
$Dist = if ($env:DIST) { $env:DIST } else { Join-Path $Root 'dist\release' }
$TargetDir = if ($env:CARGO_TARGET_DIR) { $env:CARGO_TARGET_DIR } else { Join-Path $Root 'target' }
New-Item -ItemType Directory -Force -Path $Dist | Out-Null

if (-not $env:DESIGNCRAFT_BUILD_SHA) { $env:DESIGNCRAFT_BUILD_SHA = (git -C $Root rev-parse HEAD 2>$null) }
if (-not $env:DESIGNCRAFT_BUILD_DATE) { $env:DESIGNCRAFT_BUILD_DATE = (Get-Date).ToUniversalTime().ToString('yyyy-MM-dd') }

Write-Output "DesignCraft $Version for Windows $Arch ($Target)"

if (-not $SkipBuild) {
  # Static CRT: no VC++ redistributable needed. Scoped to the target so host build scripts and
  # proc-macros are unaffected.
  $flagVar = 'CARGO_TARGET_' + ($Target.ToUpper() -replace '-', '_') + '_RUSTFLAGS'
  [Environment]::SetEnvironmentVariable($flagVar, '-C target-feature=+crt-static')
  # Fail the build (rather than warn) if the icon/VERSIONINFO can't be embedded.
  $env:DESIGNCRAFT_REQUIRE_WINRES = '1'
  Invoke-Native "cargo build ($Target)" { cargo build --release --locked -p pgs -p pubgen-cli -p designcraft-cli --target $Target }
}

$Bin = Join-Path $TargetDir "$Target\release"

# Check binaries' PE headers before packaging. Machine (COFF header) must match -Arch, so an
# x64 build can never ship labelled arm64. Subsystem (optional header): 2 = Windows GUI, 3 = console.
# The app must be GUI (no console window opens with it); CLI binaries must stay console so their output
# reaches the terminal.
function Get-PeHeader([string] $Path) {
  $bytes = [System.IO.File]::ReadAllBytes($Path)
  $pe = [BitConverter]::ToInt32($bytes, 0x3C)
  return @{ Machine = [BitConverter]::ToUInt16($bytes, $pe + 4); Subsystem = [BitConverter]::ToUInt16($bytes, $pe + 0x5C) }
}
$Machine = switch ($Arch) { 'x64' { 0x8664 } 'x86' { 0x14C } 'arm64' { 0xAA64 } }
$BinChecks = @(
  @('pgs.exe', 2),
  @('pgs-cli.exe', 3),
  @('pgs-export.exe', 3),
  @('designcraft-cli.exe', 3)
)
foreach ($check in $BinChecks) {
  $exePath = Join-Path $Bin $check[0]
  if (Test-Path $exePath) {
    $h = Get-PeHeader $exePath
    if ($h.Machine -ne $Machine) { throw "$($check[0]) is for machine 0x$('{0:X}' -f $h.Machine), expected 0x$('{0:X}' -f $Machine) ($Arch)" }
    if ($h.Subsystem -ne $check[1]) { throw "$($check[0]) has PE subsystem $($h.Subsystem), expected $($check[1])" }
    Write-Output "ok $($check[0]): $Arch, PE subsystem $($h.Subsystem)"
  }
}
$Stage = Join-Path $TargetDir "windows-package\$Arch"
Remove-Item -Recurse -Force $Stage -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Force -Path $Stage | Out-Null
foreach ($check in $BinChecks) {
  $exePath = Join-Path $Bin $check[0]
  if (Test-Path $exePath) {
    Copy-Item $exePath $Stage
  }
}

$StageExes = Get-ChildItem -Path $Stage -Filter '*.exe' | ForEach-Object { $_.FullName }
if ($StageExes) {
  & (Join-Path $PSScriptRoot 'sign.ps1') @StageExes
}

# Required embedded-font notices ship independently of optional font inputs.
Copy-Item -LiteralPath (Join-Path $Root 'NOTICE') -Destination $Stage

# ---- font notices for the MSI ------------------------------------------------------------------
# The portable ZIP already carries these notices; stage the complete files before MSI creation too.
$Docs = Join-Path $Stage 'font-licences'
New-Item -ItemType Directory -Force -Path $Docs | Out-Null
$HasCraftFontLicences = 0
if ($env:CRAFT_FONTS_DIR) {
  foreach ($ofl in Get-ChildItem -Path (Join-Path $env:CRAFT_FONTS_DIR 'fonts\*\OFL.txt') -File -ErrorAction SilentlyContinue) {
    Copy-Item -LiteralPath $ofl.FullName -Destination (Join-Path $Docs "OFL-$($ofl.Directory.Name).txt")
    $HasCraftFontLicences = 1
  }
}
# ---- end font notices for the MSI --------------------------------------------------------------

if (-not $PortableOnly) {
  # ---- MSI ---------------------------------------------------------------------------------------
  $Msi = Join-Path $Dist "designcraft-$Version-windows-$Arch.msi"
  Invoke-Native 'wix build' {
    wix build (Join-Path $PSScriptRoot 'designcraft.wxs') (Join-Path $PSScriptRoot 'installer-ui.wxs') -arch $Arch `
      -d "Version=$MsiVersion" -d "BinDir=$Stage" -d "IconPath=$(Join-Path $Root 'assets\app-icon\designcraft.ico')" `
      -d "DocsDir=$Docs" -d "HasCraftFontLicences=$HasCraftFontLicences" `
      -o $Msi
  }
  # Check the compiled scope, path and registration contract before signing.
  & (Join-Path $PSScriptRoot 'check-install-policy.ps1') -MsiPath $Msi
  # wix writes its debug symbols (.wixpdb) next to the MSI; keep them out of the release assets.
  Remove-Item -Force -ErrorAction SilentlyContinue ([IO.Path]::ChangeExtension($Msi, '.wixpdb'))
  & (Join-Path $PSScriptRoot 'sign.ps1') $Msi
}

# ---- portable zip ------------------------------------------------------------------------------
$Portable = Join-Path $TargetDir "windows-package\designcraft-$Version-windows-$Arch-portable"
Remove-Item -Recurse -Force $Portable -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Force -Path $Portable | Out-Null
Copy-Item (Join-Path $Stage '*.exe') $Portable
# The marker that makes DesignCraft keep its data in .\DesignCraftData next to the exe instead of
# %APPDATA% (#574; see apps/designcraft/src/profile.rs).
Set-Content -Encoding ascii -Path (Join-Path $Portable 'portable.txt') -Value @(
  'Publication Generator Studio (PGS) portable: preferences, logs and crash recovery are kept in the DesignCraftData folder',
  'next to pgs.exe. Delete this file to use %APPDATA%\DesignCraft instead.'
)
foreach ($f in 'README.md', 'LICENSE', 'LICENSE-MIT', 'LICENSE-APACHE', 'NOTICE') {
  $p = Join-Path $Root $f
  if (Test-Path $p) { Copy-Item $p $Portable }
}
# Builds made with craft-fonts (CRAFT_FONTS_DIR) embed its fonts: ship their licences.
if ($env:CRAFT_FONTS_DIR) {
  foreach ($ofl in Get-ChildItem -Path (Join-Path $env:CRAFT_FONTS_DIR 'fonts\*\OFL.txt') -ErrorAction SilentlyContinue) {
    Copy-Item $ofl.FullName (Join-Path $Portable "OFL-$($ofl.Directory.Name).txt")
  }
}

# Assets: include templates, pubgen glossaries, hyphenation, and icc profiles
$AssetsSrc = Join-Path $Root 'assets'
if (Test-Path $AssetsSrc) {
  Copy-Item -Recurse -Force $AssetsSrc $Portable
}

# Pubgen samples: include all files from plan/pubgen/samples, including everything in brs, DUAL_LANG,
# and SINGLE_LANG (with all CSVs, ZIPs, pubgen packages, and documents), excluding book_cover folder.
# Every profile lands flat at samples\<profile> with its own example files inside. A stale copy of the sample tree
# inside a profile ("…\plan\pubgen\samples\…", which a manual re-copy leaves behind) is skipped: shipping it nests the
# samples inside their own repository path, e.g. samples\brs\chart\plan\pubgen\samples\brs.
$SamplesSrc = Join-Path $Root 'plan\pubgen\samples'
$SamplesDest = Join-Path $Portable 'samples'
if (Test-Path $SamplesSrc) {
  New-Item -ItemType Directory -Force -Path $SamplesDest | Out-Null
  $canonicalSamplesSrc = (Resolve-Path $SamplesSrc).Path
  $script:skippedNestedCopies = 0
  Get-ChildItem -Path $SamplesSrc -Recurse | Where-Object {
    if ($_.PSIsContainer) { return $false }
    $relative = $_.FullName.Substring($canonicalSamplesSrc.Length).TrimStart('\', '/')
    if ($relative.StartsWith("book_cover\") -or $relative.StartsWith("book_cover/") -or $relative -eq "book_cover") {
      return $false
    }
    if ($relative -match '(^|[\\/])(plan|pubgen)([\\/]|$)') {
      $script:skippedNestedCopies++
      return $false
    }
    # For brs, DUAL_LANG, and SINGLE_LANG: include EVERYTHING (all files and extensions)
    $isTargetSample = ($relative -match '^(brs|DUAL_LANG|SINGLE_LANG)[\\/]')
    if (-not $isTargetSample -and ($_.Extension -eq '.csv' -or $_.Extension -eq '.zip')) {
      return $false
    }
    return $true
  } | ForEach-Object {
    $relative = $_.FullName.Substring($canonicalSamplesSrc.Length).TrimStart('\', '/')
    $targetFile = Join-Path $SamplesDest $relative
    $targetParent = Split-Path -Parent $targetFile
    if (-not (Test-Path $targetParent)) {
      New-Item -ItemType Directory -Force -Path $targetParent | Out-Null
    }
    Copy-Item -LiteralPath $_.FullName -Destination $targetFile -Force
  }
  if ($script:skippedNestedCopies -gt 0) {
    Write-Warning "samples: skipped $($script:skippedNestedCopies) file(s) from a copy of the sample tree nested inside plan\pubgen\samples; delete that folder in the source tree to silence this."
  }

  # The portable app opens its example data from samples\<profile>, so the flat layout is part of the release
  # contract: check it here and fail the build, instead of zipping samples the app cannot find.
  foreach ($expected in @('brs\ntp202610.xlsx', 'brs\U1.rtf', 'SINGLE_LANG\BAB_02_2025.xlsx', 'images\bps_logo.png')) {
    if (-not (Test-Path (Join-Path $SamplesDest $expected))) {
      throw "samples: samples\$expected is missing; plan\pubgen\samples must ship every profile's example files flat at samples\<profile>"
    }
  }
  $dualLangFiles = Get-ChildItem -Path (Join-Path $SamplesDest 'DUAL_LANG') -Recurse -File -ErrorAction SilentlyContinue
  if (-not $dualLangFiles -or $dualLangFiles.Count -eq 0) {
    throw "samples: DUAL_LANG sample folder is empty in $SamplesDest"
  }
  $NestedSampleDirs = @(Get-ChildItem -Path $SamplesDest -Recurse -Directory | Where-Object { $_.Name -eq 'plan' -or $_.Name -eq 'pubgen' })
  if ($NestedSampleDirs.Count -gt 0) {
    $relativeNested = $NestedSampleDirs | ForEach-Object { $_.FullName.Substring($SamplesDest.Length + 1) }
    throw "samples: the portable package would ship nested sample folders ($($relativeNested -join ', ')); samples belong directly under samples\<profile>"
  }
}

# Exclude sensitive stock maps from loose assets directory since they are embedded into the exe
$PortableMapsDir = Join-Path $Portable 'assets\pubgen\maps'
if (Test-Path $PortableMapsDir) {
  Remove-Item -Recurse -Force $PortableMapsDir -ErrorAction SilentlyContinue
}
$PortableMapsZip = Join-Path $Portable 'assets\pubgen\pubgen_maps.zip'
if (Test-Path $PortableMapsZip) {
  Remove-Item -Force $PortableMapsZip -ErrorAction SilentlyContinue
}

$Zip = Join-Path $Dist "designcraft-$Version-windows-$Arch-portable.zip"
Remove-Item -Force $Zip -ErrorAction SilentlyContinue
# ZipArchive entry names are written by hand: Windows PowerShell 5.1's Compress-Archive writes
# backslash separators, which the in-app updater's extractor rejects as unsafe paths.
Add-Type -AssemblyName System.IO.Compression
Add-Type -AssemblyName System.IO.Compression.FileSystem
$ZipStream = [System.IO.File]::Open($Zip, [System.IO.FileMode]::CreateNew)
$Archive = New-Object System.IO.Compression.ZipArchive($ZipStream, [System.IO.Compression.ZipArchiveMode]::Create)
try {
  Get-ChildItem -LiteralPath $Portable -Recurse -File | ForEach-Object {
    $Relative = $_.FullName.Substring($Portable.Length + 1).Replace('\', '/')
    # Extension methods are not callable on instances in Windows PowerShell 5.1.
    [void][System.IO.Compression.ZipFileExtensions]::CreateEntryFromFile($Archive, $_.FullName, $Relative, [System.IO.Compression.CompressionLevel]::Optimal)
  }
} finally {
  $Archive.Dispose()
  $ZipStream.Dispose()
}

# Guard: the in-app updater rejects entry names containing '\' or ':' (they look like unsafe
# paths to its extractor) and cannot install from an empty archive, so fail the build instead
# of shipping an uninstallable update.
$Check = [System.IO.Compression.ZipFile]::OpenRead($Zip)
try {
  $Unsafe = @($Check.Entries | Where-Object { $_.FullName -match '[\\:]' })
  if ($Unsafe.Count -gt 0) {
    throw "portable zip has $($Unsafe.Count) entries with backslash or colon in their names; the in-app updater would reject them (first: $($Unsafe[0].FullName))"
  }
  if ($Check.Entries.Count -eq 0) {
    throw 'portable zip is empty; the in-app updater cannot install it'
  }
} finally {
  $Check.Dispose()
}

# Publish the portable archive checksum beside the ZIP. The in-app updater requires this value.
$Sha256 = (Get-FileHash -Algorithm SHA256 -Path $Zip).Hash.ToLowerInvariant()
$Sums = Join-Path $Dist 'SHA256SUMS.txt'
"$Sha256  $([System.IO.Path]::GetFileName($Zip))" | Set-Content -Encoding ascii $Sums

# Smoke-test the CLI when this machine can run it. An ARM64 build made on an x64 runner can't run
# here; .github/workflows/windows-arm64.yml installs and runs it on ARM64 instead.
$HostArch = [System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture.ToString().ToLowerInvariant()
if ($Arch -ne 'arm64' -or $HostArch -eq 'arm64') {
  Invoke-Native 'designcraft-cli --version' { & (Join-Path $Stage 'designcraft-cli.exe') --version }
} else {
  Write-Output "skipping designcraft-cli --version: an $Arch build doesn't run on this $HostArch machine"
}
if ($PortableOnly) {
  Get-Item $Zip | Format-Table Name, Length
} else {
  Get-Item $Msi, $Zip | Format-Table Name, Length
}
