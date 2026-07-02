<?php
require_once __DIR__ . '/includes/auth.php';
$user = require_login();
verify_csrf();
$action_id = (int) ($_POST['action_id'] ?? 0);

$stmt = $pdo->prepare('SELECT 1 FROM saved_actions WHERE user_id = ? AND action_id = ?');
$stmt->execute([$user['id'], $action_id]);

if ($stmt->fetchColumn()) {
    $pdo->prepare('DELETE FROM saved_actions WHERE user_id = ? AND action_id = ?')->execute([$user['id'], $action_id]);
} else {
    $pdo->prepare('INSERT IGNORE INTO saved_actions (user_id, action_id) VALUES (?, ?)')->execute([$user['id'], $action_id]);
}

header('Location: ' . ($_SERVER['HTTP_REFERER'] ?? 'actions.php'));
exit;

