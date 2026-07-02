<?php
require_once __DIR__ . '/includes/layout.php';
$user = current_user();
render_header('Compilador de Actions - LUMA', 'compiler');
?>
<script src="https://cdnjs.cloudflare.com/ajax/libs/jszip/3.10.1/jszip.min.js"></script>
<main class="compiler-container" data-logged-in="<?= $user ? '1' : '0' ?>">
    <div class="compiler-header text-center">
        <h1>Compilador de Actions</h1>
        <p class="lead">Empaqueta una carpeta como <code>.lm</code>, inspecciona paquetes existentes y publica tu Action.</p>
    </div>
    <div class="compiler-grid">
        <div class="compiler-card">
            <div class="card-header"><h2>Compilar</h2><p>Convierte una carpeta con <code>manifest.json</code> en un archivo instalable.</p></div>
            <div class="drop-zone" id="pack-drop-zone">
                <div class="drop-icon"><i class="ms-Icon ms-Icon--FolderHorizontal"></i></div>
                <p>Arrastra la carpeta de tu Action aqui<br><span class="text-muted">o haz clic para seleccionar</span></p>
                <input type="file" id="folder-input" webkitdirectory directory multiple hidden>
            </div>
            <div class="file-list-container" id="pack-file-list" style="display: none;">
                <h4>Archivos detectados:</h4>
                <ul id="pack-files"></ul>
                <div id="pack-status" class="status-msg"></div>
                <button class="btn btn-primary w-100 mt-2" id="btn-pack" disabled>Generar .lm</button>
            </div>
        </div>
        <div class="compiler-card">
            <div class="card-header"><h2>Inspeccionar</h2><p>Lee un <code>.lm</code> y descarga su fuente como ZIP.</p></div>
            <div class="drop-zone" id="unpack-drop-zone">
                <div class="drop-icon"><i class="ms-Icon ms-Icon--Package"></i></div>
                <p>Arrastra tu archivo <code>.lm</code> aqui<br><span class="text-muted">o haz clic para seleccionar</span></p>
                <input type="file" id="lm-input" accept=".lm,.zip" hidden>
            </div>
            <div class="file-list-container" id="unpack-file-list" style="display: none;">
                <div class="manifest-info" id="unpack-manifest-info"></div>
                <h4>Contenido del paquete:</h4>
                <ul id="unpack-files"></ul>
                <div id="unpack-status" class="status-msg"></div>
                <button class="btn btn-secondary w-100 mt-2" id="btn-unpack" disabled>Descargar fuente (.zip)</button>
            </div>
        </div>
    </div>
</main>

<div id="compile-result-modal" class="modal">
    <div class="modal-content compile-choice-modal">
        <button class="close-modal" type="button">&times;</button>
        <h3>Action compilada</h3>
        <p>¿Quieres publicarla directamente o prefieres descargar el archivo <code>.lm</code>?</p>
        <div class="modal-actions">
            <button class="btn btn-primary" type="button" id="btn-publish-compiled">Publicar ahora</button>
            <button class="btn btn-secondary" type="button" id="btn-download-compiled">Descargar .lm</button>
        </div>
    </div>
</div>

<div id="error-modal" class="modal">
    <div class="modal-content">
        <button class="close-modal" type="button">&times;</button>
        <h3 class="error-title">Error</h3>
        <p id="error-message"></p>
    </div>
</div>
<script src="forms.js"></script>
<script src="compiler.js"></script>
<?php render_footer(); ?>
