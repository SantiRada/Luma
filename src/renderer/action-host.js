const detail = document.querySelector('#detail');
const closeButton = document.querySelector('#close');
const currentWindow = window.__TAURI__?.window?.getCurrentWindow?.();
const invoke = window.__TAURI__?.core?.invoke;

function setError(message) {
  detail.textContent = message;
  detail.classList.add('error');
}

async function closeHost() {
  if (currentWindow) {
    await currentWindow.close().catch(() => {});
    return;
  }

  window.close();
}

closeButton.addEventListener('click', () => {
  closeHost();
});

window.addEventListener('keydown', (event) => {
  if (event.key === 'Escape') {
    event.preventDefault();
    closeHost();
  }
});

async function bootAction() {
  try {
    if (typeof invoke !== 'function') {
      throw new Error('LUMA no expuso el puente interno para cargar Actions.');
    }

    const html = await invoke('get_current_window_action_html');
    if (!html || typeof html !== 'string') {
      throw new Error('La Action no devolvio contenido HTML.');
    }

    document.open();
    document.write(html);
    document.close();
  } catch (error) {
    setError(String(error || 'No se pudo cargar la Action.'));
  }
}

bootAction();
