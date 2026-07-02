const invoke = window.__TAURI__?.core?.invoke;
const currentWindow = window.__TAURI__?.window?.getCurrentWindow?.();
const source = document.querySelector('#source');
const result = document.querySelector('#result');
const status = document.querySelector('#status');
const language = document.querySelector('#language');
const translateButton = document.querySelector('#translate');
let currentRequest = 0;

function log(message) {
  if (typeof invoke !== 'function') return;
  invoke('luma_debug_log', { message: `translate-this: ${message}` }).catch(() => {});
}

log(`script loaded href=${window.location.href}`);

function setStatus(message, isError = false) {
  status.textContent = message;
  status.classList.toggle('is-error', isError);
  log(`status="${message}" error=${isError}`);
}

function resizeArea(element) {
  element.style.height = 'auto';
  const maxHeight = Math.floor(window.innerHeight * 0.9) - 96;
  element.style.height = `${Math.max(82, Math.min(element.scrollHeight + 2, maxHeight))}px`;
}

function refreshLayout() {
  document.body.classList.toggle('is-long', source.value.length > 400);
  resizeArea(source);
  resizeArea(result);
}

async function translate() {
  log('translate clicked');
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
      throw new Error('LUMA no expuso el traductor.');
    }
    log(`translate request target=${language.value} chars=${text.length}`);
    const translated = await invoke('translate_text', { text, targetLanguage: language.value });
    if (requestId !== currentRequest) return;
    result.textContent = translated || '';
    log(`translate result chars=${(translated || '').length}`);
    setStatus(translated ? 'Traduccion lista.' : 'No hubo texto para traducir.');
  } catch (error) {
    if (requestId !== currentRequest) return;
    result.textContent = '';
    log(`translate failed ${error}`);
    setStatus(`No se pudo traducir: ${error}`, true);
  } finally {
    if (requestId === currentRequest) {
      translateButton.disabled = false;
      refreshLayout();
    }
  }
}

function closeWindow() {
  log('close requested');
  if (typeof invoke === 'function') {
    invoke('hide_current_window').catch(() => window.close());
    return;
  }
  window.close();
}

source.addEventListener('input', refreshLayout);
source.addEventListener('input', () => log(`input chars=${source.value.length}`));
language.addEventListener('change', () => {
  log(`language changed ${language.value}`);
  if (source.value.trim()) translate().catch(() => {});
});
translateButton.addEventListener('click', () => translate().catch(() => {}));
document.querySelector('#close').addEventListener('click', closeWindow);
window.addEventListener('resize', () => {
  resizeArea(source);
  resizeArea(result);
});
window.addEventListener('keydown', (event) => {
  if (event.key === 'Escape') {
    event.preventDefault();
    closeWindow();
  }
  if ((event.ctrlKey || event.metaKey) && event.key === 'Enter') {
    event.preventDefault();
    translate().catch(() => {});
  }
});

refreshLayout();
log('initial layout rendered');
requestAnimationFrame(() => source.focus());
