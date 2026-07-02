const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;
const currentWindow = window.__TAURI__.window.getCurrentWindow();

const selection = document.querySelector('#selection');
const hint = document.querySelector('#hint');
const title = document.querySelector('#title');
const translation = document.querySelector('#translation');
const status = document.querySelector('#status');
const copyButton = document.querySelector('#copy');
const closeButton = document.querySelector('#close');

let active = false;
let translatedText = '';

function setHint(text) {
  hint.textContent = text;
}

function resetSelection() {
  selection.style.display = 'none';
}

function setActive(nextActive) {
  active = nextActive;
  document.body.classList.toggle('is-active', nextActive);
}

async function hideWindow() {
  try {
    await currentWindow.hide();
  } catch (_error) {
    // Best effort. Shift+Tab remains the hard kill switch.
  }
}

function showPanel(message) {
  document.body.classList.add('is-panel');
  title.textContent = 'Traduciendo...';
  translation.textContent = message || 'Preparando OCR...';
  status.textContent = message || 'Detectando idioma y traduciendo a espa\u00f1ol.';
  status.classList.remove('is-error');
  copyButton.disabled = true;
  translatedText = '';
}

function showResult(text) {
  translatedText = text.trim();
  title.textContent = 'Traducci\u00f3n lista';
  translation.textContent = translatedText || 'No se detect\u00f3 texto para traducir.';
  status.textContent = translatedText ? 'Listo para copiar.' : 'La regi\u00f3n seleccionada no ten\u00eda texto legible.';
  status.classList.remove('is-error');
  copyButton.disabled = !translatedText;
}

function showError(error) {
  document.body.classList.add('is-panel');
  setActive(false);
  resetSelection();
  translatedText = '';
  title.textContent = 'No se pudo traducir';
  translation.textContent = '';
  status.textContent = String(error || 'Ocurri\u00f3 un error al traducir.');
  status.classList.add('is-error');
  copyButton.disabled = true;
}

function updateNativeSelection(nativeSelection) {
  if (!nativeSelection) return;

  if (nativeSelection.cancel) {
    cancel();
    return;
  }

  if (!active) {
    setActive(true);
  }

  selection.style.display = nativeSelection.width > 2 && nativeSelection.height > 2 ? 'block' : 'none';
  selection.style.left = `${nativeSelection.x}px`;
  selection.style.top = `${nativeSelection.y}px`;
  selection.style.width = `${nativeSelection.width}px`;
  selection.style.height = `${nativeSelection.height}px`;
  setHint(nativeSelection.done ? 'Traduciendo...' : 'Soltar para traducir');

  if (nativeSelection.done) {
    setActive(false);
    resetSelection();
    showPanel('Leyendo texto...');
  }
}

async function cancel() {
  document.body.classList.remove('is-panel');
  setActive(false);
  resetSelection();
  await hideWindow();
}

copyButton.addEventListener('click', async () => {
  if (!translatedText) return;

  try {
    await navigator.clipboard.writeText(translatedText);
  } catch (_error) {
    try {
      await invoke('write_clipboard_text', { text: translatedText });
    } catch (_fallbackError) {
      status.textContent = 'No se pudo copiar.';
      return;
    }
  }

  status.textContent = 'Copiado.';
});

closeButton.addEventListener('click', () => {
  cancel().catch(() => {});
});

window.addEventListener('keydown', (event) => {
  if (event.key === 'Escape') {
    event.preventDefault();
    event.stopPropagation();
    cancel();
  }
});

function start() {
  document.body.classList.remove('is-panel');
  setActive(true);
  resetSelection();
  setHint('Arrastra para seleccionar texto');
}

listen('luma-overlay-start', () => {
  start();
});

listen('luma-native-selection', (event) => {
  updateNativeSelection(event.payload);
});

listen('luma-translate-image-status', (event) => {
  const message = event.payload?.message;
  if (typeof message === 'string' && message.trim()) {
    showPanel(message);
  }
});

listen('luma-translate-image-result', (event) => {
  const payload = event.payload || {};
  if (typeof payload.error === 'string' && payload.error.trim()) {
    showError(payload.error);
    return;
  }

  showResult(typeof payload.text === 'string' ? payload.text : '');
});
