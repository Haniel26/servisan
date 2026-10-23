// PHP Version Manager & Switcher Client Logic

let isProcessingPhp = false;

function openPhpModal() {
    document.getElementById('phpModal').classList.add('show');
    loadPhpVersions();
}

function closePhpModal() {
    if (isProcessingPhp) {
        showToast('Proses download/switch PHP sedang berjalan...');
        return;
    }
    document.getElementById('phpModal').classList.remove('show');
}

function closePhpModalBackdrop(e) {
    if (e.target && e.target.id === 'phpModal') {
        closePhpModal();
    }
}

function loadPhpVersions() {
    const installedList = document.getElementById('phpInstalledList');
    const availableList = document.getElementById('phpAvailableList');
    const countSpan = document.getElementById('phpInstalledCount');
    const secBanner = document.getElementById('phpSecurityBanner');
    const secMsg = document.getElementById('phpSecurityMessage');

    installedList.innerHTML = '<div style="text-align:center;padding:16px;color:#64748b;font-size:12.5px;">Memindai runtime PHP yang terpasang...</div>';

    fetch('?action=php_list')
        .then(function(res) { return res.json(); })
        .then(function(data) {
            if (!data.ok) {
                installedList.innerHTML = '<div style="color:#ef4444;padding:10px;font-size:12.5px;">Gagal memuat daftar versi PHP.</div>';
                return;
            }

            const installed = data.installed || [];
            const available = data.available || [];
            countSpan.textContent = installed.length + ' runtime ditemukan';

            // Check security advisory on active version
            const activePhp = installed.find(function(c) { return c.is_active; });
            if (activePhp && activePhp.has_security_warning) {
                secBanner.style.display = 'block';
                secMsg.textContent = activePhp.security_message || 'Versi PHP aktif rentan terhadap CVE-2026-109618/19. Upgrade ke versi aman disarankan.';
            } else {
                secBanner.style.display = 'none';
            }

            // Render installed versions
            if (installed.length === 0) {
                installedList.innerHTML = '<div style="color:#64748b;padding:10px;font-size:12.5px;font-style:italic;">Belum ada versi PHP terpasang.</div>';
            } else {
                let html = '';
                installed.forEach(function(c) {
                    const sourceClass = (c.source || '').toLowerCase();
                    html += `
                        <div class="php-version-card ${c.is_active ? 'is-active' : ''}">
                            <div>
                                <div style="display:flex;align-items:center;gap:8px;">
                                    <strong style="font-size:13.5px;color:#0f172a;">PHP ${escapeHtml(c.version)}</strong>
                                    <span class="badge-source ${sourceClass}">${escapeHtml(c.source)}</span>
                                    ${c.is_active ? '<span class="badge-active-pill"><svg width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round"><polyline points="20 6 9 17 4 12"></polyline></svg> Active</span>' : ''}
                                </div>
                                <div style="font-size:11px;color:#64748b;font-family:monospace;margin-top:3px;">
                                    ${escapeHtml(c.path)}
                                </div>
                                ${c.has_security_warning ? `<div class="badge-warning-pill"><svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"></path></svg> ${escapeHtml(c.security_message || 'Security patch available')}</div>` : ''}
                            </div>
                            <div>
                                ${c.is_active
                                    ? '<span style="font-size:12px;color:#16a34a;font-weight:600;display:inline-flex;align-items:center;gap:4px;"><svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3"><polyline points="20 6 9 17 4 12"></polyline></svg> Sedang Aktif</span>'
                                    : `
                                    <div style="display:flex;align-items:center;gap:6px;">
                                        <button type="button" class="btn-action-sm outline" onclick="switchPhpVersion('${escapeHtml(c.id)}', this)">Switch to This</button>
                                        ${c.source === 'Servisan'
                                            ? `<button type="button" class="btn-action-sm danger" onclick="deletePhpVersion('${escapeHtml(c.id)}', '${escapeHtml(c.version)}', this)" title="Hapus runtime ini dari disk">
                                                <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                                                    <polyline points="3 6 5 6 21 6"></polyline>
                                                    <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"></path>
                                                </svg>
                                               </button>`
                                            : ''
                                        }
                                    </div>
                                    `
                                }
                            </div>
                        </div>
                    `;
                });
                installedList.innerHTML = html;
            }

            // Render available downloads
            let availHtml = '';
            available.forEach(function(a) {
                availHtml += `
                    <div class="php-version-card">
                        <div>
                            <div style="display:flex;align-items:center;gap:8px;">
                                <strong style="font-size:13.5px;color:#0f172a;">${escapeHtml(a.label)}</strong>
                                <span style="font-size:11.5px;color:#0284c7;font-family:monospace;font-weight:600;">v${escapeHtml(a.version)}</span>
                            </div>
                            <div style="font-size:11.5px;color:#475569;margin-top:3px;">
                                ${escapeHtml(a.note)}
                            </div>
                        </div>
                        <div>
                            ${a.is_installed
                                ? '<span style="font-size:12px;color:#64748b;font-weight:500;">Sudah Terpasang</span>'
                                : `<button type="button" class="btn-action-sm" onclick="installPhpVersion('${escapeHtml(a.id)}', true, this)">Download & Switch</button>`
                            }
                        </div>
                    </div>
                `;
            });
            availableList.innerHTML = availHtml;
        })
        .catch(function(err) {
            installedList.innerHTML = `<div style="color:#ef4444;padding:10px;font-size:12.5px;">Error: ${escapeHtml(err.message)}</div>`;
        });
}

function deletePhpVersion(target, version, btnEl) {
    if (isProcessingPhp) return;
    if (!confirm(`Hapus runtime PHP ${version} dari disk?\n\nTindakan ini akan menghapus folder instalasi di bin/php/${target}. Kamu bisa mengunduhnya kembali kapan saja jika diperlukan.`)) {
        return;
    }

    isProcessingPhp = true;
    const consoleBox = document.getElementById('phpConsoleBox');
    const consoleEl = document.getElementById('phpConsole');
    const statusEl = document.getElementById('phpConsoleStatus');

    // Disable all action buttons during operation
    document.querySelectorAll('#phpModal button').forEach(function(b) {
        b.disabled = true;
        b.style.opacity = '0.6';
    });

    if (btnEl) {
        btnEl.innerHTML = '<span class="inline-spinner dark"></span>';
    }

    consoleBox.style.display = 'block';
    consoleBox.scrollIntoView({ behavior: 'smooth', block: 'nearest' });
    statusEl.innerHTML = '<span class="inline-spinner"></span> Menghapus runtime PHP ' + escapeHtml(version) + ' dari disk...';
    consoleEl.textContent = 'Menghapus instalasi PHP ' + target + ' dari bin/php/...\n';

    const formData = new FormData();
    formData.append('target', target);

    fetch('?action=php_delete', {
        method: 'POST',
        body: formData,
    })
    .then(function(res) { return res.json(); })
    .then(function(res) {
        isProcessingPhp = false;
        document.querySelectorAll('#phpModal button').forEach(function(b) {
            b.disabled = false;
            b.style.opacity = '1';
        });
        if (res.ok) {
            consoleEl.textContent += res.output || 'Instalasi berhasil dihapus.\n';
            statusEl.innerHTML = '<svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="#22c55e" stroke-width="3" stroke-linecap="round" stroke-linejoin="round" style="margin-right:4px;"><polyline points="20 6 9 17 4 12"></polyline></svg> Runtime PHP berhasil dihapus!';
            showToast('PHP ' + version + ' berhasil dihapus dari disk.');
            loadPhpVersions();
        } else {
            consoleEl.textContent += '\nError: ' + (res.error || 'Gagal menghapus versi.');
            statusEl.textContent = 'Gagal Menghapus';
        }
    })
    .catch(function(err) {
        isProcessingPhp = false;
        document.querySelectorAll('#phpModal button').forEach(function(b) {
            b.disabled = false;
            b.style.opacity = '1';
        });
        consoleEl.textContent += '\nError koneksi: ' + err.message;
        statusEl.textContent = 'Terjadi Kesalahan';
    });
}

function switchPhpVersion(target, btnEl) {
    if (isProcessingPhp) return;
    isProcessingPhp = true;

    const consoleBox = document.getElementById('phpConsoleBox');
    const consoleEl = document.getElementById('phpConsole');
    const statusEl = document.getElementById('phpConsoleStatus');
    const timerEl = document.getElementById('phpConsoleTimer');
    const progressContainer = document.getElementById('phpProgressContainer');
    const progressBar = document.getElementById('phpProgressBar');

    // Disable all action buttons during operation
    document.querySelectorAll('#phpModal button').forEach(function(b) {
        if (b.classList.contains('btn-action-sm') || b.id === 'btnSecurityUpgrade') {
            b.disabled = true;
            b.style.opacity = '0.6';
        }
    });

    if (btnEl) {
        btnEl.dataset.origHtml = btnEl.innerHTML;
        btnEl.innerHTML = '<span class="inline-spinner dark"></span> Mengganti...';
    }

    consoleBox.style.display = 'block';
    consoleBox.scrollIntoView({ behavior: 'smooth', block: 'nearest' });
    if (progressContainer) progressContainer.style.display = 'block';
    if (progressBar) {
        progressBar.style.width = '20%';
        progressBar.style.background = '#0284c7';
    }

    const startTime = Date.now();
    function getTs() {
        const elapsed = Math.floor((Date.now() - startTime) / 1000);
        const m = String(Math.floor(elapsed / 60)).padStart(2, '0');
        const s = String(elapsed % 60).padStart(2, '0');
        return `[${m}:${s}]`;
    }

    if (timerEl) timerEl.textContent = '00:00';
    statusEl.innerHTML = '<span class="inline-spinner"></span> Mengganti runtime PHP ke: ' + escapeHtml(target) + '...';
    consoleEl.textContent = getTs() + ' Memperbarui konfigurasi ke target: ' + target + '...\n';
    consoleEl.textContent += getTs() + ' Meregenerasi config/generated/php.ini dan OPcache...\n';

    const timerInterval = setInterval(function() {
        const elapsed = Math.floor((Date.now() - startTime) / 1000);
        const m = String(Math.floor(elapsed / 60)).padStart(2, '0');
        const s = String(elapsed % 60).padStart(2, '0');
        if (timerEl) timerEl.textContent = `${m}:${s}`;
        if (progressBar && elapsed >= 1) {
            progressBar.style.width = '75%';
        }
    }, 500);

    const formData = new FormData();
    formData.append('target', target);

    fetch('?action=php_switch', {
        method: 'POST',
        body: formData,
    })
    .then(function(res) { return res.json(); })
    .then(function(res) {
        clearInterval(timerInterval);
        if (res.ok) {
            if (progressBar) {
                progressBar.style.width = '100%';
                progressBar.style.background = '#22c55e';
            }
            consoleEl.textContent += '\n' + (res.output || 'Sukses berganti runtime.');
            consoleEl.textContent += `\n${getTs()} Hot-reload terpicu! Memuat ulang dashboard...`;
            consoleEl.scrollTop = consoleEl.scrollHeight;
            statusEl.innerHTML = '<svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="#22c55e" stroke-width="3" stroke-linecap="round" stroke-linejoin="round" style="margin-right:4px;"><polyline points="20 6 9 17 4 12"></polyline></svg> Sukses! Mereload dashboard...';
            if (btnEl) {
                btnEl.innerHTML = '<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3"><polyline points="20 6 9 17 4 12"></polyline></svg> Aktif!';
            }
            showToast('PHP berhasil diganti ke ' + target + '! Memuat ulang...');
            setTimeout(function() { location.reload(); }, 2200);
        } else {
            isProcessingPhp = false;
            if (progressBar) progressBar.style.background = '#ef4444';
            document.querySelectorAll('#phpModal button').forEach(function(b) {
                b.disabled = false;
                b.style.opacity = '1';
            });
            if (btnEl && btnEl.dataset.origHtml) {
                btnEl.innerHTML = btnEl.dataset.origHtml;
            }
            consoleEl.textContent += `\n${getTs()} Error: ` + (res.error || 'Gagal berganti versi.');
            statusEl.textContent = 'Gagal Berganti Versi';
        }
    })
    .catch(function(err) {
        clearInterval(timerInterval);
        isProcessingPhp = false;
        if (progressBar) progressBar.style.background = '#ef4444';
        document.querySelectorAll('#phpModal button').forEach(function(b) {
            b.disabled = false;
            b.style.opacity = '1';
        });
        if (btnEl && btnEl.dataset.origHtml) {
            btnEl.innerHTML = btnEl.dataset.origHtml;
        }
        consoleEl.textContent += `\n${getTs()} Error koneksi: ` + err.message;
        statusEl.textContent = 'Terjadi Kesalahan';
    });
}

function installPhpVersion(version, andSwitch, btnEl) {
    if (isProcessingPhp) return;
    isProcessingPhp = true;

    const consoleBox = document.getElementById('phpConsoleBox');
    const consoleEl = document.getElementById('phpConsole');
    const statusEl = document.getElementById('phpConsoleStatus');
    const timerEl = document.getElementById('phpConsoleTimer');
    const progressContainer = document.getElementById('phpProgressContainer');
    const progressBar = document.getElementById('phpProgressBar');

    // Disable all action buttons during operation
    document.querySelectorAll('#phpModal button').forEach(function(b) {
        if (b.classList.contains('btn-action-sm') || b.id === 'btnSecurityUpgrade') {
            b.disabled = true;
            b.style.opacity = '0.6';
        }
    });

    if (btnEl) {
        btnEl.dataset.origHtml = btnEl.innerHTML;
        btnEl.innerHTML = '<span class="inline-spinner"></span> Menghubungi CDN...';
    }

    consoleBox.style.display = 'block';
    consoleBox.scrollIntoView({ behavior: 'smooth', block: 'nearest' });
    if (progressContainer) progressContainer.style.display = 'block';
    if (progressBar) {
        progressBar.style.width = '4%';
        progressBar.style.background = '#0284c7';
    }

    const startTime = Date.now();
    let lastReportedPhase = '';
    let lastLoggedBytes = 0;

    function getTs() {
        const elapsed = Math.floor((Date.now() - startTime) / 1000);
        const m = String(Math.floor(elapsed / 60)).padStart(2, '0');
        const s = String(elapsed % 60).padStart(2, '0');
        return `[${m}:${s}]`;
    }

    if (timerEl) timerEl.textContent = '00:00';
    statusEl.innerHTML = '<span class="inline-spinner"></span> Menghubungi windows.php.net (~35MB)...';
    consoleEl.textContent = getTs() + ' Memulai koneksi resmi ke windows.php.net untuk PHP ' + version + '...\n';
    consoleEl.textContent += getTs() + ' Menyiapkan alokasi transfer archive ZIP x64 Thread-Safe...\n';

    // Heartbeat interval for timer & polling live download bytes
    const pollInterval = setInterval(function() {
        const elapsed = Math.floor((Date.now() - startTime) / 1000);
        const m = String(Math.floor(elapsed / 60)).padStart(2, '0');
        const s = String(elapsed % 60).padStart(2, '0');
        if (timerEl) timerEl.textContent = `${m}:${s}`;

        fetch('?action=php_install_status&version=' + encodeURIComponent(version))
            .then(function(r) { return r.json(); })
            .then(function(st) {
                if (!isProcessingPhp) return;
                const bytes = st.bytes || 0;
                const est = st.estimated_total || 36700160;
                const mb = (bytes / (1024 * 1024)).toFixed(1);
                const estMb = (est / (1024 * 1024)).toFixed(1);

                if (st.stage === 'downloading' || bytes > 0) {
                    const pct = Math.min(98, Math.max(6, Math.round((bytes / est) * 100)));
                    if (progressBar) progressBar.style.width = pct + '%';
                    statusEl.innerHTML = `<span class="inline-spinner"></span> Mengunduh PHP ${escapeHtml(version)}: ${mb} MB / ~${estMb} MB (${pct}%)`;
                    if (btnEl) btnEl.innerHTML = `<span class="inline-spinner"></span> ${pct}% (${mb}MB)`;

                    // Log milestone in console every ~4MB so the user continuously sees active data
                    if (bytes - lastLoggedBytes > 4 * 1024 * 1024) {
                        lastLoggedBytes = bytes;
                        consoleEl.textContent += `${getTs()} Mentransfer data: ${mb} MB / ~${estMb} MB (${pct}%)...\n`;
                        consoleEl.scrollTop = consoleEl.scrollHeight;
                    }
                } else if (st.stage === 'extracting') {
                    if (lastReportedPhase !== 'extracting') {
                        lastReportedPhase = 'extracting';
                        if (progressBar) progressBar.style.width = '99%';
                        statusEl.innerHTML = `<span class="inline-spinner"></span> Mengekstrak archive via tar.exe...`;
                        if (btnEl) btnEl.innerHTML = `<span class="inline-spinner"></span> Mengekstrak...`;
                        consoleEl.textContent += `\n${getTs()} Unduhan selesai! Mengekstrak runtime ke bin/php/php-${version} via tar.exe...\n`;
                        consoleEl.scrollTop = consoleEl.scrollHeight;
                    }
                } else if (st.stage === 'configuring') {
                    if (lastReportedPhase !== 'configuring') {
                        lastReportedPhase = 'configuring';
                        statusEl.innerHTML = `<span class="inline-spinner"></span> Menyiapkan Visual C++ DLLs & konfigurasi...`;
                        consoleEl.textContent += `${getTs()} Menyalin Visual C++ Runtime DLLs dan setup ekstensi...\n`;
                        consoleEl.scrollTop = consoleEl.scrollHeight;
                    }
                }
            })
            .catch(function() {});
    }, 600);

    const formData = new FormData();
    formData.append('version', version);
    if (andSwitch) {
        formData.append('switch', '1');
    }

    fetch('?action=php_install', {
        method: 'POST',
        body: formData,
    })
    .then(function(res) { return res.json(); })
    .then(function(res) {
        clearInterval(pollInterval);
        if (res.ok) {
            if (progressBar) {
                progressBar.style.width = '100%';
                progressBar.style.background = '#22c55e';
            }
            consoleEl.textContent += '\n' + (res.output || 'Instalasi selesai!');
            consoleEl.textContent += `\n${getTs()} Sukses! Menyinkronkan server... Dashboard akan dimuat ulang.`;
            consoleEl.scrollTop = consoleEl.scrollHeight;

            statusEl.innerHTML = '<svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="#22c55e" stroke-width="3" stroke-linecap="round" stroke-linejoin="round" style="margin-right:4px;"><polyline points="20 6 9 17 4 12"></polyline></svg> Selesai! Mereload dashboard...';
            if (btnEl) {
                btnEl.innerHTML = '<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3"><polyline points="20 6 9 17 4 12"></polyline></svg> Terpasang!';
            }
            showToast('PHP ' + version + ' berhasil diunduh & dipasang! Memuat ulang...');
            setTimeout(function() { location.reload(); }, 2500);
        } else {
            isProcessingPhp = false;
            if (progressBar) progressBar.style.background = '#ef4444';
            document.querySelectorAll('#phpModal button').forEach(function(b) {
                b.disabled = false;
                b.style.opacity = '1';
            });
            if (btnEl && btnEl.dataset.origHtml) {
                btnEl.innerHTML = btnEl.dataset.origHtml;
            }
            consoleEl.textContent += `\n${getTs()} Error: ` + (res.error || 'Gagal mengunduh runtime.');
            statusEl.textContent = 'Instalasi Gagal';
        }
    })
    .catch(function(err) {
        clearInterval(pollInterval);
        isProcessingPhp = false;
        if (progressBar) progressBar.style.background = '#ef4444';
        document.querySelectorAll('#phpModal button').forEach(function(b) {
            b.disabled = false;
            b.style.opacity = '1';
        });
        if (btnEl && btnEl.dataset.origHtml) {
            btnEl.innerHTML = btnEl.dataset.origHtml;
        }
        consoleEl.textContent += `\n${getTs()} Error koneksi: ` + err.message;
        statusEl.textContent = 'Terjadi Kesalahan';
    });
}
