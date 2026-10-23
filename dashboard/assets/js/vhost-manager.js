// Virtual Hosts & External Projects Manager Client Logic

let isProcessingVhost = false;

function openVhostModal() {
    document.getElementById('vhostModal').classList.add('show');
    loadVhosts();
}

function closeVhostModal() {
    document.getElementById('vhostModal').classList.remove('show');
}

function closeVhostModalBackdrop(e) {
    if (e.target && e.target.id === 'vhostModal') {
        closeVhostModal();
    }
}

function onVhostPathInput() {
    document.getElementById('vhostDetectPreview').style.display = 'none';
}

function cancelVhostDetect() {
    document.getElementById('vhostDetectPreview').style.display = 'none';
    document.getElementById('vhostInputPath').value = '';
}

function detectVhost() {
    const pathInput = document.getElementById('vhostInputPath');
    const rawPath = (pathInput.value || '').trim();
    if (!rawPath) {
        alert('Silakan masukkan path folder project terlebih dahulu.');
        pathInput.focus();
        return;
    }

    const btn = document.getElementById('btnVhostDetect');
    const origHtml = btn.innerHTML;
    btn.innerHTML = '<span class="inline-spinner"></span> Memeriksa...';
    btn.disabled = true;

    fetch('?action=vhost_detect&path=' + encodeURIComponent(rawPath))
        .then(function(r) { return r.json(); })
        .then(function(data) {
            btn.innerHTML = origHtml;
            btn.disabled = false;
            if (!data.ok) {
                alert('Gagal mendeteksi folder:\n' + (data.error || 'Path tidak ditemukan atau bukan direktori valid.'));
                return;
            }

            document.getElementById('vhostInputName').value = data.name || '';
            document.getElementById('vhostInputDomain').value = data.domain || '';
            document.getElementById('vhostPreviewFramework').textContent = data.framework || 'Standard PHP (root)';
            document.getElementById('vhostPreviewDocRoot').textContent = data.doc_root || '';
            document.getElementById('vhostDetectPreview').style.display = 'block';
        })
        .catch(function(err) {
            btn.innerHTML = origHtml;
            btn.disabled = false;
            alert('Terjadi kesalahan koneksi: ' + err.message);
        });
}

function submitAddVhost() {
    if (isProcessingVhost) return;

    const pathInput = document.getElementById('vhostInputPath');
    const nameInput = document.getElementById('vhostInputName');
    const domainInput = document.getElementById('vhostInputDomain');
    const btn = document.getElementById('btnSubmitAddVhost');

    const path = (pathInput.value || '').trim();
    const name = (nameInput.value || '').trim();
    const domain = (domainInput.value || '').trim();

    if (!path) {
        alert('Path folder tidak boleh kosong.');
        return;
    }

    isProcessingVhost = true;
    btn.innerHTML = '<span class="inline-spinner"></span> Menghubungkan...';
    btn.disabled = true;

    const formData = new FormData();
    formData.append('path', path);
    if (name) formData.append('name', name);
    if (domain) formData.append('domain', domain);

    fetch('?action=vhost_add', {
        method: 'POST',
        body: formData,
    })
    .then(function(r) { return r.json(); })
    .then(function(data) {
        isProcessingVhost = false;
        btn.innerHTML = '<span>Hubungkan Project & Aktifkan VHost</span>';
        btn.disabled = false;

        if (data.ok) {
            showToast('Project ' + (name || domain) + ' berhasil dihubungkan!');
            cancelVhostDetect();
            loadVhosts();
        } else {
            alert('Gagal menghubungkan project:\n' + (data.error || 'Terjadi kesalahan sistem.'));
        }
    })
    .catch(function(err) {
        isProcessingVhost = false;
        btn.innerHTML = '<span>Hubungkan Project & Aktifkan VHost</span>';
        btn.disabled = false;
        alert('Terjadi kesalahan: ' + err.message);
    });
}

function loadVhosts() {
    const listEl = document.getElementById('vhostList');
    const countSpan = document.getElementById('vhostCount');

    listEl.innerHTML = '<div style="text-align:center;padding:16px;color:#64748b;font-size:12.5px;">Memuat daftar virtual host...</div>';

    fetch('?action=vhost_list')
        .then(function(r) { return r.json(); })
        .then(function(data) {
            if (!data.ok) {
                listEl.innerHTML = '<div style="color:#ef4444;padding:10px;font-size:12.5px;">Gagal memuat daftar virtual host.</div>';
                return;
            }

            const list = data.vhosts || [];
            countSpan.textContent = list.length + ' project terhubung';

            if (list.length === 0) {
                listEl.innerHTML = `
                    <div style="background:#ffffff;border:1px dashed #cbd5e1;border-radius:6px;padding:24px 16px;text-align:center;">
                        <svg width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="#94a3b8" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round" style="margin-bottom:8px;">
                            <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"></path>
                        </svg>
                        <div style="font-size:13px;font-weight:600;color:#334155;margin-bottom:4px;">Belum Ada Project Eksternal Terhubung</div>
                        <div style="font-size:12px;color:#64748b;max-width:440px;margin:0 auto;line-height:1.4;">
                            Anda dapat menghubungkan folder project apa pun di hard drive Anda tanpa memindahkannya dari lokasi aslinya.
                        </div>
                    </div>
                `;
                return;
            }

            const serverPort = window.SERVISAN_PORT || (window.location.port ? parseInt(window.location.port, 10) : 8080);
            const portSuffix = serverPort === 80 ? '' : ':' + serverPort;

            let html = '';
            list.forEach(function(item) {
                const domainUrl = 'http://' + item.domain + portSuffix;
                const folderUrl = 'http://localhost' + portSuffix + '/' + item.name + '/';

                html += `
                    <div class="php-version-card" style="margin-bottom:10px;align-items:flex-start;padding:12px 14px;">
                        <div style="flex:1;min-width:0;padding-right:12px;">
                            <div style="display:flex;align-items:center;gap:8px;margin-bottom:4px;flex-wrap:wrap;">
                                <strong style="font-size:14px;color:#0f172a;">${escapeHtml(item.name)}</strong>
                                <a href="${escapeHtml(domainUrl)}" target="_blank" style="font-size:12px;color:#0284c7;font-family:monospace;font-weight:600;text-decoration:none;display:inline-flex;align-items:center;gap:3px;" title="Buka via domain">
                                    <span>${escapeHtml(item.domain)}${escapeHtml(portSuffix)}</span>
                                    <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"></path><polyline points="15 3 21 3 21 9"></polyline><line x1="10" y1="14" x2="21" y2="3"></line></svg>
                                </a>
                                <span class="badge-source servisan" style="font-size:10px;">${escapeHtml(item.framework || (item.is_laravel ? 'Laravel / public' : 'PHP Root'))}</span>
                                ${!item.hosts_synced ? `
                                    <span class="badge-warning-pill" style="margin-top:0;font-size:10.5px;padding:1px 6px;" title="Domain ${escapeHtml(item.domain)} belum terdaftar di Windows hosts (butuh izin Administrator). Gunakan alias .test atau tambahkan ke hosts.">
                                        <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><circle cx="12" cy="12" r="10"></circle><line x1="12" y1="8" x2="12" y2="12"></line><line x1="12" y1="16" x2="12.01" y2="16"></line></svg> Hosts pending
                                    </span>
                                ` : ''}
                            </div>
                            <div style="font-size:11.5px;color:#64748b;font-family:Consolas,monospace;word-break:break-all;line-height:1.4;">
                                <span style="color:#94a3b8;">Target:</span> ${escapeHtml(item.target_path)}
                            </div>
                            <div style="font-size:11px;color:#0369a1;margin-top:4px;display:flex;align-items:center;gap:12px;flex-wrap:wrap;">
                                <a href="${escapeHtml(folderUrl)}" target="_blank" style="color:#0369a1;text-decoration:none;">
                                    Subfolder: localhost${escapeHtml(portSuffix)}/${escapeHtml(item.name)}/
                                </a>
                                <span style="color:#94a3b8;">&bull;</span>
                                <a href="http://${escapeHtml(item.name)}.test${escapeHtml(portSuffix)}" target="_blank" style="color:#0369a1;text-decoration:none;" title="Buka via standar .test">
                                    Alias: ${escapeHtml(item.name)}.test${escapeHtml(portSuffix)}
                                </a>
                            </div>
                        </div>
                        <div style="display:flex;align-items:center;gap:6px;flex-shrink:0;">
                            <a href="${escapeHtml(domainUrl)}" target="_blank" class="btn-action-sm outline" style="text-decoration:none;display:inline-flex;align-items:center;gap:4px;" title="Buka di Browser">
                                <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"></circle><line x1="2" y1="12" x2="22" y2="12"></line><path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"></path></svg>
                                <span>Buka</span>
                            </a>
                            <button type="button" class="btn-action-sm outline" onclick="openProjectFolder('${escapeHtml(item.target_path.replace(/\\/g, '\\\\'))}')" title="Buka di Explorer">
                                <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"></path></svg>
                                <span>Folder</span>
                            </button>
                            <button type="button" class="btn-action-sm outline" onclick="openProjectTerminal('${escapeHtml(item.target_path.replace(/\\/g, '\\\\'))}')" title="Buka Shell di Folder Ini">
                                <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="4 17 10 11 4 5"></polyline><line x1="12" y1="19" x2="20" y2="19"></line></svg>
                                <span>Shell</span>
                            </button>
                            <button type="button" class="btn-action-sm danger" onclick="deleteVhost('${escapeHtml(item.name)}', '${escapeHtml(item.domain)}')" title="Putuskan project & hapus VHost">
                                <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                                    <polyline points="3 6 5 6 21 6"></polyline>
                                    <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"></path>
                                </svg>
                            </button>
                        </div>
                    </div>
                `;
            });
            listEl.innerHTML = html;
        })
        .catch(function(err) {
            listEl.innerHTML = `<div style="color:#ef4444;padding:10px;font-size:12.5px;">Error: ${escapeHtml(err.message)}</div>`;
        });
}

function openProjectFolder(path) {
    const formData = new FormData();
    formData.append('path', path);
    fetch('?action=open_project_folder', { method: 'POST', body: formData })
        .then(function() { showToast('Membuka folder project di Explorer...'); })
        .catch(function() {});
}

function openProjectTerminal(path) {
    const formData = new FormData();
    formData.append('path', path);
    fetch('?action=open_project_terminal', { method: 'POST', body: formData })
        .then(function() { showToast('Membuka terminal shell di direktori project...'); })
        .catch(function() {});
}

function deleteVhost(name, domain) {
    if (!confirm(`Putuskan project "${name}" (${domain})?\n\nTautan junction di www/ dan domain di file hosts akan dihapus. File asli project Anda tidak akan tersentuh.`)) {
        return;
    }

    const formData = new FormData();
    formData.append('target', name);

    fetch('?action=vhost_remove', {
        method: 'POST',
        body: formData,
    })
    .then(function(r) { return r.json(); })
    .then(function(data) {
        if (data.ok) {
            showToast('Project ' + name + ' berhasil diputuskan.');
            loadVhosts();
        } else {
            alert('Gagal memutuskan project: ' + (data.error || 'Terjadi kesalahan.'));
        }
    })
    .catch(function(err) {
        alert('Terjadi kesalahan: ' + err.message);
    });
}
