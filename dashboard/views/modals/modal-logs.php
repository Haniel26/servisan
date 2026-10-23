<!-- Server Logs Modal (Last 50 Lines) -->
<div class="modal-backdrop" id="logsModal" onclick="closeModal(event, 'logsModal')">
    <div class="modal-box" style="width: 760px; max-width: 95vw; display: flex; flex-direction: column; max-height: 85vh;">
        <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 12px;">
            <div style="display: flex; align-items: center; gap: 8px;">
                <h3 class="modal-title" style="margin: 0;">Server Logs</h3>
                <span style="font-size: 12px; color: #64748b; font-weight: normal;">(50 baris terakhir)</span>
            </div>
            <div style="display: flex; gap: 6px;">
                <button type="button" onclick="fetch('?action=open_logs'); showToast('Membuka folder logs di Explorer...');" style="background:#f1f5f9;border:1px solid #cbd5e1;padding:4px 10px;border-radius:4px;font-size:12px;cursor:pointer;color:#334155;">Open Folder</button>
                <button type="button" onclick="loadLogs(currentLogType)" style="background:#0284c7;color:#fff;border:none;padding:4px 10px;border-radius:4px;font-size:12px;cursor:pointer;">Refresh</button>
                <button type="button" onclick="document.getElementById('logsModal').classList.remove('show')" style="background:#f1f5f9;border:1px solid #cbd5e1;padding:4px 10px;border-radius:4px;font-size:12px;cursor:pointer;color:#64748b;">✕</button>
            </div>
        </div>

        <!-- Log Tabs -->
        <div style="display: flex; gap: 4px; border-bottom: 1px solid #e2e8f0; margin-bottom: 10px;">
            <button type="button" class="log-tab active" id="tab-php" onclick="switchLogTab('php')">PHP (php.log)</button>
            <button type="button" class="log-tab" id="tab-mysql" onclick="switchLogTab('mysql')">MySQL (mysql.log)</button>
            <button type="button" class="log-tab" id="tab-servisan" onclick="switchLogTab('servisan')">Servisan (servisan.log)</button>
        </div>

        <!-- Log Output Console -->
        <div id="logConsole" style="background: #0f172a; color: #38bdf8; padding: 14px 16px; border-radius: 6px; font-family: Consolas, 'Courier New', monospace; font-size: 12.5px; line-height: 1.5; height: 320px; overflow-y: auto; white-space: pre-wrap; word-break: break-all;">Loading logs...</div>
    </div>
</div>
