<script lang="ts">
  import NotePanel from "$lib/components/NotePanel.svelte";
  import TodoPanel from "$lib/components/TodoPanel.svelte";
  import type { ModuleType } from "$lib/types/workbench";
  import { onMount } from "svelte";

  let activeModule = $state<ModuleType>("todo");
  let isDarkTheme = $state(true);
  let isHeaderCollapsed = $state(false);

  onMount(() => {
    // 从 localStorage 或系统偏好加载主题
    const savedTheme = localStorage.getItem("theme");
    if (savedTheme) {
      isDarkTheme = savedTheme === "dark";
    } else {
      isDarkTheme = window.matchMedia("(prefers-color-scheme: dark)").matches;
    }
    applyTheme();

    // 加载折叠状态
    const savedCollapse = localStorage.getItem("header-collapsed");
    if (savedCollapse) {
      isHeaderCollapsed = savedCollapse === "true";
    }

    // 加载上次打开的模块
    const savedModule = localStorage.getItem("active-module") as ModuleType;
    if (savedModule && (savedModule === "todo" || savedModule === "note")) {
      activeModule = savedModule;
    }
  });

  function switchModule(module: ModuleType) {
    activeModule = module;
    localStorage.setItem("active-module", module);
  }

  function toggleTheme() {
    isDarkTheme = !isDarkTheme;
    localStorage.setItem("theme", isDarkTheme ? "dark" : "light");
    applyTheme();
  }

  function applyTheme() {
    if (isDarkTheme) {
      document.body.setAttribute("data-theme", "dark");
    } else {
      document.body.setAttribute("data-theme", "light");
    }
  }

  function toggleHeader() {
    isHeaderCollapsed = !isHeaderCollapsed;
    localStorage.setItem("header-collapsed", isHeaderCollapsed.toString());
  }
</script>

<main class="page">
  <section class="workspace">
    <div class="hero" class:collapsed={isHeaderCollapsed}>
      <div class="hero-content">
        <div class="hero-top">
          <p class="eyebrow">Desk Tools</p>
          <div class="header-actions">
            <button type="button" class="icon-btn" onclick={toggleTheme} title="切换主题">
              {#if isDarkTheme}
                <svg viewBox="0 0 24 24" width="18" height="18" stroke="currentColor" stroke-width="2" fill="none" stroke-linecap="round" stroke-linejoin="round">
                  <circle cx="12" cy="12" r="5"></circle>
                  <line x1="12" y1="1" x2="12" y2="3"></line>
                  <line x1="12" y1="21" x2="12" y2="23"></line>
                  <line x1="4.22" y1="4.22" x2="5.64" y2="5.64"></line>
                  <line x1="18.36" y1="18.36" x2="19.78" y2="19.78"></line>
                  <line x1="1" y1="12" x2="3" y2="12"></line>
                  <line x1="21" y1="12" x2="23" y2="12"></line>
                  <line x1="4.22" y1="19.78" x2="5.64" y2="18.36"></line>
                  <line x1="18.36" y1="5.64" x2="19.78" y2="4.22"></line>
                </svg>
              {:else}
                <svg viewBox="0 0 24 24" width="18" height="18" stroke="currentColor" stroke-width="2" fill="none" stroke-linecap="round" stroke-linejoin="round">
                  <path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z"></path>
                </svg>
              {/if}
            </button>
            <button type="button" class="icon-btn" onclick={toggleHeader} title={isHeaderCollapsed ? "展开头部" : "折叠头部"}>
              {#if isHeaderCollapsed}
                <svg viewBox="0 0 24 24" width="18" height="18" stroke="currentColor" stroke-width="2" fill="none" stroke-linecap="round" stroke-linejoin="round">
                  <polyline points="6 9 12 15 18 9"></polyline>
                </svg>
              {:else}
                <svg viewBox="0 0 24 24" width="18" height="18" stroke="currentColor" stroke-width="2" fill="none" stroke-linecap="round" stroke-linejoin="round">
                  <polyline points="18 15 12 9 6 15"></polyline>
                </svg>
              {/if}
            </button>
          </div>
        </div>
        {#if !isHeaderCollapsed}
          <div class="hero-body">
            <h1>多功能桌面工具</h1>
            <p class="subtitle">
              当前已接入两个模块：待办清单和 Markdown 笔记，后续可以继续扩展提醒、日历、知识库等功能。
            </p>
          </div>
        {/if}
      </div>

      <div class="module-switcher">
        <button type="button" class:active={activeModule === "todo"} onclick={() => switchModule("todo")}>
          待办清单
        </button>
        <button type="button" class:active={activeModule === "note"} onclick={() => switchModule("note")}>
          Markdown 笔记
        </button>
      </div>
    </div>

    {#if activeModule === "todo"}
      <TodoPanel />
    {:else}
      <NotePanel />
    {/if}
  </section>
</main>

<style>
  :global(:root) {
    /* 默认暗色主题 */
    --bg-body: radial-gradient(circle at top, #243b67 0%, #101828 42%, #0b1120 100%);
    --bg-panel: rgba(15, 23, 42, 0.82);
    --bg-panel-light: rgba(15, 23, 42, 0.58);
    --bg-panel-lighter: rgba(15, 23, 42, 0.52);
    --bg-input: rgba(15, 23, 42, 0.9);
    --bg-header: rgba(15, 23, 42, 0.95);
    --bg-button: rgba(30, 41, 59, 0.92);
    --bg-modal: rgba(30, 41, 59, 0.95);
    --bg-pre: rgba(2, 6, 23, 0.88);
    
    --text-main: #e5eefc;
    --text-main-alt: #f8fafc;
    --text-muted: #b6c2d9;
    --text-muted-alt: #94a3b8;
    --text-accent: #93c5fd;
    
    --border-main: rgba(148, 163, 184, 0.18);
    --border-light: rgba(148, 163, 184, 0.16);
    --border-lighter: rgba(148, 163, 184, 0.12);
    --border-input: rgba(148, 163, 184, 0.2);
    --border-dashed: rgba(148, 163, 184, 0.25);
    --border-focus: rgba(191, 219, 254, 0.5);
    
    --bg-primary-btn: linear-gradient(135deg, #2563eb, #3b82f6);
    --bg-primary-btn-hover: linear-gradient(135deg, #3b82f6, #2563eb);
    --bg-active-btn: linear-gradient(135deg, #0f766e, #14b8a6);
    --shadow-primary: rgba(37, 99, 235, 0.2);
    --shadow-panel: rgba(15, 23, 42, 0.35);
  }

  :global(body[data-theme="light"]) {
    /* 亮色主题 */
    --bg-body: radial-gradient(circle at top, #f1f5f9 0%, #e2e8f0 42%, #cbd5e1 100%);
    --bg-panel: rgba(255, 255, 255, 0.85);
    --bg-panel-light: rgba(255, 255, 255, 0.65);
    --bg-panel-lighter: rgba(255, 255, 255, 0.5);
    --bg-input: rgba(255, 255, 255, 0.9);
    --bg-header: rgba(248, 250, 252, 0.95);
    --bg-button: rgba(241, 245, 249, 0.92);
    --bg-modal: rgba(255, 255, 255, 0.98);
    --bg-pre: rgba(241, 245, 249, 0.88);
    
    --text-main: #0f172a;
    --text-main-alt: #1e293b;
    --text-muted: #64748b;
    --text-muted-alt: #475569;
    --text-accent: #2563eb;
    
    --border-main: rgba(148, 163, 184, 0.3);
    --border-light: rgba(148, 163, 184, 0.25);
    --border-lighter: rgba(148, 163, 184, 0.2);
    --border-input: rgba(148, 163, 184, 0.4);
    --border-dashed: rgba(148, 163, 184, 0.5);
    --border-focus: rgba(37, 99, 235, 0.5);
    
    --bg-primary-btn: linear-gradient(135deg, #3b82f6, #2563eb);
    --bg-primary-btn-hover: linear-gradient(135deg, #60a5fa, #3b82f6);
    --bg-active-btn: linear-gradient(135deg, #14b8a6, #0d9488);
    --shadow-primary: rgba(37, 99, 235, 0.25);
    --shadow-panel: rgba(15, 23, 42, 0.08);
  }

  :global(body) {
    margin: 0;
    font-family: Inter, "Microsoft YaHei", "PingFang SC", sans-serif;
    color: var(--text-main);
    background: var(--bg-body);
    transition: background 0.3s ease, color 0.3s ease;
  }

  :global(*) {
    box-sizing: border-box;
  }

  .page {
    min-height: 100vh;
    padding: 32px 20px;
  }

  .workspace {
    width: calc(100vw - 40px);
    max-width: none;
    margin: 0 auto;
  }

  .hero {
    display: flex;
    flex-direction: column;
    justify-content: space-between;
    gap: 20px;
    align-items: flex-start;
    padding: 28px;
    margin-bottom: 20px;
    border: 1px solid var(--border-main);
    border-radius: 24px;
    background: var(--bg-panel);
    box-shadow: 0 24px 80px var(--shadow-panel);
    backdrop-filter: blur(14px);
    transition: all 0.3s ease;
    overflow: hidden;
  }

  .hero.collapsed {
    flex-direction: row;
    align-items: center;
    padding: 16px 28px;
  }

  .hero-content {
    width: 100%;
  }

  .hero.collapsed .hero-content {
    width: auto;
    display: flex;
    align-items: center;
  }

  .hero-top {
    display: flex;
    justify-content: space-between;
    align-items: center;
    width: 100%;
  }

  .hero.collapsed .hero-top {
    gap: 20px;
  }

  .header-actions {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .icon-btn {
    background: transparent;
    border: none;
    color: var(--text-muted);
    cursor: pointer;
    padding: 8px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 50%;
    transition: background 0.2s, color 0.2s;
    min-height: auto;
  }

  .icon-btn:hover {
    background: var(--bg-button);
    color: var(--text-main);
  }

  .hero-body {
    margin-top: 8px;
  }

  .eyebrow {
    margin: 0;
    font-size: 0.88rem;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--text-accent);
  }

  h1 {
    margin: 0;
    font-size: clamp(2rem, 5vw, 3rem);
    line-height: 1.1;
  }

  .subtitle {
    margin: 12px 0 0;
    color: var(--text-muted);
  }

  .module-switcher {
    display: flex;
    gap: 10px;
    flex-wrap: wrap;
    align-items: center;
  }

  .hero.collapsed .module-switcher {
    margin-left: auto;
  }

  button {
    min-height: 42px;
    padding: 0 18px;
    border: 1px solid var(--border-input);
    border-radius: 14px;
    font: inherit;
    font-weight: 600;
    color: var(--text-main-alt);
    background: var(--bg-button);
    cursor: pointer;
    transition:
      transform 0.15s ease,
      border-color 0.15s ease,
      opacity 0.15s ease;
  }

  button:hover:enabled {
    transform: translateY(-1px);
    border-color: var(--border-focus);
  }

  .module-switcher button.active {
    background: var(--bg-active-btn);
    color: white;
  }

  @media (max-width: 840px) {
    .hero {
      flex-direction: column;
      align-items: stretch;
    }

    .module-switcher,
    .module-switcher button {
      width: 100%;
    }
  }
</style>
