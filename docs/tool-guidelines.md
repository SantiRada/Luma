# Tool Product Guidelines

LUMA tools should feel fast, focused, and almost invisible.

## Launcher behavior

- A tool must have one clear job.
- The launcher copy should be short enough to scan in under a second.
- Tool names should be direct: `Image Plus`, `Copiar color`, `TT`.
- Tags should cover user intent, not implementation details.

## UI principles

- Prefer one compact panel over multi-step screens.
- Use black and white as the base palette.
- Keep controls native-feeling and predictable.
- Do not add onboarding text unless the tool cannot be understood without it.
- Use DM Sans or the system sans-serif fallback.
- Use subtle texture, borders, and shadow; avoid decorative color.

## Runtime patterns

Use the smallest surface that fits the task:

- `background-action`: quick actions like copying a generated value.
- `window`: tools that need inputs, choices, preview, or results.
- `overlay`: tools that need screen selection, canvas drawing, or eyedropper behavior.

## Suggested contracts for the first tools

### Traductor de imagenes

Input:

- Clipboard image.
- Source language.
- Target language.

Output:

- Detected and translated text.
- Copy button.

Permissions:

- `clipboard:read`
- `clipboard:write`
- `network`

### Copiar de imagenes

Input:

- Clipboard image.
- User-drawn rectangle.

Output:

- OCR text copied to clipboard.

Permissions:

- `clipboard:read`
- `clipboard:write`
- `screen:overlay`

### Copiar color

Input:

- One screen click.

Output:

- HEX value copied to clipboard.

Permissions:

- `screen:read`
- `clipboard:write`

### Transcribir audio

Input:

- System audio stream.

Output:

- Transcript text.

Permissions:

- `audio:system`
- `clipboard:write`

### TT

Input:

- Microphone speech.
- Target language.

Output:

- Transcribed and translated text.

Permissions:

- `microphone`
- `clipboard:write`
- `network`

### Image Plus

Input:

- Local image file.
- Target format.

Output:

- Converted file.

Permissions:

- `files:read`
- `files:write`

### Audio Plus

Input:

- Local audio file.
- Target format.

Output:

- Converted file.

Permissions:

- `files:read`
- `files:write`

### Video Downloader

Input:

- Video URL.
- Quality choice.
- Audio/video choice.

Output:

- Downloaded media file.

Permissions:

- `network`
- `files:write`
