<?php
require_once __DIR__ . '/includes/layout.php';
require_database();

$reset_link = '';
if ($_SERVER['REQUEST_METHOD'] === 'POST') {
    verify_csrf();
    $email = strtolower(trim($_POST['email'] ?? ''));
    $stmt = $pdo->prepare('SELECT id FROM users WHERE email = ?');
    $stmt->execute([$email]);
    $user = $stmt->fetch();

    if ($user) {
        $token = bin2hex(random_bytes(32));
        $stmt = $pdo->prepare('INSERT INTO password_resets (user_id, token_hash, expires_at) VALUES (?, ?, DATE_ADD(NOW(), INTERVAL 1 HOUR))');
        $stmt->execute([$user['id'], hash('sha256', $token)]);
        $reset_link = 'reset-password.php?token=' . $token;
    } else {
        $reset_link = 'Si el email existe, se generó un enlace de recuperación.';
    }
}

render_header('Recuperar contraseña - LUMA');
?>
<main class="auth-page">
    <form class="auth-card" method="post">
        <h1>Recuperar contraseña</h1>
        <p>En producción este enlace se enviaría por email. En local se muestra aquí para probar.</p>
        <?php if ($reset_link): ?><div class="status-msg success"><?= e($reset_link) ?></div><?php endif; ?>
        <input type="hidden" name="csrf_token" value="<?= e(csrf_token()) ?>">
        <label>Email<input class="lm-input" type="email" name="email" required></label>
        <button class="btn btn-primary w-100" type="submit">Generar enlace</button>
        <div class="form-links">
            <a class="auth-link" href="login.php">Volver a entrar</a>
        </div>
    </form>
</main>
<?php render_footer(); ?>
