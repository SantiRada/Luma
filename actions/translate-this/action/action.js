const invoke = window.__TAURI__?.core?.invoke;
const currentWindow = window.__TAURI__?.window?.getCurrentWindow?.();

const source = document.querySelector('#source');
const result = document.querySelector('#result');
const status = document.querySelector('#status');
const language = document.querySelector('#language');
const translateButton = document.querySelector('#translate');

const COMPACT_SIZE = { width: 520, height: 420 };
const LONG_SIZE = { width: 880, height: 520 };
let currentRequest = 0;
let currentSizeKey = '';

function setStatus(message, isError = false) {
  status.textContent = message;
  status.classList.toggle('is-error', isError);
}

function resizeArea(element) {
  element.style.height = 'auto';
  const maxHeight = Math.floor(window.innerHeight * 0.9) - 96;
  const nextHeight = Math.max(82, Math.min(element.scrollHeight + 2, maxHeight));
  element.style.height = `${nextHeight}px`;
}

function refreshLayout() {
  const isLong = source.value.length > 400;
  document.body.classList.toggle('is-long', isLong);
  resizeArea(source);
  resizeArea(result);

  const nextSize = isLong ? LONG_SIZE : COMPACT_SIZE;
  const nextSizeKey = `${nextSize.width}x${nextSize.height}`;
  if (nextSizeKey !== currentSizeKey && typeof invoke === 'function') {
    currentSizeKey = nextSizeKey;
    invoke('show_current_window_panel', nextSize).catch(() => {});
  }
}

async function translate() {
  const text = source.value.trim();
  const requestId = currentRequest + 1;
  currentRequest = requestId;

  if (!text) {
    result.textContent = '';
    setStatus('');
    refreshLayout();
    return;
  }

  translateButton.disabled = true;
  setStatus('Traduciendo...');

  try {
    if (typeof invoke !== 'function') {
      throw new Error('LUMA no expuso el traductor para esta ventana.');
    }

    const translated = await invoke('translate_text', {
      text,
      targetLanguage: language.value,
    });

    if (requestId !== currentRequest) return;
    result.textContent = translated || '';
    setStatus(translated ? 'Traduccion lista.' : 'No hubo texto para traducir.');
  } catch (error) {
    if (requestId !== currentRequest) return;
    result.textContent = '';
    setStatus(`No se pudo traducir: ${error}`, true);
  } finally {
    if (requestId === currentRequest) {
      translateButton.disabled = false;
      refreshLayout();
    }
  }
}

source.addEventListener('input', () => {
  refreshLayout();
});

language.addEventListener('change', () => {
  if (source.value.trim()) {
    translate().catch(() => {});
  }
});

translateButton.addEventListener('click', () => {
  translate().catch(() => {});
});

window.addEventListener('keydown', (event) => {
  if (event.key === 'Escape') {
    event.preventDefault();
    if (currentWindow) {
      currentWindow.close().catch(() => window.close());
    } else {
      window.close();
    }
  }

  if ((event.ctrlKey || event.metaKey) && event.key === 'Enter') {
    event.preventDefault();
    translate().catch(() => {});
  }
});

window.addEventListener('resize', () => {
  resizeArea(source);
  resizeArea(result);
});

refreshLayout();
