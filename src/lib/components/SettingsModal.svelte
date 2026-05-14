<script lang="ts">
  import { getSettings, updateSettings } from "$lib/services/settings";
  import type { AppSettings, StorageMode } from "$lib/types/workbench";

  let { onclose } = $props<{ onclose: () => void }>();

  let storageMode = $state<StorageMode>("local");
  let endpoint = $state("");
  let bucket = $state("");
  let accessKey = $state("");
  let secretKey = $state("");
  let loading = $state(true);
  let saving = $state(false);
  let error = $state("");
  let message = $state("");

  async function fetchSettings() {
    loading = true;
    error = "";
    try {
      const settings = await getSettings();
      storageMode = settings.storageMode;
      if (settings.minio) {
        endpoint = settings.minio.endpoint;
        bucket = settings.minio.bucket;
        accessKey = settings.minio.accessKey;
        secretKey = settings.minio.secretKey;
      }
    } catch (err) {
      error = `加载设置失败：${String(err)}`;
    } finally {
      loading = false;
    }
  }

  async function handleSave() {
    saving = true;
    error = "";
    message = "";

    const settings: AppSettings = {
      storageMode,
      minio:
        storageMode === "minio"
          ? {
              endpoint: endpoint.trim(),
              bucket: bucket.trim(),
              accessKey: accessKey.trim(),
              secretKey: secretKey.trim(),
            }
          : null,
    };

    try {
      const response = await updateSettings(settings);
      message = response;
    } catch (err) {
      error = `保存设置失败：${String(err)}`;
    } finally {
      saving = false;
    }
  }

  function handleModeChange(mode: StorageMode) {
    storageMode = mode;
    message = "";
  }

  $effect(() => {
    fetchSettings();
  });
</script>

<div
  class="modal-overlay"
  onclick={onclose}
  role="presentation"
  onkeydown={(e) => e.key === "Escape" && onclose()}
>
  <div
    class="modal-content"
    onclick={(e) => e.stopPropagation()}
    role="dialog"
    aria-modal="true"
    tabindex="-1"
  >
    <div class="modal-header">
      <h3>设置</h3>
      <button type="button" class="close-btn" onclick={onclose}>&times;</button>
    </div>

    <div class="modal-body">
      {#if loading}
        <div class="loading">加载中...</div>
      {:else}
        {#if error}
          <div class="error-message">{error}</div>
        {/if}

        {#if message}
          <div class="success-message">{message}</div>
        {/if}

        <div class="form-group">
          <label>存储模式</label>
          <div class="mode-selector">
            <button
              type="button"
              class="mode-option"
              class:active={storageMode === "local"}
              onclick={() => handleModeChange("local")}
            >
              <svg viewBox="0 0 24 24" width="20" height="20" stroke="currentColor" stroke-width="2" fill="none" stroke-linecap="round" stroke-linejoin="round">
                <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"></path>
              </svg>
              <div>
                <strong>本地存储</strong>
                <span>笔记和图片保存在应用数据目录</span>
              </div>
            </button>
            <button
              type="button"
              class="mode-option"
              class:active={storageMode === "minio"}
              onclick={() => handleModeChange("minio")}
            >
              <svg viewBox="0 0 24 24" width="20" height="20" stroke="currentColor" stroke-width="2" fill="none" stroke-linecap="round" stroke-linejoin="round">
                <ellipse cx="12" cy="5" rx="9" ry="3"></ellipse>
                <path d="M21 12c0 1.66-4 3-9 3s-9-1.34-9-3"></path>
                <path d="M3 5v14c0 1.66 4 3 9 3s9-1.34 9-3V5"></path>
                <path d="M3 12v7c0 1.66 4 3 9 3s9-1.34 9-3v-7"></path>
              </svg>
              <div>
                <strong>MinIO 服务器</strong>
                <span>S3 兼容对象存储</span>
              </div>
            </button>
          </div>
        </div>

        {#if storageMode === "minio"}
          <div class="minio-config">
            <div class="form-group">
              <label for="minio-endpoint">服务地址 (Endpoint)</label>
              <input
                id="minio-endpoint"
                type="text"
                bind:value={endpoint}
                placeholder="http://192.168.1.100:9000"
              />
            </div>

            <div class="form-group">
              <label for="minio-bucket">Bucket 名称</label>
              <input
                id="minio-bucket"
                type="text"
                bind:value={bucket}
                placeholder="desk-tools"
              />
            </div>

            <div class="form-group">
              <label for="minio-ak">Access Key</label>
              <input
                id="minio-ak"
                type="password"
                bind:value={accessKey}
                placeholder="输入 Access Key"
              />
            </div>

            <div class="form-group">
              <label for="minio-sk">Secret Key</label>
              <input
                id="minio-sk"
                type="password"
                bind:value={secretKey}
                placeholder="输入 Secret Key"
              />
            </div>
          </div>
        {/if}
      {/if}
    </div>

    <div class="modal-footer">
      <button type="button" class="text-button" onclick={onclose}>取消</button>
      <button
        type="button"
        class="primary-btn"
        onclick={handleSave}
        disabled={saving || loading}
      >
        {saving ? "保存中..." : "保存设置"}
      </button>
    </div>
  </div>
</div>

<style>
  .modal-overlay {
    position: fixed;
    top: 0;
    left: 0;
    width: 100vw;
    height: 100vh;
    background: rgba(15, 23, 42, 0.7);
    backdrop-filter: blur(4px);
    display: flex;
    justify-content: center;
    align-items: center;
    z-index: 1000;
  }

  :global(body[data-theme="light"]) .modal-overlay {
    background: rgba(255, 255, 255, 0.7);
  }

  .modal-content {
    background: var(--bg-modal);
    border: 1px solid var(--border-input);
    border-radius: 16px;
    width: 100%;
    max-width: 460px;
    box-shadow: 0 20px 40px var(--shadow-panel);
    overflow: hidden;
    animation: modal-fade-in 0.2s ease-out;
  }

  @keyframes modal-fade-in {
    from {
      opacity: 0;
      transform: translateY(10px) scale(0.98);
    }
    to {
      opacity: 1;
      transform: translateY(0) scale(1);
    }
  }

  .modal-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 16px 20px;
    border-bottom: 1px solid var(--border-light);
  }

  .modal-header h3 {
    margin: 0;
    font-size: 1.1rem;
    color: var(--text-main);
  }

  .close-btn {
    background: transparent;
    border: none;
    color: var(--text-muted-alt);
    font-size: 1.5rem;
    line-height: 1;
    padding: 0 4px;
    cursor: pointer;
    min-height: auto;
    width: auto;
  }

  .close-btn:hover {
    color: var(--text-main);
  }

  .modal-body {
    padding: 20px;
    display: flex;
    flex-direction: column;
    gap: 16px;
    max-height: 60vh;
    overflow-y: auto;
  }

  .modal-footer {
    display: flex;
    justify-content: flex-end;
    gap: 12px;
    padding: 16px 20px;
    border-top: 1px solid var(--border-light);
    background: var(--bg-panel-light);
  }

  .loading {
    text-align: center;
    padding: 2rem;
    color: var(--text-muted-alt);
  }

  .form-group {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .form-group label {
    font-size: 0.9rem;
    color: var(--text-muted);
    font-weight: 500;
  }

  .form-group input[type="text"],
  .form-group input[type="password"] {
    width: 100%;
    padding: 10px 14px;
    border: 1px solid var(--border-input);
    border-radius: 12px;
    background: var(--bg-input);
    color: var(--text-main);
    font-size: 0.95rem;
    outline: none;
    transition: border-color 0.2s ease;
  }

  .form-group input:focus {
    border-color: var(--border-focus);
  }

  .mode-selector {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .mode-option {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 14px 16px;
    border: 1px solid var(--border-light);
    border-radius: 12px;
    background: var(--bg-panel-light);
    color: var(--text-main);
    cursor: pointer;
    transition: border-color 0.2s, background 0.2s;
    min-height: auto;
  }

  .mode-option:hover {
    border-color: var(--border-focus);
    background: var(--bg-panel-lighter);
  }

  .mode-option.active {
    border-color: var(--text-accent);
    background: rgba(147, 197, 253, 0.08);
  }

  .mode-option svg {
    flex-shrink: 0;
    color: var(--text-muted);
  }

  .mode-option.active svg {
    color: var(--text-accent);
  }

  .mode-option > div {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .mode-option strong {
    font-size: 0.95rem;
  }

  .mode-option span {
    font-size: 0.8rem;
    color: var(--text-muted-alt);
  }

  .minio-config {
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding: 16px;
    border: 1px solid var(--border-light);
    border-radius: 12px;
    background: var(--bg-panel-light);
  }

  .text-button {
    background: transparent;
    border: none;
    color: var(--text-accent);
    font-size: 0.85rem;
    padding: 4px 12px;
    cursor: pointer;
    border-radius: 6px;
    min-height: auto;
  }

  .text-button:hover {
    background: rgba(147, 197, 253, 0.1);
  }

  .primary-btn {
    background: var(--bg-primary-btn);
    color: white;
    border: none;
    border-radius: 10px;
    padding: 0 16px;
    min-height: 38px;
    font-weight: 600;
    cursor: pointer;
    transition: transform 0.1s ease, opacity 0.15s ease;
  }

  .primary-btn:hover:enabled {
    transform: translateY(-1px);
    box-shadow: 0 4px 12px var(--shadow-primary);
    background: var(--bg-primary-btn-hover);
  }

  .primary-btn:disabled {
    cursor: not-allowed;
    opacity: 0.45;
  }

  .error-message {
    background: rgba(239, 68, 68, 0.1);
    color: #ef4444;
    padding: 0.75rem 1rem;
    border-radius: 12px;
    border: 1px solid rgba(239, 68, 68, 0.2);
    margin-bottom: 4px;
  }

  :global(body[data-theme="dark"]) .error-message {
    color: #fca5a5;
  }

  .success-message {
    background: rgba(52, 211, 153, 0.1);
    color: #10b981;
    padding: 0.75rem 1rem;
    border-radius: 12px;
    border: 1px solid rgba(52, 211, 153, 0.2);
    margin-bottom: 4px;
  }

  :global(body[data-theme="dark"]) .success-message {
    color: #a7f3d0;
    background: rgba(6, 78, 59, 0.2);
  }
</style>
