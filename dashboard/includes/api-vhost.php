<?php

// Virtual Hosts actions: vhost_list, vhost_detect, vhost_add, vhost_remove, open_project_folder, open_project_terminal

if (!isset($_GET['action'])) {
    return;
}

$action = $_GET['action'];

// Handle vhost_list action (list registered virtual hosts)
if ($action === 'vhost_list') {
    $exe = get_servisan_cmd($docRoot);
    $output = @shell_exec($exe . ' vhost list --json 2>&1');
    $vhosts = json_decode($output ? $output : '', true);
    header('Content-Type: application/json');
    echo json_encode(array(
        'ok' => is_array($vhosts),
        'vhosts' => is_array($vhosts) ? $vhosts : array(),
        'port' => isset($_SERVER['SERVER_PORT']) ? (int)$_SERVER['SERVER_PORT'] : 8080,
        'raw' => $output,
    ));
    exit;
}

// Handle vhost_detect action (inspect folder, detect framework and suggest domain)
if ($action === 'vhost_detect') {
    $path = isset($_GET['path']) ? trim($_GET['path']) : '';
    if ($path === '' || !is_dir($path)) {
        header('Content-Type: application/json');
        echo json_encode(array('ok' => false, 'error' => 'Path folder tidak ditemukan atau bukan direktori.'));
        exit;
    }
    $publicDir = rtrim($path, '/\\') . DIRECTORY_SEPARATOR . 'public';
    $hasPublic = is_dir($publicDir);
    $isLaravel = file_exists(rtrim($path, '/\\') . DIRECTORY_SEPARATOR . 'artisan');
    $hasWp = file_exists(rtrim($path, '/\\') . DIRECTORY_SEPARATOR . 'wp-config.php');

    $docRootVhost = $hasPublic ? $publicDir : $path;
    $framework = 'Standard PHP (root)';
    if ($isLaravel) {
        $framework = 'Laravel (public/)';
    } elseif ($hasPublic) {
        $framework = 'Modern Framework (public/)';
    } elseif ($hasWp) {
        $framework = 'WordPress (root)';
    }

    $rawName = basename(rtrim($path, '/\\'));
    $slug = preg_replace('/[^a-zA-Z0-9]+/', '-', strtolower($rawName));
    $slug = trim($slug, '-');
    if ($slug === '') {
        $slug = 'project';
    }

    header('Content-Type: application/json');
    echo json_encode(array(
        'ok' => true,
        'path' => $path,
        'name' => $slug,
        'domain' => $slug . '.test',
        'doc_root' => $docRootVhost,
        'framework' => $framework,
    ));
    exit;
}

// Handle vhost_add action
if ($action === 'vhost_add') {
    $exe = get_servisan_cmd($docRoot);
    $path = isset($_POST['path']) ? trim($_POST['path']) : '';
    $name = isset($_POST['name']) ? trim($_POST['name']) : '';
    $domain = isset($_POST['domain']) ? trim($_POST['domain']) : '';
    if ($path === '') {
        header('Content-Type: application/json');
        echo json_encode(array('ok' => false, 'error' => 'Path project tidak boleh kosong.'));
        exit;
    }
    $args = ' ' . escapeshellarg($path);
    if ($name !== '') {
        $args .= ' --name ' . escapeshellarg($name);
    }
    if ($domain !== '') {
        $args .= ' --domain ' . escapeshellarg($domain);
    }
    $cmd = $exe . ' vhost add' . $args . ' 2>&1';
    $output = @shell_exec($cmd);
    header('Content-Type: application/json');
    echo json_encode(array(
        'ok' => true,
        'cmd' => $cmd,
        'output' => $output ? $output : '',
    ));
    exit;
}

// Handle vhost_remove action
if ($action === 'vhost_remove') {
    $exe = get_servisan_cmd($docRoot);
    $target = isset($_POST['target']) ? trim($_POST['target']) : '';
    if ($target === '') {
        header('Content-Type: application/json');
        echo json_encode(array('ok' => false, 'error' => 'Target vhost tidak boleh kosong.'));
        exit;
    }
    $cmd = $exe . ' vhost delete ' . escapeshellarg($target) . ' 2>&1';
    $output = @shell_exec($cmd);
    header('Content-Type: application/json');
    echo json_encode(array(
        'ok' => true,
        'cmd' => $cmd,
        'output' => $output ? $output : '',
    ));
    exit;
}

// Handle open_project_folder action (open folder in Windows File Explorer)
if ($action === 'open_project_folder') {
    $path = isset($_POST['path']) ? trim($_POST['path']) : '';
    if ($path !== '' && is_dir($path) && $isWindows) {
        pclose(popen('explorer ' . escapeshellarg($path), 'r'));
    }
    header('Content-Type: application/json');
    echo json_encode(array('ok' => true));
    exit;
}

// Handle open_project_terminal action (open terminal with Servisan environment at folder)
if ($action === 'open_project_terminal') {
    $path = isset($_POST['path']) ? trim($_POST['path']) : '';
    if ($path !== '' && is_dir($path) && $isWindows) {
        $exe = get_servisan_cmd($docRoot);
        $title = 'Servisan Shell - ' . basename($path);
        $cmd = 'start "' . $title . '" cmd.exe /k "cd /d "' . $path . '" && ' . $exe . ' shell"';
        pclose(popen($cmd, 'r'));
    }
    header('Content-Type: application/json');
    echo json_encode(array('ok' => true));
    exit;
}
