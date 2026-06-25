# LUMA Actions

Actions are the extension unit for LUMA.

An Action is not an app, not a tool, and not a plugin. It is a focused capability that LUMA can discover, show in the launcher, and execute.

Compiled Actions use the `.lm` extension.

## Source Structure

Each Action starts as a folder with this structure:

```text
my-action/
  manifest.json
  action/
    index.html
    action.js
    styles.css
```

The `manifest.json` file is the contract. The `action/` folder contains the Action runtime files.

## Bundle Structure

A `.lm` file is a zip archive with a required root manifest:

```text
my-action-1.0.0.lm
  manifest.json
  action/
    index.html
    action.js
    styles.css
```

LUMA will install and execute Actions from this bundle format.

## Installing

Use the `+` button in LUMA and select a `.lm` file. LUMA extracts the Action into the user data folder and runs it from the desktop app, not from a browser.

## Manifest

Minimum manifest:

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
    "entry": "action/index.html"
  },
  "permissions": [],
  "tags": ["example"]
}
```

## Runtime Types

- `window`: opens a compact Action-owned UI.
- `overlay`: opens a transparent screen overlay.
- `background`: runs without visible UI.

The first Actions should use `window` unless they truly need an overlay or background execution.

## Permissions

Actions must request only the permissions they need:

- `clipboard:read`
- `clipboard:write`
- `screen:read`
- `screen:overlay`
- `microphone`
- `audio:system`
- `files:read`
- `files:write`
- `network`

## UI Guidelines

- Keep the Action focused on one job.
- Avoid onboarding copy inside the UI.
- Prefer one compact panel over multi-screen flows.
- Use the LUMA dark, minimal visual language.
- Use native controls where possible.
- Return a result quickly and make copy/export obvious.

## Commands

Validate an Action:

```bash
npm run action:validate -- actions/template
```

Compile an Action into `.lm`:

```bash
npm run action:pack -- actions/template
```

Custom output:

```bash
npm run action:pack -- actions/template dist/actions/example.lm
```
