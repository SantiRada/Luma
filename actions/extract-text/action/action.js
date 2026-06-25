const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;
const currentWindow = window.__TAURI__.window.getCurrentWindow();

const overlay = document.querySelector('#overlay');
const selection = document.querySelector('#selection');
const hint = document.querySelector('#hint');
const canvas = document.querySelector('#capture');
const context = canvas.getContext('2d', { willReadFrequently: true });

let active = false;
let dragging = false;
let originX = 0;
let originY = 0;
let startX = 0;
let startY = 0;
let currentX = 0;
let currentY = 0;
let tesseractPromise = null;
let workerPromise = null;

function setHint(text) {
  hint.textContent = text;
}

function drawSelection() {
  const x = Math.min(startX, currentX);
  const y = Math.min(startY, currentY);
  const width = Math.abs(currentX - startX);
  const height = Math.abs(currentY - startY);

  selection.style.display = width > 2 && height > 2 ? 'block' : 'none';
  selection.style.transform = `translate3d(${x}px, ${y}px, 0)`;
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

function sleep(ms) {
  return new Promise((resolve) => window.setTimeout(resolve, ms));
}

function loadTesseract() {
  if (window.Tesseract) return Promise.resolve(window.Tesseract);

  if (!tesseractPromise) {
    tesseractPromise = new Promise((resolve, reject) => {
      const script = document.createElement('script');
      script.src = './vendor/tesseract.min.js';
      script.onload = () => resolve(window.Tesseract);
      script.onerror = () => reject(new Error('OCR runtime not found.'));
      document.head.appendChild(script);
    });
  }

  return tesseractPromise;
}

async function getWorker() {
  if (!workerPromise) {
    const TesseractRuntime = await loadTesseract();
    workerPromise = TesseractRuntime.createWorker('eng', 1, {
      workerPath: './vendor/worker.min.js',
      corePath: './vendor/tesseract-core.wasm.js',
      langPath: './vendor/lang-data',
      gzip: false,
      workerBlobURL: false,
      logger: (message) => {
        if (message.status === 'recognizing text' && typeof message.progress === 'number') {
          setHint(`Reading ${Math.round(message.progress * 100)}%`);
        }
      },
    });
  }

  return workerPromise;
}

function captureToCanvas(capture) {
  canvas.width = capture.width;
  canvas.height = capture.height;
  const pixels = new Uint8ClampedArray(capture.pixels);
  context.putImageData(new ImageData(pixels, capture.width, capture.height), 0, 0);
  return canvas;
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
  setHint('Reading text...');
  await currentWindow.hide();
  await sleep(90);

  const capture = await invoke('capture_screen_region', {
    x: Math.round(originX + left),
    y: Math.round(originY + top),
    width: Math.round(width),
    height: Math.round(height),
  });

  const worker = await getWorker();
  const result = await worker.recognize(captureToCanvas(capture));
  const text = (result?.data?.text || '').trim();

  if (text) {
    await invoke('write_clipboard_text', { text });
  }
}

async function cancel() {
  if (!active) return;
  setActive(false);
  resetSelection();
  await currentWindow.hide();
}

overlay.addEventListener('pointerdown', (event) => {
  if (!active) return;

  event.preventDefault();
  event.stopPropagation();
  overlay.setPointerCapture(event.pointerId);
  dragging = true;
  startX = event.clientX;
  startY = event.clientY;
  currentX = startX;
  currentY = startY;
  setHint('Release to extract');
  drawSelection();
});

overlay.addEventListener('pointermove', (event) => {
  if (!active || !dragging) return;

  event.preventDefault();
  event.stopPropagation();
  currentX = event.clientX;
  currentY = event.clientY;
  drawSelection();
});

overlay.addEventListener('pointerup', (event) => {
  if (!active || !dragging) return;

  event.preventDefault();
  event.stopPropagation();
  currentX = event.clientX;
  currentY = event.clientY;
  extractSelection().catch(() => currentWindow.hide());
});

window.addEventListener('keydown', (event) => {
  if (event.key === 'Escape') {
    event.preventDefault();
    event.stopPropagation();
    cancel();
  }
});

async function start(payload) {
  try {
    const bounds = payload || await invoke('get_virtual_screen_bounds');
    originX = bounds.x || 0;
    originY = bounds.y || 0;
    setActive(true);
    resetSelection();
    setHint('Drag to select text');
  } catch (_error) {
    setActive(false);
    await currentWindow.hide();
  }
}

listen('luma-overlay-start', (event) => {
  start(event.payload);
});

window.setTimeout(() => {
  if (!active) start();
}, 50);
