<?php
require_once __DIR__ . '/includes/layout.php';
render_header('Documentación LUMA Actions', 'docs');
?>
<main class="docs-simple">
    <aside class="docs-menu">
        <a href="#inicio">Inicio rápido</a>
        <a href="#concepto">Qué es una Action</a>
        <a href="#estructura">Estructura</a>
        <a href="#manifest">Manifest</a>
        <a href="#runtime">Runtime</a>
        <a href="#permisos">Permisos</a>
        <a href="#components">Components</a>
        <a href="#api">API de LUMA</a>
        <a href="#ui">UI y experiencia</a>
        <a href="#validar">Validar y compilar</a>
        <a href="#publicar">Publicar</a>
        <a href="#instalar">Instalar y actualizar</a>
        <a href="#checklist">Checklist</a>
        <a href="#problemas">Problemas comunes</a>
    </aside>

    <article class="docs-article">
        <h1>Documentación para crear Actions</h1>
        <p class="lead">Una Action es una mini app instalable en LUMA. Debe resolver una tarea concreta, abrir rápido y pedir la menor atención posible.</p>

        <section id="inicio">
            <h2>Inicio rápido</h2>
            <p>La forma más simple de crear una Action es copiar la plantilla, editar el manifest, probar la carpeta y luego compilarla como <code>.lm</code>.</p>
            <ol>
                <li>Crea una carpeta para tu Action o copia <code>actions/template</code>.</li>
                <li>Edita <code>manifest.json</code> con un <code>id</code>, nombre, descripción, versión, runtime y permisos.</li>
                <li>Construye la UI dentro de <code>action/index.html</code>, <code>action/styles.css</code> y <code>action/action.js</code>.</li>
                <li>Valida el manifest antes de compilar.</li>
                <li>Compila la carpeta desde el Compilador web o con el comando local.</li>
                <li>Instala el <code>.lm</code> en LUMA con el botón <code>+</code>, o publícalo desde la web.</li>
            </ol>
        </section>

        <section id="concepto">
            <h2>Qué es una Action</h2>
            <p>Una Action no es una app grande ni un plugin general. Es una capacidad enfocada que LUMA puede descubrir, instalar y ejecutar desde el launcher.</p>
            <p>Ejemplos de buenas Actions: copiar un color, extraer texto de una zona de pantalla, formatear texto del portapapeles, generar un snippet, abrir una búsqueda rápida o transformar una imagen.</p>
            <p>La regla principal: si necesita muchas pantallas, configuración permanente o navegación compleja, probablemente no debería ser una Action.</p>
        </section>

        <section id="estructura">
            <h2>Estructura de archivos</h2>
            <p>Una Action fuente debe tener el manifest en la raíz y sus archivos ejecutables dentro de <code>action/</code>.</p>
            <pre><code>my-action/
  manifest.json
  README.md
  action/
    index.html
    styles.css
    action.js
    assets/
      icon.png</code></pre>
            <p>El archivo compilado <code>.lm</code> es un ZIP con esa misma estructura. LUMA espera encontrar <code>manifest.json</code> en la raíz del paquete.</p>
            <pre><code>my-action_0-1-0.lm
  manifest.json
  action/
    index.html
    styles.css
    action.js</code></pre>
            <p>No incluyas archivos que no usa la Action: fuentes pesadas, capturas, dependencias sin uso o carpetas de desarrollo. Un paquete liviano abre más rápido.</p>
        </section>

        <section id="manifest">
            <h2>Manifest</h2>
            <p>El manifest es el contrato entre tu Action y LUMA. Define identidad, versión, entrada, tamaño, permisos y componentes requeridos.</p>
            <pre><code>{
  "schemaVersion": "1.0.0",
  "id": "luma.action.example",
  "name": "Example Action",
  "description": "Does one focused thing.",
  "version": "0.1.0",
  "author": "Your Name",
  "runtime": {
    "type": "window",
    "entry": "action/index.html",
    "width": 520,
    "height": 360
  },
  "permissions": ["clipboard:write"],
  "components": [],
  "tags": ["utility"]
}</code></pre>

            <h3>Campos obligatorios</h3>
            <table class="docs-table">
                <thead><tr><th>Campo</th><th>Qué hace</th><th>Regla</th></tr></thead>
                <tbody>
                    <tr><td><code>schemaVersion</code></td><td>Versión del formato de manifest.</td><td>Debe ser <code>1.0.0</code>.</td></tr>
                    <tr><td><code>id</code></td><td>Identidad estable de la Action.</td><td>Debe empezar con <code>luma.action.</code> y usar minúsculas, números, puntos o guiones.</td></tr>
                    <tr><td><code>name</code></td><td>Nombre visible en LUMA.</td><td>1 a 48 caracteres.</td></tr>
                    <tr><td><code>description</code></td><td>Resumen corto de lo que hace.</td><td>1 a 140 caracteres.</td></tr>
                    <tr><td><code>version</code></td><td>Versión instalable.</td><td>Formato semver: <code>0.1.0</code>, <code>1.2.3</code>, <code>1.0.0-beta.1</code>.</td></tr>
                    <tr><td><code>runtime</code></td><td>Cómo se abre la Action.</td><td>Debe incluir <code>type</code> y <code>entry</code>.</td></tr>
                    <tr><td><code>permissions</code></td><td>Capacidades que solicita.</td><td>Array, aunque esté vacío.</td></tr>
                </tbody>
            </table>

            <h3>Campos opcionales</h3>
            <table class="docs-table">
                <thead><tr><th>Campo</th><th>Uso</th></tr></thead>
                <tbody>
                    <tr><td><code>author</code></td><td>Autor o equipo responsable.</td></tr>
                    <tr><td><code>components</code></td><td>Components compartidos que LUMA debe instalar o precargar.</td></tr>
                    <tr><td><code>tags</code></td><td>Palabras para clasificar y buscar la Action.</td></tr>
                    <tr><td><code>icon</code></td><td>Ruta a un icono dentro del paquete, por ejemplo <code>action/assets/icon.png</code>.</td></tr>
                    <tr><td><code>store</code></td><td>Metadatos extra para publicación, como categoría o keywords.</td></tr>
                </tbody>
            </table>

            <p>El <code>id</code> no debería cambiar nunca. Para publicar una actualización, conserva el mismo <code>id</code> y cambia <code>version</code>.</p>
        </section>

        <section id="runtime">
            <h2>Runtime</h2>
            <p>El runtime define cómo LUMA abre la Action.</p>
            <table class="docs-table">
                <thead><tr><th>Tipo</th><th>Cuándo usarlo</th><th>Comportamiento</th></tr></thead>
                <tbody>
                    <tr><td><code>window</code></td><td>Actions con UI compacta.</td><td>Abre una ventana centrada, siempre arriba, con tamaño fijo.</td></tr>
                    <tr><td><code>overlay</code></td><td>Actions que necesitan actuar sobre la pantalla.</td><td>Abre una capa transparente sobre el escritorio.</td></tr>
                    <tr><td><code>background</code></td><td>Actions sin UI visible.</td><td>Reservado. El schema lo acepta, pero LUMA todavía no lo abre como runtime productivo.</td></tr>
                </tbody>
            </table>
            <p>Para la mayoría de Actions nuevas, empieza con <code>window</code>. Usa <code>overlay</code> solo si el usuario debe seleccionar o inspeccionar algo en pantalla. No uses <code>background</code> para una Action publicada hasta que LUMA lo habilite completamente.</p>
            <pre><code>"runtime": {
  "type": "window",
  "entry": "action/index.html",
  "width": 520,
  "height": 360
}</code></pre>
            <p><code>entry</code> siempre debe apuntar a un archivo dentro de <code>action/</code>. El ancho mínimo es 280 y el máximo 1200. El alto mínimo es 180 y el máximo 900.</p>
        </section>

        <section id="permisos">
            <h2>Permisos</h2>
            <p>Declara solo lo que la Action necesita. Esto ayuda a que LUMA pueda explicar, limitar y preparar capacidades correctamente.</p>
            <table class="docs-table">
                <thead><tr><th>Permiso</th><th>Uso esperado</th></tr></thead>
                <tbody>
                    <tr><td><code>clipboard:read</code></td><td>Leer contenido del portapapeles.</td></tr>
                    <tr><td><code>clipboard:write</code></td><td>Copiar texto, imágenes o resultados al portapapeles.</td></tr>
                    <tr><td><code>screen:read</code></td><td>Capturar o leer una región de pantalla.</td></tr>
                    <tr><td><code>screen:overlay</code></td><td>Mostrar una capa sobre la pantalla.</td></tr>
                    <tr><td><code>microphone</code></td><td>Captura desde micrófono.</td></tr>
                    <tr><td><code>audio:system</code></td><td>Audio del sistema.</td></tr>
                    <tr><td><code>files:read</code></td><td>Leer archivos elegidos o permitidos.</td></tr>
                    <tr><td><code>files:write</code></td><td>Crear o modificar archivos.</td></tr>
                    <tr><td><code>network</code></td><td>Conectar con internet o servicios externos.</td></tr>
                </tbody>
            </table>
        </section>

        <section id="components">
            <h2>Components</h2>
            <p>Los Components son capacidades compartidas que LUMA puede instalar, preparar y reutilizar entre varias Actions. Sirven para tareas pesadas o de sistema, como OCR, captura, audio o acceso avanzado al portapapeles.</p>
            <p>Si una Action instalada requiere un Component, LUMA puede prepararlo al iniciar para que el uso posterior sea rápido. Si varias Actions usan el mismo Component, se carga una sola vez.</p>
            <table class="docs-table">
                <thead><tr><th>Component</th><th>Estado actual</th><th>Uso</th></tr></thead>
                <tbody>
                    <tr><td><code>luma.component.ocr</code></td><td>Disponible</td><td>OCR compartido para extraer texto desde imágenes o regiones de pantalla.</td></tr>
                </tbody>
            </table>
            <pre><code>"components": ["luma.component.ocr"],
"permissions": ["screen:read", "screen:overlay", "clipboard:write"]</code></pre>
            <p>No declares Components por costumbre. Si tu Action no usa OCR, no pidas <code>luma.component.ocr</code>.</p>
        </section>

        <section id="api">
            <h2>API de LUMA disponible para Actions</h2>
            <p>Las Actions corren dentro de una ventana Tauri, por lo que pueden usar <code>window.__TAURI__</code> cuando LUMA expone una capacidad.</p>
            <pre><code>const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;
const currentWindow = window.__TAURI__.window.getCurrentWindow();</code></pre>

            <h3>Comandos actuales</h3>
            <table class="docs-table">
                <thead><tr><th>Comando</th><th>Retorna</th><th>Uso</th></tr></thead>
                <tbody>
                    <tr><td><code>hide_current_window</code></td><td><code>void</code></td><td>Oculta la ventana actual cuando la Action termina o debe salir de pantalla.</td></tr>
                    <tr><td><code>get_virtual_screen_bounds</code></td><td><code>{ x, y, width, height }</code></td><td>Obtiene el rectángulo total de monitores para overlays.</td></tr>
                    <tr><td><code>extract_text_from_screen_region</code></td><td><code>void</code></td><td>Ejecuta OCR en una región y copia el texto detectado al portapapeles.</td></tr>
                    <tr><td><code>ocr_screen_region</code></td><td><code>string</code></td><td>Ejecuta OCR en una región y devuelve el texto. Actualmente implementado para Windows.</td></tr>
                </tbody>
            </table>

            <h3>Ejemplo: ocultar ventana</h3>
            <pre><code>const { invoke } = window.__TAURI__.core;

document.querySelector('#done').addEventListener('click', async () => {
  await invoke('hide_current_window');
});</code></pre>

            <h3>Ejemplo: OCR de una región</h3>
            <pre><code>const { invoke } = window.__TAURI__.core;

await invoke('extract_text_from_screen_region', {
  x: 100,
  y: 120,
  width: 500,
  height: 220
});</code></pre>

            <h3>Eventos de overlay</h3>
            <p>Las Actions de tipo <code>overlay</code> pueden escuchar eventos emitidos por LUMA.</p>
            <table class="docs-table">
                <thead><tr><th>Evento</th><th>Payload</th><th>Uso</th></tr></thead>
                <tbody>
                    <tr><td><code>luma-overlay-start</code></td><td><code>{ x, y }</code></td><td>LUMA avisa que el overlay está activo y entrega el origen de pantalla.</td></tr>
                    <tr><td><code>luma-native-selection</code></td><td><code>{ x, y, width, height, done, cancel }</code></td><td>Selección nativa de pantalla para flujos como Extract Text.</td></tr>
                </tbody>
            </table>
            <pre><code>listen('luma-overlay-start', (event) => {
  console.log(event.payload.x, event.payload.y);
});</code></pre>
        </section>

        <section id="ui">
            <h2>UI y experiencia</h2>
            <p>Una Action debe sentirse inmediata. Evita pantallas de bienvenida, menús innecesarios y flujos largos.</p>
            <ul>
                <li>Abre directamente en la tarea principal.</li>
                <li>Usa botones claros para acciones finales: copiar, aplicar, guardar, convertir.</li>
                <li>Muestra feedback corto: <code>Copiado</code>, <code>Listo</code>, <code>No se detectó texto</code>.</li>
                <li>Soporta <code>Escape</code> para cancelar o cerrar cuando tenga sentido.</li>
                <li>Evita dependencias grandes si Vanilla HTML/CSS/JS alcanza.</li>
                <li>No bloquees el hilo principal con tareas pesadas.</li>
                <li>No envíes datos a internet sin pedir permiso <code>network</code> y explicarlo.</li>
            </ul>

            <h3>Plantilla mínima</h3>
            <pre><code>&lt;!doctype html&gt;
&lt;html lang="es"&gt;
  &lt;head&gt;
    &lt;meta charset="UTF-8" /&gt;
    &lt;meta name="viewport" content="width=device-width, initial-scale=1.0" /&gt;
    &lt;title&gt;My Action&lt;/title&gt;
    &lt;link rel="stylesheet" href="./styles.css" /&gt;
  &lt;/head&gt;
  &lt;body&gt;
    &lt;main class="action-shell"&gt;
      &lt;button id="run" type="button"&gt;Run&lt;/button&gt;
      &lt;output id="result"&gt;&lt;/output&gt;
    &lt;/main&gt;
    &lt;script src="./action.js"&gt;&lt;/script&gt;
  &lt;/body&gt;
&lt;/html&gt;</code></pre>
        </section>

        <section id="validar">
            <h2>Validar y compilar</h2>
            <p>Antes de publicar, valida que el manifest cumpla el schema y que la entrada exista.</p>
            <pre><code>npm run action:validate -- actions/my-action</code></pre>
            <p>Para compilar localmente:</p>
            <pre><code>npm run action:pack -- actions/my-action</code></pre>
            <p>Para elegir una salida específica:</p>
            <pre><code>npm run action:pack -- actions/my-action dist/actions/my-action.lm</code></pre>
            <p>También puedes usar el <a href="compiler.php">Compilador web</a>: arrastras la carpeta, genera el <code>.lm</code> y luego eliges si descargarlo o publicarlo.</p>
        </section>

        <section id="publicar">
            <h2>Publicar</h2>
            <p>Hay dos caminos para publicar:</p>
            <ol>
                <li>Desde el <a href="compiler.php">Compilador</a>: compila, elige <code>Publicar ahora</code>, revisa los datos pre-rellenados y confirma.</li>
                <li>Desde <a href="upload.php">Subir Action</a>: selecciona un archivo <code>.lm</code>, completa la ficha y publica.</li>
            </ol>
            <p>Al publicar, la web guarda el paquete en:</p>
            <pre><code>web/actions/NOMBRE_DEL_ACTION/NOMBRE_DEL_ACTION_NUM_VERSION.lm</code></pre>
            <p>La ficha pública usa estos datos:</p>
            <ul>
                <li><strong>Nombre público:</strong> hasta 160 caracteres.</li>
                <li><strong>Versión:</strong> debe coincidir con la intención de release.</li>
                <li><strong>Descripción corta:</strong> hasta 280 caracteres.</li>
                <li><strong>Descripción completa:</strong> hasta 5000 caracteres.</li>
                <li><strong>Categoría:</strong> utilidad, diseño, desarrollo, productividad u otro.</li>
                <li><strong>Tags:</strong> palabras separadas por coma.</li>
            </ul>
        </section>

        <section id="instalar">
            <h2>Instalar y actualizar</h2>
            <p>Para instalar una Action en LUMA, abre el launcher, toca el botón <code>+</code> y selecciona el archivo <code>.lm</code>.</p>
            <p>Si instalas una Action con un <code>id</code> nuevo, LUMA la agrega. Si instalas una Action con el mismo <code>id</code> y una <code>version</code> distinta, LUMA la trata como actualización y reemplaza la anterior. Si el <code>id</code> y la <code>version</code> son iguales, LUMA rechaza la instalación porque ya está instalada.</p>
            <p>Por eso, para publicar una nueva versión:</p>
            <ol>
                <li>No cambies el <code>id</code>.</li>
                <li>Sube el número de <code>version</code>.</li>
                <li>Compila de nuevo el paquete.</li>
                <li>Instálalo o publícalo como actualización.</li>
            </ol>
        </section>

        <section id="checklist">
            <h2>Checklist antes de publicar</h2>
            <ul>
                <li>El manifest tiene <code>schemaVersion</code>, <code>id</code>, <code>name</code>, <code>description</code>, <code>version</code>, <code>runtime</code> y <code>permissions</code>.</li>
                <li><code>runtime.entry</code> apunta a un archivo existente dentro de <code>action/</code>.</li>
                <li>Los permisos son los mínimos necesarios.</li>
                <li>Los Components declarados se usan realmente.</li>
                <li>La Action abre directo en la tarea.</li>
                <li>La UI responde rápido y no se queda cargando sin feedback.</li>
                <li>Escape cancela overlays o flujos de selección.</li>
                <li>No hay archivos pesados o sin uso dentro del paquete.</li>
                <li>La versión cambió si es una actualización.</li>
                <li>Probaste instalar el <code>.lm</code> desde el botón <code>+</code> de LUMA.</li>
            </ul>
        </section>

        <section id="problemas">
            <h2>Problemas comunes</h2>
            <table class="docs-table">
                <thead><tr><th>Problema</th><th>Causa probable</th><th>Solución</th></tr></thead>
                <tbody>
                    <tr><td>LUMA no instala el paquete.</td><td>Falta <code>manifest.json</code> en la raíz o el archivo no es ZIP válido.</td><td>Compila de nuevo y verifica la estructura.</td></tr>
                    <tr><td>La Action no aparece.</td><td>Manifest inválido o instalación rechazada.</td><td>Ejecuta <code>npm run action:validate -- actions/my-action</code>.</td></tr>
                    <tr><td>La actualización no se aplica.</td><td>El <code>version</code> no cambió.</td><td>Incrementa la versión conservando el mismo <code>id</code>.</td></tr>
                    <tr><td>La ventana abre en blanco.</td><td><code>runtime.entry</code> apunta a un archivo incorrecto o falta un asset.</td><td>Revisa rutas relativas desde <code>action/index.html</code>.</td></tr>
                    <tr><td>Un overlay no cierra.</td><td>No maneja cancelación o Escape.</td><td>Escucha <code>keydown</code> y llama a <code>hide_current_window</code>.</td></tr>
                    <tr><td>OCR o pantalla no funciona.</td><td>Faltan permisos o Component.</td><td>Agrega <code>screen:read</code>, <code>screen:overlay</code> y, si corresponde, <code>luma.component.ocr</code>.</td></tr>
                </tbody>
            </table>
        </section>
    </article>
</main>
<?php render_footer(); ?>
