<?php
require_once __DIR__ . '/includes/layout.php';
$user = require_login();
$stmt = $pdo->prepare('SELECT * FROM actions WHERE user_id = ? ORDER BY created_at DESC');
$stmt->execute([$user['id']]);
$actions = $stmt->fetchAll();

render_header('Mis Actions - LUMA', 'my-actions');
?>
<main class="pt-navbar">
    <section class="actions-section">
        <div class="container">
            <div class="section-header"><h2>Mis Actions publicadas</h2><p>Actions que subiste a la web de LUMA. No incluye las que solo descargaste.</p></div>
            <p><a class="btn btn-primary" href="upload.php">Publicar nueva Action</a></p>
            <div class="actions-grid">
                <?php if (!$actions): ?><p class="empty-state">Todavia no publicaste Actions.</p><?php endif; ?>
                <?php foreach ($actions as $action): ?>
                    <article class="action-card">
                        <div class="action-header"><div class="action-icon">LM</div><span class="action-badge"><?= e($action['category']) ?></span></div>
                        <h3><?= e($action['name']) ?></h3>
                        <p><?= e($action['short_description']) ?></p>
                        <div class="action-footer"><span>v<?= e($action['version']) ?> · <?= (int) $action['downloads'] ?> descargas</span><a class="action-download" href="action.php?slug=<?= e($action['slug']) ?>">Ver</a></div>
                    </article>
                <?php endforeach; ?>
            </div>
        </div>
    </section>
</main>
<?php render_footer(); ?>

