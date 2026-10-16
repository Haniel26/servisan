<?php

declare(strict_types=1);

$mysqlHost = '127.0.0.1';
$mysqlPort = 3307;
$mysqlUser = 'root';
$mysqlPass = '';

function check_mysql(string $host, int $port, string $user, string $pass): array
{
    if (class_exists('mysqli')) {
        $conn = @new mysqli($host, $user, $pass, null, $port);
        if ($conn->connect_errno) {
            return [false, 'mysqli: ' . $conn->connect_error];
        }
        $version = (string) $conn->server_info;
        $conn->close();
        return [true, $version];
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

$extensions = get_loaded_extensions();
sort($extensions);

?>
<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Servisan</title>
<link rel="icon" href="/favicon.ico" sizes="32x32" type="image/x-icon">
<style>
    :root { color-scheme: light dark; }
    body { font-family: system-ui, Segoe UI, sans-serif; max-width: 42rem; margin: 3rem auto; padding: 0 1.5rem; line-height: 1.5; }
    h1 { margin-bottom: .25rem; }
    .sub { color: #888; margin-top: 0; }
    .card { border: 1px solid #8884; border-radius: .5rem; padding: 1rem 1.25rem; margin: 1rem 0; }
    .row { display: flex; justify-content: space-between; gap: 1rem; padding: .2rem 0; }
    .ok { color: #1a9b52; font-weight: 600; }
    .bad { color: #d23f3f; font-weight: 600; }
    code { background: #8882; padding: .1rem .35rem; border-radius: .25rem; }
    ul.ext { columns: 3; font-size: .9rem; padding-left: 1.2rem; }
</style>
</head>
<body>
<div style="display:flex; align-items:center; gap:.75rem;">
  <img src="/logo.png" alt="Servisan" width="64" height="64" style="border-radius:.5rem;">
  <div>
    <h1 style="margin:0;">Servisan</h1>
    <p class="sub" style="margin:0;">PHP built-in server is running.</p>
  </div>
</div>

<div class="card">
    <div class="row"><span>PHP version</span><span><?= htmlspecialchars(PHP_VERSION) ?></span></div>
    <div class="row"><span>SAPI</span><span><?= htmlspecialchars(PHP_SAPI) ?></span></div>
    <div class="row"><span>Document root</span><span><code><?= htmlspecialchars($_SERVER['DOCUMENT_ROOT'] ?? '') ?></code></span></div>
</div>

<div class="card">
    <div class="row">
        <span>MySQL <code><?= htmlspecialchars($mysqlHost) ?>:<?= $mysqlPort ?></code></span>
        <?php if ($mysqlOk): ?>
            <span class="ok">connected (<?= htmlspecialchars($mysqlInfo) ?>)</span>
        <?php else: ?>
            <span class="bad">unavailable</span>
        <?php endif; ?>
    </div>
    <?php if (!$mysqlOk): ?>
        <p class="sub" style="margin:.5rem 0 0;font-size:.85rem"><?= htmlspecialchars($mysqlInfo) ?></p>
        <p class="sub" style="margin:.5rem 0 0;font-size:.85rem">Start MySQL from the Servisan tray menu, then reload.</p>
    <?php endif; ?>
</div>

<div class="card">
    <div class="row"><span>Loaded extensions</span><span><?= count($extensions) ?></span></div>
    <ul class="ext">
        <?php foreach ($extensions as $ext): ?>
            <li><?= htmlspecialchars($ext) ?></li>
        <?php endforeach; ?>
    </ul>
</div>

<p class="sub" style="font-size:.85rem">
    Replace this file with your project. Put subfolders under <code>www/</code> and browse to them directly.
</p>
</body>
</html>
