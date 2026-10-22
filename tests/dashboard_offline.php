<?php

declare(strict_types=1);

// Regression test: the dashboard must render when MySQL is offline.
putenv('SERVISAN_MYSQL_HOST=127.0.0.1');
putenv('SERVISAN_MYSQL_PORT=54321');
putenv('SERVISAN_VERSION=0.0.0-test');
putenv('SERVISAN_DASHBOARD_TOKEN=test-dashboard-token');
$_SERVER['REMOTE_ADDR'] = '127.0.0.1';

ob_start();
require __DIR__ . '/../www/index.php';
$html = (string) ob_get_clean();

$checks = [
    'no fatal error' => !str_contains($html, 'Fatal error'),
    'MySQL shown as stopped' => str_contains($html, 'stopped'),
    'configured port rendered' => str_contains($html, '54321') && !str_contains($html, '3307'),
    'version rendered' => str_contains($html, '0.0.0-test'),
    'log text is selectable' => str_contains($html, 'user-select: text'),
    'dev fallback skips update check' => str_contains($html, "CURRENT_VERSION === 'dev'"),
    'dashboard token rendered' => str_contains($html, 'test-dashboard-token'),
    'actions enabled for local manager' => !str_contains($html, 'Aksi dashboard (terminal, logs, migrasi) dinonaktifkan'),
];

$failures = array_keys(array_filter($checks, static fn (bool $ok): bool => !$ok));
if ($failures !== []) {
    fwrite(STDERR, 'Dashboard regression test failed: ' . implode(', ', $failures) . PHP_EOL);
    exit(1);
}

echo 'Dashboard regression test passed.' . PHP_EOL;
