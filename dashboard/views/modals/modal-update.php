<!-- Servisan Update Modal -->
<div class="modal-backdrop" id="updateModal" onclick="closeModal(event, 'updateModal')">
    <div class="modal-box" style="width: 580px; max-width: 95vw;">
        <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 14px;">
            <div style="display: flex; align-items: center; gap: 10px;">
                <div style="width: 32px; height: 32px; border-radius: 6px; background: #e0f2fe; display: flex; align-items: center; justify-content: center; color: #0284c7;">
                    <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                        <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path>
                        <polyline points="7 10 12 15 17 10"></polyline>
                        <line x1="12" y1="15" x2="12" y2="3"></line>
                    </svg>
                </div>
                <div>
                    <h3 class="modal-title" style="margin: 0;">Servisan Update Available</h3>
                    <div style="font-size: 12px; color: #64748b; margin-top: 2px;">Versi baru telah dirilis di GitHub</div>
                </div>
            </div>
            <button type="button" onclick="document.getElementById('updateModal').classList.remove('show')" style="background:#f1f5f9;border:1px solid #cbd5e1;padding:4px 10px;border-radius:4px;font-size:12px;cursor:pointer;color:#64748b;">✕</button>
        </div>

        <div class="modal-body" style="padding: 0;">
            <div style="background: #f8fafc; border: 1px solid #e2e8f0; border-radius: 6px; padding: 12px 16px; margin-bottom: 14px; display: flex; justify-content: space-between; align-items: center;">
                <div>
                    <div style="font-size: 11px; text-transform: uppercase; letter-spacing: 0.5px; color: #64748b; font-weight: 600;">Current Version</div>
                    <div style="font-size: 15px; font-weight: 700; color: #334155;">v<?= htmlspecialchars($servisanVersion) ?></div>
                </div>
                <div style="color: #94a3b8; font-size: 20px;">→</div>
                <div>
                    <div style="font-size: 11px; text-transform: uppercase; letter-spacing: 0.5px; color: #0284c7; font-weight: 600;">New Version</div>
                    <div id="updateLatestTag" style="font-size: 15px; font-weight: 700; color: #0284c7;">v...</div>
                </div>
                <div style="text-align: right;">
                    <div style="font-size: 11px; text-transform: uppercase; letter-spacing: 0.5px; color: #64748b; font-weight: 600;">Released</div>
                    <div id="updateDate" style="font-size: 13px; color: #475569; font-weight: 500;">...</div>
                </div>
            </div>

            <div style="margin-bottom: 14px;">
                <div style="font-size: 12.5px; font-weight: 600; color: #334155; margin-bottom: 6px;">Release Notes & Changelog:</div>
                <div id="updateChangelog" style="background: #ffffff; border: 1px solid #e2e8f0; border-radius: 6px; padding: 12px 14px; max-height: 180px; overflow-y: auto; font-size: 12.5px; line-height: 1.6; color: #334155; white-space: pre-wrap; word-break: break-word; font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;">Loading release notes...</div>
            </div>

            <div style="font-size: 12.5px; font-weight: 600; color: #334155; margin-bottom: 8px;">Direct 1-Click Downloads:</div>
            <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 10px; margin-bottom: 14px;">
                <a id="btnDownloadInstaller" href="#" target="_blank" style="display: flex; flex-direction: column; padding: 10px 12px; background: #0284c7; color: #ffffff; text-decoration: none; border-radius: 6px; transition: background 0.2s;">
                    <span style="font-size: 13px; font-weight: 600; display: flex; align-items: center; gap: 6px;">
                        <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path><polyline points="7 10 12 15 17 10"></polyline><line x1="12" y1="15" x2="12" y2="3"></line></svg>
                        Windows Installer
                    </span>
                    <span style="font-size: 11px; opacity: 0.85; margin-top: 2px;">Standard Setup (.exe)</span>
                </a>
                <a id="btnDownloadPortable" href="#" target="_blank" style="display: flex; flex-direction: column; padding: 10px 12px; background: #f1f5f9; color: #0f172a; text-decoration: none; border: 1px solid #cbd5e1; border-radius: 6px; transition: background 0.2s;">
                    <span style="font-size: 13px; font-weight: 600; display: flex; align-items: center; gap: 6px;">
                        <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path><polyline points="7 10 12 15 17 10"></polyline><line x1="12" y1="15" x2="12" y2="3"></line></svg>
                        Portable Archive
                    </span>
                    <span style="font-size: 11px; color: #64748b; margin-top: 2px;">Standalone Zip (.zip)</span>
                </a>
            </div>
        </div>

        <div style="display: flex; justify-content: space-between; align-items: center; border-top: 1px solid #e2e8f0; padding-top: 12px;">
            <a id="btnViewRelease" href="https://github.com/Haniel26/servisan/releases/latest" target="_blank" style="font-size: 12.5px; color: #0284c7; text-decoration: none; display: flex; align-items: center; gap: 4px;">
                <span>View on GitHub Releases</span>
                <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"></path><polyline points="15 3 21 3 21 9"></polyline><line x1="10" y1="14" x2="21" y2="3"></line></svg>
            </a>
            <button class="btn-modal-close" style="margin-top:0;" onclick="document.getElementById('updateModal').classList.remove('show')">Close</button>
        </div>
    </div>
</div>
