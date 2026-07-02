# Translate Image

Translate Image permite marcar una región de la pantalla, extraer el texto con OCR y traducirlo automáticamente al español.

## Flujo

1. El usuario ejecuta la Action desde LUMA.
2. LUMA muestra el selector de región en pantalla.
3. Al soltar el mouse, la Action lee el texto con OCR.
4. El texto detectado se traduce automáticamente a español.
5. La traducción se muestra en una ventana pequeña con un botón para copiar.

## Permisos

- `screen:read`: captura la región seleccionada.
- `screen:overlay`: permite dibujar el selector sobre la pantalla.
- `clipboard:write`: permite copiar la traducción.
- `network`: permite consultar el servicio de traducción.
