<#
.SYNOPSIS
    Build a self-contained, portable Servisan release for Windows end users.
#>
[CmdletBinding()]
param()

$ErrorActionPreference = 'Stop'
$RepoRoot = Split-Path -Parent $PSScriptRoot
$StageRoot = Join-Path $RepoRoot "target\package"
$Stage = Join-Path $StageRoot "servisan-portable"
$Zip = $null

function Get-RepoVersion {
    $line = Select-String -Path (Join-Path $RepoRoot 'Cargo.toml') -Pattern '^version\s*=' | Select-Object -First 1
    if (-not $line -or -not ($line.Line -match '"(.*)"')) {
        throw "Could not discover version from Cargo.toml"
    }
    return $Matches[1]
}

function Copy-Tree {
    param([string]$Source, [string]$Destination, [string[]]$Exclude = @())
    if (-not (Test-Path -LiteralPath $Source)) { return }
    if (-not (Test-Path -LiteralPath $Destination)) {
        New-Item -ItemType Directory -Path $Destination -Force | Out-Null
    }
    $args = @($Source, $Destination, '/E', '/NFL', '/NDL', '/NJH', '/NJS', '/NP', '/R:1', '/W:1')
    if ($Exclude.Count) { $args += '/XD'; $args += $Exclude }
    & robocopy @args | Out-Null
    if ($LASTEXITCODE -ge 8) { throw "robocopy failed on $Source ($LASTEXITCODE)" }
    $global:LASTEXITCODE = 0
}

$version = Get-RepoVersion
$Zip = Join-Path (Split-Path $RepoRoot -Leaf) "target\release\servisan-${version}-portable.zip"
$Zip = Join-Path $RepoRoot "target\release\servisan-${version}-portable.zip"

Write-Host "Version: $version"

# Build the app binary if needed.
$builtExe = Join-Path $RepoRoot 'target\release\servisan.exe'
if (-not (Test-Path -LiteralPath $builtExe)) {
    Write-Host 'Building release binary...' -ForegroundColor Cyan
    & cargo build --release --manifest-path (Join-Path $RepoRoot 'Cargo.toml')
    if ($LASTEXITCODE -ne 0) { throw "cargo build failed" }
}

# Install PHP and MySQL binaries into the repo tree if missing.
if (-not (Test-Path -LiteralPath (Join-Path $RepoRoot 'bin\php\php.exe')) -or
    -not (Test-Path -LiteralPath (Join-Path $RepoRoot 'bin\mysql\bin\mysqld.exe'))) {
    Write-Host 'Installing PHP and MySQL runtimes...' -ForegroundColor Cyan
    & (Join-Path $RepoRoot 'scripts\install-binaries.ps1')
    if ($LASTEXITCODE -ne 0) { throw "install-binaries failed" }
}

if (Test-Path -LiteralPath $StageRoot) {
    Remove-Item -LiteralPath $StageRoot -Recurse -Force
}
New-Item -ItemType Directory -Path $Stage -Force | Out-Null

Write-Host "Staging release under $Stage" -ForegroundColor Cyan

Copy-Item -LiteralPath $builtExe -Destination (Join-Path $Stage 'servisan.exe')
Copy-Item -LiteralPath (Join-Path $RepoRoot 'README.md') -Destination $Stage -ErrorAction SilentlyContinue
Copy-Item -LiteralPath (Join-Path $RepoRoot 'LICENSE') -Destination $Stage -ErrorAction SilentlyContinue
Copy-Tree -Source (Join-Path $RepoRoot 'assets') -Destination (Join-Path $Stage 'assets')
Copy-Tree -Source (Join-Path $RepoRoot 'bin') -Destination (Join-Path $Stage 'bin')
Copy-Tree -Source (Join-Path $RepoRoot 'config') -Destination (Join-Path $Stage 'config')
Copy-Tree -Source (Join-Path $RepoRoot 'www') -Destination (Join-Path $Stage 'www')

# Empty runtime folders. Paired data/logs remain under the portable root.
New-Item -ItemType Directory -Path (Join-Path $Stage 'data') -Force | Out-Null
New-Item -ItemType Directory -Path (Join-Path $Stage 'logs') -Force | Out-Null
New-Item -ItemType Directory -Path (Join-Path $Stage 'config\generated') -Force | Out-Null
New-Item -ItemType File -Path (Join-Path $Stage 'www\.gitkeep') -Force | Out-Null
New-Item -ItemType Directory -Path (Join-Path $Stage 'logs') -Force | Out-Null

if (Test-Path -LiteralPath (Join-Path $RepoRoot 'scripts')) {
    Copy-Tree -Source (Join-Path $RepoRoot 'scripts') -Destination (Join-Path $Stage 'scripts')
}

# Bundle Microsoft Visual C++ runtime DLLs (vcruntime140, msvcp140) app-locally so target machines do not require VC++ Redistributable.
Write-Host "Bundling Visual C++ runtime DLLs (App-Local)..." -ForegroundColor Cyan
$vcDlls = @('vcruntime140.dll', 'vcruntime140_1.dll', 'msvcp140.dll', 'msvcp140_1.dll', 'msvcp140_2.dll', 'msvcp140_atomic_wait.dll', 'msvcp140_codecvt_ids.dll', 'vcomp140.dll')
$sys32 = "$env:SystemRoot\System32"
foreach ($dir in @($Stage, (Join-Path $Stage 'bin\php'), (Join-Path $Stage 'bin\mysql\bin'))) {
    if (Test-Path -LiteralPath $dir) {
        foreach ($dll in $vcDlls) {
            $src = Join-Path $sys32 $dll
            if (Test-Path -LiteralPath $src) {
                Copy-Item -LiteralPath $src -Destination (Join-Path $dir $dll) -Force -ErrorAction SilentlyContinue
            }
        }
    }
}

# Cache-free release package: remove unused repository metadata.
Get-ChildItem -LiteralPath $Stage -Recurse -Filter '*.pdb' -ErrorAction SilentlyContinue |
    Remove-Item -Force -ErrorAction SilentlyContinue

Write-Host "Reading data volume..."
$size = [math]::Round(((Get-ChildItem $Stage -Recurse -File | Measure-Object Length -Sum).Sum /1MB), 1)
Write-Host "Staged: $size MB"

if (Test-Path -LiteralPath $Zip) { Remove-Item -LiteralPath $Zip -Force }
Write-Host "Compressing $Zip..." -ForegroundColor Cyan
Compress-Archive -Path (Join-Path $Stage '*') -DestinationPath $Zip -Force

$hash = (Get-FileHash -LiteralPath $Zip -Algorithm SHA256).Hash
Set-Content -LiteralPath (Join-Path $RepoRoot "target\release\$(Split-Path $Zip -Leaf).sha256") -Value "$hash  $(Split-Path $Zip -Leaf)"
Write-Host '' -ForegroundColor Green
Write-Host 'Done.' -ForegroundColor Green
Write-Output "Release: $Zip"
Write-Output "SHA256 : $hash"
