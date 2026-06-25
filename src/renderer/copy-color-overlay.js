const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

const overlay = document.querySelector('#overlay');
const loupe = document.querySelector('#loupe');
const grid = document.querySelector('#grid');
const hex = document.querySelector('#hex');
const swatch = document.querySelector('#swatch');
const hint = document.querySelector('#loupe p');

let latestHex = '#FFFFFF';
let active = false;
let pointerX = 24;
let pointerY = 24;
let screenOriginX = 0;
let screenOriginY = 0;
let raf = 0;

function buildGrid() {
  grid.innerHTML = '';

  for (let index = 0; index < 81; index += 1) {
    const pixel = document.createElement('span');
    pixel.className = index === 40 ? 'pixel center' : 'pixel';
    grid.appendChild(pixel);
  }
}

function colorFromRgbInt(color) {
  const rgb = Number(color || 0) & 0xFFFFFF;
  return `#${rgb.toString(16).padStart(6, '0').toUpperCase()}`;
}

function renderSample(sample) {
  if (!active) return;

  latestHex = sample.hex;
  hex.textContent = sample.hex;
  swatch.style.background = sample.hex;
  hint.textContent = 'Click to Copy';

  const pixels = grid.children;
  sample.pixels.forEach((color, index) => {
    pixels[index].style.background = colorFromRgbInt(color);
  });
  scheduleLoupe();
}

function placeLoupe() {
  raf = 0;

  const gapX = 24;
  const gapY = 18;
  const width = 286;
  const height = 96;
  let x = pointerX + gapX;
  let y = pointerY + gapY;

  if (x + width > window.innerWidth) {
    x = pointerX - width - gapX;
  }

  if (y + height > window.innerHeight) {
    y = pointerY - height - gapY;
  }

  x = Math.max(10, Math.min(x, window.innerWidth - width - 10));
  y = Math.max(10, Math.min(y, window.innerHeight - height - 10));
  loupe.style.transform = `translate3d(${x}px, ${y}px, 0)`;
}

function scheduleLoupe() {
  if (!raf) {
    raf = requestAnimationFrame(placeLoupe);
  }
}

function stopSampling() {
  cancelAnimationFrame(raf);
  raf = 0;
}

overlay.addEventListener('pointermove', (event) => {
  pointerX = event.clientX;
  pointerY = event.clientY;
  scheduleLoupe();
});

async function finish() {
  if (!active) return;
  active = false;
  stopSampling();
  invoke('finish_copy_color', { hex: latestHex });
}

async function cancel() {
  if (!active) return;
  active = false;
  stopSampling();
  await invoke('cancel_copy_color');
}

overlay.addEventListener('pointerdown', (event) => {
  event.preventDefault();
  event.stopPropagation();
});

overlay.addEventListener('click', (event) => {
  event.preventDefault();
  event.stopPropagation();
  finish();
});

window.addEventListener('keydown', (event) => {
  if (event.key === 'Escape') {
    event.preventDefault();
    event.stopPropagation();
    cancel();
  }
});

buildGrid();

listen('copy-color-start', (event) => {
  active = true;
  latestHex = '#FFFFFF';
  screenOriginX = event.payload?.x || 0;
  screenOriginY = event.payload?.y || 0;
  hex.textContent = latestHex;
  swatch.style.background = latestHex;
  hint.textContent = 'Click to Copy';
  scheduleLoupe();
});

listen('copy-color-sample', (event) => {
  renderSample(event.payload);
});
