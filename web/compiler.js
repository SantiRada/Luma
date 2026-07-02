document.addEventListener('DOMContentLoaded', () => {
    const packDropZone = document.getElementById('pack-drop-zone');
    const folderInput = document.getElementById('folder-input');
    const packFileList = document.getElementById('pack-file-list');
    const packFilesUl = document.getElementById('pack-files');
    const btnPack = document.getElementById('btn-pack');
    const packStatus = document.getElementById('pack-status');
    const unpackDropZone = document.getElementById('unpack-drop-zone');
    const lmInput = document.getElementById('lm-input');
    const unpackFileList = document.getElementById('unpack-file-list');
    const unpackFilesUl = document.getElementById('unpack-files');
    const btnUnpack = document.getElementById('btn-unpack');
    const unpackStatus = document.getElementById('unpack-status');
    const unpackManifestInfo = document.getElementById('unpack-manifest-info');
    const modal = document.getElementById('error-modal');
    const modalMsg = document.getElementById('error-message');
    const compileResultModal = document.getElementById('compile-result-modal');
    const btnPublishCompiled = document.getElementById('btn-publish-compiled');
    const btnDownloadCompiled = document.getElementById('btn-download-compiled');
    const compilerRoot = document.querySelector('.compiler-container');

    let filesToPack = [];
    let currentManifestInfo = null;
    let currentZipBlob = null;
    let lastCompiledBlob = null;
    let lastCompiledName = '';

    function openCompiledDb() {
        return new Promise((resolve, reject) => {
            const request = indexedDB.open('luma-compiled-actions', 1);
            request.onupgradeneeded = () => request.result.createObjectStore('pending');
            request.onsuccess = () => resolve(request.result);
            request.onerror = () => reject(request.error);
        });
    }

    async function savePendingCompiledAction(blob, filename, manifest) {
        const db = await openCompiledDb();
        await new Promise((resolve, reject) => {
            const transaction = db.transaction('pending', 'readwrite');
            transaction.objectStore('pending').put({ blob, filename, manifest, createdAt: Date.now() }, 'latest');
            transaction.oncomplete = resolve;
            transaction.onerror = () => reject(transaction.error);
        });
        db.close();
    }

    function showError(message) {
            modalMsg.innerText = message;
            modal.classList.add('active');
    }

    document.querySelectorAll('.close-modal').forEach((button) => {
        button.addEventListener('click', () => {
            modal?.classList.remove('active');
            compileResultModal?.classList.remove('active');
        });
    });

    function downloadBlob(blob, filename) {
        const url = URL.createObjectURL(blob);
        const link = document.createElement('a');
        link.href = url;
        link.download = filename;
        document.body.appendChild(link);
        link.click();
        link.remove();
        setTimeout(() => URL.revokeObjectURL(url), 1000);
    }

    function preventDefaults(event) {
        event.preventDefault();
        event.stopPropagation();
    }

    async function handlePackFiles(files) {
        filesToPack = files;
        packFilesUl.innerHTML = '';
        packFileList.style.display = 'block';
        btnPack.disabled = true;
        packStatus.innerText = 'Validando archivos...';
        packStatus.className = 'status-msg';

        let manifestData = null;
        for (const item of files) {
            const li = document.createElement('li');
            li.innerText = item.path;
            packFilesUl.appendChild(li);

            if (item.path.toLowerCase() === 'manifest.json') {
                try {
                    manifestData = JSON.parse(await item.file.text());
                } catch (_error) {
                    packStatus.innerText = 'manifest.json no es JSON valido.';
                    packStatus.className = 'status-msg error';
                    return;
                }
            }
        }

        if (!manifestData) {
            packStatus.innerText = 'No se encontro manifest.json en la raiz.';
            packStatus.className = 'status-msg error';
            return;
        }

        if (!manifestData.id || !manifestData.name || !manifestData.version || !manifestData.runtime) {
            packStatus.innerText = 'El manifest debe incluir id, name, version y runtime.';
            packStatus.className = 'status-msg error';
            return;
        }

        currentManifestInfo = manifestData;
        packStatus.innerText = `Validado: ${manifestData.name} (${files.length} archivos).`;
        packStatus.className = 'status-msg success';
        btnPack.disabled = false;
    }

    packDropZone.addEventListener('click', () => folderInput.click());
    ['dragenter', 'dragover', 'dragleave', 'drop'].forEach((name) => packDropZone.addEventListener(name, preventDefaults));
    ['dragenter', 'dragover'].forEach((name) => packDropZone.addEventListener(name, () => packDropZone.classList.add('dragover')));
    ['dragleave', 'drop'].forEach((name) => packDropZone.addEventListener(name, () => packDropZone.classList.remove('dragover')));

    packDropZone.addEventListener('drop', (event) => {
        const files = Array.from(event.dataTransfer.files).map((file) => ({ file, path: file.webkitRelativePath || file.name }));
        handlePackFiles(files);
    });

    folderInput.addEventListener('change', (event) => {
        const files = Array.from(event.target.files).map((file) => {
            const parts = (file.webkitRelativePath || file.name).split('/');
            if (parts.length > 1) parts.shift();
            return { file, path: parts.join('/') };
        });
        handlePackFiles(files);
    });

    btnPack.addEventListener('click', async () => {
        btnPack.disabled = true;
        btnPack.innerText = 'Generando...';
        try {
            const zip = new JSZip();
            filesToPack.forEach((item) => zip.file(item.path, item.file));
            const blob = await zip.generateAsync({ type: 'blob', compression: 'DEFLATE' });
            const outName = `${currentManifestInfo.id || 'action'}.lm`;
            lastCompiledBlob = blob;
            lastCompiledName = outName;
            packStatus.innerText = 'Empaquetado exitoso. Elige si quieres descargarlo o publicarlo.';
            packStatus.className = 'status-msg success';
            compileResultModal?.classList.add('active');
        } catch (error) {
            showError(`Error al compilar: ${error.message}`);
        } finally {
            btnPack.innerText = 'Generar .lm';
            btnPack.disabled = false;
        }
    });

    btnDownloadCompiled?.addEventListener('click', () => {
        if (!lastCompiledBlob) return;
        downloadBlob(lastCompiledBlob, lastCompiledName || 'action.lm');
        compileResultModal?.classList.remove('active');
    });

    btnPublishCompiled?.addEventListener('click', async () => {
        if (!lastCompiledBlob) return;
        try {
            await savePendingCompiledAction(lastCompiledBlob, lastCompiledName || 'action.lm', currentManifestInfo || {});
            if (compilerRoot?.dataset.loggedIn !== '1') {
                window.location.href = `login.php?next=${encodeURIComponent('upload.php?compiled=1')}`;
                return;
            }
            window.location.href = 'upload.php?compiled=1';
        } catch (error) {
            showError(`No se pudo preparar la publicación: ${error.message}`);
        }
    });

    unpackDropZone.addEventListener('click', () => lmInput.click());
    ['dragenter', 'dragover', 'dragleave', 'drop'].forEach((name) => unpackDropZone.addEventListener(name, preventDefaults));
    ['dragenter', 'dragover'].forEach((name) => unpackDropZone.addEventListener(name, () => unpackDropZone.classList.add('dragover')));
    ['dragleave', 'drop'].forEach((name) => unpackDropZone.addEventListener(name, () => unpackDropZone.classList.remove('dragover')));
    unpackDropZone.addEventListener('drop', (event) => event.dataTransfer.files[0] && handleUnpackFile(event.dataTransfer.files[0]));
    lmInput.addEventListener('change', (event) => event.target.files[0] && handleUnpackFile(event.target.files[0]));

    async function handleUnpackFile(file) {
        if (!file.name.endsWith('.lm') && !file.name.endsWith('.zip')) {
            showError('El archivo debe ser .lm o .zip.');
            return;
        }

        unpackFileList.style.display = 'block';
        unpackFilesUl.innerHTML = '';
        unpackManifestInfo.innerHTML = '';
        btnUnpack.disabled = true;
        unpackStatus.innerText = 'Leyendo archivo...';
        currentZipBlob = file;

        try {
            const zip = await JSZip.loadAsync(await file.arrayBuffer());
            let manifestFile = null;
            let fileCount = 0;
            zip.forEach((relativePath, entry) => {
                if (entry.dir) return;
                fileCount++;
                const li = document.createElement('li');
                li.innerText = relativePath;
                unpackFilesUl.appendChild(li);
                if (relativePath.toLowerCase() === 'manifest.json') manifestFile = entry;
            });

            if (manifestFile) {
                const manifest = JSON.parse(await manifestFile.async('string'));
                unpackManifestInfo.innerHTML = `<strong>ID:</strong> ${manifest.id || 'N/A'}<br><strong>Nombre:</strong> ${manifest.name || 'N/A'}<br><strong>Version:</strong> ${manifest.version || 'N/A'}`;
            }

            unpackStatus.innerText = `Cargado con exito (${fileCount} archivos).`;
            unpackStatus.className = 'status-msg success';
            btnUnpack.disabled = false;
        } catch (error) {
            unpackStatus.innerText = 'No se pudo leer el paquete.';
            unpackStatus.className = 'status-msg error';
            showError(error.message);
        }
    }

    btnUnpack.addEventListener('click', () => {
        if (!currentZipBlob) return;
        downloadBlob(currentZipBlob, currentZipBlob.name.replace(/\.lm$/i, '') + '_source.zip');
    });
});
