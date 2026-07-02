<?php
require_once __DIR__ . '/includes/layout.php';
require_once __DIR__ . '/includes/actions.php';

$user = require_login();
$error = '';

if ($_SERVER['REQUEST_METHOD'] === 'POST') {
    verify_csrf();
    try {
        $action_id = store_published_action($user, $_FILES['lm_file'] ?? [], $_POST);
        $_SESSION['flash'] = 'Action publicada correctamente.';
        $stmt = $pdo->prepare('SELECT slug FROM actions WHERE id = ?');
        $stmt->execute([$action_id]);
        header('Location: action.php?slug=' . urlencode((string) $stmt->fetchColumn()));
        exit;
    } catch (Throwable $e) {
        $error = $e->getMessage();
    }
}

render_header('Publicar Action - LUMA', 'account');
?>
<main class="dashboard-page">
    <div class="container publish-layout">
        <section class="dashboard-panel publish-panel">
            <h1>Publicar Action</h1>
            <p>Sube un archivo <code>.lm</code>. LUMA leerá el manifest, guardará el paquete y creará su ficha pública.</p>
            <?php if ($error): ?><div class="status-msg error"><?= e($error) ?></div><?php endif; ?>
            <form method="post" enctype="multipart/form-data" class="stack-form js-limited-form" data-upload-form>
                <input type="hidden" name="csrf_token" value="<?= e(csrf_token()) ?>">

                <label class="file-picker">
                    <span>Archivo .lm</span>
                    <input type="file" name="lm_file" accept=".lm,.zip" required>
                    <span class="file-picker-ui">
                        <span class="file-picker-name" data-empty="Seleccionar archivo">Seleccionar archivo</span>
                        <span class="btn btn-secondary">Buscar</span>
                    </span>
                </label>
                <div id="compiled-upload-status" class="status-msg" hidden></div>

                <label>
                    <span class="label-row"><span>Nombre público</span><span class="char-counter" data-for="upload-name">(0/160)</span></span>
                    <input id="upload-name" class="lm-input" name="name" placeholder="Extract Text" data-max="160" required>
                </label>
                <label>Versión<input class="lm-input" name="version" placeholder="0.1.0" maxlength="40" required></label>
                <label>
                    <span class="label-row"><span>Descripción corta</span><span class="char-counter" data-for="upload-short-description">(0/280)</span></span>
                    <input id="upload-short-description" class="lm-input" name="short_description" data-max="280" required>
                </label>
                <label>
                    <span class="label-row"><span>Descripción completa</span><span class="char-counter" data-for="upload-description">(0/5000)</span></span>
                    <textarea id="upload-description" class="lm-input" name="description" rows="6" data-max="5000"></textarea>
                </label>
                <label>Categoría
                    <select class="lm-input" name="category">
                        <option value="utility">Utilidad</option>
                        <option value="design">Diseño</option>
                        <option value="dev">Desarrollo</option>
                        <option value="productivity">Productividad</option>
                        <option value="other">Otro</option>
                    </select>
                </label>
                <label>Tags<input class="lm-input" name="tags" placeholder="ocr,texto,pantalla" maxlength="255"></label>
                <button class="btn btn-primary" type="submit">Publicar</button>
            </form>
        </section>
    </div>
</main>
<script src="forms.js"></script>
<?php render_footer(); ?>
