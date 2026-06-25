const fs = require('fs');
const path = require('path');

const requiredFiles = [
  'src/renderer/index.html',
  'src/renderer/copy-color-overlay.html',
  'src/renderer/copy-color-overlay.js',
  'src/renderer/copy-color-overlay.css',
  'src/renderer/icons/add_24_regular.svg',
  'src/renderer/styles.css',
  'src/renderer/renderer.js',
  'actions/README.md',
  'actions/schema/lm-action.schema.json',
  'actions/template/manifest.json',
  'actions/template/action/index.html',
  'scripts/lm/validate-action.js',
  'scripts/lm/pack-action.js',
  'scripts/start.js',
  'src-tauri/Cargo.toml',
  'src-tauri/capabilities/default.json',
  'src-tauri/icons/icon.ico',
  'src-tauri/icons/icon.png',
  'src-tauri/tauri.conf.json',
  'src-tauri/src/main.rs',
  'docs/wsb-format.md',
  'docs/tool-guidelines.md',
];

const missing = requiredFiles.filter((file) => !fs.existsSync(path.join(__dirname, '..', file)));

if (missing.length) {
  console.error(`Missing files:\n${missing.join('\n')}`);
  process.exit(1);
}

console.log('LUMA project files are present.');
