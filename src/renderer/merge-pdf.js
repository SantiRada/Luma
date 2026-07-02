const invoke = window.__TAURI__?.core?.invoke;

const picker = document.querySelector('#picker');
const pickerLabel = document.querySelector('#pickerLabel');
const fileCount = document.querySelector('#fileCount');
const fileList = document.querySelector('#fileList');
const clearButton = document.querySelector('#clear');
const exportButton = document.querySelector('#export');
const status = document.querySelector('#status');

let selectedFiles = [];
let isWorking = false;

function log(message) {
  if (typeof invoke !== 'function') return;
  invoke('luma_debug_log', { message: `merge-pdf: ${message}` }).catch(() => {});
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
  fileCount.textContent = `${count} ${count === 1 ? 'archivo' : 'archivos'}`;
  pickerLabel.textContent = count
    ? 'Los PDFs se unirán en el orden mostrado.'
    : 'Elegí dos o más archivos para unirlos en un solo PDF.';
  exportButton.disabled = count < 2 || isWorking;
  clearButton.disabled = count === 0 || isWorking;
}

async function pickFiles() {
  if (isWorking || typeof invoke !== 'function') return;

  setStatus('Seleccionando PDF...');
  log('pick requested');

  try {
    const files = await invoke('merge_pdf_pick_files');
    if (!files.length) {
      setStatus(selectedFiles.length ? 'Selección sin cambios.' : '');
      log('pick canceled');
      return;
    }

    selectedFiles = files;
    renderFiles();
    setStatus(`${files.length} PDF listos.`);
    log(`picked count=${files.length}`);
  } catch (error) {
    setStatus(String(error), true);
    log(`pick failed: ${error}`);
  }
}

async function exportPdf() {
  if (isWorking || selectedFiles.length < 2 || typeof invoke !== 'function') return;

  isWorking = true;
  renderFiles();
  setStatus('Uniendo PDFs...');
  log(`export requested count=${selectedFiles.length}`);

  try {
    const savedPath = await invoke('merge_pdf_export', { files: selectedFiles });
    setStatus(`PDF guardado: ${fileName(savedPath)}`);
    log(`export completed path=${savedPath}`);
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

picker.addEventListener('click', pickFiles);
clearButton.addEventListener('click', clearFiles);
exportButton.addEventListener('click', exportPdf);
document.querySelector('#close').addEventListener('click', closeWindow);
window.addEventListener('keydown', (event) => {
  if (event.key === 'Escape') {
    event.preventDefault();
    closeWindow();
  }
  if ((event.ctrlKey || event.metaKey) && event.key === 'Enter') {
    event.preventDefault();
    exportPdf();
  }
});

renderFiles();
log('script loaded');
