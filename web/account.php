<?php
require_once __DIR__ . '/includes/layout.php';
$user = require_login();

$message = '';
$error = '';
$is_editing = ($_GET['edit'] ?? '') === '1';

if ($_SERVER['REQUEST_METHOD'] === 'POST') {
    verify_csrf();
    $name = trim($_POST['name'] ?? '');
    $email = trim($_POST['email'] ?? '');
    $password = (string) ($_POST['password'] ?? '');
    $password_confirm = (string) ($_POST['password_confirm'] ?? '');

    if ($name === '' || !filter_var($email, FILTER_VALIDATE_EMAIL)) {
        $error = 'Completa un nombre y email válidos.';
        $is_editing = true;
    } elseif ($password !== '' && strlen($password) < 8) {
        $error = 'La contraseña debe tener al menos 8 caracteres.';
        $is_editing = true;
    } elseif ($password !== $password_confirm) {
        $error = 'Las contraseñas no coinciden.';
        $is_editing = true;
    } else {
        try {
            if ($password !== '') {
                $stmt = $pdo->prepare('UPDATE users SET name = ?, email = ?, password_hash = ? WHERE id = ?');
                $stmt->execute([$name, $email, password_hash($password, PASSWORD_DEFAULT), $user['id']]);
            } else {
                $stmt = $pdo->prepare('UPDATE users SET name = ?, email = ? WHERE id = ?');
                $stmt->execute([$name, $email, $user['id']]);
            }
            $_SESSION['flash'] = 'Perfil actualizado.';
            header('Location: account.php');
            exit;
        } catch (PDOException $e) {
            $error = 'Ese email ya está en uso.';
            $is_editing = true;
        }
    }
}

$message = flash_message() ?? '';
$stmt = $pdo->prepare('SELECT id, name, email, created_at FROM users WHERE id = ?');
$stmt->execute([$user['id']]);
$user = $stmt->fetch() ?: $user;

$actions_count = $pdo->prepare('SELECT COUNT(*) FROM actions WHERE user_id = ?');
$saved_count = $pdo->prepare('SELECT COUNT(*) FROM saved_actions WHERE user_id = ?');
$saved_actions_stmt = $pdo->prepare('SELECT a.*, u.name AS author_name FROM saved_actions s JOIN actions a ON a.id = s.action_id JOIN users u ON u.id = a.user_id WHERE s.user_id = ? ORDER BY s.created_at DESC');
$actions_count->execute([$user['id']]);
$saved_count->execute([$user['id']]);
$saved_actions_stmt->execute([$user['id']]);
$saved_actions = $saved_actions_stmt->fetchAll();

render_header('Mi cuenta - LUMA', 'account');
?>
<main class="dashboard-page">
    <div class="container account-layout">
        <?php if ($message): ?><div class="status-msg success"><?= e($message) ?></div><?php endif; ?>
        <?php if ($error): ?><div class="status-msg error"><?= e($error) ?></div><?php endif; ?>

        <?php if ($is_editing): ?>
            <section class="dashboard-panel account-edit-panel">
                <h1>Editar perfil</h1>
                <form method="post" class="stack-form">
                    <input type="hidden" name="csrf_token" value="<?= e(csrf_token()) ?>">
                    <label>Nombre<input class="lm-input" name="name" value="<?= e($user['name']) ?>" maxlength="120" required></label>
                    <label>Email<input class="lm-input" type="email" name="email" value="<?= e($user['email']) ?>" maxlength="190" required></label>
                    <label>Nueva contraseña<input class="lm-input" type="password" name="password" minlength="8" autocomplete="new-password"></label>
                    <label>Repetir contraseña<input class="lm-input" type="password" name="password_confirm" minlength="8" autocomplete="new-password"></label>
                    <div class="form-links">
                        <button class="btn btn-primary" type="submit">Guardar cambios</button>
                        <a class="btn btn-secondary" href="account.php">Cancelar</a>
                    </div>
                </form>
            </section>
        <?php else: ?>
            <section class="account-summary-card">
                <div class="account-user">
                    <div class="account-avatar" aria-hidden="true"><i class="ms-Icon ms-Icon--Contact"></i></div>
                    <div>
                        <h1><?= e($user['name']) ?></h1>
                        <p><?= e($user['email']) ?></p>
                    </div>
                </div>
                <div class="account-stats">
                    <div><strong><?= (int) $saved_count->fetchColumn() ?></strong><span>Guardados</span></div>
                    <div><strong><?= (int) $actions_count->fetchColumn() ?></strong><span>Subidos</span></div>
                </div>
                <div class="account-actions">
                    <a class="btn btn-secondary" href="account.php?edit=1">Editar perfil</a>
                    <a class="btn btn-primary" href="upload.php">Subir Action</a>
                </div>
            </section>

            <section class="saved-inline-section" id="saved">
                <div class="section-title-row">
                    <h2>Mis guardados</h2>
                </div>
                <div class="actions-grid">
                    <?php if (!$saved_actions): ?><p class="empty-state">No guardaste Actions todavía.</p><?php endif; ?>
                    <?php foreach ($saved_actions as $action): ?>
                        <article class="action-card">
                            <div class="action-header">
                                <div class="action-icon">LM</div>
                                <span class="action-badge"><?= e($action['category']) ?></span>
                            </div>
                            <h3><?= e($action['name']) ?></h3>
                            <p><?= e($action['short_description']) ?></p>
                            <div class="action-footer">
                                <span class="action-author"><?= e($action['author_name']) ?></span>
                                <a class="action-download" href="action.php?slug=<?= e($action['slug']) ?>">Ver</a>
                            </div>
                        </article>
                    <?php endforeach; ?>
                </div>
            </section>
        <?php endif; ?>
    </div>
</main>
<?php render_footer(); ?>
