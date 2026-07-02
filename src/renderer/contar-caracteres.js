const invoke = window.__TAURI__?.core?.invoke;
const source = document.querySelector('#source');
const status = document.querySelector('#status');

const counters = {
  characters: document.querySelector('#characters'),
  spaces: document.querySelector('#spaces'),
  words: document.querySelector('#words'),
  stanzas: document.querySelector('#stanzas'),
  paragraphs: document.querySelector('#paragraphs'),
};

function log(message) {
  if (typeof invoke !== 'function') return;
  invoke('luma_debug_log', { message: `contar-caracteres: ${message}` }).catch(() => {});
}

function resizeInput() {
  source.style.height = 'auto';
  const maxHeight = Math.floor(window.innerHeight * 0.9) - 160;
  source.style.height = `${Math.max(82, Math.min(source.scrollHeight + 2, maxHeight))}px`;
}

function nonEmptyBlocks(text) {
  return text
    .trim()
    .split(/\n\s*\n+/)
    .map((block) => block.trim())
    .filter(Boolean);
}

function countText() {
  const text = source.value;
  const trimmed = text.trim();
  const stanzas = nonEmptyBlocks(text);
  const paragraphs = trimmed
    ? text.split(/\r?\n/).map((line) => line.trim()).filter(Boolean)
    : [];
  const words = trimmed ? trimmed.match(/\S+/g) || [] : [];

  counters.characters.textContent = String([...text].length);
  counters.spaces.textContent = String((text.match(/[ \t]/g) || []).length);
  counters.words.textContent = String(words.length);
  counters.stanzas.textContent = String(stanzas.length);
  counters.paragraphs.textContent = String(paragraphs.length);
  status.textContent = trimmed ? 'Conteo listo.' : '';
  log(`count chars=${[...text].length} words=${words.length}`);
}

function closeWindow() {
  log('close requested');
  if (typeof invoke === 'function') {
    invoke('hide_current_window').catch(() => window.close());
    return;
  }
  window.close();
}

source.addEventListener('input', () => {
  resizeInput();
  log(`input chars=${source.value.length}`);
});
document.querySelector('#count').addEventListener('click', countText);
document.querySelector('#close').addEventListener('click', closeWindow);
window.addEventListener('resize', resizeInput);
window.addEventListener('keydown', (event) => {
  if (event.key === 'Escape') {
    event.preventDefault();
    closeWindow();
  }
  if ((event.ctrlKey || event.metaKey) && event.key === 'Enter') {
    event.preventDefault();
    countText();
  }
});

resizeInput();
log('script loaded');
requestAnimationFrame(() => source.focus());
