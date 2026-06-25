const pickButton = document.querySelector('#pick');
const copyButton = document.querySelector('#copy');
const hexInput = document.querySelector('#hex');
const preview = document.querySelector('#preview');
const statusText = document.querySelector('#status');

function setStatus(message) {
  statusText.textContent = message;
}

function setColor(hex) {
  hexInput.value = hex.toUpperCase();
  preview.style.background = hex;
}

async function copyHex() {
  const hex = hexInput.value;

  try {
    await navigator.clipboard.writeText(hex);
    setStatus(`${hex} copiado al portapapeles.`);
  } catch (_error) {
    hexInput.select();
    setStatus('No se pudo copiar automaticamente. Copialo manualmente.');
  }
}

async function pickColor() {
  if (!('EyeDropper' in window)) {
    setStatus('El gotero no esta disponible en este entorno.');
    return;
  }

  try {
    setStatus('Elige un punto de la pantalla.');
    const eyeDropper = new window.EyeDropper();
    const result = await eyeDropper.open();
    setColor(result.sRGBHex);
    await copyHex();
  } catch (error) {
    if (error && error.name === 'AbortError') {
      setStatus('Seleccion cancelada.');
      return;
    }

    setStatus('No se pudo leer el color.');
  }
}

pickButton.addEventListener('click', pickColor);
copyButton.addEventListener('click', copyHex);

setColor(hexInput.value);
