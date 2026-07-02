document.addEventListener('DOMContentLoaded', () => {
    function openCompiledDb() {
        return new Promise((resolve, reject) => {
            const request = indexedDB.open('luma-compiled-actions', 1);
            request.onupgradeneeded = () => request.result.createObjectStore('pending');
            request.onsuccess = () => resolve(request.result);
            request.onerror = () => reject(request.error);
        });
    }

    async function getPendingCompiledAction() {
        const db = await openCompiledDb();
        const pending = await new Promise((resolve, reject) => {
            const transaction = db.transaction('pending', 'readonly');
            const request = transaction.objectStore('pending').get('latest');
            request.onsuccess = () => resolve(request.result || null);
            request.onerror = () => reject(request.error);
        });
        db.close();
        return pending;
    }

    async function clearPendingCompiledAction() {
        const db = await openCompiledDb();
        await new Promise((resolve, reject) => {
            const transaction = db.transaction('pending', 'readwrite');
            transaction.objectStore('pending').delete('latest');
            transaction.oncomplete = resolve;
            transaction.onerror = () => reject(transaction.error);
        });
        db.close();
    }

    function updateCounter(input) {
        const max = Number(input.dataset.max || input.getAttribute('maxlength') || 0);
        if (!max || !input.id) return true;

        const counter = document.querySelector(`.char-counter[data-for="${input.id}"]`);
        if (!counter) return true;

        const length = input.value.length;
        counter.textContent = `(${length}/${max})`;
        counter.classList.toggle('is-limit', length === max);
        counter.classList.toggle('is-over', length > max);
        input.classList.toggle('is-invalid', length > max);

        return length <= max;
    }

    document.querySelectorAll('[data-max]').forEach((input) => {
        input.addEventListener('input', () => updateCounter(input));
        updateCounter(input);
    });

    document.querySelectorAll('.js-limited-form').forEach((form) => {
        form.addEventListener('submit', (event) => {
            const valid = Array.from(form.querySelectorAll('[data-max]')).every(updateCounter);
            if (!valid) {
                event.preventDefault();
            }
        });
    });

    document.querySelectorAll('.file-picker input[type="file"]').forEach((input) => {
        const nameNode = input.closest('.file-picker')?.querySelector('.file-picker-name');
        input.addEventListener('change', () => {
            if (!nameNode) return;
            nameNode.textContent = input.files?.[0]?.name || nameNode.dataset.empty || 'Seleccionar archivo';
            if (input.files?.[0]) {
                input.form?.removeAttribute('data-compiled-ready');
            }
        });
    });

    async function hydrateCompiledUpload() {
        const form = document.querySelector('form[data-upload-form]');
        if (!form || !new URLSearchParams(window.location.search).has('compiled')) return;

        const fileInput = form.querySelector('input[type="file"][name="lm_file"]');
        const fileName = form.querySelector('.file-picker-name');
        const status = document.getElementById('compiled-upload-status');

        try {
            const pending = await getPendingCompiledAction();
            if (!pending?.blob) {
                if (status) {
                    status.textContent = 'No encontré un archivo compilado pendiente. Selecciona un .lm manualmente.';
                    status.className = 'status-msg warning';
                    status.hidden = false;
                }
                return;
            }

            form.dataset.compiledReady = '1';
            form._compiledAction = pending;
            if (fileInput) fileInput.required = false;
            if (fileName) fileName.textContent = pending.filename || 'Action compilada.lm';

            const manifest = pending.manifest || {};
            const fields = {
                name: manifest.name || '',
                version: manifest.version || '0.1.0',
                short_description: manifest.description || '',
                description: manifest.description || '',
                tags: Array.isArray(manifest.tags) ? manifest.tags.join(',') : '',
            };

            Object.entries(fields).forEach(([name, value]) => {
                const input = form.elements[name];
                if (input && !input.value) {
                    input.value = value;
                    input.dispatchEvent(new Event('input', { bubbles: true }));
                }
            });

            if (status) {
                status.textContent = 'Archivo compilado listo para publicar.';
                status.className = 'status-msg success';
                status.hidden = false;
            }
        } catch (error) {
            if (status) {
                status.textContent = `No se pudo cargar la Action compilada: ${error.message}`;
                status.className = 'status-msg error';
                status.hidden = false;
            }
        }
    }

    document.querySelectorAll('form[data-upload-form]').forEach((form) => {
        form.addEventListener('submit', async (event) => {
            if (event.defaultPrevented || form.dataset.compiledReady !== '1' || !form._compiledAction?.blob) return;
            event.preventDefault();

            const submitButton = form.querySelector('[type="submit"]');
            const status = document.getElementById('compiled-upload-status');
            const formData = new FormData(form);
            formData.delete('lm_file');
            formData.append('lm_file', new File([form._compiledAction.blob], form._compiledAction.filename || 'action.lm', { type: 'application/octet-stream' }));

            if (submitButton) submitButton.disabled = true;
            if (status) {
                status.textContent = 'Publicando...';
                status.className = 'status-msg';
                status.hidden = false;
            }

            try {
                const response = await fetch('publish-action.php', { method: 'POST', body: formData });
                const result = await response.json();
                if (!result.ok) throw new Error(result.error || 'No se pudo publicar.');
                await clearPendingCompiledAction();
                window.location.href = `action.php?slug=${encodeURIComponent(result.slug)}`;
            } catch (error) {
                if (status) {
                    status.textContent = error.message;
                    status.className = 'status-msg error';
                }
                if (submitButton) submitButton.disabled = false;
            }
        });
    });

    hydrateCompiledUpload();
});
