<?php
require_once __DIR__ . '/includes/layout.php';

$error = '';
$next = $_GET['next'] ?? $_POST['next'] ?? 'account.php';
if (!is_string($next) || preg_match('/^https?:\/\//i', $next) || strpos($next, '//') === 0) {
    $next = 'account.php';
}
if (!db_available()) {
    $error = 'La base de datos no está disponible. Importa web/lumadb.sql y revisa la conexión.';
} elseif ($_SERVER['REQUEST_METHOD'] === 'POST') {
    verify_csrf();
    $email = strtolower(trim($_POST['email'] ?? ''));
    $password = $_POST['password'] ?? '';
    $stmt = $pdo->prepare('SELECT * FROM users WHERE email = ?');
    $stmt->execute([$email]);
    $user = $stmt->fetch();

    if ($user && password_verify($password, $user['password_hash'])) {
        $_SESSION['user_id'] = (int) $user['id'];
        header('Location: ' . $next);
        exit;
    }

    $error = 'Email o contraseña incorrectos.';
}

render_header('Entrar - LUMA');
?>
<main class="auth-page">
    <form class="auth-card" method="post">
        <h1>Entrar</h1>
        <p>Accede para publicar y guardar Actions.</p>
        <?php if ($error): ?><div class="status-msg error"><?= e($error) ?></div><?php endif; ?>
        <input type="hidden" name="csrf_token" value="<?= e(csrf_token()) ?>">
        <input type="hidden" name="next" value="<?= e($next) ?>">
        <label>Email<input class="lm-input" type="email" name="email" required></label>
        <label>Contraseña<input class="lm-input" type="password" name="password" required></label>
        <a class="auth-link auth-link-muted" href="forgot-password.php">¿Olvidaste tu contraseña?</a>
        <button class="btn btn-primary w-100" type="submit">Entrar</button>
        <div class="form-links">
            <span>¿No tenés cuenta?</span>
            <a class="auth-link" href="register.php">Crear cuenta</a>
        </div>
    </form>
</main>
<?php render_footer(); ?>
