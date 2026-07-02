const invoke = window.__TAURI__?.core?.invoke;

const urlInput = document.querySelector('#url');
const preview = document.querySelector('#preview');
const platformIcon = document.querySelector('#platformIcon');
const videoTitle = document.querySelector('#videoTitle');
const platformLabel = document.querySelector('#platformLabel');
const qualitySelect = document.querySelector('#quality');
const downloadButton = document.querySelector('#download');
const status = document.querySelector('#status');

let previewTimer = 0;
let detectedVideo = null;
let isWorking = false;

const platformCopy = {
  youtube: { text: 'YT', label: 'YouTube' },
  instagram: { text: 'IG', label: 'Instagram' },
  twitter: { text: 'X', label: 'Twitter / X' },
  tiktok: { text: 'TT', label: 'TikTok' },
  vimeo: { text: 'V', label: 'Vimeo' },
  facebook: { text: 'F', label: 'Facebook' },
  video: { text: '▶', label: 'Video' },
};

function log(message) {
  if (typeof invoke !== 'function') return;
  invoke('luma_debug_log', { message: `video-downloader: ${message}` }).catch(() => {});
}

function fileName(path) {
  return path.split(/[\\/]/).filter(Boolean).pop() || path;
}

function setStatus(message, isError = false) {
  status.textContent = message;
  status.classList.toggle('error', isError);
}

function setBusy(nextBusy) {
  isWorking = nextBusy;
  urlInput.disabled = nextBusy;
  qualitySelect.disabled = nextBusy || !detectedVideo;
  downloadButton.disabled = nextBusy || !detectedVideo;
}

function resetPreview(message = 'El preview aparecerá al detectar el video.') {
  detectedVideo = null;
  preview.classList.add('empty');
  platformIcon.className = 'platform-icon';
  platformIcon.textContent = '?';
  videoTitle.textContent = 'Esperando enlace';
  platformLabel.textContent = message;
  qualitySelect.replaceChildren(new Option('Mejor calidad', 'best'));
  qualitySelect.disabled = true;
  downloadButton.disabled = true;
}

function renderPreview(data) {
  detectedVideo = data;
  const platform = platformCopy[data.platform] || platformCopy.video;
  preview.classList.remove('empty');
  platformIcon.className = `platform-icon ${data.platform || 'video'}`;
  platformIcon.textContent = platform.text;
  videoTitle.textContent = data.title || 'Video detectado';
  platformLabel.textContent = platform.label;
  qualitySelect.replaceChildren();

  for (const option of data.qualities || [{ id: 'best', label: 'Mejor calidad' }]) {
    qualitySelect.append(new Option(option.label, option.id));
  }

  qualitySelect.disabled = false;
  downloadButton.disabled = false;
}

async function loadPreview() {
  const url = urlInput.value.trim();
  if (!url) {
    resetPreview();
    setStatus('');
    return;
  }

  setStatus('Detectando video...');
  resetPreview('Detectando enlace...');
  log('preview requested');

  try {
    const data = await invoke('video_downloader_preview', { url });
    renderPreview(data);
    setStatus('Video detectado.');
    log(`preview completed platform=${data.platform}`);
  } catch (error) {
    resetPreview('No se pudo detectar el video.');
    setStatus(String(error), true);
    log(`preview failed: ${error}`);
  }
}

function schedulePreview() {
  clearTimeout(previewTimer);
  previewTimer = window.setTimeout(loadPreview, 650);
}

async function downloadVideo() {
  if (!detectedVideo || isWorking || typeof invoke !== 'function') return;

  setBusy(true);
  setStatus('Preparando descarga...');
  log(`download requested quality=${qualitySelect.value}`);

  try {
    const result = await invoke('video_downloader_download', {
      url: urlInput.value.trim(),
      quality: qualitySelect.value,
      title: detectedVideo.title,
    });
    setStatus(`Video guardado: ${fileName(result.outputPath)}`);
    log(`download completed path=${result.outputPath}`);
  } catch (error) {
    const message = String(error);
    setStatus(message === 'Guardado cancelado.' ? message : `No se pudo descargar: ${message}`, true);
    log(`download failed: ${error}`);
  } finally {
    setBusy(false);
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

urlInput.addEventListener('input', schedulePreview);
urlInput.addEventListener('paste', () => window.setTimeout(loadPreview, 80));
downloadButton.addEventListener('click', downloadVideo);
document.querySelector('#close').addEventListener('click', closeWindow);
window.addEventListener('keydown', (event) => {
  if (event.key === 'Escape') {
    event.preventDefault();
    closeWindow();
  }
  if ((event.ctrlKey || event.metaKey) && event.key === 'Enter') {
    event.preventDefault();
    downloadVideo();
  }
});

resetPreview();
log('script loaded');
