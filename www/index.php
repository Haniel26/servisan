<?php

declare(strict_types=1);

// Handle phpinfo request
if (isset($_GET['phpinfo'])) {
    phpinfo();
    exit;
}

// The Rust manager injects these so the dashboard reflects app.toml
// and Cargo.toml instead of keeping a second copy of the values.
$mysqlHost = getenv('SERVISAN_MYSQL_HOST') ?: '127.0.0.1';
$mysqlPort = (int) (getenv('SERVISAN_MYSQL_PORT') ?: 3307);
$mysqlUser = 'root';
$mysqlPass = '';
$servisanVersion = getenv('SERVISAN_VERSION') ?: 'dev';

function check_mysql(string $host, int $port, string $user, string $pass): array
{
    if (class_exists('mysqli')) {
        // PHP 8.1+ throws on connection failure instead of setting connect_errno.
        try {
            $conn = @new mysqli($host, $user, $pass, null, $port);
            if ($conn->connect_errno) {
                return [false, 'mysqli: ' . $conn->connect_error];
            }
            $version = (string) $conn->server_info;
            $conn->close();
            return [true, $version];
        } catch (Throwable $e) {
            return [false, 'mysqli: ' . $e->getMessage()];
        }
    }

    if (class_exists('PDO') && in_array('mysql', PDO::getAvailableDrivers(), true)) {
        try {
            $pdo = new PDO("mysql:host={$host};port={$port}", $user, $pass);
            $version = (string) $pdo->query('SELECT VERSION()')->fetchColumn();
            return [true, $version];
        } catch (PDOException $e) {
            return [false, 'PDO: ' . $e->getMessage()];
        }
    }

    return [false, 'no mysqli or pdo_mysql extension loaded'];
}

[$mysqlOk, $mysqlInfo] = check_mysql($mysqlHost, $mysqlPort, $mysqlUser, $mysqlPass);

$mysqlDisplayName = 'MySQL 8.4';
if ($mysqlOk) {
    if (stripos($mysqlInfo, 'mariadb') !== false) {
        preg_match('/(\d+\.\d+(\.\d+)?)/', $mysqlInfo, $m);
        $mysqlDisplayName = 'MariaDB ' . ($m[1] ?? '11.4');
    } else {
        preg_match('/(\d+\.\d+(\.\d+)?)/', $mysqlInfo, $m);
        $mysqlDisplayName = 'MySQL ' . ($m[1] ?? '8.4');
    }
} else {
    $mysqlDisplayName = 'MySQL / MariaDB';
}

$pmaInstalled = is_dir(__DIR__ . '/phpmyadmin');
$docRoot = $_SERVER['DOCUMENT_ROOT'] ?? __DIR__;

// Handle open_root action (open Windows File Explorer directly)
if (isset($_GET['action']) && $_GET['action'] === 'open_root') {
    if (PHP_OS_FAMILY === 'Windows') {
        pclose(popen('explorer ' . escapeshellarg($docRoot), 'r'));
    }
    header('Content-Type: application/json');
    echo json_encode(['ok' => true]);
    exit;
}

// Handle open_terminal action (open Command Prompt with servisan shell)
if (isset($_GET['action']) && $_GET['action'] === 'open_terminal') {
    if (PHP_OS_FAMILY === 'Windows') {
        $rootDir = dirname($docRoot);
        $candidates = [
            $rootDir . DIRECTORY_SEPARATOR . 'servisan.exe',
            $rootDir . DIRECTORY_SEPARATOR . 'target' . DIRECTORY_SEPARATOR . 'release' . DIRECTORY_SEPARATOR . 'servisan.exe',
            $rootDir . DIRECTORY_SEPARATOR . 'target' . DIRECTORY_SEPARATOR . 'debug' . DIRECTORY_SEPARATOR . 'servisan.exe',
        ];
        $exe = 'servisan';
        foreach ($candidates as $candidate) {
            if (file_exists($candidate)) {
                $exe = '"' . $candidate . '"';
                break;
            }
        }
        $cmd = 'start "Servisan Shell" cmd.exe /k "cd /d "' . $rootDir . '" && ' . $exe . ' shell"';
        pclose(popen($cmd, 'r'));
    }
    header('Content-Type: application/json');
    echo json_encode(['ok' => true]);
    exit;
}

// Handle open_logs action (open logs folder in Windows File Explorer)
if (isset($_GET['action']) && $_GET['action'] === 'open_logs') {
    if (PHP_OS_FAMILY === 'Windows') {
        $logsDir = dirname($docRoot) . DIRECTORY_SEPARATOR . 'logs';
        if (!is_dir($logsDir)) {
            @mkdir($logsDir, 0777, true);
        }
        pclose(popen('explorer ' . escapeshellarg($logsDir), 'r'));
    }
    header('Content-Type: application/json');
    echo json_encode(['ok' => true]);
    exit;
}

// Helper to tail the last N lines of a log file
function tail_file(string $filepath, int $lines = 50): string {
    if (!file_exists($filepath)) {
        return "Log file not found: " . basename($filepath);
    }
    $file = @fopen($filepath, "r");
    if (!$file) {
        return "Cannot open log file: " . basename($filepath);
    }
    $buffer = [];
    while (($line = fgets($file)) !== false) {
        $buffer[] = $line;
        if (count($buffer) > $lines) {
            array_shift($buffer);
        }
    }
    fclose($file);
    if (empty($buffer)) {
        return "Log file is currently empty (" . basename($filepath) . ").";
    }
    return implode("", $buffer);
}

// Handle get_logs action (return last 50 lines in JSON)
if (isset($_GET['action']) && $_GET['action'] === 'get_logs') {
    $type = $_GET['type'] ?? 'php';
    $logsDir = dirname($docRoot) . DIRECTORY_SEPARATOR . 'logs';
    $filename = match ($type) {
        'mysql' => 'mysql.log',
        'servisan' => 'servisan.log',
        default => 'php.log',
    };
    $content = tail_file($logsDir . DIRECTORY_SEPARATOR . $filename, 50);
    header('Content-Type: application/json');
    echo json_encode([
        'ok' => true,
        'type' => $type,
        'filename' => $filename,
        'content' => $content,
    ]);
    exit;
}

$extensions = get_loaded_extensions();
sort($extensions);

?>
<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Servisan - Local Development Server</title>
<link rel="icon" href="/favicon.ico" sizes="32x32" type="image/x-icon">
<style>
    *, *::before, *::after {
        box-sizing: border-box;
    }
    body {
        margin: 0;
        padding: 0;
        background-color: #f0f2f5;
        font-family: 'Segoe UI', -apple-system, BlinkMacSystemFont, Roboto, Helvetica, Arial, sans-serif;
        color: #2d3748;
        display: flex;
        flex-direction: column;
        min-height: 100vh;
        user-select: none;
    }

    /* Log output and modal text are meant to be read and copied. */
    #logConsole,
    .modal-body {
        user-select: text;
    }

    /* Main Area & Footer */
    .app-main {
        flex: 1;
        display: flex;
        justify-content: center;
        align-items: center;
        padding: 32px 16px;
    }
    .app-footer {
        padding: 12px 20px 24px 20px;
        font-size: 13px;
        color: #64748b;
        letter-spacing: -0.1px;
        display: flex;
        justify-content: center;
        align-items: center;
        gap: 6px;
    }
    .app-footer a {
        color: #475569;
        text-decoration: none;
        font-weight: 500;
        transition: color 0.15s;
    }
    .app-footer a:hover {
        color: #0284c7;
        text-decoration: underline;
    }

    /* Laragon Window Container */
    .laragon-window {
        width: 780px;
        max-width: 95vw;
        height: 480px;
        background: #ffffff;
        border: 1px solid #d2d6dc;
        border-radius: 8px;
        box-shadow: 0 12px 30px rgba(0, 0, 0, 0.08), 0 4px 12px rgba(0, 0, 0, 0.04);
        display: flex;
        flex-direction: column;
        justify-content: space-between;
        position: relative;
        overflow: hidden;
    }

    /* Top Bar */
    .top-bar {
        height: 52px;
        padding: 8px 16px;
        display: flex;
        justify-content: space-between;
        align-items: center;
        border-bottom: 1px solid #edf2f7;
    }
    .top-left {
        display: flex;
        align-items: center;
        gap: 12px;
        position: relative;
    }
    .brand-logo {
        width: 34px;
        height: 34px;
        border-radius: 6px;
        object-fit: contain;
    }
    .brand-name {
        font-size: 17px;
        font-weight: 700;
        color: #0f172a;
        letter-spacing: -0.3px;
        margin-right: 4px;
    }
    .menu-btn {
        font-size: 14px;
        color: #475569;
        cursor: pointer;
        padding: 4px 8px;
        border-radius: 4px;
        font-weight: 500;
        transition: background 0.15s;
    }
    .menu-btn:hover {
        background: #edf2f7;
        color: #0f172a;
    }

    /* Dropdown Menu */
    .dropdown-menu {
        display: none;
        position: absolute;
        top: calc(100% + 6px);
        right: 0;
        left: auto;
        background: #ffffff;
        border: 1px solid #e2e8f0;
        border-radius: 6px;
        box-shadow: 0 10px 15px -3px rgba(0, 0, 0, 0.1);
        width: 210px;
        z-index: 100;
        padding: 6px 0;
    }
    .dropdown-menu.show {
        display: block;
    }
    .dropdown-item {
        display: flex;
        align-items: center;
        gap: 8px;
        padding: 8px 16px;
        font-size: 13px;
        color: #4a5568;
        text-decoration: none;
        cursor: pointer;
    }
    .dropdown-item:hover {
        background: #f7fafc;
        color: #2b6cb0;
    }
    .dropdown-divider {
        height: 1px;
        background: #edf2f7;
        margin: 4px 0;
    }

    .top-right {
        display: flex;
        align-items: center;
        gap: 8px;
    }
    .badge-version {
        font-size: 12px;
        font-weight: 600;
        color: #0284c7;
        background: #e0f2fe;
        padding: 3px 10px;
        border-radius: 9999px;
        letter-spacing: 0.3px;
        transition: all 0.2s ease;
        user-select: none;
        display: inline-flex;
        align-items: center;
        gap: 6px;
    }
    .badge-version.has-update {
        background: #eff6ff;
        color: #1d4ed8;
        border: 1px solid #bfdbfe;
        cursor: pointer;
        animation: pulseUpdate 2.4s infinite;
    }
    .badge-version.has-update:hover {
        background: #dbeafe;
        color: #1e40af;
    }
    .update-pulse-dot {
        width: 7px;
        height: 7px;
        border-radius: 50%;
        background: #2563eb;
        display: inline-block;
    }
    @keyframes pulseUpdate {
        0%, 100% { box-shadow: 0 0 0 0 rgba(37, 99, 235, 0.4); }
        50% { box-shadow: 0 0 0 4px rgba(37, 99, 235, 0); }
    }
    .icon-link {
        color: #64748b;
        display: inline-flex;
        align-items: center;
        justify-content: center;
        width: 32px;
        height: 32px;
        border-radius: 6px;
        text-decoration: none;
        transition: all 0.15s ease;
    }
    .icon-link:hover {
        color: #0f172a;
        background: #f1f5f9;
    }
    .btn-ghost {
        background: transparent;
        border: none;
        cursor: pointer;
        padding: 0;
    }

    /* Services Body */
    .services-body {
        flex: 1;
        padding: 28px 48px;
        display: flex;
        flex-direction: column;
        gap: 22px;
    }
    .service-row {
        display: grid;
        grid-template-columns: 360px 100px 1fr;
        align-items: center;
        font-size: 16px;
        color: #2d3748;
    }
    .service-info {
        display: flex;
        align-items: center;
        gap: 16px;
    }
    
    /* Clean Monochrome Service Badge */
    .service-badge {
        display: inline-flex;
        align-items: center;
        justify-content: center;
        width: 32px;
        height: 32px;
        border-radius: 6px;
        background: #f1f5f9;
        color: #475569;
        flex-shrink: 0;
        user-select: none;
    }
    .service-badge svg {
        width: 18px;
        height: 18px;
    }
    .service-badge.offline {
        color: #94a3b8;
        background: #f8fafc;
    }

    .service-name {
        font-weight: 500;
        font-size: 16px;
        color: #1a202c;
        display: flex;
        align-items: center;
        gap: 6px;
    }
    .lock-icon {
        color: #a0aec0;
        font-size: 14px;
    }
    .service-port {
        font-size: 16px;
        color: #1a202c;
    }
    .reload-action a {
        color: #0044cc;
        text-decoration: none;
        font-size: 15px;
        font-weight: 500;
    }
    .reload-action a:hover {
        text-decoration: underline;
    }

    .tray-note {
        margin-top: auto;
        padding: 9px 14px;
        background: #f8fafc;
        border: 1px dashed #cbd5e1;
        border-radius: 6px;
        display: flex;
        align-items: center;
        gap: 10px;
        font-size: 13px;
        color: #64748b;
    }
    .tray-note svg {
        flex-shrink: 0;
        color: #0284c7;
    }
    .tray-note strong {
        color: #334155;
    }

    /* Bottom Bar */
    .bottom-bar {
        padding: 16px 20px;
        display: grid;
        grid-template-columns: repeat(4, 1fr);
        gap: 12px;
        background: #ffffff;
        border-top: 1px solid #edf2f7;
    }
    .btn-dock {
        background: #ffffff;
        border: 1px solid #cbd5e1;
        border-radius: 6px;
        padding: 8px 12px;
        font-size: 14px;
        font-weight: 500;
        color: #334155;
        cursor: pointer;
        display: flex;
        align-items: center;
        justify-content: center;
        gap: 8px;
        text-decoration: none;
        box-shadow: 0 1px 2px rgba(0, 0, 0, 0.04);
        transition: all 0.15s ease-in-out;
    }
    .btn-dock:hover {
        background: #f8fafc;
        border-color: #94a3b8;
        color: #0f172a;
    }
    .btn-dock:active {
        background: #f1f5f9;
        transform: translateY(1px);
    }
    .btn-dock svg {
        flex-shrink: 0;
    }

    /* Modal */
    .modal-backdrop {
        display: none;
        position: fixed;
        top: 0;
        left: 0;
        width: 100%;
        height: 100%;
        background: rgba(15, 23, 42, 0.4);
        backdrop-filter: blur(2px);
        z-index: 200;
        justify-content: center;
        align-items: center;
    }
    .modal-backdrop.show {
        display: flex;
    }
    .modal-box {
        background: #ffffff;
        border-radius: 8px;
        width: 480px;
        max-width: 90vw;
        padding: 24px;
        box-shadow: 0 20px 25px -5px rgba(0, 0, 0, 0.1), 0 10px 10px -5px rgba(0, 0, 0, 0.04);
        position: relative;
    }
    .modal-title {
        font-size: 18px;
        font-weight: 600;
        margin-top: 0;
        margin-bottom: 12px;
        color: #0f172a;
    }
    .modal-body {
        font-size: 14px;
        color: #475569;
        line-height: 1.5;
    }
    .code-box {
        background: #0f172a;
        color: #38bdf8;
        padding: 12px 16px;
        border-radius: 6px;
        font-family: Consolas, 'Courier New', monospace;
        font-size: 14px;
        margin: 12px 0;
        display: flex;
        justify-content: space-between;
        align-items: center;
    }
    .btn-modal-close {
        background: #f1f5f9;
        border: 1px solid #cbd5e1;
        padding: 6px 16px;
        border-radius: 6px;
        font-size: 13px;
        cursor: pointer;
        float: right;
        margin-top: 12px;
    }
    .btn-modal-close:hover {
        background: #e2e8f0;
    }

    /* Log Tabs */
    .log-tab {
        background: transparent;
        border: none;
        border-bottom: 2px solid transparent;
        padding: 6px 14px;
        font-size: 13px;
        font-weight: 500;
        color: #64748b;
        cursor: pointer;
        transition: all 0.15s;
    }
    .log-tab:hover {
        color: #0f172a;
    }
    .log-tab.active {
        color: #0284c7;
        border-bottom-color: #0284c7;
        font-weight: 600;
    }

    /* Toast Notification */
    .toast {
        visibility: hidden;
        min-width: 260px;
        background-color: #0f172a;
        color: #fff;
        text-align: center;
        border-radius: 6px;
        padding: 10px 16px;
        position: fixed;
        z-index: 300;
        bottom: 24px;
        left: 50%;
        transform: translateX(-50%);
        font-size: 14px;
        box-shadow: 0 4px 12px rgba(0,0,0,0.15);
    }
    .toast.show {
        visibility: visible;
        animation: fadein 0.3s, fadeout 0.3s 2.2s;
    }
    @keyframes fadein {
        from { bottom: 10px; opacity: 0; }
        to { bottom: 24px; opacity: 1; }
    }
    @keyframes fadeout {
        from { bottom: 24px; opacity: 1; }
        to { bottom: 10px; opacity: 0; }
    }
</style>
</head>
<body>

<main class="app-main">
<div class="laragon-window">
    <!-- Top Bar -->
    <div class="top-bar">
        <div class="top-left">
            <img src="/logo.png" alt="Servisan" class="brand-logo" onerror="this.src='/favicon.ico'">
            <span class="brand-name">Servisan</span>
        </div>

        <div class="top-right">
            <div style="position: relative;">
                <div class="menu-btn" onclick="toggleMenu(event)">Menu ▾</div>
                
                <div class="dropdown-menu" id="dropdownMenu">
                    <a class="dropdown-item" href="?phpinfo=1" target="_blank">PHP Info (phpinfo)</a>
                    <a class="dropdown-item" href="javascript:showExtensions()">Loaded Extensions (<?= count($extensions) ?>)</a>
                    <div class="dropdown-divider"></div>
                    <?php if ($pmaInstalled): ?>
                        <a class="dropdown-item" href="/phpmyadmin/" target="_blank">Open phpMyAdmin</a>
                    <?php endif; ?>
                    <a class="dropdown-item" href="javascript:openTerminal()">Terminal Shell</a>
                    <a class="dropdown-item" href="javascript:openLogs()">Server Logs</a>
                    <a class="dropdown-item" id="menuCheckUpdates" href="javascript:checkUpdateManual()">
                        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" style="color:#0284c7;"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path><polyline points="7 10 12 15 17 10"></polyline><line x1="12" y1="15" x2="12" y2="3"></line></svg>
                        <span>Check for Updates...</span>
                    </a>
                    <div class="dropdown-divider"></div>
                    <a class="dropdown-item" href="https://github.com/Haniel26/servisan" target="_blank">GitHub Repository</a>
                </div>
            </div>
            <span class="badge-version" id="versionBadge" onclick="checkUpdateManual()" style="cursor:pointer;" title="Servisan v<?= htmlspecialchars($servisanVersion) ?> - Click to check for updates">v<?= htmlspecialchars($servisanVersion) ?></span>
            <a href="https://github.com/Haniel26/servisan" target="_blank" class="icon-link" title="GitHub Repository">
                <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor">
                    <path fill-rule="evenodd" clip-rule="evenodd" d="M12 2C6.477 2 2 6.484 2 12.017c0 4.425 2.865 8.18 6.839 9.504.5.092.682-.217.682-.483 0-.237-.008-.868-.013-1.703-2.782.605-3.369-1.343-3.369-1.343-.454-1.158-1.11-1.466-1.11-1.466-.908-.62.069-.608.069-.608 1.003.07 1.53 1.032 1.53 1.032.892 1.53 2.341 1.088 2.91.832.092-.647.35-1.088.636-1.338-2.22-.253-4.555-1.113-4.555-4.951 0-1.093.39-1.988 1.029-2.688-.103-.253-.446-1.272.098-2.65 0 0 .84-.27 2.75 1.026A9.564 9.564 0 0112 6.844c.85.004 1.705.115 2.504.337 1.909-1.296 2.747-1.027 2.747-1.027.546 1.379.202 2.398.1 2.651.64.7 1.028 1.595 1.028 2.688 0 3.848-2.339 4.695-4.566 4.943.359.309.678.92.678 1.855 0 1.338-.012 2.419-.012 2.747 0 .268.18.58.688.482A10.019 10.019 0 0022 12.017C22 6.484 17.522 2 12 2z"></path>
                </svg>
            </a>
            <a href="https://github.com/Haniel26/servisan#readme" target="_blank" class="icon-link" title="Documentation">
                <svg width="17" height="17" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                    <circle cx="12" cy="12" r="10"></circle>
                    <path d="M9.09 9a3 3 0 0 1 5.83 1c0 2-3 3-3 3"></path>
                    <line x1="12" y1="17" x2="12.01" y2="17"></line>
                </svg>
            </a>
        </div>
    </div>

    <!-- Active Services (Laragon Center Style) -->
    <div class="services-body">
        <!-- PHP Service Row -->
        <div class="service-row">
            <div class="service-info">
                <div class="service-badge" title="PHP Engine">
                    <span style="font-size: 11px; font-weight: 700; color: #475569; letter-spacing: 0.5px;">PHP</span>
                </div>
                <div class="service-name">
                    PHP <?= htmlspecialchars(PHP_VERSION) ?>
                </div>
            </div>
            <div class="service-port">
                <?= htmlspecialchars((string)($_SERVER['SERVER_PORT'] ?? 8080)) ?>
            </div>
            <div class="reload-action">
                <a href="javascript:location.reload()">Reload</a>
            </div>
        </div>

        <!-- MySQL / MariaDB Service Row -->
        <div class="service-row">
            <div class="service-info">
                <div class="service-badge <?= $mysqlOk ? '' : 'offline' ?>" title="Database Engine">
                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                        <ellipse cx="12" cy="5" rx="9" ry="3"></ellipse>
                        <path d="M21 12c0 1.66-4 3-9 3s-9-1.34-9-3"></path>
                        <path d="M3 5v14c0 1.66 4 3 9 3s9-1.34 9-3V5"></path>
                    </svg>
                </div>
                <div class="service-name">
                    <?= htmlspecialchars($mysqlDisplayName) ?>
                </div>
            </div>
            <div class="service-port">
                <?= $mysqlPort ?>
            </div>
            <div class="reload-action">
                <?php if (!$mysqlOk): ?>
                    <span style="color:#e53e3e;font-size:14px;font-weight:500;" title="<?= htmlspecialchars($mysqlInfo) ?>">stopped</span>
                <?php else: ?>
                    <span style="color:#16a34a;font-size:14px;font-weight:500;">running</span>
                <?php endif; ?>
            </div>
        </div>

        <!-- Tray Management Helper Note -->
        <div class="tray-note">
            <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <circle cx="12" cy="12" r="10"></circle>
                <line x1="12" y1="16" x2="12" y2="12"></line>
                <line x1="12" y1="8" x2="12.01" y2="8"></line>
            </svg>
            <span>Service management (Start / Stop / Restart) dikontrol melalui ikon <strong>Servisan System Tray</strong> di taskbar Windows Anda.</span>
        </div>
    </div>

    <!-- Bottom Dock (Laragon Style Buttons) -->
    <div class="bottom-bar">
        <!-- Logs Button -->
        <button class="btn-dock" type="button" onclick="openLogs()" title="Buka folder logs di Windows File Explorer">
            <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="#0f172a" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"></path>
                <polyline points="14 2 14 8 20 8"></polyline>
                <line x1="16" y1="13" x2="8" y2="13"></line>
                <line x1="16" y1="17" x2="8" y2="17"></line>
            </svg>
            <span>Logs</span>
        </button>

        <!-- Database Button -->
        <a class="btn-dock" href="<?= $pmaInstalled ? '/phpmyadmin/' : 'javascript:showDbModal()' ?>" <?= $pmaInstalled ? 'target="_blank" rel="noopener"' : '' ?> title="Database Manager">
            <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="#0f172a" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <ellipse cx="12" cy="5" rx="9" ry="3"></ellipse>
                <path d="M21 12c0 1.66-4 3-9 3s-9-1.34-9-3"></path>
                <path d="M3 5v14c0 1.66 4 3 9 3s9-1.34 9-3V5"></path>
            </svg>
            <span>Database</span>
        </a>

        <!-- Terminal Button -->
        <button class="btn-dock" type="button" onclick="openTerminal()" title="Buka Servisan Shell di Terminal">
            <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="#0f172a" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <polyline points="4 17 10 11 4 5"></polyline>
                <line x1="12" y1="19" x2="20" y2="19"></line>
            </svg>
            <span>Terminal</span>
        </button>

        <!-- Root Button -->
        <button class="btn-dock" type="button" onclick="openRoot()" title="Buka folder www di Windows File Explorer">
            <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="#0f172a" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"></path>
            </svg>
            <span>Root</span>
        </button>
    </div>
</div>
</main>

<!-- Page Footer Credit -->
<footer class="app-footer">
    <span>Servisan &bull; Crafted by <a href="https://github.com/Haniel26" target="_blank" rel="noopener">Haniel26</a> &bull; <a href="https://github.com/Haniel26/servisan" target="_blank" rel="noopener">Open Source on GitHub</a></span>
</footer>

<!-- Terminal Modal -->
<div class="modal-backdrop" id="terminalModal" onclick="closeModal(event, 'terminalModal')">
    <div class="modal-box">
        <h3 class="modal-title">Servisan Terminal</h3>
        <div class="modal-body">
            <p>Untuk membuka shell dengan <code>php</code>, <code>mysql</code>, dan <code>mysqladmin</code> otomatis tersedia di PATH, jalankan perintah ini di terminal Anda:</p>
            <div class="code-box">
                <span id="cmdText">servisan shell</span>
                <button type="button" onclick="copyCommand()" style="background:#0284c7;color:#fff;border:none;padding:4px 10px;border-radius:4px;cursor:pointer;font-size:12px;">Copy</button>
            </div>
            <p style="font-size:13px;color:#64748b;margin:0;">Atau klik kanan tray icon Servisan di taskbar dan pilih <strong>Open Shell</strong>.</p>
        </div>
        <button class="btn-modal-close" onclick="document.getElementById('terminalModal').classList.remove('show')">Close</button>
    </div>
</div>

<!-- Database Modal -->
<div class="modal-backdrop" id="dbModal" onclick="closeModal(event, 'dbModal')">
    <div class="modal-box">
        <h3 class="modal-title">Database Access</h3>
        <div class="modal-body">
            <p>Koneksi MySQL / MariaDB lokal aktif pada:</p>
            <div class="code-box">
                <span>Host: 127.0.0.1:<?= $mysqlPort ?> | User: root</span>
            </div>
            <p style="font-size:13px;color:#64748b;">phpMyAdmin belum terpasang di folder <code>www/phpmyadmin</code>. Untuk memasangnya secara otomatis, jalankan:</p>
            <div class="code-box">
                <span>servisan phpmyadmin install</span>
            </div>
        </div>
        <button class="btn-modal-close" onclick="document.getElementById('dbModal').classList.remove('show')">Close</button>
    </div>
</div>

<!-- Extensions Modal -->
<div class="modal-backdrop" id="extModal" onclick="closeModal(event, 'extModal')">
    <div class="modal-box" style="width: 580px;">
        <h3 class="modal-title">Loaded PHP Extensions (<?= count($extensions) ?>)</h3>
        <div class="modal-body" style="max-height: 280px; overflow-y: auto;">
            <div style="display: grid; grid-template-columns: repeat(3, 1fr); gap: 6px; font-size: 13px; font-family: monospace;">
                <?php foreach ($extensions as $ext): ?>
                    <span style="background:#f1f5f9;padding:3px 6px;border-radius:4px;"><?= htmlspecialchars($ext) ?></span>
                <?php endforeach; ?>
            </div>
        </div>
        <button class="btn-modal-close" onclick="document.getElementById('extModal').classList.remove('show')">Close</button>
    </div>
</div>

<!-- Server Logs Modal (Last 50 Lines) -->
<div class="modal-backdrop" id="logsModal" onclick="closeModal(event, 'logsModal')">
    <div class="modal-box" style="width: 760px; max-width: 95vw; display: flex; flex-direction: column; max-height: 85vh;">
        <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 12px;">
            <div style="display: flex; align-items: center; gap: 8px;">
                <h3 class="modal-title" style="margin: 0;">Server Logs</h3>
                <span style="font-size: 12px; color: #64748b; font-weight: normal;">(50 baris terakhir)</span>
            </div>
            <div style="display: flex; gap: 6px;">
                <button type="button" onclick="fetch('?action=open_logs'); showToast('Membuka folder logs di Explorer...');" style="background:#f1f5f9;border:1px solid #cbd5e1;padding:4px 10px;border-radius:4px;font-size:12px;cursor:pointer;color:#334155;">Open Folder</button>
                <button type="button" onclick="loadLogs(currentLogType)" style="background:#0284c7;color:#fff;border:none;padding:4px 10px;border-radius:4px;font-size:12px;cursor:pointer;">Refresh</button>
                <button type="button" onclick="document.getElementById('logsModal').classList.remove('show')" style="background:#f1f5f9;border:1px solid #cbd5e1;padding:4px 10px;border-radius:4px;font-size:12px;cursor:pointer;color:#64748b;">✕</button>
            </div>
        </div>

        <!-- Log Tabs -->
        <div style="display: flex; gap: 4px; border-bottom: 1px solid #e2e8f0; margin-bottom: 10px;">
            <button type="button" class="log-tab active" id="tab-php" onclick="switchLogTab('php')">PHP (php.log)</button>
            <button type="button" class="log-tab" id="tab-mysql" onclick="switchLogTab('mysql')">MySQL (mysql.log)</button>
            <button type="button" class="log-tab" id="tab-servisan" onclick="switchLogTab('servisan')">Servisan (servisan.log)</button>
        </div>

        <!-- Log Output Console -->
        <div id="logConsole" style="background: #0f172a; color: #38bdf8; padding: 14px 16px; border-radius: 6px; font-family: Consolas, 'Courier New', monospace; font-size: 12.5px; line-height: 1.5; height: 320px; overflow-y: auto; white-space: pre-wrap; word-break: break-all;">Loading logs...</div>
    </div>
</div>

<!-- Servisan Update Modal -->
<div class="modal-backdrop" id="updateModal" onclick="closeModal(event, 'updateModal')">
    <div class="modal-box" style="width: 580px; max-width: 95vw;">
        <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 14px;">
            <div style="display: flex; align-items: center; gap: 10px;">
                <div style="width: 32px; height: 32px; border-radius: 6px; background: #e0f2fe; display: flex; align-items: center; justify-content: center; color: #0284c7;">
                    <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                        <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path>
                        <polyline points="7 10 12 15 17 10"></polyline>
                        <line x1="12" y1="15" x2="12" y2="3"></line>
                    </svg>
                </div>
                <div>
                    <h3 class="modal-title" style="margin: 0;">Servisan Update Available</h3>
                    <div style="font-size: 12px; color: #64748b; margin-top: 2px;">Versi baru telah dirilis di GitHub</div>
                </div>
            </div>
            <button type="button" onclick="document.getElementById('updateModal').classList.remove('show')" style="background:#f1f5f9;border:1px solid #cbd5e1;padding:4px 10px;border-radius:4px;font-size:12px;cursor:pointer;color:#64748b;">✕</button>
        </div>

        <div class="modal-body" style="padding: 0;">
            <div style="background: #f8fafc; border: 1px solid #e2e8f0; border-radius: 6px; padding: 12px 16px; margin-bottom: 14px; display: flex; justify-content: space-between; align-items: center;">
                <div>
                    <div style="font-size: 11px; text-transform: uppercase; letter-spacing: 0.5px; color: #64748b; font-weight: 600;">Current Version</div>
                    <div style="font-size: 15px; font-weight: 700; color: #334155;">v<?= htmlspecialchars($servisanVersion) ?></div>
                </div>
                <div style="color: #94a3b8; font-size: 20px;">→</div>
                <div>
                    <div style="font-size: 11px; text-transform: uppercase; letter-spacing: 0.5px; color: #0284c7; font-weight: 600;">New Version</div>
                    <div id="updateLatestTag" style="font-size: 15px; font-weight: 700; color: #0284c7;">v...</div>
                </div>
                <div style="text-align: right;">
                    <div style="font-size: 11px; text-transform: uppercase; letter-spacing: 0.5px; color: #64748b; font-weight: 600;">Released</div>
                    <div id="updateDate" style="font-size: 13px; color: #475569; font-weight: 500;">...</div>
                </div>
            </div>

            <div style="margin-bottom: 14px;">
                <div style="font-size: 12.5px; font-weight: 600; color: #334155; margin-bottom: 6px;">Release Notes & Changelog:</div>
                <div id="updateChangelog" style="background: #ffffff; border: 1px solid #e2e8f0; border-radius: 6px; padding: 12px 14px; max-height: 180px; overflow-y: auto; font-size: 12.5px; line-height: 1.6; color: #334155; white-space: pre-wrap; word-break: break-word; font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;">Loading release notes...</div>
            </div>

            <div style="font-size: 12.5px; font-weight: 600; color: #334155; margin-bottom: 8px;">Direct 1-Click Downloads:</div>
            <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 10px; margin-bottom: 14px;">
                <a id="btnDownloadInstaller" href="#" target="_blank" style="display: flex; flex-direction: column; padding: 10px 12px; background: #0284c7; color: #ffffff; text-decoration: none; border-radius: 6px; transition: background 0.2s;">
                    <span style="font-size: 13px; font-weight: 600; display: flex; align-items: center; gap: 6px;">
                        <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path><polyline points="7 10 12 15 17 10"></polyline><line x1="12" y1="15" x2="12" y2="3"></line></svg>
                        Windows Installer
                    </span>
                    <span style="font-size: 11px; opacity: 0.85; margin-top: 2px;">Standard Setup (.exe)</span>
                </a>
                <a id="btnDownloadPortable" href="#" target="_blank" style="display: flex; flex-direction: column; padding: 10px 12px; background: #f1f5f9; color: #0f172a; text-decoration: none; border: 1px solid #cbd5e1; border-radius: 6px; transition: background 0.2s;">
                    <span style="font-size: 13px; font-weight: 600; display: flex; align-items: center; gap: 6px;">
                        <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path><polyline points="7 10 12 15 17 10"></polyline><line x1="12" y1="15" x2="12" y2="3"></line></svg>
                        Portable Archive
                    </span>
                    <span style="font-size: 11px; color: #64748b; margin-top: 2px;">Standalone Zip (.zip)</span>
                </a>
            </div>
        </div>

        <div style="display: flex; justify-content: space-between; align-items: center; border-top: 1px solid #e2e8f0; padding-top: 12px;">
            <a id="btnViewRelease" href="https://github.com/Haniel26/servisan/releases/latest" target="_blank" style="font-size: 12.5px; color: #0284c7; text-decoration: none; display: flex; align-items: center; gap: 4px;">
                <span>View on GitHub Releases</span>
                <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"></path><polyline points="15 3 21 3 21 9"></polyline><line x1="10" y1="14" x2="21" y2="3"></line></svg>
            </a>
            <button class="btn-modal-close" style="margin-top:0;" onclick="document.getElementById('updateModal').classList.remove('show')">Close</button>
        </div>
    </div>
</div>

<div class="toast" id="toast">Copied to clipboard!</div>

<script>
    function toggleMenu(e) {
        e.stopPropagation();
        document.getElementById('dropdownMenu').classList.toggle('show');
    }

    window.onclick = function() {
        document.getElementById('dropdownMenu').classList.remove('show');
    };

    function showTerminalModal() {
        document.getElementById('terminalModal').classList.add('show');
    }

    function showDbModal() {
        document.getElementById('dbModal').classList.add('show');
    }

    function showExtensions() {
        document.getElementById('extModal').classList.add('show');
    }

    function closeModal(e, id) {
        if (e.target.id === id) {
            document.getElementById(id).classList.remove('show');
        }
    }

    function copyCommand() {
        navigator.clipboard.writeText('servisan shell').then(() => {
            showToast('Copied "servisan shell" to clipboard!');
        });
    }

    const docRootPath = <?= json_encode($docRoot, JSON_HEX_TAG | JSON_HEX_APOS | JSON_HEX_QUOT | JSON_HEX_AMP) ?>;

    function openTerminal() {
        fetch('?action=open_terminal')
            .then(res => {
                if (res.ok) {
                    showToast('Membuka Servisan Terminal Shell...');
                } else {
                    showTerminalModal();
                }
            })
            .catch(() => {
                showTerminalModal();
            });
    }

    let currentLogType = 'php';

    function openLogs() {
        document.getElementById('logsModal').classList.add('show');
        switchLogTab('php');
    }

    function switchLogTab(type) {
        currentLogType = type;
        document.querySelectorAll('.log-tab').forEach(t => t.classList.remove('active'));
        const tab = document.getElementById('tab-' + type);
        if (tab) tab.classList.add('active');
        loadLogs(type);
    }

    function loadLogs(type) {
        const consoleEl = document.getElementById('logConsole');
        consoleEl.textContent = 'Memuat 50 log terakhir (' + type + '.log)...';
        fetch('?action=get_logs&type=' + encodeURIComponent(type))
            .then(res => res.json())
            .then(data => {
                if (data.ok) {
                    consoleEl.textContent = data.content;
                    consoleEl.scrollTop = consoleEl.scrollHeight;
                } else {
                    consoleEl.textContent = 'Gagal memuat log.';
                }
            })
            .catch(err => {
                consoleEl.textContent = 'Error: ' + err;
            });
    }

    function openRoot() {
        fetch('?action=open_root')
            .then(res => {
                if (res.ok) {
                    showToast('Membuka folder www di File Explorer...');
                } else {
                    copyRoot();
                }
            })
            .catch(() => {
                copyRoot();
            });
    }

    function copyRoot() {
        navigator.clipboard.writeText(docRootPath).then(() => {
            showToast('Root directory copied: ' + docRootPath);
        }).catch(() => {
            showToast('Root: ' + docRootPath);
        });
    }

    function showToast(msg) {
        const toast = document.getElementById('toast');
        toast.textContent = msg;
        toast.className = 'toast show';
        setTimeout(() => {
            toast.className = 'toast';
        }, 2500);
    }

    // Update Checker (Option B)
    const CURRENT_VERSION = <?= json_encode($servisanVersion, JSON_HEX_TAG | JSON_HEX_APOS | JSON_HEX_QUOT | JSON_HEX_AMP) ?>;
    let cachedRelease = null;

    function compareSemver(a, b) {
        const pa = String(a).replace(/^v/, '').split('.').map(n => parseInt(n, 10) || 0);
        const pb = String(b).replace(/^v/, '').split('.').map(n => parseInt(n, 10) || 0);
        for (let i = 0; i < Math.max(pa.length, pb.length); i++) {
            const na = pa[i] || 0;
            const nb = pb[i] || 0;
            if (na > nb) return 1;
            if (na < nb) return -1;
        }
        return 0;
    }

    function checkForUpdates(isManual) {
        if (CURRENT_VERSION === 'dev') {
            if (isManual) {
                showToast('Versi development: pengecekan update dilewati.');
            }
            return;
        }
        if (isManual) {
            showToast('Memeriksa pembaruan di GitHub...');
        }
        fetch('https://api.github.com/repos/Haniel26/servisan/releases/latest', {
            cache: 'no-cache',
            headers: { 'Accept': 'application/vnd.github.v3+json' }
        })
        .then(res => {
            if (!res.ok) throw new Error('GitHub status ' + res.status);
            return res.json();
        })
        .then(release => {
            cachedRelease = release;
            const latestTag = release.tag_name || '';
            const hasUpdate = compareSemver(latestTag, CURRENT_VERSION) > 0;

            const badge = document.getElementById('versionBadge');
            const menuCheck = document.getElementById('menuCheckUpdates');

            if (hasUpdate) {
                if (badge) {
                    badge.classList.add('has-update');
                    badge.innerHTML = '<span class="update-pulse-dot"></span> ' + latestTag + ' Available';
                    badge.title = 'Versi baru ' + latestTag + ' tersedia! Klik untuk detail.';
                }
                if (menuCheck) {
                    menuCheck.innerHTML = `
                        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" style="color:#2563eb;"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path><polyline points="7 10 12 15 17 10"></polyline><line x1="12" y1="15" x2="12" y2="3"></line></svg>
                        <span>Update Available (${latestTag})</span>
                    `;
                }
                if (isManual) {
                    showUpdateModal();
                }
            } else {
                if (badge) {
                    badge.classList.remove('has-update');
                    badge.textContent = 'v' + CURRENT_VERSION;
                    badge.title = 'Servisan v' + CURRENT_VERSION + ' - Up to date';
                }
                if (isManual) {
                    showToast('Servisan v' + CURRENT_VERSION + ' sudah versi terbaru!');
                }
            }
        })
        .catch(err => {
            if (isManual) {
                showToast('Gagal memeriksa update. Cek koneksi internet.');
            }
        });
    }

    function checkUpdateManual() {
        if (cachedRelease && compareSemver(cachedRelease.tag_name, CURRENT_VERSION) > 0) {
            showUpdateModal();
        } else {
            checkForUpdates(true);
        }
    }

    function showUpdateModal() {
        if (!cachedRelease) {
            checkForUpdates(true);
            return;
        }

        const latestTag = cachedRelease.tag_name || '';
        document.getElementById('updateLatestTag').textContent = latestTag;

        if (cachedRelease.published_at) {
            const d = new Date(cachedRelease.published_at);
            document.getElementById('updateDate').textContent = d.toISOString().split('T')[0];
        }

        const body = (cachedRelease.body || '').trim();
        document.getElementById('updateChangelog').textContent = body || 'Tidak ada catatan rilis tambahan.';

        // Find installer (.exe) and portable (.zip) download URLs
        let installerUrl = cachedRelease.html_url;
        let portableUrl = cachedRelease.html_url;

        if (Array.isArray(cachedRelease.assets)) {
            for (const asset of cachedRelease.assets) {
                const name = (asset.name || '').toLowerCase();
                if (name.endsWith('.exe')) {
                    installerUrl = asset.browser_download_url;
                } else if (name.endsWith('.zip')) {
                    portableUrl = asset.browser_download_url;
                }
            }
        }

        document.getElementById('btnDownloadInstaller').href = installerUrl;
        document.getElementById('btnDownloadPortable').href = portableUrl;
        document.getElementById('btnViewRelease').href = cachedRelease.html_url;

        document.getElementById('updateModal').classList.add('show');
    }

    // Auto-check silently when dashboard loads
    window.addEventListener('load', () => {
        setTimeout(() => {
            checkForUpdates(false);
        }, 1200);
    });
</script>

</body>
</html>
