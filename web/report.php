<?php
require_once __DIR__ . '/includes/layout.php';
require_database();
$user = current_user();
$done = false;

if ($_SERVER['REQUEST_METHOD'] === 'POST') {
    verify_csrf();
    $title = trim($_POST['title'] ?? '');
    $body = trim($_POST['body'] ?? '');
    if ($title !== '' && $body !== '') {
        $stmt = $pdo->prepare('INSERT INTO reports (user_id, title, body) VALUES (?, ?, ?)');
        $stmt->execute([$user['id'] ?? null, $title, $body]);
        $done = true;
    }
}

render_header('Reportar problema - LUMA');
?>
<main class="auth-page">
    <form class="auth-card" method="post">
        <h1>Reportar problema</h1>
        <p>Contanos que paso y como reproducirlo.</p>
        <?php if ($done): ?><div class="status-msg success">Gracias. Recibimos tu reporte.</div><?php endif; ?>
        <input type="hidden" name="csrf_token" value="<?= e(csrf_token()) ?>">
        <label>Titulo<input class="lm-input" name="title" required></label>
        <label>Detalle<textarea class="lm-input" name="body" rows="7" required></textarea></label>
        <button class="btn btn-primary w-100" type="submit">Enviar</button>
    </form>
</main>
<?php render_footer(); ?>
