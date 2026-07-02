const { emit, listen } = window.__TAURI__.event;

const canvas = document.querySelector('#capture');
const context = canvas.getContext('2d', { willReadFrequently: true });

let tesseractPromise = null;
let workerPromise = null;
let ready = false;

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
      cacheMethod: 'none',
    });

    workerPromise.then(() => {
      ready = true;
      emit('luma-component-ready', { componentId: 'luma.component.ocr' });
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

async function recognize(requestId, capture) {
  try {
    const worker = await getWorker();
    const result = await worker.recognize(captureToCanvas(capture));
    const text = (result?.data?.text || '').trim();
    await emit('luma-component-ocr-result', { requestId, text });
  } catch (error) {
    await emit('luma-component-ocr-result', {
      requestId,
      error: String(error || 'OCR failed.'),
    });
  }
}

listen('luma-component-ocr-recognize', (event) => {
  const { requestId, capture } = event.payload || {};
  if (!requestId || !capture) return;
  recognize(requestId, capture);
});

listen('luma-component-ocr-warmup', () => {
  getWorker().catch((error) => {
    ready = false;
    emit('luma-component-error', {
      componentId: 'luma.component.ocr',
      error: String(error || 'OCR warmup failed.'),
    });
  });
});

window.setTimeout(() => {
  if (!ready) {
    getWorker().catch(() => {});
  }
}, 100);
