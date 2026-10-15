<#
.SYNOPSIS
    Build Servisan in release mode and place the binary at the repo root.

.DESCRIPTION
    The PRD directory layout expects `servisan.exe` next to `bin/`, `www/`,
    `data/` and `config/`. Cargo only writes to `target/release/`, so this
    script builds and copies the binary to the repository root.

.EXAMPLE
    .\scripts\build-release.ps1
#>
[CmdletBinding()]
param()

$ErrorActionPreference = 'Stop'

$RepoRoot = Split-Path -Parent $PSScriptRoot
$BuiltExe = Join-Path $RepoRoot 'target\release\servisan.exe'
$RootExe  = Join-Path $RepoRoot 'servisan.exe'

Write-Host 'Building (release) ...' -ForegroundColor Cyan
& cargo build --release --manifest-path (Join-Path $RepoRoot 'Cargo.toml')
if ($LASTEXITCODE -ne 0) {
    throw "cargo build failed with exit code $LASTEXITCODE"
}

if (-not (Test-Path -LiteralPath $BuiltExe)) {
    throw "Built binary not found: $BuiltExe"
}

Copy-Item -LiteralPath $BuiltExe -Destination $RootExe -Force

$size = [math]::Round((Get-Item -LiteralPath $RootExe).Length / 1MB, 2)
Write-Host ''
Write-Host 'Done.' -ForegroundColor Green
Write-Host "Binary : $RootExe ($size MB)"
