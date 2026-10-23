// ServiSan Dashboard Core Client Logic

function confirmShutdown() {
    const menu = document.getElementById('dropdownMenu');
    const wrapper = document.querySelector('.dock-menu-wrapper');
    if (menu) menu.classList.remove('show');
    if (wrapper) wrapper.classList.remove('open');

    if (!confirm("Matikan semua service ServiSan (PHP & MySQL) dan tutup dashboard?\n\nSemua proses lokal akan dihentikan secara aman.")) {
        return;
    }

    const overlay = document.getElementById('shutdownOverlay');
    if (overlay) {
        overlay.style.display = 'flex';
    }

    fetch('?action=shutdown')
        .then(function() {
            setTimeout(function() {
                window.close();
            }, 500);
        })
        .catch(function() {
            setTimeout(function() {
                window.close();
            }, 500);
        });
}

function toggleMenu(e) {
    e.stopPropagation();
    const menu = document.getElementById('dropdownMenu');
    const wrapper = document.querySelector('.dock-menu-wrapper');
    if (menu) {
        const isOpen = menu.classList.toggle('show');
        if (wrapper) wrapper.classList.toggle('open', isOpen);
    }
}

window.onclick = function() {
    const menu = document.getElementById('dropdownMenu');
    const wrapper = document.querySelector('.dock-menu-wrapper');
    if (menu) menu.classList.remove('show');
    if (wrapper) wrapper.classList.remove('open');
};

function showTerminalModal() {
    document.getElementById('terminalModal').classList.add('show');
}

function showDbModal() {
    document.getElementById('dbModal').classList.add('show');
}

function showExtensions() {
    document.getElementById('extModal').classList.add('show');
}

function closeModal(e, id) {
    if (e.target.id === id) {
        if (id === 'migrateModal' && typeof isMigrating !== 'undefined' && isMigrating) {
            showToast('Proses migrasi sedang berlangsung, mohon tunggu...');
            return;
        }
        document.getElementById(id).classList.remove('show');
    }
}

function copyCommand() {
    navigator.clipboard.writeText('servisan shell').then(function() {
        showToast('Copied "servisan shell" to clipboard!');
    });
}

var docRootPath = window.SERVISAN_DOC_ROOT || '';

function openTerminal() {
    fetch('?action=open_terminal')
        .then(function(res) {
            if (res.ok) {
                showToast('Membuka Servisan Terminal Shell...');
            } else {
                showTerminalModal();
            }
        })
        .catch(function() {
            showTerminalModal();
        });
}

let currentLogType = 'php';

function openLogs() {
    document.getElementById('logsModal').classList.add('show');
    switchLogTab('php');
}

function switchLogTab(type) {
    currentLogType = type;
    document.querySelectorAll('.log-tab').forEach(function(t) {
        t.classList.remove('active');
    });
    const tab = document.getElementById('tab-' + type);
    if (tab) tab.classList.add('active');
    loadLogs(type);
}

function loadLogs(type) {
    const consoleEl = document.getElementById('logConsole');
    consoleEl.textContent = 'Memuat 50 log terakhir (' + type + '.log)...';
    fetch('?action=get_logs&type=' + encodeURIComponent(type))
        .then(function(res) { return res.json(); })
        .then(function(data) {
            if (data.ok) {
                consoleEl.textContent = data.content;
                consoleEl.scrollTop = consoleEl.scrollHeight;
            } else {
                consoleEl.textContent = 'Gagal memuat log.';
            }
        })
        .catch(function(err) {
            consoleEl.textContent = 'Error: ' + err;
        });
}

function openRoot() {
    fetch('?action=open_root')
        .then(function(res) {
            if (res.ok) {
                showToast('Membuka folder www di File Explorer...');
            } else {
                copyRoot();
            }
        })
        .catch(function() {
            copyRoot();
        });
}

function copyRoot() {
    navigator.clipboard.writeText(docRootPath).then(function() {
        showToast('Root directory copied: ' + docRootPath);
    }).catch(function() {
        showToast('Root: ' + docRootPath);
    });
}

function showToast(msg) {
    const toast = document.getElementById('toast');
    if (!toast) return;
    toast.textContent = msg;
    toast.className = 'toast show';
    setTimeout(function() {
        toast.className = 'toast';
    }, 2500);
}

// Update Checker
var CURRENT_VERSION = window.CURRENT_VERSION || window.SERVISAN_VERSION || '0.2.0';
let cachedRelease = null;

function compareSemver(a, b) {
    const pa = String(a).replace(/^v/, '').split('.').map(function(n) { return parseInt(n, 10) || 0; });
    const pb = String(b).replace(/^v/, '').split('.').map(function(n) { return parseInt(n, 10) || 0; });
    for (let i = 0; i < Math.max(pa.length, pb.length); i++) {
        const na = pa[i] || 0;
        const nb = pb[i] || 0;
        if (na > nb) return 1;
        if (na < nb) return -1;
    }
    return 0;
}

function checkForUpdates(isManual) {
    if (CURRENT_VERSION === 'dev') {
        if (isManual) {
            showToast('Versi development: pengecekan update dilewati.');
        }
        return;
    }
    if (isManual) {
        showToast('Memeriksa pembaruan di GitHub...');
    }
    fetch('https://api.github.com/repos/Haniel26/servisan/releases/latest', {
        cache: 'no-cache',
        headers: { 'Accept': 'application/vnd.github.v3+json' }
    })
    .then(function(res) {
        if (!res.ok) throw new Error('GitHub status ' + res.status);
        return res.json();
    })
    .then(function(release) {
        cachedRelease = release;
        const latestTag = release.tag_name || '';
        const hasUpdate = compareSemver(latestTag, CURRENT_VERSION) > 0;

        const badge = document.getElementById('versionBadge');
        const menuCheck = document.getElementById('menuCheckUpdates');

        if (hasUpdate) {
            if (badge) {
                badge.classList.add('has-update');
                badge.innerHTML = '<span class="update-pulse-dot"></span> ' + latestTag + ' Available';
                badge.title = 'Versi baru ' + latestTag + ' tersedia! Klik untuk detail.';
            }
            if (menuCheck) {
                menuCheck.innerHTML = `
                    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" style="color:#2563eb;"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path><polyline points="7 10 12 15 17 10"></polyline><line x1="12" y1="15" x2="12" y2="3"></line></svg>
                    <span>Update Available (${latestTag})</span>
                `;
            }
            if (isManual) {
                showUpdateModal();
            }
        } else {
            if (badge) {
                badge.classList.remove('has-update');
                badge.textContent = 'v' + CURRENT_VERSION;
                badge.title = 'Servisan v' + CURRENT_VERSION + ' - Up to date';
            }
            if (isManual) {
                showToast('Servisan v' + CURRENT_VERSION + ' sudah versi terbaru!');
            }
        }
    })
    .catch(function(err) {
        if (isManual) {
            showToast('Gagal memeriksa update. Cek koneksi internet.');
        }
    });
}

function checkUpdateManual() {
    if (cachedRelease && compareSemver(cachedRelease.tag_name, CURRENT_VERSION) > 0) {
        showUpdateModal();
    } else {
        checkForUpdates(true);
    }
}

function showUpdateModal() {
    if (!cachedRelease) {
        checkForUpdates(true);
        return;
    }

    const latestTag = cachedRelease.tag_name || '';
    document.getElementById('updateLatestTag').textContent = latestTag;

    if (cachedRelease.published_at) {
        const d = new Date(cachedRelease.published_at);
        document.getElementById('updateDate').textContent = d.toISOString().split('T')[0];
    }

    const body = (cachedRelease.body || '').trim();
    document.getElementById('updateChangelog').textContent = body || 'Tidak ada catatan rilis tambahan.';

    let installerUrl = cachedRelease.html_url;
    let portableUrl = cachedRelease.html_url;

    if (Array.isArray(cachedRelease.assets)) {
        for (const asset of cachedRelease.assets) {
            const name = (asset.name || '').toLowerCase();
            if (name.endsWith('.exe')) {
                installerUrl = asset.browser_download_url;
            } else if (name.endsWith('.zip')) {
                portableUrl = asset.browser_download_url;
            }
        }
    }

    document.getElementById('btnDownloadInstaller').href = installerUrl;
    document.getElementById('btnDownloadPortable').href = portableUrl;
    document.getElementById('btnViewRelease').href = cachedRelease.html_url;

    document.getElementById('updateModal').classList.add('show');
}

// Auto-check on load and handle query pages
window.addEventListener('load', function() {
    setTimeout(function() {
        checkForUpdates(false);
    }, 1200);

    if (window.location.search.indexOf('page=migrate') !== -1) {
        if (typeof openMigrateModal === 'function') openMigrateModal();
    } else if (window.location.search.indexOf('page=php') !== -1) {
        if (typeof openPhpModal === 'function') openPhpModal();
    } else if (window.location.search.indexOf('page=vhost') !== -1) {
        if (typeof openVhostModal === 'function') openVhostModal();
    }
});
