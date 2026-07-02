<?php
require_once __DIR__ . '/auth.php';

function text_length(string $value): int
{
    return function_exists('mb_strlen') ? mb_strlen($value, 'UTF-8') : strlen($value);
}

function read_lm_manifest(string $tmp_path): array
{
    if (!class_exists('ZipArchive')) {
        throw new RuntimeException('El servidor no tiene habilitada la extension ZIP de PHP.');
    }

    $zip = new ZipArchive();
    if ($zip->open($tmp_path) !== true) {
        throw new RuntimeException('El archivo .lm no se pudo leer como paquete ZIP.');
    }

    $manifest_raw = $zip->getFromName('manifest.json');
    $zip->close();

    if ($manifest_raw === false) {
        throw new RuntimeException('El paquete no contiene manifest.json en la raiz.');
    }

    $manifest = json_decode($manifest_raw, true);
    if (!is_array($manifest)) {
        throw new RuntimeException('manifest.json no es JSON valido.');
    }

    foreach (['id', 'name', 'version', 'runtime'] as $field) {
        if (empty($manifest[$field])) {
            throw new RuntimeException("manifest.json debe incluir {$field}.");
        }
    }

    return $manifest;
}

function store_published_action(array $user, array $file, array $input): int
{
    global $pdo;

    if (($file['error'] ?? UPLOAD_ERR_NO_FILE) !== UPLOAD_ERR_OK) {
        throw new RuntimeException('No se pudo subir el archivo .lm.');
    }

    if (($file['size'] ?? 0) > 20 * 1024 * 1024) {
        throw new RuntimeException('El archivo supera el limite de 20MB.');
    }

    $manifest = read_lm_manifest($file['tmp_name']);
    $name = trim($input['name'] ?? $manifest['name']);
    $version = trim($input['version'] ?? $manifest['version']);
    $short = trim($input['short_description'] ?? ($manifest['description'] ?? 'Action para LUMA.'));
    $description = trim($input['description'] ?? $short);
    $category = $input['category'] ?? 'other';
    $tags = trim($input['tags'] ?? implode(',', $manifest['tags'] ?? []));

    if ($name === '' || $version === '' || $short === '') {
        throw new RuntimeException('Completa nombre, versión y descripción corta.');
    }

    if (text_length($name) > 160 || text_length($version) > 40 || text_length($short) > 280 || text_length($description) > 5000 || text_length($tags) > 255) {
        throw new RuntimeException('Uno de los campos supera el límite de caracteres permitido.');
    }

    $allowed_categories = ['utility', 'design', 'dev', 'productivity', 'other'];
    if (!in_array($category, $allowed_categories, true)) {
        $category = 'other';
    }

    $base_slug = slugify($name);
    $slug = $base_slug;
    $i = 2;
    while (true) {
        $stmt = $pdo->prepare('SELECT id FROM actions WHERE slug = ?');
        $stmt->execute([$slug]);
        if (!$stmt->fetchColumn()) {
            break;
        }
        $slug = $base_slug . '-' . $i++;
    }

    $folder = __DIR__ . '/../actions/' . $base_slug;
    if (!is_dir($folder) && !mkdir($folder, 0775, true)) {
        throw new RuntimeException('No se pudo crear la carpeta de la Action.');
    }

    $safe_version = slugify($version);
    $filename = $base_slug . '_' . $safe_version . '.lm';
    $destination = $folder . '/' . $filename;
    if (!move_uploaded_file($file['tmp_name'], $destination)) {
        throw new RuntimeException('No se pudo guardar el archivo .lm.');
    }

    $relative_path = 'actions/' . $base_slug . '/' . $filename;

    $stmt = $pdo->prepare(
        'INSERT INTO actions (user_id, manifest_id, slug, name, version, short_description, description, category, tags, file_path, file_size)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)'
    );
    $stmt->execute([
        $user['id'],
        $manifest['id'],
        $slug,
        $name,
        $version,
        $short,
        $description,
        $category,
        $tags,
        $relative_path,
        filesize($destination),
    ]);

    return (int) $pdo->lastInsertId();
}
