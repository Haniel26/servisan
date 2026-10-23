<!-- Database Modal -->
<div class="modal-backdrop" id="dbModal" onclick="closeModal(event, 'dbModal')">
    <div class="modal-box">
        <h3 class="modal-title">Database Access</h3>
        <div class="modal-body">
            <p>Koneksi MySQL / MariaDB lokal aktif pada:</p>
            <div class="code-box">
                <span>Host: 127.0.0.1:<?= $mysqlPort ?> | User: root</span>
            </div>
            <p style="font-size:13px;color:#64748b;">phpMyAdmin belum terpasang di folder <code>www/phpmyadmin</code>. Untuk memasangnya secara otomatis, jalankan:</p>
            <div class="code-box">
                <span>servisan phpmyadmin install</span>
            </div>
        </div>
        <button class="btn-modal-close" onclick="document.getElementById('dbModal').classList.remove('show')">Close</button>
    </div>
</div>
