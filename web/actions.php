<?php
require_once __DIR__ . '/includes/layout.php';
require_database();

$query = trim($_GET['q'] ?? '');
$category = $_GET['category'] ?? 'all';
$params = [];
$where = ['a.is_public = 1'];

if ($query !== '') {
    $where[] = '(a.name LIKE ? OR a.short_description LIKE ? OR a.tags LIKE ?)';
    $like = '%' . $query . '%';
    array_push($params, $like, $like, $like);
}

if ($category !== 'all') {
    $where[] = 'a.category = ?';
    $params[] = $category;
}

$sql = 'SELECT a.*, u.name AS author_name FROM actions a JOIN users u ON u.id = a.user_id WHERE ' . implode(' AND ', $where) . ' ORDER BY a.created_at DESC';
$stmt = $pdo->prepare($sql);
$stmt->execute($params);
$actions = $stmt->fetchAll();

render_header('Actions - LUMA', 'actions');
?>
<main class="pt-navbar">
    <section class="actions-section">
        <div class="container">
            <div class="section-header">
                <h2>Descubre Actions</h2>
                <p>Mini herramientas publicadas por la comunidad para resolver tareas rapidas dentro de LUMA.</p>
            </div>
            <form class="actions-controls" method="get">
                <div class="search-bar"><input type="text" name="q" value="<?= e($query) ?>" placeholder="Buscar Actions..."></div>
                <div class="filters">
                    <?php foreach (['all' => 'Todos', 'utility' => 'Utilidades', 'design' => 'Diseño', 'dev' => 'Desarrollo', 'productivity' => 'Productividad', 'other' => 'Otros'] as $key => $label): ?>
                        <button class="filter-btn <?= $category === $key ? 'active' : '' ?>" name="category" value="<?= e($key) ?>"><?= e($label) ?></button>
                    <?php endforeach; ?>
                </div>
            </form>
            <div class="actions-grid">
                <?php if (!$actions): ?>
                    <p class="empty-state">Todavia no hay Actions publicadas.</p>
                <?php endif; ?>
                <?php foreach ($actions as $action): ?>
                    <article class="action-card">
                        <div class="action-header">
                            <div class="action-icon">LM</div>
                            <span class="action-badge"><?= e($action['category']) ?></span>
                        </div>
                        <h3><?= e($action['name']) ?></h3>
                        <p><?= e($action['short_description']) ?></p>
                        <div class="action-footer">
                            <span class="action-author"><?= e($action['author_name']) ?> · v<?= e($action['version']) ?></span>
                            <a class="action-download" href="action.php?slug=<?= e($action['slug']) ?>">Ver</a>
                        </div>
                    </article>
                <?php endforeach; ?>
            </div>
        </div>
    </section>
</main>
<?php render_footer(); ?>
