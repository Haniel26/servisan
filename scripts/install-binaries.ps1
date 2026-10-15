<#
.SYNOPSIS
    Copy portable PHP and MySQL/MariaDB binaries from an existing Laragon
    installation into Servisan's local `bin/` directory.

.DESCRIPTION
    Implements the PRD "Binary Reuse Strategy": reuses Laragon's portable
    binaries but keeps Servisan fully self-contained. Large binaries are
    intentionally git-ignored; this script regenerates them.

.PARAMETER LaragonRoot
    Root of the existing Laragon installation. Defaults to D:\laragon.

.PARAMETER PhpVersion
    PHP build folder name inside <LaragonRoot>\bin\php.

.PARAMETER MysqlVersion
    MySQL build folder name inside <LaragonRoot>\bin\mysql.

.EXAMPLE
    .\scripts\install-binaries.ps1
    .\scripts\install-binaries.ps1 -LaragonRoot C:\laragon -PhpVersion php-8.3.30-Win32-vs16-x64
#>
[CmdletBinding()]
param(
    [string]$LaragonRoot = 'D:\laragon',
    [string]$PhpVersion   = 'php-8.3.30-Win32-vs16-x64',
    [string]$MysqlVersion = 'mysql-8.4.3-winx64'
)

$ErrorActionPreference = 'Stop'

$RepoRoot = Split-Path -Parent $PSScriptRoot
$PhpSrc   = Join-Path $LaragonRoot "bin\php\$PhpVersion"
$MysqlSrc = Join-Path $LaragonRoot "bin\mysql\$MysqlVersion"
$PhpDst   = Join-Path $RepoRoot 'bin\php'
$MysqlDst = Join-Path $RepoRoot 'bin\mysql'

function Copy-Tree {
    param(
        [Parameter(Mandatory)][string]$Source,
        [Parameter(Mandatory)][string]$Destination,
        [string[]]$ExcludeDirs = @(),
        [string[]]$ExcludeFiles = @()
    )

    if (-not (Test-Path -LiteralPath $Source)) {
        throw "Source not found: $Source"
    }
    if (-not (Test-Path -LiteralPath $Destination)) {
        New-Item -ItemType Directory -Path $Destination -Force | Out-Null
    }

    Write-Host "Copying $Source" -ForegroundColor Cyan
    Write-Host "     -> $Destination"

    $args = @($Source, $Destination, '/E', '/NFL', '/NDL', '/NJH', '/NJS', '/NP', '/R:1', '/W:1')
    if ($ExcludeDirs.Count)  { $args += '/XD'; $args += $ExcludeDirs }
    if ($ExcludeFiles.Count) { $args += '/XF'; $args += $ExcludeFiles }

    & robocopy @args | Out-Null

    # robocopy: 0-7 = success (bit flags), >= 8 = failure
    if ($LASTEXITCODE -ge 8) {
        throw "robocopy failed with exit code $LASTEXITCODE"
    }
    $global:LASTEXITCODE = 0
}

Copy-Tree -Source $PhpSrc -Destination $PhpDst `
    -ExcludeDirs @('dev', 'extras') `
    -ExcludeFiles @('php.ini', 'php.ini-development', 'php.ini-production')

Copy-Tree -Source $MysqlSrc -Destination $MysqlDst `
    -ExcludeDirs @('docs') `
    -ExcludeFiles @('my.ini')

Write-Host ''
Write-Host 'Done.' -ForegroundColor Green
Write-Host "PHP   : $PhpSrc"
Write-Host "MySQL : $MysqlSrc"
