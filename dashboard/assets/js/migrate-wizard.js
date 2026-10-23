// Database Migration Wizard Client Logic

if (typeof window.escapeHtml !== 'function') {
    window.escapeHtml = function(str) {
        if (!str) return '';
        return String(str)
            .replace(/&/g, '&amp;')
            .replace(/</g, '&lt;')
            .replace(/>/g, '&gt;')
            .replace(/"/g, '&quot;')
            .replace(/'/g, '&#039;');
    };
}

if (typeof window.formatBytes !== 'function') {
    window.formatBytes = function(bytes) {
        if (!bytes || bytes <= 0) return '0 B';
        const k = 1024;
        const dm = 1;
        const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
        const i = Math.floor(Math.log(bytes) / Math.log(k));
        return parseFloat((bytes / Math.pow(k, i)).toFixed(dm)) + ' ' + sizes[i];
    };
}

let currentMigrateFrom = '';
let migrateScanData = null;
let isMigrating = false;

function openMigrateModal() {
    document.getElementById('migrateModal').classList.add('show');
    loadMigrateScan();
}

function closeMigrateModal() {
    if (isMigrating) {
        showToast('Proses migrasi sedang berlangsung, mohon tunggu...');
        return;
    }
    document.getElementById('migrateModal').classList.remove('show');
}

function scanCustomFolder() {
    if (isMigrating) return;
    const pathInput = document.getElementById('migrateCustomPath');
    const customPath = (pathInput.value || '').trim();
    if (!customPath) {
        showToast('Silakan masukkan path folder.');
        return;
    }
    currentMigrateFrom = customPath;
    const resetBtn = document.getElementById('btnResetAutoScan');
    if (resetBtn) resetBtn.style.display = 'inline-block';
    loadMigrateScan(customPath);
}

function resetToAutoScan() {
    if (isMigrating) return;
    currentMigrateFrom = '';
    const pathInput = document.getElementById('migrateCustomPath');
    if (pathInput) pathInput.value = '';
    const resetBtn = document.getElementById('btnResetAutoScan');
    if (resetBtn) resetBtn.style.display = 'none';
    loadMigrateScan('');
}

function backToMigrationList() {
    if (isMigrating) return;
    const content = document.getElementById('migrateContent');
    const consoleBox = document.getElementById('migrateConsoleBox');
    const btnStart = document.getElementById('btnStartMigrate');
    const btnRescan = document.getElementById('btnMigrateRescan');
    const btnBack = document.getElementById('btnMigrateBack');

    consoleBox.style.display = 'none';
    content.style.display = 'block';
    if (btnBack) btnBack.style.display = 'none';
    if (btnRescan) btnRescan.style.display = 'inline-block';
    if (btnStart) btnStart.style.display = 'inline-block';
    updateSelectedCount();
}

function loadMigrateScan(fromPath) {
    if (isMigrating) return;
    if (typeof fromPath === 'undefined') {
        fromPath = currentMigrateFrom;
    }

    const loading = document.getElementById('migrateLoading');
    const content = document.getElementById('migrateContent');
    const consoleBox = document.getElementById('migrateConsoleBox');
    const btnStart = document.getElementById('btnStartMigrate');
    const btnRescan = document.getElementById('btnMigrateRescan');
    const btnBack = document.getElementById('btnMigrateBack');
    const container = document.getElementById('migrateSources');

    loading.style.display = 'block';
    content.style.display = 'none';
    consoleBox.style.display = 'none';
    if (btnStart) {
        btnStart.style.display = 'inline-block';
        btnStart.disabled = true;
    }
    if (btnRescan) {
        btnRescan.style.display = 'inline-block';
        btnRescan.disabled = false;
    }
    if (btnBack) {
        btnBack.style.display = 'none';
    }
    container.innerHTML = '';

    const url = fromPath ? ('?action=migrate_scan&from=' + encodeURIComponent(fromPath)) : '?action=migrate_scan';

    fetch(url)
        .then(function(res) { return res.json(); })
        .then(function(data) {
            loading.style.display = 'none';
            content.style.display = 'block';
            migrateScanData = data;

            if (!data.ok || !data.sources || data.sources.length === 0) {
                container.innerHTML = `
                    <div style="background:#f8fafc;border:1px dashed #cbd5e1;border-radius:6px;padding:24px;text-align:center;color:#64748b;font-size:13px;">
                        <div style="font-weight:600;color:#334155;margin-bottom:4px;">Tidak ada server lokal atau database terdeteksi</div>
                        <div>Servisan telah memindai path instalasi standar Laragon, XAMPP, dan WAMP pada semua drive komputer Anda.</div>
                        <div style="margin-top:10px;font-size:12px;color:#0284c7;">Gunakan kolom <strong>Pilih Folder Manual</strong> di atas jika instalasi Anda berada di lokasi khusus.</div>
                    </div>
                `;
                return;
            }

            let html = '';
            let totalDbs = 0;

            data.sources.forEach(function(src, sIdx) {
                const isRunning = !!src.is_running;
                const dbs = src.databases || [];
                totalDbs += dbs.length;

                html += `
                    <div style="background:#f8fafc;border:1px solid #e2e8f0;border-radius:6px;padding:12px 14px;margin-bottom:12px;">
                        <div style="display:flex;justify-content:space-between;align-items:center;margin-bottom:8px;">
                            <div style="display:flex;align-items:center;gap:8px;">
                                <strong style="font-size:14px;color:#0f172a;">${escapeHtml(src.source)}</strong>
                                <span style="font-size:12px;color:#64748b;font-family:monospace;">${escapeHtml(src.root)}</span>
                            </div>
                            <div>
                                ${isRunning 
                                    ? '<span style="background:#ecfdf5;color:#059669;padding:2px 8px;border-radius:12px;font-size:11px;font-weight:600;">MySQL Running (Port ' + src.port + ')</span>'
                                    : '<span style="background:#e0f2fe;color:#0284c7;padding:2px 8px;border-radius:12px;font-size:11px;font-weight:600;">Offline (Auto-Start Ready)</span>'
                                }
                            </div>
                        </div>
                `;

                if (!isRunning) {
                    html += `
                        <div style="background:#f0f9ff;border:1px solid #bae6fd;border-radius:6px;padding:8px 12px;font-size:12px;color:#0369a1;margin-bottom:10px;line-height:1.4;display:flex;align-items:flex-start;gap:8px;">
                            <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" style="flex-shrink:0;margin-top:2px;color:#0284c7;"><polygon points="13 2 3 14 12 14 11 22 21 10 12 10 13 2"></polygon></svg>
                            <div>
                                <strong>Zero-Touch Mode:</strong> MySQL ${escapeHtml(src.source)} sedang tidak aktif. Servisan akan otomatis menyalakan instance database di background port darurat secara silent saat migrasi berjalan. Anda tidak perlu menyentuh atau membuka aplikasi ${escapeHtml(src.source)}.
                            </div>
                        </div>
                    `;
                }

                if (dbs.length === 0) {
                    html += '<div style="font-size:12.5px;color:#94a3b8;font-style:italic;">Tidak ada database pengguna ditemukan di instalasi ini.</div>';
                } else {
                    html += `
                        <div style="border:1px solid #e2e8f0;border-radius:6px;overflow:hidden;background:#ffffff;">
                            <div style="background:#f1f5f9;padding:6px 10px;display:flex;align-items:center;justify-content:space-between;font-size:11.5px;font-weight:600;color:#475569;border-bottom:1px solid #e2e8f0;">
                                <label style="display:flex;align-items:center;gap:6px;cursor:pointer;">
                                    <input type="checkbox" onchange="toggleSelectAll(${sIdx}, this.checked)" checked>
                                    <span>Pilih Semua Database</span>
                                </label>
                                <span>Ukuran Estimasi</span>
                            </div>
                            <div style="max-height:160px;overflow-y:auto;">
                    `;

                    dbs.forEach(function(db) {
                        const sizeStr = formatBytes(db.estimated_size);
                        html += `
                            <div style="display:flex;align-items:center;justify-content:space-between;padding:6px 10px;border-bottom:1px solid #f1f5f9;font-size:12.5px;">
                                <label style="display:flex;align-items:center;gap:6px;cursor:pointer;color:#334155;">
                                    <input type="checkbox" class="migrate-db-chk s-${sIdx}" value="${escapeHtml(db.name)}" onchange="updateSelectedCount()" checked>
                                    <span style="font-weight:500;">${escapeHtml(db.name)}</span>
                                </label>
                                <span style="color:#64748b;font-size:11.5px;font-family:monospace;">${sizeStr}</span>
                            </div>
                        `;
                    });

                    html += '</div></div>';
                }

                html += '</div>';
            });

            if (!data.servisan_mysql_ok) {
                html += `
                    <div style="background:#fef2f2;border:1px solid #fee2e2;border-radius:6px;padding:8px 12px;font-size:12px;color:#991b1b;margin-top:10px;display:flex;align-items:center;gap:8px;">
                        <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" style="flex-shrink:0;color:#dc2626;"><circle cx="12" cy="12" r="10"></circle><line x1="12" y1="8" x2="12" y2="12"></line><line x1="12" y1="16" x2="12.01" y2="16"></line></svg>
                        <div>
                            <strong>MySQL Servisan sedang mati.</strong> Pastikan Servisan telah di-Start sebelum menjalankan migrasi.
                        </div>
                    </div>
                `;
            }

            container.innerHTML = html;
            updateSelectedCount();
        })
        .catch(function(err) {
            loading.style.display = 'none';
            content.style.display = 'block';
            container.innerHTML = `<div style="color:#ef4444;font-size:13px;padding:16px;">Gagal memindai server lokal: ${escapeHtml(err.message)}</div>`;
        });
}

function toggleSelectAll(sIdx, checked) {
    document.querySelectorAll('.migrate-db-chk.s-' + sIdx).forEach(function(el) {
        el.checked = checked;
    });
    updateSelectedCount();
}

function updateSelectedCount() {
    const checked = document.querySelectorAll('.migrate-db-chk:checked');
    const count = checked.length;
    const countEl = document.getElementById('migrateSelectedCount');
    if (countEl) {
        countEl.textContent = count + ' database dipilih';
    }
    const btnStart = document.getElementById('btnStartMigrate');
    if (btnStart) {
        btnStart.disabled = (count === 0);
    }
}

function startMigration() {
    if (isMigrating) return;

    const checkedEls = document.querySelectorAll('.migrate-db-chk:checked');
    const checked = [];
    for (let i = 0; i < checkedEls.length; i++) {
        checked.push(checkedEls[i].value);
    }

    if (checked.length === 0) {
        showToast('Pilih setidaknya 1 database untuk dimigrasi.');
        return;
    }

    const overwriteEl = document.getElementById('migrateOverwrite');
    const overwrite = overwriteEl ? overwriteEl.checked : false;

    const content = document.getElementById('migrateContent');
    const consoleBox = document.getElementById('migrateConsoleBox');
    const consoleEl = document.getElementById('migrateConsole');
    const statusEl = document.getElementById('migrateConsoleStatus');
    const btnStart = document.getElementById('btnStartMigrate');
    const btnRescan = document.getElementById('btnMigrateRescan');
    const btnBack = document.getElementById('btnMigrateBack');
    const btnClose = document.getElementById('btnMigrateClose');
    const btnHeaderClose = document.getElementById('btnMigrateHeaderClose');

    isMigrating = true;

    content.style.display = 'none';
    consoleBox.style.display = 'block';
    if (btnStart) btnStart.style.display = 'none';
    if (btnRescan) btnRescan.style.display = 'none';
    if (btnBack) btnBack.style.display = 'none';
    if (btnClose) {
        btnClose.disabled = true;
        btnClose.style.opacity = '0.5';
        btnClose.style.cursor = 'not-allowed';
    }
    if (btnHeaderClose) {
        btnHeaderClose.disabled = true;
        btnHeaderClose.style.opacity = '0.5';
        btnHeaderClose.style.cursor = 'not-allowed';
    }

    statusEl.innerHTML = '<span style="display:inline-block;width:14px;height:14px;border:2px solid #cbd5e1;border-top-color:#0284c7;border-radius:50%;animation:spin 0.8s linear infinite;margin-right:6px;vertical-align:middle;"></span> Sedang mengimpor database...';
    consoleEl.textContent = 'Memulai migrasi database...\nDatabase yang akan dimigrasikan: ' + checked.join(', ') + '\n';
    if (currentMigrateFrom) {
        consoleEl.textContent += 'Sumber Kustom: ' + currentMigrateFrom + '\n';
    }
    consoleEl.textContent += '\n';

    const formData = new FormData();
    checked.forEach(function(db) { formData.append('databases[]', db); });
    if (overwrite) formData.append('overwrite', '1');
    if (currentMigrateFrom) formData.append('from', currentMigrateFrom);

    const onFinished = function() {
        isMigrating = false;
        if (btnClose) {
            btnClose.disabled = false;
            btnClose.style.opacity = '1';
            btnClose.style.cursor = 'pointer';
        }
        if (btnHeaderClose) {
            btnHeaderClose.disabled = false;
            btnHeaderClose.style.opacity = '1';
            btnHeaderClose.style.cursor = 'pointer';
        }
        if (btnBack) {
            btnBack.style.display = 'inline-block';
        }
        if (btnRescan) {
            btnRescan.style.display = 'none';
        }
    };

    fetch('?action=migrate_import', {
        method: 'POST',
        body: formData,
    })
    .then(function(res) { return res.json(); })
    .then(function(res) {
        onFinished();
        if (res.ok) {
            consoleEl.textContent += res.output || '(Tidak ada output dari proses)';
            statusEl.textContent = 'Migrasi Selesai';
            showToast('Proses migrasi database telah selesai!');
        } else {
            consoleEl.textContent += '\nError: Gagal menjalankan impor.';
            statusEl.textContent = 'Migrasi Gagal';
        }
        consoleEl.scrollTop = consoleEl.scrollHeight;
    })
    .catch(function(err) {
        onFinished();
        consoleEl.textContent += '\nError koneksi: ' + err.message;
        statusEl.textContent = 'Terjadi Kesalahan';
    });
}
