<?php

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
$servisanVersion = getenv('SERVISAN_VERSION') ?: '0.2.0';
// Load shared helpers
require_once __DIR__ . '/includes/helpers.php';

$servisanHome = get_servisan_home();
$pmaInstalled = is_dir($servisanHome . '/www/phpmyadmin');
$docRoot = getenv('SERVISAN_WWW_DIR') ? getenv('SERVISAN_WWW_DIR') : ($servisanHome . DIRECTORY_SEPARATOR . 'www');
$isWindows = defined('PHP_OS_FAMILY') ? (PHP_OS_FAMILY === 'Windows') : (DIRECTORY_SEPARATOR === '\\');

// Inspect MySQL status
list($mysqlOk, $mysqlInfo) = check_mysql($mysqlHost, $mysqlPort, $mysqlUser, $mysqlPass);
$mysqlDisplayName = 'MySQL 8.4';
if ($mysqlOk) {
    if (stripos($mysqlInfo, 'mariadb') !== false) {
        preg_match('/(\d+\.\d+(\.\d+)?)/', $mysqlInfo, $m);
        $mysqlDisplayName = 'MariaDB ' . (isset($m[1]) ? $m[1] : '11.4');
    } else {
        preg_match('/(\d+\.\d+(\.\d+)?)/', $mysqlInfo, $m);
        $mysqlDisplayName = 'MySQL ' . (isset($m[1]) ? $m[1] : '8.4');
    }
} else {
    $mysqlDisplayName = 'MySQL / MariaDB';
}

// Dispatch API Controllers
require_once __DIR__ . '/includes/api-system.php';
require_once __DIR__ . '/includes/api-php.php';
require_once __DIR__ . '/includes/api-vhost.php';
require_once __DIR__ . '/includes/api-migrate.php';

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
<link rel="stylesheet" href="/assets/css/style.css">
<style>
    /* Critical UI styles: ensure log text and modal body are selectable */
    #logConsole, .modal-body { user-select: text; }
</style>
</head>
<body>

<main class="app-main">
<div class="laragon-window">
    <!-- Top Bar Header -->
    <header class="top-bar">
        <div class="top-left">
            <img src="/logo.png" alt="Servisan" class="brand-logo" onerror="this.src='/favicon.ico'">
            <span class="brand-name">Servisan</span>
        </div>

        <div class="top-right">
            <!-- Version label (small gray text, not a button) -->
            <span class="version-text" id="versionBadge" onclick="checkUpdateManual()" title="Servisan v<?= htmlspecialchars($servisanVersion) ?> - Klik untuk periksa update">v<?= htmlspecialchars($servisanVersion) ?></span>

            <a href="https://github.com/Haniel26/servisan" target="_blank" class="header-icon-btn" title="GitHub Repository">
                <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor">
                    <path fill-rule="evenodd" clip-rule="evenodd" d="M12 2C6.477 2 2 6.484 2 12.017c0 4.425 2.865 8.18 6.839 9.504.5.092.682-.217.682-.483 0-.237-.008-.868-.013-1.703-2.782.605-3.369-1.343-3.369-1.343-.454-1.158-1.11-1.466-1.11-1.466-.908-.62.069-.608.069-.608 1.003.07 1.53 1.032 1.53 1.032.892 1.53 2.341 1.088 2.91.832.092-.647.35-1.088.636-1.338-2.22-.253-4.555-1.113-4.555-4.951 0-1.093.39-1.988 1.029-2.688-.103-.253-.446-1.272.098-2.65 0 0 .84-.27 2.75 1.026A9.564 9.564 0 0112 6.844c.85.004 1.705.115 2.504.337 1.909-1.296 2.747-1.027 2.747-1.027.546 1.379.202 2.398.1 2.651.64.7 1.028 1.595 1.028 2.688 0 3.848-2.339 4.695-4.566 4.943.359.309.678.92.678 1.855 0 1.338-.012 2.419-.012 2.747 0 .268.18.58.688.482A10.019 10.019 0 0022 12.017C22 6.484 17.522 2 12 2z"></path>
                </svg>
            </a>
            <a href="https://github.com/Haniel26/servisan#readme" target="_blank" class="header-icon-btn" title="Bantuan & Dokumentasi">
                <svg width="17" height="17" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                    <circle cx="12" cy="12" r="10"></circle>
                    <path d="M9.09 9a3 3 0 0 1 5.83 1c0 2-3 3-3 3"></path>
                    <line x1="12" y1="17" x2="12.01" y2="17"></line>
                </svg>
            </a>
        </div>
    </header>

    <!-- Services Body -->
    <div class="services-body">
        <!-- Service Grid Header -->
        <div class="service-table-header">
            <div class="col-service">Service</div>
            <div class="col-port">Port</div>
            <div class="col-status">Status</div>
            <div class="col-action">Aksi</div>
        </div>

        <!-- PHP Service Row -->
        <div class="service-row">
            <div class="col-service">
                <div class="service-brand">
                    <div class="service-icon-badge" title="PHP Engine">
                        <span>PHP</span>
                    </div>
                    <div class="service-title-wrap">
                        <span class="service-title">PHP <?= htmlspecialchars(PHP_VERSION) ?></span>
                    </div>
                </div>
            </div>
            <div class="col-port">
                <span class="port-chip">:<?= (int)(isset($_SERVER['SERVER_PORT']) ? $_SERVER['SERVER_PORT'] : 8080) ?></span>
            </div>
            <div class="col-status">
                <span class="status-badge-pill running"><span class="status-dot"></span> Running</span>
            </div>
            <div class="col-action">
                <button type="button" class="btn-action-outline" onclick="openPhpModal()" title="Ganti atau update versi PHP">
                    <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                        <polyline points="16 18 22 12 16 6"></polyline>
                        <polyline points="8 6 2 12 8 18"></polyline>
                    </svg>
                    <span>Switch</span>
                </button>
            </div>
        </div>

        <!-- MySQL / MariaDB Service Row -->
        <div class="service-row">
            <div class="col-service">
                <div class="service-brand">
                    <div class="service-icon-badge <?= $mysqlOk ? '' : 'offline' ?>" title="Database Engine">
                        <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                            <ellipse cx="12" cy="5" rx="9" ry="3"></ellipse>
                            <path d="M21 12c0 1.66-4 3-9 3s-9-1.34-9-3"></path>
                            <path d="M3 5v14c0 1.66 4 3 9 3s9-1.34 9-3V5"></path>
                        </svg>
                    </div>
                    <div class="service-title-wrap">
                        <span class="service-title"><?= htmlspecialchars($mysqlDisplayName) ?></span>
                    </div>
                </div>
            </div>
            <div class="col-port">
                <span class="port-chip">:<?= $mysqlPort ?></span>
            </div>
            <div class="col-status">
                <?php if ($mysqlOk): ?>
                    <span class="status-badge-pill running"><span class="status-dot"></span> Running</span>
                <?php else: ?>
                    <span class="status-badge-pill stopped" title="<?= htmlspecialchars($mysqlInfo) ?>"><span class="status-dot"></span> stopped</span>
                <?php endif; ?>
            </div>
            <div class="col-action">
                <a class="btn-action-outline" href="<?= $pmaInstalled ? '/phpmyadmin/' : 'javascript:showDbModal()' ?>" <?= $pmaInstalled ? 'target="_blank" rel="noopener"' : '' ?> title="Buka Database Manager">
                    <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                        <ellipse cx="12" cy="5" rx="9" ry="3"></ellipse>
                        <path d="M21 12c0 1.66-4 3-9 3s-9-1.34-9-3"></path>
                        <path d="M3 5v14c0 1.66 4 3 9 3s9-1.34 9-3V5"></path>
                    </svg>
                    <span>Database</span>
                </a>
            </div>
        </div>

        <!-- Info Banner (40px margin-top, dismissible) -->
        <div class="info-banner" id="trayInfoBanner">
            <div class="info-banner-left">
                <svg class="info-banner-icon" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                    <circle cx="12" cy="12" r="10"></circle>
                    <line x1="12" y1="16" x2="12" y2="12"></line>
                    <line x1="12" y1="8" x2="12.01" y2="8"></line>
                </svg>
                <span>Service management (Start / Stop / Restart) dikontrol melalui ikon <strong>Servisan System Tray</strong> di taskbar Windows Anda.</span>
            </div>
            <button type="button" class="info-banner-close" onclick="this.closest('.info-banner').remove()" title="Tutup pemberitahuan" aria-label="Tutup">
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                    <line x1="18" y1="6" x2="6" y2="18"></line>
                    <line x1="6" y1="6" x2="18" y2="18"></line>
                </svg>
            </button>
        </div>
    </div>

    <!-- Bottom Dock Menu (6 equal-width buttons in 1 row) -->
    <nav class="bottom-bar">
        <!-- Start Menu Button (Windows Start / macOS Dock Style) -->
        <div class="dock-menu-wrapper">
            <button class="btn-dock btn-dock-menu" id="dockMenuBtn" type="button" onclick="toggleMenu(event)" title="Buka Menu Servisan">
                <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                    <rect x="3" y="3" width="7" height="7"></rect>
                    <rect x="14" y="3" width="7" height="7"></rect>
                    <rect x="14" y="14" width="7" height="7"></rect>
                    <rect x="3" y="14" width="7" height="7"></rect>
                </svg>
                <span>Menu</span>
            </button>

            <!-- Dock Flyout Pop-up Menu -->
            <div class="dock-flyout-menu" id="dropdownMenu">
                <a class="dropdown-item" href="?phpinfo=1" target="_blank">
                    <svg class="menu-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                        <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"></path>
                        <polyline points="14 2 14 8 20 8"></polyline>
                        <line x1="16" y1="13" x2="8" y2="13"></line>
                        <line x1="16" y1="17" x2="8" y2="17"></line>
                        <polyline points="10 9 9 9 8 9"></polyline>
                    </svg>
                    <span>phpinfo()</span>
                </a>
                <a class="dropdown-item" href="javascript:showExtensions()">
                    <svg class="menu-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                        <path d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z"></path>
                        <polyline points="3.27 6.96 12 12.01 20.73 6.96"></polyline>
                        <line x1="12" y1="22.08" x2="12" y2="12"></line>
                    </svg>
                    <span>Loaded Extensions</span>
                    <span class="menu-badge"><?= count($extensions) ?></span>
                </a>

                <div class="dropdown-divider"></div>

                <a class="dropdown-item" href="javascript:openPhpModal()">
                    <svg class="menu-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                        <polyline points="16 18 22 12 16 6"></polyline>
                        <polyline points="8 6 2 12 8 18"></polyline>
                    </svg>
                    <span>PHP Version Manager</span>
                </a>
                <a class="dropdown-item" href="javascript:openMigrateModal()">
                    <svg class="menu-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                        <ellipse cx="12" cy="5" rx="9" ry="3"></ellipse>
                        <path d="M3 5v14c0 1.66 4 3 9 3s9-1.34 9-3V5"></path>
                        <path d="M12 12l4 4m0 0l-4 4m4-4H8"></path>
                    </svg>
                    <span>Database Migration</span>
                </a>
                <a class="dropdown-item" href="javascript:openVhostModal()">
                    <svg class="menu-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                        <path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71"></path>
                        <path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71"></path>
                    </svg>
                    <span>Virtual Hosts & Projects</span>
                </a>
                <?php if ($pmaInstalled): ?>
                    <a class="dropdown-item" href="/phpmyadmin/" target="_blank">
                        <svg class="menu-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                            <ellipse cx="12" cy="5" rx="9" ry="3"></ellipse>
                            <path d="M21 12c0 1.66-4 3-9 3s-9-1.34-9-3"></path>
                            <path d="M3 5v14c0 1.66 4 3 9 3s9-1.34 9-3V5"></path>
                        </svg>
                        <span>Open phpMyAdmin</span>
                    </a>
                <?php endif; ?>
                <a class="dropdown-item" href="javascript:openTerminal()">
                    <svg class="menu-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                        <polyline points="4 17 10 11 4 5"></polyline>
                        <line x1="12" y1="19" x2="20" y2="19"></line>
                    </svg>
                    <span>Terminal Shell</span>
                </a>
                <a class="dropdown-item" href="javascript:openLogs()">
                    <svg class="menu-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                        <line x1="8" y1="6" x2="21" y2="6"></line>
                        <line x1="8" y1="12" x2="21" y2="12"></line>
                        <line x1="8" y1="18" x2="21" y2="18"></line>
                        <line x1="3" y1="6" x2="3.01" y2="6"></line>
                        <line x1="3" y1="12" x2="3.01" y2="12"></line>
                        <line x1="3" y1="18" x2="3.01" y2="18"></line>
                    </svg>
                    <span>Server Logs</span>
                </a>

                <div class="dropdown-divider"></div>

                <a class="dropdown-item" id="menuCheckUpdates" href="javascript:checkUpdateManual()">
                    <svg class="menu-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                        <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path>
                        <polyline points="7 10 12 15 17 10"></polyline>
                        <line x1="12" y1="15" x2="12" y2="3"></line>
                    </svg>
                    <span>Check for Updates...</span>
                </a>
                <a class="dropdown-item" href="https://github.com/Haniel26/servisan" target="_blank">
                    <svg class="menu-icon" viewBox="0 0 24 24" fill="currentColor">
                        <path fill-rule="evenodd" clip-rule="evenodd" d="M12 2C6.477 2 2 6.484 2 12.017c0 4.425 2.865 8.18 6.839 9.504.5.092.682-.217.682-.483 0-.237-.008-.868-.013-1.703-2.782.605-3.369-1.343-3.369-1.343-.454-1.158-1.11-1.466-1.11-1.466-.908-.62.069-.608.069-.608 1.003.07 1.53 1.032 1.53 1.032.892 1.53 2.341 1.088 2.91.832.092-.647.35-1.088.636-1.338-2.22-.253-4.555-1.113-4.555-4.951 0-1.093.39-1.988 1.029-2.688-.103-.253-.446-1.272.098-2.65 0 0 .84-.27 2.75 1.026A9.564 9.564 0 0112 6.844c.85.004 1.705.115 2.504.337 1.909-1.296 2.747-1.027 2.747-1.027.546 1.379.202 2.398.1 2.651.64.7 1.028 1.595 1.028 2.688 0 3.848-2.339 4.695-4.566 4.943.359.309.678.92.678 1.855 0 1.338-.012 2.419-.012 2.747 0 .268.18.58.688.482A10.019 10.019 0 0022 12.017C22 6.484 17.522 2 12 2z"></path>
                    </svg>
                    <span>GitHub Repository</span>
                </a>

                <div class="dropdown-divider"></div>
                <a class="dropdown-item dropdown-item-danger" href="javascript:confirmShutdown()">
                    <svg class="menu-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                        <path d="M18.36 6.64a9 9 0 1 1-12.73 0"></path>
                        <line x1="12" y1="2" x2="12" y2="12"></line>
                    </svg>
                    <span>Shutdown ServiSan</span>
                </a>
            </div>
        </div>

        <!-- Logs Button -->
        <button class="btn-dock" type="button" onclick="openLogs()" title="Buka Server Logs">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"></path>
                <polyline points="14 2 14 8 20 8"></polyline>
                <line x1="16" y1="13" x2="8" y2="13"></line>
                <line x1="16" y1="17" x2="8" y2="17"></line>
            </svg>
            <span>Logs</span>
        </button>

        <!-- Terminal Button -->
        <button class="btn-dock" type="button" onclick="openTerminal()" title="Buka Servisan Shell di Terminal">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <polyline points="4 17 10 11 4 5"></polyline>
                <line x1="12" y1="19" x2="20" y2="19"></line>
            </svg>
            <span>Terminal</span>
        </button>

        <!-- Projects Button -->
        <button class="btn-dock" type="button" onclick="openVhostModal()" title="Kelola Virtual Hosts & External Projects">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71"></path>
                <path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71"></path>
            </svg>
            <span>Projects</span>
        </button>

        <!-- Root Button -->
        <button class="btn-dock" type="button" onclick="openRoot()" title="Buka folder www di Windows File Explorer">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"></path>
            </svg>
            <span>Root</span>
        </button>

        <!-- Reload Button -->
        <button class="btn-dock" type="button" onclick="window.location.reload()" title="Muat ulang status server">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <path d="M23 4v6h-6"></path>
                <path d="M1 20v-6h6"></path>
                <path d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15"></path>
            </svg>
            <span>Reload</span>
        </button>
    </nav>
</div>
</main>

<!-- Page Footer Credit -->
<footer class="app-footer">
    <span>Servisan &bull; Crafted by <a href="https://github.com/Haniel26" target="_blank" rel="noopener">Haniel26</a> &bull; <a href="https://github.com/Haniel26/servisan" target="_blank" rel="noopener">Open Source on GitHub</a></span>
</footer>

<!-- Modals Partials -->
<?php
include __DIR__ . '/views/modals/modal-terminal.php';
include __DIR__ . '/views/modals/modal-db.php';
include __DIR__ . '/views/modals/modal-ext.php';
include __DIR__ . '/views/modals/modal-logs.php';
include __DIR__ . '/views/modals/modal-php.php';
include __DIR__ . '/views/modals/modal-vhost.php';
include __DIR__ . '/views/modals/modal-update.php';
include __DIR__ . '/views/modals/modal-migrate.php';
?>

<div class="toast" id="toast">Copied to clipboard!</div>

<?php
include __DIR__ . '/views/modals/overlay-shutdown.php';
?>

<!-- Client Scripts -->
<script>
    window.SERVISAN_DOC_ROOT = <?= json_encode($docRoot, JSON_HEX_TAG | JSON_HEX_APOS | JSON_HEX_QUOT | JSON_HEX_AMP) ?>;
    window.SERVISAN_VERSION = <?= json_encode($servisanVersion, JSON_HEX_TAG | JSON_HEX_APOS | JSON_HEX_QUOT | JSON_HEX_AMP) ?>;
    window.SERVISAN_PORT = <?= isset($_SERVER['SERVER_PORT']) ? (int)$_SERVER['SERVER_PORT'] : 8080 ?>;
    var CURRENT_VERSION = window.SERVISAN_VERSION;
    // Development mode check: skip automated update queries in dev
    if (CURRENT_VERSION === 'dev') {
        window.SERVISAN_SKIP_UPDATE = true;
    }
</script>
<script src="/assets/js/app.js"></script>
<script src="/assets/js/migrate-wizard.js"></script>
<script src="/assets/js/php-manager.js"></script>
<script src="/assets/js/vhost-manager.js"></script>

</body>
</html>
