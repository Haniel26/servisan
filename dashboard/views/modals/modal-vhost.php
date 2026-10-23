<!-- Virtual Hosts & Projects Modal -->
<div class="modal-backdrop" id="vhostModal" onclick="closeVhostModalBackdrop(event)">
    <div class="modal-box" style="width: 740px; max-width: 95vw; display: flex; flex-direction: column; max-height: 90vh;">
        <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 12px;">
            <div style="display: flex; align-items: center; gap: 8px;">
                <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" style="color: #0284c7;">
                    <path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71"></path>
                    <path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71"></path>
                </svg>
                <h3 class="modal-title" style="margin: 0;">Virtual Hosts & External Projects</h3>
            </div>
            <div style="display: flex; gap: 6px;">
                <button type="button" onclick="loadVhosts()" style="background:#f1f5f9;border:1px solid #cbd5e1;padding:4px 10px;border-radius:4px;font-size:12px;cursor:pointer;color:#334155;">Refresh</button>
                <button type="button" onclick="closeVhostModal()" style="background:#f1f5f9;border:1px solid #cbd5e1;padding:4px 10px;border-radius:4px;font-size:12px;cursor:pointer;color:#64748b;">✕</button>
            </div>
        </div>

        <div class="modal-body" style="overflow-y: auto; max-height: calc(90vh - 120px); padding-right: 4px;">
            <!-- Add New Project Section -->
            <div style="background: #f8fafc; border: 1px solid #e2e8f0; border-radius: 6px; padding: 14px 16px; margin-bottom: 16px;">
                <div style="font-size: 13px; font-weight: 600; color: #0f172a; margin-bottom: 8px; display: flex; align-items: center; gap: 6px;">
                    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" style="color: #0284c7;">
                        <line x1="12" y1="5" x2="12" y2="19"></line>
                        <line x1="5" y1="12" x2="19" y2="12"></line>
                    </svg>
                    <span>Hubungkan Project di Luar Folder www</span>
                </div>
                <div style="font-size: 12px; color: #64748b; margin-bottom: 10px;">
                    Pilih folder project di hard drive Anda (misal project Laravel atau Laragon). Servisan akan membuat junction di <code>www/</code> dan mendaftarkan domain <code>*.test</code> secara otomatis.
                </div>

                <div style="display: flex; gap: 6px; margin-bottom: 10px;">
                    <input type="text" id="vhostInputPath" placeholder="Contoh: D:\Project\my-app atau D:\laragon\www\toko-online" style="flex: 1; padding: 7px 10px; border: 1px solid #cbd5e1; border-radius: 4px; font-size: 12px; font-family: Consolas, monospace;" oninput="onVhostPathInput()">
                    <button type="button" id="btnVhostDetect" onclick="detectVhost()" style="background: #0284c7; color: #ffffff; border: none; padding: 7px 14px; border-radius: 4px; font-size: 12px; font-weight: 500; cursor: pointer; white-space: nowrap; display: inline-flex; align-items: center; gap: 6px;">
                        <span>Periksa Folder</span>
                    </button>
                </div>

                <!-- Detection Details Preview (appears when valid path detected) -->
                <div id="vhostDetectPreview" style="display: none; background: #ffffff; border: 1px solid #cbd5e1; border-radius: 6px; padding: 12px; margin-top: 10px;">
                    <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 10px; margin-bottom: 10px;">
                        <div>
                            <label style="font-size: 11px; font-weight: 600; color: #475569; display: block; margin-bottom: 4px;">NAMA ALIAS PROJECT (FOLDER DI WWW):</label>
                            <input type="text" id="vhostInputName" style="width: 100%; box-sizing: border-box; padding: 6px 8px; border: 1px solid #cbd5e1; border-radius: 4px; font-size: 12px; font-family: Consolas, monospace;">
                        </div>
                        <div>
                            <label style="font-size: 11px; font-weight: 600; color: #475569; display: block; margin-bottom: 4px;">DOMAIN LOKAL (*.TEST):</label>
                            <input type="text" id="vhostInputDomain" style="width: 100%; box-sizing: border-box; padding: 6px 8px; border: 1px solid #cbd5e1; border-radius: 4px; font-size: 12px; font-family: Consolas, monospace;">
                        </div>
                    </div>

                    <div style="background: #f1f5f9; border-radius: 4px; padding: 8px 10px; font-size: 11.5px; color: #334155; margin-bottom: 10px; display: flex; align-items: center; justify-content: space-between;">
                        <div>
                            <span style="color: #64748b;">Framework:</span> <strong id="vhostPreviewFramework">-</strong>
                            <span style="color: #94a3b8; margin: 0 6px;">&bull;</span>
                            <span style="color: #64748b;">Document Root:</span> <code id="vhostPreviewDocRoot" style="font-size: 11px;">-</code>
                        </div>
                        <span id="vhostPreviewBadge" class="badge-source servisan">Auto-Detected</span>
                    </div>

                    <div style="display: flex; justify-content: flex-end; gap: 8px;">
                        <button type="button" onclick="cancelVhostDetect()" style="background: #f1f5f9; border: 1px solid #cbd5e1; padding: 6px 12px; border-radius: 4px; font-size: 12px; cursor: pointer; color: #475569;">Batal</button>
                        <button type="button" id="btnSubmitAddVhost" onclick="submitAddVhost()" style="background: #0284c7; color: #ffffff; border: none; padding: 6px 16px; border-radius: 4px; font-size: 12px; font-weight: 600; cursor: pointer; display: inline-flex; align-items: center; gap: 6px;">
                            <span>Hubungkan Project & Aktifkan VHost</span>
                        </button>
                    </div>
                </div>
            </div>

            <!-- Active Registered VHosts List -->
            <div>
                <div style="font-size: 13px; font-weight: 600; color: #334155; margin-bottom: 8px; display: flex; justify-content: space-between; align-items: center;">
                    <span>Project & Virtual Host Terhubung:</span>
                    <span id="vhostCount" style="font-size: 12px; color: #64748b; font-weight: normal;">Memuat...</span>
                </div>
                <div id="vhostList">
                    <div style="text-align: center; padding: 20px; color: #64748b; font-size: 13px;">Memuat daftar project...</div>
                </div>
            </div>
        </div>

        <div style="display: flex; justify-content: space-between; align-items: center; border-top: 1px solid #e2e8f0; padding-top: 12px; margin-top: 12px;">
            <div style="font-size: 11.5px; color: #64748b; display: flex; align-items: center; gap: 6px;">
                <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"></circle><line x1="12" y1="16" x2="12" y2="12"></line><line x1="12" y1="8" x2="12.01" y2="8"></line></svg>
                <span>Domain *.test otomatis sinkron ke <code>C:\Windows\System32\drivers\etc\hosts</code></span>
            </div>
            <button type="button" class="btn-modal-close" style="margin-top:0;" onclick="closeVhostModal()">Tutup</button>
        </div>
    </div>
</div>
