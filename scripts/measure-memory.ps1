#requires -version 5.1
<#
.SYNOPSIS
    Measure real RAM usage of a Servisan installation.

.DESCRIPTION
    Collects Private Bytes and Working Set for servisan.exe and its child
    processes (php.exe, mysqld.exe, etc.), samples over time, and prints
    per-process and total statistics.

.PARAMETER Root
    Servisan root folder. Defaults to the parent of this script's folder.

.PARAMETER Seconds
    Sampling duration in seconds. Default 30.

.PARAMETER IntervalMs
    Sampling interval in milliseconds. Default 1000.

.PARAMETER Csv
    Optional path to export samples as CSV.

.PARAMETER Json
    Optional path to export a JSON summary.

.PARAMETER IncludeAll
    Include every php.exe/mysqld.exe on the machine, not only Servisan processes.

.EXAMPLE
    .\scripts\measure-memory.ps1

.EXAMPLE
    .\scripts\measure-memory.ps1 -Seconds 120 -IntervalMs 500 -Csv .\memory.csv
#>
[CmdletBinding()]
param(
    [string]$Root = '',
    [ValidateRange(1, 86400)][int]$Seconds = 30,
    [ValidateRange(100, 60000)][int]$IntervalMs = 1000,
    [string]$Csv = '',
    [string]$Json = '',
    [switch]$IncludeAll
)

$ErrorActionPreference = 'Stop'

if ([string]::IsNullOrWhiteSpace($Root)) {
    $scriptDir = $PSScriptRoot
    if ([string]::IsNullOrWhiteSpace($scriptDir)) {
        $scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
    }
    $Root = Split-Path -Parent $scriptDir
}

function Get-ServisanProcessIds {
    param(
        [string]$Root,
        [switch]$IncludeAll
    )

    $names = @('servisan', 'php', 'php-cgi', 'mysqld', 'mariadbd')
    $ids = [System.Collections.Generic.HashSet[int]]::new()

    foreach ($p in @(Get-Process -Name 'servisan' -ErrorAction SilentlyContinue)) {
        [void]$ids.Add([int]$p.Id)
    }

    $all = @(Get-CimInstance Win32_Process -ErrorAction SilentlyContinue)
    $children = @{}
    foreach ($p in $all) {
        $parent = [int]$p.ParentProcessId
        if (-not $children.ContainsKey($parent)) {
            $children[$parent] = [System.Collections.Generic.List[int]]::new()
        }
        $children[$parent].Add([int]$p.ProcessId)
    }

    $queue = [System.Collections.Generic.Queue[int]]::new()
    foreach ($id in @($ids)) {
        $queue.Enqueue($id)
    }

    while ($queue.Count -gt 0) {
        $current = $queue.Dequeue()
        if ($children.ContainsKey($current)) {
            foreach ($child in $children[$current]) {
                if ($ids.Add($child)) {
                    $queue.Enqueue($child)
                }
            }
        }
    }

    foreach ($p in @(Get-Process -ErrorAction SilentlyContinue)) {
        try {
            if ($p.Path -and $p.Path.StartsWith($Root, [System.StringComparison]::OrdinalIgnoreCase)) {
                [void]$ids.Add([int]$p.Id)
            }
        } catch { }
    }

    if ($IncludeAll) {
        foreach ($name in $names) {
            foreach ($p in @(Get-Process -Name $name -ErrorAction SilentlyContinue)) {
                [void]$ids.Add([int]$p.Id)
            }
        }
    }

    return @($ids)
}

function Get-Snapshot {
    param([int[]]$Ids)

    $rows = [System.Collections.Generic.List[object]]::new()
    foreach ($id in $Ids) {
        $p = Get-Process -Id $id -ErrorAction SilentlyContinue
        if (-not $p) {
            continue
        }

        $rows.Add([pscustomobject]@{
            Name         = $p.ProcessName
            PID          = $p.Id
            PrivateMB    = [math]::Round($p.PrivateMemorySize64 / 1MB, 1)
            WorkingSetMB = [math]::Round($p.WorkingSet64 / 1MB, 1)
            Path         = $p.Path
        })
    }

    return @($rows)
}

function Get-Totals {
    param($Rows)

    $arr = @($Rows)
    $private = 0.0
    $working = 0.0

    foreach ($r in $arr) {
        $private += [double]$r.PrivateMB
        $working += [double]$r.WorkingSetMB
    }

    return [pscustomobject]@{
        Processes    = $arr.Count
        PrivateMB    = [math]::Round($private, 1)
        WorkingSetMB = [math]::Round($working, 1)
    }
}

if (-not (Test-Path -LiteralPath $Root)) {
    throw "Servisan root not found: $Root"
}

Write-Host ''
Write-Host 'Servisan memory measurement' -ForegroundColor Cyan
Write-Host ("Root   : {0}" -f $Root)
Write-Host ("Window : {0}s, interval {1}ms" -f $Seconds, $IntervalMs)
Write-Host ''

$ids = @(Get-ServisanProcessIds -Root $Root -IncludeAll:$IncludeAll)
if ($ids.Count -eq 0) {
    Write-Host 'No Servisan processes found.' -ForegroundColor Yellow
    Write-Host 'Start Servisan first (servisan.exe, or cargo run -- start), then run this script again.'
    exit 1
}

$first = @(Get-Snapshot -Ids $ids)
$firstTotals = Get-Totals -Rows $first

Write-Host 'Current per-process snapshot' -ForegroundColor Green
$first | Format-Table Name, PID, PrivateMB, WorkingSetMB, Path -AutoSize

Write-Host 'Current total' -ForegroundColor Green
$firstTotals | Format-List

$samples = [System.Collections.Generic.List[object]]::new()
$deadline = (Get-Date).AddSeconds($Seconds)

Write-Host ("Sampling for {0}s..." -f $Seconds) -ForegroundColor Cyan
do {
    $rows = @(Get-Snapshot -Ids $ids)
    $totals = Get-Totals -Rows $rows

    $samples.Add([pscustomobject]@{
        Time         = (Get-Date).ToString('yyyy-MM-dd HH:mm:ss')
        Processes    = $totals.Processes
        PrivateMB    = $totals.PrivateMB
        WorkingSetMB = $totals.WorkingSetMB
    })

    Start-Sleep -Milliseconds $IntervalMs
} while ((Get-Date) -lt $deadline)

Write-Host ''
Write-Host 'Summary' -ForegroundColor Green

$summary = [pscustomobject]@{
    Samples         = $samples.Count
    PrivateMinMB    = ($samples | Measure-Object PrivateMB -Minimum).Minimum
    PrivateMaxMB    = ($samples | Measure-Object PrivateMB -Maximum).Maximum
    PrivateAvgMB    = [math]::Round(($samples | Measure-Object PrivateMB -Average).Average, 1)
    WorkingSetMinMB = ($samples | Measure-Object WorkingSetMB -Minimum).Minimum
    WorkingSetMaxMB = ($samples | Measure-Object WorkingSetMB -Maximum).Maximum
    WorkingSetAvgMB = [math]::Round(($samples | Measure-Object WorkingSetMB -Average).Average, 1)
}

$summary | Format-List

Write-Host 'Metrics:' -ForegroundColor DarkGray
Write-Host '  PrivateMB    = private committed memory (Private Bytes).' -ForegroundColor DarkGray
Write-Host '  WorkingSetMB = resident physical pages, including shared DLLs.' -ForegroundColor DarkGray
Write-Host 'Virtual address space is intentionally not used; it is not physical RAM.' -ForegroundColor DarkGray

if ($Csv) {
    $csvDir = Split-Path -Parent $Csv
    if ($csvDir -and -not (Test-Path -LiteralPath $csvDir)) {
        New-Item -ItemType Directory -Path $csvDir -Force | Out-Null
    }
    $samples | Export-Csv -LiteralPath $Csv -NoTypeInformation -Encoding UTF8
    Write-Host ("CSV written: {0}" -f $Csv) -ForegroundColor Green
}

if ($Json) {
    $jsonDir = Split-Path -Parent $Json
    if ($jsonDir -and -not (Test-Path -LiteralPath $jsonDir)) {
        New-Item -ItemType Directory -Path $jsonDir -Force | Out-Null
    }

    $payload = [pscustomobject]@{
        Root      = $Root
        Generated = (Get-Date).ToString('o')
        Snapshot  = $first
        Summary   = $summary
        Samples   = $samples
    }

    $payload | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath $Json -Encoding UTF8
    Write-Host ("JSON written: {0}" -f $Json) -ForegroundColor Green
}
