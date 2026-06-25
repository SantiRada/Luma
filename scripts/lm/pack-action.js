const fs = require('fs');
const path = require('path');
const AdmZip = require('adm-zip');
const { validateAction } = require('./validate-action');

const ignoredNames = new Set([
  '.DS_Store',
  'README.md',
  'Thumbs.db',
]);

function safeBundleName(manifest) {
  const slug = manifest.id.replace(/^luma\.action\./, '').replace(/[^a-z0-9.-]/gi, '-');
  return `${slug}-${manifest.version}.lm`;
}

function addDirectory(zip, sourceDir, rootDir) {
  for (const entry of fs.readdirSync(sourceDir, { withFileTypes: true })) {
    if (ignoredNames.has(entry.name)) continue;

    const absolutePath = path.join(sourceDir, entry.name);
    const relativePath = path.relative(rootDir, absolutePath).replace(/\\/g, '/');

    if (entry.isDirectory()) {
      addDirectory(zip, absolutePath, rootDir);
      continue;
    }

    zip.addLocalFile(absolutePath, path.posix.dirname(relativePath));
  }
}

function packAction(actionDir, outputFile) {
  const { actionDir: absoluteActionDir, manifest } = validateAction(actionDir);
  const outputPath = path.resolve(
    outputFile || path.join('dist', 'actions', safeBundleName(manifest))
  );

  fs.mkdirSync(path.dirname(outputPath), { recursive: true });

  const zip = new AdmZip();
  addDirectory(zip, absoluteActionDir, absoluteActionDir);
  zip.writeZip(outputPath);

  return {
    manifest,
    outputPath,
  };
}

if (require.main === module) {
  const actionDir = process.argv[2];
  const outputFile = process.argv[3];

  if (!actionDir) {
    console.error('Usage: npm run action:pack -- <action-folder> [output-file.lm]');
    process.exit(1);
  }

  try {
    const result = packAction(actionDir, outputFile);
    console.log(`Action packed: ${result.outputPath}`);
  } catch (error) {
    console.error(error.message);
    process.exit(1);
  }
}

module.exports = {
  packAction,
};
