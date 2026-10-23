<?php

// System actions: open_root, open_terminal, shutdown, open_logs, get_logs

if (!isset($_GET['action'])) {
    return;
}

$action = $_GET['action'];

// Handle open_root action (open Windows File Explorer directly)
if ($action === 'open_root') {
    if ($isWindows) {
        pclose(popen('explorer ' . escapeshellarg($docRoot), 'r'));
    }
    header('Content-Type: application/json');
    echo json_encode(array('ok' => true));
    exit;
}

// Handle open_terminal action (open Command Prompt with servisan shell)
if ($action === 'open_terminal') {
    if ($isWindows) {
        $rootDir = get_servisan_home();
        $exe = get_servisan_cmd();
        $cmd = 'start "Servisan Shell" cmd.exe /k "cd /d "' . $rootDir . '" && ' . $exe . ' shell"';
        pclose(popen($cmd, 'r'));
    }
    header('Content-Type: application/json');
    echo json_encode(array('ok' => true));
    exit;
}

// Handle shutdown action (gracefully stop all services and terminate background servisan process)
if ($action === 'shutdown') {
    if ($isWindows) {
        $exe = get_servisan_cmd();
        // Execute graceful stop followed by tray taskkill in a detached process with a 1s pause
        // so PHP has ample time to flush the HTTP 200 response cleanly to the client before exiting.
        $cmd = 'cmd /c "start /b "" cmd /c (timeout /t 1 /nobreak >nul & ' . $exe . ' stop & taskkill /F /IM servisan.exe >nul 2>&1)"';
        @pclose(@popen($cmd, 'r'));
    }
    header('Content-Type: application/json');
    echo json_encode(array(
        'ok' => true,
        'message' => 'Servisan services and tray process are shutting down.'
    ));
    exit;
}

// Handle open_logs action (open logs folder in Windows File Explorer)
if ($action === 'open_logs') {
    if ($isWindows) {
        $logsDir = get_servisan_home() . DIRECTORY_SEPARATOR . 'logs';
        if (!is_dir($logsDir)) {
            @mkdir($logsDir, 0777, true);
        }
        pclose(popen('explorer ' . escapeshellarg($logsDir), 'r'));
    }
    header('Content-Type: application/json');
    echo json_encode(array('ok' => true));
    exit;
}

// Handle get_logs action (return last 50 lines in JSON)
if ($action === 'get_logs') {
    $type = isset($_GET['type']) ? $_GET['type'] : 'php';
    $logsDir = get_servisan_home() . DIRECTORY_SEPARATOR . 'logs';
    switch ($type) {
        case 'mysql':
            $filename = 'mysql.log';
            break;
        case 'servisan':
            $filename = 'servisan.log';
            break;
        default:
            $filename = 'php.log';
            break;
    }
    $content = tail_file($logsDir . DIRECTORY_SEPARATOR . $filename, 50);
    header('Content-Type: application/json');
    echo json_encode(array(
        'ok' => true,
        'type' => $type,
        'filename' => $filename,
        'content' => $content,
    ));
    exit;
}
