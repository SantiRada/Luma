<?php
require_once __DIR__ . '/includes/layout.php';
require_login();

header('Location: account.php#saved');
exit;
