<!-- Extensions Modal -->
<div class="modal-backdrop" id="extModal" onclick="closeModal(event, 'extModal')">
    <div class="modal-box" style="width: 580px;">
        <h3 class="modal-title">Loaded PHP Extensions (<?= count($extensions) ?>)</h3>
        <div class="modal-body" style="max-height: 280px; overflow-y: auto;">
            <div style="display: grid; grid-template-columns: repeat(3, 1fr); gap: 6px; font-size: 13px; font-family: monospace;">
                <?php foreach ($extensions as $ext): ?>
                    <span style="background:#f1f5f9;padding:3px 6px;border-radius:4px;"><?= htmlspecialchars($ext) ?></span>
                <?php endforeach; ?>
            </div>
        </div>
        <button class="btn-modal-close" onclick="document.getElementById('extModal').classList.remove('show')">Close</button>
    </div>
</div>
