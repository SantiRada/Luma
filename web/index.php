<?php
require_once __DIR__ . '/includes/layout.php';
render_header('LUMA - Launcher rapido para Actions', 'home');
?>
<main>
    <section class="hero" id="home">
        <div class="container hero-content">
            <div class="hero-badge">v0.1.0 - Acceso anticipado</div>
            <h1 class="hero-title">Herramientas rapidas, <span class="gradient-text">sin distracciones</span></h1>
            <p class="hero-subtitle">LUMA es un launcher de escritorio minimalista. Abre mini herramientas llamadas Actions, ejecuta una tarea concreta y vuelve a tu flujo en segundos.</p>
            <div class="hero-actions">
                <a href="#download" class="btn btn-primary">Descargar gratis</a>
                <a href="actions.php" class="btn btn-secondary">Explorar Actions</a>
            </div>
        </div>
    </section>

    <section class="features">
        <div class="container">
            <div class="section-header">
                <h2>Disenado para foco y velocidad</h2>
                <p>Actions pequenas, instalables y faciles de compartir. Nada de paneles enormes para tareas simples.</p>
            </div>
            <div class="features-grid">
                <div class="feature-card">
                    <div class="feature-icon"><i class="ms-Icon ms-Icon--LightningBolt"></i></div>
                    <h3>Rapido</h3>
                    <p>Atajo, busqueda, accion. LUMA prioriza apertura inmediata y poco consumo.</p>
                </div>
                <div class="feature-card">
                    <div class="feature-icon"><i class="ms-Icon ms-Icon--Puzzle"></i></div>
                    <h3>Extensible</h3>
                    <p>Crea Actions con HTML, CSS y JavaScript, empaquetalas como <code>.lm</code> y publicalas.</p>
                </div>
                <div class="feature-card">
                    <div class="feature-icon"><i class="ms-Icon ms-Icon--Lock"></i></div>
                    <h3>Local primero</h3>
                    <p>Las mejores Actions resuelven tareas sin enviar datos sensibles fuera de tu equipo.</p>
                </div>
                <div class="feature-card">
                    <div class="feature-icon"><i class="ms-Icon ms-Icon--CloudUpload"></i></div>
                    <h3>Comunidad</h3>
                    <p>Publica tus Actions, guarda las que te interesan y mantenlas disponibles para otros usuarios.</p>
                </div>
            </div>
        </div>
    </section>

    <section class="download" id="download">
        <div class="container">
            <div class="download-card">
                <h2>Empieza con LUMA</h2>
                <p>Descarga la app, instala Actions o crea las tuyas con el compilador web.</p>
                <div class="download-options">
                    <a href="download-luma.php" class="btn btn-primary btn-large">Descargar para Windows<span class="btn-subtext">Instalador .exe</span></a>
                    <a href="compiler.php" class="btn btn-secondary btn-large">Crear Action<span class="btn-subtext">Compilador .lm</span></a>
                </div>
                <a class="download-history-link" href="versions.php">Descargar versiones anteriores</a>
            </div>
        </div>
    </section>
</main>
<?php render_footer(); ?>
