<#
.SYNOPSIS
    Install portable PHP and MySQL/MariaDB runtimes into Servisan's `bin/` directory.

.DESCRIPTION
    Dual-mode binary installer:
    1. Fast mode: Automatically reuses binaries from an existing local Laragon installation (C:\laragon or D:\laragon).
    2. Standalone mode: If Laragon is not found (or -DownloadOfficial is specified), downloads official
       clean portable zip archives directly from windows.php.net and official mirrors.

.PARAMETER LaragonRoot
    Optional root of an existing Laragon installation. If omitted, automatically checks D:\laragon and C:\laragon.

.PARAMETER DownloadOfficial
    Force downloading official portable runtimes even if Laragon is installed.

.PARAMETER IncludePhpMyAdmin
    Also download or copy phpMyAdmin into www/phpmyadmin.
#>
[CmdletBinding()]
param(
    [string]$LaragonRoot = '',
    [string]$PhpVersion   = '',
    [string]$MysqlVersion = '',
    [switch]$DownloadOfficial,
    [switch]$IncludePhpMyAdmin
)

$ErrorActionPreference = 'Stop'

$RepoRoot = Split-Path -Parent $PSScriptRoot
$PhpDst   = Join-Path $RepoRoot 'bin\php'
$MysqlDst = Join-Path $RepoRoot 'bin\mysql'
$PmaDst   = Join-Path $RepoRoot 'www\phpmyadmin'

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

    Write-Host "  Copying $Source" -ForegroundColor Cyan
    Write-Host "       -> $Destination"

    $args = @($Source, $Destination, '/E', '/NFL', '/NDL', '/NJH', '/NJS', '/NP', '/R:1', '/W:1')
    if ($ExcludeDirs.Count)  { $args += '/XD'; $args += $ExcludeDirs }
    if ($ExcludeFiles.Count) { $args += '/XF'; $args += $ExcludeFiles }

    & robocopy @args | Out-Null
    if ($LASTEXITCODE -ge 8) {
        throw "robocopy failed with exit code $LASTEXITCODE"
    }
    $global:LASTEXITCODE = 0
}

function Download-And-Extract {
    param(
        [Parameter(Mandatory)][string]$Url,
        [Parameter(Mandatory)][string]$Destination,
        [string]$SubFolder = '',
        [Parameter(Mandatory)][string]$Label
    )

    $tempZip = Join-Path ([System.IO.Path]::GetTempPath()) ("servisan_" + [System.IO.Path]::GetRandomFileName() + ".zip")
    $extractTmp = Join-Path ([System.IO.Path]::GetTempPath()) ("servisan_ext_" + [System.IO.Path]::GetRandomFileName())

    Write-Host "  Downloading $Label..." -ForegroundColor Cyan
    Write-Host "  From: $Url" -ForegroundColor DarkGray
    
    $webClient = New-Object System.Net.WebClient
    $webClient.Headers.Add("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) ServisanInstaller/1.0")
    $webClient.DownloadFile($Url, $tempZip)

    Write-Host "  Extracting $Label..." -ForegroundColor Cyan
    Expand-Archive -LiteralPath $tempZip -DestinationPath $extractTmp -Force

    $sourceDir = $extractTmp
    if ($SubFolder -ne '') {
        $found = Get-ChildItem -Path $extractTmp -Directory | Where-Object { $_.Name -like "$SubFolder*" } | Select-Object -First 1
        if ($found) {
            $sourceDir = $found.FullName
        }
    }

    if (-not (Test-Path -LiteralPath $Destination)) {
        New-Item -ItemType Directory -Path $Destination -Force | Out-Null
    }

    $args = @($sourceDir, $Destination, '/E', '/NFL', '/NDL', '/NJH', '/NJS', '/NP', '/R:1', '/W:1')
    & robocopy @args | Out-Null
    $global:LASTEXITCODE = 0

    Remove-Item -LiteralPath $tempZip -Force -ErrorAction SilentlyContinue
    Remove-Item -LiteralPath $extractTmp -Recurse -Force -ErrorAction SilentlyContinue
    Write-Host "  [OK] $Label installed to $Destination" -ForegroundColor Green
}

# Determine source strategy
$useLaragon = $false
$detectedLaragon = $null

if (-not $DownloadOfficial) {
    if ($LaragonRoot -ne '' -and (Test-Path -LiteralPath $LaragonRoot)) {
        $detectedLaragon = $LaragonRoot
        $useLaragon = $true
    } else {
        foreach ($cand in @('D:\laragon', 'C:\laragon')) {
            if (Test-Path -LiteralPath (Join-Path $cand 'bin\php')) {
                $detectedLaragon = $cand
                $useLaragon = $true
                break
            }
        }
    }
}

Write-Host "=== Servisan Runtime Binary Installer ===" -ForegroundColor Yellow

if ($useLaragon) {
    Write-Host "Found Laragon installation at: $detectedLaragon" -ForegroundColor Green
    
    # Auto-detect PHP folder in Laragon if not explicitly specified
    if ($PhpVersion -eq '') {
        $phpDir = Get-ChildItem -Path (Join-Path $detectedLaragon 'bin\php') -Directory |
            Sort-Object Name -Descending |
            Select-Object -First 1
        if (-not $phpDir) { throw "No PHP versions found in $detectedLaragon\bin\php" }
        $PhpVersion = $phpDir.Name
    }

    # Auto-detect MySQL folder in Laragon if not explicitly specified
    if ($MysqlVersion -eq '') {
        $mysqlDir = Get-ChildItem -Path (Join-Path $detectedLaragon 'bin\mysql') -Directory |
            Sort-Object Name -Descending |
            Select-Object -First 1
        if (-not $mysqlDir) { throw "No MySQL versions found in $detectedLaragon\bin\mysql" }
        $MysqlVersion = $mysqlDir.Name
    }

    $PhpSrc   = Join-Path $detectedLaragon "bin\php\$PhpVersion"
    $MysqlSrc = Join-Path $detectedLaragon "bin\mysql\$MysqlVersion"

    Write-Host "Copying portable PHP ($PhpVersion)..."
    Copy-Tree -Source $PhpSrc -Destination $PhpDst `
        -ExcludeDirs @('dev', 'extras') `
        -ExcludeFiles @('php.ini', 'php.ini-development', 'php.ini-production')

    Write-Host "Copying portable MySQL ($MysqlVersion)..."
    Copy-Tree -Source $MysqlSrc -Destination $MysqlDst `
        -ExcludeDirs @('docs') `
        -ExcludeFiles @('my.ini')

    if ($IncludePhpMyAdmin) {
        $pmaSource = Join-Path $detectedLaragon 'etc\apps\phpmyadmin'
        if (Test-Path -LiteralPath $pmaSource) {
            Write-Host "Copying phpMyAdmin..."
            Copy-Tree -Source $pmaSource -Destination $PmaDst
        }
    }
} else {
    Write-Host "Laragon not detected or -DownloadOfficial was requested." -ForegroundColor Cyan
    Write-Host "Downloading clean official portable runtimes (PHP & MariaDB/MySQL)..." -ForegroundColor Cyan

    # Official Windows PHP (8.3 Thread Safe x64)
    $phpUrl = "https://windows.php.net/downloads/releases/php-8.3.35-Win32-vs16-x64.zip"
    Download-And-Extract -Url $phpUrl -Destination $PhpDst -Label "PHP 8.3"

    # Official MariaDB LTS portable package (Full binary compatibility with MySQL 8.x mysqld / mysqladmin)
    $dbUrl = "https://archive.mariadb.org/mariadb-11.4.5/winx64-packages/mariadb-11.4.5-winx64.zip"
    Download-And-Extract -Url $dbUrl -Destination $MysqlDst -SubFolder "mariadb-" -Label "MySQL/MariaDB Server"

    if ($IncludePhpMyAdmin) {
        $pmaUrl = "https://files.phpmyadmin.net/phpMyAdmin/5.2.2/phpMyAdmin-5.2.2-all-languages.zip"
        Download-And-Extract -Url $pmaUrl -Destination $PmaDst -SubFolder "phpMyAdmin-" -Label "phpMyAdmin"
    }
}

# Bundle Microsoft Visual C++ runtime DLLs (vcruntime140, msvcp140) app-locally
Write-Host "Ensuring Visual C++ runtime DLLs (vcruntime140 / msvcp140) are bundled..." -ForegroundColor Cyan
$vcDlls = @('vcruntime140.dll', 'vcruntime140_1.dll', 'msvcp140.dll', 'msvcp140_1.dll', 'msvcp140_2.dll', 'msvcp140_atomic_wait.dll', 'msvcp140_codecvt_ids.dll', 'vcomp140.dll')
$sys32 = "$env:SystemRoot\System32"
foreach ($dir in @($PhpDst, (Join-Path $MysqlDst 'bin'))) {
    if (Test-Path -LiteralPath $dir) {
        foreach ($dll in $vcDlls) {
            $src = Join-Path $sys32 $dll
            if (Test-Path -LiteralPath $src) {
                Copy-Item -LiteralPath $src -Destination (Join-Path $dir $dll) -Force -ErrorAction SilentlyContinue
            }
        }
    }
}

Write-Host ''
Write-Host "=== Setup Complete! ===" -ForegroundColor Green
Write-Host "PHP   installed to: $PhpDst"
Write-Host "MySQL installed to: $MysqlDst"
if ($IncludePhpMyAdmin) {
    Write-Host "phpMyAdmin  to: $PmaDst"
}
Write-Host ''
Write-Host "Next step: run 'cargo run -- init' or 'servisan.exe init' to initialize databases." -ForegroundColor Yellow
