<?php

if (!function_exists('get_servisan_home')) {
    function get_servisan_home() {
        $env = getenv('SERVISAN_HOME');
        if ($env && is_dir($env)) {
            return rtrim($env, '/\\');
        }
        $home = dirname(dirname(__DIR__));
        if (file_exists($home . DIRECTORY_SEPARATOR . 'servisan.exe') || file_exists($home . DIRECTORY_SEPARATOR . 'Cargo.toml') || is_dir($home . DIRECTORY_SEPARATOR . 'bin')) {
            return $home;
        }
        return dirname(__DIR__);
    }
}

if (!function_exists('get_servisan_cmd')) {
    function get_servisan_cmd($docRoot = null) {
        $rootDir = get_servisan_home();
        $candidates = array(
            $rootDir . DIRECTORY_SEPARATOR . 'servisan.exe',
            $rootDir . DIRECTORY_SEPARATOR . 'target' . DIRECTORY_SEPARATOR . 'release' . DIRECTORY_SEPARATOR . 'servisan.exe',
            $rootDir . DIRECTORY_SEPARATOR . 'target' . DIRECTORY_SEPARATOR . 'debug' . DIRECTORY_SEPARATOR . 'servisan.exe',
        );
        foreach ($candidates as $candidate) {
            if (file_exists($candidate)) {
                return '"' . $candidate . '"';
            }
        }
        return 'servisan';
    }
}

if (!function_exists('check_mysql')) {
    function check_mysql($host, $port, $user, $pass) {
        if (class_exists('mysqli')) {
            // PHP 8.1+ throws on connection failure instead of setting connect_errno.
            try {
                $conn = @new mysqli($host, $user, $pass, null, $port);
                if ($conn->connect_errno) {
                    return array(false, 'mysqli: ' . $conn->connect_error);
                }
                $version = (string) $conn->server_info;
                $conn->close();
                return array(true, $version);
            } catch (\Exception $e) {
                return array(false, 'mysqli: ' . $e->getMessage());
            }
        }

        if (class_exists('PDO') && in_array('mysql', PDO::getAvailableDrivers(), true)) {
            try {
                $pdo = new PDO("mysql:host={$host};port={$port}", $user, $pass);
                $version = (string) $pdo->query('SELECT VERSION()')->fetchColumn();
                return array(true, $version);
            } catch (PDOException $e) {
                return array(false, 'PDO: ' . $e->getMessage());
            }
        }

        return array(false, 'no mysqli or pdo_mysql extension loaded');
    }
}

if (!function_exists('tail_file')) {
    function tail_file($filepath, $lines = 50) {
        if (!file_exists($filepath)) {
            return "Log file not found: " . basename($filepath);
        }
        $file = @fopen($filepath, "r");
        if (!$file) {
            return "Cannot open log file: " . basename($filepath);
        }
        $buffer = array();
        while (($line = fgets($file)) !== false) {
            $buffer[] = $line;
            if (count($buffer) > $lines) {
                array_shift($buffer);
            }
        }
        fclose($file);
        if (empty($buffer)) {
            return "Log file is currently empty (" . basename($filepath) . ").";
        }
        return implode("", $buffer);
    }
}
