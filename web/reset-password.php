<?php
require_once __DIR__ . '/includes/layout.php';
require_database();

$token = $_GET['token'] ?? ($_POST['token'] ?? '');
$error = '';
$done = false;

if ($_SERVER['REQUEST_METHOD'] === 'POST') {
    verify_csrf();
    $password = $_POST['password'] ?? '';
    $stmt = $pdo->prepare('SELECT * FROM password_resets WHERE token_hash = ? AND used_at IS NULL AND expires_at > NOW()');
    $stmt->execute([hash('sha256', $token)]);
    $reset = $stmt->fetch();

    if (!$reset || strlen($password) < 8) {
        $error = 'Enlace inválido o contraseña demasiado corta.';
    } else {
        $pdo->prepare('UPDATE users SET password_hash = ? WHERE id = ?')->execute([password_hash($password, PASSWORD_DEFAULT), $reset['user_id']]);
        $pdo->prepare('UPDATE password_resets SET used_at = NOW() WHERE id = ?')->execute([$reset['id']]);
        $done = true;
    }
}

render_header('Nueva contraseña - LUMA');
?>
<main class="auth-page">
    <form class="auth-card" method="post">
        <h1>Nueva contraseña</h1>
        <?php if ($done): ?><div class="status-msg success">Contraseña actualizada. <a class="auth-link" href="login.php">Entrar</a></div><?php endif; ?>
        <?php if ($error): ?><div class="status-msg error"><?= e($error) ?></div><?php endif; ?>
        <input type="hidden" name="csrf_token" value="<?= e(csrf_token()) ?>">
        <input type="hidden" name="token" value="<?= e($token) ?>">
        <label>Contraseña nueva<input class="lm-input" type="password" name="password" minlength="8" required></label>
        <button class="btn btn-primary w-100" type="submit">Guardar</button>
    </form>
</main>
<?php render_footer(); ?>
