<?php
require_once __DIR__ . '/includes/actions.php';
require_database();

header('Content-Type: application/json; charset=utf-8');

try {
    $user = require_login();
    verify_csrf();
    $action_id = store_published_action($user, $_FILES['lm_file'] ?? [], $_POST);
    $stmt = $pdo->prepare('SELECT slug FROM actions WHERE id = ?');
    $stmt->execute([$action_id]);
    echo json_encode(['ok' => true, 'slug' => $stmt->fetchColumn()]);
} catch (Throwable $e) {
    http_response_code(400);
    echo json_encode(['ok' => false, 'error' => $e->getMessage()]);
}
