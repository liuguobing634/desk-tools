<script lang="ts">
  import { createNote, deleteNote, listNotes, loadNote, renderMarkdown, saveNote as persistNote, listNoteGroups, moveNoteToGroup } from "$lib/services/note";
  import { importImage } from "$lib/services/note";
  import { convertFileSrc } from "@tauri-apps/api/core";
  import NoteGroupManager from "$lib/components/NoteGroupManager.svelte";
  import SettingsModal from "$lib/components/SettingsModal.svelte";
  import type { NoteDocument, NoteSummary, NoteViewMode, NoteGroup, GroupedNotes } from "$lib/types/workbench";

  let notes = $state<NoteSummary[]>([]);
  let selectedNoteId = $state("");
  let noteSavedContent = $state("");
  let noteDraft = $state("");
  let noteTitle = $state("");
  let noteViewMode = $state<NoteViewMode>("split");
  let noteLoading = $state(true);
  let noteSaving = $state(false);
  let noteListLoading = $state(true);
  let noteError = $state("");
  let noteSaveMessage = $state("");
  let createTitle = $state("");
  let createGroupId = $state<string | null>(null);
  let showCreateModal = $state(false);
  let selectedDeleteIds = $state<string[]>([]);
  let groups = $state<NoteGroup[]>([]);
  let selectedGroupId = $state<string | null>(null);
  let showGroupManager = $state(false);
  let showSettings = $state(false);
  let groupLoading = $state(true);

  // 侧边栏宽度控制
  let sidebarWidth = $state(220);
  let isDragging = $state(false);

  // 用于引用 textarea DOM 元素，以便获取和设置光标位置
  let textareaRef = $state<HTMLTextAreaElement | null>(null);

  const noteDirty = $derived(noteDraft !== noteSavedContent);
  const notePreviewHtml = $derived(renderMarkdown(noteDraft));
  const selectedNote = $derived(notes.find((note) => note.id === selectedNoteId) ?? null);
  const selectedDeleteCount = $derived(selectedDeleteIds.length);
  const filteredNotes = $derived(
    selectedGroupId 
      ? notes.filter(note => note.groupId === selectedGroupId)
      : notes
  );
  const groupedNotes = $derived<GroupedNotes>(
    groups.map(group => ({
      group,
      notes: notes.filter(note => note.groupId === group.id)
    })).filter(group => group.notes.length > 0)
  );

  function applyNoteDocument(document: NoteDocument) {
    selectedNoteId = document.id;
    noteTitle = document.title;
    noteSavedContent = document.content;
    noteDraft = document.content;
  }

  async function fetchNotes(preferredNoteId?: string) {
    noteListLoading = true;
    noteError = "";

    try {
      const nextNotes = await listNotes();
      notes = nextNotes;

      const targetNoteId = preferredNoteId ?? selectedNoteId ?? nextNotes[0]?.id ?? "";

      if (targetNoteId) {
        await openNote(targetNoteId);
      } else {
        selectedNoteId = "";
        noteTitle = "";
        noteSavedContent = "";
        noteDraft = "";
        selectedDeleteIds = [];
        noteLoading = false;
      }
    } catch (error) {
      noteError = `读取笔记列表失败：${String(error)}`;
      noteLoading = false;
    } finally {
      noteListLoading = false;
    }
  }

  async function fetchGroups() {
    groupLoading = true;
    try {
      groups = await listNoteGroups();
    } catch (error) {
      noteError = `加载分组失败：${String(error)}`;
    } finally {
      groupLoading = false;
    }
  }

  function selectGroup(groupId: string | null) {
    selectedGroupId = groupId;
  }

  function toggleGroupManager() {
    showGroupManager = !showGroupManager;
    if (!showGroupManager) {
      // 重新加载分组，因为用户可能在管理面板中做了修改
      fetchGroups();
    }
  }

  async function openNote(id: string) {
    noteLoading = true;
    noteError = "";
    noteSaveMessage = "";

    try {
      console.log('id,', id)
      const document = await loadNote(id);
      applyNoteDocument(document);
    } catch (error) {
      noteError = `读取笔记失败：${String(error)}`;
    } finally {
      noteLoading = false;
    }
  }

  async function handleCreateNote() {
    noteError = "";
    noteSaveMessage = "";

    try {
      const document = await createNote(createTitle.trim(), createGroupId || undefined);
      createTitle = "";
      createGroupId = null;
      showCreateModal = false;
      await fetchNotes(document.id);
      noteSaveMessage = "已创建新笔记。";
    } catch (error) {
      noteError = `创建笔记失败：${String(error)}`;
    }
  }

  function openCreateModal() {
    createTitle = "";
    createGroupId = selectedGroupId;
    showCreateModal = true;
  }

  function closeCreateModal() {
    showCreateModal = false;
    createTitle = "";
    createGroupId = null;
  }

  async function saveNote() {
    if (!selectedNoteId) {
      return;
    }

    noteSaving = true;
    noteError = "";
    noteSaveMessage = "";

    try {
      const document = await persistNote(selectedNoteId, noteDraft);
      applyNoteDocument(document);
      notes = notes
        .map((note) =>
          note.id === document.id
            ? {
                groupId: document.groupId ?? null,
                id: document.id,
                title: document.title,
                fileName: document.fileName,
                updatedAt: document.updatedAt,
              }
            : note
        )
        .sort((a, b) => b.updatedAt - a.updatedAt);
      noteSaveMessage = "笔记已保存到本地 Markdown 文件。";
    } catch (error) {
      noteError = `保存笔记失败：${String(error)}`;
    } finally {
      noteSaving = false;
    }
  }

  async function deleteSelectedNotes() {
    if (selectedDeleteIds.length === 0) {
      return;
    }

    noteSaving = true;
    noteError = "";
    noteSaveMessage = "";

    try {
      const idsToDelete = [...selectedDeleteIds];

      for (const id of idsToDelete) {
        await deleteNote(id);
      }

      const remainingNotes = notes.filter((note) => !idsToDelete.includes(note.id));
      notes = remainingNotes;
      selectedDeleteIds = [];

      if (idsToDelete.includes(selectedNoteId)) {
        const nextSelectedId = remainingNotes[0]?.id ?? "";

        if (nextSelectedId) {
          await openNote(nextSelectedId);
        } else {
          selectedNoteId = "";
          noteTitle = "";
          noteSavedContent = "";
          noteDraft = "";
          noteLoading = false;
        }
      }

      noteSaveMessage = `已删除 ${idsToDelete.length} 篇笔记。`;
    } catch (error) {
      noteError = `批量删除笔记失败：${String(error)}`;
    } finally {
      noteSaving = false;
    }
  }

  function toggleDeleteSelection(id: string, checked: boolean) {
    if (checked) {
      selectedDeleteIds = [...selectedDeleteIds, id];
      return;
    }

    selectedDeleteIds = selectedDeleteIds.filter((selectedId) => selectedId !== id);
  }

  async function handleMoveNote(noteId: string, groupId: string) {
    noteSaving = true;
    noteError = "";
    noteSaveMessage = "";

    try {
      const targetGroupId = groupId === "" ? null : groupId;
      const document = await moveNoteToGroup(noteId, targetGroupId);
      
      // 更新笔记列表中的分组信息
      notes = notes.map(note => 
        note.id === noteId 
          ? { ...note, groupId: document.groupId ?? null }
          : note
      );
      
      // 如果当前正在编辑这个笔记，也更新编辑状态
      if (selectedNoteId === noteId) {
        applyNoteDocument(document);
      }
      
      noteSaveMessage = `笔记已移动到${targetGroupId ? "分组" : "无分组"}`;
    } catch (error) {
      noteError = `移动笔记失败：${String(error)}`;
    } finally {
      noteSaving = false;
    }
  }

  function shortenFileName(fileName: string, maxLength = 22) {
    if (fileName.length <= maxLength) {
      return fileName;
    }

    const dotIndex = fileName.lastIndexOf(".");
    const hasExtension = dotIndex > 0 && dotIndex < fileName.length - 1;
    const extension = hasExtension ? fileName.slice(dotIndex) : "";
    const baseName = hasExtension ? fileName.slice(0, dotIndex) : fileName;
    const reservedLength = extension.length + 3;
    const availableBaseLength = Math.max(maxLength - reservedLength, 6);
    const headLength = Math.ceil(availableBaseLength / 2);
    const tailLength = Math.floor(availableBaseLength / 2);

    return `${baseName.slice(0, headLength)}...${baseName.slice(-tailLength)}${extension}`;
  }

  async function handleImportImage() {
    try {
      const imagePath = await importImage();
      if (imagePath) {
        // MinIO storage returns HTTP URLs, local storage returns filesystem paths
        const url = imagePath.startsWith("http://") || imagePath.startsWith("https://")
          ? imagePath
          : convertFileSrc(imagePath);
        const imgMarkdown = `![图片](${url})`;

        if (textareaRef) {
          const start = textareaRef.selectionStart;
          const end = textareaRef.selectionEnd;
          
          // 在光标位置插入 markdown 文本
          noteDraft = noteDraft.substring(0, start) + imgMarkdown + noteDraft.substring(end);
          
          // 插入后恢复焦点，并把光标放在插入内容的后面
          setTimeout(() => {
            if (textareaRef) {
              textareaRef.focus();
              const newPos = start + imgMarkdown.length;
              textareaRef.setSelectionRange(newPos, newPos);
            }
          }, 0);
        } else {
          // 兜底方案，如果在纯预览模式下调用或者找不到 textarea 时，默认加到末尾
          const prefix = noteDraft && !noteDraft.endsWith('\n') ? '\n\n' : '';
          noteDraft += `${prefix}${imgMarkdown}\n`;
        }
      }
    } catch (error) {
      noteError = `插入图片失败：${String(error)}`;
    }
  }

  function resetNoteMessage() {
    if (noteSaveMessage) {
      noteSaveMessage = "";
    }
  }

  function handlePointerDown(e: PointerEvent) {
    // 仅响应鼠标左键
    if (e.button !== 0) return;
    isDragging = true;
    e.preventDefault(); // 阻止默认选中文本行为

    // 在 document 级别绑定，确保拖到组件外面也能持续触发
    document.addEventListener('pointermove', handlePointerMove);
    document.addEventListener('pointerup', handlePointerUp);
    document.body.style.cursor = 'col-resize';
  }

  function handlePointerMove(e: PointerEvent) {
    if (!isDragging) return;
    
    const container = document.querySelector('.notes-workspace');
    if (container) {
      const rect = container.getBoundingClientRect();
      const newWidth = e.clientX - rect.left;
      // 限制侧边栏宽度在 180px 到 500px 之间，防止过小或过大影响布局
      sidebarWidth = Math.max(180, Math.min(newWidth, 500));
    }
  }

  function handlePointerUp() {
    isDragging = false;
    document.removeEventListener('pointermove', handlePointerMove);
    document.removeEventListener('pointerup', handlePointerUp);
    document.body.style.cursor = '';
  }

  function exportPdf() {
    if (!selectedNoteId) return;
    
    // 如果当前是纯编辑模式，先切换到分栏或预览模式以便渲染出预览 DOM
    if (noteViewMode === "edit") {
      noteViewMode = "preview";
    }
    
    // 给 Svelte 一点时间更新 DOM，然后调用浏览器原生的打印功能
    setTimeout(() => {
      window.print();
    }, 150);
  }

  fetchNotes();
  fetchGroups();
</script>

<section class="panel">
  <div class="panel-header">
    <div>
      <h2>Markdown 笔记</h2>
      <p class="panel-tip">每篇笔记单独保存为本地 Markdown 文件，支持多笔记切换、编辑、预览和删除。</p>
    </div>

    <div class="note-actions">
      <div class="filters">
        <button type="button" class:active={noteViewMode === "edit"} onclick={() => (noteViewMode = "edit")}>
          编辑
        </button>
        <button
          type="button"
          class:active={noteViewMode === "preview"}
          onclick={() => (noteViewMode = "preview")}
        >
          预览
        </button>
        <button
          type="button"
          class:active={noteViewMode === "split"}
          onclick={() => (noteViewMode = "split")}
        >
          分栏
        </button>
      </div>

      <button type="button" class="export-pdf-btn" onclick={exportPdf} disabled={noteLoading || !selectedNoteId} title="通过系统打印对话框另存为 PDF">
        <svg viewBox="0 0 24 24" width="16" height="16" stroke="currentColor" stroke-width="2" fill="none" stroke-linecap="round" stroke-linejoin="round" style="margin-right: 6px; vertical-align: middle;">
          <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"></path>
          <polyline points="14 2 14 8 20 8"></polyline>
          <line x1="16" y1="13" x2="8" y2="13"></line>
          <line x1="16" y1="17" x2="8" y2="17"></line>
          <polyline points="10 9 9 9 8 9"></polyline>
        </svg>
        导出 PDF
      </button>

      <button type="button" onclick={saveNote} disabled={noteLoading || noteSaving || !noteDirty || !selectedNoteId}>
        {noteSaving ? "保存中..." : "保存笔记"}
      </button>
    </div>
  </div>

  {#if noteError}
    <p class="error-tip">{noteError}</p>
  {/if}

  {#if noteSaveMessage}
    <p class="success-tip">{noteSaveMessage}</p>
  {/if}

  <div class="notes-workspace" style="--sidebar-width: {sidebarWidth}px;">
    <aside class="notes-sidebar">
      <div class="sidebar-header">
        <div class="sidebar-title-icon" title="笔记列表">
          <svg viewBox="0 0 24 24" width="20" height="20" stroke="currentColor" stroke-width="2" fill="none" stroke-linecap="round" stroke-linejoin="round">
            <line x1="8" y1="6" x2="21" y2="6"></line>
            <line x1="8" y1="12" x2="21" y2="12"></line>
            <line x1="8" y1="18" x2="21" y2="18"></line>
            <line x1="3" y1="6" x2="3.01" y2="6"></line>
            <line x1="3" y1="12" x2="3.01" y2="12"></line>
            <line x1="3" y1="18" x2="3.01" y2="18"></line>
          </svg>
        </div>
        <div class="header-actions">
          <button type="button" class="text-button" onclick={() => { fetchNotes(selectedNoteId); fetchGroups(); }} disabled={noteListLoading || noteSaving}>
            刷新
          </button>
          <button type="button" class="text-button" onclick={toggleGroupManager} disabled={noteListLoading || noteSaving}>
            分组管理
          </button>
          <button type="button" class="text-button" onclick={() => (showSettings = !showSettings)} title="设置">
            <svg viewBox="0 0 24 24" width="16" height="16" stroke="currentColor" stroke-width="2" fill="none" stroke-linecap="round" stroke-linejoin="round">
              <circle cx="12" cy="12" r="3"></circle>
              <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"></path>
            </svg>
          </button>
        </div>
      </div>

      <!-- 分组选择器 -->
      <div class="group-selector">
        <select bind:value={selectedGroupId} disabled={groupLoading || noteSaving}>
          <option value={null}>所有笔记</option>
          {#each groups as group}
            <option value={group.id}>{group.name}</option>
          {/each}
        </select>
      </div>

      <div class="add-note-wrapper">
        <button type="button" class="add-note-btn" onclick={openCreateModal} disabled={noteSaving || noteListLoading} title="新建笔记">
          <svg viewBox="0 0 24 24" width="20" height="20" stroke="currentColor" stroke-width="2" fill="none" stroke-linecap="round" stroke-linejoin="round">
            <line x1="12" y1="5" x2="12" y2="19"></line>
            <line x1="5" y1="12" x2="19" y2="12"></line>
          </svg>
          新建笔记
        </button>
      </div>

      <div class="batch-actions">
        <span>{selectedDeleteCount > 0 ? `已勾选 ${selectedDeleteCount} 篇` : "未勾选笔记"}</span>
        <button
          type="button"
          class="delete-selected-button"
          onclick={deleteSelectedNotes}
          disabled={selectedDeleteCount === 0 || noteSaving || noteListLoading}
        >
          删除勾选
        </button>
      </div>

      {#if noteListLoading && filteredNotes.length === 0}
        <div class="empty-state sidebar-empty">
          <p>正在加载笔记列表...</p>
        </div>
      {:else if filteredNotes.length > 0}
        <ul class="note-list">
          {#each filteredNotes as note (note.id)}
            <li class:selected={note.id === selectedNoteId}>
              <label class="note-check">
                <input
                  type="checkbox"
                  checked={selectedDeleteIds.includes(note.id)}
                  onchange={(event) => toggleDeleteSelection(note.id, (event.currentTarget as HTMLInputElement).checked)}
                  disabled={noteSaving}
                />
              </label>
              <button type="button" class="note-item" onclick={() => openNote(note.id)} disabled={noteSaving}>
                <strong>{note.title}</strong>
                <span title={note.fileName}>{shortenFileName(note.fileName)}</span>
              </button>
              <div class="note-actions">
                <select 
                  value={note.groupId || ""} 
                  onchange={(event) => handleMoveNote(note.id, (event.currentTarget as HTMLSelectElement).value)}
                  disabled={noteSaving}
                  title="移动到分组"
                >
                  <option value="">无分组</option>
                  {#each groups as group}
                    <option value={group.id}>{group.name}</option>
                  {/each}
                </select>
              </div>
            </li>
          {/each}
        </ul>
      {:else}
        <div class="empty-state sidebar-empty">
          <p>{selectedGroupId ? "该分组下还没有笔记" : "还没有笔记，先创建第一篇 Markdown 笔记。"}</p>
        </div>
      {/if}
    </aside>

    <div class="resizer" onpointerdown={handlePointerDown} aria-hidden="true"></div>

    <div class="note-main">
      <div class="note-meta">
        <span>{selectedNote ? `当前笔记：${selectedNote.title}` : "当前没有打开的笔记"}</span>
        <span>{noteDirty ? "有未保存修改" : "已与本地文件同步"}</span>
      </div>

      {#if noteLoading}
        <div class="empty-state">
          <h3>正在加载笔记</h3>
          <p>请稍候，正在从本地 Markdown 文件读取内容。</p>
        </div>
      {:else if selectedNoteId}
        <div class:single-column={noteViewMode !== "split"} class="note-layout">
          {#if noteViewMode !== "preview"}
            <section class="note-pane">
              <div class="pane-title">
                {noteTitle || "编辑区"}
                <button type="button" class="btn btn-sm btn-outline import-img-btn" onclick={handleImportImage} disabled={noteSaving} title="插入本地图片">
                  <svg viewBox="0 0 24 24" width="16" height="16" stroke="currentColor" stroke-width="2" fill="none" stroke-linecap="round" stroke-linejoin="round">
                    <rect x="3" y="3" width="18" height="18" rx="2" ry="2"></rect>
                    <circle cx="8.5" cy="8.5" r="1.5"></circle>
                    <polyline points="21 15 16 10 5 21"></polyline>
                  </svg>
                  插入图片
                </button>
              </div>
              <textarea
                bind:this={textareaRef}
                bind:value={noteDraft}
                class="note-editor"
                placeholder="# 我的笔记&#10;&#10;- 支持 Markdown 编辑&#10;- 支持本地文件保存&#10;- 支持预览"
                disabled={noteSaving}
                oninput={resetNoteMessage}
              ></textarea>
            </section>
          {/if}

          {#if noteViewMode !== "edit"}
            <section class="note-pane">
              <div class="pane-title">预览区</div>
              <div class="markdown-preview">{@html notePreviewHtml}</div>
            </section>
          {/if}
        </div>
      {:else}
        <div class="empty-state">
          <h3>请选择一篇笔记</h3>
          <p>从左侧打开笔记，或者先创建一篇新的 Markdown 笔记。</p>
        </div>
      {/if}
    </div>
  </div>
</section>

{#if showCreateModal}
  <div class="modal-overlay" onclick={closeCreateModal} role="presentation" onkeydown={(e) => e.key === 'Escape' && closeCreateModal()}>
    <div class="modal-content" onclick={(e) => e.stopPropagation()} role="dialog" aria-modal="true" tabindex="-1">
      <div class="modal-header">
        <h3>新建笔记</h3>
        <button type="button" class="close-btn" onclick={closeCreateModal}>&times;</button>
      </div>
      
      <div class="modal-body">
        <div class="form-group">
          <label for="new-note-title">标题</label>
          <input 
            id="new-note-title"
            bind:value={createTitle} 
            class="note-name-input" 
            placeholder="新笔记标题，例如：项目会议记录" 
            autofocus
            onkeydown={(e) => e.key === 'Enter' && handleCreateNote()}
          />
        </div>
        
        <div class="form-group">
          <label for="new-note-group">分组</label>
          <select id="new-note-group" bind:value={createGroupId} class="note-group-select">
            <option value={null}>无分组</option>
            {#each groups as group}
              <option value={group.id}>{group.name}</option>
            {/each}
          </select>
        </div>
      </div>
      
      <div class="modal-footer">
        <button type="button" class="text-button" onclick={closeCreateModal}>取消</button>
        <button type="button" class="primary-btn" onclick={handleCreateNote} disabled={noteSaving || noteListLoading}>确认创建</button>
      </div>
    </div>
  </div>
{/if}

{#if showGroupManager}
  <NoteGroupManager onclose={toggleGroupManager} />
{/if}

{#if showSettings}
  <SettingsModal onclose={() => (showSettings = false)} />
{/if}

<style>
  h2,
  h3 {
    margin: 0;
  }

  :global(code) {
    padding: 0.15em 0.4em;
    border-radius: 0.45em;
    background: var(--bg-input);
  }

  .panel {
    padding: 28px;
    border: 1px solid var(--border-main);
    border-radius: 24px;
    background: var(--bg-panel);
    box-shadow: 0 24px 80px var(--shadow-panel);
    backdrop-filter: blur(14px);
  }

  .panel-header,
  .filters,
  .note-actions,
  .note-meta {
    display: flex;
    align-items: center;
  }

  .export-pdf-btn {
    display: flex;
    align-items: center;
    background: var(--bg-button);
    color: var(--text-main);
    border: 1px solid var(--border-input);
    transition: background 0.2s, transform 0.15s;
  }

  .export-pdf-btn:hover:enabled {
    background: var(--bg-active-btn);
    color: white;
    transform: translateY(-1px);
  }

  /* 打印/导出 PDF 专用样式 */
  @media print {
    /* 隐藏不需要打印的区域 */
    .panel-header,
    .notes-sidebar,
    .resizer,
    .note-meta,
    .pane-title,
    :global(.hero) {
      display: none !important;
    }

    /* 调整主体布局，使其占满整个纸张 */
    .notes-workspace {
      display: block !important;
      width: 100% !important;
      grid-template-columns: none !important;
      gap: 0 !important;
    }

    .note-main {
      width: 100% !important;
      overflow: visible !important;
      height: auto !important;
    }

    .note-layout {
      display: block !important;
      overflow: visible !important;
      height: auto !important;
    }

    /* 隐藏编辑器，只保留预览区 */
    .note-editor {
      display: none !important;
    }

    .note-pane {
      border: none !important;
      padding: 0 !important;
      background: transparent !important;
      overflow: visible !important;
      height: auto !important;
    }

    .markdown-preview {
      display: block !important;
      width: 100% !important;
      height: auto !important;
      overflow: visible !important;
      color: black !important;
      background: white !important;
    }

    /* 保证背景和文字颜色适合打印 */
    :global(body) {
      background: white !important;
      color: black !important;
    }

    :global(.page),
    :global(.workspace) {
      padding: 0 !important;
      margin: 0 !important;
      min-height: auto !important;
      width: 100% !important;
    }

    .panel {
      border: none !important;
      box-shadow: none !important;
      padding: 0 !important;
      background: transparent !important;
    }
  }

  .panel-header {
    justify-content: space-between;
    align-items: flex-start;
    gap: 20px;
    margin-bottom: 24px;
  }

  .panel-tip {
    margin: 12px 0 0;
    color: var(--text-muted);
  }

  .notes-workspace {
    display: grid;
    /* 通过 CSS 变量动态控制侧边栏宽度，加入 resizer 和右侧主体 */
    grid-template-columns: var(--sidebar-width) 6px minmax(0, 1fr);
    gap: 12px;
  }

  .resizer {
    cursor: col-resize;
    border-radius: 4px;
    background-color: var(--border-light);
    transition: background-color 0.2s ease;
    height: 100%;
  }

  .resizer:hover,
  .resizer:active {
    background-color: var(--border-focus);
  }

  .notes-sidebar {
    display: flex;
    flex-direction: column;
    gap: 10px;
    min-height: 500px;
    max-height: 600px;
    overflow-y: auto;
    padding: 12px;
    border-radius: 20px;
    background: var(--bg-panel-light);
    border: 1px solid var(--border-light);
  }

  .sidebar-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 4px 6px;
  }

  .sidebar-title-icon {
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--text-muted);
    padding: 4px;
    border-radius: 6px;
  }

  .header-actions {
    display: flex;
    gap: 0.5rem;
  }

  .group-selector {
    padding: 0.5rem 0.25rem;
    border-bottom: 1px solid var(--border-light);
  }

  .group-selector select {
    width: 100%;
    padding: 0.6rem 1rem;
    border: 1px solid var(--border-input);
    border-radius: 12px;
    background: var(--bg-input);
    color: var(--text-main);
    font-size: 0.95rem;
    outline: none;
    cursor: pointer;
    transition: border-color 0.2s ease;
  }

  .group-selector select:hover,
  .group-selector select:focus {
    border-color: var(--border-focus);
  }

  .group-selector select option {
    background: var(--bg-panel);
    color: var(--text-main);
    padding: 0.5rem;
  }

  .text-button {
    background: transparent;
    border: none;
    color: var(--text-accent);
    font-size: 0.85rem;
    padding: 4px 8px;
    cursor: pointer;
    border-radius: 6px;
    min-height: auto;
  }

  .text-button:hover:enabled {
    background: rgba(147, 197, 253, 0.1);
  }

  .text-button:disabled {
    color: var(--text-muted-alt);
    cursor: not-allowed;
  }

  .add-note-wrapper {
    padding: 10px 6px;
    border-bottom: 1px solid var(--border-light);
  }

  .add-note-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
    width: 100%;
    min-height: 42px;
    padding: 0 16px;
    font-weight: 600;
    color: white;
    background: var(--bg-primary-btn);
    border: none;
    border-radius: 12px;
    cursor: pointer;
    transition: transform 0.15s ease, opacity 0.15s ease, box-shadow 0.15s ease;
    box-shadow: 0 4px 12px var(--shadow-primary);
  }

  .add-note-btn:hover:enabled {
    transform: translateY(-2px);
    background: var(--bg-primary-btn-hover);
    box-shadow: 0 6px 16px var(--shadow-primary);
  }

  .batch-actions {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 6px;
    font-size: 0.8rem;
    color: var(--text-muted);
  }

  .delete-selected-button {
    background: transparent;
    border: 1px solid rgba(239, 68, 68, 0.4);
    color: #fca5a5;
    padding: 4px 10px;
    border-radius: 6px;
    font-size: 0.8rem;
    cursor: pointer;
    min-height: auto;
  }

  .delete-selected-button:hover:enabled {
    background: rgba(239, 68, 68, 0.15);
    border-color: #ef4444;
    color: #fecaca;
  }

  .delete-selected-button:disabled {
    border-color: var(--border-lighter);
    color: var(--text-muted-alt);
    cursor: not-allowed;
  }

  input,
  button,
  textarea {
    border: 1px solid var(--border-input);
    border-radius: 14px;
    font: inherit;
  }

  button {
    min-height: 46px;
    padding: 0 18px;
    font-weight: 600;
    color: white;
    background: var(--bg-primary-btn);
    cursor: pointer;
    transition:
      transform 0.15s ease,
      border-color 0.15s ease,
      opacity 0.15s ease;
  }

  button:hover:enabled {
    transform: translateY(-1px);
    border-color: var(--border-focus);
    background: var(--bg-primary-btn-hover);
  }

  button:disabled {
    cursor: not-allowed;
    opacity: 0.45;
  }

  .filters {
    gap: 10px;
    margin-right: 16px;
  }

  .filters button {
    min-height: 42px;
    background: var(--bg-button);
    color: var(--text-main-alt);
  }

  .filters button.active {
    background: var(--bg-active-btn);
    color: white;
  }

  .note-list {
    display: grid;
    gap: 8px;
    margin: 0;
    padding: 0;
    list-style: none;
  }

  .note-list li {
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: 10px;
    border: 1px solid transparent;
    border-radius: 14px;
    background: transparent;
    transition:
      background 0.2s,
      border-color 0.2s;
  }

  .note-list li:hover {
    background: var(--bg-panel-lighter);
    border-color: var(--border-lighter);
  }

  .note-list li.selected {
    background: var(--bg-panel-light);
    border-color: var(--border-light);
  }

  .note-list button.note-item {
    background: transparent;
    border: none;
    color: var(--text-main);
    padding: 0;
    min-height: auto;
  }

  .note-list button.note-item:hover:enabled {
    transform: none;
  }

  .note-check {
    display: flex;
    align-items: center;
    justify-content: center;
    padding-top: 4px;
  }

  .note-check input {
    width: 16px;
    height: 16px;
    margin: 0;
  }

  .note-item {
    display: grid;
    gap: 4px;
    justify-items: start;
    text-align: left;
  }

  .note-item strong {
    font-size: 0.96rem;
  }

  .note-item span {
    font-size: 0.8rem;
    color: var(--text-muted-alt);
  }

  .note-layout {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 16px;
  }

  .note-layout.single-column {
    grid-template-columns: minmax(0, 1fr);
  }

  .note-pane {
    min-height: 520px;
    border: 1px solid var(--border-light);
    border-radius: 18px;
    overflow: hidden;
    background: var(--bg-panel-light);
  }

  .pane-title {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 14px 16px;
    border-bottom: 1px solid var(--border-lighter);
    color: var(--text-muted);
    background: var(--bg-header);
  }

  .import-img-btn {
    display: flex;
    align-items: center;
    gap: 4px;
    background: transparent;
    color: var(--text-accent);
    border: 1px solid var(--border-light);
    border-radius: 6px;
    padding: 4px 8px;
    font-size: 0.8rem;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .import-img-btn:hover:enabled {
    background: rgba(147, 197, 253, 0.1);
    border-color: var(--border-focus);
  }

  .note-editor {
    width: 100%;
    min-height: 470px;
    padding: 18px;
    border: 0;
    border-radius: 0;
    resize: vertical;
    line-height: 1.7;
    outline: none;
    color: var(--text-main);
    background: var(--bg-input);
  }

  .markdown-preview {
    padding: 18px;
    color: var(--text-main);
    line-height: 1.75;
    overflow-wrap: anywhere;
  }

  .markdown-preview :global(h1),
  .markdown-preview :global(h2),
  .markdown-preview :global(h3) {
    margin: 0 0 0.75em;
  }

  .markdown-preview :global(img) {
    max-width: 100%;
    height: auto;
    border-radius: 8px;
    margin: 12px 0;
  }

  .markdown-preview :global(p),
  .markdown-preview :global(ul),
  .markdown-preview :global(ol),
  .markdown-preview :global(blockquote) {
    margin: 0 0 1em;
  }

  .markdown-preview :global(a) {
    color: var(--text-accent);
  }

  .markdown-preview :global(pre) {
    overflow: auto;
    padding: 14px;
    border-radius: 14px;
    background: var(--bg-pre);
  }

  .markdown-preview :global(blockquote) {
    padding-left: 14px;
    border-left: 3px solid #38bdf8;
    color: var(--text-muted);
  }

  .empty-state {
    padding: 36px 18px;
    border: 1px dashed var(--border-dashed);
    border-radius: 18px;
    text-align: center;
    color: var(--text-muted);
  }

  .sidebar-empty {
    padding: 18px 14px;
  }

  .sidebar-empty p {
    margin: 0;
  }

  .error-tip,
  .success-tip {
    margin: 0 0 16px;
    padding: 12px 14px;
    border-radius: 14px;
  }

  .error-tip {
    border: 1px solid rgba(248, 113, 113, 0.35);
    color: #fecaca;
    background: rgba(127, 29, 29, 0.2);
  }

  :global(body[data-theme="light"]) .error-tip {
    color: #ef4444;
    background: rgba(254, 226, 226, 0.5);
  }

  .success-tip {
    border: 1px solid rgba(52, 211, 153, 0.35);
    color: #a7f3d0;
    background: rgba(6, 78, 59, 0.2);
  }

  :global(body[data-theme="light"]) .success-tip {
    color: #10b981;
    background: rgba(209, 250, 229, 0.5);
  }

  .note-meta {
    gap: 16px;
    margin-bottom: 16px;
    font-size: 0.9rem;
    color: var(--text-muted);
  }

  /* 模态框样式 */
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
    max-width: 400px;
    box-shadow: 0 20px 40px var(--shadow-panel);
    overflow: hidden;
    animation: modal-fade-in 0.2s ease-out;
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

  .note-name-input {
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

  .note-name-input:focus {
    border-color: var(--border-focus);
  }

  .note-group-select {
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

  .note-group-select:focus {
    border-color: var(--border-focus);
  }

  .note-group-select option {
    background: var(--bg-panel);
    color: var(--text-main);
  }

  .modal-footer {
    display: flex;
    justify-content: flex-end;
    gap: 12px;
    padding: 16px 20px;
    border-top: 1px solid var(--border-light);
    background: var(--bg-panel-light);
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
    transition: transform 0.1s ease;
  }

  .primary-btn:hover:enabled {
    transform: translateY(-1px);
    box-shadow: 0 4px 12px var(--shadow-primary);
    background: var(--bg-primary-btn-hover);
  }

  /* 笔记操作样式 */
  .note-actions {
    margin-left: auto;
    min-width: 120px;
  }

  .note-actions select {
    width: 100%;
    padding: 0.4rem 0.6rem;
    border: 1px solid var(--border-input);
    border-radius: 8px;
    background: var(--bg-input);
    color: var(--text-main);
    font-size: 0.875rem;
    outline: none;
    cursor: pointer;
    transition: border-color 0.2s ease;
  }

  .note-actions select:hover,
  .note-actions select:focus {
    border-color: var(--border-focus);
  }

  .note-actions select option {
    background: var(--bg-panel);
    color: var(--text-main);
  }

  @media (max-width: 640px) {
    .note-actions {
      min-width: 100px;
    }
    
    .note-actions select {
      font-size: 0.75rem;
    }
  }

  @media (max-width: 840px) {
    .panel-header,
    .filters {
      flex-direction: column;
      align-items: stretch;
    }

    .filters {
      margin: 0 0 16px;
    }

    .filters button,
    .panel-header button {
      width: 100%;
    }

    .notes-workspace {
      grid-template-columns: 1fr;
    }

    .resizer {
      display: none;
    }

    .notes-sidebar {
      min-height: auto;
    }

    .note-layout {
      grid-template-columns: 1fr;
    }
  }
</style>
