# LUMA

LUMA is a minimal desktop launcher for focused Actions distributed as `.lm` bundles.

The app runs in the background. Press `Shift + |` to open a small centered launcher, search for an Action, and run it.

## Run locally

```bash
npm install
npm start
```

The first `npm start` compiles the native Tauri app once. After that, `npm start` runs the compiled debug executable directly, which avoids recompiling on every launch.

Use this only when changing native Rust/Tauri code:

```bash
npm run dev
```

Tauri also needs the Rust toolchain and Microsoft build tools installed locally. On Windows, install:

- Rust: `https://rustup.rs`
- Visual Studio Build Tools with MSVC and Windows SDK: `https://aka.ms/vs/17/release/vs_BuildTools.exe`

## Current state

- Tauri desktop shell with global shortcut.
- Single-instance app behavior.
- Minimal black-and-white launcher UI.
- Local Action discovery from manifests.
- `.lm` Action packaging documentation.
- Concept manifests for the first planned Actions.

## Actions

LUMA extensions are called Actions. Source Actions live in `actions/` and compiled bundles use `.lm`.

Validate an Action:

```bash
npm run action:validate -- actions/template
```

Compile an Action:

```bash
npm run action:pack -- actions/template
```

## Documentation

- [Actions](./actions/README.md)
