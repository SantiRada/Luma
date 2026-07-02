<?php
if (session_status() !== PHP_SESSION_ACTIVE) {
    session_start();
}

require_once __DIR__ . '/db.php';

function current_user(): ?array
{
    global $pdo;

    if (!db_available() || empty($_SESSION['user_id'])) {
        return null;
    }

    $stmt = $pdo->prepare('SELECT id, name, email, created_at FROM users WHERE id = ?');
    $stmt->execute([$_SESSION['user_id']]);
    $user = $stmt->fetch();

    return $user ?: null;
}

function require_login(): array
{
    require_database();

    $user = current_user();
    if (!$user) {
        header('Location: login.php');
        exit;
    }

    return $user;
}

function is_product_admin(?array $user = null): bool
{
    $user = $user ?? current_user();
    return $user && strtolower((string) $user['email']) === 'santynrada@gmail.com';
}

function require_product_admin(): array
{
    $user = require_login();
    if (!is_product_admin($user)) {
        http_response_code(403);
        exit('No tenés permisos para acceder a esta sección.');
    }

    return $user;
}

function csrf_token(): string
{
    if (empty($_SESSION['csrf_token'])) {
        $_SESSION['csrf_token'] = bin2hex(random_bytes(32));
    }

    return $_SESSION['csrf_token'];
}

function verify_csrf(): void
{
    $token = $_POST['csrf_token'] ?? '';
    if (!$token || !hash_equals($_SESSION['csrf_token'] ?? '', $token)) {
        http_response_code(403);
        exit('Token invalido.');
    }
}

function e(?string $value): string
{
    return htmlspecialchars((string) $value, ENT_QUOTES, 'UTF-8');
}

function slugify(string $value): string
{
    $value = strtolower(trim($value));
    $value = preg_replace('/[^a-z0-9]+/i', '-', $value) ?? '';
    $value = trim($value, '-');

    return $value !== '' ? $value : 'action';
}

function is_saved_action(int $action_id): bool
{
    global $pdo;
    $user = current_user();
    if (!$user) {
        return false;
    }

    $stmt = $pdo->prepare('SELECT 1 FROM saved_actions WHERE user_id = ? AND action_id = ?');
    $stmt->execute([$user['id'], $action_id]);

    return (bool) $stmt->fetchColumn();
}
