<?php
require_once __DIR__ . '/includes/layout.php';
require_database();

$slug = $_GET['slug'] ?? '';
$stmt = $pdo->prepare('SELECT a.*, u.name AS author_name FROM actions a JOIN users u ON u.id = a.user_id WHERE a.slug = ? AND a.is_public = 1');
$stmt->execute([$slug]);
$action = $stmt->fetch();
if (!$action) {
    http_response_code(404);
    exit('Action no encontrada.');
}

render_header($action['name'] . ' - LUMA', 'actions');
?>
<main class="dashboard-page">
    <div class="container dashboard-grid">
        <section class="dashboard-panel">
            <span class="action-badge"><?= e($action['category']) ?></span>
            <h1><?= e($action['name']) ?></h1>
            <p class="lead"><?= e($action['short_description']) ?></p>
            <p><?= nl2br(e($action['description'])) ?></p>
            <p><strong>Version:</strong> <?= e($action['version']) ?> · <strong>Autor:</strong> <?= e($action['author_name']) ?> · <strong>Descargas:</strong> <?= (int) $action['downloads'] ?></p>
            <div class="hero-actions">
                <a class="btn btn-primary" href="download.php?id=<?= (int) $action['id'] ?>">Descargar .lm</a>
                <?php if (current_user()): ?>
                    <form method="post" action="save-action.php">
                        <input type="hidden" name="csrf_token" value="<?= e(csrf_token()) ?>">
                        <input type="hidden" name="action_id" value="<?= (int) $action['id'] ?>">
                        <button class="btn btn-secondary" type="submit"><?= is_saved_action((int) $action['id']) ? 'Quitar guardado' : 'Guardar' ?></button>
                    </form>
                <?php endif; ?>
            </div>
        </section>
    </div>
</main>
<?php render_footer(); ?>
