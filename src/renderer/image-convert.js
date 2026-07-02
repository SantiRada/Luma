const invoke = window.__TAURI__?.core?.invoke;

const folderMode = document.querySelector('#folderMode');
const picker = document.querySelector('#picker');
const pickerTitle = document.querySelector('#pickerTitle');
const pickerLabel = document.querySelector('#pickerLabel');
const format = document.querySelector('#format');
const prefix = document.querySelector('#prefix');
const fileCount = document.querySelector('#fileCount');
const fileList = document.querySelector('#fileList');
const clearButton = document.querySelector('#clear');
const exportButton = document.querySelector('#export');
const status = document.querySelector('#status');

let selectedFiles = [];
let sourceKind = 'file';
let isWorking = false;

function log(message) {
  if (typeof invoke !== 'function') return;
  invoke('luma_debug_log', { message: `image-convert: ${message}` }).catch(() => {});
}

function fileName(path) {
  return path.split(/[\\/]/).filter(Boolean).pop() || path;
}

function setStatus(message, isError = false) {
  status.textContent = message;
  status.classList.toggle('error', isError);
}

function renderFiles() {
  fileList.replaceChildren();

  for (const file of selectedFiles) {
    const item = document.createElement('li');
    const name = document.createElement('span');
    name.textContent = fileName(file);
    name.title = file;
    item.append(name);
    fileList.append(item);
  }

  const count = selectedFiles.length;
  const label = count === 1 ? 'imagen' : 'imágenes';
  fileCount.textContent = `${count} ${label}`;
  exportButton.disabled = count === 0 || isWorking;
  clearButton.disabled = count === 0 || isWorking;
}

function renderMode() {
  const isFolder = folderMode.checked;
  pickerTitle.textContent = isFolder ? 'Seleccionar carpeta' : 'Seleccionar imagen';
  pickerLabel.textContent = isFolder
    ? 'Usá como input todas las imágenes de una carpeta.'
    : 'Elegí una imagen para convertirla a otro formato.';
}

async function pickInput() {
  if (isWorking || typeof invoke !== 'function') return;

  const isFolder = folderMode.checked;
  setStatus(isFolder ? 'Seleccionando carpeta...' : 'Seleccionando imagen...');
  log(`pick requested folder=${isFolder}`);

  try {
    const selection = await invoke('image_convert_pick_input', { folderMode: isFolder });
    selectedFiles = selection.paths || [];
    sourceKind = selection.sourceKind || (isFolder ? 'folder' : 'file');
    renderFiles();

    if (!selectedFiles.length) {
      setStatus(isFolder ? 'No se encontraron imágenes en esa carpeta.' : '');
      log('pick empty');
      return;
    }

    setStatus(`${selectedFiles.length} ${selectedFiles.length === 1 ? 'imagen lista' : 'imágenes listas'}.`);
    log(`picked count=${selectedFiles.length} source=${sourceKind}`);
  } catch (error) {
    setStatus(String(error), true);
    log(`pick failed: ${error}`);
  }
}

async function exportImages() {
  if (isWorking || !selectedFiles.length || typeof invoke !== 'function') return;

  isWorking = true;
  renderFiles();
  setStatus('Convirtiendo imágenes...');
  log(`export requested count=${selectedFiles.length} format=${format.value}`);

  try {
    const result = await invoke('image_convert_export', {
      files: selectedFiles,
      outputFormat: format.value,
      prefix: prefix.value,
    });
    const count = result.count || 0;
    setStatus(`${count} ${count === 1 ? 'imagen exportada' : 'imágenes exportadas'}.`);
    log(`export completed count=${count}`);
  } catch (error) {
    const message = String(error);
    setStatus(message === 'Guardado cancelado.' ? message : `No se pudo exportar: ${message}`, true);
    log(`export failed: ${error}`);
  } finally {
    isWorking = false;
    renderFiles();
  }
}

function clearFiles() {
  selectedFiles = [];
  sourceKind = folderMode.checked ? 'folder' : 'file';
  renderFiles();
  setStatus('');
  log('selection cleared');
}

function closeWindow() {
  log('close requested');
  if (typeof invoke === 'function') {
    invoke('hide_current_window').catch(() => window.close());
    return;
  }
  window.close();
}

folderMode.addEventListener('change', () => {
  clearFiles();
  renderMode();
  log(`mode changed folder=${folderMode.checked}`);
});
picker.addEventListener('click', pickInput);
clearButton.addEventListener('click', clearFiles);
exportButton.addEventListener('click', exportImages);
document.querySelector('#close').addEventListener('click', closeWindow);
window.addEventListener('keydown', (event) => {
  if (event.key === 'Escape') {
    event.preventDefault();
    closeWindow();
  }
  if ((event.ctrlKey || event.metaKey) && event.key === 'Enter') {
    event.preventDefault();
    exportImages();
  }
});

renderMode();
renderFiles();
log('script loaded');
