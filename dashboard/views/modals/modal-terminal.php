<!-- Terminal Modal -->
<div class="modal-backdrop" id="terminalModal" onclick="closeModal(event, 'terminalModal')">
    <div class="modal-box">
        <h3 class="modal-title">Servisan Terminal</h3>
        <div class="modal-body">
            <p>Untuk membuka shell dengan <code>php</code>, <code>mysql</code>, dan <code>mysqladmin</code> otomatis tersedia di PATH, jalankan perintah ini di terminal Anda:</p>
            <div class="code-box">
                <span id="cmdText">servisan shell</span>
                <button type="button" onclick="copyCommand()" style="background:#0284c7;color:#fff;border:none;padding:4px 10px;border-radius:4px;cursor:pointer;font-size:12px;">Copy</button>
            </div>
            <p style="font-size:13px;color:#64748b;margin:0;">Atau klik kanan tray icon Servisan di taskbar dan pilih <strong>Open Shell</strong>.</p>
        </div>
        <button class="btn-modal-close" onclick="document.getElementById('terminalModal').classList.remove('show')">Close</button>
    </div>
</div>
