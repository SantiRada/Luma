<?php
require_once __DIR__ . '/includes/layout.php';

$error = '';
if (!db_available()) {
    $error = 'La base de datos no está disponible. Importa web/lumadb.sql y revisa la conexión.';
} elseif ($_SERVER['REQUEST_METHOD'] === 'POST') {
    verify_csrf();
    $name = trim($_POST['name'] ?? '');
    $email = strtolower(trim($_POST['email'] ?? ''));
    $password = $_POST['password'] ?? '';

    if ($name === '' || !filter_var($email, FILTER_VALIDATE_EMAIL) || strlen($password) < 8) {
        $error = 'Completa nombre, email válido y una contraseña de al menos 8 caracteres.';
    } else {
        try {
            $stmt = $pdo->prepare('INSERT INTO users (name, email, password_hash) VALUES (?, ?, ?)');
            $stmt->execute([$name, $email, password_hash($password, PASSWORD_DEFAULT)]);
            $_SESSION['user_id'] = (int) $pdo->lastInsertId();
            header('Location: account.php');
            exit;
        } catch (PDOException $e) {
            $error = 'Ese email ya está registrado.';
        }
    }
}

render_header('Crear cuenta - LUMA');
?>
<main class="auth-page">
    <form class="auth-card" method="post">
        <h1>Crear cuenta</h1>
        <p>Publica Actions, guarda favoritas y administra tu perfil.</p>
        <?php if ($error): ?><div class="status-msg error"><?= e($error) ?></div><?php endif; ?>
        <input type="hidden" name="csrf_token" value="<?= e(csrf_token()) ?>">
        <label>Nombre<input class="lm-input" name="name" required></label>
        <label>Email<input class="lm-input" type="email" name="email" required></label>
        <label>Contraseña<input class="lm-input" type="password" name="password" minlength="8" required></label>
        <button class="btn btn-primary w-100" type="submit">Registrarme</button>
        <div class="form-links">
            <span>¿Ya tenés cuenta?</span>
            <a class="auth-link" href="login.php">Entrar</a>
        </div>
    </form>
</main>
<?php render_footer(); ?>
