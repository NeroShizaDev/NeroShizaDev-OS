param(
    [string]$ProjectRoot = (Split-Path $PSScriptRoot -Parent)
)

# Patch: bootloader-x86_64-common - PageAlreadyMapped
# VirtualBox UEFI already occupies the GDT frame -> bootloader panics.
# Fix: ignore PageAlreadyMapped (frame already identity-mapped, that is fine).

$commonSrc = Get-ChildItem "$env:USERPROFILE\.cargo\registry\src" -Recurse -Filter "lib.rs" |
    Where-Object { $_.FullName -like "*bootloader-x86_64-common-0.11.15*" } |
    Select-Object -First 1

if (-not $commonSrc) {
    Write-Host "[patches] bootloader-x86_64-common not found in cargo cache, skip"
    exit 0
}

$target  = $commonSrc.FullName
$content = [System.IO.File]::ReadAllText($target)
$marker  = "PageAlreadyMapped(_)) => {}"

if ($content.Contains($marker)) {
    Write-Host "[patches] bootloader-x86_64-common: already patched, skip"
    exit 0
}

$patchSrc = "$ProjectRoot\tools\patches\bootloader-x86_64-common\src\lib.rs"
if (-not (Test-Path $patchSrc)) {
    Write-Host "[patches] ERROR: patch source not found: $patchSrc"
    exit 1
}

$patched = [System.IO.File]::ReadAllText($patchSrc)
if (-not $patched.Contains($marker)) {
    Write-Host "[patches] ERROR: patch source missing expected marker"
    exit 1
}

[System.IO.File]::WriteAllText($target, $patched, [System.Text.Encoding]::UTF8)
Write-Host "[patches] bootloader-x86_64-common: patched OK"

$base    = "$ProjectRoot\tools\image_builder\target\x86_64-pc-windows-msvc\debug"
$bDir    = "$base\build"
$fpDir   = "$base\.fingerprint"

if (Test-Path $bDir) {
    Get-ChildItem $bDir -Directory |
        Where-Object { $_.Name -like "bootloader-*" } |
        ForEach-Object { Remove-Item $_.FullName -Recurse -Force -ErrorAction SilentlyContinue }
}
if (Test-Path $fpDir) {
    Get-ChildItem $fpDir -Directory |
        Where-Object { $_.Name -like "bootloader-*" } |
        ForEach-Object { Remove-Item $_.FullName -Recurse -Force -ErrorAction SilentlyContinue }
}

Write-Host "[patches] build cache cleared, bootloader will recompile"
