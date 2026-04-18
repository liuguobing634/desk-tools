<script lang="ts">
  import { loadTodos, saveTodos } from "$lib/services/todo";
  import type { FilterType, TodoItem } from "$lib/types/workbench";

  function getLocalYMD(d: Date = new Date()) {
    const y = d.getFullYear();
    const m = String(d.getMonth() + 1).padStart(2, '0');
    const day = String(d.getDate()).padStart(2, '0');
    return `${y}-${m}-${day}`;
  }

  let todayYMD = getLocalYMD();
  let currentDate = $state(todayYMD);

  let newTodo = $state("");
  let todos = $state<TodoItem[]>([]);
  let filter = $state<FilterType>("all");
  let loading = $state(true);
  let errorMessage = $state("");
  let showHistory = $state(false);

  const activeTodos = $derived(todos.filter(t => !t.archived && t.date === currentDate));
  const archivedTodos = $derived([...todos.filter(t => t.archived)].sort((a, b) => {
    const dateA = a.date || "";
    const dateB = b.date || "";
    if (dateA !== dateB) return dateB > dateA ? 1 : -1;
    return b.createdAt - a.createdAt;
  }));

  function changeDate(days: number) {
    const d = new Date(currentDate);
    d.setDate(d.getDate() + days);
    currentDate = getLocalYMD(d);
  }

  const filteredTodos = $derived.by(() => {
    if (filter === "active") {
      return activeTodos.filter((todo) => !todo.done);
    }

    if (filter === "completed") {
      return activeTodos.filter((todo) => todo.done);
    }

    return activeTodos;
  });

  const completedCount = $derived(activeTodos.filter((todo) => todo.done).length);
  const remainingCount = $derived(activeTodos.length - completedCount);

  async function fetchTodos() {
    loading = true;
    errorMessage = "";

    try {
      const loaded = await loadTodos();
      let modified = false;

      const nextTodos = loaded.map(t => {
        let changed = false;
        let tDate = t.date;
        let tArchived = t.archived;

        if (!tDate) {
          tDate = getLocalYMD(new Date(t.createdAt));
          changed = true;
          if (!tArchived) {
            tArchived = true;
          }
        }

        if (changed) {
          modified = true;
          return { ...t, date: tDate, archived: tArchived };
        }
        return t;
      });

      todos = nextTodos;
      if (modified) {
        await saveTodos(todos);
      }
    } catch (error) {
      errorMessage = `读取待办数据失败：${String(error)}`;
    } finally {
      loading = false;
    }
  }

  async function persistTodos(nextTodos: TodoItem[]) {
    errorMessage = "";

    try {
      await saveTodos(nextTodos);
      todos = nextTodos;
    } catch (error) {
      errorMessage = `保存待办数据失败：${String(error)}`;
    }
  }

  fetchTodos();

  async function addTodo(event: SubmitEvent) {
    event.preventDefault();

    const text = newTodo.trim();

    if (!text) {
      return;
    }

    const nextTodos = [
      {
        id: crypto.randomUUID(),
        text,
        done: false,
        createdAt: Date.now(),
        date: currentDate
      },
      ...todos
    ];

    newTodo = "";
    await persistTodos(nextTodos);
  }

  async function toggleTodo(id: string) {
    const nextTodos = todos.map((todo) => (todo.id === id ? { ...todo, done: !todo.done } : todo));
    await persistTodos(nextTodos);
  }

  async function removeTodo(id: string) {
    const nextTodos = todos.filter((todo) => todo.id !== id);
    await persistTodos(nextTodos);
  }

  async function archiveCompleted() {
    const nextTodos = todos.map((todo) => 
      !todo.archived && todo.done && todo.date === currentDate ? { ...todo, archived: true } : todo
    );
    await persistTodos(nextTodos);
  }

  async function unarchiveTodo(id: string) {
    const nextTodos = todos.map((todo) => (todo.id === id ? { ...todo, archived: false } : todo));
    await persistTodos(nextTodos);
  }

  async function clearArchived() {
    const nextTodos = todos.filter((todo) => !todo.archived);
    await persistTodos(nextTodos);
  }
</script>

<section class="panel">
  <div class="panel-header">
    <div>
      <div class="header-title">
        <h2>待办清单</h2>
        <div class="date-picker">
          <button class="icon-btn" onclick={() => changeDate(-1)} aria-label="上一天" disabled={loading}>◀</button>
          <input type="date" bind:value={currentDate} disabled={loading} />
          <button class="icon-btn" onclick={() => changeDate(1)} aria-label="下一天" disabled={loading}>▶</button>
          {#if currentDate !== todayYMD}
            <button class="today-btn" onclick={() => currentDate = todayYMD}>回到今天</button>
          {/if}
        </div>
      </div>
      <p class="panel-tip">管理每日任务，按天规划你的工作。</p>
    </div>

    <div class="stats">
      <article>
        <strong>{activeTodos.length}</strong>
        <span>全部任务</span>
      </article>
      <article>
        <strong>{remainingCount}</strong>
        <span>待完成</span>
      </article>
      <article>
        <strong>{completedCount}</strong>
        <span>已完成</span>
      </article>
    </div>
  </div>

  <form class="composer" onsubmit={addTodo}>
    <input
      bind:value={newTodo}
      class="text-input"
      placeholder="输入新的待办事项，例如：整理今天的工作计划"
      disabled={loading}
    />
    <button type="submit" disabled={loading}>添加任务</button>
  </form>

  {#if errorMessage}
    <p class="error-tip">{errorMessage}</p>
  {/if}

  <div class="toolbar">
    <div class="filters" aria-label="任务筛选">
      <button type="button" class:active={filter === "all"} onclick={() => (filter = "all")}>全部</button>
      <button type="button" class:active={filter === "active"} onclick={() => (filter = "active")}>
        进行中
      </button>
      <button
        type="button"
        class:active={filter === "completed"}
        onclick={() => (filter = "completed")}
      >
        已完成
      </button>
    </div>

    <div class="toolbar-actions">
      <button
        type="button"
        class="secondary-button"
        onclick={() => (showHistory = true)}
      >
        历史待办 ({archivedTodos.length})
      </button>
      <button
        type="button"
        class="secondary-button"
        onclick={archiveCompleted}
        disabled={completedCount === 0 || loading}
      >
        归档已完成
      </button>
    </div>
  </div>

  {#if loading}
    <div class="empty-state">
      <h3>正在加载待办数据</h3>
      <p>请稍候，正在从本地文件读取任务列表。</p>
    </div>
  {:else if filteredTodos.length > 0}
    <ul class="todo-list">
      {#each filteredTodos as todo (todo.id)}
        <li class:done={todo.done}>
          <label class="todo-row">
            <input
              type="checkbox"
              checked={todo.done}
              onchange={() => toggleTodo(todo.id)}
              disabled={loading}
            />
            <span>{todo.text}</span>
          </label>

          <button
            type="button"
            class="danger-button"
            onclick={() => removeTodo(todo.id)}
            disabled={loading}
          >
            删除
          </button>
        </li>
      {/each}
    </ul>
  {:else if todos.length === 0}
    <div class="empty-state">
      <h3>还没有任务</h3>
      <p>先添加一条待办，后面我们还可以继续扩展分类、提醒和时间计划。</p>
    </div>
  {:else}
    <div class="empty-state">
      <h3>当前筛选下没有任务</h3>
      <p>切换筛选条件，或者继续添加新的待办事项。</p>
    </div>
  {/if}
</section>

{#if showHistory}
  <div class="modal-overlay" onclick={() => (showHistory = false)} role="presentation" onkeydown={(e) => e.key === 'Escape' && (showHistory = false)}>
    <div class="modal-content" onclick={(e) => e.stopPropagation()} role="dialog" aria-modal="true" tabindex="-1">
      <div class="modal-header">
        <h3>历史待办</h3>
        <button type="button" class="close-btn" onclick={() => (showHistory = false)}>&times;</button>
      </div>

      <div class="modal-body">
        {#if archivedTodos.length > 0}
          <ul class="todo-list">
            {#each archivedTodos as todo (todo.id)}
              <li class="done">
                <label class="todo-row">
                  <input type="checkbox" checked disabled />
                  {#if todo.date}
                    <span class="date-badge">{todo.date}</span>
                  {/if}
                  <span>{todo.text}</span>
                </label>
                <div class="archived-actions">
                  <button type="button" class="secondary-button btn-sm" onclick={() => unarchiveTodo(todo.id)}>
                    恢复
                  </button>
                  <button type="button" class="danger-button btn-sm" onclick={() => removeTodo(todo.id)}>
                    删除
                  </button>
                </div>
              </li>
            {/each}
          </ul>
        {:else}
          <div class="empty-state">
            <p>暂无历史归档的待办事项。</p>
          </div>
        {/if}
      </div>

      <div class="modal-footer">
        <button type="button" class="danger-button" onclick={clearArchived} disabled={archivedTodos.length === 0}>
          清空历史
        </button>
        <button type="button" class="secondary-button" onclick={() => (showHistory = false)}>关闭</button>
      </div>
    </div>
  </div>
{/if}

<style>
  h2,
  h3 {
    margin: 0;
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
  .composer,
  .filters,
  .toolbar,
  .todo-row,
  li {
    display: flex;
    align-items: center;
  }

  .panel-header {
    justify-content: space-between;
    align-items: flex-start;
    gap: 20px;
    margin-bottom: 24px;
  }

  .header-title {
    display: flex;
    align-items: center;
    gap: 16px;
    flex-wrap: wrap;
  }

  .date-picker {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .date-picker input[type="date"] {
    padding: 4px 8px;
    border: 1px solid var(--border-input);
    border-radius: 8px;
    background: var(--bg-input);
    color: var(--text-main);
    font-family: inherit;
    height: 32px;
  }

  .icon-btn {
    min-height: 32px;
    width: 32px;
    padding: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 8px;
    background: var(--bg-button);
    color: var(--text-main-alt);
  }

  .today-btn {
    min-height: 32px;
    padding: 0 12px;
    font-size: 0.85rem;
    border-radius: 8px;
    background: var(--bg-primary-btn);
    color: white;
  }

  .panel-tip {
    margin: 12px 0 0;
    color: var(--text-muted);
  }

  .stats {
    display: grid;
    grid-template-columns: repeat(3, minmax(96px, 1fr));
    gap: 12px;
    min-width: 300px;
  }

  .stats article {
    padding: 14px;
    border-radius: 16px;
    text-align: center;
    background: var(--bg-panel-light);
  }

  .stats strong {
    display: block;
    font-size: 1.5rem;
    color: var(--text-main);
  }

  .stats span {
    font-size: 0.9rem;
    color: var(--text-muted);
  }

  .composer {
    gap: 12px;
    margin-bottom: 18px;
  }

  .text-input {
    flex: 1;
    min-height: 52px;
    padding: 0 16px;
    color: var(--text-main);
    background: var(--bg-input);
  }

  input,
  button {
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

  .filters,
  .toolbar {
    gap: 12px;
  }

  .filters {
    flex-wrap: wrap;
  }

  .toolbar {
    justify-content: space-between;
    margin-bottom: 18px;
  }

  .filters button,
  .secondary-button,
  .danger-button {
    min-height: 42px;
    background: var(--bg-button);
    color: var(--text-main-alt);
  }

  .filters button.active {
    background: var(--bg-active-btn);
    color: white;
  }

  .danger-button {
    color: #ef4444;
  }

  .todo-list {
    display: grid;
    gap: 12px;
    margin: 0;
    padding: 0;
    list-style: none;
  }

  li {
    justify-content: space-between;
    gap: 14px;
    padding: 16px 18px;
    border: 1px solid var(--border-light);
    border-radius: 18px;
    background: var(--bg-panel-light);
  }

  .todo-row {
    gap: 12px;
    flex: 1;
    min-width: 0;
    cursor: pointer;
  }

  .todo-row input {
    width: 18px;
    height: 18px;
    margin: 0;
  }

  .todo-row span {
    overflow-wrap: anywhere;
    color: var(--text-main);
  }

  li.done .todo-row span {
    color: var(--text-muted-alt);
    text-decoration: line-through;
  }

  .empty-state {
    padding: 36px 18px;
    border: 1px dashed var(--border-dashed);
    border-radius: 18px;
    text-align: center;
    color: var(--text-muted);
  }

  .error-tip {
    margin: 0 0 16px;
    padding: 12px 14px;
    border: 1px solid rgba(248, 113, 113, 0.35);
    border-radius: 14px;
    color: #fecaca;
    background: rgba(127, 29, 29, 0.2);
  }

  :global(body[data-theme="light"]) .error-tip {
    color: #ef4444;
    background: rgba(254, 226, 226, 0.5);
  }

  .toolbar-actions {
    display: flex;
    gap: 8px;
    flex-wrap: wrap;
  }

  .btn-sm {
    min-height: 32px;
    padding: 0 12px;
    font-size: 0.85rem;
    border-radius: 8px;
  }

  .archived-actions {
    display: flex;
    gap: 8px;
  }

  .date-badge {
    font-size: 0.75rem;
    padding: 2px 6px;
    border-radius: 4px;
    background: var(--bg-panel-light);
    color: var(--text-muted);
    border: 1px solid var(--border-light);
    white-space: nowrap;
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
    max-width: 600px;
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

  .modal-footer {
    display: flex;
    justify-content: flex-end;
    gap: 12px;
    padding: 16px 20px;
    border-top: 1px solid var(--border-light);
    background: var(--bg-panel-light);
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

  @media (max-width: 840px) {
    .panel-header,
    .composer,
    .toolbar,
    li {
      flex-direction: column;
      align-items: stretch;
    }

    .stats {
      min-width: 0;
      width: 100%;
    }

    .filters button,
    .secondary-button,
    .danger-button,
    button {
      width: 100%;
    }
  }
</style>
