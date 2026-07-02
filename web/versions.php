<?php
require_once __DIR__ . '/includes/layout.php';
require_once __DIR__ . '/includes/product_versions.php';
require_database();

$versions = product_versions_all();
$user = current_user();

render_header('Versiones anteriores - LUMA', 'home');
?>
<main class="versions-page">
    <section class="container versions-hero">
        <a class="back-link" href="index.php#download">‹ Volver al inicio</a>
        <p class="eyebrow">Historial de versiones</p>
        <h1>Versiones anteriores</h1>
        <p class="lead">Descargá una versión específica de LUMA. Se recomienda usar siempre la versión más reciente.</p>
        <?php if (is_product_admin($user)): ?>
            <a class="btn btn-secondary btn-small" href="versions-admin.php">Cargar nueva versión</a>
        <?php endif; ?>
    </section>

    <section class="container versions-list">
        <?php if (!$versions): ?>
            <div class="empty-state">Todavía no hay versiones publicadas.</div>
        <?php endif; ?>

        <?php foreach ($versions as $index => $version): ?>
            <article class="version-row <?= (int) $version['is_current'] === 1 ? 'is-current' : '' ?>">
                <div class="version-number">
                    <strong>v<?= e($version['version']) ?></strong>
                    <span><?= (int) $version['is_current'] === 1 ? 'Actual' : 'Anterior' ?></span>
                </div>
                <div class="version-info">
                    <h2><?= (int) $version['is_current'] === 1 ? 'Última versión estable' : 'Versión anterior' ?></h2>
                    <div class="version-meta">
                        <?php if ((int) $version['is_current'] === 1): ?><span class="version-pill recommended">Recomendada</span><?php endif; ?>
                        <span class="version-pill">Windows</span>
                        <span class="version-pill muted"><?= e(date('d/m/Y', strtotime((string) $version['published_at']))) ?></span>
                    </div>
                    <details class="version-changelog">
                        <summary>Ver changelog</summary>
                        <p><?= nl2br(e($version['changelog'])) ?></p>
                    </details>
                </div>
                <div class="version-action">
                    <a class="btn btn-secondary" href="download-luma.php?id=<?= (int) $version['id'] ?>">Descargar</a>
                </div>
            </article>
        <?php endforeach; ?>
    </section>
</main>
<?php render_footer(); ?>
