<!-- Fullscreen Shutdown Overlay -->
<div id="shutdownOverlay" style="display:none;position:fixed;top:0;left:0;right:0;bottom:0;background:rgba(15,23,42,0.95);backdrop-filter:blur(6px);z-index:99999;align-items:center;justify-content:center;color:#ffffff;text-align:center;padding:24px;">
    <div style="background:#1e293b;border:1px solid #334155;border-radius:12px;padding:36px 32px;max-width:440px;width:100%;box-shadow:0 25px 50px -12px rgba(0,0,0,0.5);">
        <div style="width:56px;height:56px;border-radius:50%;background:rgba(239,68,68,0.12);border:1px solid rgba(239,68,68,0.25);display:flex;align-items:center;justify-content:center;margin:0 auto 20px auto;color:#ef4444;">
            <svg width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round">
                <path d="M18.36 6.64a9 9 0 1 1-12.73 0"></path>
                <line x1="12" y1="2" x2="12" y2="12"></line>
            </svg>
        </div>
        <h2 style="font-size:19px;font-weight:700;color:#f8fafc;margin:0 0 10px 0;">ServiSan Telah Dimatikan</h2>
        <p style="font-size:13.5px;color:#94a3b8;line-height:1.5;margin:0 0 24px 0;">
            Semua service PHP &amp; MySQL serta proses background tray telah dihentikan secara aman.
        </p>
        <div style="display:flex;flex-direction:column;gap:10px;">
            <button type="button" onclick="window.close()" style="background:#ef4444;color:#ffffff;border:none;border-radius:6px;padding:10px 18px;font-size:13.5px;font-weight:600;cursor:pointer;transition:background 0.15s;display:flex;align-items:center;justify-content:center;gap:8px;">
                <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                    <line x1="18" y1="6" x2="6" y2="18"></line>
                    <line x1="6" y1="6" x2="18" y2="18"></line>
                </svg>
                <span>Tutup Tab Browser</span>
            </button>
            <span style="font-size:11.5px;color:#64748b;">(Anda dapat menutup tab ini secara manual jika browser memblokir penutupan otomatis)</span>
        </div>
    </div>
</div>
