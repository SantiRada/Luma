<?php
require_once __DIR__ . '/includes/product_versions.php';

if (!db_available()) {
    $fallback = __DIR__ . '/versions/LUMA_0.1.0.exe';
    if (!is_file($fallback)) {
        require_database();
    }

    header('Content-Type: application/vnd.microsoft.portable-executable');
    header('Content-Disposition: attachment; filename="' . basename($fallback) . '"');
    header('Content-Length: ' . filesize($fallback));
    readfile($fallback);
    exit;
}

$id = (int) ($_GET['id'] ?? 0);
$version = $id > 0 ? product_version_by_id($id) : product_version_latest();

if (!$version) {
    http_response_code(404);
    exit('Versión no encontrada.');
}

$path = __DIR__ . '/' . $version['file_path'];
if (!is_file($path)) {
    http_response_code(404);
    exit('Archivo no encontrado.');
}

$stmt = $pdo->prepare('UPDATE product_versions SET downloads = downloads + 1 WHERE id = ?');
$stmt->execute([$version['id']]);

header('Content-Type: application/vnd.microsoft.portable-executable');
header('Content-Disposition: attachment; filename="' . basename($path) . '"');
header('Content-Length: ' . filesize($path));
readfile($path);
exit;
