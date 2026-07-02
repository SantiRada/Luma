<?php
require_once __DIR__ . '/auth.php';

function product_versions_all(): array
{
    global $pdo;
    $stmt = $pdo->query('SELECT * FROM product_versions ORDER BY is_current DESC, published_at DESC, id DESC');
    return $stmt->fetchAll();
}

function product_version_latest(): ?array
{
    global $pdo;
    $stmt = $pdo->query('SELECT * FROM product_versions ORDER BY is_current DESC, published_at DESC, id DESC LIMIT 1');
    $version = $stmt->fetch();
    return $version ?: null;
}

function product_version_by_id(int $id): ?array
{
    global $pdo;
    $stmt = $pdo->prepare('SELECT * FROM product_versions WHERE id = ?');
    $stmt->execute([$id]);
    $version = $stmt->fetch();
    return $version ?: null;
}

function product_version_safe_filename(string $version): string
{
    $safe = preg_replace('/[^0-9A-Za-z._-]+/', '-', trim($version)) ?? '';
    $safe = trim($safe, '.-_');
    return 'LUMA_' . ($safe !== '' ? $safe : 'version') . '.exe';
}

function store_product_version(array $user, array $file, array $input): void
{
    global $pdo;

    $version = trim((string) ($input['version'] ?? ''));
    $changelog = trim((string) ($input['changelog'] ?? ''));
    $is_current = !empty($input['is_current']) ? 1 : 0;

    if ($version === '' || !preg_match('/^[0-9]+\\.[0-9]+\\.[0-9]+(?:[-+][0-9A-Za-z.-]+)?$/', $version)) {
        throw new RuntimeException('Usá una versión semver válida, por ejemplo 0.1.0.');
    }

    if ($changelog === '') {
        throw new RuntimeException('Agregá un changelog para esta versión.');
    }

    if (($file['error'] ?? UPLOAD_ERR_NO_FILE) !== UPLOAD_ERR_OK) {
        throw new RuntimeException('No se pudo subir el instalador.');
    }

    $extension = strtolower(pathinfo((string) ($file['name'] ?? ''), PATHINFO_EXTENSION));
    if ($extension !== 'exe') {
        throw new RuntimeException('El archivo debe ser un instalador .exe.');
    }

    $folder = __DIR__ . '/../versions';
    if (!is_dir($folder) && !mkdir($folder, 0775, true)) {
        throw new RuntimeException('No se pudo crear la carpeta de versiones.');
    }

    $filename = product_version_safe_filename($version);
    $destination = $folder . '/' . $filename;
    if (!move_uploaded_file($file['tmp_name'], $destination)) {
        throw new RuntimeException('No se pudo guardar el instalador.');
    }

    if ($is_current) {
        $pdo->exec('UPDATE product_versions SET is_current = 0');
    }

    $stmt = $pdo->prepare(
        'INSERT INTO product_versions (version, changelog, file_path, file_size, is_current, created_by)
         VALUES (?, ?, ?, ?, ?, ?)
         ON DUPLICATE KEY UPDATE changelog = VALUES(changelog), file_path = VALUES(file_path), file_size = VALUES(file_size), is_current = VALUES(is_current), created_by = VALUES(created_by), published_at = CURRENT_TIMESTAMP'
    );
    $stmt->execute([
        $version,
        $changelog,
        'versions/' . $filename,
        filesize($destination),
        $is_current,
        $user['id'],
    ]);
}
