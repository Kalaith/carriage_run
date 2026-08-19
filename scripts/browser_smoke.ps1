<#
.SYNOPSIS
    Capture the release UI at desktop, touch-sized, and fullscreen viewports.

.DESCRIPTION
    The shared capture harness owns the game process and scene seeding. This
    wrapper runs the same scene manifest at three supported browser sizes and
    keeps the evidence flat in docs/verification as required by the project
    release checklist.
#>
param(
    [int]$Frames = 90,
    [switch]$SkipBuild,
    [switch]$Visible
)

$ErrorActionPreference = "Stop"
$gameDir = Split-Path -Parent $PSScriptRoot
$shared = Join-Path (Split-Path -Parent $gameDir) "macroquad-toolkit\scripts\capture_ui.ps1"
$scenes = @("title", "map", "loadout", "shop", "guards", "upgrades", "settings", "gameplay")
$outDir = Join-Path $gameDir "docs\verification"

foreach ($viewport in @(
    @{ Name = "desktop"; Width = 1280; Height = 720 },
    @{ Name = "touch"; Width = 960; Height = 540 },
    @{ Name = "fullscreen"; Width = 1920; Height = 1080 }
)) {
    & $shared -GameDir $gameDir -Prefix "CARRIAGE" -Scenes $scenes -Frames $Frames -WindowWidth $viewport.Width -WindowHeight $viewport.Height -OutputDir "docs\verification" -Release -SkipBuild:$SkipBuild -Visible:$Visible
    if ($LASTEXITCODE -ne 0) { throw "Capture failed for $($viewport.Name)" }

    foreach ($scene in $scenes) {
        $source = Join-Path $outDir "ui_$scene.png"
        $destination = Join-Path $outDir "smoke_$($viewport.Name)_$scene.png"
        if (-not (Test-Path -LiteralPath $source)) { throw "Missing capture $source" }
        Move-Item -LiteralPath $source -Destination $destination -Force
    }
}

Write-Host "Browser smoke captures written to $outDir"
