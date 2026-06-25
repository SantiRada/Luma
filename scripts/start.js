const { spawn } = require('child_process');
const fs = require('fs');
const path = require('path');

const root = path.join(__dirname, '..');
const executableName = process.platform === 'win32' ? 'luma.exe' : 'luma';
const executablePath = path.join(root, 'src-tauri', 'target', 'debug', executableName);
const cargoBin = path.join(process.env.USERPROFILE || '', '.cargo', 'bin');
const env = {
  ...process.env,
  PATH: `${cargoBin}${path.delimiter}${process.env.PATH || ''}`,
};

function run(command, args, options = {}) {
  return new Promise((resolve, reject) => {
    const child = spawn(command, args, {
      cwd: root,
      env,
      stdio: 'inherit',
      shell: process.platform === 'win32',
      ...options,
    });

    child.on('error', reject);
    child.on('exit', (code) => {
      if (code === 0) {
        resolve();
        return;
      }

      reject(new Error(`${command} exited with code ${code}`));
    });
  });
}

async function main() {
  if (!fs.existsSync(executablePath)) {
    console.log('No hay binario nativo todavia. Compilando una vez...');
    await run('cargo', ['build', '--manifest-path', 'src-tauri/Cargo.toml']);
  }

  await run(executablePath, []);
}

main().catch((error) => {
  console.error(error.message);
  process.exit(1);
});
