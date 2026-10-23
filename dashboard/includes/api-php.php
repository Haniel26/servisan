<?php

// PHP Version Manager actions: php_list, php_switch, php_install, php_install_status, php_delete

if (!isset($_GET['action'])) {
    return;
}

$action = $_GET['action'];

// Handle php_list action (list installed versions and download targets via servisan php list --json)
if ($action === 'php_list') {
    $exe = get_servisan_cmd($docRoot);
    $output = @shell_exec($exe . ' php list --json 2>&1');
    $data = json_decode($output ? $output : '', true);
    header('Content-Type: application/json');
    echo json_encode(array(
        'ok' => is_array($data),
        'installed' => (isset($data['installed']) && is_array($data['installed'])) ? $data['installed'] : array(),
        'available' => (isset($data['available']) && is_array($data['available'])) ? $data['available'] : array(),
        'active_php_version' => PHP_VERSION,
        'raw' => $output,
    ));
    exit;
}

// Handle php_switch action
if ($action === 'php_switch') {
    $exe = get_servisan_cmd($docRoot);
    $target = isset($_POST['target']) ? trim($_POST['target']) : '';
    if ($target === '') {
        header('Content-Type: application/json');
        echo json_encode(array('ok' => false, 'error' => 'No target PHP version specified'));
        exit;
    }
    $cmd = $exe . ' php switch ' . escapeshellarg($target) . ' 2>&1';
    $output = @shell_exec($cmd);
    header('Content-Type: application/json');
    echo json_encode(array(
        'ok' => true,
        'cmd' => $cmd,
        'output' => $output ? $output : '',
    ));
    exit;
}

// Handle php_install action
if ($action === 'php_install') {
    $exe = get_servisan_cmd($docRoot);
    $version = isset($_POST['version']) ? trim($_POST['version']) : '';
    $switch = !empty($_POST['switch']) && $_POST['switch'] !== 'false' && $_POST['switch'] !== '0';
    if ($version === '') {
        header('Content-Type: application/json');
        echo json_encode(array('ok' => false, 'error' => 'No version specified'));
        exit;
    }
    $switchArg = $switch ? ' --switch' : '';
    $cmd = $exe . ' php install ' . escapeshellarg($version) . $switchArg . ' 2>&1';
    $output = @shell_exec($cmd);
    header('Content-Type: application/json');
    echo json_encode(array(
        'ok' => true,
        'cmd' => $cmd,
        'output' => $output ? $output : '',
    ));
    exit;
}

// Handle php_install_status action (poll real-time download and extraction progress)
if ($action === 'php_install_status') {
    $version = isset($_GET['version']) ? trim($_GET['version']) : '';
    $tmpDir = dirname($docRoot) . DIRECTORY_SEPARATOR . 'tmp';

    // Look for any active download zip files
    $zipFiles = glob($tmpDir . DIRECTORY_SEPARATOR . 'php-*-download.zip');
    $bytes = 0;
    $activeFile = null;
    if (!empty($zipFiles)) {
        $activeFile = $zipFiles[0];
        clearstatcache(true, $activeFile);
        $bytes = (int)@filesize($activeFile);
    }

    // Check status file if present
    $statusFile = $tmpDir . DIRECTORY_SEPARATOR . 'php-install.status';
    $statusContent = '';
    if (file_exists($statusFile)) {
        $statusContent = trim((string)@file_get_contents($statusFile));
    }

    // Official x64 zip is approximately 35 MB
    $estimatedTotal = 36700160;

    $stage = 'connecting';
    if ($bytes > 0) {
        $stage = 'downloading';
    }
    if (substr($statusContent, 0, 10) === 'extracting') {
        $stage = 'extracting';
    } elseif (substr($statusContent, 0, 11) === 'configuring') {
        $stage = 'configuring';
    } elseif (substr($statusContent, 0, 4) === 'done') {
        $stage = 'done';
    }

    header('Content-Type: application/json');
    echo json_encode(array(
        'ok' => true,
        'stage' => $stage,
        'bytes' => $bytes,
        'estimated_total' => $estimatedTotal,
        'status_raw' => $statusContent,
        'file' => $activeFile ? basename($activeFile) : null,
    ));
    exit;
}

// Handle php_delete action (uninstall and remove an inactive PHP runtime from bin/php/*)
if ($action === 'php_delete') {
    $exe = get_servisan_cmd($docRoot);
    $target = isset($_POST['target']) ? trim($_POST['target']) : '';
    if ($target === '') {
        header('Content-Type: application/json');
        echo json_encode(array('ok' => false, 'error' => 'No target specified'));
        exit;
    }
    $cmd = $exe . ' php remove ' . escapeshellarg($target) . ' 2>&1';
    $output = @shell_exec($cmd);
    header('Content-Type: application/json');
    echo json_encode(array(
        'ok' => true,
        'cmd' => $cmd,
        'output' => $output ? $output : '',
    ));
    exit;
}
