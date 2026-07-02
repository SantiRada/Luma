<?php
require_once __DIR__ . '/includes/layout.php';
require_once __DIR__ . '/includes/product_versions.php';

$user = require_product_admin();
$error = '';

if ($_SERVER['REQUEST_METHOD'] === 'POST') {
    verify_csrf();
    try {
        store_product_version($user, $_FILES['installer'] ?? [], $_POST);
        $_SESSION['flash'] = 'Versión publicada correctamente.';
        header('Location: versions.php');
        exit;
    } catch (Throwable $e) {
        $error = $e->getMessage();
    }
}

render_header('Cargar versión - LUMA', 'account');
?>
<main class="dashboard-page">
    <div class="container publish-layout">
        <section class="dashboard-panel publish-panel">
            <h1>Cargar nueva versión</h1>
            <p>Esta sección solo está disponible para el administrador del producto. El archivo se guardará en <code>web/versions/LUMA_NUM_VERSION.exe</code>.</p>
            <?php if ($error): ?><div class="status-msg error"><?= e($error) ?></div><?php endif; ?>
            <form method="post" enctype="multipart/form-data" class="stack-form">
                <input type="hidden" name="csrf_token" value="<?= e(csrf_token()) ?>">
                <label>Versión<input class="lm-input" name="version" placeholder="0.1.1" required></label>
                <label>Changelog<textarea class="lm-input" name="changelog" rows="8" placeholder="- Mejora..." required></textarea></label>
                <label class="file-picker">
                    <span>Instalador .exe</span>
                    <input type="file" name="installer" accept=".exe" required>
                    <span class="file-picker-ui">
                        <span class="file-picker-name" data-empty="Seleccionar archivo">Seleccionar archivo</span>
                        <span class="btn btn-secondary">Buscar</span>
                    </span>
                </label>
                <label class="check-row"><input type="checkbox" name="is_current" value="1" checked> Marcar como versión actual</label>
                <button class="btn btn-primary" type="submit">Publicar versión</button>
            </form>
        </section>
    </div>
</main>
<script src="forms.js"></script>
<?php render_footer(); ?>
