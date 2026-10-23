<?php

// Database Migration actions: migrate_scan, migrate_import

if (!isset($_GET['action'])) {
    return;
}

$action = $_GET['action'];

// Handle migrate_scan action (scan for local server databases via servisan migrate list --json)
if ($action === 'migrate_scan') {
    $exe = get_servisan_cmd($docRoot);
    $from = isset($_GET['from']) ? trim($_GET['from']) : '';
    $fromArg = $from !== '' ? ' --from ' . escapeshellarg($from) : '';
    $output = @shell_exec($exe . ' migrate list' . $fromArg . ' --json 2>&1');
    $sources = json_decode($output ? $output : '', true);
    header('Content-Type: application/json');
    echo json_encode(array(
        'ok' => is_array($sources),
        'sources' => is_array($sources) ? $sources : array(),
        'servisan_mysql_ok' => isset($mysqlOk) ? $mysqlOk : false,
        'from' => $from,
        'raw' => $output,
    ));
    exit;
}

// Handle migrate_import action (import selected databases into Servisan MySQL)
if ($action === 'migrate_import') {
    $exe = get_servisan_cmd($docRoot);
    $dbs = isset($_POST['databases']) ? $_POST['databases'] : array();
    if (is_string($dbs)) {
        $dbs = array_filter(array_map('trim', explode(',', $dbs)));
    }
    $overwrite = !empty($_POST['overwrite']) && $_POST['overwrite'] !== 'false' && $_POST['overwrite'] !== '0';
    $from = isset($_POST['from']) ? trim($_POST['from']) : '';

    $args = '';
    if ($from !== '') {
        $args .= ' --from ' . escapeshellarg($from);
    }
    if (!empty($dbs) && is_array($dbs)) {
        foreach ($dbs as $db) {
            $args .= ' ' . escapeshellarg($db);
        }
    }
    if ($overwrite) {
        $args .= ' --overwrite';
    }

    $cmd = $exe . ' migrate import' . $args . ' 2>&1';
    $output = @shell_exec($cmd);
    header('Content-Type: application/json');
    echo json_encode(array(
        'ok' => true,
        'cmd' => $cmd,
        'output' => $output ? $output : '',
    ));
    exit;
}
