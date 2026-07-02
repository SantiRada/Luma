<?php
require_once __DIR__ . '/includes/auth.php';
require_database();
$id = (int) ($_GET['id'] ?? 0);
$stmt = $pdo->prepare('SELECT * FROM actions WHERE id = ? AND is_public = 1');
$stmt->execute([$id]);
$action = $stmt->fetch();

if (!$action || !is_file(__DIR__ . '/' . $action['file_path'])) {
    http_response_code(404);
    exit('Archivo no encontrado.');
}

$pdo->prepare('UPDATE actions SET downloads = downloads + 1 WHERE id = ?')->execute([$id]);
$path = __DIR__ . '/' . $action['file_path'];
header('Content-Type: application/octet-stream');
header('Content-Disposition: attachment; filename="' . basename($path) . '"');
header('Content-Length: ' . filesize($path));
readfile($path);
exit;
