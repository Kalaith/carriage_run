<#
.SYNOPSIS
    Headless screenshot harness for Carriage Run.

.DESCRIPTION
    Builds the debug exe and drives it through the env-var capture hook in
    src/main.rs (CARRIAGE_CAPTURE_*). The game boots straight into a chosen
    scene, simulates a fixed number of frames at a fixed timestep, writes a PNG,
    and exits -- no interactive window or input needed. Each PNG is sanity
    checked so a blank/black capture fails loudly.

.EXAMPLE
    ./scripts/capture_ui.ps1
    ./scripts/capture_ui.ps1 -Scenes gameplay,map -Frames 150
#>
param(
    [string[]]$Scenes = @("gameplay", "map"),
    [int]$Frames = 150,
    [string]$OutputDir = "docs\verification",
    [switch]$SkipBuild
)

$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent $PSScriptRoot
# The shared cargo target lives at the WebHatchery root (two levels above the
# crate), matching the sibling capture scripts.
$targetRoot = Split-Path -Parent (Split-Path -Parent $repoRoot)
$exe = Join-Path $targetRoot ".cargo-target\debug\carriage_run.exe"
$outDir = Join-Path $repoRoot $OutputDir

Set-Location $repoRoot
if (-not $SkipBuild) {
    Write-Host "Building debug exe..."
    cargo build
}
if (-not (Test-Path -LiteralPath $exe)) {
    throw "Missing executable: $exe"
}

New-Item -ItemType Directory -Force -Path $outDir | Out-Null

foreach ($scene in $Scenes) {
    $path = Join-Path $outDir ("ui_{0}.png" -f $scene)
    if (Test-Path -LiteralPath $path) { Remove-Item -LiteralPath $path -Force }

    $env:CARRIAGE_CAPTURE_PATH = $path
    $env:CARRIAGE_CAPTURE_SCENE = $scene
    $env:CARRIAGE_CAPTURE_FRAMES = "$Frames"
    try {
        & $exe
    }
    finally {
        Remove-Item Env:\CARRIAGE_CAPTURE_PATH, Env:\CARRIAGE_CAPTURE_SCENE, Env:\CARRIAGE_CAPTURE_FRAMES -ErrorAction SilentlyContinue
    }

    if (-not (Test-Path -LiteralPath $path)) {
        throw "Capture failed: $path was not created."
    }
    $bytes = (Get-Item -LiteralPath $path).Length
    if ($bytes -lt 40000) {
        throw "Capture failed: $path is only $bytes bytes (likely blank)."
    }
    Write-Host ("Captured {0} ({1} bytes)" -f $path, $bytes)
}
