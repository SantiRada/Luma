<?php
$db_host = getenv('LUMA_DB_HOST') ?: '127.0.0.1';
$db_name = getenv('LUMA_DB_NAME') ?: 'lumadb';
$db_user = getenv('LUMA_DB_USER') ?: 'root';
$db_pass = getenv('LUMA_DB_PASS') ?: '';
$pdo = null;
$db_error = null;

if (!class_exists('PDO')) {
    $db_error = 'La extension PDO de PHP no esta disponible en el servidor.';
} else {
    try {
        $pdo = new PDO(
            "mysql:host={$db_host};dbname={$db_name};charset=utf8mb4",
            $db_user,
            $db_pass,
            [
                PDO::ATTR_ERRMODE => PDO::ERRMODE_EXCEPTION,
                PDO::ATTR_DEFAULT_FETCH_MODE => PDO::FETCH_ASSOC,
                PDO::ATTR_EMULATE_PREPARES => false,
            ]
        );
    } catch (Throwable $error) {
        $db_error = $error->getMessage();
    }
}

function db_available(): bool
{
    global $pdo;
    return $pdo instanceof PDO;
}

function require_database(): void
{
    if (db_available()) {
        return;
    }

    http_response_code(503);
    render_db_unavailable();
    exit;
}

function render_db_unavailable(): void
{
    global $db_name, $db_user, $db_error;
    ?>
<!DOCTYPE html>
<html lang="es">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>LUMA - Base de datos no disponible</title>
    <link rel="icon" href="images/icon.ico" sizes="any">
    <link rel="stylesheet" href="styles.css">
</head>
<body>
    <main class="auth-page">
        <section class="auth-card">
            <h1>Base de datos no disponible</h1>
            <p>LUMA Web ya esta cargando, pero falta conectar MySQL.</p>
            <div class="status-msg warning">
                Importa <code>web/lumadb.sql</code> y verifica que exista la base <code><?= htmlspecialchars($db_name, ENT_QUOTES, 'UTF-8') ?></code>
                con el usuario <code><?= htmlspecialchars($db_user, ENT_QUOTES, 'UTF-8') ?></code>.
            </div>
            <?php if ($db_error): ?>
                <p class="muted">Detalle: <?= htmlspecialchars($db_error, ENT_QUOTES, 'UTF-8') ?></p>
            <?php endif; ?>
        </section>
    </main>
</body>
</html>
    <?php
}
