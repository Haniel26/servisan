<?php

// Regression test: dashboard CLI calls must reach the child process as
// separate argv entries, so database names and folders are never
// reinterpreted by a shell.

declare(strict_types=1);

putenv('SERVISAN_DASHBOARD_TOKEN=test-dashboard-token');
$_SERVER['REMOTE_ADDR'] = '127.0.0.1';

ob_start();
require __DIR__ . '/../www/index.php';
ob_end_clean();

$failures = [];

$code = 'foreach (array_slice($argv, 1) as $arg) { echo "<{$arg}>\n"; }';
$args = ['my db; rm -rf /', '--overwrite', 'quote"inside', 'back\\slash'];
$result = run_process_array(array_merge([PHP_BINARY, '-r', $code, '--'], $args));

if ($result['exit_code'] !== 0) {
    $failures[] = 'runner exited with code ' . $result['exit_code'] . ': ' . $result['output'];
}

$lines = array_values(array_filter(
    explode("\n", trim($result['stdout'])),
    static fn (string $line): bool => $line !== ''
));

if (count($lines) !== count($args)) {
    $failures[] = 'arguments were split, got ' . count($lines) . ' line(s): ' . json_encode($lines);
}

foreach ($args as $index => $arg) {
    $expected = '<' . $arg . '>';
    if (($lines[$index] ?? null) !== $expected) {
        $failures[] = 'argument ' . $index . ' arrived as ' . json_encode($lines[$index] ?? null)
            . ' instead of ' . json_encode($expected);
    }
}

if ($failures !== []) {
    fwrite(STDERR, "Dashboard CLI test failed:\n- " . implode("\n- ", $failures) . "\n");
    exit(1);
}

echo "Dashboard CLI test passed.\n";
