<!-- Database Migration Modal -->
<div class="modal-backdrop" id="migrateModal" onclick="closeModal(event, 'migrateModal')">
    <div class="modal-box" style="width: 660px; max-width: 95vw; max-height: 85vh; display: flex; flex-direction: column;">
        <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 14px;">
            <div style="display: flex; align-items: center; gap: 10px;">
                <div style="width: 32px; height: 32px; border-radius: 6px; background: #e0f2fe; display: flex; align-items: center; justify-content: center; color: #0284c7;">
                    <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                        <path d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z"></path>
                        <polyline points="3.27 6.96 12 12.01 20.73 6.96"></polyline>
                        <line x1="12" y1="22.08" x2="12" y2="12"></line>
                    </svg>
                </div>
                <div>
                    <h3 class="modal-title" style="margin: 0;">Database Migration</h3>
                    <div style="font-size: 12px; color: #64748b; margin-top: 2px;">Impor database MySQL dari Laragon, XAMPP, atau WAMP ke Servisan</div>
                </div>
            </div>
            <button type="button" id="btnMigrateHeaderClose" onclick="closeMigrateModal()" style="background:#f1f5f9;border:1px solid #cbd5e1;padding:4px 10px;border-radius:4px;font-size:12px;cursor:pointer;color:#64748b;">✕</button>
        </div>

        <div class="modal-body" style="padding: 0; overflow-y: auto; flex: 1;" id="migrateModalBody">
            <div id="migrateLoading" style="text-align: center; padding: 32px 16px; color: #64748b; font-size: 13px;">
                <div style="display: inline-block; width: 24px; height: 24px; border: 2px solid #cbd5e1; border-top-color: #0284c7; border-radius: 50%; animation: spin 0.8s linear infinite; margin-bottom: 8px;"></div>
                <div>Memindai instalasi Laragon, XAMPP, dan WAMP...</div>
            </div>

            <div id="migrateContent" style="display: none;">
                <!-- Manual folder selection bar -->
                <div style="background: #f8fafc; border: 1px solid #e2e8f0; border-radius: 6px; padding: 10px 14px; margin-bottom: 14px;">
                    <div style="font-size: 12px; font-weight: 600; color: #334155; margin-bottom: 6px; display: flex; align-items: center; justify-content: space-between;">
                        <span style="display: flex; align-items: center; gap: 6px;">
                            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" style="color: #0284c7;"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"></path></svg>
                            <span>Pilih Folder Manual / Custom Source:</span>
                        </span>
                        <a href="javascript:resetToAutoScan()" id="btnResetAutoScan" style="font-size: 11.5px; color: #0284c7; text-decoration: none; display: none;">← Kembali ke Auto-Detect</a>
                    </div>
                    <div style="display: flex; gap: 6px;">
                        <input type="text" id="migrateCustomPath" placeholder="Contoh: D:\laragon atau D:\laragon\data\mysql atau E:\xampp" style="flex: 1; padding: 6px 10px; border: 1px solid #cbd5e1; border-radius: 4px; font-size: 12px; font-family: Consolas, monospace;">
                        <button type="button" onclick="scanCustomFolder()" style="background: #0284c7; color: #ffffff; border: none; padding: 6px 12px; border-radius: 4px; font-size: 12px; font-weight: 500; cursor: pointer; white-space: nowrap;">Pindai Folder</button>
                    </div>
                </div>

                <div id="migrateSources"></div>

                <div style="margin-top: 14px; border-top: 1px solid #e2e8f0; padding-top: 12px; display: flex; justify-content: space-between; align-items: center;">
                    <label style="font-size: 12px; color: #475569; display: flex; align-items: center; gap: 6px; cursor: pointer;">
                        <input type="checkbox" id="migrateOverwrite">
                        <span>Overwrite jika database sudah ada di Servisan</span>
                    </label>
                    <span id="migrateSelectedCount" style="font-size: 12px; color: #64748b; font-weight: 500;">0 database dipilih</span>
                </div>
            </div>

            <!-- Console view during and after migration -->
            <div id="migrateConsoleBox" style="display: none;">
                <div style="font-size: 12.5px; font-weight: 600; color: #334155; margin-bottom: 6px; display: flex; justify-content: space-between; align-items: center;">
                    <span id="migrateConsoleStatus">Proses Impor Berjalan...</span>
                </div>
                <div id="migrateConsole" style="background: #0f172a; color: #38bdf8; padding: 14px 16px; border-radius: 6px; font-family: Consolas, 'Courier New', monospace; font-size: 12px; line-height: 1.5; height: 260px; overflow-y: auto; white-space: pre-wrap; word-break: break-all;"></div>
            </div>
        </div>

        <div style="display: flex; justify-content: space-between; align-items: center; border-top: 1px solid #e2e8f0; padding-top: 12px; margin-top: 12px;">
            <div>
                <button type="button" id="btnMigrateRescan" onclick="loadMigrateScan()" style="background:#f1f5f9;border:1px solid #cbd5e1;padding:6px 14px;border-radius:6px;font-size:12.5px;cursor:pointer;color:#334155;">Rescan</button>
                <button type="button" id="btnMigrateBack" onclick="backToMigrationList()" style="background:#f1f5f9;border:1px solid #cbd5e1;padding:6px 14px;border-radius:6px;font-size:12.5px;cursor:pointer;color:#334155;display:none;">← Kembali ke Daftar Database</button>
            </div>
            <div style="display: flex; gap: 8px;">
                <button type="button" id="btnMigrateClose" class="btn-modal-close" style="margin-top:0;" onclick="closeMigrateModal()">Close</button>
                <button type="button" id="btnStartMigrate" onclick="startMigration()" style="background:#0284c7;color:#fff;border:none;padding:6px 16px;border-radius:6px;font-size:12.5px;font-weight:600;cursor:pointer;">Start Migration</button>
            </div>
        </div>
    </div>
</div>
