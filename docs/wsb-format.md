# LUMA Bundle Format

LUMA tools are distributed as `.wsb` files.

A `.wsb` file is a zip archive with this minimum structure:

```text
my-tool.wsb
  manifest.json
  tool/
    index.html
    renderer.js
    styles.css
```

The launcher reads `manifest.json`, installs the bundle into the user data folder, and uses the manifest to decide how the tool should be shown and executed.

## manifest.json

```json
{
  "schemaVersion": "1.0.0",
  "id": "vendor.tool-name",
  "name": "Tool Name",
  "description": "Short sentence shown in the launcher.",
  "version": "1.0.0",
  "author": "Vendor",
  "status": "ready",
  "tags": ["ocr", "clipboard"],
  "permissions": ["clipboard:read", "clipboard:write"],
  "entry": {
    "type": "window",
    "path": "tool/index.html",
    "width": 640,
    "height": 520
  }
}
```

## Required fields

- `schemaVersion`: current value is `1.0.0`.
- `id`: stable reverse-domain identifier. Example: `acme.image-translator`.
- `name`: display name in the launcher.
- `description`: short explanation for search and selection.
- `version`: semantic version.
- `permissions`: explicit list of capabilities requested by the tool.
- `entry`: execution contract.

## Entry types

Current launcher support:

- `placeholder`: listed in the launcher, not executable yet.
- `clipboard-text`: copies a static `value` to the clipboard. Useful for smoke tests.
- `external-url`: opens `url` in the browser.

Planned support:

- `window`: opens a small tool-owned UI.
- `overlay`: opens a transparent screen overlay.
- `background-action`: runs without visible UI and returns a result.

## Permission names

Initial permission vocabulary:

- `clipboard:read`
- `clipboard:write`
- `screen:read`
- `screen:overlay`
- `microphone`
- `audio:system`
- `files:read`
- `files:write`
- `network`

Every tool should request only what it needs. The store will later use these permissions for review, user consent, and trust signals.

## Store source

The public store endpoint is reserved as:

```text
https://santiagorada.com/wsb
```

The desktop app already keeps this URL as a constant so the future store can be connected without changing the launcher model.
