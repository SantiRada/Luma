const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;
const currentWindow = window.__TAURI__.window.getCurrentWindow();

const overlay = document.querySelector('#overlay');
const selection = document.querySelector('#selection');
const hint = document.querySelector('#hint');
const useNativeSelection = true;

let active = false;
let dragging = false;
let originX = 0;
let originY = 0;
let startX = 0;
let startY = 0;
let currentX = 0;
let currentY = 0;
let boundsPromise = null;

function setHint(text) {
  hint.textContent = text;
}

function drawSelection() {
  const x = Math.min(startX, currentX);
  const y = Math.min(startY, currentY);
  const width = Math.abs(currentX - startX);
  const height = Math.abs(currentY - startY);

  selection.style.display = width > 2 && height > 2 ? 'block' : 'none';
  selection.style.left = `${x}px`;
  selection.style.top = `${y}px`;
  selection.style.width = `${width}px`;
  selection.style.height = `${height}px`;
}

function resetSelection() {
  dragging = false;
  selection.style.display = 'none';
}

function setActive(nextActive) {
  active = nextActive;
  document.body.classList.toggle('is-active', nextActive);
}

function withTimeout(promise, ms, message) {
  return Promise.race([
    promise,
    new Promise((_, reject) => {
      window.setTimeout(() => reject(new Error(message)), ms);
    }),
  ]);
}

async function hideOverlay() {
  try {
    await withTimeout(invoke('hide_current_window'), 700, 'Overlay hide timed out.');
  } catch (_error) {
    await withTimeout(currentWindow.hide(), 700, 'Overlay hide timed out.');
  }
}

function applyBounds(bounds) {
  if (!bounds) return;
  originX = Number(bounds.x) || 0;
  originY = Number(bounds.y) || 0;
}

function refreshBounds() {
  if (!boundsPromise) {
    boundsPromise = invoke('get_virtual_screen_bounds')
      .then(applyBounds)
      .finally(() => {
        boundsPromise = null;
      });
  }

  return boundsPromise;
}

function beginDrag(event) {
  event.preventDefault();
  event.stopPropagation();
  if (typeof event.pointerId === 'number') {
    overlay.setPointerCapture(event.pointerId);
  }
  dragging = true;
  startX = event.clientX;
  startY = event.clientY;
  currentX = startX;
  currentY = startY;
  setHint('Release to extract');
  drawSelection();
}

function finishDrag(event) {
  if (!active || !dragging) return;

  event?.preventDefault?.();
  event?.stopPropagation?.();

  if (typeof event?.clientX === 'number') {
    currentX = event.clientX;
    currentY = event.clientY;
  }

  extractSelection().catch(() => cancel());
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

  dragging = !nativeSelection.done;
  selection.style.display = nativeSelection.width > 2 && nativeSelection.height > 2 ? 'block' : 'none';
  selection.style.left = `${nativeSelection.x}px`;
  selection.style.top = `${nativeSelection.y}px`;
  selection.style.width = `${nativeSelection.width}px`;
  selection.style.height = `${nativeSelection.height}px`;
  setHint(nativeSelection.done ? 'Extracting...' : 'Release to extract');

  if (nativeSelection.done) {
    setActive(false);
    resetSelection();
  }
}

async function extractSelection() {
  const left = Math.min(startX, currentX);
  const top = Math.min(startY, currentY);
  const width = Math.abs(currentX - startX);
  const height = Math.abs(currentY - startY);

  if (width < 8 || height < 8) {
    resetSelection();
    setHint('Drag to select text');
    return;
  }

  setActive(false);
  dragging = false;
  hideOverlay().catch(() => {});

  const region = {
    x: Math.round(originX + left),
    y: Math.round(originY + top),
    width: Math.round(width),
    height: Math.round(height),
  };

  invoke('extract_text_from_screen_region', region).catch(() => {});
}

async function cancel() {
  setActive(false);
  resetSelection();
  await hideOverlay();
}

overlay.addEventListener('pointerdown', (event) => {
  if (useNativeSelection) return;
  if (typeof event.button === 'number' && event.button !== 0) return;

  if (!active) {
    start();
  }

  if (!active) return;
  beginDrag(event);
});

overlay.addEventListener('pointermove', (event) => {
  if (useNativeSelection) return;
  if (!active || !dragging) return;

  event.preventDefault();
  event.stopPropagation();
  currentX = event.clientX;
  currentY = event.clientY;
  drawSelection();

  if (typeof event.buttons === 'number' && event.buttons === 0) {
    finishDrag(event);
  }
});

overlay.addEventListener('pointerup', (event) => {
  if (useNativeSelection) return;
  finishDrag(event);
});

if (!useNativeSelection) {
  overlay.addEventListener('pointercancel', finishDrag);
  overlay.addEventListener('lostpointercapture', finishDrag);
  document.addEventListener('pointerup', finishDrag, true);
  document.addEventListener('mouseup', finishDrag, true);
  window.addEventListener('pointerup', finishDrag, true);
  window.addEventListener('mouseup', finishDrag, true);
}

window.addEventListener('keydown', (event) => {
  if (event.key === 'Escape') {
    event.preventDefault();
    event.stopPropagation();
    cancel();
  }
});

function start(payload) {
  applyBounds(payload);
  setActive(true);
  resetSelection();
  setHint('Drag to select text');
  refreshBounds().catch(() => {});
}

listen('luma-overlay-start', (event) => {
  start(event.payload);
});

listen('luma-native-selection', (event) => {
  updateNativeSelection(event.payload);
});

window.addEventListener('focus', () => {
  if (!active && document.visibilityState === 'visible') {
    start();
  }
});

document.addEventListener('visibilitychange', () => {
  if (!active && document.visibilityState === 'visible') {
    start();
  }
});
