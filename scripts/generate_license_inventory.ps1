<#
.SYNOPSIS
    Refresh the dependency and asset evidence used by release review.
#>
param([string]$Output = "docs\THIRD_PARTY_LICENSES.md")

$ErrorActionPreference = "Stop"
$gameDir = Split-Path -Parent $PSScriptRoot
Push-Location $gameDir
try {
    $metadata = cargo metadata --manifest-path (Join-Path $gameDir "Cargo.toml") --no-deps --format-version 1 | ConvertFrom-Json
    $packages = @($metadata.packages | Where-Object { $_.name -in @("carriage_run", "macroquad-toolkit") } | ForEach-Object {
        "| $($_.name) | $($_.version) | Cargo metadata; see Cargo.lock |"
    })
    $assets = @(Get-ChildItem assets -Recurse -File | ForEach-Object {
        "| $($_.FullName.Substring($gameDir.Length + 1).Replace('\', '/')) | checked-in | Project asset inventory |"
    })
    $lines = @(
        "# Third-party dependency and asset license inventory"
        ""
        "Generated $(Get-Date -Format 'yyyy-MM-dd') from Cargo metadata and the checked-in asset tree."
        ""
        "## Cargo packages"
        ""
        "| Package | Version | Evidence |"
        "| --- | --- | --- |"
        $packages
        ""
        "## Assets"
        ""
        "| Path | Status | Evidence |"
        "| --- | --- | --- |"
        $assets
    )
    Set-Content -LiteralPath $Output -Value $lines -Encoding utf8
    Write-Host "Wrote $Output"
}
finally {
    Pop-Location
}
