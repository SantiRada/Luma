<?php
require_once __DIR__ . '/auth.php';

function render_header(string $title, string $active = ''): void
{
    if (!headers_sent()) {
        header('Content-Type: text/html; charset=UTF-8');
    }

    $user = current_user();
    ?>
<!DOCTYPE html>
<html lang="es">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title><?= e($title) ?></title>
    <meta name="description" content="LUMA es un launcher de escritorio rapido, minimalista y extensible con Actions.">
    <link rel="icon" href="images/icon.ico" sizes="any">
    <link rel="icon" type="image/png" href="images/icon.png">
    <link rel="apple-touch-icon" href="images/icon.png">
    <link rel="preconnect" href="https://fonts.googleapis.com">
    <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
    <link href="https://fonts.googleapis.com/css2?family=Inter:wght@300;400;500;600;700;800&family=Outfit:wght@400;600;800&family=Fira+Code:wght@400;500&display=swap" rel="stylesheet">
    <link rel="stylesheet" href="https://static2.sharepointonline.com/files/fabric/office-ui-fabric-core/11.0.0/css/fabric.min.css">
    <link rel="stylesheet" href="styles.css">
</head>
<body>
    <nav class="navbar">
        <div class="container nav-content">
            <a href="index.php" class="logo"><img src="images/icon.png" alt="" class="logo-icon">LUMA</a>
            <button class="nav-toggle" type="button" aria-label="Abrir menú" aria-controls="site-menu" aria-expanded="false">
                <span></span>
                <span></span>
                <span></span>
            </button>
            <div class="site-menu" id="site-menu">
            <div class="nav-links">
                <a href="index.php" class="<?= $active === 'home' ? 'active' : '' ?>">Inicio</a>
                <a href="actions.php" class="<?= $active === 'actions' ? 'active' : '' ?>">Actions</a>
                <a href="docs.php" class="<?= $active === 'docs' ? 'active' : '' ?>">Docs</a>
                <a href="compiler.php" class="<?= $active === 'compiler' ? 'active' : '' ?>">Compilador</a>
            </div>
            <div class="nav-actions">
                <?php if ($user): ?>
                    <a href="account.php" class="btn btn-outline nav-btn <?= $active === 'account' ? 'active' : '' ?>">Mi cuenta</a>
                <?php else: ?>
                    <a href="login.php" class="btn btn-outline nav-btn">Entrar</a>
                <?php endif; ?>
            </div>
            </div>
        </div>
    </nav>
    <?php
}

function render_footer(): void
{
    ?>
    <footer>
        <div class="container footer-content">
            <div class="footer-brand">
                <span class="logo"><img src="images/icon.png" alt="" class="logo-icon">LUMA</span>
                <p>El launcher de escritorio que respeta tu foco.</p>
            </div>
            <div class="footer-links">
                <a href="docs.php">Documentación</a>
                <a href="upload.php">Publicar Action</a>
                <a href="report.php">Reportar problema</a>
            </div>
        </div>
    </footer>
<script src="nav.js"></script>
</body>
</html>
    <?php
}

function flash_message(): ?string
{
    if (empty($_SESSION['flash'])) {
        return null;
    }

    $message = $_SESSION['flash'];
    unset($_SESSION['flash']);

    return $message;
}

