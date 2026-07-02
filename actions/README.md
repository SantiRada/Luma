# LUMA Actions

Una Action es la unidad instalable de LUMA: una mini app enfocada en una sola tarea, descubierta por el launcher y ejecutada desde el escritorio.

Las Actions compiladas usan extensión `.lm`. Un `.lm` es un archivo ZIP con `manifest.json` en la raíz y los archivos de ejecución dentro de `action/`.

## Crear una Action

Estructura mínima:

```text
my-action/
  manifest.json
  README.md
  action/
    index.html
    styles.css
    action.js
```

Puedes partir desde `actions/template`.

## Manifest mínimo

```json
{
  "schemaVersion": "1.0.0",
  "id": "luma.action.example",
  "name": "Example Action",
  "description": "Does one focused thing.",
  "version": "0.1.0",
  "author": "LUMA",
  "runtime": {
    "type": "window",
    "entry": "action/index.html",
    "width": 520,
    "height": 360
  },
  "permissions": [],
  "components": [],
  "tags": ["example"]
}
```

Reglas principales:

- `schemaVersion` debe ser `1.0.0`.
- `id` debe empezar con `luma.action.` y mantenerse estable para futuras versiones.
- `version` usa semver, por ejemplo `0.1.0`.
- `runtime.entry` debe apuntar a un archivo dentro de `action/`.
- `permissions` es obligatorio aunque esté vacío.
- `components` se declara solo si la Action necesita un Component compartido.

## Runtime

- `window`: ventana compacta con UI propia. Es el punto de partida recomendado.
- `overlay`: capa transparente sobre la pantalla. Úsalo para selección visual o lectura de pantalla.
- `background`: reservado para ejecución sin UI visible. El schema lo acepta, pero LUMA todavía no lo abre como runtime productivo.

Tamaños permitidos para `window`:

- `width`: 280 a 1200.
- `height`: 180 a 900.

## Permisos

Permisos soportados:

- `clipboard:read`
- `clipboard:write`
- `screen:read`
- `screen:overlay`
- `microphone`
- `audio:system`
- `files:read`
- `files:write`
- `network`

Declara solo los permisos necesarios.

## Components

Los Components son capacidades compartidas que LUMA puede instalar y precargar una vez para varias Actions.

Component disponible:

- `luma.component.ocr`: OCR compartido para extraer texto desde imágenes o regiones de pantalla.

Ejemplo:

```json
{
  "components": ["luma.component.ocr"],
  "permissions": ["screen:read", "screen:overlay", "clipboard:write"]
}
```

## API disponible

Las Actions corren en una ventana Tauri y pueden usar APIs expuestas por LUMA:

```js
const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;
const currentWindow = window.__TAURI__.window.getCurrentWindow();
```

Comandos actuales:

- `hide_current_window`: oculta la ventana actual.
- `get_virtual_screen_bounds`: devuelve `{ x, y, width, height }` para overlays multi-monitor.
- `extract_text_from_screen_region`: recibe `{ x, y, width, height }`, ejecuta OCR y copia el texto al portapapeles.
- `ocr_screen_region`: recibe `{ x, y, width, height }` y devuelve texto. Actualmente implementado en Windows.

Eventos de overlay:

- `luma-overlay-start`: payload `{ x, y }`.
- `luma-native-selection`: payload `{ x, y, width, height, done, cancel }`.

## Validar

```bash
npm run action:validate -- actions/my-action
```

## Compilar

```bash
npm run action:pack -- actions/my-action
```

Con salida personalizada:

```bash
npm run action:pack -- actions/my-action dist/actions/my-action.lm
```

También puedes usar la web en `web/compiler.php`: arrastras la carpeta, generas el `.lm` y eliges si descargarlo o publicarlo.

## Instalar y actualizar

Instala una Action desde LUMA con el botón `+` y selecciona el `.lm`.

Regla de actualización:

- `id` nuevo: LUMA instala una nueva Action.
- mismo `id` y distinta `version`: LUMA actualiza la Action anterior.
- mismo `id` y misma `version`: LUMA rechaza la instalación porque ya está instalada.

Para publicar una nueva versión, conserva el `id` y cambia `version`.

## Checklist

- El manifest valida contra `actions/schema/lm-action.schema.json`.
- `runtime.entry` existe dentro de `action/`.
- La Action abre directo en la tarea.
- Los permisos son mínimos.
- Los Components declarados se usan realmente.
- El paquete no incluye archivos pesados o sin uso.
- Escape cancela overlays o flujos de selección.
- La versión cambió si es una actualización.
- El `.lm` fue probado instalándolo desde LUMA.
