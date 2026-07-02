const fs = require('fs');
const path = require('path');

const allowedPermissions = new Set([
  'clipboard:read',
  'clipboard:write',
  'screen:read',
  'screen:overlay',
  'microphone',
  'audio:system',
  'files:read',
  'files:write',
  'network',
]);

const allowedComponents = new Set([
  'luma.component.ocr',
  'luma.component.video-downloader',
]);

function readJson(filePath) {
  return JSON.parse(fs.readFileSync(filePath, 'utf8'));
}

function fail(message) {
  throw new Error(message);
}

function validateAction(actionDir) {
  const absoluteActionDir = path.resolve(actionDir);
  const manifestPath = path.join(absoluteActionDir, 'manifest.json');

  if (!fs.existsSync(manifestPath)) {
    fail(`Missing manifest.json in ${absoluteActionDir}`);
  }

  const manifest = readJson(manifestPath);

  if (manifest.schemaVersion !== '1.0.0') {
    fail('manifest.schemaVersion must be "1.0.0".');
  }

  if (!/^luma\.action\.[a-z0-9][a-z0-9.-]*$/.test(manifest.id || '')) {
    fail('manifest.id must match luma.action.<name>.');
  }

  if (!manifest.name || manifest.name.length > 48) {
    fail('manifest.name is required and must be 48 characters or fewer.');
  }

  if (!manifest.description || manifest.description.length > 140) {
    fail('manifest.description is required and must be 140 characters or fewer.');
  }

  if (!/^[0-9]+\.[0-9]+\.[0-9]+(?:[-+][0-9A-Za-z.-]+)?$/.test(manifest.version || '')) {
    fail('manifest.version must be semantic version format, such as 1.0.0.');
  }

  if (!manifest.runtime || !['window', 'overlay', 'background'].includes(manifest.runtime.type)) {
    fail('manifest.runtime.type must be window, overlay, or background.');
  }

  if (!manifest.runtime.entry || !manifest.runtime.entry.startsWith('action/')) {
    fail('manifest.runtime.entry must point inside the action/ folder.');
  }

  const entryPath = path.join(absoluteActionDir, manifest.runtime.entry);
  if (!fs.existsSync(entryPath)) {
    fail(`Runtime entry not found: ${manifest.runtime.entry}`);
  }

  if (!Array.isArray(manifest.permissions)) {
    fail('manifest.permissions must be an array.');
  }

  for (const permission of manifest.permissions) {
    if (!allowedPermissions.has(permission)) {
      fail(`Unsupported permission: ${permission}`);
    }
  }

  if (manifest.components !== undefined) {
    if (!Array.isArray(manifest.components)) {
      fail('manifest.components must be an array when provided.');
    }

    for (const component of manifest.components) {
      if (!allowedComponents.has(component)) {
        fail(`Unsupported component: ${component}`);
      }
    }
  }

  return {
    actionDir: absoluteActionDir,
    manifest,
    manifestPath,
  };
}

if (require.main === module) {
  const actionDir = process.argv[2];

  if (!actionDir) {
    console.error('Usage: npm run action:validate -- <action-folder>');
    process.exit(1);
  }

  try {
    const result = validateAction(actionDir);
    console.log(`Action valid: ${result.manifest.name} (${result.manifest.id})`);
  } catch (error) {
    console.error(error.message);
    process.exit(1);
  }
}

module.exports = {
  validateAction,
};
