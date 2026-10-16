<#
.SYNOPSIS
    Build the Windows Installer (.exe setup) for Servisan using Inno Setup.
#>
[CmdletBinding()]
param()

$ErrorActionPreference = 'Stop'
$RepoRoot = Split-Path -Parent $PSScriptRoot

# Locate ISCC.exe (Inno Setup Compiler)
$isccCmd = Get-Command iscc -ErrorAction SilentlyContinue
$iscc = if ($isccCmd) { $isccCmd.Source } else { $null }
if (-not $iscc) {
    $candidates = @(
        "$env:LOCALAPPDATA\Programs\Inno Setup 6\ISCC.exe",
        "C:\Program Files (x86)\Inno Setup 6\ISCC.exe",
        "C:\Program Files\Inno Setup 6\ISCC.exe",
        "C:\Program Files (x86)\Inno Setup 5\ISCC.exe"
    )
    foreach ($cand in $candidates) {
        if (Test-Path -LiteralPath $cand) {
            $iscc = $cand
            break
        }
    }
}

if (-not $iscc) {
    Write-Host "Inno Setup Compiler (ISCC.exe) was not found." -ForegroundColor Yellow
    Write-Host "You can install it automatically via winget:" -ForegroundColor Cyan
    Write-Host "  winget install JRSoftware.InnoSetup --accept-package-agreements --accept-source-agreements"
    Write-Host "Or download manually from: https://jrsoftware.org/isdl.php"
    exit 1
}

# Ensure staging directory exists
$stageDir = Join-Path $RepoRoot "target\package\servisan-portable"
if (-not (Test-Path -LiteralPath $stageDir)) {
    Write-Host "Staging files first via package-portable.ps1..." -ForegroundColor Cyan
    & (Join-Path $PSScriptRoot "package-portable.ps1")
    if ($LASTEXITCODE -ne 0) { throw "package-portable failed" }
}

$issFile = Join-Path $PSScriptRoot "installer.iss"
Write-Host "Compiling installer using $iscc..." -ForegroundColor Cyan
& $iscc $issFile

if ($LASTEXITCODE -ne 0) {
    throw "Inno Setup compilation failed with exit code $LASTEXITCODE"
}

$setupExe = Join-Path $RepoRoot "target\release\servisan-0.1.0-setup.exe"
if (Test-Path -LiteralPath $setupExe) {
    $size = [math]::Round((Get-Item -LiteralPath $setupExe).Length / 1MB, 2)
    $hash = (Get-FileHash -LiteralPath $setupExe -Algorithm SHA256).Hash
    Set-Content -LiteralPath "$setupExe.sha256" -Value "$hash  $(Split-Path $setupExe -Leaf)"
    Write-Host ""
    Write-Host "Installer created successfully!" -ForegroundColor Green
    Write-Host "File   : $setupExe ($size MB)"
    Write-Host "SHA256 : $hash"
}
