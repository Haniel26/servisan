<!-- PHP Version Manager Modal -->
<div class="modal-backdrop" id="phpModal" onclick="closePhpModalBackdrop(event)">
    <div class="modal-box" style="width: 720px; max-width: 95vw; display: flex; flex-direction: column; max-height: 90vh;">
        <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 12px;">
            <div style="display: flex; align-items: center; gap: 8px;">
                <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" style="color: #0284c7;">
                    <polyline points="16 18 22 12 16 6"></polyline>
                    <polyline points="8 6 2 12 8 18"></polyline>
                </svg>
                <h3 class="modal-title" style="margin: 0;">PHP Version Manager & Switcher</h3>
            </div>
            <div style="display: flex; gap: 6px;">
                <button type="button" onclick="loadPhpVersions()" style="background:#f1f5f9;border:1px solid #cbd5e1;padding:4px 10px;border-radius:4px;font-size:12px;cursor:pointer;color:#334155;">Refresh</button>
                <button type="button" onclick="closePhpModal()" style="background:#f1f5f9;border:1px solid #cbd5e1;padding:4px 10px;border-radius:4px;font-size:12px;cursor:pointer;color:#64748b;">✕</button>
            </div>
        </div>

        <div class="modal-body" style="overflow-y: auto; max-height: calc(90vh - 120px); padding-right: 4px;">
            <!-- Security Warning Box -->
            <div id="phpSecurityBanner" style="display: none; background: #fffbeb; border: 1px solid #fde68a; border-radius: 6px; padding: 12px 14px; margin-bottom: 14px;">
                <div style="display: flex; gap: 10px; align-items: flex-start;">
                    <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="#d97706" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" style="flex-shrink: 0; margin-top: 1px;">
                        <path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"></path>
                        <line x1="12" y1="8" x2="12" y2="12"></line>
                        <line x1="12" y1="16" x2="12.01" y2="16"></line>
                    </svg>
                    <div style="flex: 1;">
                        <div style="font-size: 13px; font-weight: 600; color: #92400e; margin-bottom: 2px;">Security Advisory: Critical RCE Vulnerabilities</div>
                        <div style="font-size: 12px; color: #b45309; line-height: 1.4;" id="phpSecurityMessage">
                            PHP aktif saat ini rentan terhadap CVE-2026-109618 dan CVE-2026-109619. Upgrade ke PHP 8.3.36 sangat disarankan.
                        </div>
                        <div style="margin-top: 8px;" id="phpSecurityAction">
                            <button type="button" id="btnSecurityUpgrade" onclick="installPhpVersion('8.3', true, this)" style="background: #d97706; color: #ffffff; border: none; padding: 6px 14px; border-radius: 4px; font-size: 11.5px; font-weight: 600; cursor: pointer; display: inline-flex; align-items: center; gap: 6px; transition: all 0.15s ease;">
                                <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"></path></svg>
                                <span>1-Click Upgrade ke PHP 8.3.36 (Secured)</span>
                            </button>
                        </div>
                    </div>
                </div>
            </div>

            <!-- Live Action Console (prominently shown during download/switch) -->
            <div id="phpConsoleBox" style="display: none; margin-bottom: 16px; background: #0f172a; border-radius: 6px; border: 1px solid #1e293b; overflow: hidden; box-shadow: 0 4px 6px -1px rgba(0,0,0,0.1);">
                <div style="font-size: 12px; font-weight: 600; color: #e2e8f0; background: #1e293b; padding: 7px 12px; display: flex; justify-content: space-between; align-items: center;">
                    <span id="phpConsoleStatus" style="display: flex; align-items: center; gap: 6px;">Sedang memproses...</span>
                    <span id="phpConsoleTimer" style="font-family: monospace; font-size: 11px; color: #94a3b8; font-weight: normal;">00:00</span>
                </div>
                <!-- Real-time Progress Bar -->
                <div id="phpProgressContainer" style="display: none; height: 4px; background: #334155; width: 100%; position: relative;">
                    <div id="phpProgressBar" style="height: 100%; background: #0284c7; width: 0%; transition: width 0.25s ease;"></div>
                </div>
                <div id="phpConsole" style="color: #38bdf8; padding: 12px 14px; font-family: Consolas, 'Courier New', monospace; font-size: 12px; line-height: 1.5; height: 140px; overflow-y: auto; white-space: pre-wrap; word-break: break-all;"></div>
            </div>

            <!-- Installed PHP Versions -->
            <div style="margin-bottom: 18px;">
                <div style="font-size: 13px; font-weight: 600; color: #334155; margin-bottom: 8px; display: flex; justify-content: space-between; align-items: center;">
                    <span>Versi PHP Terpasang (Installed Runtimes):</span>
                    <span id="phpInstalledCount" style="font-size: 12px; color: #64748b; font-weight: normal;">Memuat...</span>
                </div>
                <div id="phpInstalledList">
                    <div style="text-align: center; padding: 20px; color: #64748b; font-size: 13px;">Memuat daftar versi PHP...</div>
                </div>
            </div>

            <!-- Available Official Releases (1-Click Download) -->
            <div>
                <div style="font-size: 13px; font-weight: 600; color: #334155; margin-bottom: 8px; display: flex; justify-content: space-between; align-items: center;">
                    <span>Unduh Build Resmi windows.php.net (1-Click Install):</span>
                    <span style="font-size: 11.5px; color: #64748b; font-weight: normal;">Thread-Safe x64 Builds</span>
                </div>
                <div id="phpAvailableList">
                    <div style="text-align: center; padding: 20px; color: #64748b; font-size: 13px;">Memuat opsi rilis resmi...</div>
                </div>
            </div>
        </div>

        <div style="display: flex; justify-content: flex-end; border-top: 1px solid #e2e8f0; padding-top: 12px; margin-top: 12px;">
            <button type="button" class="btn-modal-close" style="margin-top:0;" onclick="closePhpModal()">Tutup</button>
        </div>
    </div>
</div>
