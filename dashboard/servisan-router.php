<?php
// servisan-router.php - Servisan Local Development Server & VHost Router
// Compatible with PHP 5.6 through PHP 8.5+

function servisan_strip_prefix($path) {
    if (substr($path, 0, 4) === "\\\\?\\") {
        return substr($path, 4);
    }
    return $path;
}

function servisan_mime_type($file) {
    $ext = strtolower(pathinfo($file, PATHINFO_EXTENSION));
    $mimes = array(
        'css'   => 'text/css',
        'js'    => 'application/javascript',
        'json'  => 'application/json',
        'png'   => 'image/png',
        'jpg'   => 'image/jpeg',
        'jpeg'  => 'image/jpeg',
        'gif'   => 'image/gif',
        'svg'   => 'image/svg+xml',
        'ico'   => 'image/x-icon',
        'webp'  => 'image/webp',
        'woff'  => 'font/woff',
        'woff2' => 'font/woff2',
        'ttf'   => 'font/ttf',
        'eot'   => 'application/vnd.ms-fontobject',
        'pdf'   => 'application/pdf',
        'zip'   => 'application/zip',
        'txt'   => 'text/plain; charset=UTF-8',
        'html'  => 'text/html; charset=UTF-8',
        'htm'   => 'text/html; charset=UTF-8',
        'mp4'   => 'video/mp4',
        'webm'  => 'video/webm',
        'mp3'   => 'audio/mpeg',
        'map'   => 'application/json',
        'xml'   => 'application/xml',
    );
    return isset($mimes[$ext]) ? $mimes[$ext] : 'application/octet-stream';
}

function servisan_serve_static($file) {
    $mime = servisan_mime_type($file);
    header('Content-Type: ' . $mime);
    header('Content-Length: ' . filesize($file));
    readfile($file);
    exit;
}

function servisan_serve_index_php($indexFile, $docRoot, $scriptName) {
    chdir(dirname($indexFile));
    $_SERVER['SCRIPT_FILENAME'] = $indexFile;
    $_SERVER['DOCUMENT_ROOT']   = $docRoot;
    $_SERVER['SCRIPT_NAME']     = $scriptName;
    $_SERVER['PHP_SELF']        = $scriptName;
    require $indexFile;
    exit;
}

function servisan_render_directory_listing($title, $dirPath, $urlPrefix, $vhostName, $isDomain) {
    $items = @scandir($dirPath);
    if (!is_array($items)) {
        $items = array();
    }

    $directories = array();
    $files = array();

    foreach ($items as $item) {
        if ($item === '.' || $item === '..') {
            continue;
        }
        $full = rtrim($dirPath, '/\\') . DIRECTORY_SEPARATOR . $item;
        if (is_dir($full)) {
            $directories[] = $item;
        } else {
            $files[] = array(
                'name' => $item,
                'size' => filesize($full),
                'mtime' => filemtime($full),
            );
        }
    }

    sort($directories);
    usort($files, function($a, $b) {
        return strcmp($a['name'], $b['name']);
    });

    $port = isset($_SERVER['SERVER_PORT']) ? (int)$_SERVER['SERVER_PORT'] : 8080;
    $portSuffix = ($port === 80) ? '' : ':' . $port;

    header('Content-Type: text/html; charset=UTF-8');
    ?>
<!DOCTYPE html>
<html lang="id">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title><?= htmlspecialchars($vhostName) ?> &bull; Servisan Project Explorer</title>
    <link rel="icon" type="image/x-icon" href="/favicon.ico">
    <style>
        * { box-sizing: border-box; }
        body {
            font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
            background: #f8fafc;
            color: #0f172a;
            margin: 0;
            padding: 24px;
        }
        .container {
            max-width: 900px;
            margin: 0 auto;
            background: #ffffff;
            border: 1px solid #e2e8f0;
            border-radius: 8px;
            box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.05);
            overflow: hidden;
        }
        .header {
            background: #0f172a;
            color: #ffffff;
            padding: 16px 20px;
            display: flex;
            justify-content: space-between;
            align-items: center;
        }
        .header-left {
            display: flex;
            align-items: center;
            gap: 12px;
        }
        .header-title {
            font-size: 16px;
            font-weight: 600;
        }
        .header-meta {
            font-size: 12px;
            color: #94a3b8;
            margin-top: 2px;
            font-family: Consolas, monospace;
        }
        .badge {
            background: #0284c7;
            color: #ffffff;
            font-size: 11px;
            font-weight: 600;
            padding: 2px 8px;
            border-radius: 9999px;
            text-transform: uppercase;
        }
        .nav-links a {
            color: #38bdf8;
            text-decoration: none;
            font-size: 12.5px;
            display: inline-flex;
            align-items: center;
            gap: 4px;
        }
        .notice-banner {
            background: #f0f9ff;
            border-bottom: 1px solid #bae6fd;
            padding: 12px 20px;
            font-size: 12.5px;
            color: #0369a1;
            line-height: 1.5;
            display: flex;
            align-items: flex-start;
            gap: 10px;
        }
        .file-list {
            list-style: none;
            margin: 0;
            padding: 0;
        }
        .file-item {
            display: flex;
            align-items: center;
            justify-content: space-between;
            padding: 10px 20px;
            border-bottom: 1px solid #f1f5f9;
            font-size: 13px;
            transition: background 0.15s;
        }
        .file-item:last-child {
            border-bottom: none;
        }
        .file-item:hover {
            background: #f8fafc;
        }
        .file-left {
            display: flex;
            align-items: center;
            gap: 10px;
            min-width: 0;
        }
        .file-name {
            color: #0284c7;
            text-decoration: none;
            font-weight: 500;
            font-family: Consolas, monospace;
            word-break: break-all;
        }
        .file-name:hover {
            text-decoration: underline;
        }
        .file-name.is-dir {
            color: #0f172a;
            font-weight: 600;
        }
        .file-meta {
            font-size: 11.5px;
            color: #64748b;
            font-family: Consolas, monospace;
            flex-shrink: 0;
        }
        .footer {
            background: #f8fafc;
            border-top: 1px solid #e2e8f0;
            padding: 12px 20px;
            font-size: 12px;
            color: #64748b;
            display: flex;
            justify-content: space-between;
            align-items: center;
        }
    </style>
</head>
<body>
<div class="container">
    <div class="header">
        <div class="header-left">
            <div>
                <div style="display:flex;align-items:center;gap:8px;">
                    <span class="header-title"><?= htmlspecialchars($vhostName) ?></span>
                    <span class="badge"><?= $isDomain ? 'Virtual Host' : 'Linked Subfolder' ?></span>
                </div>
                <div class="header-meta"><?= htmlspecialchars($dirPath) ?></div>
            </div>
        </div>
        <div class="nav-links">
            <a href="http://localhost<?= $portSuffix ?>/">
                <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M3 9l9-7 9 7v11a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"></path><polyline points="9 22 9 12 15 12 15 22"></polyline></svg>
                <span>Dashboard Servisan</span>
            </a>
        </div>
    </div>

    <div class="notice-banner">
        <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" style="flex-shrink:0;margin-top:2px;">
            <circle cx="12" cy="12" r="10"></circle>
            <line x1="12" y1="16" x2="12" y2="12"></line>
            <line x1="12" y1="8" x2="12.01" y2="8"></line>
        </svg>
        <div>
            Project ini berhasil terhubung ke Servisan. Folder ini belum memiliki file <code>index.php</code> atau <code>index.html</code> di root. Anda dapat menjelajahi file dan subfolder di bawah ini:
        </div>
    </div>

    <ul class="file-list">
        <?php if ($urlPrefix !== '/' && $urlPrefix !== ''): ?>
            <li class="file-item">
                <div class="file-left">
                    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="#64748b" stroke-width="2"><polyline points="15 18 9 12 15 6"></polyline></svg>
                    <a href="<?= htmlspecialchars(dirname($urlPrefix)) ?>/" class="file-name is-dir">.. (Direktori Induk)</a>
                </div>
            </li>
        <?php endif; ?>

        <?php foreach ($directories as $dir): ?>
            <?php $dirLink = rtrim($urlPrefix, '/') . '/' . rawurlencode($dir) . '/'; ?>
            <li class="file-item">
                <div class="file-left">
                    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="#0284c7" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"></path></svg>
                    <a href="<?= htmlspecialchars($dirLink) ?>" class="file-name is-dir"><?= htmlspecialchars($dir) ?>/</a>
                </div>
                <span class="file-meta">Folder</span>
            </li>
        <?php endforeach; ?>

        <?php foreach ($files as $f): ?>
            <?php
            $fileLink = rtrim($urlPrefix, '/') . '/' . rawurlencode($f['name']);
            $sizeKb = number_format($f['size'] / 1024, 1) . ' KB';
            $date = date('d M Y H:i', $f['mtime']);
            ?>
            <li class="file-item">
                <div class="file-left">
                    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="#64748b" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"></path><polyline points="14 2 14 8 20 8"></polyline></svg>
                    <a href="<?= htmlspecialchars($fileLink) ?>" class="file-name"><?= htmlspecialchars($f['name']) ?></a>
                </div>
                <span class="file-meta"><?= $sizeKb ?> &bull; <?= $date ?></span>
            </li>
        <?php endforeach; ?>

        <?php if (empty($directories) && empty($files)): ?>
            <li class="file-item" style="color: #64748b; font-style: italic;">Direktori ini kosong.</li>
        <?php endif; ?>
    </ul>

    <div class="footer">
        <span>Servisan Virtual Host Router</span>
        <span>PHP <?= htmlspecialchars(PHP_VERSION) ?></span>
    </div>
</div>
</body>
</html>
    <?php
    exit;
}

// -----------------------------------------------------------------------------
// MAIN ROUTING LOGIC
// -----------------------------------------------------------------------------

$dashboardDir = dirname(__FILE__);
$servisanHome = getenv('SERVISAN_HOME') ? getenv('SERVISAN_HOME') : dirname($dashboardDir);
$wwwDir = getenv('SERVISAN_WWW_DIR') ? getenv('SERVISAN_WWW_DIR') : ($servisanHome . DIRECTORY_SEPARATOR . 'www');

$host = isset($_SERVER['HTTP_HOST']) ? strtolower($_SERVER['HTTP_HOST']) : '';
$hostParts = explode(':', $host);
$cleanHost = $hostParts[0];

$rawUri = isset($_SERVER['REQUEST_URI']) ? $_SERVER['REQUEST_URI'] : '/';
$reqPath = urldecode(parse_url($rawUri, PHP_URL_PATH));
if ($reqPath === null || $reqPath === false) {
    $reqPath = '/';
}

$vhostsFile = $servisanHome . DIRECTORY_SEPARATOR . 'config' . DIRECTORY_SEPARATOR . 'vhosts.json';
$vhosts = array();
if (file_exists($vhostsFile)) {
    $decoded = json_decode(@file_get_contents($vhostsFile), true);
    if (is_array($decoded)) {
        $vhosts = $decoded;
    }
}

// -----------------------------------------------------------------------------
// 1. Servisan Dashboard Routing on localhost (http://localhost:8080/)
// -----------------------------------------------------------------------------
if ($cleanHost === 'localhost' || $cleanHost === '127.0.0.1' || $cleanHost === '') {
    // Root dashboard page
    if ($reqPath === '/' || $reqPath === '/index.php') {
        $indexPhp = $dashboardDir . DIRECTORY_SEPARATOR . 'index.php';
        if (is_file($indexPhp)) {
            servisan_serve_index_php($indexPhp, $dashboardDir, $reqPath);
        }
    }

    // Dashboard favicon
    if ($reqPath === '/favicon.ico') {
        $fav = $dashboardDir . DIRECTORY_SEPARATOR . 'favicon.ico';
        if (is_file($fav)) {
            servisan_serve_static($fav);
        }
    }

    // Dashboard assets (/assets/css/..., /assets/js/...)
    if (substr($reqPath, 0, 8) === '/assets/') {
        $assetFile = $dashboardDir . str_replace('/', DIRECTORY_SEPARATOR, $reqPath);
        if (is_file($assetFile)) {
            servisan_serve_static($assetFile);
        }
    }

    // Protect dashboard internal directories
    if (substr($reqPath, 0, 10) === '/includes/' || substr($reqPath, 0, 7) === '/views/') {
        header('HTTP/1.1 403 Forbidden');
        echo 'Forbidden';
        exit;
    }
}

// -----------------------------------------------------------------------------
// 2. Virtual Host via Domain (*.test / custom domain)
// -----------------------------------------------------------------------------
if ($cleanHost !== 'localhost' && $cleanHost !== '127.0.0.1' && $cleanHost !== '') {
    $matchedVhost = null;
    foreach ($vhosts as $vh) {
        $vDomain = isset($vh['domain']) ? strtolower($vh['domain']) : '';
        $vName = isset($vh['name']) ? strtolower($vh['name']) : '';
        if ($vDomain === $cleanHost || ($vName !== '' && ($vName . '.test' === $cleanHost || $vName . '.local' === $cleanHost))) {
            $matchedVhost = $vh;
            break;
        }
    }

    if ($matchedVhost !== null) {
        $docRoot = servisan_strip_prefix(isset($matchedVhost['doc_root']) ? $matchedVhost['doc_root'] : '');
        if (!$docRoot || !is_dir($docRoot)) {
            // Target folder missing on disk
            header('HTTP/1.1 500 Internal Server Error');
            echo "Directory target tidak ditemukan: " . htmlspecialchars($docRoot);
            exit;
        }

        $targetPath = rtrim($docRoot, '/\\') . str_replace('/', DIRECTORY_SEPARATOR, $reqPath);

        // Case A: Existing file
        if (is_file($targetPath)) {
            if (strtolower(pathinfo($targetPath, PATHINFO_EXTENSION)) === 'php') {
                servisan_serve_index_php($targetPath, $docRoot, $reqPath);
            } else {
                servisan_serve_static($targetPath);
            }
        }

        // Case B: Existing directory
        if (is_dir($targetPath)) {
            // Check trailing slash
            if (substr($rawUri, -1) !== '/' && parse_url($rawUri, PHP_URL_PATH) !== '') {
                $query = parse_url($rawUri, PHP_URL_QUERY);
                $redirectUrl = parse_url($rawUri, PHP_URL_PATH) . '/' . ($query ? '?' . $query : '');
                header('Location: ' . $redirectUrl, true, 302);
                exit;
            }

            $idxPhp = rtrim($targetPath, '/\\') . DIRECTORY_SEPARATOR . 'index.php';
            if (is_file($idxPhp)) {
                servisan_serve_index_php($idxPhp, $docRoot, rtrim($reqPath, '/') . '/index.php');
            }

            $idxHtml = rtrim($targetPath, '/\\') . DIRECTORY_SEPARATOR . 'index.html';
            if (is_file($idxHtml)) {
                servisan_serve_static($idxHtml);
            }

            // Neither index.php nor index.html: render Servisan Project Explorer
            servisan_render_directory_listing(
                $matchedVhost['name'],
                $targetPath,
                $reqPath,
                $matchedVhost['domain'],
                true
            );
        }

        // Case C: File not found on disk -> Check front-controller for SPA / Framework (Laravel routes)
        $rootIndexPhp = rtrim($docRoot, '/\\') . DIRECTORY_SEPARATOR . 'index.php';
        if (is_file($rootIndexPhp)) {
            servisan_serve_index_php($rootIndexPhp, $docRoot, '/index.php');
        }

        $rootIndexHtml = rtrim($docRoot, '/\\') . DIRECTORY_SEPARATOR . 'index.html';
        if (is_file($rootIndexHtml)) {
            servisan_serve_static($rootIndexHtml);
        }

        // Not found
        header('HTTP/1.1 404 Not Found');
        echo "404 Not Found: " . htmlspecialchars($reqPath) . " pada virtual host " . htmlspecialchars($cleanHost);
        exit;
    }
}

// -----------------------------------------------------------------------------
// 3. Subfolder Projects Access on localhost (http://localhost:8080/<project_name>/...)
// -----------------------------------------------------------------------------
if ($cleanHost === 'localhost' || $cleanHost === '127.0.0.1') {
    // Strip leading slash and find first segment
    $trimmed = ltrim($reqPath, '/');
    $parts = explode('/', $trimmed);
    $firstSegment = isset($parts[0]) ? $parts[0] : '';

    if ($firstSegment !== '' && $firstSegment !== 'phpmyadmin') {
        $matchedVhost = null;
        foreach ($vhosts as $vh) {
            $vName = isset($vh['name']) ? $vh['name'] : '';
            if (strcasecmp($vName, $firstSegment) === 0) {
                $matchedVhost = $vh;
                break;
            }
        }

        $docRoot = null;
        $displayName = $firstSegment;

        if ($matchedVhost !== null) {
            $docRoot = servisan_strip_prefix(isset($matchedVhost['doc_root']) ? $matchedVhost['doc_root'] : '');
            $displayName = $matchedVhost['name'];
        } else {
            // Check if folder exists directly inside www/
            $candidateWwwDir = $wwwDir . DIRECTORY_SEPARATOR . $firstSegment;
            if (is_dir($candidateWwwDir)) {
                $publicDir = $candidateWwwDir . DIRECTORY_SEPARATOR . 'public';
                $docRoot = is_dir($publicDir) ? $publicDir : $candidateWwwDir;
            }
        }

        if ($docRoot && is_dir($docRoot)) {
            // Ensure trailing slash on folder base
            if (count($parts) === 1 && substr($rawUri, -1) !== '/') {
                $query = parse_url($rawUri, PHP_URL_QUERY);
                $redirectUrl = '/' . $firstSegment . '/' . ($query ? '?' . $query : '');
                header('Location: ' . $redirectUrl, true, 302);
                exit;
            }

            // Subpath within project
            $subPath = substr($trimmed, strlen($firstSegment));
            if ($subPath === '' || $subPath === false) {
                $subPath = '/';
            }
            $targetPath = rtrim($docRoot, '/\\') . str_replace('/', DIRECTORY_SEPARATOR, $subPath);

            if (is_file($targetPath)) {
                if (strtolower(pathinfo($targetPath, PATHINFO_EXTENSION)) === 'php') {
                    servisan_serve_index_php($targetPath, $docRoot, $reqPath);
                } else {
                    servisan_serve_static($targetPath);
                }
            }

            if (is_dir($targetPath)) {
                $idxPhp = rtrim($targetPath, '/\\') . DIRECTORY_SEPARATOR . 'index.php';
                if (is_file($idxPhp)) {
                    servisan_serve_index_php($idxPhp, $docRoot, rtrim($reqPath, '/') . '/index.php');
                }

                $idxHtml = rtrim($targetPath, '/\\') . DIRECTORY_SEPARATOR . 'index.html';
                if (is_file($idxHtml)) {
                    servisan_serve_static($idxHtml);
                }

                // Render directory listing
                servisan_render_directory_listing(
                    $displayName,
                    $targetPath,
                    '/' . $firstSegment . (rtrim($subPath, '/') ? rtrim($subPath, '/') : ''),
                    $displayName,
                    false
                );
            }

            // Fallback for Laravel/SPA under subfolder
            $rootIndexPhp = rtrim($docRoot, '/\\') . DIRECTORY_SEPARATOR . 'index.php';
            if (is_file($rootIndexPhp)) {
                servisan_serve_index_php($rootIndexPhp, $docRoot, $reqPath);
            }
        }
    }
}

// -----------------------------------------------------------------------------
// 4. Default fallback: serve files from www/ (e.g. phpmyadmin, etc.)
// -----------------------------------------------------------------------------
return false;
