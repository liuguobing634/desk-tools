<script lang="ts">
  import { createNoteGroup, deleteNoteGroup, listNoteGroups, updateNoteGroup } from "$lib/services/note";
  import type { NoteGroup } from "$lib/types/workbench";

  let { onclose } = $props<{ onclose: () => void }>();

  let groups = $state<NoteGroup[]>([]);
  let editingGroupId = $state("");
  let editingGroupName = $state("");
  let editingGroupColor = $state("");
  let showCreateForm = $state(false);
  let loading = $state(true);
  let error = $state("");

  const defaultColors = ["#3b82f6", "#ef4444", "#10b981", "#f59e0b", "#8b5cf6", "#ec4899", "#06b6d4"];

  async function fetchGroups() {
    loading = true;
    error = "";

    try {
      groups = await listNoteGroups();
    } catch (err) {
      error = `加载分组失败：${String(err)}`;
    } finally {
      loading = false;
    }
  }

  async function handleCreateGroup() {
    if (!editingGroupName.trim()) {
      error = "分组名称不能为空";
      return;
    }

    error = "";

    try {
      const group = await createNoteGroup(editingGroupName.trim(), editingGroupColor);
      groups = [...groups, group];
      editingGroupName = "";
      editingGroupColor = "";
      showCreateForm = false;
    } catch (err) {
      error = `创建分组失败：${String(err)}`;
    }
  }

  async function handleUpdateGroup(group: NoteGroup) {
    if (!editingGroupName.trim()) {
      error = "分组名称不能为空";
      return;
    }

    error = "";

    try {
      const updatedGroup = await updateNoteGroup(group.id, editingGroupName.trim(), editingGroupColor);
      groups = groups.map(g => g.id === group.id ? updatedGroup : g);
      cancelEdit();
    } catch (err) {
      error = `更新分组失败：${String(err)}`;
    }
  }

  async function handleDeleteGroup(group: NoteGroup) {
    if (!confirm(`确定要删除分组"${group.name}"吗？此操作无法撤销。`)) {
      return;
    }

    error = "";

    try {
      await deleteNoteGroup(group.id);
      groups = groups.filter(g => g.id !== group.id);
    } catch (err) {
      error = `删除分组失败：${String(err)}`;
    }
  }

  function startEdit(group: NoteGroup) {
    editingGroupId = group.id;
    editingGroupName = group.name;
    editingGroupColor = group.color || "";
  }

  function cancelEdit() {
    editingGroupId = "";
    editingGroupName = "";
    editingGroupColor = "";
  }

  $effect(() => {
    fetchGroups();
  });
</script>

<div class="modal-overlay" onclick={onclose} role="presentation" onkeydown={(e) => e.key === 'Escape' && onclose()}>
  <div class="modal-content group-manager" onclick={(e) => e.stopPropagation()} role="dialog" aria-modal="true" tabindex="-1">
    <div class="modal-header group-header">
      <h3>笔记分组管理</h3>
      <div class="header-actions">
        <button 
          type="button" 
          class="btn btn-primary btn-sm"
          onclick={() => showCreateForm = !showCreateForm}
        >
          {showCreateForm ? '取消新建' : '新建分组'}
        </button>
        <button type="button" class="close-btn" onclick={onclose}>&times;</button>
      </div>
    </div>

    <div class="modal-body group-body">
      {#if error}
        <div class="error-message">{error}</div>
      {/if}

      {#if showCreateForm}
        <div class="create-group-form">
          <div class="form-group">
            <label for="group-name">分组名称</label>
            <input 
              id="group-name"
              type="text" 
              bind:value={editingGroupName}
              placeholder="输入分组名称"
            />
          </div>

          <div class="form-group">
            <label>分组颜色</label>
            <div class="color-picker">
              {#each defaultColors as color}
                <button
                  type="button"
                  class="color-option {editingGroupColor === color ? 'selected' : ''}"
                  style="background-color: {color}"
                  onclick={() => editingGroupColor = color}
                  title="{color}"
                ></button>
              {/each}
              <input 
                type="color" 
                bind:value={editingGroupColor}
                class="color-input"
              />
            </div>
          </div>

          <div class="form-actions">
            <button type="button" class="btn btn-secondary" onclick={cancelEdit}>取消</button>
            <button type="button" class="btn btn-primary" onclick={handleCreateGroup}>创建</button>
          </div>
        </div>
      {/if}

      {#if loading}
        <div class="loading">加载中...</div>
      {:else if groups.length === 0}
        <div class="empty-state">
          <p>还没有分组，点击"新建分组"开始创建。</p>
        </div>
      {:else}
        <div class="groups-list">
          {#each groups as group}
            <div class="group-item">
              {#if editingGroupId === group.id}
                <div class="group-edit-form">
                  <input 
                    type="text" 
                    bind:value={editingGroupName}
                    placeholder="分组名称"
                  />
                  <div class="color-picker">
                    {#each defaultColors as color}
                      <button
                        type="button"
                        class="color-option {editingGroupColor === color ? 'selected' : ''}"
                        style="background-color: {color}"
                        onclick={() => editingGroupColor = color}
                        title="{color}"
                      ></button>
                    {/each}
                    <input 
                      type="color" 
                      bind:value={editingGroupColor}
                      class="color-input"
                    />
                  </div>
                  <div class="edit-actions">
                    <button type="button" class="btn btn-success btn-sm" onclick={() => handleUpdateGroup(group)}>保存</button>
                    <button type="button" class="btn btn-secondary btn-sm" onclick={cancelEdit}>取消</button>
                  </div>
                </div>
              {:else}
                <div class="group-info">
                  <span
                    class="group-color"
                    style="background-color: {group.color || '#3b82f6'}"
                  ></span>
                  <span class="group-name">{group.name}</span>
                </div>
                <div class="group-actions">
                  <button type="button" class="btn btn-outline btn-sm" onclick={() => startEdit(group)}>编辑</button>
                  <button type="button" class="btn btn-danger btn-sm" onclick={() => handleDeleteGroup(group)}>删除</button>
                </div>
              {/if}
            </div>
          {/each}
        </div>
      {/if}
    </div>
  </div>
</div>

<style>
  .btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    padding: 0.4rem 0.8rem;
    border-radius: 8px;
    font-size: 0.9rem;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s ease;
    border: 1px solid transparent;
    outline: none;
  }

  .btn:hover:enabled {
    transform: translateY(-1px);
  }

  .btn-sm {
    padding: 0.25rem 0.6rem;
    font-size: 0.8rem;
    border-radius: 6px;
  }

  .btn-primary {
    background: var(--bg-primary-btn);
    color: white;
  }

  .btn-primary:hover {
    background: var(--bg-primary-btn-hover);
  }

  .btn-secondary {
    background: var(--bg-panel-light);
    color: var(--text-main);
    border-color: var(--border-light);
  }

  .btn-secondary:hover {
    background: var(--border-light);
  }

  .btn-success {
    background: linear-gradient(135deg, #10b981, #059669);
    color: white;
  }

  .btn-success:hover {
    background: linear-gradient(135deg, #34d399, #10b981);
  }

  .btn-danger {
    background: rgba(239, 68, 68, 0.15);
    color: #ef4444;
    border-color: rgba(239, 68, 68, 0.3);
  }

  .btn-danger:hover {
    background: rgba(239, 68, 68, 0.25);
    color: #b91c1c;
  }

  :global(body[data-theme="dark"]) .btn-danger {
    color: #fca5a5;
  }
  
  :global(body[data-theme="dark"]) .btn-danger:hover {
    color: #fecaca;
  }

  .btn-outline {
    background: transparent;
    color: var(--text-accent);
    border-color: var(--border-light);
  }

  .btn-outline:hover {
    background: var(--bg-panel-light);
    border-color: var(--border-focus);
  }

  /* 模态框及分组样式 */
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
    max-width: 500px;
    box-shadow: 0 20px 40px var(--shadow-panel);
    overflow: hidden;
    animation: modal-fade-in 0.2s ease-out;
    display: flex;
    flex-direction: column;
    max-height: 85vh;
  }

  @keyframes modal-fade-in {
    from { opacity: 0; transform: translateY(10px) scale(0.98); }
    to { opacity: 1; transform: translateY(0) scale(1); }
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

  .modal-body {
    padding: 20px;
    overflow-y: auto;
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

  .header-actions {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .create-group-form {
    background: var(--bg-panel-light);
    padding: 1rem;
    border-radius: 12px;
    margin-bottom: 1rem;
    border: 1px solid var(--border-light);
  }

  .form-group {
    margin-bottom: 1rem;
  }

  .form-group label {
    display: block;
    margin-bottom: 0.5rem;
    font-weight: 500;
    color: var(--text-muted);
  }

  .form-group input[type="text"] {
    width: 100%;
    padding: 0.6rem 1rem;
    border: 1px solid var(--border-input);
    border-radius: 12px;
    background: var(--bg-input);
    color: var(--text-main);
    outline: none;
    transition: border-color 0.2s ease;
  }

  .form-group input[type="text"]:focus {
    border-color: var(--border-focus);
  }

  .color-picker {
    display: flex;
    gap: 0.5rem;
    align-items: center;
    flex-wrap: wrap;
  }

  .color-option {
    width: 24px;
    height: 24px;
    border: 2px solid transparent;
    border-radius: 50%;
    cursor: pointer;
    transition: border-color 0.2s;
  }

  .color-option.selected {
    border-color: var(--text-main);
  }

  .color-input {
    width: 32px;
    height: 32px;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    background: transparent;
  }

  .form-actions {
    display: flex;
    gap: 0.5rem;
    justify-content: flex-end;
    margin-top: 1rem;
  }

  .groups-list {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .group-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.75rem 1rem;
    background: var(--bg-panel-lighter);
    border-radius: 12px;
    border: 1px solid var(--border-light);
  }

  .group-info {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .group-color {
    width: 16px;
    height: 16px;
    border-radius: 50%;
    flex-shrink: 0;
  }

  .group-name {
    font-weight: 500;
    color: var(--text-main);
  }

  .group-actions {
    display: flex;
    gap: 0.25rem;
  }

  .group-edit-form {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    flex: 1;
  }

  .group-edit-form input {
    flex: 1;
    padding: 0.4rem 0.8rem;
    border: 1px solid var(--border-input);
    border-radius: 8px;
    background: var(--bg-input);
    color: var(--text-main);
    outline: none;
    transition: border-color 0.2s ease;
  }

  .group-edit-form input:focus {
    border-color: var(--border-focus);
  }

  .edit-actions {
    display: flex;
    gap: 0.5rem;
    margin-left: 0.5rem;
  }

  .loading, .empty-state {
    text-align: center;
    padding: 2rem;
    color: var(--text-muted-alt);
  }

  .error-message {
    background: rgba(239, 68, 68, 0.1);
    color: #ef4444;
    padding: 0.75rem 1rem;
    border-radius: 12px;
    margin-bottom: 1rem;
    border: 1px solid rgba(239, 68, 68, 0.2);
  }
  
  :global(body[data-theme="dark"]) .error-message {
    color: #fca5a5;
  }
</style>