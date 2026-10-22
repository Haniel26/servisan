<?php

declare(strict_types=1);

// Regression test: dashboard actions must require loopback access, the
// manager token, POST for state changes, and safe database names.

$guard = __DIR__ . '/dashboard_action_guard.php';
$token = 'test-token-0123456789abcdef';

/**
 * @param list<string> $args
 * @return array{ok: bool, error?: string, content?: string, cmd?: string}
 */
function run_guard(string $guard, array $args): array
{
    $command = escapeshellarg(PHP_BINARY) . ' ' . escapeshellarg($guard);
    foreach ($args as $arg) {
        $command .= ' ' . escapeshellarg($arg);
    }

    $lines = [];
    $status = 0;
    exec($command . ' 2>&1', $lines, $status);

    $raw = implode("\n", $lines);
    $decoded = json_decode($raw, true);
    if (!is_array($decoded)) {
        fwrite(STDERR, "Guard returned invalid JSON (exit {$status}):\n{$raw}\n");
        exit(1);
    }

    return $decoded;
}

$failures = [];

// Loopback + valid token + read-only action is allowed.
$allowed = run_guard($guard, ['GET', '127.0.0.1', $token, $token, 'get_logs']);
if (($allowed['ok'] ?? false) !== true || !array_key_exists('content', $allowed)) {
    $failures[] = 'authorised get_logs should succeed';
}

// State-changing actions must use POST.
$getWrite = run_guard($guard, ['GET', '127.0.0.1', $token, $token, 'open_logs']);
if (($getWrite['ok'] ?? true) !== false || !str_contains((string) ($getWrite['error'] ?? ''), 'POST')) {
    $failures[] = 'GET on a write action should be rejected with 405';
}

// A wrong token is rejected.
$badToken = run_guard($guard, ['POST', '127.0.0.1', $token, 'wrong-token', 'open_logs']);
if (($badToken['ok'] ?? true) !== false || !str_contains((string) ($badToken['error'] ?? ''), 'token')) {
    $failures[] = 'invalid token should be rejected';
}

// Requests from another machine are rejected.
$remote = run_guard($guard, ['POST', '203.0.113.9', $token, $token, 'open_logs']);
if (($remote['ok'] ?? true) !== false || !str_contains((string) ($remote['error'] ?? ''), 'this computer')) {
    $failures[] = 'non-loopback request should be rejected';
}

// Without the manager there is no token, so actions stay disabled.
$noManager = run_guard($guard, ['POST', '127.0.0.1', '', '', 'open_logs']);
if (($noManager['ok'] ?? true) !== false || !str_contains((string) ($noManager['error'] ?? ''), 'Servisan manager')) {
    $failures[] = 'missing manager token should be rejected';
}

// Database names that look like CLI flags must never reach the shell.
$injection = run_guard($guard, [
    'POST',
    '127.0.0.1',
    $token,
    $token,
    'migrate_import',
    base64_encode(json_encode(['databases' => ['--overwrite'], 'from' => ''])),
]);
if (($injection['ok'] ?? true) !== false || !str_contains((string) ($injection['error'] ?? ''), 'tidak valid')) {
    $failures[] = 'flag-like database names should be rejected';
}

// Shell metacharacters and the characters escapeshellarg() mangles on
// Windows are rejected before the CLI is spawned.
$metachars = run_guard($guard, [
    'POST',
    '127.0.0.1',
    $token,
    $token,
    'migrate_import',
    base64_encode(json_encode(['databases' => ['app; rm -rf /'], 'from' => ''])),
]);
if (($metachars['ok'] ?? true) !== false || !str_contains((string) ($metachars['error'] ?? ''), 'tidak valid')) {
    $failures[] = 'database names with shell metacharacters should be rejected';
}

// An empty list must not fall through to "import everything".
$emptyList = run_guard($guard, [
    'POST',
    '127.0.0.1',
    $token,
    $token,
    'migrate_import',
    base64_encode(json_encode(['databases' => [], 'from' => ''])),
]);
if (($emptyList['ok'] ?? true) !== false || !str_contains((string) ($emptyList['error'] ?? ''), 'setidaknya satu')) {
    $failures[] = 'empty database selection should be rejected';
}

// Nested or non-string values must not reach array_map()/the shell.
$nonString = run_guard($guard, [
    'POST',
    '127.0.0.1',
    $token,
    $token,
    'migrate_import',
    base64_encode(json_encode(['databases' => [['app']], 'from' => ''])),
]);
if (($nonString['ok'] ?? true) !== false || !str_contains((string) ($nonString['error'] ?? ''), 'tidak valid')) {
    $failures[] = 'non-string database entries should be rejected';
}

$unknownFolder = run_guard($guard, [
    'POST',
    '127.0.0.1',
    $token,
    $token,
    'migrate_import',
    base64_encode(json_encode(['databases' => ['app'], 'from' => '/definitely/not/a/folder/'])),
]);
if (($unknownFolder['ok'] ?? true) !== false || !str_contains((string) ($unknownFolder['error'] ?? ''), 'tidak ditemukan')) {
    $failures[] = 'missing source folder should be rejected';
}

if ($failures !== []) {
    fwrite(STDERR, 'Dashboard security test failed: ' . implode(', ', $failures) . PHP_EOL);
    exit(1);
}

echo 'Dashboard security test passed.' . PHP_EOL;
