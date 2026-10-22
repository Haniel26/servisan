<?php

declare(strict_types=1);

// Test harness for the dashboard action guards. It fakes a request to
// www/index.php and prints whatever JSON the action returns.
//
// Usage: php dashboard_action_guard.php <method> <remote_addr> <env_token>
//        <header_token> <action> [base64_post_json]

$method = $argv[1] ?? 'GET';
$remoteAddr = $argv[2] ?? '127.0.0.1';
$envToken = $argv[3] ?? '';
$headerToken = $argv[4] ?? '';
$action = $argv[5] ?? '';
$postPayload = $argv[6] ?? '';

if ($envToken === '') {
    putenv('SERVISAN_DASHBOARD_TOKEN');
} else {
    putenv('SERVISAN_DASHBOARD_TOKEN=' . $envToken);
}

$_SERVER['REQUEST_METHOD'] = $method;
$_SERVER['REMOTE_ADDR'] = $remoteAddr;
$_SERVER['HTTP_X_SERVISAN_TOKEN'] = $headerToken;
$_GET['action'] = $action;

if ($postPayload !== '') {
    $decoded = json_decode((string) base64_decode($postPayload, true), true);
    if (is_array($decoded)) {
        $_POST = $decoded;
    }
}

require __DIR__ . '/../www/index.php';
